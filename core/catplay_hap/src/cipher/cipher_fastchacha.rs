use log::{debug, warn};

use crate::{
    cipher::{HomeKitChaChaNonce, HomeKitCipherError},
    fast_chacha::{FastChaCha20, FastPoly1305, is_asm_available_chacha20},
    hkdf_extract_and_expand,
};

pub struct HomeKitCipherFast {
    key: [u8; 32],
    warned_poly: bool,
    progressive_tag_state: Option<ProgressiveTagState>,
}

struct ProgressiveTagState {
    poly: FastPoly1305,
    nonce_bytes: [u8; 12],
    // aad: Vec<u8>,
    expected_total_len: usize,
    expected_payload_len: usize,
    processed_ciphertext_len: usize,
}

impl HomeKitCipherFast {
    pub fn new(key: [u8; 32]) -> Self {
        if !is_asm_available_chacha20() {
            warn!("Using fallback ChaCha20 without ASM");
        }

        Self {
            key,
            warned_poly: false,
            progressive_tag_state: None,
        }
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
        let data_and_tag_len = data_and_tag.len();
        self.decrypt_progressive(data_and_tag, aad, nonce, 0, data_and_tag_len, data_and_tag_len)
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
        let expected_payload_len = expected_total_len - TAG_SIZE;
        let nonce_bytes = nonce_to_bytes(nonce.0);

        if start_payload < end_payload {
            let ciphertext_chunk = &data_and_tag[start_payload..end_payload];
            if let Err(err) = self.update_progressive_tag_state(
                aad,
                &nonce_bytes,
                expected_total_len,
                expected_payload_len,
                start_payload,
                ciphertext_chunk,
            ) {
                self.progressive_tag_state = None;
                return Err(err);
            }
        }

        if start_payload < end_payload {
            apply_keystream_at_offset(self.key, nonce_bytes, start_payload, &mut data_and_tag[start_payload..end_payload]);
        }

        if final_chunk {
            let tag = &data_and_tag[payload_len..];
            let expected_tag =
                match self.finalize_progressive_tag_state(aad, &nonce_bytes, expected_total_len, expected_payload_len, start_payload) {
                    Ok(tag) => tag,
                    Err(err) => {
                        self.progressive_tag_state = None;
                        return Err(err);
                    }
                };

            self.progressive_tag_state = None;
            if !constant_time_eq(tag, &expected_tag) {
                debug!(
                    "ChaCha20-Poly1305 signature mismatch nonce={} aad_len={} payload_len={} total_len={} start_payload={} end_payload={} actual_tag={:02x?} expected_tag={:02x?}",
                    nonce.0,
                    aad.len(),
                    payload_len,
                    data_and_tag_len,
                    start_payload,
                    end_payload,
                    tag,
                    expected_tag
                );
                return Err(HomeKitCipherError::InvalidSignature);
            }
        }

        Ok(&mut data_and_tag[start_payload..end_payload])
    }

    pub fn encrypt(&mut self, data: &mut [u8], aad: &[u8], nonce: HomeKitChaChaNonce) -> Result<[u8; 16], HomeKitCipherError> {
        let nonce_bytes = nonce_to_bytes(nonce.0);
        let mut chacha = FastChaCha20::new_with_counter(self.key, nonce_bytes, 1);
        chacha.apply_keystream(data);
        Ok(self.compute_tag(data, aad, &nonce_bytes))
    }

    fn compute_tag(&mut self, ciphertext: &[u8], aad: &[u8], nonce_bytes: &[u8; 12]) -> [u8; 16] {
        let mut poly = self.new_poly1305(nonce_bytes);

        poly.update(aad);
        update_padding(&mut poly, aad.len());
        poly.update(ciphertext);
        update_padding(&mut poly, ciphertext.len());
        poly.update(&(aad.len() as u64).to_le_bytes());
        poly.update(&(ciphertext.len() as u64).to_le_bytes());
        poly.finalize()
    }

