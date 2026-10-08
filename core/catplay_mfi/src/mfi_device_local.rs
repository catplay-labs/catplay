use p256::{
    ecdsa::{Signature, SigningKey, signature::hazmat::PrehashSigner},
    pkcs8::DecodePrivateKey,
};

use crate::{MfiDevice, MfiI2cError, MfiResult};

/// Software MFi backend for a DER PKCS#7 certificate and a DER PKCS#8 P-256 key.
/// The certificate is returned verbatim; the challenge is a 32-byte SHA-256 digest.
pub struct MfiDeviceLocal {
    certificate: Vec<u8>,
    key: SigningKey,
}

impl MfiDeviceLocal {
    pub fn new(certificate_p7b: &[u8], identity_pk8: &[u8]) -> MfiResult<Self> {
        if certificate_p7b.is_empty() {
            return Err(MfiI2cError::UnexpectedSize(0));
        }

        let key = SigningKey::from_pkcs8_der(identity_pk8).map_err(|_| MfiI2cError::InvalidLocalKey)?;
        Ok(Self {
            certificate: certificate_p7b.to_vec(),
            key,
        })
    }
}

impl MfiDevice for MfiDeviceLocal {
    fn read_certificate(&self) -> MfiResult<Vec<u8>> {
        Ok(self.certificate.clone())
    }

    fn generate_challenge_response(&self, challenge: &[u8]) -> MfiResult<Vec<u8>> {
        if challenge.len() != 32 {
            return Err(MfiI2cError::InvalidChallengeSize(challenge.len()));
        }

        let signature: Signature = self
            .key
            .sign_prehash(challenge)
            .map_err(|_| MfiI2cError::LocalSigning)?;
        Ok(signature.to_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use p256::ecdsa::{Signature, signature::hazmat::PrehashVerifier};

    use super::*;

    // Synthetic PKCS#8 fixture: P-256 scalar 1. No production credential is stored here.
    fn test_pk8() -> Vec<u8> {
        let mut der = vec![
            0x30, 0x41, 0x02, 0x01, 0x00, 0x30, 0x13, 0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01, 0x06, 0x08, 0x2a, 0x86, 0x48,
            0xce, 0x3d, 0x03, 0x01, 0x07, 0x04, 0x27, 0x30, 0x25, 0x02, 0x01, 0x01, 0x04, 0x20,
        ];
        der.extend_from_slice(&[0; 31]);
        der.push(1);
        der
    }

    #[test]
    fn certificate_is_returned_verbatim_and_signature_is_raw_p256() {
        let cert = b"sample DER PKCS#7 bytes";
        let device = MfiDeviceLocal::new(cert, &test_pk8()).unwrap();
        assert_eq!(device.read_certificate().unwrap(), cert);

        let digest = [0x42; 32];
        let response = device.generate_challenge_response(&digest).unwrap();
        assert_eq!(response.len(), 64);
        let signature = Signature::from_slice(&response).unwrap();
        device
            .key
            .verifying_key()
            .verify_prehash(&digest, &signature)
            .unwrap();
    }

    #[test]
    fn rejects_invalid_inputs() {
        assert!(MfiDeviceLocal::new(&[], &test_pk8()).is_err());
        assert!(MfiDeviceLocal::new(b"cert", b"not a key").is_err());
        let device = MfiDeviceLocal::new(b"cert", &test_pk8()).unwrap();
        assert!(matches!(
            device.generate_challenge_response(&[0; 31]),
            Err(MfiI2cError::InvalidChallengeSize(31))
        ));
    }
}
