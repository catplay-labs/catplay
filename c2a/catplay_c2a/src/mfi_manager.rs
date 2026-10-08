use std::{error::Error, fs, path::Path, sync::Arc, thread::spawn};

use catplay_mfi::{
    MfiDeficeRef, MfiDevice, MfiDeviceI2C, MfiDeviceLocal,
    server::{MfiDeviceRemoteClient, MfiDeviceServer},
};
use log::info;
use uuid::Uuid;

use crate::AppConfig;

pub struct MfiManager {
    device: Option<Arc<dyn MfiDevice>>,
}

impl MfiManager {
    pub fn new() -> Self {
        Self { device: None }
    }

    fn cert_hash(&self, mfi: &Arc<dyn MfiDevice>) -> Result<Uuid, Box<dyn Error>> {
        let mfi_cert: Vec<u8> = mfi.read_certificate()?;
        Ok(Uuid::new_v5(&Uuid::NAMESPACE_DNS, &mfi_cert))
    }

    pub fn get_device(&self) -> Arc<dyn MfiDevice> {
        self.device.as_ref().unwrap().clone()
    }

    pub fn start(&mut self, config: &AppConfig) -> Result<(), Box<dyn Error>> {
        let local_files = config.persist_dir.as_deref().and_then(|dir| {
            let certificate = Path::new(dir).join("certificate.p7b");
            let identity = Path::new(dir).join("identity.pk8");
            (certificate.is_file() && identity.is_file()).then_some((certificate, identity))
        });

        let mfi: MfiDeficeRef = if let Some((certificate, identity)) = local_files {
            let certificate = fs::read(certificate)?;
            let identity = fs::read(identity)?;
            let device = MfiDeviceLocal::new(&certificate, &identity)?;
            info!("Using local MFi backend from persist_dir (certificate.p7b + identity.pk8)");
            Arc::new(device)
        } else if let Some(mfi_client) = &config.mfi.client {
            Arc::new(MfiDeviceRemoteClient::new(mfi_client.remote.clone())?)
        } else if let Some(mfi_i2c) = &config.mfi.i2c {
            Arc::new(MfiDeviceI2C::new(
                mfi_i2c.bus_offset,
                mfi_i2c.dev_addr, /*, Duration::from_millis(mfi_i2c.timeout_ms)*/
            )?)
        } else {
            return Err("mfi backend not defined".into());
        };

        if config.mfi.selftest {
            info!("Performing MFi self-test");
            let mfi_cert: Vec<u8> = mfi.read_certificate()?;
            let uuid = self.cert_hash(&mfi)?;

            info!("MFI booted cert size = {} hash = {}", mfi_cert.len(), uuid);
        }

        self.device.replace(mfi.clone());

        if let Some(mfi_server) = &config.mfi.server
            && mfi_server.enabled
        {
            info!("Starting MFI server on {}", mfi_server.bind.clone());

            let mut server = MfiDeviceServer::new(mfi_server.bind.clone(), mfi.clone());
            server
                .bind()
                .map_err(|err| format!("failed to bind mfi server: {}", err))?;
            spawn(move || server.listen());
            info!("MFI server is now running in background")
        }

        Ok(())
    }
}
