extern crate alloc;

use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use core::any::TypeId;

use crate::decoder::{CsmDecode, CsmPacket, CsmPacketBox, CsmStruct};

struct PacketEntry {
    decode: fn(u16, Option<&[u8]>) -> CsmPacketBox,
}

pub struct CsmPacketRegistry {
    id_to_entry: BTreeMap<u16, Arc<PacketEntry>>,
    type_to_id: BTreeMap<TypeId, u16>,
}

pub enum CsmPacketRegistration {
    Single {
        id: u16,
        register_fn: fn(&mut CsmPacketRegistry, u16),
    },
    Batch {
        register_fn: fn(&mut CsmPacketRegistry),
    },
}

impl CsmPacketRegistry {
    pub fn new() -> Self {
        Self {
            id_to_entry: BTreeMap::new(),
            type_to_id: BTreeMap::new(),
        }
    }

    pub fn register_type<T>(&mut self, id: u16)
    where
        T: 'static + Default + CsmPacket + CsmStruct,
    {
        self.register_types(&[(id, TypeId::of::<T>())], decode_type::<T>);
    }

    /// Register one decoder for several packet IDs. The generated iAP2 registry
    /// can supply multiple (ID, type) pairs without adding one callback per type.
    pub(crate) fn register_types(&mut self, types: &[(u16, TypeId)], decode: fn(u16, Option<&[u8]>) -> CsmPacketBox) {
        for (index, (id, _)) in types.iter().enumerate() {
            if self.id_to_entry.contains_key(id) || types[..index].iter().any(|(previous, _)| previous == id) {
                panic!("Attempted duplicate CSM registration at id 0x{id:04X}");
            }
        }

        let entry = Arc::new(PacketEntry { decode });
        for &(id, type_id) in types {
            self.id_to_entry.insert(id, Arc::clone(&entry));
            self.type_to_id.insert(type_id, id);
        }
    }

    pub fn all_known_ids(&self) -> Vec<u16> {
        self.id_to_entry.keys().copied().collect()
    }

    pub fn contains_id(&self, id: u16) -> bool {
        self.id_to_entry.contains_key(&id)
    }

    pub fn create_by_id(&self, id: u16) -> Option<CsmPacketBox> {
        self.id_to_entry.get(&id).map(|f| (f.decode)(id, None))
    }

    pub fn create_by_id_from_bytes(&self, id: u16, data: &[u8]) -> Option<CsmPacketBox> {
        self.id_to_entry
            .get(&id)
            .map(|f| (f.decode)(id, Some(data)))
    }

    pub fn id_of(&self, pkt: &dyn CsmPacket) -> Option<u16> {
        self.type_to_id.get(&pkt.as_any().type_id()).copied()
    }

    pub const fn as_registration<T: 'static + Default + CsmPacket + CsmStruct>(id: u16) -> CsmPacketRegistration {
        CsmPacketRegistration::Single {
            id,
            register_fn: |reg, id| reg.register_types(&[(id, TypeId::of::<T>())], decode_type::<T>),
        }
    }

    pub const fn as_batch_registration(register_fn: fn(&mut CsmPacketRegistry)) -> CsmPacketRegistration {
        CsmPacketRegistration::Batch { register_fn }
    }
}

fn decode_type<T>(_: u16, data: Option<&[u8]>) -> CsmPacketBox
where
    T: 'static + Default + CsmPacket + CsmStruct,
{
    match data {
        Some(data) => T::decode_from_bytes(data).into(),
        None => T::default().into(),
    }
}

impl Default for CsmPacketRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "iap2"))]
mod tests {
    use super::*;
    use crate::{
        decoder::{CsmPacketId, CsmPayloadEncode},
        msg::iap2::{PowerUpdate, StartPowerUpdates},
    };

    fn decode_power_packet(id: u16, data: Option<&[u8]>) -> CsmPacketBox {
        match id {
            PowerUpdate::PACKET_ID => decode_type::<PowerUpdate>(id, data),
            StartPowerUpdates::PACKET_ID => decode_type::<StartPowerUpdates>(id, data),
            _ => unreachable!(),
        }
    }

    #[test]
    fn one_callback_registers_multiple_packet_types() {
        let mut registry = CsmPacketRegistry::new();
        registry.register_types(
            &[
                (PowerUpdate::PACKET_ID, TypeId::of::<PowerUpdate>()),
                (StartPowerUpdates::PACKET_ID, TypeId::of::<StartPowerUpdates>()),
            ],
            decode_power_packet,
        );

        assert!(Arc::ptr_eq(
            registry.id_to_entry.get(&PowerUpdate::PACKET_ID).unwrap(),
            registry
                .id_to_entry
                .get(&StartPowerUpdates::PACKET_ID)
                .unwrap(),
        ));
        assert_eq!(registry.id_of(&PowerUpdate::default()), Some(PowerUpdate::PACKET_ID));
        assert_eq!(registry.id_of(&StartPowerUpdates::default()), Some(StartPowerUpdates::PACKET_ID));
        assert_eq!(
            registry
                .create_by_id(StartPowerUpdates::PACKET_ID)
                .unwrap()
                .cast::<StartPowerUpdates>(),
            Some(&StartPowerUpdates::default()),
        );

        let value = PowerUpdate {
            maximum_current_drawn_from_accessory: Some(42),
            ..PowerUpdate::default()
        };
        assert_eq!(
            registry
                .create_by_id_from_bytes(PowerUpdate::PACKET_ID, &value.serialize())
                .unwrap()
                .cast::<PowerUpdate>(),
            Some(&value),
        );
    }
}
