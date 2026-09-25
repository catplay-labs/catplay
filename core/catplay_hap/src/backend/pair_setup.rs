use alloc::{
    str,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use catplay_tracing_macro::trace_time;
use log::debug;
use ring::{
    rand::{SecureRandom, SystemRandom},
    signature::KeyPair,
};
use uuid::Uuid;

use crate::{
    HomekitIdentity,
    backend::{
        hkdf::{client_proof, hkdf_extract_and_expand, sha512, verify_client_proof, verify_server_proof},
        tlv::Method,
    },
    cipher::{HomeKitChaChaNonce, HomeKitCipher},
    create_ed25519_pubkey, key32_from_vec,
    storage::HomekitStorage,
    tlv::TlvContainer,
};

#[cfg(feature = "openssl")]
use super::srp_openssl::{Client as SrpClient, Server as SrpServer};
use super::tlv::{self, Encodable, Type, Value};
#[cfg(not(feature = "openssl"))]
use crate::RingSha512;
#[cfg(not(feature = "openssl"))]
type SrpClient = srp::Client<srp::groups::G3072, RingSha512>;
#[cfg(not(feature = "openssl"))]
type SrpServer = srp::Server<srp::groups::G3072, RingSha512>;

pub const CARPLAY_MAGIC_PIN: &str = "3939";

#[allow(clippy::needless_return)]
fn srp_compute_verifier(pin: &str, salt: &[u8]) -> Vec<u8> {
    #[cfg(feature = "openssl")]
    {
        return SrpClient::new()
            .compute_verifier(b"Pair-Setup", pin.as_bytes(), salt)
            .expect("OpenSSL SRP verifier computation failed");
    }

    #[cfg(not(feature = "openssl"))]
    {
        SrpClient::new().compute_verifier(b"Pair-Setup", pin.as_bytes(), salt)
    }
}

#[allow(clippy::needless_return)]
fn srp_compute_server_public_ephemeral(b: &[u8], verifier: &[u8]) -> Vec<u8> {
    #[cfg(feature = "openssl")]
    {
        return SrpServer::new()
            .compute_public_ephemeral(b, verifier)
            .expect("OpenSSL SRP server public ephemeral computation failed");
    }

    #[cfg(not(feature = "openssl"))]
    {
        SrpServer::new().compute_public_ephemeral(b, verifier)
    }
}

#[allow(clippy::needless_return)]
fn srp_compute_client_public_ephemeral(a: &[u8]) -> Vec<u8> {
    #[cfg(feature = "openssl")]
    {
        return SrpClient::new()
            .compute_public_ephemeral(a)
            .expect("OpenSSL SRP client public ephemeral computation failed");
    }

    #[cfg(not(feature = "openssl"))]
    {
        SrpClient::new().compute_public_ephemeral(a)
    }
}

#[allow(clippy::needless_return)]
fn srp_process_server_reply_legacy(b: &[u8], verifier: &[u8], a_pub: &[u8], b_pub: Option<&[u8]>) -> core::result::Result<Vec<u8>, ()> {
    #[cfg(feature = "openssl")]
    {
        let server = SrpServer::new();
        return match b_pub {
            Some(b_pub) => server
                .process_reply_legacy_with_b_pub(b, verifier, a_pub, b_pub)
                .map(|state| state.key().to_vec())
                .map_err(|_| ()),
            None => server
                .process_reply_legacy(b, verifier, a_pub)
                .map(|state| state.key().to_vec())
                .map_err(|_| ()),
        };
    }

    #[cfg(not(feature = "openssl"))]
    {
        #[allow(deprecated)]
        SrpServer::new()
            .process_reply_legacy(b, verifier, a_pub)
            .map(|state| state.key().to_vec())
            .map_err(|_| ())
    }
}

#[allow(clippy::needless_return)]
fn srp_process_client_reply(a: &[u8], pin: &str, salt: &[u8], b_pub: &[u8]) -> core::result::Result<Vec<u8>, ()> {
    #[cfg(feature = "openssl")]
    {
        return SrpClient::new()
            .process_reply(a, b"Pair-Setup", pin.as_bytes(), salt, b_pub)
            .map(|state| state.key().to_vec())
            .map_err(|_| ());
    }

    #[cfg(not(feature = "openssl"))]
    {
        SrpClient::new()
            .process_reply(a, b"Pair-Setup", pin.as_bytes(), salt, b_pub)
            .map(|state| state.key().to_vec())
            .map_err(|_| ())
    }
}

#[derive(Debug, Clone)]
enum StepNumber {
    Unknown = 0,
    SrpStartRequest = 1,
    SrpStartResponse = 2,
    SrpVerifyRequest = 3,
    SrpVerifyResponse = 4,
    ExchangeRequest = 5,
    ExchangeResponse = 6,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PairSetup {
    // Accessory
    Start {
        pin: String,
    },
    M2 {
        salt: [u8; 16],
        verifier: Vec<u8>,
        b: [u8; 64],
        b_pub: Vec<u8>, // 384 bytes OR less
    },
    M4 {
        shared_secret: [u8; 64],
    },

    // Controller
    M1 {
        pin: String,
    },
    M3 {
        a_pub: Vec<u8>, // 384 bytes OR less
        a_proof: [u8; 64],
        shared_secret: [u8; 64],
    },
    M5 {
        shared_secret: [u8; 64],
    },

    // Shared
    Finished {
        device_pairing_id: Uuid,
    },
}

impl PairSetup {
    fn cipher_nonce(label: &[u8; 8]) -> HomeKitChaChaNonce {
        HomeKitChaChaNonce(u64::from_le_bytes(*label))
    }

    pub fn client(pin: &str) -> (Self, Vec<u8>) {
        (PairSetup::M1 { pin: pin.into() }, Self::start())
    }

    pub fn server(pin: &str) -> Self {
        PairSetup::Start { pin: pin.into() }
    }
}

#[cfg(feature = "std")]
extern crate std;

impl PairSetup {
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
            PairSetup::Start { pin } => Self::handle_m1(pin, decoded),
            PairSetup::M1 { pin } => Self::handle_m2(pin, decoded),

            PairSetup::M2 { salt, verifier, b, b_pub } => Self::handle_m3(salt, verifier, b, b_pub, decoded),
            PairSetup::M4 { shared_secret } => Self::handle_m5(homekit, shared_secret, decoded),
            PairSetup::M3 {
                a_pub,
                a_proof,
                shared_secret,
            } => Self::handle_m4(homekit, a_pub, a_proof, shared_secret, decoded),
            PairSetup::M5 { shared_secret } => Self::handle_m6(homekit, shared_secret, decoded),
            PairSetup::Finished { .. } => Ok((self, vec![])),
        };

        match ret {
            Ok(res) => Ok((res.0, res.1.encode())),
            Err(code) => {
                let err = tlv::ErrorContainer::new(_state, code);
                Err((code, err.encode()))
            }
        }
    }

    fn start() -> Vec<u8> {
        vec![Value::Method(Method::None), Value::State(StepNumber::SrpStartRequest as u8)].encode()
    }
}

