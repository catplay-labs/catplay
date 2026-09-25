use catplay_hap::aes::Aes128Cbc;

use crate::{
    rtp::{RTP_HEADER_SIZE, RTP_PACKET_MAX, RtpHeader, RtpPacketBorrow},
    rtsp_frame::{RtspError, RtspResult},
};

pub struct RtpAesCbcDecoder {
    cipher: Aes128Cbc,
}

impl RtpAesCbcDecoder {
    pub fn new(key: &[u8; 16], iv: &[u8; 16]) -> Self {
        Self {
            cipher: Aes128Cbc::new(key, iv),
        }
    }

    /// Decrypts `src` payload in-place, where `src` is a UDP-received buffer.
    ///
    /// Layout: `[RTP header 12B][AES-CBC encrypted payload][optional trailing bytes]`
    pub fn decode_rtp_payload<'a>(&self, src: &'a mut [u8]) -> RtspResult<RtpPacketBorrow<'a>> {
        let n = src.len();

        if n > RTP_PACKET_MAX {
            return Err(RtspError::PayloadTooBig(n, RTP_PACKET_MAX));
        }

        if n < RTP_HEADER_SIZE {
            return Err(RtspError::PayloadTooShort(n, RTP_HEADER_SIZE));
        }

        let header = RtpHeader::from_buf(src).ok_or(RtspError::RtpInvalidHeader)?;
        let payload = &mut src[RTP_HEADER_SIZE..];
        if payload.is_empty() {
            return Err(RtspError::Empty);
        }

        let encrypted_len = payload.len() / 16 * 16;
        if encrypted_len > 0 {
            self.cipher
                .decrypt_in_place(&mut payload[..encrypted_len])?;
        }

        Ok(RtpPacketBorrow::new(header, payload))
    }

    /// Encrypts `src` plaintext in-place, where the `plaintext` data starts at a 12-byte offset.
    ///
    /// `src` needs to be sized at least `12 + plaintext.len()`.
    ///
    /// Returns a slice which represents UDP packet ready to be sent.
    pub fn encode_rtp_payload_in_place<'a>(
        &self,
        output: &'a mut [u8],
        plaintext_len: usize,
        header: &RtpHeader,
    ) -> RtspResult<&'a mut [u8]> {
        let total_len = RTP_HEADER_SIZE + plaintext_len;
        if total_len > RTP_PACKET_MAX {
            return Err(RtspError::PayloadTooBig(total_len, RTP_PACKET_MAX));
        }

        if output.len() < total_len {
            return Err(RtspError::RtpTooBigForBuffer(plaintext_len, output.len()));
        }

        let header_buf = header.to_buf();
        output[..RTP_HEADER_SIZE].copy_from_slice(&header_buf);

        let payload = &mut output[RTP_HEADER_SIZE..total_len];
        let encrypted_len = payload.len() / 16 * 16;
        if encrypted_len > 0 {
            self.cipher
                .encrypt_in_place(&mut payload[..encrypted_len])?;
        }

        // "The remaining bytes are just copied unencrypted"
        Ok(&mut output[..total_len])
    }
}

#[cfg(test)]
mod tests {
    use crate::rtp::AsRtpPacket;

    use super::*;

    #[test]
    fn test_encode_decode_rtp_aes_cbc() {
        let key = [0x12u8; 16];
        let iv = [0x34u8; 16];
        let cipher = RtpAesCbcDecoder::new(&key, &iv);

        let original_payload = *b"0123456789abcdef0123456789ABCDEFTAIL";
        let mut buffer = [0u8; 1500];
        buffer[RTP_HEADER_SIZE..RTP_HEADER_SIZE + original_payload.len()].copy_from_slice(&original_payload);

        let header = RtpHeader {
            version: 2,
            padding: false,
            extension: false,
            csrc_count: 0,
            marker: false,
            payload_type: 96,
            sequence_number: 77,
            timestamp: 0x01020304,
            ssrc: 0x11223344,
        };

        let encoded = cipher
            .encode_rtp_payload_in_place(&mut buffer, original_payload.len(), &header)
            .expect("encryption failed");

        let decrypted = cipher
            .decode_rtp_payload(encoded)
            .expect("decryption failed");
        assert_eq!(decrypted.payload(), &original_payload);
        assert_eq!(decrypted.header(), &header);
    }
}
