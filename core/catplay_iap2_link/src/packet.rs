use alloc::vec::Vec;
use catplay_util::{ModSeq, ModSeq8};

use crate::{FileTransferPayload, PayloadDecodable};

use super::{Header, LSPPayload, LinkControl};
use core::fmt;

/// Represents a single iAP2 link-layer packet as transmitted over a transport.
///
/// ## Transport Invariant
///
/// Exactly one transport-level transfer completion corresponds to exactly one
/// iAP2 link-layer packet. While multiple transfers may be in flight concurrently,
/// packet boundaries are never crossed or coalesced.
///
/// This invariant is required for compatibility with strict automotive head units
/// (e.g. most QNX-based systems), which process iAP2 packets
/// strictly per transfer completion rather than as a continuous byte stream.
///
/// Apple’s iAP2 documentation does not explicitly define required behavior with
/// respect to transport-level transfer boundaries, leaving room for interpretation
/// by accessory and head unit implementations.
///
/// Note:
/// - iOS as a receiver is stream-oriented and tolerant to arbitrary fragmentation
///   or coalescing of packets across transport transfers.
/// - This invariant is therefore **a transmitter-side policy**, not a protocol
///   requirement, and is enforced to match observed automotive behavior.
///
/// For stream-oriented receivers (e.g. Bluetooth RFCOMM), packets are reconstructed
/// from a raw byte stream. In such cases, no transport-level packet index exists.
#[derive(Debug)]
pub struct PacketFrame<T: AsRef<[u8]>> {
    /// Optional monotonic packet index used for tracing, debugging and correlation.
    ///
    /// Present for transport layers with explicit transfer completions (e.g. USB),
    /// absent for stream-oriented transports (e.g. Bluetooth).
    pub packet_index: Option<u64>,

    /// Serialized iAP2 packet buffer.
    pub frame: T,
}

impl<T: AsRef<[u8]>> PacketFrame<T> {
    pub fn new(packet_index: Option<u64>, frame: T) -> Self {
        Self { packet_index, frame }
    }

    pub fn into_inner(self) -> T {
        self.frame
    }
}

impl<T: AsRef<[u8]>> AsRef<[u8]> for PacketFrame<T> {
    fn as_ref(&self) -> &[u8] {
        self.frame.as_ref()
    }
}

#[derive(PartialEq, Clone)]
pub enum PacketOrDetect {
    /// iAP2 detection sequence
    Detect,
    /// iAP2 packet
    Packet(Packet),
    /// iAP1 legacy (Lingo) packet
    Legacy(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Packet {
    pub header: Header,
    // The payload is not always session payload; payload function depends on the header
    pub payload: Option<Vec<u8>>,
}

impl fmt::Display for PacketOrDetect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PacketOrDetect::Detect => f.write_str("DETECT"),
            PacketOrDetect::Packet(packet) => packet.fmt(f),
            #[cfg(feature = "lingo")]
            PacketOrDetect::Legacy(body) => {
                #[cfg(feature = "lingo_parser")]
                if let Ok(packet) = catplay_lingo::Packet::decode(&catplay_lingo::Frame { body }) {
                    return write!(f, "LEGACY {packet:?}");
                }
                write!(f, "LEGACY {body:02x?}")
            }
        }
    }
}

impl fmt::Debug for PacketOrDetect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PacketOrDetect::Detect => f.write_str("DETECT"),
            PacketOrDetect::Packet(packet) => packet.fmt(f),
            #[cfg(feature = "lingo")]
            PacketOrDetect::Legacy(body) => {
                #[cfg(feature = "lingo_parser")]
                if let Ok(packet) = catplay_lingo::Packet::decode(&catplay_lingo::Frame { body }) {
                    return write!(f, "LEGACY {packet:?}");
                }
                write!(f, "LEGACY {body:02x?}")
            }
        }
    }
}

impl From<Packet> for PacketOrDetect {
    fn from(value: Packet) -> Self {
        PacketOrDetect::Packet(value)
    }
}

