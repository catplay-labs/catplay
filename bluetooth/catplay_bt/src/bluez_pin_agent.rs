use std::{
    marker::PhantomData,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use dbus::MethodErr;
use dbus::arg::{RefArg, Variant};
use dbus::nonblock::Proxy;
use dbus_crossroads::Crossroads;
use log::{debug, warn};
use tokio::sync::oneshot;
use tokio::time;

use crate::bluez_parse_path;

type ArcMutexOption<T> = Arc<Mutex<Option<T>>>;
type ArcMutex<T> = Arc<Mutex<T>>;

#[derive(Clone, Debug)]
pub struct PairingRequest {
    pub hci: String,
    pub remote_mac: String,
    pub remote_name: Option<String>,
    pub passkey: u32,
    responder: ArcMutexOption<oneshot::Sender<bool>>,
}

impl PairingRequest {
    pub fn accept(self) {
        self.resolve(true)
    }

    pub fn reject(self) {
        self.resolve(false)
    }

    pub fn is_resolved(&self) -> bool {
        let responder = self.responder.lock().unwrap();
        responder.is_none()
    }

    pub fn resolve(self, accept: bool) {
        debug!("ResolvePending: {} {} {}", &self.hci, &self.remote_mac, accept);

        let mut responder = self.responder.lock().unwrap();
        let Some(responder): Option<oneshot::Sender<bool>> = responder.take() else {
            return;
        };

        let _ = responder.send(accept);
    }
}

#[derive(Clone)]
pub struct BluezPinAgentStore {
    pending: ArcMutexOption<PairingRequest>,
}

impl BluezPinAgentStore {
    /// Gets a copy of pending pairing request.
    ///
    /// Next calls will continue to return a copy of the same request, unless it's accepted or cancelled.
    pub fn get_pending(&self) -> Option<PairingRequest> {
        self.pending
            .lock()
            .unwrap()
            .as_ref()
            .cloned()
            .take_if(|p| !p.is_resolved())
    }

    fn store(&self, req: PairingRequest) {
        let mut pending = self.pending.lock().unwrap();
        *pending = Some(req);
    }
}

pub struct BluezPinAgent {
    release_callback: ArcMutex<dyn FnMut() + Send + 'static>,
    pending: BluezPinAgentStore,
    released: Arc<AtomicBool>,
}

impl BluezPinAgent {
    fn variant<T: RefArg + 'static>(value: T) -> Variant<Box<dyn RefArg + 'static>> {
        Variant(Box::new(value))
    }

    fn get_remote_name(device: &str) -> Option<String> {
        let conn = dbus::blocking::Connection::new_system().ok()?;
        let proxy = conn.with_proxy("org.bluez", device, Duration::from_millis(800));

        let get_prop = |key: &str| -> Option<String> {
            let value: Result<(dbus::arg::Variant<Box<dyn dbus::arg::RefArg + 'static>>,), dbus::Error> =
                proxy.method_call("org.freedesktop.DBus.Properties", "Get", ("org.bluez.Device1", key.to_string()));
            let (value,) = value.ok()?;
            value.0.as_str().map(ToString::to_string)
        };

        get_prop("Alias").or_else(|| get_prop("Name"))
    }

    fn spawn_trust_after_pairing(device: String) {
        // iOS can reject connections when busy, and without Trusted=true, BlueZ will permanently wipe link keys
        // on such rejected connection
        tokio::spawn(async move {
            if let Err(err) = Self::trust_after_pairing(device.clone()).await {
                warn!("Failed to mark paired device as trusted {device}: {err}");
            }
        });
    }

    async fn trust_after_pairing(device: String) -> Result<(), dbus::Error> {
        let (resource, conn) = dbus_tokio::connection::new_system_sync()?;
        let resource_task = tokio::spawn(async move {
            let _ = resource.await;
        });

        let proxy = Proxy::new("org.bluez", device.clone(), Duration::from_secs(2), conn);

        for attempt in 0..40 {
            if Self::device_is_paired_or_bonded(&proxy).await {
                let set_trusted: Result<(), dbus::Error> = proxy
                    .method_call(
                        "org.freedesktop.DBus.Properties",
                        "Set",
                        ("org.bluez.Device1", "Trusted".to_string(), Self::variant(true)),
                    )
                    .await;

                resource_task.abort();
                set_trusted?;
                debug!("Marked paired device as trusted: {device}");
                return Ok(());
            }

            if attempt == 0 {
                debug!("Waiting for BlueZ pairing state before trusting {device}");
            }
            time::sleep(Duration::from_millis(250)).await;
        }

        resource_task.abort();
        warn!("Timed out waiting for BlueZ pairing state before trusting {device}");
        Ok(())
    }

    async fn device_is_paired_or_bonded(proxy: &Proxy<'_, Arc<dbus::nonblock::SyncConnection>>) -> bool {
        Self::get_device_bool(proxy, "Bonded")
            .await
            .unwrap_or(false)
            || Self::get_device_bool(proxy, "Paired")
                .await
                .unwrap_or(false)
    }

    async fn get_device_bool(proxy: &Proxy<'_, Arc<dbus::nonblock::SyncConnection>>, key: &str) -> Result<bool, dbus::Error> {
        let (value,): (Variant<Box<dyn RefArg + 'static>>,) = proxy
            .method_call("org.freedesktop.DBus.Properties", "Get", ("org.bluez.Device1", key.to_string()))
            .await?;

        Ok(value.0.as_i64().unwrap_or(0) != 0)
    }

    pub fn new(release_callback: impl FnMut() + Send + 'static) -> (Self, BluezPinAgentStore) {
        let pending = Arc::new(Mutex::new(None));
        let agent = BluezPinAgentStore { pending };
        let me = Self {
            release_callback: Arc::new(Mutex::new(release_callback)),
            pending: agent.clone(),
            released: Arc::new(AtomicBool::new(false)),
        };
        (me, agent)
    }

    pub fn is_released(&self) -> bool {
        self.released.load(Ordering::Acquire)
    }

    fn authorize_service(&self, device: &str, uuid: &str) {
        debug!("AuthorizeService {} {}", device, uuid)
    }

    fn request_pin_code(&self, device: &str) -> Result<String, MethodErr> {
        debug!("RequestPinCode for device: {}", device);
        Err(MethodErr::failed("legacy pairing requested"))
    }

    fn request_passkey(&self, device: &str) -> Result<u32, MethodErr> {
        debug!("RequestPasskey for device: {}", device);
        Err(MethodErr::failed("legacy pairing requested"))
    }

    fn display_passkey(&self, device: &str, passkey: u32) -> Result<(), MethodErr> {
        debug!("DisplayPasskey for device {}: {}", device, passkey);
        Err(MethodErr::failed("legacy pairing requested"))
    }

    fn display_pin_code(&self, device: &str, pincode: &str) -> Result<(), MethodErr> {
        debug!("DisplayPinCode for device {}: {}", device, pincode);
        Err(MethodErr::failed("legacy pairing requested"))
    }

    fn begin_request_confirmation(&self, device: &str, passkey: u32) -> Result<oneshot::Receiver<bool>, MethodErr> {
        debug!("RequestConfirmation for device {}: {}", device, passkey);
        let (tx, rx) = oneshot::channel();

        let remote = bluez_parse_path(device);
        let Some((hci, remote_mac)) = remote else {
            debug!("invalid path: {}", device);
            return Err(MethodErr::failed("unparsable remote"));
        };

        let req = PairingRequest {
            hci,
            remote_mac,
            remote_name: Self::get_remote_name(device),
            passkey,
            responder: Arc::new(Mutex::new(Some(tx))),
        };
        self.pending.store(req);

        Ok(rx)
    }

    fn request_authorization(&self, device: &str) {
        debug!("RequestAuthorization for device {}", device);
    }

    fn cancel(&self) {
        debug!("Cancel received");
        let Some(req) = self.pending.get_pending() else {
            return;
        };
        req.reject();
    }

    fn release(&self) {
        debug!("Release received");
        self.released.store(true, Ordering::Release);
        (self.release_callback.lock().unwrap())();
    }
}

pub(crate) fn register_pin_agent_interface(cr: &mut Crossroads, path: &'static str, agent: BluezPinAgent) {
    let iface = cr.register("org.bluez.Agent1", |b| {
        b.method(
            "AuthorizeService",
            ("device", "uuid"),
            (),
            |_, agent: &mut BluezPinAgent, (device, uuid): (dbus::Path<'static>, String)| {
                let device = device.to_string();
                agent.authorize_service(&device, &uuid);
                Ok(())
            },
        );

        b.method(
            "RequestPinCode",
            ("device",),
            ("pin_code",),
            |_, agent: &mut BluezPinAgent, (device,): (dbus::Path<'static>,)| {
                let pin_code = agent.request_pin_code(&device.to_string())?;
                Ok((pin_code,))
            },
        );

        b.method(
            "RequestPasskey",
            ("device",),
            ("passkey",),
            |_, agent: &mut BluezPinAgent, (device,): (dbus::Path<'static>,)| {
                let passkey = agent.request_passkey(&device.to_string())?;
                Ok((passkey,))
            },
        );

        b.method(
            "DisplayPasskey",
            ("device", "passkey"),
            (),
            |_, agent: &mut BluezPinAgent, (device, passkey): (dbus::Path<'static>, u32)| {
                agent.display_passkey(&device.to_string(), passkey)?;
                Ok(())
            },
        );

        b.method(
            "DisplayPinCode",
            ("device", "pin_code"),
            (),
            |_, agent: &mut BluezPinAgent, (device, pin_code): (dbus::Path<'static>, String)| {
                agent.display_pin_code(&device.to_string(), &pin_code)?;
                Ok(())
            },
        );

        b.method_with_cr_async(
            "RequestConfirmation",
            ("device", "passkey"),
            (),
            |mut ctx, cr, (device, passkey): (dbus::Path<'static>, u32)| {
                let device_s = device.to_string();
                let rx = match cr.data_mut::<BluezPinAgent>(ctx.path()) {
                    Some(agent) => agent.begin_request_confirmation(&device_s, passkey),
                    None => Err(MethodErr::no_path(ctx.path())),
                };

                async move {
                    let result = match rx {
                        Ok(rx) => match rx.await {
                            Ok(true) => {
                                BluezPinAgent::spawn_trust_after_pairing(device_s);
                                Ok(())
                            }
                            Ok(false) => Err(MethodErr::failed("User rejected")),
                            Err(_) => Err(MethodErr::failed("agent dropped")),
                        },
                        Err(err) => Err(err),
                    };

                    ctx.reply(result);
                    PhantomData::<()>
                }
            },
        );

        b.method(
            "RequestAuthorization",
            ("device",),
            (),
            |_, agent: &mut BluezPinAgent, (device,): (dbus::Path<'static>,)| {
                agent.request_authorization(&device.to_string());
                Ok(())
            },
        );

        b.method("Cancel", (), (), |_, agent: &mut BluezPinAgent, _: ()| {
            agent.cancel();
            Ok(())
        });

        b.method("Release", (), (), |_, agent: &mut BluezPinAgent, _: ()| {
            agent.release();
            Ok(())
        });
    });

    cr.insert(path, &[iface], agent);
}
