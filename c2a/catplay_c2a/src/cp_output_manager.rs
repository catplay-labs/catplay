use std::{error::Error, path::PathBuf};

use log::info;

use crate::{AppConfig, HomeKitManager, MfiManager, ProdGadget, ProdGadgetConfig};
use catplay_util::{EventReconciler, EventSleeper, Reconciler};

#[derive(EventSleeper, EventReconciler)]
#[reconcile_error[()]]
pub struct CarPlayOutputManager {
    enabled: bool,
    #[sleep]
    #[reconcile]
    gadget_manager: Option<Reconciler<ProdGadget>>,
}

impl CarPlayOutputManager {
    pub fn new() -> Self {
        Self {
            enabled: false,
            gadget_manager: None,
        }
    }

    pub fn start(&mut self, config: &AppConfig, _mfi: &MfiManager, homekit: &HomeKitManager) -> Result<(), Box<dyn Error>> {
        let gadget = &config.gadget;
        if !gadget.enabled {
            info!("Gadget manager is disabled");
            return Ok(());
        }

        info!("Creating GadgetManager output!!!");

        let config = config.clone();
        let persist_dir = config.persist_dir.map(PathBuf::from);
        let bt_cache_file = persist_dir
            .as_ref()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "persist_dir is required for bluetooth last-connect cache",
                )
            })?
            .join("catplay_last_connect.txt");
        let cfg = ProdGadgetConfig {
            homekit_rx: homekit.homekit_rx.clone().unwrap(),
            homekit_tx: homekit.homekit_tx.clone().unwrap(),
            mfi: Some(_mfi.get_device()),
            udc_tx: config.gadget.udc_car,
            udc_rx: config.gadget.udc_extra,
            bt_name: config.bluetooth.name,
            hci: config.bluetooth.device,
            iface: config.wifi.device,
            wpa: config.wifi_network.wpa,
            ssid: config.wifi_network.ssid,
            channel: config.wifi_network.channel,
            passphrase: config.wifi_network.password,
            bt_cache_file: bt_cache_file.to_str().expect("corrupted file path").into(),
            persist_dir,
            hardware: config.hardware,
        };
        let g = ProdGadget::new(cfg);
        info!("GadgetManager output configured");

        self.gadget_manager = Some(g);
        self.enabled = true;
        Ok(())
    }
}