pub struct PairSetupCache<'a> {
    pub salt: [u8; 16],
    pub b: [u8; 64],
    pub pin: &'a str,

    pub verifier: [u8; 384], // 384 bytes OR less
    pub verifier_len: usize,
    pub computed_verifier: bool,

    pub b_pub: [u8; 384], // 384 bytes OR less
    pub b_pub_len: usize,
    pub computed_b_pub: bool,
}

impl<'a> PairSetupCache<'a> {
    pub fn random(pin: &'a str) -> Self {
        let rng = SystemRandom::new();
        let mut salt = [0u8; 16];
        rng.fill(&mut salt).unwrap();
        let mut b = [0u8; 64];
        rng.fill(&mut b).unwrap();

        let mut me = Self {
            salt,
            b,
            pin,
            verifier: [0u8; _],
            verifier_len: 384,
            computed_verifier: false,
            b_pub: [0u8; 384],
            b_pub_len: 0,
            computed_b_pub: false,
        };
        me.compute_verifier();
        me.compute_public_ephemeral();
        me
    }

    pub fn compute_verifier(&mut self) -> Vec<u8> {
        if self.computed_verifier {
            return self.verifier[..self.verifier_len].into();
        }

        let verifier = srp_compute_verifier(self.pin, &self.salt);
        self.verifier[..verifier.len()].copy_from_slice(&verifier);
        self.verifier_len = verifier.len();

        self.computed_verifier = true;
        self.verifier[..verifier.len()].into()
    }

