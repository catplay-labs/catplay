use bytes::{BufMut, BytesMut};
use catplay_hap::cipher::{HomeKitChaChaNonce, HomeKitCipher};
use catplay_tokio::{BytesMutUtil, Decoder, Encoder};
use log::{debug, error};

// Ethernet MTU 1500 - IPv6 header 40 - TCP header 20 = 1440 bytes (TCP payload/MSS without options).
const AIRPLAY_CIPHER_MTU_MAX: usize = 1440;
const AIRPLAY_CIPHER_OVERHEAD_BYTES: usize = 16 + 2;
const AIRPLAY_CIPHER_HEADER_LEN: usize = 2;

pub struct AirPlayCipherCodec {
    read_cipher: HomeKitCipher,
    write_cipher: HomeKitCipher,
    counter_rx: HomeKitChaChaNonce,
    counter_tx: HomeKitChaChaNonce,
}

impl AirPlayCipherCodec {
    #[allow(unused)]
    pub fn new(shared_secret: [u8; 32], channel_type: AirPlayCipherSaltType, server: bool) -> Self {
        let (mut read_cipher, mut write_cipher) = create_chacha_ciphers(&shared_secret, channel_type, server);

        Self {
            read_cipher,
            write_cipher,
            counter_rx: HomeKitChaChaNonce(0),
            counter_tx: HomeKitChaChaNonce(0),
        }
    }

    pub fn estimate_encrypted_size(&self, unencrypted: usize) -> usize {
        let chunks = unencrypted.div_ceil(1024);
        unencrypted + chunks * AIRPLAY_CIPHER_OVERHEAD_BYTES
    }

    fn encode_mtu_in_place(&mut self, data: &mut BytesMut, mtu: usize) -> RtspResult<()> {
        let max_plaintext = mtu.saturating_sub(AIRPLAY_CIPHER_OVERHEAD_BYTES);
        if max_plaintext == 0 {
            return Err(RtspError::PayloadTooBig(0, 0));
        }
        if max_plaintext > u16::MAX as usize {
            return Err(RtspError::PayloadTooBig(max_plaintext, u16::MAX as usize));
        }
        if data.is_empty() {
            return Ok(());
        }

        let plaintext_len = data.len();
        let chunks = plaintext_len.div_ceil(max_plaintext);
        let encrypted_len = plaintext_len + chunks * AIRPLAY_CIPHER_OVERHEAD_BYTES;
        data.resize(encrypted_len, 0);

        let nonce_base = self.counter_tx.0;
        let mut plain_cursor = plaintext_len;
        let mut encrypted_cursor = encrypted_len;
        for idx in (0..chunks).rev() {
            let len = max_plaintext.min(plain_cursor);
            plain_cursor -= len;

            let chunk_encrypted_len = len + AIRPLAY_CIPHER_OVERHEAD_BYTES;
            encrypted_cursor -= chunk_encrypted_len;
            let chunk_start = encrypted_cursor;
            let payload_start = chunk_start + AIRPLAY_CIPHER_HEADER_LEN;
            let tag_start = payload_start + len;

            let aad = u16::to_le_bytes(len as u16);
            data.copy_within(plain_cursor..plain_cursor + len, payload_start);
            data[chunk_start..payload_start].copy_from_slice(&aad);

            let nonce = HomeKitChaChaNonce(nonce_base + idx as u64);
            let tag = self
                .write_cipher
                .encrypt(&mut data[payload_start..payload_start + len], &aad, nonce)?;
            data[tag_start..tag_start + 16].copy_from_slice(tag.as_ref());
        }
        self.counter_tx.0 += chunks as u64;

        Ok(())
    }

    pub fn encode_in_place(&mut self, data: &mut BytesMut) -> RtspResult<()> {
        self.encode_mtu_in_place(data, AIRPLAY_CIPHER_MTU_MAX)
    }

    pub fn encode(&mut self, mut item: BytesMut, dst: &mut BytesMut) -> RtspResult<()> {
        self.encode_in_place(&mut item)?;
        dst.put(item);
        Ok(())
    }

    pub fn decode(&mut self, src: &mut BytesMut) -> RtspResult<Option<BytesMut>> {
        BytesMutUtil::ensure_writable(src, AIRPLAY_CIPHER_MTU_MAX);

        if src.len() < 2 {
            return Ok(None);
        }

        let len = u16::from_le_bytes([src[0], src[1]]) as usize;
        let tx_mtu_plain_limit = AIRPLAY_CIPHER_MTU_MAX - AIRPLAY_CIPHER_OVERHEAD_BYTES;
        if len > tx_mtu_plain_limit {
            error!(
                "AirPlayCipherCodec::decode PayloadTooBig len={} limit={} src_len={} counter_rx={}",
                len,
                tx_mtu_plain_limit,
                src.len(),
                self.counter_rx.0
            );
            return Err(RtspError::PayloadTooBig(len, tx_mtu_plain_limit));
        }

        let packet_size = len + AIRPLAY_CIPHER_OVERHEAD_BYTES;

        if src.len() < packet_size {
            BytesMutUtil::ensure_writable(src, packet_size);
            debug!(
                "Still waiting for encrypted frame sized {}+18b overhead; have {} buffered",
                len,
                src.len()
            );
            return Ok(None); // not enough data yet
        }

        // Split out full packet: [aad | ciphertext | tag]
        let mut packet = src.split_to(packet_size);
        let aad: [u8; 2] = [packet[0], packet[1]];
        debug_assert_eq!(packet_size, packet.len(), "invalid packet len");

        let nonce = self.counter_rx.advance()?;
        self.read_cipher
            .decrypt(&mut packet[AIRPLAY_CIPHER_HEADER_LEN..], &aad, nonce)?;

        packet.copy_within(AIRPLAY_CIPHER_HEADER_LEN..AIRPLAY_CIPHER_HEADER_LEN + len, 0);
        packet.truncate(len);
        Ok(Some(packet))
    }

