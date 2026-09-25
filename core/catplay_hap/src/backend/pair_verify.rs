use alloc::{str, string::ToString, vec, vec::Vec};

use log::debug;
use ring::agreement::{EphemeralPrivateKey, PublicKey};
use uuid::Uuid;

use crate::{
    backend::hkdf::hkdf_extract_and_expand,
    cipher::{HomeKitChaChaNonce, HomeKitCipher},
    key32_from_vec,
    ring_util::{create_ed25519_pubkey, create_x25519_key_ephermal, create_x25519_pubkey, x25519_agree_ephermal},
    storage::HomekitStorage,
    tlv::{TlvContainer, Value},
};

use super::tlv::{self, Encodable, Type};

#[derive(Debug, Clone)]
#[repr(u8)]
enum StepNumber {
    Unknown = 0,
    StartReq = 1,
    StartRes = 2,
    FinishReq = 3,
    FinishRes = 4,
}

#[derive(Debug)]
pub enum PairVerify {
    Start {
        ephermal_pub: PublicKey,
        ephermal: EphemeralPrivateKey,
    },

    Verify {
        b_pub: [u8; 32],

        remote_uuid: Uuid,
        shared_secret: [u8; 32],
    },

    StartController {
        ephermal_pub: PublicKey,
        ephermal: EphemeralPrivateKey,
    },

    VerifyController {
        a_pub: [u8; 32],
        b_pub: PublicKey,

        shared_secret: [u8; 32],
        session_key: [u8; 32],
    },

    Finish {
        device_pairing_id: Uuid,
        device_pubkey: [u8; 32],
        shared_secret: [u8; 32],
    },
}

impl PairVerify {
    pub fn client() -> Result<(Self, Vec<u8>), ring::error::Unspecified> {
        let ephermal = create_x25519_key_ephermal()?;
        let ephermal_pub = ephermal.compute_public_key()?;
        let payload = Self::start(ephermal_pub.as_ref().to_vec());

        Ok((PairVerify::StartController { ephermal, ephermal_pub }, payload))
    }

    pub fn server() -> Result<Self, ring::error::Unspecified> {
        let ephermal = create_x25519_key_ephermal()?;
        let ephermal_pub = ephermal.compute_public_key()?;

        Ok(PairVerify::Start { ephermal_pub, ephermal })
    }
}

impl PairVerify {
    fn cipher_nonce(label: &[u8; 8]) -> HomeKitChaChaNonce {
        HomeKitChaChaNonce(u64::from_le_bytes(*label))
    }

    fn start(pubkey: Vec<u8>) -> Vec<u8> {
        vec![Value::PublicKey(pubkey), Value::State(StepNumber::StartReq as u8)].encode()
    }

    pub fn handle(self, homekit: &dyn HomekitStorage, body: &[u8]) -> Result<(Self, Vec<u8>), (tlv::Error, Vec<u8>)> {
        let decoded = tlv::decode(body);
        let Some(_state) = decoded
            .get(&(Type::State as u8))
            .and_then(|d| d.first())
            .cloned()
        else {
            let err = tlv::ErrorContainer::new(StepNumber::Unknown as u8, tlv::Error::Unknown);
            return Err((tlv::Error::Unknown, err.encode()));
        };

        let ret = match self {
            PairVerify::Start { ephermal_pub, ephermal } => Self::handle_m1(homekit, ephermal_pub, ephermal, decoded),
            PairVerify::StartController { ephermal_pub, ephermal } => Self::handle_m2(homekit, ephermal_pub, ephermal, decoded),

            PairVerify::VerifyController {
                a_pub,
                b_pub,
                shared_secret,
                session_key,
            } => Self::handle_m3(homekit, b_pub, a_pub, shared_secret, session_key, decoded),
            PairVerify::Verify {
                b_pub: remote_pub,
                remote_uuid,
                shared_secret,
            } => Self::handle_m4(remote_uuid, remote_pub, shared_secret),
            PairVerify::Finish { .. } => Ok((self, vec![])),
        };

        match ret {
            Ok(res) => Ok((res.0, res.1.encode())),
            Err(code) => {
                let err = tlv::ErrorContainer::new(_state, code);
                Err((code, err.encode()))
            }
        }
    }
}