    pub fn compute_public_ephemeral(&mut self) -> Vec<u8> {
        if self.computed_b_pub {
            return self.b_pub[..self.b_pub_len].into();
        }

        let verifier = self.compute_verifier();
        let b_pub = srp_compute_server_public_ephemeral(&self.b, &verifier);
        self.b_pub[..b_pub.len()].copy_from_slice(&b_pub);
        self.b_pub_len = b_pub.len();

        self.computed_b_pub = true;
        self.b_pub[..b_pub.len()].into()
    }
}

#[test]
fn generate_cache() {
    let cache = PairSetupCache::random(CARPLAY_MAGIC_PIN);

    println!("const CARPLAY_CACHE: PairSetupCache = PairSetupCache {{");
    println!("  salt: {:?},", cache.salt);
    println!("  b: {:?},", cache.b);
    println!("  pin: CARPLAY_MAGIC_PIN,");

    println!("  verifier: {:?},", cache.verifier);
    println!("  verifier_len: {:?},", cache.verifier_len);
    println!("  computed_verifier: true,");

    println!("  b_pub: {:?},", cache.b_pub);
    println!("  b_pub_len: {:?},", cache.b_pub_len);
    println!("  computed_b_pub: true }};");
}

const CARPLAY_CACHE: PairSetupCache = PairSetupCache {
    salt: [39, 51, 162, 216, 81, 217, 76, 46, 237, 110, 176, 62, 249, 168, 157, 222],
    b: [
        171, 121, 136, 173, 109, 51, 18, 16, 240, 244, 53, 12, 205, 131, 1, 40, 61, 215, 42, 84, 120, 225, 220, 99, 106, 159, 152, 109, 24,
        54, 135, 95, 76, 114, 239, 230, 92, 14, 232, 201, 169, 180, 122, 218, 16, 89, 144, 33, 168, 38, 57, 164, 12, 196, 215, 151, 145,
        128, 60, 228, 195, 18, 7, 253,
    ],
    pin: CARPLAY_MAGIC_PIN,
    verifier: [
        188, 42, 72, 107, 255, 95, 147, 222, 183, 206, 123, 209, 244, 166, 241, 26, 36, 93, 150, 20, 32, 72, 17, 79, 78, 24, 252, 214, 15,
        158, 126, 223, 150, 100, 159, 21, 87, 82, 163, 80, 247, 171, 172, 249, 215, 56, 122, 190, 112, 162, 189, 37, 156, 93, 1, 165, 28,
        138, 33, 225, 187, 169, 73, 198, 130, 16, 28, 123, 238, 163, 166, 53, 242, 158, 119, 10, 130, 229, 12, 40, 25, 241, 18, 177, 186,
        45, 115, 207, 184, 86, 102, 202, 154, 45, 139, 232, 36, 80, 162, 81, 141, 1, 245, 210, 239, 183, 71, 224, 73, 120, 197, 132, 172,
        28, 153, 62, 118, 21, 99, 134, 105, 47, 112, 146, 38, 198, 162, 12, 242, 184, 96, 89, 51, 187, 175, 188, 153, 181, 215, 14, 64,
        146, 241, 175, 217, 110, 28, 136, 22, 192, 81, 152, 131, 103, 102, 163, 33, 246, 63, 111, 97, 107, 75, 167, 167, 139, 24, 18, 67,
        182, 86, 229, 103, 109, 189, 131, 93, 3, 229, 244, 236, 208, 83, 184, 189, 68, 110, 55, 156, 240, 229, 52, 16, 130, 102, 127, 142,
        52, 155, 212, 4, 113, 159, 132, 254, 50, 134, 178, 80, 197, 164, 31, 153, 0, 93, 252, 198, 187, 151, 56, 199, 105, 30, 89, 252, 91,
        20, 170, 109, 250, 196, 93, 187, 68, 146, 129, 203, 187, 181, 88, 230, 25, 251, 89, 63, 43, 100, 60, 122, 119, 165, 21, 135, 204,
        137, 187, 119, 198, 205, 91, 79, 251, 78, 218, 236, 165, 239, 99, 99, 191, 97, 147, 143, 48, 130, 131, 21, 82, 88, 207, 8, 146, 20,
        87, 47, 109, 48, 4, 63, 78, 7, 151, 84, 103, 200, 251, 233, 246, 105, 70, 229, 33, 27, 185, 254, 182, 82, 127, 170, 70, 122, 170,
        130, 209, 191, 109, 103, 218, 42, 249, 195, 15, 173, 243, 75, 242, 19, 169, 141, 221, 171, 126, 110, 219, 30, 189, 211, 156, 24,
        249, 237, 150, 96, 58, 175, 194, 245, 230, 191, 108, 144, 212, 66, 165, 106, 124, 196, 46, 108, 221, 16, 198, 154, 235, 100, 178,
        81, 66, 167, 151, 28, 58, 158, 223, 95, 113, 240, 44, 208, 216, 229, 249, 212, 122,
    ],
    verifier_len: 384,
    computed_verifier: true,
    b_pub: [
        221, 69, 255, 226, 177, 204, 247, 161, 64, 66, 194, 47, 76, 33, 126, 2, 233, 182, 189, 254, 10, 178, 0, 234, 247, 240, 48, 33, 225,
        249, 116, 35, 52, 153, 223, 39, 176, 183, 88, 171, 113, 215, 249, 193, 76, 83, 158, 69, 187, 83, 106, 153, 174, 98, 169, 87, 204,
        86, 158, 118, 180, 21, 155, 109, 34, 254, 171, 82, 110, 193, 39, 174, 114, 215, 71, 81, 56, 62, 187, 57, 154, 51, 194, 5, 82, 40,
        247, 170, 218, 139, 71, 128, 34, 152, 201, 155, 119, 14, 230, 6, 217, 83, 72, 80, 52, 173, 141, 231, 185, 227, 235, 164, 131, 60,
        119, 1, 9, 223, 36, 197, 233, 11, 160, 52, 18, 70, 114, 68, 131, 2, 14, 65, 249, 176, 61, 173, 222, 133, 188, 114, 213, 13, 49,
        118, 180, 235, 82, 188, 220, 27, 131, 30, 61, 107, 217, 122, 246, 38, 182, 141, 254, 160, 30, 79, 178, 70, 51, 233, 104, 177, 5,
        43, 245, 78, 135, 220, 227, 137, 161, 64, 5, 110, 108, 154, 15, 221, 65, 79, 116, 35, 60, 255, 82, 113, 140, 177, 165, 24, 167, 30,
        213, 192, 180, 97, 204, 180, 188, 135, 176, 157, 218, 91, 19, 205, 112, 253, 9, 238, 136, 120, 252, 11, 163, 27, 181, 204, 23, 74,
        70, 48, 241, 202, 215, 248, 105, 58, 151, 152, 11, 1, 99, 3, 76, 49, 99, 89, 76, 252, 145, 200, 9, 170, 53, 145, 253, 157, 160,
        111, 196, 210, 33, 96, 95, 250, 2, 96, 251, 38, 228, 236, 100, 100, 41, 52, 242, 190, 90, 144, 151, 34, 147, 171, 71, 43, 239, 100,
        115, 177, 174, 189, 99, 72, 184, 185, 134, 42, 42, 151, 7, 251, 107, 225, 108, 23, 92, 185, 27, 28, 123, 106, 20, 156, 151, 179,
        92, 135, 123, 71, 202, 9, 11, 224, 239, 218, 69, 56, 178, 127, 97, 206, 67, 42, 221, 94, 51, 178, 214, 112, 68, 156, 238, 45, 52,
        86, 52, 181, 162, 50, 106, 100, 198, 47, 96, 13, 148, 7, 108, 125, 222, 46, 177, 177, 244, 210, 115, 30, 77, 107, 207, 62, 77, 230,
        209, 67, 242, 208, 78, 55, 245, 16, 234, 173, 111, 228,
    ],
    b_pub_len: 384,
    computed_b_pub: true,
};