impl Packet {
    #[allow(unused)]
    pub const PACKET_OVERHEAD: usize = 10;
    pub const PACKET_SIZE_WITHOUT_PAYLOAD: usize = 9;

    pub fn size(&self) -> usize {
        match &self.payload {
            Some(v) => v.len() + Self::PACKET_OVERHEAD,
            None => Self::PACKET_SIZE_WITHOUT_PAYLOAD,
        }
    }

    fn fix_length(&mut self) {
        let mut length = 9u16;
        if let Some(payload) = &self.payload {
            if payload.len() > 65525 {
                panic!("Unacceptable iAP2 payload > 65525 bytes!");
            }

            length += payload.len() as u16 + 1;
        }
        self.header.length = length;
    }

    pub fn new_syn(own_seq: ModSeq8, lsp: &LSPPayload) -> Packet {
        let mut packet = Packet {
            header: Header {
                session_id: LSPPayload::SESSION_ID_CONTROL,
                length: 0,
                control: LinkControl::SYN,
                seq: own_seq,
                ack: ModSeq(0),
            },
            payload: Some(lsp.to_bytes()),
        };

        packet.fix_length();
        packet
    }

    pub fn new_syn_ack(own_seq: ModSeq8, peer_seq: ModSeq8, lsp: &LSPPayload) -> Packet {
        let mut packet = Packet {
            header: Header {
                session_id: LSPPayload::SESSION_ID_CONTROL,
                length: 0,
                control: LinkControl::SYN | LinkControl::ACK,
                seq: own_seq,
                ack: peer_seq,
            },
            payload: Some(lsp.to_bytes()),
        };

        packet.fix_length();
        packet
    }

    pub fn new_ack(own_seq: ModSeq8, peer_seq: ModSeq8, session_id: u8, payload: Option<Vec<u8>>) -> Packet {
        let mut packet = Packet {
            header: Header {
                session_id,
                length: 0,
                control: LinkControl::ACK,
                seq: own_seq,
                ack: peer_seq,
            },
            payload,
        };

        packet.fix_length();
        packet
    }

    pub fn new_eak(own_seq: ModSeq8, peer_seq: ModSeq8, session_id: u8, payload: Vec<u8>) -> Packet {
        let mut packet = Packet {
            header: Header {
                session_id,
                length: 0,
                control: LinkControl::ACK | LinkControl::EAK,
                seq: own_seq,
                ack: peer_seq,
            },
            payload: Some(payload),
        };

        packet.fix_length();
        packet
    }

    pub fn is_ackable(&self) -> bool {
        let ctl = &self.header.control;
        ctl.is_syn() || ((ctl.is_ack() || ctl.is_syn_ack()) && self.payload.is_some())
    }
}

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[LEN {} CTL ({}) SEQ {} ACK {} SID {}]\n",
            self.header.length,
            self.header.control.to_str(),
            self.header.seq.value(),
            self.header.ack.value(),
            self.header.session_id,
        )?;

        if let Some(payload) = &self.payload {
            for chunk in payload.chunks(8) {
                f.write_str("    [")?;
                for byte in chunk {
                    write!(f, "{byte:02X} ")?;
                }
                f.write_str("]\n")?;
            }
        }

        if (self.header.control.is_syn() || self.header.control.is_ack())
            && let Some(lsp) = LSPPayload::from_packet(self)
        {
            writeln!(f, "    {lsp:?}")?;
        } else if let Some(file_transfer) = FileTransferPayload::from_packet(self)
            && self.header.session_id == 2
        {
            writeln!(f, "    {file_transfer:?} [assumed FileTransferPayload]")?;
        } else {
            #[cfg(feature = "csm_parser")]
            {
                let registry = catplay_csm::decoder::CsmPacketRegistry::static_registry();
                if let Some(payload) = &self.payload
                    && let Some(p) = registry.decode(payload)
                {
                    writeln!(f, "    {p:?}")?;
                }
            }
        }
        Ok(())
    }
}
