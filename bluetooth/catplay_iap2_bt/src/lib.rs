use std::{io, str::FromStr};
use std::{net::TcpStream, sync::Mutex};
use std::{sync::Arc, time::Duration};

use catplay_bt::{BluezError, BluezManager, CARPLAY_EIR_ACCESSORY, CARPLAY_EIR_PHONE, CARPLAY_HCI_CLASS_ACCESSORY_MGMT_BYTES, HciSocket};
use catplay_iap2_client::{
    CsmRemote, CsmSession, CsmSessionCallback,
    tokio::{AsyncClient, AsyncClientStream},
};
use catplay_util::{EventReconciler, event_select, sleep};
use log::{error, info, warn};
use macaddr::MacAddr6;

#[derive(thiserror::Error, Debug)]
pub enum BluetoothError {
    #[error("Invalid adapter name: {0}")]
    InvalidAdapter(String),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("BlueZ error: {0}")]
    Bluez(#[from] BluezError),
}

pub type BluetoothResult<T> = Result<T, BluetoothError>;

pub struct BluetoothManager {
    hcihacks: bool,
    carplay: bool,
    server: bool,

    enabled: bool,
    mac_addr: Option<String>,
    adapter: String,
    bluez: Option<BluezManager>,
    csm: CsmSessionCallback,

    last_connect: Arc<Mutex<Option<MacAddr6>>>,
}

impl BluetoothManager {
    pub fn new<T: CsmSession, F: Fn() -> T + Send + Sync + 'static>(
        server: bool,
        hcihacks: bool,
        carplay: bool,
        adapter: &str,
        csm: F,
    ) -> Self {
        let csm: CsmSessionCallback = Arc::new(move || Box::new(csm()));

        Self {
            server,
            hcihacks,
            carplay,

            enabled: false,
            mac_addr: None,
            adapter: adapter.into(),
            bluez: None,
            csm,
            last_connect: Default::default(),
        }
    }

    /// Sets Powered = true on Bluetooth adapter.
    pub async fn start_power(adapter: &str, alias: &str) -> BluetoothResult<()> {
        let bluez = BluezManager::new();
        bluez.set_alias(adapter, alias).await?;
        bluez.set_powered(adapter, true).await?;
        Ok(())
    }

    /// Invites trusted iPhone to connect to iAP2 profile.
    pub async fn invite_iphone(adapter: &str, iphone_bt_mac: &MacAddr6) -> BluetoothResult<()> {
        let bluez = BluezManager::new();
        bluez
            .iap2_connect(adapter, &iphone_bt_mac.to_string())
            .await?;
        Ok(())
    }

    /// Query local MAC address of Bluetooth device.
    ///
    /// Early in the boot process this function may fail, and can be externally retried again in intervals.
    pub async fn query_mac(adapter: &str) -> BluetoothResult<MacAddr6> {
        let bluez = BluezManager::new();
        let addr = bluez.get_address(adapter).await?;
        let mac = MacAddr6::from_str(&addr).map_err(|_| io::Error::other("invalid MAC address"))?;
        Ok(mac)
    }

    /// Starts Bluetooth iAP2 server - requires BlueZ to be running and given HCI device to be active.
    ///
    /// Early in the boot process this function may fail, and can be externally retried again in intervals.
    pub async fn start(&mut self) -> BluetoothResult<()> {
        let bluez = BluezManager::new();
        let adapter = &self.adapter;

        fn parse_hci_number(iface: &str) -> Option<u16> {
            let hci = iface.strip_prefix("hci")?;
            hci.parse::<u16>().ok()
        }

        let adapter_index = match parse_hci_number(&self.adapter) {
            Some(num) => num,
            None => {
                return Err(BluetoothError::InvalidAdapter(self.adapter.clone()));
            }
        };

        info!("Querying Bluez for adapters...");
        let addr = bluez.get_address(adapter).await?;
        info!("Found BT addr = {} at adapter {}", addr, adapter);

        self.mac_addr.replace(addr);
        self.enabled = true;
        self.bluez.replace(bluez);

        // TODO: detect BlueZ restart and re-register
        self.start_iap2().await?;

        const HCIHACK_ATTEMPTS: usize = 40;
        const HCIHACKS_DELAY: Duration = Duration::from_millis(50);

        for attempt in 1..=HCIHACK_ATTEMPTS {
            // Sometimes we get MGMT status "Busy" while bluetoothd is still settling.
            match self.do_hcihacks(adapter_index).await {
                Ok(()) => {
                    if self.hcihacks && self.carplay {
                        if attempt == 1 {
                            info!("Applied CarPlay HCI tweaks");
                        } else {
                            info!("Applied CarPlay HCI tweaks after {} attempts", attempt);
                        }
                    }
                    return Ok(());
                }
                Err(_) if attempt < HCIHACK_ATTEMPTS => {
                    sleep(HCIHACKS_DELAY).await;
                }
                Err(_err) => {
                    return Ok(());
                    // return Err(err.into());
                }
            }
        }

        Ok(())
    }

    async fn do_hcihacks(&self, adapter_index: u16) -> io::Result<()> {
        let hci = HciSocket::new(adapter_index)?;

        if self.hcihacks && self.carplay && !self.server {
            hci.add_uuid(CARPLAY_EIR_ACCESSORY)
                .map_err(|err| io::Error::new(err.kind(), format!("failed to register CarPlay BT Accessory EIR: {}", err)))?;
        }
        if self.hcihacks && self.carplay && self.server {
            hci.add_uuid(CARPLAY_EIR_PHONE)
                .map_err(|err| io::Error::new(err.kind(), format!("failed to register CarPlay BT iPhone EIR: {}", err)))?;
        }

        if self.hcihacks && self.carplay && !self.server {
            hci.set_class(CARPLAY_HCI_CLASS_ACCESSORY_MGMT_BYTES)
                .map_err(|err| io::Error::new(err.kind(), format!("failed to update HCI class to match CarPlay receiver: {}", err)))?;
        }

        Ok(())
    }

    pub fn get_last_connected(&mut self) -> Option<MacAddr6> {
        *self.last_connect.lock().unwrap()
    }

    async fn start_iap2(&mut self) -> BluetoothResult<()> {
        let handle = tokio::runtime::Handle::current();
        let csm = self.csm.clone();
        let server = self.server;

        let last_connect = self.last_connect.clone();
        self.bluez
            .as_mut()
            .unwrap()
            .register_iap2(server, move |conn| {
                info!("New Bluetooth iAP2 conn from peer: {}", conn.peer);

                last_connect.lock().unwrap().replace(conn.peer);
                let socket = tokio::net::TcpStream::from_std(TcpStream::from(conn.socket));
                let session = (csm)();

                match socket {
                    Err(err) => {
                        error!("Failed to wrap Bluetooth socket: {}", err);
                    }
                    Ok(socket) => {
                        handle.spawn(async move {
                            let task = AsyncClient::new(server, CsmRemote::bluetooth(conn.peer, conn.local), session);
                            let mut t = AsyncClientStream::new(task.0, task.1, socket);

                            loop {
                                if let Err(err) = t.reconcile().await {
                                    warn!("Reached final/error state for Bluetooth iAP2 session with {}: {}", conn.peer, err);
                                    return;
                                }

                                event_select!(t);
                            }
                        });
                    }
                }
            })
            .await?;

        info!("Started iAP2 Bluetooth server");
        Ok(())
    }
}