    fn new_poly1305(&mut self, nonce_bytes: &[u8; 12]) -> FastPoly1305 {
        let mut poly_key_stream = [0u8; 64];
        let mut poly_chacha = FastChaCha20::new_with_counter(self.key, *nonce_bytes, 0);
        poly_chacha.keystream_only(&mut poly_key_stream);

        let mut poly_key = [0u8; 32];
        poly_key.copy_from_slice(&poly_key_stream[..32]);

        let poly = FastPoly1305::new(&poly_key);
        if !self.warned_poly && poly.is_fallback() {
            self.warned_poly = true;
            warn!("Using fallback Poly1305 implementation without ASM");
        }
        poly
    }

    fn update_progressive_tag_state(
        &mut self,
        aad: &[u8],
        nonce_bytes: &[u8; 12],
        expected_total_len: usize,
        expected_payload_len: usize,
        start_payload: usize,
        ciphertext_chunk: &[u8],
    ) -> Result<(), HomeKitCipherError> {
        if start_payload == 0 {
            self.initialize_progressive_tag_state(aad, nonce_bytes, expected_total_len, expected_payload_len);
        } else if self.progressive_tag_state.is_none() {
            return Err(HomeKitCipherError::UnexpectedDecryptedLength);
        }

        let state = self.progressive_tag_state.as_mut().unwrap();
        if state.nonce_bytes != *nonce_bytes
            || state.expected_total_len != expected_total_len
            || state.expected_payload_len != expected_payload_len
            // || state.aad.as_slice() != aad
            || state.processed_ciphertext_len != start_payload
        {
            return Err(HomeKitCipherError::UnexpectedDecryptedLength);
        }

        state.poly.update(ciphertext_chunk);
        state.processed_ciphertext_len += ciphertext_chunk.len();
        Ok(())
    }

    fn finalize_progressive_tag_state(
        &mut self,
        aad: &[u8],
        nonce_bytes: &[u8; 12],
        expected_total_len: usize,
        expected_payload_len: usize,
        start_payload: usize,
    ) -> Result<[u8; 16], HomeKitCipherError> {
        if self.progressive_tag_state.is_none() {
            if start_payload != 0 {
                return Err(HomeKitCipherError::UnexpectedDecryptedLength);
            }

            self.initialize_progressive_tag_state(aad, nonce_bytes, expected_total_len, expected_payload_len);
        }

        let state = self.progressive_tag_state.take().unwrap();
        if state.nonce_bytes != *nonce_bytes
            || state.expected_total_len != expected_total_len
            || state.expected_payload_len != expected_payload_len
            // || state.aad.as_slice() != aad
            || state.processed_ciphertext_len != expected_payload_len
        {
            return Err(HomeKitCipherError::UnexpectedDecryptedLength);
        }

        let mut poly = state.poly;
        update_padding(&mut poly, expected_payload_len);
        poly.update(&(aad.len() as u64).to_le_bytes());
        poly.update(&(expected_payload_len as u64).to_le_bytes());
        Ok(poly.finalize())
    }

    fn initialize_progressive_tag_state(
        &mut self,
        aad: &[u8],
        nonce_bytes: &[u8; 12],
        expected_total_len: usize,
        expected_payload_len: usize,
    ) {
        let mut poly = self.new_poly1305(nonce_bytes);
        poly.update(aad);
        update_padding(&mut poly, aad.len());

        self.progressive_tag_state = Some(ProgressiveTagState {
            poly,
            nonce_bytes: *nonce_bytes,
            // aad: aad.to_vec(),
            expected_total_len,
            expected_payload_len,
            processed_ciphertext_len: 0,
        });
    }
}

fn nonce_to_bytes(nonce: u64) -> [u8; 12] {
    let mut out = [0u8; 12];
    out[4..].copy_from_slice(&nonce.to_le_bytes());
    out
}