// Accessory functions
impl PairVerify {
    fn handle_m1(
        homekit: &dyn HomekitStorage,

        b_pub: PublicKey,
        b: EphemeralPrivateKey,

        decoded: TlvContainer,
    ) -> Result<(PairVerify, tlv::Container), tlv::Error> {
        debug!("pair verify M1: received verify start request");

        let a_pub_bytes = decoded
            .get(&(Type::PublicKey as u8))
            .and_then(|v| key32_from_vec(v))
            .ok_or(tlv::Error::Unknown)?;
        let a_pub = create_x25519_pubkey(&a_pub_bytes);

        let shared_secret = x25519_agree_ephermal(b, &a_pub).map_err(|_| tlv::Error::Authentication)?;

        let device_id = homekit.device_id().to_string();

        let accessory_info = [b_pub.as_ref(), device_id.as_bytes(), a_pub.bytes()].concat();
        let accessory_signature = homekit.device_ed25519_keypair().sign(&accessory_info);

        let mut encoded_sub_tlv = vec![
            Value::Identifier(device_id),
            Value::Signature(accessory_signature.as_ref().to_vec()),
        ]
        .encode();

        let session_key = hkdf_extract_and_expand(b"Pair-Verify-Encrypt-Salt", &shared_secret, b"Pair-Verify-Encrypt-Info")?;

        let nonce = Self::cipher_nonce(b"PV-Msg02");
        let mut cipher = HomeKitCipher::new(session_key);

        let tag = cipher
            .encrypt(&mut encoded_sub_tlv, &[], nonce)
            .map_err(|_| tlv::Error::Unknown)?;
        encoded_sub_tlv.extend_from_slice(&tag);

        debug!("pair verify M2: sending verify start response");

        Ok((
            PairVerify::VerifyController {
                b_pub: b_pub.clone(),
                a_pub: a_pub_bytes,
                shared_secret,

                session_key,
            },
            vec![
                Value::State(StepNumber::StartRes as u8),
                Value::PublicKey(b_pub.as_ref().to_vec()),
                Value::EncryptedData(encoded_sub_tlv),
            ],
        ))
    }

    fn handle_m3(
        homekit: &dyn HomekitStorage,

        b_pub: PublicKey,
        a_pub: [u8; 32],
        shared_secret: [u8; 32],
        session_key: [u8; 32],

        mut decoded: TlvContainer,
    ) -> Result<(PairVerify, tlv::Container), tlv::Error> {
        debug!("pair verify M3: received verify finish request");
        let mut data = decoded
            .remove(&(Type::EncryptedData as u8))
            .ok_or(tlv::Error::Unknown)?;

        let nonce = Self::cipher_nonce(b"PV-Msg03");
        let mut cipher = HomeKitCipher::new(session_key);

        let decrypted = cipher
            .decrypt(&mut data, &[], nonce)
            .map_err(|_| tlv::Error::Authentication)?;

        let sub_tlv = tlv::decode(decrypted);

        let device_pairing_id = sub_tlv
            .get(&(Type::Identifier as u8))
            .ok_or(tlv::Error::Unknown)?;
        let device_signature = sub_tlv
            .get(&(Type::Signature as u8))
            .ok_or(tlv::Error::Unknown)?;

        let pairing_uuid = Uuid::parse_str(str::from_utf8(device_pairing_id)?)?;
        let pairing_public_key = homekit
            .find_paired_by_id(&pairing_uuid)
            .ok_or(tlv::Error::Authentication)?; // don't remember this device

        let device_info: Vec<u8> = [&a_pub[..], device_pairing_id, b_pub.as_ref()].concat();

        create_ed25519_pubkey(&pairing_public_key)
            .verify(&device_info, device_signature)
            .map_err(|_| tlv::Error::Authentication)?;

        debug!("pair verify M4: sending verify finish response");

        Ok((
            PairVerify::Finish {
                device_pairing_id: pairing_uuid,
                device_pubkey: pairing_public_key,
                shared_secret,
            },
            vec![Value::State(StepNumber::FinishRes as u8)],
        ))
    }
}