// Accessory functions
impl PairSetup {
    #[trace_time]
    fn handle_m1(pin: String, mut decoded: TlvContainer) -> Result<(PairSetup, tlv::Container), tlv::Error> {
        let method = decoded
            .remove(&(Type::Method as u8))
            .ok_or(tlv::Error::Unknown)?;
        let is_mfi = method.len() == 1 && method[0] == 1;

        debug!("pair setup M1: received SRP start request mfi={}", is_mfi);

        if is_mfi {
            debug!("pair setup M1: rejecting MFI mode");
            return Err(tlv::Error::Unavailable);
        }

        // TODO
        // If the accessory is already paired, it must respond with the following TLV items:
        // kTLVType_State <M2>
        // kTLVType_Error <kTLVError_Unavailable>

        // if self.unsuccessful_tries > 100 {
        //     return Err(tlv::Error::MaxTries);
        // }

        // TODO
        // If the accessory is currently performing a PairSetup procedure with a different controller, it must respond with
        // the following TLV items:
        // kTLVType_State <M2>
        // kTLVType_Error <kTLVError_Busy>

        let mut cache = CARPLAY_CACHE;
        if cache.pin != pin || !cfg!(feature = "carplay_pair_setup_cache") {
            cache = PairSetupCache::random(&pin);
        } else {
            debug!("pair setup M1: using cached salt/b/verifier to speed-up pairing flow");
        }

        let b = cache.b;
        let verifier = cache.compute_verifier();
        let b_pub = cache.compute_public_ephemeral();
        // while b_pub.len() < 384 {
        //     b_pub.push(0);
        // }

        Ok((
            PairSetup::M2 {
                salt: cache.salt,
                verifier,
                b,
                b_pub: b_pub.clone(),
            },
            vec![
                Value::State(StepNumber::SrpStartResponse as u8),
                Value::PublicKey(b_pub),
                Value::Salt(cache.salt),
            ],
        ))
    }

