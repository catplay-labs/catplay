use catplay_hap::cipher::{HomeKitChaChaNonce, HomeKitCipher};
use catplay_tokio::AlignedOffsetBuf;
use log::trace;

use crate::{
    cipher::{AirPlayCipherSaltType, create_chacha_ciphers},
    rtp::{RTP_BUFFER_PAD, RTP_HEADER_SIZE, RTP_PACKET_MAX, RtpHeader, RtpPacketBorrow},
    rtsp_frame::{RtspError, RtspResult},
};

pub const RTP_PACKET_OVERHEAD_CHACHA: usize = RTP_HEADER_SIZE + AUTH_TAG_SIZE + NONCE_SIZE;

const AUTH_TAG_SIZE: usize = 16;
const NONCE_SIZE: usize = 8;

pub type RtpChaChaBuffer = AlignedOffsetBuf<RTP_PACKET_MAX, RTP_BUFFER_PAD>;

/// ```[RTP Header 12B][encrypted RTP payload N b][auth tag 16B][nonce 8B]```
pub struct RtpChaChaDecoder {
    read_cipher: HomeKitCipher,
    write_cipher: HomeKitCipher,

    counter: HomeKitChaChaNonce,
}

impl RtpHeader {
    fn _as_legacy_aad(&self) -> [u8; 12] {
        // If (MainAudio || AltAudio) && clientOSBuildVersion <= 13A1
        self.to_buf()
    }

    fn as_modern_aad(&self) -> [u8; 8] {
        let combined: u64 = ((self.timestamp as u64) << 32) | (self.ssrc as u64);
        combined.to_be_bytes()
    }
}

impl RtpChaChaDecoder {
    pub fn new(shared_secret: [u8; 32], stream_connection_id: u64, receiver: bool) -> Self {
        let (read_cipher, write_cipher) =
            create_chacha_ciphers(&shared_secret, AirPlayCipherSaltType::DataStream { stream_connection_id }, receiver);

        Self {
            read_cipher,
            write_cipher,
            counter: HomeKitChaChaNonce(0),
        }
    }

    /// Decrypts `src` payload in-place, where `src` is a UDP-received buffer.
    ///
    /// Returns RTP header and payload.
    pub fn decode_rtp_payload<'a>(&mut self, src: &'a mut [u8]) -> RtspResult<RtpPacketBorrow<'a>> {
        let n = src.len();

        if n > RTP_PACKET_MAX {
            return Err(RtspError::PayloadTooBig(n, RTP_PACKET_MAX));
        }

        if n < RTP_PACKET_OVERHEAD_CHACHA {
            return Err(RtspError::PayloadTooShort(n, RTP_PACKET_OVERHEAD_CHACHA));
        }

        let header = RtpHeader::from_buf(src);
        let Some(header) = header else {
            return Err(RtspError::RtpInvalidHeader);
        };

        let aad = header.as_modern_aad();
        let nonce = HomeKitChaChaNonce(u64::from_le_bytes(src[n - NONCE_SIZE..].try_into().unwrap()));
        let ciphertext = &mut src[RTP_HEADER_SIZE..n - NONCE_SIZE];

        #[cfg(debug_assertions)]
        trace!("Audio decrypt: aad: {aad:?} header: {header:?} buf_size: {n}");

        if ciphertext.is_empty() {
            return Err(RtspError::Empty);
        }

        let payload = self.read_cipher.decrypt(ciphertext, &aad, nonce)?;
        Ok(RtpPacketBorrow::new(header, payload))
    }

    /// Encrypts `src` plaintext in-place, where the `plaintext` data starts at a 12-byte offset.
    ///
    /// `src` needs to be sized at least `12 + plaintext.len() + 16 + 8`.
    ///
    /// Returns a slice which represents UDP packet ready to be sent.
    pub fn encode_rtp_payload_in_place<'a>(
        &mut self,
        output: &'a mut [u8],
        plaintext_len: usize,
        header: &RtpHeader,
    ) -> RtspResult<&'a mut [u8]> {
        let total_len = RTP_PACKET_OVERHEAD_CHACHA + plaintext_len;
        let mut i = 0;

        if output.len() < total_len {
            return Err(RtspError::RtpTooBigForBuffer(plaintext_len, output.len()));
        }

        let header_buf = header.to_buf();
        output[i..i + RTP_HEADER_SIZE].copy_from_slice(&header_buf);

        i += RTP_HEADER_SIZE;

        let aad = header.as_modern_aad();
        let nonce = self.counter.advance()?;
        let nonce_bytes = &nonce.0.to_le_bytes();
        debug_assert!(nonce_bytes.len() == NONCE_SIZE);
        let ciphertext = &mut output[i..i + plaintext_len];

        i += plaintext_len;

        let tag = self.write_cipher.encrypt(ciphertext, &aad, nonce)?;
        debug_assert!(tag.len() == AUTH_TAG_SIZE);

        output[i..i + AUTH_TAG_SIZE].copy_from_slice(&tag);
        i += AUTH_TAG_SIZE;
        output[i..i + NONCE_SIZE].copy_from_slice(nonce_bytes);

        Ok(&mut output[..total_len])
    }
}

#[cfg(test)]
mod tests {
    use crate::rtp::AsRtpPacket;

    use super::*;

    #[test]
    fn test_encode_decode_rtp_chacha() {
        let key_bytes = [0x42u8; 32];
        let scid = 1234u64;
        let mut codec = RtpChaChaDecoder::new(key_bytes, scid, false);

        let original_payload = b"this is secret RTP payload";
        let mut buffer = [0u8; 1024];

        // Plaintext needs to start exactly at offset 12 before encrypting
        buffer[12..12 + original_payload.len()].copy_from_slice(original_payload);

        let header = RtpHeader {
            version: 2,
            padding: false,
            extension: false,
            csrc_count: 0,
            marker: false,
            payload_type: 96,
            sequence_number: 1234,
            timestamp: 0xDEADBEEF,
            ssrc: 0x11223344,
        };

        let encoded = codec
            .encode_rtp_payload_in_place(&mut buffer, original_payload.len(), &header)
            .expect("encryption failed");

        let mut codec = RtpChaChaDecoder::new(key_bytes, scid, true);

        let decrypted = codec
            .decode_rtp_payload(encoded)
            .expect("decryption failed");
        assert_eq!(decrypted.payload(), original_payload);
        assert_eq!(decrypted.header(), &header);
    }
}
