use catplay_util::{AbortOnDropHandle, spawn, spawn_blocking};
use dbus::arg::RefArg;
use dbus::channel::MatchingReceiver;
use dbus::message::MatchRule;
use dbus::nonblock::{Proxy, SyncConnection};
use dbus_crossroads::Crossroads;
use log::{debug, info};
use macaddr::MacAddr6;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::Duration,
};

use crate::{
    BluezIapConnectionHandler, BluezPinAgent, BluezPinAgentStore, IAP_CHANNEL, IAP_CLIENT_UUID, IAP_SERVER_UUID, IAP_V2_UUID,
    IapConnection, IapProfile, PairingRequest, generate_iap2_sdp, register_iap_profile_interface, register_pin_agent_interface,
};

pub type BluezValue = dbus::arg::Variant<Box<dyn RefArg + 'static>>;

struct DbusConnection {
    conn: Arc<SyncConnection>,
    resource_task: AbortOnDropHandle<()>,
}

impl DbusConnection {
    fn _new() -> BluezResult<Self> {
        let (resource, conn) = dbus_tokio::connection::new_system_sync()?;
        let resource_task = spawn(async move {
            let _ = resource.await;
        });

        Ok(Self { conn, resource_task })
    }

    async fn new_async() -> BluezResult<Self> {
        let (resource, conn) = spawn_blocking(dbus_tokio::connection::new_system_sync)
            .await
            .map_err(|err| BluezError::Task(err.to_string()))??;
        let resource_task = spawn(async move {
            let _ = resource.await;
        });

        Ok(Self { conn, resource_task })
    }
}

pub struct BluezManager {
    agent_conn: Option<DbusConnection>,
    agent_store: Option<BluezPinAgentStore>,
    agent_registred: Arc<AtomicU32>,

    iap2_conn: Option<DbusConnection>,
    iap2_registered: Arc<AtomicU32>,
}

#[derive(thiserror::Error, Debug)]
pub enum BluezError {
    #[error("BlueZ DBus error: {0}")]
    Dbus(#[from] dbus::Error),
    #[error("BlueZ DBus path error: {0}")]
    Path(String),
    #[error("BlueZ DBus value error: {0}")]
    Value(String),
    #[error("BlueZ async task error: {0}")]
    Task(String),
}
pub type BluezResult<T> = Result<T, BluezError>;

pub(crate) fn bluez_parse_path(path: &str) -> Option<(String, String)> {
    // "/org/bluez/hci0/dev_AA:BB:CC:DD:EE:FF"
    let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
    if parts.len() < 3 {
        return None;
    }

    let iface = parts[2]; // "hci0"
    let dev = parts.get(3)?;

    if !dev.starts_with("dev_") {
        return None;
    }
    let raw_mac = &dev[4..];
    let mac = raw_mac.replace('_', ":").to_lowercase();

    Some((iface.to_string(), mac))
}

static TOKENS: AtomicU32 = AtomicU32::new(0);

impl Default for BluezManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BluezManager {
    pub fn new() -> Self {
        Self {
            agent_conn: None,
            agent_store: None,
            agent_registred: Arc::new(AtomicU32::new(0)),

            iap2_conn: None,
            iap2_registered: Arc::new(AtomicU32::new(0)),
        }
    }

    fn variant<T: RefArg + 'static>(value: T) -> BluezValue {
        dbus::arg::Variant(Box::new(value))
    }

    async fn open_client_connection() -> BluezResult<DbusConnection> {
        DbusConnection::new_async().await
    }

    /// Returns false if iAP2 was not registered or was released by BlueZ since.
    pub fn is_iap2_active(&self) -> bool {
        self.iap2_registered.load(Ordering::Acquire) != 0
    }

    /// Returns false if pin agent is not registered or was released by BlueZ since.
    pub fn is_pin_agent_active(&self) -> bool {
        self.agent_registred.load(Ordering::Acquire) != 0
    }