    pub fn decrypt_into_prefix_in_place(&mut self, src: &mut BytesMut, plaintext_prefix_len: usize) -> RtspResult<usize> {
        BytesMutUtil::ensure_writable(src, AIRPLAY_CIPHER_MTU_MAX);

        let mut read = plaintext_prefix_len.min(src.len());
        let mut write = plaintext_prefix_len.min(src.len());
        let total = src.len();

        while total.saturating_sub(read) >= AIRPLAY_CIPHER_HEADER_LEN {
            let len = u16::from_le_bytes([src[read], src[read + 1]]) as usize;
            let tx_mtu_plain_limit = AIRPLAY_CIPHER_MTU_MAX - AIRPLAY_CIPHER_OVERHEAD_BYTES;
            if len > tx_mtu_plain_limit {
                error!(
                    "AirPlayCipherCodec::decrypt_into_prefix_in_place PayloadTooBig len={} limit={} prefix={} read={} write={} total={} available={} counter_rx={}",
                    len,
                    tx_mtu_plain_limit,
                    plaintext_prefix_len,
                    read,
                    write,
                    total,
                    total.saturating_sub(read),
                    self.counter_rx.0
                );
                return Err(RtspError::PayloadTooBig(len, tx_mtu_plain_limit));
            }

            let packet_size = AIRPLAY_CIPHER_OVERHEAD_BYTES + len;
            if total - read < packet_size {
                debug!(
                    "Still waiting for encrypted frame sized {}+18b overhead; have {} buffered",
                    len,
                    total - read
                );
                break;
            }

            let aad = [src[read], src[read + 1]];
            let nonce = self.counter_rx.advance()?;
            self.read_cipher
                .decrypt(&mut src[read + AIRPLAY_CIPHER_HEADER_LEN..read + packet_size], &aad, nonce)?;

            src.copy_within(read + AIRPLAY_CIPHER_HEADER_LEN..read + AIRPLAY_CIPHER_HEADER_LEN + len, write);
            read += packet_size;
            write += len;
        }

        if read != write {
            let remainder = total - read;
            if remainder > 0 {
                src.copy_within(read..total, write);
            }
            src.truncate(write + remainder);
        }

        Ok(write)
    }
}

use crate::{
    cipher::{AirPlayCipherSaltType, create_chacha_ciphers},
    rtsp_frame::{RtspError, RtspResult},
};

impl Decoder for AirPlayCipherCodec {
    type Item = BytesMut;
    type Error = RtspError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.decode(src)
    }
}

impl Encoder<BytesMut> for AirPlayCipherCodec {
    type Error = RtspError;

    fn encode(&mut self, item: BytesMut, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.encode(item, dst)
    }
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};

    use crate::cipher::AirPlayCipherSaltType;

    use super::AirPlayCipherCodec;

    #[test]
    fn test_encoder_decoder() {
        let shared_secret: [u8; 32] = [
            123, 249, 137, 93, 180, 161, 216, 87, 56, 87, 55, 160, 67, 38, 150, 159, 37, 59, 4, 204, 14, 26, 146, 221, 170, 109, 99, 204,
            98, 138, 6, 56,
        ];
        let mut encoder = AirPlayCipherCodec::new(shared_secret, AirPlayCipherSaltType::Control, true);
        let mut decoder = AirPlayCipherCodec::new(shared_secret, AirPlayCipherSaltType::Control, false);

        let mut encoded = BytesMut::new();
        let mut decoded = BytesMut::new();

        let test_string = "HelloCipher";
        let test_data = test_string.repeat(200);
        let data = test_data.as_bytes();

        let src = BytesMut::from(data);
        println!("Pre-encode: {:?}", src);

        encoder.encode(src, &mut encoded).unwrap();

        println!("Encoded: {:?}", encoded);

        assert!(test_string.as_bytes() != &encoded[..test_string.len()]);

        loop {
            let chunk = decoder.decode(&mut encoded).unwrap();
            match chunk {
                None => break,
                Some(c) => decoded.put_slice(&c),
            }
        }

        println!("Decoded: {:?}", decoded);
        assert_eq!(decoded, BytesMut::from(data));
    }
}