    #[trace_time]
    fn handle_m3(
        salt: [u8; 16],
        verifier: Vec<u8>,
        b: [u8; 64],
        b_pub: Vec<u8>,
        mut decoded: TlvContainer,
    ) -> Result<(PairSetup, tlv::Container), tlv::Error> {
        debug!("pair setup M3: received SRP verify request");

        let a_pub = decoded
            .remove(&(Type::PublicKey as u8))
            .ok_or(tlv::Error::Unknown)?;
        let a_proof = decoded
            .remove(&(Type::Proof as u8))
            .ok_or(tlv::Error::Unknown)?;

        let verifier_key = srp_process_server_reply_legacy(&b, &verifier, &a_pub, Some(&b_pub))
            .inspect_err(|e| debug!("pair_setup error at process_reply: {e:?}"))
            .map_err(|_| tlv::Error::Authentication)?;

        let shared_secret = sha512(&verifier_key);

        let b_proof = verify_client_proof(&b_pub, &a_pub, &a_proof, &salt, &shared_secret)
            .inspect_err(|e| debug!("pair_setup error at verify_client_proof: {e:?}"))
            .map_err(|_| tlv::Error::Authentication)?;

        debug!("pair setup M4: sending SRP verify response (no MFI)");
        Ok((
            PairSetup::M4 { shared_secret },
            vec![Value::State(StepNumber::SrpVerifyResponse as u8), Value::Proof(b_proof.to_vec())],
        ))
    }

