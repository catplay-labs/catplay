use alloc::vec::Vec;
use bytes::{Buf, BytesMut};

use super::{Header, IAP2_HEADER_START, Packet};
use crate::{IAP2_HEADER_SIZE, PacketOrDetect, iap2_check_checksum_fast, iap2_gen_checksum_fast};

const IAP2_HANDSHAKE: &[u8; 6] = &[0xFF, 0x55, 0x02, 0x00, 0xEE, 0x10];

#[derive(Clone, Copy)]
pub struct PacketCoder {
    // For testing purposes, throw exceptions if the buffer does not contain all expected data
    // For production usage, just return "None" as a signal to retry with more data.
    pub throw_on_incomplete_packets: bool,
    seen_handshake: bool,
}

#[derive(thiserror::Error, Debug, PartialEq, Clone)]
pub enum PacketCoderError {
    #[error("Packet read too short, try again with more data")]
    TooShort,

    #[error("Packet header has invalid magic")]
    HeaderInvalidMagic,
    #[error("Link packet header checksum invalid (expected {expected:#02x} vs actual {actual:#02x})")]
    HeaderChecksumInvalid { expected: u8, actual: u8 },
    #[error("Invalid iAP2 handshake received!")]
    InvalidHandshakeReceived,
    #[error("Not enough bytes for header got {received} vs expected 9")]
    HeaderNotEnoughBytes { received: usize },
    #[error("Link packet header not parsable")]
    HeaderNotParsable,
    #[error("Expected length field to be between 9..65525, got {length}")]
    InvalidLength { length: usize },
    #[error("Not enough payload bytes: got {received} vs expected {expected}")]
    PayloadTooShort { received: usize, expected: usize },
    #[error("Payload checksum invalid (expected {expected:#02x} vs actual {actual:#02x})")]
    PayloadChecksumInvalid { expected: u8, actual: u8 },
}

impl PacketCoder {
    pub fn encode(&mut self, item: PacketOrDetect, dst: &mut BytesMut) -> Result<(), PacketCoderError> {
        match item {
            PacketOrDetect::Packet(item) => {
                let header = item.header.to_bytes();
                let payload = item.payload;

                dst.extend_from_slice(&header);
                if let Some(payload) = payload {
                    dst.extend_from_slice(&payload);
                    dst.extend_from_slice(&[iap2_gen_checksum_fast(&payload)]);
                }
                Ok(())
            }
            PacketOrDetect::Detect => {
                dst.extend_from_slice(IAP2_HANDSHAKE);
                Ok(())
            }
        }
    }

    pub fn decode(&mut self, src: &mut BytesMut) -> Result<Option<PacketOrDetect>, PacketCoderError> {
        if src.len() < 2 {
            return Ok(None);
        }

        if u16::from_be_bytes([src[0], src[1]]) != IAP2_HEADER_START {
            if src.len() >= IAP2_HANDSHAKE.len() {
                return match &src[0..6] == IAP2_HANDSHAKE {
                    true => {
                        self.seen_handshake = true;
                        src.advance(6);
                        Ok(Some(PacketOrDetect::Detect))
                    }
                    false => Err(PacketCoderError::InvalidHandshakeReceived),
                };
            }

            return Ok(None);
        }

        let mut readable = src.len();

        // The header is 9 bytes
        if readable < IAP2_HEADER_SIZE {
            if self.throw_on_incomplete_packets {
                return Err(PacketCoderError::HeaderNotEnoughBytes { received: readable });
            }
            return Ok(None); // not enough data
        }

        let header = Header::parse(src)?;
        readable -= IAP2_HEADER_SIZE;

        if header.length < IAP2_HEADER_SIZE as u16 {
            // Need at least 9 header bytes
            return Err(PacketCoderError::InvalidLength {
                length: header.length as _,
            });
        }

        let payload_size = header.length as usize - IAP2_HEADER_SIZE;

        if readable < payload_size {
            if self.throw_on_incomplete_packets {
                return Err(PacketCoderError::PayloadTooShort {
                    received: readable,
                    expected: header.length as _,
                });
            }
            return Ok(None); // try again later (not enough data)
        }

        let data = src.split_to(header.length as usize); // also advances the cursor
        let full_payload: &[u8] = &data[IAP2_HEADER_SIZE..]; // payload including checksum

        // "The Payload Checksum byte is present if and only if Payload Data is present."
        // 0 bytes left - this packet type has no payload
        // 1 byte left - a checksum for an empty payload array [which always has a value of 0]
        // >1 bytes - a payload followed by a checksum

        if full_payload.is_empty() {
            return Ok(Some(PacketOrDetect::Packet(Packet { header, payload: None })));
        }

        if full_payload.len() == 1 && full_payload[0] == 0 {
            return Ok(Some(PacketOrDetect::Packet(Packet {
                header,
                payload: Vec::new().into(),
            })));
        }

        let real_payload: &[u8] = &full_payload[..full_payload.len() - 1]; // without checksum byte
        if !iap2_check_checksum_fast(full_payload) {
            let expected: u8 = iap2_gen_checksum_fast(real_payload);
            let actual = full_payload[full_payload.len() - 1];

            return Err(PacketCoderError::PayloadChecksumInvalid { expected, actual });
        }

        let packet = Packet {
            header,
            payload: Some(real_payload.to_vec()),
        };
        Ok(Some(PacketOrDetect::Packet(packet)))
    }
}

