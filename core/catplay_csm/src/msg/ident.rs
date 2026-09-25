use crate::{decoder::*, packet_type};

use super::*;

packet_type! {
    pub struct StartIdentification: 0x1D00 {
    }
}

packet_type! {
    pub struct IdentificationInformation: 0x1D01 {
        #[csm_id( 0)] pub name: CsmString,
        #[csm_id( 1)] pub model_identifier: CsmString,
        #[csm_id( 2)] pub manufacturer: CsmString,
        #[csm_id( 3)] pub serial_number: CsmString,
        #[csm_id( 4)] pub firmware_version: CsmString,
        #[csm_id( 5)] pub hardware_version: CsmString,
        #[csm_id( 6)] pub messages_sent_by_accessory: CsmByteArray,
        #[csm_id( 7)] pub messages_received_from_device: CsmByteArray,
        #[csm_id( 8)] pub power_providing_capability: PowerProvidingCapability,
        #[csm_id( 9)] pub maximum_current_drawn_from_device: u16,
        #[csm_id(10)] pub supported_external_accessory_protocol: CsmVec<ExternalAccessoryProtocol>,
        #[csm_id(11)] pub app_match_team_id: Option<CsmString>,
        #[csm_id(12)] pub current_language: CsmString,
        #[csm_id(13)] pub supported_language: CsmVec<CsmString>, // 1+

        #[csm_id(14)] pub serial_transport_component: Option<SerialTransportComponent>,
        #[csm_id(15)] pub usbdevice_transport_component: Option<USBDeviceTransportComponent>,
        #[csm_id(16)] pub usbhost_transport_component: Option<USBHostTransportComponent>,
        #[csm_id(17)] pub bluetooth_transport_component: CsmVec<BluetoothTransportComponent>,
        #[csm_id(18)] pub iap2_hidcomponent: CsmVec<IAP2HIDComponent>,

        #[csm_id(20)] pub vehicle_information_component: Option<VehicleInformationComponent>,
        #[csm_id(21)] pub vehicle_status_component: Option<VehicleStatusComponent>,
        #[csm_id(22)] pub location_information_component: Option<LocationInformationComponent>,

        #[csm_id(23)] pub usbhost_hidcomponent: CsmVec<USBHostHIDComponent>,
        #[csm_id(24)] pub wireless_car_play_transport_component: CsmVec<WirelessCarPlayTransportComponent>,
        #[csm_id(29)] pub bluetooth_hidcomponent: Option<BluetoothHIDComponent>,
    }
}

impl IdentificationInformation {
    pub fn pack_ids(ids: &[u16]) -> CsmByteArray {
        let mut out: CsmVec<u8> = CsmVec::with_capacity(ids.len() * 2);
        for id in ids {
            let id = id.to_be_bytes();
            out.push(id[0]);
            out.push(id[1]);
        }

        out.into()
    }

    pub fn unpack_ids(ids: &CsmByteArray) -> CsmVec<u16> {
        let mut out = CsmVec::with_capacity(ids.data.len() / 2);
        let i = 0;
        while i + 1 < out.len() {
            let id = u16::from_be_bytes([ids.data[i], ids.data[i + 1]]);
            out.push(id);
        }

        out
    }

    pub fn has_id(ids: &[u8], id: u16) -> bool {
        let i = 0;
        while i + 1 < ids.len() / 2 {
            let tmp = u16::from_be_bytes([ids[i], ids[i + 1]]);
            if id == tmp {
                return true;
            }
        }

        false
    }

    pub fn wants_rx(&self, id: u16) -> bool {
        Self::has_id(&self.messages_received_from_device.data, id)
    }

    pub fn wants_tx(&self, id: u16) -> bool {
        Self::has_id(&self.messages_sent_by_accessory.data, id)
    }
}

packet_type! {
    pub struct IdentificationAccepted: 0x1D02 {
    }
}