    #[trace_time]
    fn handle_m5(
        homekit: &dyn HomekitStorage,
        shared_secret: [u8; 64],
        mut decoded: TlvContainer,
    ) -> Result<(PairSetup, tlv::Container), tlv::Error> {
        debug!("pair setup M5: received exchange request");

        let mut data = decoded
            .remove(&(Type::EncryptedData as u8))
            .ok_or(tlv::Error::Unknown)?;

        let encryption_key = hkdf_extract_and_expand(b"Pair-Setup-Encrypt-Salt", &shared_secret, b"Pair-Setup-Encrypt-Info")?;
        let nonce = Self::cipher_nonce(b"PS-Msg05");
        let mut cipher = HomeKitCipher::new(encryption_key);

        let sub_tlv = cipher
            .decrypt(&mut data, &[], nonce)
            .map_err(|_| tlv::Error::Authentication)
            .map(|d| tlv::decode(d))?;

        let device_pairing_id = sub_tlv
            .get(&(Type::Identifier as u8))
            .ok_or(tlv::Error::Unknown)?;
        let device_pubkey_raw = sub_tlv
            .get(&(Type::PublicKey as u8))
            .and_then(|v| key32_from_vec(v))
            .ok_or(tlv::Error::Unknown)?;

        let device_pubkey = create_ed25519_pubkey(&device_pubkey_raw);
        let device_signature = sub_tlv
            .get(&(Type::Signature as u8))
            .ok_or(tlv::Error::Unknown)?;

        let device_x = hkdf_extract_and_expand(
            b"Pair-Setup-Controller-Sign-Salt",
            &shared_secret,
            b"Pair-Setup-Controller-Sign-Info",
        )?;

        let device_info = [&device_x, &device_pairing_id[..], &device_pubkey_raw].concat();

        device_pubkey
            .verify(&device_info, device_signature)
            .map_err(|_| tlv::Error::Authentication)?;

        // if let Some(max_peers) = config.lock().await.max_peers {
        //     if storage.lock().await.count_pairings().await? + 1 > max_peers {
        //         return Err(tlv::Error::MaxPeers);
        //     }
        // }

        // let pairing = Pairing::new(pairing_uuid, Permissions::Admin, device_ltpk.to_bytes());
        // storage.lock().await.save_pairing(&pairing).await?;

        // debug!("pairing: {:?}", &pairing);

        let accessory_x = hkdf_extract_and_expand(b"Pair-Setup-Accessory-Sign-Salt", &shared_secret, b"Pair-Setup-Accessory-Sign-Info")?;

        let device_id = homekit.device_id().to_string();

        let accessory_info = [
            &accessory_x,
            device_id.as_bytes(),
            homekit.device_ed25519_keypair().public_key().as_ref(),
        ]
        .concat();
        let accessory_signature = homekit.device_ed25519_keypair().sign(&accessory_info);

        let mut encoded_sub_tlv = vec![
            Value::Identifier(device_id),
            Value::PublicKey(
                homekit
                    .device_ed25519_keypair()
                    .public_key()
                    .as_ref()
                    .to_vec(),
            ),
            Value::Signature(accessory_signature.as_ref().to_vec()),
        ]
        .encode();

        let nonce = Self::cipher_nonce(b"PS-Msg06");

        let tag = cipher
            .encrypt(&mut encoded_sub_tlv, &[], nonce)
            .map_err(|_| tlv::Error::Unknown)?;
        encoded_sub_tlv.extend_from_slice(&tag);

        debug!("pair setup M6: sending exchange response");

        let device_pairing_id = Uuid::parse_str(str::from_utf8(device_pairing_id)?)?;

        debug!("pair setup completed with controller {:?}", device_pairing_id);

        homekit
            .add_paired(device_pairing_id, device_pubkey_raw)
            .map_err(|err| {
                debug!("add_paired failed: {err:?}");
                tlv::Error::Unknown
            })?;

        Ok((
            PairSetup::Finished { device_pairing_id },
            vec![
                Value::State(StepNumber::ExchangeResponse as u8),
                Value::EncryptedData(encoded_sub_tlv),
            ],
        ))
    }
}

