use aes::Aes128;
use aes::cipher::{KeyIvInit, StreamCipher};
use alloc::{string::String, string::ToString, sync::Arc, vec::Vec};
use catplay_mfi::MfiDevice;
use ctr::Ctr128BE;
use log::debug;

use crate::{backend::hkdf::sha1_salt, key32_from_vec};
use crate::{
    backend::hkdf::sha256_salt,
    ring_util::{create_x25519_key_ephermal, create_x25519_pubkey, x25519_agree_ephermal},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum MfiSapError {
    #[error("MFi session not ready: /auth-setup is required first")]
    SessionNotReady,
    #[error("Invalid key size for /auth-setup")]
    InvalidKeySize,
}

pub enum MfiSapStep {
    Start { _version: u8, client_pubkey: [u8; 32] },
}

type Aes128Ctr = Ctr128BE<Aes128>;

#[derive(Debug)]
pub struct MfiSapResponse {
    pub public_key: [u8; 32],
    pub certificate: Vec<u8>,
    pub signature: Vec<u8>,
}

pub struct MfiSapSuccess {
    pub shared_secret: [u8; 32],
    pub response: Option<MfiSapResponse>,
    aes_ctr: Aes128Ctr,
}

impl MfiSapSuccess {
    fn new(shared_secret: [u8; 32], response: Option<MfiSapResponse>, aes_ctr: Aes128Ctr) -> Self {
        Self {
            shared_secret,
            response,
            aes_ctr,
        }
    }

    pub fn xcrypt_audio_key(&mut self, key: &[u8]) -> Result<[u8; 16], MfiSapError> {
        if key.len() != 16 {
            return Err(MfiSapError::InvalidKeySize);
        }
        let mut decrypted = key.to_vec();
        self.aes_ctr.apply_keystream(&mut decrypted);

        match decrypted.try_into() {
            Ok(v) => Ok(v),
            Err(_) => Err(MfiSapError::InvalidKeySize),
        }
    }
}

#[allow(clippy::large_enum_variant)]
pub enum MfiSapSession {
    ServerStart,
    ClientStart {
        client_secret: ring::agreement::EphemeralPrivateKey,
    },
    Finish {
        success: MfiSapSuccess,
    },
}

impl MfiSapSession {
    pub fn server() -> Self {
        Self::ServerStart
    }

    pub fn client() -> Result<(Self, Vec<u8>), String> {
        let client_secret = create_x25519_key_ephermal().map_err(|_| "failed to generate keypair")?;
        let client_pubkey = client_secret
            .compute_public_key()
            .map_err(|_| "failed to compute public key")?;

        let mut request = Vec::with_capacity(33);
        request.push(1);
        request.extend_from_slice(client_pubkey.as_ref());

        Ok((Self::ClientStart { client_secret }, request))
    }
}

impl MfiSapSession {
    pub fn parse(body: &[u8]) -> Result<MfiSapStep, String> {
        if body.len() != 33 {
            return Err("invalid msg len".into());
        }

        let version = body[0];
        let client_pubkey = &body[1..33];

        Ok(MfiSapStep::Start {
            _version: version,
            client_pubkey: client_pubkey.try_into().unwrap(),
        })
    }

    pub fn handle(self, mfi: Arc<dyn MfiDevice>, body: &[u8]) -> Result<(Self, Vec<u8>), String> {
        match self {
            Self::ServerStart => Self::handle_client_request(mfi, body),
            Self::ClientStart { client_secret } => Self::handle_server_response(client_secret, body),
            Self::Finish { .. } => Ok((self, Vec::new())),
        }
    }

    pub fn handle_client_response(self, body: &[u8]) -> Result<(Self, Vec<u8>), String> {
        match self {
            Self::ClientStart { client_secret } => Self::handle_server_response(client_secret, body),
            _ => Err("invalid auth-setup client state".into()),
        }
    }

    fn handle_client_request(mfi: Arc<dyn MfiDevice>, body: &[u8]) -> Result<(Self, Vec<u8>), String> {
        if body.len() != 33 {
            return Err("invalid msg len".into());
        }

        let version = body[0];
        let client_pubkey = &body[1..33];

        let step = MfiSapStep::Start {
            _version: version,
            client_pubkey: client_pubkey.try_into().unwrap(),
        };

        match step {
            MfiSapStep::Start { client_pubkey, .. } => Self::handle_m1(mfi, &client_pubkey),
        }
    }

    fn handle_m1(mfi: Arc<dyn MfiDevice>, client_pubkey: &[u8]) -> Result<(Self, Vec<u8>), String> {
        let Some(client_pubkey) = key32_from_vec(client_pubkey) else {
            return Err("invalid key len".into());
        };

        let server_secret = create_x25519_key_ephermal().map_err(|_| "failed to generate keypair")?;
        let server_pubkey = server_secret
            .compute_public_key()
            .map_err(|_| "failed to compute public key")?;

        let shared_secret = x25519_agree_ephermal(server_secret, &create_x25519_pubkey(&client_pubkey)).map_err(|_| "ECDH failed")?;

        let mut aes_key = [0u8; 16];
        let mut aes_iv = [0u8; 16];
        aes_key.copy_from_slice(&sha1_salt(b"AES-KEY", &shared_secret)[..16]);
        aes_iv.copy_from_slice(&sha1_salt(b"AES-IV", &shared_secret)[..16]);

        let mut cipher = Aes128Ctr::new_from_slices(&aes_key, &aes_iv).map_err(|_| "invalid AES setup")?;

        // Hash (server_pubkey || client_pubkey)
        let cert = mfi.read_certificate().map_err(|e| e.to_string())?;
        let digest = if cert.len() > 640 {
            debug!("Old chip, using sha1 for auth-setup");
            sha1_salt(server_pubkey.as_ref(), &client_pubkey).to_vec()
        } else {
            debug!("Modern chip, using sha256 for auth-setup");
            sha256_salt(server_pubkey.as_ref(), &client_pubkey).to_vec()
        };

        // Sign with MFi
        let (mut signature, certificate) = {
            let sig = mfi
                .generate_challenge_response(&digest)
                .map_err(|e| e.to_string())?;
            (sig, cert)
        };

        // Encrypt signature
        cipher.apply_keystream(&mut signature);

        // Response: [32:pub] [4:cert_len] [cert] [4:sig_len] [sig]
        let mut response: Vec<u8> = Vec::with_capacity(32 + 4 + certificate.len() + 4 + signature.len());
        response.extend_from_slice(server_pubkey.as_ref());
        response.extend_from_slice(&(certificate.len() as u32).to_be_bytes());
        response.extend_from_slice(&certificate);
        response.extend_from_slice(&(signature.len() as u32).to_be_bytes());
        response.extend_from_slice(&signature);

        Ok((
            Self::Finish {
                success: MfiSapSuccess::new(shared_secret, None, cipher),
            },
            response,
        ))
    }

    fn handle_server_response(client_secret: ring::agreement::EphemeralPrivateKey, body: &[u8]) -> Result<(Self, Vec<u8>), String> {
        if body.len() < 40 {
            return Err("invalid msg len".into());
        }

        let Some(server_pubkey) = key32_from_vec(&body[..32]) else {
            return Err("invalid key len".into());
        };

        let cert_len = u32::from_be_bytes(body[32..36].try_into().unwrap()) as usize;
        let sig_len_offset = 36usize
            .checked_add(cert_len)
            .ok_or("invalid certificate len")?;
        let sig_offset = sig_len_offset
            .checked_add(4)
            .ok_or("invalid signature len")?;

        if body.len() < sig_offset {
            return Err("invalid certificate len".into());
        }

        let sig_len = u32::from_be_bytes(body[sig_len_offset..sig_offset].try_into().unwrap()) as usize;
        let sig_end = sig_offset
            .checked_add(sig_len)
            .ok_or("invalid signature len")?;
        if body.len() != sig_end {
            return Err("invalid signature len".into());
        }

        let certificate = body[36..sig_len_offset].to_vec();
        let shared_secret = x25519_agree_ephermal(client_secret, &create_x25519_pubkey(&server_pubkey)).map_err(|_| "ECDH failed")?;

        let mut aes_key = [0u8; 16];
        let mut aes_iv = [0u8; 16];
        aes_key.copy_from_slice(&sha1_salt(b"AES-KEY", &shared_secret)[..16]);
        aes_iv.copy_from_slice(&sha1_salt(b"AES-IV", &shared_secret)[..16]);

        let mut cipher = Aes128Ctr::new_from_slices(&aes_key, &aes_iv).map_err(|_| "invalid AES setup")?;
        let mut signature = body[sig_offset..sig_end].to_vec();
        cipher.apply_keystream(&mut signature);

        Ok((
            Self::Finish {
                success: MfiSapSuccess::new(
                    shared_secret,
                    Some(MfiSapResponse {
                        public_key: server_pubkey,
                        certificate,
                        signature,
                    }),
                    cipher,
                ),
            },
            Vec::new(),
        ))
    }

    pub fn xcrypt_audio_key(&mut self, key: &[u8]) -> Result<[u8; 16], MfiSapError> {
        let Self::Finish { success } = self else {
            return Err(MfiSapError::SessionNotReady);
        };

        success.xcrypt_audio_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use catplay_mfi::MfiResult;

    struct TestMfiDevice;

    impl MfiDevice for TestMfiDevice {
        fn read_certificate(&self) -> MfiResult<Vec<u8>> {
            Ok(vec![0x42; 641])
        }

        fn generate_challenge_response(&self, challenge: &[u8]) -> MfiResult<Vec<u8>> {
            Ok(challenge.iter().copied().cycle().take(64).collect())
        }
    }

    #[test]
    fn iphone_reverse_auth_setup_round_trip() {
        let mfi = Arc::new(TestMfiDevice);
        let (client, client_request) = MfiSapSession::client().unwrap();
        let server = MfiSapSession::server();

        let (mut server, server_response) = server.handle(mfi.clone(), &client_request).unwrap();
        let (client, client_response) = client.handle(mfi, &server_response).unwrap();

        assert!(client_response.is_empty());

        let MfiSapSession::Finish { success: server_success } = &server else {
            panic!("server did not finish");
        };

        let MfiSapSession::Finish { success: client_success } = &client else {
            panic!("client did not finish");
        };
        let Some(client_response_parts) = &client_success.response else {
            panic!("client did not receive response parts");
        };

        assert_eq!(server_success.shared_secret, client_success.shared_secret);
        assert_eq!(&server_response[..32], &client_response_parts.public_key);
        assert_eq!(client_response_parts.certificate, vec![0x42; 641]);
        assert_eq!(client_response_parts.signature.len(), 64);

        let key = [0xA5; 16];
        let encrypted = server.xcrypt_audio_key(&key).unwrap();
        assert_ne!(encrypted, key);

        let MfiSapSession::Finish {
            success: mut client_success,
        } = client
        else {
            panic!("client did not finish");
        };
        assert_eq!(client_success.xcrypt_audio_key(&encrypted).unwrap(), key);
    }
}