// Controller functions
impl PairVerify {
    fn handle_m2(
        homekit: &dyn HomekitStorage,

        a_pub: PublicKey,
        a: EphemeralPrivateKey,
        mut decoded: TlvContainer,
    ) -> Result<(PairVerify, tlv::Container), tlv::Error> {
        debug!("pair verify M2: received verify start response (controller)");

        let b_pub_bytes = decoded
            .get(&(Type::PublicKey as u8))
            .and_then(|v| key32_from_vec(v))
            .ok_or(tlv::Error::Unknown)?;
        let b_pub = create_x25519_pubkey(&b_pub_bytes);
        let mut encrypted = decoded
            .remove(&(Type::EncryptedData as u8))
            .ok_or(tlv::Error::Unknown)?;

        let controller_id = homekit.device_id().to_string();

        let shared_secret = x25519_agree_ephermal(a, &b_pub).map_err(|_| tlv::Error::Authentication)?;
        let session_key = hkdf_extract_and_expand(b"Pair-Verify-Encrypt-Salt", &shared_secret, b"Pair-Verify-Encrypt-Info")?;

        let nonce = Self::cipher_nonce(b"PV-Msg02");
        let mut cipher = HomeKitCipher::new(session_key);

        let decrypted = cipher
            .decrypt(&mut encrypted, &[], nonce)
            .map_err(|_| tlv::Error::Authentication)?;

        let sub_tlv = tlv::decode(decrypted);

        let accessory_id = sub_tlv
            .get(&(Type::Identifier as u8))
            .ok_or(tlv::Error::Unknown)?;
        let accessory_signature = sub_tlv
            .get(&(Type::Signature as u8))
            .ok_or(tlv::Error::Unknown)?;

        let accessory_info = [b_pub.as_ref(), accessory_id, a_pub.as_ref()].concat();
        let accessory_uuid = Uuid::parse_str(str::from_utf8(accessory_id)?)?;
        let accessory_pubkey = homekit
            .find_paired_by_id(&accessory_uuid)
            .ok_or(tlv::Error::Authentication)?;

        create_ed25519_pubkey(&accessory_pubkey)
            .verify(&accessory_info, accessory_signature)
            .map_err(|_| tlv::Error::Authentication)?;

        let controller_info = [a_pub.as_ref(), controller_id.as_bytes(), b_pub.bytes()].concat();
        let controller_signature = homekit.device_ed25519_keypair().sign(&controller_info);

        let mut sub_tlv = vec![
            Value::Identifier(controller_id),
            Value::Signature(controller_signature.as_ref().to_vec()),
        ]
        .encode();

        let nonce = Self::cipher_nonce(b"PV-Msg03");
        let tag = cipher
            .encrypt(&mut sub_tlv, &[], nonce)
            .map_err(|_| tlv::Error::Unknown)?;
        sub_tlv.extend_from_slice(&tag);

        debug!("pair verify M3: sending verify finish request");

        Ok((
            PairVerify::Verify {
                remote_uuid: accessory_uuid,
                b_pub: accessory_pubkey,
                shared_secret,
            },
            vec![Value::State(StepNumber::FinishReq as u8), Value::EncryptedData(sub_tlv)],
        ))
    }

    fn handle_m4(remote_uuid: Uuid, b_pub: [u8; 32], shared_secret: [u8; 32]) -> Result<(PairVerify, tlv::Container), tlv::Error> {
        debug!("pair verify M4: received verify finish response (controller)");

        Ok((
            PairVerify::Finish {
                device_pairing_id: remote_uuid,
                device_pubkey: b_pub,
                shared_secret,
            },
            vec![],
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::panic;

    use catplay_tracing::logger::setup_test_logger;
    use ring::signature::KeyPair;

    use super::*;
    use crate::storage::HomekitStorageFile;

    #[test]
    fn test_pair_verify_end_to_end() {
        setup_test_logger(false);

        let hk_controller = &*HomekitStorageFile::memory();
        let hk_accessory = &*HomekitStorageFile::memory();

        let controller_id = hk_controller.device_id();
        let controller_pub = hk_controller
            .device_ed25519_keypair()
            .public_key()
            .as_ref()
            .to_vec();
        hk_accessory
            .add_paired(controller_id, controller_pub.clone().try_into().unwrap())
            .unwrap();

        let accessory_id = hk_accessory.device_id();
        let accessory_pub = hk_accessory
            .device_ed25519_keypair()
            .public_key()
            .as_ref()
            .to_vec();
        hk_controller
            .add_paired(accessory_id, accessory_pub.clone().try_into().unwrap())
            .unwrap();

        let (controller, payload_m1) = PairVerify::client().unwrap();
        let accessory = PairVerify::server().unwrap();

        // --- M1: Controller -> Accessory ---
        let (accessory, payload_m2) = accessory
            .handle(hk_accessory, &payload_m1)
            .expect("accessory M1 failed");
        // --- M2: Accessory -> Controller ---
        let (controller, payload_m3) = controller
            .handle(hk_controller, &payload_m2)
            .expect("controller M2 failed");
        // --- M3: Controller -> Accessory ---
        let (accessory, payload_m4) = accessory
            .handle(hk_accessory, &payload_m3)
            .expect("accessory M3 failed");
        // --- M4: Accessory -> Controller ---
        let (controller, _) = controller
            .handle(hk_controller, &payload_m4)
            .expect("controller M4 failed");

        let controller_secret: [u8; 32];

        match controller {
            PairVerify::Finish {
                device_pairing_id,
                device_pubkey,
                shared_secret,
            } => {
                controller_secret = shared_secret;
                assert_eq!(device_pubkey.to_vec(), accessory_pub, "controller result: pubkey mismatch");
                assert_eq!(device_pairing_id, accessory_id, "controller result: uuid mismatch");
            }
            _ => panic!("Unexpected controller state: {controller:?}"),
        };

        match accessory {
            PairVerify::Finish {
                device_pairing_id,
                device_pubkey,
                shared_secret,
            } => {
                assert_eq!(shared_secret, controller_secret, "shared secrets differ");

                assert_eq!(device_pubkey.to_vec(), controller_pub, "accessory result: pubkey mismatch");
                assert_eq!(device_pairing_id, controller_id, "accessory result: uuid mismatch");
            }
            _ => panic!("Unexpected accessory state: {controller:?}"),
        };
    }
}
