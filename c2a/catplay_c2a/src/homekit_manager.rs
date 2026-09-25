use std::{error::Error, path::Path};

use catplay_hap::{HomekitStorageFile, HomekitStorageRef};

use crate::{AppConfig, MfiManager};

#[derive(Default)]
pub struct HomeKitManager {
    pub homekit_tx: Option<HomekitStorageRef>,
    pub homekit_rx: Option<HomekitStorageRef>,
}

impl HomeKitManager {
    pub fn start(&mut self, config: &AppConfig, _mfi: &MfiManager) -> Result<(), Box<dyn Error>> {
        let persist_dir = config
            .persist_dir
            .as_deref()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "persist_dir is required for HomeKit storage"))?;
        let db_path_tx = Path::new(persist_dir).join("homekit_tx_db.bin");
        let db_path_rx = Path::new(persist_dir).join("homekit_rx_db.bin");

        self.homekit_tx
            .replace(HomekitStorageFile::file(db_path_tx.to_str().unwrap())?);
        self.homekit_rx
            .replace(HomekitStorageFile::file(db_path_rx.to_str().unwrap())?);

        Ok(())
    }
}