fn update_padding(poly: &mut FastPoly1305, len: usize) {
    const ZERO_PAD: [u8; 16] = [0; 16];
    let rem = len % 16;
    if rem != 0 {
        poly.update(&ZERO_PAD[..16 - rem]);
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut diff = 0u8;
    for (&lhs, &rhs) in a.iter().zip(b.iter()) {
        diff |= lhs ^ rhs;
    }
    diff == 0
}

fn apply_keystream_at_offset(key: [u8; 32], nonce: [u8; 12], stream_offset: usize, data: &mut [u8]) {
    if data.is_empty() {
        return;
    }

    let mut chacha = FastChaCha20::new_with_counter(key, nonce, 1 + (stream_offset / 64) as u32);
    let in_block_offset = stream_offset % 64;
    let mut processed = 0usize;

    if in_block_offset != 0 {
        let mut block = [0u8; 64];
        chacha.keystream_only(&mut block);
        let take = (64 - in_block_offset).min(data.len());
        for i in 0..take {
            data[i] ^= block[in_block_offset + i];
        }
        processed = take;
    }

    if processed < data.len() {
        chacha.apply_keystream(&mut data[processed..]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decrypt_progressive_short_non_final_chunk_is_noop() {
        let key = [0x10u8; 32];
        let aad = b"aad-short";
        let nonce = HomeKitChaChaNonce(6);
        let mut short = *b"tiny-chunk";
        let short_len = short.len();

        let mut dec = HomeKitCipherFast::new(key);
        let ret = dec
            .decrypt_progressive(&mut short, aad, nonce, 0, short_len, 16)
            .unwrap();

        assert!(ret.is_empty());
        assert_eq!(&short, b"tiny-chunk");
    }

    #[test]
    fn decrypt_progressive_rejects_end_past_expected_total_len() {
        let key = [0x12u8; 32];
        let aad = b"aad-bounds";
        let nonce = HomeKitChaChaNonce(8);
        let mut buf = [0u8; 32];

        let mut dec = HomeKitCipherFast::new(key);
        let err = dec
            .decrypt_progressive(&mut buf, aad, nonce, 0, 17, 16)
            .unwrap_err();

        assert!(matches!(err, HomeKitCipherError::UnexpectedDecryptedLength));
    }

    #[test]
    fn decrypt_progressive_decrypts_in_chunks_and_verifies_tag_on_final_chunk() {
        let key = [0x11u8; 32];
        let aad = b"aad-chunked";
        let nonce = HomeKitChaChaNonce(7);
        let plaintext = b"this is a longer message for chunked decryption verification".to_vec();

        let mut enc = HomeKitCipherFast::new(key);
        let mut ciphertext = plaintext.clone();
        let tag = enc.encrypt(&mut ciphertext, aad, nonce).unwrap();

        let mut frame = ciphertext;
        frame.extend_from_slice(&tag);

        let mut dec = HomeKitCipherFast::new(key);
        let cut1 = 9usize;
        let cut2 = 31usize;
        let full_len = frame.len();

        dec.decrypt_progressive(&mut frame, aad, nonce, 0, cut1, full_len)
            .unwrap();
        dec.decrypt_progressive(&mut frame, aad, nonce, cut1, cut2, full_len)
            .unwrap();
        let decrypted_last = dec
            .decrypt_progressive(&mut frame, aad, nonce, cut2, full_len, full_len)
            .unwrap();

        assert_eq!(decrypted_last, &plaintext[cut2..]);
        assert_eq!(&frame[..plaintext.len()], plaintext.as_slice());
    }

    #[test]
    fn decrypt_progressive_handles_non_aligned_chunks_across_64_byte_boundary() {
        let key = [0x22u8; 32];
        let aad = b"aad-boundary";
        let nonce = HomeKitChaChaNonce(11);

        let plaintext: Vec<u8> = (0..160)
            .map(|i| (i as u8).wrapping_mul(7).wrapping_add(3))
            .collect();

        let mut enc = HomeKitCipherFast::new(key);
        let mut ciphertext = plaintext.clone();
        let tag = enc.encrypt(&mut ciphertext, aad, nonce).unwrap();

        let mut frame = ciphertext;
        frame.extend_from_slice(&tag);
        let full_len = frame.len();

        let mut dec = HomeKitCipherFast::new(key);
        let cuts = [5usize, 37usize, 73usize, 109usize, 143usize];
        let mut prev = 0usize;
        for &cut in &cuts {
            dec.decrypt_progressive(&mut frame, aad, nonce, prev, cut, full_len)
                .unwrap();
            prev = cut;
        }
        let last = dec
            .decrypt_progressive(&mut frame, aad, nonce, prev, full_len, full_len)
            .unwrap();

        assert_eq!(last, &plaintext[prev..]);
        assert_eq!(&frame[..plaintext.len()], plaintext.as_slice());
    }
}
