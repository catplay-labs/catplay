use ring::aead::{Aad, CHACHA20_POLY1305, LessSafeKey, Nonce, UnboundKey};

use crate::{
    cipher::{HomeKitChaChaNonce, HomeKitCipherError},
    hkdf_extract_and_expand,
};

#[allow(unused)]
struct Key32;
impl ring::hkdf::KeyType for Key32 {
    fn len(&self) -> usize {
        32
    }
}

/*impl NonceSequence for HomeKitChaChaNonce {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        let mut nonce = [0u8; 12];
        nonce[4..].copy_from_slice(&self.0.to_le_bytes());
        self.0 += 1;
        Ok(Nonce::assume_unique_for_key(nonce))
    }
}*/

impl From<HomeKitChaChaNonce> for Nonce {
    fn from(value: HomeKitChaChaNonce) -> Self {
        let mut nonce = [0u8; 12];
        nonce[4..].copy_from_slice(&value.0.to_le_bytes());
        Nonce::assume_unique_for_key(nonce)
    }
}

pub struct HomeKitCipherRing {
    cipher: LessSafeKey,
}

impl HomeKitCipherRing {
    pub fn new(key: [u8; 32]) -> Self {
        let cipher = LessSafeKey::new(UnboundKey::new(&CHACHA20_POLY1305, &key).unwrap());
        Self { cipher }
    }

    pub fn compute_key(shared_secret: &[u8; 32], salt: &[u8], info: &[u8]) -> [u8; 32] {
        hkdf_extract_and_expand(salt, shared_secret, info).unwrap()
    }

    pub fn decrypt<'a>(
        &mut self,
        data_and_tag: &'a mut [u8],
        aad: &[u8],
        nonce: HomeKitChaChaNonce,
    ) -> Result<&'a mut [u8], HomeKitCipherError> {
        const TAG_SIZE: usize = 16;
        let data_and_tag_len = data_and_tag.len();
        if data_and_tag_len < TAG_SIZE {
            return Err(HomeKitCipherError::PayloadTooSmall);
        }

        let payload = self
            .cipher
            .open_in_place(nonce.into(), Aad::from(&aad), data_and_tag)
            .map_err(|_| HomeKitCipherError::InvalidSignature)?;
        if payload.len() + TAG_SIZE != data_and_tag_len {
            return Err(HomeKitCipherError::UnexpectedDecryptedLength);
        }
        Ok(payload)
    }

    pub fn decrypt_progressive<'a>(
        &mut self,
        data_and_tag: &'a mut [u8],
        aad: &[u8],
        nonce: HomeKitChaChaNonce,
        start: usize,
        end: usize,
        expected_total_len: usize,
    ) -> Result<&'a mut [u8], HomeKitCipherError> {
        const TAG_SIZE: usize = 16;
        let data_and_tag_len = data_and_tag.len();
        let final_chunk = end == expected_total_len;

        if data_and_tag_len < TAG_SIZE {
            if !final_chunk {
                return Ok(&mut data_and_tag[..0]);
            }
            return Err(HomeKitCipherError::PayloadTooSmall);
        }
        if start > end || end > data_and_tag_len || end > expected_total_len {
            return Err(HomeKitCipherError::UnexpectedDecryptedLength);
        }
        if final_chunk && data_and_tag_len != expected_total_len {
            return Err(HomeKitCipherError::UnexpectedDecryptedLength);
        }

        let payload_len = data_and_tag_len - TAG_SIZE;
        let start_payload = start.min(payload_len);
        let end_payload = end.min(payload_len);

        if final_chunk {
            let payload = self.decrypt(data_and_tag, aad, nonce)?;
            return Ok(&mut payload[start_payload..end_payload]);
        }

        Ok(&mut data_and_tag[start_payload..end_payload])
    }

    pub fn encrypt(&mut self, data: &mut [u8], aad: &[u8], nonce: HomeKitChaChaNonce) -> Result<[u8; 16], HomeKitCipherError> {
        let tag = self
            .cipher
            .seal_in_place_separate_tag(nonce.into(), Aad::from(&aad), data)
            .map_err(|_| HomeKitCipherError::Unspecified)?;
        let tag = tag.as_ref().try_into().unwrap();
        Ok(tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decrypt_progressive_short_non_final_chunk_is_noop() {
        let key = [0x34u8; 32];
        let aad = b"ring-short";
        let nonce = HomeKitChaChaNonce(22);
        let mut short = *b"tiny-chunk";
        let short_len = short.len();

        let mut dec = HomeKitCipherRing::new(key);
        let ret = dec
            .decrypt_progressive(&mut short, aad, nonce, 0, short_len, 16)
            .unwrap();

        assert!(ret.is_empty());
        assert_eq!(&short, b"tiny-chunk");
    }

    #[test]
    fn decrypt_progressive_rejects_end_past_expected_total_len() {
        let key = [0x35u8; 32];
        let aad = b"ring-bounds";
        let nonce = HomeKitChaChaNonce(23);
        let mut buf = [0u8; 32];

        let mut dec = HomeKitCipherRing::new(key);
        let err = dec
            .decrypt_progressive(&mut buf, aad, nonce, 0, 17, 16)
            .unwrap_err();

        assert!(matches!(err, HomeKitCipherError::UnexpectedDecryptedLength));
    }

    #[test]
    fn decrypt_progressive_only_decrypts_on_final_chunk() {
        let key = [0x33u8; 32];
        let aad = b"ring-aad";
        let nonce = HomeKitChaChaNonce(21);
        let plaintext = b"ring progressive decrypt should happen only on final chunk".to_vec();

        let mut enc = HomeKitCipherRing::new(key);
        let mut ciphertext = plaintext.clone();
        let tag = enc
            .encrypt(&mut ciphertext, aad, HomeKitChaChaNonce(nonce.0))
            .unwrap();

        let mut frame = ciphertext.clone();
        frame.extend_from_slice(&tag);
        let full_len = frame.len();

        let mut dec = HomeKitCipherRing::new(key);
        let cut1 = 10usize;
        let cut2 = 29usize;

        let c1 = dec
            .decrypt_progressive(&mut frame, aad, HomeKitChaChaNonce(nonce.0), 0, cut1, full_len)
            .unwrap()
            .to_vec();
        assert_eq!(c1, ciphertext[..cut1].to_vec());
        assert_eq!(&frame[..plaintext.len()], ciphertext.as_slice());

        let c2 = dec
            .decrypt_progressive(&mut frame, aad, HomeKitChaChaNonce(nonce.0), cut1, cut2, full_len)
            .unwrap()
            .to_vec();
        assert_eq!(c2, ciphertext[cut1..cut2].to_vec());
        assert_eq!(&frame[..plaintext.len()], ciphertext.as_slice());

        let last = dec
            .decrypt_progressive(&mut frame, aad, HomeKitChaChaNonce(nonce.0), cut2, full_len, full_len)
            .unwrap();
        assert_eq!(last, &plaintext[cut2..]);
        assert_eq!(&frame[..plaintext.len()], plaintext.as_slice());
    }
}