impl PacketCoder {
    pub fn new(throw_on_incomplete_packets: bool) -> Self {
        Self {
            throw_on_incomplete_packets,
            seen_handshake: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Header, LSPPayload, LSPSession, LinkControl, PacketCoder, PacketOrDetect, SessionType, packet::*};
    use bytes::{BufMut, BytesMut};
    use catplay_util::ModSeq;

    #[test]
    fn test_empty_payload_case() {
        let mut coder = PacketCoder::new(true);
        let packet = Packet::new_ack(ModSeq(7), ModSeq(6), 2, Some(Vec::new()));

        let mut encoded = BytesMut::with_capacity(16);
        coder
            .encode(PacketOrDetect::Packet(packet.clone()), &mut encoded)
            .expect("encode failed");
        assert_eq!(&encoded[..], &[0xFF, 0x5A, 0x00, 0x0A, 0x40, 0x07, 0x06, 0x02, 0x4E, 0x00]);

        let decoded = coder.decode(&mut encoded).expect("decode failed");
        assert!(decoded.is_some(), "Packet was not decoded");

        let decoded_packet = match decoded.unwrap() {
            PacketOrDetect::Packet(packet) => packet,
            _ => panic!("Not a packet"),
        };

        assert_eq!(decoded_packet.header, packet.header);
        assert_eq!(decoded_packet.payload, Some(Vec::new()));
    }

    #[test]
    fn test_no_payload_case() {
        let mut coder = PacketCoder::new(true);
        let packet = Packet::new_ack(ModSeq(7), ModSeq(6), 2, None);

        let mut encoded = BytesMut::with_capacity(16);
        coder
            .encode(PacketOrDetect::Packet(packet.clone()), &mut encoded)
            .expect("encode failed");
        assert_eq!(&encoded[..], &[0xFF, 0x5A, 0x00, 0x09, 0x40, 0x07, 0x06, 0x02, 0x4F]);

        let decoded = coder.decode(&mut encoded).expect("decode failed");
        assert!(decoded.is_some(), "Packet was not decoded");

        let decoded_packet = match decoded.unwrap() {
            PacketOrDetect::Packet(packet) => packet,
            _ => panic!("Not a packet"),
        };

        assert_eq!(decoded_packet.header, packet.header);
        assert_eq!(decoded_packet.payload, None);
    }

    #[test]
    fn test_link_sync_payload_response() {
        let mut decoder = PacketCoder::new(true);

        let packet_bytes: [u8; 23] = [
            0xFF, 0x5A, 0x00, 0x17, 0xC0, 0x6C, 0x19, 0x00, 0x4B, 0x01, 0x7F, 0xFF, 0xFF, 0x05, 0xDC, 0x00, 0x49, 0x1E, 0x03, 0x01, 0x00,
            0x01, 0x35,
        ];

        let mut buf: BytesMut = BytesMut::with_capacity(packet_bytes.len());
        buf.put_slice(&packet_bytes);

        let decoded = decoder.decode(&mut buf).expect("decode failed");

        assert!(decoded.is_some(), "Packet was not decoded");
        let packet: Packet = match decoded.unwrap() {
            PacketOrDetect::Packet(packet) => packet,
            _ => panic!("Not a packet"),
        };
        println!("{:?}", packet);

        assert_eq!(
            packet.header,
            Header {
                length: 23,
                control: LinkControl::from_bits_truncate(192),
                seq: ModSeq(108),
                ack: ModSeq(25),
                session_id: 0
            }
        );

        let payload = &packet.payload.clone().expect("no payload");

        assert_eq!(payload, &[1, 127, 255, 255, 5, 220, 0, 73, 30, 3, 1, 0, 1]);

        // Reserialize
        let mut dst = BytesMut::with_capacity(1024);
        decoder
            .encode(PacketOrDetect::Packet(packet.clone()), &mut dst)
            .expect("encode failed");

        let decoded_again = decoder.decode(&mut dst).expect("decode failed");
        assert!(decoded_again.is_some(), "Packet was not decoded");
        let packet_again: Packet = match decoded_again.unwrap() {
            PacketOrDetect::Packet(packet) => packet,
            _ => panic!("Not a packet"),
        };

        assert_eq!(packet_again, packet);

        // Now check LinkSynchronizationPayload content
        let lsp = LSPPayload::from_bytes(payload).expect("lsp decode failed");
        print!("{:?}", lsp);

        assert_eq!(
            LSPPayload {
                max_outgoing: 127,
                max_len: 65535,
                retransmission_timeout: 1500,
                max_ack: 3,
                ack_timeout: 73,
                max_retransmissions: 30,
                sessions: vec![LSPSession {
                    id: 1,
                    session_type: SessionType::Control,
                    version: 1
                }]
            },
            lsp
        );
    }
}