// Controller functions
impl PairSetup {
    #[trace_time]
    pub fn handle_m2(pin: String, mut decoded: TlvContainer) -> Result<(PairSetup, tlv::Container), tlv::Error> {
        debug!("pair setup M2: received SRP start response");

        let salt = decoded
            .remove(&(Type::Salt as u8))
            .ok_or(tlv::Error::Unknown)?;
        let b_pub = decoded
            .remove(&(Type::PublicKey as u8))
            .ok_or(tlv::Error::Unknown)?;

        let rng = SystemRandom::new();
        let mut a = [0u8; 64];
        rng.fill(&mut a).unwrap();

        // let g: &'static SrpGroup = match b_pub.len() {
        //     384 => &G_3072,
        //     256 => &G_2048,
        //     _ => return Err(tlv::Error::Authentication),
        // };

        let a_pub = srp_compute_client_public_ephemeral(&a);
        // while a_pub.len() < 384 {
        //     a_pub.push(0);
        // }

        let verifier_key = srp_process_client_reply(&a, &pin, &salt, &b_pub).map_err(|_| tlv::Error::Authentication)?;

        let shared_secret = sha512(&verifier_key);
        let a_proof = client_proof(&a_pub, &b_pub, &salt, &shared_secret);

        debug!("pair setup M3: sending SRP verify request");

        Ok((
            PairSetup::M3 {
                a_pub: a_pub.clone(),
                a_proof,
                shared_secret,
            },
            vec![
                Value::State(StepNumber::SrpVerifyRequest as u8),
                Value::PublicKey(a_pub),
                Value::Proof(a_proof.to_vec()),
            ],
        ))
    }

    #[trace_time]
    fn handle_m4(
        homekit: &dyn HomekitIdentity,
        a_pub: Vec<u8>,
        a_proof: [u8; 64],
        shared_secret: [u8; 64],

        mut decoded: TlvContainer,
    ) -> Result<(PairSetup, tlv::Container), tlv::Error> {
        debug!("pair setup M4: received SRP verify response");

        let b_proof = decoded
            .remove(&(Type::Proof as u8))
            .ok_or(tlv::Error::Unknown)?;
        verify_server_proof(&a_pub, &a_proof, &shared_secret, &b_proof).map_err(|_| tlv::Error::Authentication)?;

        let encryption_key = hkdf_extract_and_expand(b"Pair-Setup-Encrypt-Salt", &shared_secret, b"Pair-Setup-Encrypt-Info")?;
        let nonce = Self::cipher_nonce(b"PS-Msg05");
        let mut cipher = HomeKitCipher::new(encryption_key);

        let controller_id_str = homekit.device_id().to_string();
        let controller_pubkey = *homekit.device_ed25519_keypair().public_key();

        let device_x = hkdf_extract_and_expand(
            b"Pair-Setup-Controller-Sign-Salt",
            &shared_secret,
            b"Pair-Setup-Controller-Sign-Info",
        )?;
        let device_info = [&device_x, controller_id_str.as_bytes(), controller_pubkey.as_ref()].concat();
        let signature = homekit.device_ed25519_keypair().sign(&device_info);

        let mut sub_tlv = vec![
            Value::Identifier(controller_id_str),
            Value::PublicKey(controller_pubkey.as_ref().to_vec()),
            Value::Signature(signature.as_ref().to_vec()),
        ]
        .encode();

        let tag = cipher
            .encrypt(&mut sub_tlv, &[], nonce)
            .map_err(|_| tlv::Error::Unknown)?;
        sub_tlv.extend_from_slice(&tag);

        debug!("pair setup M5: sending exchange request");

        Ok((
            PairSetup::M5 { shared_secret },
            vec![Value::State(StepNumber::ExchangeRequest as u8), Value::EncryptedData(sub_tlv)],
        ))
    }

