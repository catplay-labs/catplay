use alloc::vec::Vec;
use bytes::{Buf, BytesMut};

use super::{Header, Packet};
use crate::{IAP2_HEADER_SIZE, PacketOrDetect, iap2_check_checksum_fast, iap2_gen_checksum_fast};

const IAP2_HANDSHAKE: &[u8; 6] = &[0xFF, 0x55, 0x02, 0x00, 0xEE, 0x10];

#[derive(Clone, Copy)]
pub struct PacketCoder {
    // Transports that enforce strict completion boundaries between link layer packets are
    // referred to here as non-streaming transports.
    //
    // Apply a stricter set of checks to them that detect issues early - data in the buffer
    // after decode cycle must end exactly at packet's boundary.
    //
    // An example of a transport that's always a streaming transport is Bluetooth.
    pub streaming_mode: bool,
}

enum FrameStart {
    #[cfg(feature = "lingo")]
    Legacy,
    Iap2,
    #[cfg(not(feature = "lingo"))]
    Detect,
}

#[derive(thiserror::Error, Debug, PartialEq, Clone)]
pub enum PacketCoderError {
    #[error("Packet read too short, try again with more data")]
    TooShort,

    #[error("Packet header has invalid magic")]
    HeaderInvalidMagic,
    #[error("Link packet header checksum invalid (expected {expected:#02x} vs actual {actual:#02x})")]
    HeaderChecksumInvalid { expected: u8, actual: u8 },
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
    #[cfg(feature = "lingo")]
    #[error(transparent)]
    Legacy(#[from] catplay_lingo::LinkError),
    #[cfg(feature = "lingo")]
    #[error("invalid iAP1 packet checksum")]
    LegacyChecksumInvalid,
}

impl PacketCoder {
    fn rewind(src: &mut BytesMut) -> Option<FrameStart> {
        loop {
            let Some(start) = src
                .windows(2)
                .position(|window| window == [0xff, 0x5a] || window[0] == 0x55)
            else {
                // A single trailing byte may be the first half of the next marker.
                let keep = matches!(src.last(), Some(0xff | 0x55));
                src.advance(src.len() - usize::from(keep));
                return None;
            };
            src.advance(start);

            if src[0] == 0xff {
                return Some(FrameStart::Iap2);
            }

            #[cfg(feature = "lingo")]
            return Some(FrameStart::Legacy);

            #[cfg(not(feature = "lingo"))]
            {
                if src.len() < IAP2_HANDSHAKE.len() - 1 {
                    return None;
                }
                if src[..IAP2_HANDSHAKE.len() - 1] == IAP2_HANDSHAKE[1..] {
                    return Some(FrameStart::Detect);
                }
                src.advance(1);
            }
        }
    }

    #[cfg(feature = "lingo")]
    fn decode_legacy(src: &mut BytesMut) -> Result<Option<PacketOrDetect>, PacketCoderError> {
        let Some((frame, frame_len)) = catplay_lingo::FrameCodec::decode(src)? else {
            return Ok(None);
        };
        let consumed = frame_len + 1;
        if src.len() < consumed {
            return Ok(None);
        }
        if !iap2_check_checksum_fast(&src[1..consumed]) {
            return Err(PacketCoderError::LegacyChecksumInvalid);
        }
        // General.DetectSequence (0x00/0xEE) is the iAP2 detection frame.
        // Classify only after the iAP1 length and checksum have been validated.
        let detect = frame.body == [0x00, 0xee];
        let body = frame.body.to_vec();
        src.advance(consumed);
        if detect {
            return Ok(Some(PacketOrDetect::Detect));
        }
        Ok(Some(PacketOrDetect::Legacy(body)))
    }