packet_type! {
    pub struct IdentificationRejected: 0x1D03 {
        #[csm_id( 0)] pub name: CsmFlag,
        #[csm_id( 1)] pub model_identifier: CsmFlag,
        #[csm_id( 2)] pub manufacturer: CsmFlag,
        #[csm_id( 3)] pub serial_number: CsmFlag,
        #[csm_id( 4)] pub firmware_version: CsmFlag,
        #[csm_id( 5)] pub hardware_version: CsmFlag,
        #[csm_id( 6)] pub messages_sent_by_accessory: CsmFlag,
        #[csm_id( 7)] pub messages_received_from_device: CsmFlag,
        #[csm_id( 8)] pub power_providing_capability: CsmFlag,
        #[csm_id( 9)] pub maximum_current_drawn_from_device: CsmFlag,
        #[csm_id(10)] pub supported_external_accessory_protocol: CsmFlag,
        #[csm_id(11)] pub app_match_team_id: CsmFlag,
        #[csm_id(12)] pub current_language: CsmFlag,
        #[csm_id(13)] pub supported_language: CsmFlag,

        #[csm_id(14)] pub serial_transport_component: CsmFlag,
        #[csm_id(15)] pub usbdevice_transport_component: CsmFlag,
        #[csm_id(16)] pub usbhost_transport_component: CsmFlag,
        #[csm_id(17)] pub bluetooth_transport_component: CsmFlag,
        #[csm_id(18)] pub iap2_hidcomponent: CsmFlag,

        #[csm_id(20)] pub vehicle_information_component: CsmFlag,
        #[csm_id(21)] pub vehicle_status_component: CsmFlag,
        #[csm_id(22)] pub location_information_component: CsmFlag,

        #[csm_id(23)] pub usbhost_hidcomponent: CsmFlag,
        #[csm_id(24)] pub wireless_car_play_transport_component: CsmFlag,
        #[csm_id(29)] pub bluetooth_hidcomponent: CsmFlag,
    }
}

impl IdentificationRejected {
    pub fn reason(&self) -> CsmString {
        let map = [
            ("name", self.name),
            ("model_identifier", self.model_identifier),
            ("serial_number", self.serial_number),
            ("firmware_version", self.firmware_version),
            ("messages_sent_by_accessory", self.messages_sent_by_accessory),
            ("messages_received_from_device", self.messages_received_from_device),
            ("power_providing_capability", self.power_providing_capability),
            ("maximum_current_drawn_from_device", self.maximum_current_drawn_from_device),
            ("supported_external_accessory_protocol", self.supported_external_accessory_protocol),
            ("app_match_team_id", self.app_match_team_id),
            ("current_language", self.current_language),
            ("supported_language", self.supported_language),
            ("serial_transport_component", self.serial_transport_component),
            ("usbdevice_transport_component", self.usbdevice_transport_component),
            ("usbhost_transport_component", self.usbhost_transport_component),
            ("bluetooth_transport_component", self.bluetooth_transport_component),
            ("iap2_hidcomponent", self.iap2_hidcomponent),
            ("vehicle_information_component", self.vehicle_information_component),
            ("vehicle_status_component", self.vehicle_status_component),
            ("location_information_component", self.location_information_component),
            ("usbhost_hidcomponent", self.usbhost_hidcomponent),
            ("wireless_car_play_transport_component", self.wireless_car_play_transport_component),
            ("bluetooth_hidcomponent", self.bluetooth_hidcomponent),
        ];

        let out: CsmVec<_> = map
            .iter()
            .filter(|&i| i.1 == CsmFlag::Yes)
            .map(|i| i.0)
            .collect();
        out.join(",")
    }
}

packet_type! {
    pub struct CancelIdentification: 0x1D05 {
    }
}

packet_type! {
    pub struct IdentificationInformationUpdate: 0x1D06 {
        #[csm_id( 0)] pub name: CsmString,
        #[csm_id( 1)] pub model_identifier: CsmString,
        #[csm_id( 2)] pub manufacturer: CsmString,
        #[csm_id( 3)] pub serial_number: CsmString,
        #[csm_id( 4)] pub firmware_version: CsmString,
        #[csm_id( 5)] pub hardware_version: CsmString,
        #[csm_id( 6)] pub current_language: CsmString,
    }
}
