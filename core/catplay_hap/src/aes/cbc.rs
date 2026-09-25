use aes::Aes128;
use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit, generic_array::GenericArray};

use crate::cipher::HomeKitCipherError;

/// Thin crypto wrapper so we can swap implementations per-arch later.
///
/// `ring` does not expose a public AES-CBC API, so this backend currently uses `aes`.
pub struct Aes128Cbc {
    cipher: Aes128,
    iv: [u8; 16],
}

impl Aes128Cbc {
    pub fn new(key: &[u8; 16], iv: &[u8; 16]) -> Self {
        Self {
            cipher: Aes128::new_from_slice(key).expect("valid AES-128 key"),
            iv: *iv,
        }
    }

    pub fn encrypt_in_place(&self, data: &mut [u8]) -> Result<(), HomeKitCipherError> {
        if !data.len().is_multiple_of(16) {
            return Err(HomeKitCipherError::AesCbcBadBlock);
        }

        let mut prev = self.iv;
        for chunk in data.chunks_mut(16) {
            for (b, p) in chunk.iter_mut().zip(prev.iter()) {
                *b ^= *p;
            }

            let mut block = GenericArray::clone_from_slice(chunk);
            self.cipher.encrypt_block(&mut block);
            chunk.copy_from_slice(&block);
            prev.copy_from_slice(chunk);
        }

        Ok(())
    }

    pub fn decrypt_in_place(&self, data: &mut [u8]) -> Result<(), HomeKitCipherError> {
        if !data.len().is_multiple_of(16) {
            return Err(HomeKitCipherError::AesCbcBadBlock);
        }

        let mut prev = self.iv;
        for chunk in data.chunks_mut(16) {
            let mut ciphertext = [0u8; 16];
            ciphertext.copy_from_slice(chunk);

            let mut block = GenericArray::clone_from_slice(&ciphertext);
            self.cipher.decrypt_block(&mut block);

            for (dst, (&dec, &iv)) in chunk.iter_mut().zip(block.iter().zip(prev.iter())) {
                *dst = dec ^ iv;
            }

            prev = ciphertext;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Aes128Cbc;

    #[test]
    fn cbc_encrypt_decrypt_roundtrip_changes_intermediate() {
        let key = [0x33u8; 16];
        let iv = [0x44u8; 16];
        let plaintext = *b"0123456789abcdefFEDCBA9876543210";

        let mut encrypted = plaintext;
        let cbc = Aes128Cbc::new(&key, &iv);
        cbc.encrypt_in_place(&mut encrypted)
            .expect("CBC encrypt should succeed");

        assert_ne!(encrypted, plaintext);

        let mut decrypted = encrypted;
        cbc.decrypt_in_place(&mut decrypted)
            .expect("CBC decrypt should succeed");

        assert_eq!(decrypted, plaintext);
    }
}