    /// Gets a copy of pending pairing request (if pin agent is active).
    ///
    /// Next calls will continue to return a copy of the same request, unless it's accepted or cancelled.
    pub fn get_pairing_request(&self) -> Option<PairingRequest> {
        let agent_store = self.agent_store.as_ref()?;
        agent_store.get_pending()
    }

    pub async fn register_iap2<F: Fn(IapConnection) + Send + Sync + 'static>(
        &mut self,
        // true if iPhone, false if Accessory
        server: bool,
        on_conn: F,
    ) -> BluezResult<()> {
        let on_conn: BluezIapConnectionHandler = Arc::new(on_conn);

        let server_path = "/org/bluez/iap_server";
        let client_path: &'static str = "/org/bluez/iap_client";

        let iap2_registered = self.iap2_registered.clone();
        let iap2_registered0 = iap2_registered.clone();

        let token = TOKENS.fetch_add(1, Ordering::Release) + 1; // at least 1

        let server_profile = IapProfile::new(false, server_path, on_conn.clone(), move || {
            debug!("Server profile released");
            let _ = iap2_registered.compare_exchange(token, 0, Ordering::Acquire, Ordering::Acquire);
        });

        let client_profile = IapProfile::new(true, client_path, on_conn, move || {
            debug!("Client profile released");
            let _ = iap2_registered0.compare_exchange(token, 0, Ordering::Acquire, Ordering::Acquire);
        });

        self.iap2_conn = None;
        self.iap2_registered.store(token, Ordering::Release);

        let conn = DbusConnection::new_async()
            .await
            .inspect_err(|_| self.unregister_iap2())?;

        let mut cr = Crossroads::new();
        register_iap_profile_interface(&mut cr, server_path, server_profile);
        register_iap_profile_interface(&mut cr, client_path, client_profile);

        conn.conn.start_receive(
            MatchRule::new_method_call(),
            Box::new(move |msg, c| {
                if let Err(err) = cr.handle_message(msg, c) {
                    debug!("Failed to handle Profile1 message: {err:?}");
                }
                true
            }),
        );

        let proxy = Proxy::new("org.bluez", "/org/bluez", Duration::from_secs(10), conn.conn.clone());

        let channel = IAP_CHANNEL;

        let sdp = generate_iap2_sdp(IAP_SERVER_UUID.to_string(), channel);
        debug!("Using iAP2 UUIDs: server={}, client={}", IAP_SERVER_UUID, IAP_CLIENT_UUID);
        debug!("Using iAP2 SDP: {}", sdp);

        let server_opts: HashMap<String, BluezValue> = HashMap::from([
            ("Role".into(), Self::variant("server".to_string())),
            ("Channel".into(), Self::variant(channel)),
            ("ServiceRecord".into(), Self::variant(sdp.clone())),
            ("RequireAuthentication".into(), Self::variant(true)),
            ("RequireAuthorization".into(), Self::variant(false)),
        ]);

        let (mut server_uuid, mut client_uuid) = (IAP_SERVER_UUID, IAP_CLIENT_UUID);
        if server {
            (server_uuid, client_uuid) = (client_uuid, server_uuid)
        }

        let server_path_obj = dbus::Path::new(server_path)
            .map_err(|e| BluezError::Path(e.to_string()))
            .inspect_err(|_| self.unregister_iap2())?;
        let client_path_obj = dbus::Path::new(client_path)
            .map_err(|e| BluezError::Path(e.to_string()))
            .inspect_err(|_| self.unregister_iap2())?;

        let register_server: Result<(), dbus::Error> = proxy
            .method_call(
                "org.bluez.ProfileManager1",
                "RegisterProfile",
                (server_path_obj, server_uuid.to_string(), server_opts),
            )
            .await;
        register_server.inspect_err(|_| self.unregister_iap2())?;

        // Client with AutoConnect
        let client_opts: HashMap<String, BluezValue> = HashMap::from([
            ("Role".into(), Self::variant("client".to_string())),
            ("AutoConnect".into(), Self::variant(true)),
        ]);

        let register_client: Result<(), dbus::Error> = proxy
            .method_call(
                "org.bluez.ProfileManager1",
                "RegisterProfile",
                (client_path_obj, client_uuid.to_string(), client_opts),
            )
            .await;
        register_client.inspect_err(|_| self.unregister_iap2())?;

        if server {
            let marker_path: &'static str = "/org/bluez/iap_v2";
            let marker_opts: HashMap<String, BluezValue> = HashMap::from([
                ("Role".into(), Self::variant("server".to_string())),
                ("Channel".into(), Self::variant(channel)),
                ("ServiceRecord".into(), Self::variant(sdp.clone())),
                ("RequireAuthentication".into(), Self::variant(true)),
                ("RequireAuthorization".into(), Self::variant(false)),
                ("Name".into(), Self::variant("Wireless iAP v2".to_string())),
            ]);

            let marker_path_obj = dbus::Path::new(marker_path)
                .map_err(|e| BluezError::Path(e.to_string()))
                .inspect_err(|_| self.unregister_iap2())?;
            let register_marker: Result<(), dbus::Error> = proxy
                .method_call(
                    "org.bluez.ProfileManager1",
                    "RegisterProfile",
                    (marker_path_obj, IAP_V2_UUID.to_string(), marker_opts),
                )
                .await;
            register_marker.inspect_err(|_| self.unregister_iap2())?;
        }

        self.iap2_conn = Some(conn);

        info!("Registered iAP2 protocol");
        Ok(())
    }

    pub fn unregister_iap2(&mut self) {
        debug!("Unregistering iAP2 protocol");
        self.iap2_conn = None; // Just drop the D-BUS conn
        self.iap2_registered.store(0, Ordering::Release);
    }

    /// Pings device to connect to iAP2 profile (as an accessory pinging an iPhone)
    pub async fn iap2_connect(&self, adapter: &str, addr: &str) -> BluezResult<()> {
        let conn = Self::open_client_connection().await?;

        let device_path = format!("/org/bluez/{}/dev_{}", adapter, addr.replace(':', "_"));
        let proxy = Proxy::new("org.bluez", device_path, Duration::from_secs(10), conn.conn.clone());

        let connect_profile: Result<(), dbus::Error> = proxy
            .method_call("org.bluez.Device1", "ConnectProfile", (IAP_CLIENT_UUID.to_string(),))
            .await;

        connect_profile?;

        Ok(())
    }

    pub async fn register_pin_agent(&mut self) -> BluezResult<()> {
        self.agent_conn = None;
        self.agent_store = None;

        let token = TOKENS.fetch_add(1, Ordering::Release) + 1;

        let pin_agent_registred = self.agent_registred.clone();
        let (agent, agent_store) = BluezPinAgent::new(move || {
            let _ = pin_agent_registred.compare_exchange(token, 0, Ordering::Acquire, Ordering::Acquire);
        });

        let agent_path = "/org/bluez/Agent";
        self.agent_registred.store(token, Ordering::Release);

        let conn = DbusConnection::new_async()
            .await
            .inspect_err(|_| self.unregister_pin_agent())?;

        let mut cr = Crossroads::new();
        cr.set_async_support(Some((
            conn.conn.clone(),
            Box::new(|fut| {
                tokio::spawn(fut);
            }),
        )));
        register_pin_agent_interface(&mut cr, agent_path, agent);

        conn.conn.start_receive(
            MatchRule::new_method_call(),
            Box::new(move |msg, c| {
                if let Err(err) = cr.handle_message(msg, c) {
                    debug!("Failed to handle Agent1 message: {err:?}");
                }
                true
            }),
        );

        // Register BlueZ agent
        let proxy = Proxy::new("org.bluez", "/org/bluez", Duration::from_secs(10), conn.conn.clone());

        let agent_path_obj = dbus::Path::new(agent_path)
            .map_err(|e| BluezError::Path(e.to_string()))
            .inspect_err(|_| self.unregister_pin_agent())?;

        let register_agent: Result<(), dbus::Error> = proxy
            .method_call(
                "org.bluez.AgentManager1",
                "RegisterAgent",
                (agent_path_obj.clone(), "KeyboardDisplay".to_string()),
            )
            .await;
        register_agent.inspect_err(|_| self.unregister_pin_agent())?;

        let default_agent: Result<(), dbus::Error> = proxy
            .method_call("org.bluez.AgentManager1", "RequestDefaultAgent", (agent_path_obj,))
            .await;
        default_agent.inspect_err(|_| self.unregister_pin_agent())?;

        self.agent_store = Some(agent_store);
        self.agent_conn = Some(conn);

        info!("Registered PIN agent");
        Ok(())
    }

    pub fn unregister_pin_agent(&mut self) {
        info!("Unregistered PIN agent");
        self.agent_conn = None; // Just drop the D-BUS conn
        self.agent_store = None;
        self.agent_registred.store(0, Ordering::Release);
    }

    pub async fn set_adapter_prop(&self, adapter: &str, key: &str, val: BluezValue) -> BluezResult<()> {
        let conn = Self::open_client_connection().await?;

        let proxy = Proxy::new(
            "org.bluez",
            "/org/bluez/".to_string() + adapter,
            Duration::from_secs(10),
            conn.conn.clone(),
        );

        let set_prop: Result<(), dbus::Error> = proxy
            .method_call(
                "org.freedesktop.DBus.Properties",
                "Set",
                ("org.bluez.Adapter1", key.to_string(), val),
            )
            .await;

        set_prop?;

        debug!("Set adapter prop key={}", key);
        Ok(())
    }

    pub async fn get_adapter_prop(&self, adapter: &str, key: &str) -> BluezResult<BluezValue> {
        let conn = Self::open_client_connection().await?;

        let proxy = Proxy::new(
            "org.bluez",
            "/org/bluez/".to_string() + adapter,
            Duration::from_secs(10),
            conn.conn.clone(),
        );

        let value: Result<(BluezValue,), dbus::Error> = proxy
            .method_call("org.freedesktop.DBus.Properties", "Get", ("org.bluez.Adapter1", key.to_string()))
            .await;

        let (value,) = value?;

        Ok(value)
    }

    pub async fn get_address(&self, adapter: &str) -> BluezResult<String> {
        let val = self.get_adapter_prop(adapter, "Address").await?;
        let s = val
            .0
            .as_str()
            .ok_or_else(|| BluezError::Value("Address is not a string".to_string()))?;

        Ok(s.into())
    }

    pub async fn set_powered(&self, adapter: &str, powered: bool) -> BluezResult<()> {
        self.set_adapter_prop(adapter, "Powered", Self::variant(powered))
            .await
    }

    pub async fn set_pairable(&self, adapter: &str, pairable: bool) -> BluezResult<()> {
        self.set_adapter_prop(adapter, "Pairable", Self::variant(pairable))
            .await
    }

    pub async fn set_discoverable(&self, adapter: &str, discoverable: bool) -> BluezResult<()> {
        self.set_adapter_prop(adapter, "Discoverable", Self::variant(discoverable))
            .await?;
        self.set_adapter_prop(adapter, "DiscoverableTimeout", Self::variant(0u32))
            .await
    }

    pub async fn set_alias(&self, adapter: &str, alias: &str) -> BluezResult<()> {
        self.set_adapter_prop(adapter, "Alias", Self::variant(alias.to_string()))
            .await
    }

    pub async fn disconnect_peer(&self, adapter: &str, addr: MacAddr6) -> BluezResult<()> {
        let conn = Self::open_client_connection().await?;

        let device_path = format!("/org/bluez/{}/dev_{}", adapter, addr.to_string().replace(':', "_"));
        let proxy = Proxy::new("org.bluez", device_path, Duration::from_secs(10), conn.conn.clone());

        let disconnect: Result<(), dbus::Error> = proxy
            .method_call("org.bluez.Device1", "Disconnect", ())
            .await;

        disconnect?;

        Ok(())
    }
}
