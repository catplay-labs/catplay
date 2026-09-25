extern crate std;

use alloc::{format, string::String, sync::Arc, vec::Vec};

use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufWriter, Read, Write},
    os::unix::fs::MetadataExt,
    path::Path,
    sync::Mutex,
};

use log::{debug, error, info, warn};
use ring::{rand::SystemRandom, signature::Ed25519KeyPair};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{HomekitIdentity, HomekitStorageError, HomekitStorageRef};

use super::HomekitStorage;

#[derive(Serialize, Deserialize, Clone)]
pub struct HomekitStorageFile {
    #[serde(skip)]
    keypair: Option<Arc<Ed25519KeyPair>>,
    keypair_pkcs8: Vec<u8>,

    device_id: Uuid,
    paired: Arc<Mutex<HashMap<Uuid, [u8; 32]>>>,

    #[serde(skip)]
    file_path: Option<String>,
}

impl HomekitIdentity for HomekitStorageFile {
    fn device_ed25519_keypair(&self) -> Arc<Ed25519KeyPair> {
        self.keypair.as_ref().cloned().unwrap()
    }

    fn device_id(&self) -> Uuid {
        self.device_id
    }
}

impl HomekitStorage for HomekitStorageFile {
    /// Find previously paired device (it's pubkey) by uuid
    fn find_paired_by_id(&self, id: &Uuid) -> Option<[u8; 32]> {
        let paired = self.paired.lock().unwrap();
        paired.get(id).cloned()
    }

    fn unpair(&self, id: &Uuid) -> Result<bool, HomekitStorageError> {
        let mut paired = self.paired.lock().unwrap();
        let ret = paired.remove(id).is_some();
        self.flush()?;
        Ok(ret)
    }

    fn add_paired(&self, id: Uuid, key: [u8; 32]) -> Result<(), HomekitStorageError> {
        {
            let mut paired = self.paired.lock().unwrap();
            paired.insert(id, key);
        }

        self.flush()
    }

    fn paired_ids(&self) -> Vec<Uuid> {
        let paired = self.paired.lock().unwrap();
        paired.keys().copied().collect()
    }

    fn flush(&self) -> Result<(), HomekitStorageError> {
        if let Some(file_path) = &self.file_path {
            let file = File::create(file_path.clone()).inspect_err(|err| debug!("Failed File::create {file_path}: {err:?}"))?;
            let mut writer = BufWriter::new(file);
            let data = postcard::to_stdvec(self).map_err(|e| io::Error::other(format!("{}", e)))?;
            writer
                .write_all(&data)
                .inspect_err(|err| debug!("Failed File::write_all {file_path}: {err:?}"))?;
        }

        Ok(())
    }
}

fn random_identity() -> (Uuid, Vec<u8>) {
    let rng = SystemRandom::new();

    let uuid = Uuid::new_v4();
    let keypair_pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng)
        .expect("failed to generate keypair")
        .as_ref()
        .to_vec();

    (uuid, keypair_pkcs8)
}

impl HomekitStorageFile {
    pub fn memory() -> HomekitStorageRef {
        Self::random_identity(None)
    }

    fn random_identity(file_path: Option<&str>) -> HomekitStorageRef {
        let (device_id, keypair_pkcs8) = random_identity();
        let keypair = Ed25519KeyPair::from_pkcs8(&keypair_pkcs8).expect("failed to parse keypair");

        let me = HomekitStorageFile {
            keypair_pkcs8,
            keypair: Some(Arc::new(keypair)),
            device_id,
            paired: Arc::new(Mutex::new(HashMap::new())),
            file_path: file_path.map(|s| s.into()),
        };

        Arc::new(me)
    }

    pub fn file(file_path: &str) -> Result<HomekitStorageRef, HomekitStorageError> {
        match Self::file_try(file_path) {
            Ok(v) => Ok(v),
            Err(err) => {
                error!("Recovering corrupted HomeKit storage! Error: {err:?}");
                let me = Self::random_identity(Some(file_path));
                warn!("Attempting HomeKit flush to disk now");
                me.flush()?;
                warn!("HomeKit recovered with new identity");
                Ok(me)
            }
        }
    }

    pub fn file_try(file_path: &str) -> Result<HomekitStorageRef, HomekitStorageError> {
        let path = Path::new(&file_path);
        if !path.exists() || path.metadata()?.size() == 0 {
            info!("Initializing HomeKit with random identity");
            let me = Self::random_identity(Some(file_path));
            me.flush()?;
            return Ok(me);
        }

        let mut file = File::open(file_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let mut data: HomekitStorageFile = postcard::from_bytes(&data).map_err(|e| io::Error::other(format!("{}", e)))?;

        data.file_path = Some(file_path.into());
        data.keypair = Some(Arc::new(
            Ed25519KeyPair::from_pkcs8(&data.keypair_pkcs8).map_err(|e| io::Error::other(format!("failed to parse keypair: {e:?}")))?,
        ));

        Ok(Arc::new(data))
    }
}

#[cfg(test)]
mod tests {
    use catplay_tracing::logger::setup_test_logger;
    use uuid::Uuid;

    use crate::HomekitStorageFile;

    #[test]
    fn test_serialize() {
        setup_test_logger(true);

        let storage = HomekitStorageFile::file("/tmp/homekit_test.bin").unwrap();
        let key: [u8; 32] = [1u8; 32];

        let uuid = storage.device_id();
        let peer = Uuid::new_v4();
        storage.add_paired(peer, key).unwrap();
        storage.flush().unwrap();

        let storage = HomekitStorageFile::file("/tmp/homekit_test.bin").unwrap();
        assert_eq!(storage.device_id(), uuid);
        assert_eq!(storage.find_paired_by_id(&peer), Some(key))
    }
}