    pub fn encode(&mut self, item: PacketOrDetect, dst: &mut BytesMut) -> Result<(), PacketCoderError> {
        match item {
            #[cfg(feature = "lingo")]
            PacketOrDetect::Legacy(body) => {
                let mut encoded = Vec::new();
                catplay_lingo::FrameCodec::encode(&body, &mut catplay_lingo::Writer::new(&mut encoded))?;

                // We could prefix this forcefully by an 0xFF sync byte
                // However this may not be expected by some legacy accessories
                // and formally should only happen for iAP1-over-UART
                // (a dead standard that's unlikely to be our target)
                let checksum = iap2_gen_checksum_fast(&encoded[1..]);
                dst.extend_from_slice(&encoded);
                dst.extend_from_slice(&[checksum]);
                Ok(())
            }
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
        match Self::rewind(src) {
            #[cfg(feature = "lingo")]
            Some(FrameStart::Legacy) => return Self::decode_legacy(src),
            #[cfg(not(feature = "lingo"))]
            Some(FrameStart::Detect) => {
                src.advance(IAP2_HANDSHAKE.len() - 1);
                return Ok(Some(PacketOrDetect::Detect));
            }
            Some(FrameStart::Iap2) => {}
            None => return Ok(None),
        }

        let mut readable = src.len();

        // The header is 9 bytes
        if readable < IAP2_HEADER_SIZE {
            if !self.streaming_mode {
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
            if !self.streaming_mode {
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
            streaming_mode: !throw_on_incomplete_packets,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Header, LSPPayload, LSPSession, LinkControl, PacketCoder, PacketCoderError, PacketOrDetect, SessionType, packet::*};
    use bytes::{BufMut, BytesMut};
    use catplay_util::ModSeq;

    fn iap2_coder(strict: bool) -> PacketCoder {
        PacketCoder::new(strict)
    }

    #[test]
    fn test_empty_payload_case() {
        let mut coder = iap2_coder(true);
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
        let mut coder = iap2_coder(true);
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
        let mut decoder = iap2_coder(true);

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

    fn encode(packet: PacketOrDetect) -> Vec<u8> {
        let mut buffer = BytesMut::new();
        PacketCoder::new(false).encode(packet, &mut buffer).unwrap();
        buffer.to_vec()
    }

    #[test]
    fn decodes_fragmented_packets_with_leading_inter_packet_and_trailing_padding() {
        // Real link packets, including zero payload bytes and a zero payload checksum.
        let first = PacketOrDetect::Packet(Packet::new_ack(ModSeq(7), ModSeq(6), 1, Some(vec![0; 8])));
        let second = PacketOrDetect::Packet(Packet::new_ack(ModSeq(8), ModSeq(7), 1, None));
        let mut wire = vec![0; 5];
        wire.extend(encode(first.clone()));
        wire.extend_from_slice(&[0; 9]);
        wire.extend(encode(second.clone()));
        wire.extend_from_slice(&[0; 7]);

        // Exercise every split, including the middle of the header and payload.
        for split in 0..=wire.len() {
            let mut coder = iap2_coder(false);
            let mut buffer = BytesMut::new();
            let mut decoded = Vec::new();

            for chunk in [&wire[..split], &wire[split..]] {
                buffer.extend_from_slice(chunk);

                while let Some(packet) = coder.decode(&mut buffer).unwrap() {
                    decoded.push(packet);
                }
            }

            assert_eq!(decoded, vec![first.clone(), second.clone()], "split={split}");
            assert!(buffer.is_empty(), "split={split}");
        }
    }

    #[test]
    fn decodes_maximum_length_packet_with_trailing_hid_padding() {
        let payload = vec![0xa5; u16::MAX as usize - crate::IAP2_HEADER_SIZE - 1];
        let packet = PacketOrDetect::Packet(Packet::new_ack(ModSeq(7), ModSeq(6), 1, Some(payload)));
        let encoded = encode(packet.clone());
        assert_eq!(encoded.len(), u16::MAX as usize);

        for strict in [false, true] {
            let mut coder = iap2_coder(strict);
            let mut buffer = BytesMut::from(encoded.as_slice());
            buffer.extend_from_slice(&[0; 64]);

            assert_eq!(coder.decode(&mut buffer), Ok(Some(packet.clone())));
            assert_eq!(&buffer[..], &[0; 64]);
            assert_eq!(coder.decode(&mut buffer), Ok(None));
            assert!(buffer.is_empty());
        }
    }

    #[test]
    fn consumes_all_zero_padding_even_with_strict_incomplete_packet_checks() {
        let mut coder = iap2_coder(true);

        for length in [0, 1, 2, 512] {
            let mut buffer = BytesMut::from(vec![0; length].as_slice());
            assert_eq!(coder.decode(&mut buffer), Ok(None));
            assert!(buffer.is_empty());
        }
    }

    #[test]
    fn scanner_skips_noise_before_iap2_packet() {
        let packet = PacketOrDetect::Packet(Packet::new_ack(ModSeq(7), ModSeq(6), 1, None));
        let mut buffer = BytesMut::from(&[0, 1, 0xff, 0x54, 2, 0][..]);
        buffer.extend(encode(packet.clone()));
        let mut coder = iap2_coder(false);
        assert_eq!(coder.decode(&mut buffer), Ok(Some(packet)));
        assert!(buffer.is_empty());
    }

    #[test]
    fn rewind_keeps_split_iap2_magic() {
        let packet = PacketOrDetect::Packet(Packet::new_ack(ModSeq(7), ModSeq(6), 1, None));
        let encoded = encode(packet.clone());
        let mut coder = iap2_coder(false);
        let mut buffer = BytesMut::from(&[0, 0, 0xff][..]);
        assert_eq!(coder.decode(&mut buffer), Ok(None));
        assert_eq!(&buffer[..], &[0xff]);
        buffer.extend_from_slice(&encoded[1..]);
        assert_eq!(coder.decode(&mut buffer), Ok(Some(packet)));
        assert!(buffer.is_empty());
    }

    #[test]
    fn invalid_iap2_header_is_not_skipped() {
        let packet = PacketOrDetect::Packet(Packet::new_ack(ModSeq(7), ModSeq(6), 1, None));
        let mut encoded = encode(packet);
        *encoded.last_mut().unwrap() ^= 1;
        let mut buffer = BytesMut::from(encoded.as_slice());
        let mut coder = iap2_coder(false);

        assert!(matches!(
            coder.decode(&mut buffer),
            Err(PacketCoderError::HeaderChecksumInvalid { .. })
        ));
        assert_eq!(&buffer[..], &encoded);
        assert!(matches!(
            coder.decode(&mut buffer),
            Err(PacketCoderError::HeaderChecksumInvalid { .. })
        ));
    }

    #[test]
    fn unpadded_detection_still_works() {
        let detect = encode(PacketOrDetect::Detect);
        let mut coder = iap2_coder(false);
        let mut buffer = BytesMut::from(detect.as_slice());
        assert_eq!(coder.decode(&mut buffer), Ok(Some(PacketOrDetect::Detect)));
    }

    #[test]
    fn padded_detection_and_link_packets_work_across_every_split() {
        let packet = PacketOrDetect::Packet(Packet::new_ack(ModSeq(7), ModSeq(6), 1, Some(vec![0; 8])));
        let mut wire = vec![0; 3];
        wire.extend(encode(PacketOrDetect::Detect));
        wire.extend_from_slice(&[0; 5]);
        wire.extend(encode(packet.clone()));
        wire.extend_from_slice(&[0; 4]);
        wire.extend(encode(PacketOrDetect::Detect));
        wire.extend_from_slice(&[0; 3]);

        for split in 0..=wire.len() {
            let mut coder = iap2_coder(false);
            let mut buffer = BytesMut::new();
            let mut decoded = Vec::new();

            for chunk in [&wire[..split], &wire[split..]] {
                buffer.extend_from_slice(chunk);

                while let Some(value) = coder.decode(&mut buffer).unwrap() {
                    decoded.push(value);
                }
            }

            assert_eq!(
                decoded,
                vec![PacketOrDetect::Detect, packet.clone(), PacketOrDetect::Detect],
                "split={split}"
            );
            assert!(buffer.is_empty());
        }
    }

    #[test]
    #[cfg(feature = "lingo")]
    fn fragmented_padded_detection_rejects_an_invalid_tail() {
        let detect = encode(PacketOrDetect::Detect);

        for split in 2..detect.len() {
            let mut coder = iap2_coder(false);
            let mut buffer = BytesMut::from(&[0; 3][..]);
            buffer.extend_from_slice(&detect[..split]);
            assert_eq!(coder.decode(&mut buffer), Ok(None));

            buffer.extend_from_slice(&detect[split..]);
            *buffer.last_mut().unwrap() ^= 1;
            let mut expected = detect[1..].to_vec();
            *expected.last_mut().unwrap() ^= 1;
            assert_eq!(coder.decode(&mut buffer), Err(PacketCoderError::LegacyChecksumInvalid));
            assert_eq!(&buffer[..], &expected);
        }
    }

    #[cfg(feature = "lingo")]
    fn iap1_wire(body: &[u8]) -> Vec<u8> {
        let mut wire = vec![0x55];
        if body.len() <= u8::MAX as usize {
            wire.push(body.len() as u8);
        } else {
            wire.push(0);
            wire.extend_from_slice(&(body.len() as u16).to_be_bytes());
        }
        wire.extend_from_slice(body);
        wire.push(crate::iap2_gen_checksum_fast(&wire[1..]));
        wire
    }

    #[cfg(feature = "lingo")]
    #[test]
    fn iap1_body_is_returned_unchanged_across_padding_and_splits() {
        let first = iap1_wire(&[4, 0x12, 0x34, 0x56, 0x78, 0x9a]);
        let second = iap1_wire(&[0, 0x38, 0, 1]);
        let mut wire = vec![0; 3];
        wire.extend_from_slice(&first);
        wire.extend_from_slice(&[0; 4]);
        wire.push(0xff); // Optional iAP1 sync byte is accepted on input.
        wire.extend_from_slice(&second);
        wire.extend_from_slice(&[0; 2]);

        for split in 0..=wire.len() {
            let mut coder = PacketCoder::new(false);
            let mut buffer = BytesMut::new();
            let mut decoded = Vec::new();
            for chunk in [&wire[..split], &wire[split..]] {
                buffer.extend_from_slice(chunk);
                while let Some(packet) = coder.decode(&mut buffer).unwrap() {
                    decoded.push(packet);
                }
            }
            assert_eq!(
                decoded,
                vec![
                    PacketOrDetect::Legacy(vec![4, 0x12, 0x34, 0x56, 0x78, 0x9a]),
                    PacketOrDetect::Legacy(vec![0, 0x38, 0, 1]),
                ],
                "split={split}"
            );
            assert!(buffer.is_empty(), "split={split}");
        }
    }

    #[cfg(feature = "lingo")]
    #[test]
    fn iap1_extended_length_and_hard_errors() {
        let mut body = vec![4, 0, 1];
        body.resize(256, 0);
        let mut buffer = BytesMut::from(iap1_wire(&body).as_slice());
        assert_eq!(PacketCoder::new(false).decode(&mut buffer), Ok(Some(PacketOrDetect::Legacy(body))));
        assert!(buffer.is_empty());

        let mut bad_checksum = iap1_wire(&[0, 0]);
        *bad_checksum.last_mut().unwrap() ^= 1;
        let mut buffer = BytesMut::from(bad_checksum.as_slice());
        let mut coder = PacketCoder::new(false);
        assert_eq!(coder.decode(&mut buffer), Err(PacketCoderError::LegacyChecksumInvalid));
        assert_eq!(&buffer[..], &bad_checksum);
        assert_eq!(coder.decode(&mut buffer), Err(PacketCoderError::LegacyChecksumInvalid));

        let mut short_command = BytesMut::from(&[0x55, 1, 4, 0xfb][..]);
        assert_eq!(
            coder.decode(&mut short_command),
            Err(PacketCoderError::Legacy(catplay_lingo::LinkError::TruncatedCommand))
        );
        assert_eq!(&short_command[..], &[0x55, 1, 4, 0xfb]);

        let mut noncanonical = BytesMut::from(&[0x55, 0, 0, 3, 4, 0, 1, 0xf8][..]);
        assert_eq!(
            coder.decode(&mut noncanonical),
            Err(PacketCoderError::Legacy(catplay_lingo::LinkError::InvalidLength))
        );
        assert_eq!(&noncanonical[..], &[0x55, 0, 0, 3, 4, 0, 1, 0xf8]);
    }

    #[cfg(feature = "lingo")]
    #[test]
    fn scanner_selects_nearest_frame_magic_each_time() {
        let body = vec![0, 0x38, 0, 1];
        assert_eq!(encode(PacketOrDetect::Legacy(body.clone())), iap1_wire(&body));
        let packet = PacketOrDetect::Packet(Packet::new_ack(ModSeq(7), ModSeq(6), 1, None));
        let next_packet = PacketOrDetect::Packet(Packet::new_ack(ModSeq(8), ModSeq(7), 1, None));
        let mut wire = vec![0; 3];
        wire.extend(encode(packet.clone()));
        wire.extend_from_slice(&[0; 2]);
        wire.extend(iap1_wire(&body));
        wire.extend_from_slice(&[0; 2]);
        wire.extend(encode(PacketOrDetect::Detect));
        wire.extend(encode(next_packet.clone()));

        for split in 0..=wire.len() {
            let mut coder = PacketCoder::new(false);
            let mut buffer = BytesMut::new();
            let mut decoded = Vec::new();
            for chunk in [&wire[..split], &wire[split..]] {
                buffer.extend_from_slice(chunk);
                while let Some(value) = coder.decode(&mut buffer).unwrap() {
                    decoded.push(value);
                }
            }
            assert_eq!(
                decoded,
                vec![
                    packet.clone(),
                    PacketOrDetect::Legacy(body.clone()),
                    PacketOrDetect::Detect,
                    next_packet.clone()
                ],
                "split={split}"
            );
            assert!(buffer.is_empty(), "split={split}");
        }
    }

    #[cfg(feature = "lingo")]
    #[test]
    fn invalid_detect_frame_is_a_hard_error() {
        let mut wire = iap1_wire(&[0, 0xee]);
        *wire.last_mut().unwrap() ^= 1;
        let mut buffer = BytesMut::from(wire.as_slice());
        let mut coder = PacketCoder::new(false);

        assert_eq!(coder.decode(&mut buffer), Err(PacketCoderError::LegacyChecksumInvalid));
        assert_eq!(&buffer[..], &wire);

        *buffer.last_mut().unwrap() ^= 1;
        assert_eq!(coder.decode(&mut buffer), Ok(Some(PacketOrDetect::Detect)));
        assert!(buffer.is_empty());
    }

    #[cfg(not(feature = "lingo"))]
    #[test]
    fn scanner_ignores_legacy_without_lingo_feature() {
        let packet = PacketOrDetect::Packet(Packet::new_ack(ModSeq(7), ModSeq(6), 1, None));
        let mut wire = vec![0xff, 0x55, 2, 0, 0x38, 0xc6];
        wire.extend(encode(PacketOrDetect::Detect));
        wire.extend(encode(packet.clone()));

        for split in 0..=wire.len() {
            let mut coder = PacketCoder::new(false);
            let mut buffer = BytesMut::new();
            let mut decoded = Vec::new();
            for chunk in [&wire[..split], &wire[split..]] {
                buffer.extend_from_slice(chunk);
                while let Some(value) = coder.decode(&mut buffer).unwrap() {
                    decoded.push(value);
                }
            }
            assert_eq!(decoded, vec![PacketOrDetect::Detect, packet.clone()], "split={split}");
            assert!(buffer.is_empty(), "split={split}");
        }
    }
}