    #[trace_time]
    fn handle_m6(
        storage: &dyn HomekitStorage,
        shared_secret: [u8; 64],
        mut decoded: TlvContainer,
    ) -> Result<(PairSetup, tlv::Container), tlv::Error> {
        debug!("pair setup M6: received exchange response");

        let mut data = decoded
            .remove(&(Type::EncryptedData as u8))
            .ok_or(tlv::Error::Unknown)?;

        let encryption_key = hkdf_extract_and_expand(b"Pair-Setup-Encrypt-Salt", &shared_secret, b"Pair-Setup-Encrypt-Info")?;
        let nonce = Self::cipher_nonce(b"PS-Msg06");
        let mut cipher = HomeKitCipher::new(encryption_key);

        let sub_tlv = cipher
            .decrypt(&mut data, &[], nonce)
            .map_err(|_| tlv::Error::Authentication)
            .map(|d| tlv::decode(d))?;

        let accessory_id = sub_tlv
            .get(&(Type::Identifier as u8))
            .ok_or(tlv::Error::Unknown)?;
        let accessory_pubkey_raw = sub_tlv
            .get(&(Type::PublicKey as u8))
            .and_then(|v| key32_from_vec(v))
            .ok_or(tlv::Error::Unknown)?;
        let signature = sub_tlv
            .get(&(Type::Signature as u8))
            .ok_or(tlv::Error::Unknown)?;

        let accessory_pubkey = create_ed25519_pubkey(&accessory_pubkey_raw);

        let accessory_x = hkdf_extract_and_expand(b"Pair-Setup-Accessory-Sign-Salt", &shared_secret, b"Pair-Setup-Accessory-Sign-Info")?;
        let accessory_info = [&accessory_x, &accessory_id[..], &accessory_pubkey_raw].concat();

        accessory_pubkey
            .verify(&accessory_info, signature)
            .map_err(|_| tlv::Error::Authentication)?;

        let accessory_pairing_id = Uuid::parse_str(&String::from_utf8_lossy(accessory_id))?;
        debug!("pair setup completed with accessory {:?}", accessory_pairing_id);

        storage
            .add_paired(accessory_pairing_id, accessory_pubkey_raw)
            .map_err(|_| tlv::Error::Unknown)?;

        Ok((
            PairSetup::Finished {
                device_pairing_id: accessory_pairing_id,
            },
            vec![],
        ))
    }
}

#[test]
fn test_pair_setup_end_to_end() {
    use catplay_tracing::logger::setup_test_logger;

    use crate::HomekitStorageFile;

    setup_test_logger(false);

    let storage_controller = &*HomekitStorageFile::memory();
    let storage_accessory = &*HomekitStorageFile::memory();

    let uuid_controller = storage_controller.device_id();
    let uuid_accessory = storage_accessory.device_id();

    let (controller, payload_m1) = PairSetup::client(CARPLAY_MAGIC_PIN);
    let accessory = PairSetup::server(CARPLAY_MAGIC_PIN);

    // --- M1: Controller -> Accessory ---
    let (accessory, payload_m2) = accessory
        .handle(storage_accessory, &payload_m1)
        .expect("accessory M1 failed");
    // --- M2: Accessory -> Controller ---
    let (controller, payload_m3) = controller
        .handle(storage_controller, &payload_m2)
        .expect("controller M2 failed");
    // --- M3: Controller -> Accessory ---
    let (accessory, payload_m4) = accessory
        .handle(storage_accessory, &payload_m3)
        .expect("accessory M3 failed");
    // --- M4: Accessory -> Controller ---
    let (controller, payload_m5) = controller
        .handle(storage_controller, &payload_m4)
        .expect("controller M4 failed");

    // --- M5: Controller -> Accessory ---
    let (accessory, payload_m6) = accessory
        .handle(storage_accessory, &payload_m5)
        .expect("accessory M5 failed");
    // --- M6: Accessory -> Controller ---
    let (controller, _) = controller
        .handle(storage_controller, &payload_m6)
        .expect("controller M6 failed");

    assert_eq!(
        accessory,
        PairSetup::Finished {
            device_pairing_id: uuid_controller
        }
    );
    assert_eq!(
        controller,
        PairSetup::Finished {
            device_pairing_id: uuid_accessory
        }
    )
}
