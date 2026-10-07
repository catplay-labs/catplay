extern crate alloc;

use alloc::vec::Vec;

use crate::decoder::{
    AsCsmPacket, CsmError, CsmPacketBox, CsmPacketWithPayload, CsmPayloadEncode, CsmResult,
    packets::registry::{CsmPacketRegistry, CsmUnknownPacket},
};

impl CsmPacketRegistry {
    /// Decodes a known registered packet or returns `CsmUnknownPacket` with its id and payload.
    ///
    /// Returns None for packets with incomplete header.
    pub fn decode(&self, data: &[u8]) -> Option<CsmPacketBox> {
        let packet = CsmPacketWithPayload::try_from(data).ok()?;
        let (header, payload) = packet.split();

        let packet = self.create_by_id_from_bytes(header.id, payload);
        match packet {
            Some(p) => Some(p),
            None => Some(CsmUnknownPacket(header.id, payload.into()).into()),
        }
    }

    /// Encodes a known packet with ID from the registry, including special case of `CsmUnknownPacket`.
    pub fn encode(&self, packet: &dyn AsCsmPacket) -> CsmResult<Vec<u8>> {
        let packet = packet.as_csm();
        let unknown_id = packet.cast::<CsmUnknownPacket>().map(|p| p.id());
        let Some(id) = self.id_of(packet).or(unknown_id) else {
            return Err(CsmError::PacketUnknown);
        };

        let payload = packet.serialize();

        let packet = CsmPacketWithPayload::new(id, &payload)?;
        let buf = packet.serialize();
        Ok(buf)
    }
}
