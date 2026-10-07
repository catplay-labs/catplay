// @generated from Apple ATS Default.lktspec
// Do not edit manually. Regenerate with catplay_lingo.

#[allow(unused_imports)]
use crate::{__String as String, __Vec as Vec};
#[allow(unused_imports)]
use crate::{
    FlatPredicateToken, Iap1WireSpec, iap1, iap1_bitfield, iap1_bitflags, iap1_disjoint, iap1_enum,
    iap1_enum_open, iap1_enum_tokens, iap1_record, iap1_registry, predicate_eval::*,
};

use super::strings::LingoString;

// Lingo 0x0e: Location
iap1_enum! {
    strings = LingoString;
    pub enum AccessoryAckCommandResult: u8 {
        /// OK
        OK = 0,
        /// failed
        Failed = 2,
        /// out of resources
        OutOfResources = 3,
        /// bad parameter
        BadParameter = 4,
        /// not authenticated
        NotAuthenticated = 7,
        /// command timeout
        CommandTimeout = 15,
        /// section received OK
        SectionReceivedOK = 19,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0e, command = 0x0000,
        source = accessory,
        response = true,
        ack = true,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct AccessoryAck {
        fields {
            /// Command Result
            // Virtual schema indices: 1
            required pub command_result: AccessoryAckCommandResult => { bit: 0, key: "commandResult", when: (truthy(&true)), predicate_value: true },
            /// Command ID
            // Virtual schema indices: 2
            required pub acked_command_id: u8 => { bit: 1, key: "ackedCommandID", when: (truthy(&true)), predicate_value: false },
            /// Current Section Index
            // Virtual schema indices: 3
            optional pub sect_cur: u16 => { bit: 2, key: "sectCur", when: (eq(&command_result, &19u64)), predicate_value: false },
        }
        steps {
            1 => required command_result: AccessoryAckCommandResult => { bit: 0, key: "commandResult", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required acked_command_id: u8 => { bit: 1, key: "ackedCommandID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => optional sect_cur: u16 => { bit: 2, key: "sectCur", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&command_result, &19u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(19u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum GetAccessoryCapsLocType: u8 {
        /// system
        System = 0,
        /// NMEA GPS location
        NMEAGPSLocation = 1,
        /// location assistance
        LocationAssistance = 2,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0e, command = 0x0001,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetAccessoryCaps {
        fields {
            /// Capability Type
            // Virtual schema indices: 1
            required pub loc_type: GetAccessoryCapsLocType => { bit: 0, key: "locType", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required loc_type: GetAccessoryCapsLocType => { bit: 0, key: "locType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetAccessoryCapsLocType: u8 {
        /// system
        System = 0,
        /// NMEA GPS location
        NMEAGPSLocation = 1,
        /// location assistance
        LocationAssistance = 2,
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetAccessoryCapsSysCapsMask: u64 {
        /// power management control
        const POWER_MANAGEMENT_CONTROL = 1 << 0;
        /// asynchronous location notification
        const ASYNCHRONOUS_LOCATION_NOTIFICATION = 1 << 2;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetAccessoryCapsLocCapsMask: u64 {
        /// system
        /// Required by the specification.
        const SYSTEM = 1 << 0;
        /// NMEA GPS location
        const NMEA_GPS_LOCATION = 1 << 1;
        /// location assistance
        const LOCATION_ASSISTANCE = 1 << 2;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetAccessoryCapsNmeaGpsCaps: u64 {
        /// NMEA GPS sentence filtering
        /// Required by the specification.
        const NMEA_GPS_SENTENCE_FILTERING = 1 << 0;
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetAccessoryCapsLocAsstData: u64 {
        /// satellite ephemeris data required
        const SATELLITE_EPHEMERIS_DATA_REQUIRED = 1 << 1;
        /// satellite ephemeris data URL string
        const SATELLITE_EPHEMERIS_DATA_URL_STRING = 1 << 2;
        /// satellite ephemeris data maximum refresh interval
        const SATELLITE_EPHEMERIS_DATA_MAXIMUM_REFRESH_INTERVAL = 1 << 3;
        /// satellite ephemeris data recommended refresh interval
        const SATELLITE_EPHEMERIS_DATA_RECOMMENDED_REFRESH_INTERVAL = 1 << 4;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0e, command = 0x0002,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetAccessoryCaps {
        fields {
            /// Capability Type
            // Virtual schema indices: 1
            required pub loc_type: RetAccessoryCapsLocType => { bit: 0, key: "locType", when: (truthy(&true)), predicate_value: true },
            /// Location Lingo Major Version Supported
            // Virtual schema indices: 2
            optional pub dev_ver_major: u8 => { bit: 1, key: "devVerMajor", when: (eq(&loc_type, &0u64)), predicate_value: false },
            /// Location Lingo Minor Version supported
            // Virtual schema indices: 3
            optional pub dev_ver_minor: u8 => { bit: 2, key: "devVerMinor", when: (eq(&loc_type, &0u64)), predicate_value: false },
            /// System Capabilities
            // Virtual schema indices: 4
            optional pub sys_caps_mask: RetAccessoryCapsSysCapsMask => { bit: 3, key: "sysCapsMask", when: (eq(&loc_type, &0u64)), predicate_value: false },
            /// Location Type Support
            // Virtual schema indices: 5
            optional pub loc_caps_mask: RetAccessoryCapsLocCapsMask => { bit: 4, key: "locCapsMask", when: (eq(&loc_type, &0u64)), predicate_value: false },
            /// NMEA GPS Location Capabilities
            // Virtual schema indices: 6
            optional pub nmea_gps_caps: RetAccessoryCapsNmeaGpsCaps => { bit: 5, key: "nmeaGpsCaps", when: (eq(&loc_type, &1u64)), predicate_value: false },
            /// Supported Location Assistance Data Types
            // Virtual schema indices: 7
            optional pub loc_asst_data: RetAccessoryCapsLocAsstData => { bit: 6, key: "locAsstData", when: (eq(&loc_type, &2u64)), predicate_value: false },
        }
        steps {
            1 => required loc_type: RetAccessoryCapsLocType => { bit: 0, key: "locType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional dev_ver_major: u8 => { bit: 1, key: "devVerMajor", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&loc_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional dev_ver_minor: u8 => { bit: 2, key: "devVerMinor", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&loc_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            4 => optional sys_caps_mask: RetAccessoryCapsSysCapsMask => { bit: 3, key: "sysCapsMask", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&loc_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            5 => optional loc_caps_mask: RetAccessoryCapsLocCapsMask => { bit: 4, key: "locCapsMask", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&loc_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            6 => optional nmea_gps_caps: RetAccessoryCapsNmeaGpsCaps => { bit: 5, key: "nmeaGpsCaps", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&loc_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            7 => optional loc_asst_data: RetAccessoryCapsLocAsstData => { bit: 6, key: "locAsstData", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&loc_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum GetAccessoryControlLocType: u8 {
        /// system
        System = 0,
        /// NMEA GPS location
        NMEAGPSLocation = 1,
        /// location assistance
        LocationAssistance = 2,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0e, command = 0x0003,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetAccessoryControl {
        fields {
            /// Capability Type
            // Virtual schema indices: 1
            required pub loc_type: GetAccessoryControlLocType => { bit: 0, key: "locType", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required loc_type: GetAccessoryControlLocType => { bit: 0, key: "locType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetAccessoryControlLocType: u8 {
        /// system
        System = 0,
        /// NMEA GPS location
        NMEAGPSLocation = 1,
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetAccessoryControlSysCtlFlags: u64 {
        /// asynchronous location notifications enabled
        const ASYNCHRONOUS_LOCATION_NOTIFICATIONS_ENABLED = 1 << 2;
    }
}

iap1_bitfield! {
    pub struct RetAccessoryControlSysCtl: u64 {
        #[flags(mask = 0x4)]
        pub flags: RetAccessoryControlSysCtlFlags,
        #[group(mask = 0x3, shift = 0)]
        /// accessory GPS radio power
        pub acc_gps_radio_power: RetAccessoryControlSysCtlAccGPSRadioPower {
            /// power off
            PowerOff = 0,
            /// power on
            PowerOn = 3,
        },
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetAccessoryControlNmeaGpsCtrl: u64 {
        /// NMEA GPS sentence filtering
        const NMEA_GPS_SENTENCE_FILTERING = 1 << 0;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0e, command = 0x0004,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetAccessoryControl {
        fields {
            /// Capability Type
            // Virtual schema indices: 1
            required pub loc_type: RetAccessoryControlLocType => { bit: 0, key: "locType", when: (truthy(&true)), predicate_value: true },
            /// System Control
            // Virtual schema indices: 2
            optional pub sys_ctl: RetAccessoryControlSysCtl => { bit: 1, key: "sysCtl", when: (eq(&loc_type, &0u64)), predicate_value: false },
            /// NMEA GPS Control
            // Virtual schema indices: 3
            optional pub nmea_gps_ctrl: RetAccessoryControlNmeaGpsCtrl => { bit: 2, key: "nmeaGpsCtrl", when: (eq(&loc_type, &1u64)), predicate_value: false },
        }
        steps {
            1 => required loc_type: RetAccessoryControlLocType => { bit: 0, key: "locType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional sys_ctl: RetAccessoryControlSysCtl => { bit: 1, key: "sysCtl", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&loc_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional nmea_gps_ctrl: RetAccessoryControlNmeaGpsCtrl => { bit: 2, key: "nmeaGpsCtrl", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&loc_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetAccessoryControlLocType: u8 {
        /// system
        System = 0,
        /// NMEA GPS location
        NMEAGPSLocation = 1,
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct SetAccessoryControlSysCtlFlags: u64 {
        /// asynchronous location notifications enabled
        const ASYNCHRONOUS_LOCATION_NOTIFICATIONS_ENABLED = 1 << 2;
    }
}

iap1_bitfield! {
    pub struct SetAccessoryControlSysCtl: u64 {
        #[flags(mask = 0x4)]
        pub flags: SetAccessoryControlSysCtlFlags,
        #[group(mask = 0x3, shift = 0)]
        /// accessory GPS radio power
        pub acc_gps_radio_power: SetAccessoryControlSysCtlAccGPSRadioPower {
            /// power off
            PowerOff = 0,
            /// power on
            PowerOn = 3,
        },
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct SetAccessoryControlNmeaGpsCtrl: u64 {
        /// NMEA GPS sentence filtering
        const NMEA_GPS_SENTENCE_FILTERING = 1 << 0;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0e, command = 0x0005,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetAccessoryControl {
        fields {
            /// Capability Type
            // Virtual schema indices: 1
            required pub loc_type: SetAccessoryControlLocType => { bit: 0, key: "locType", when: (truthy(&true)), predicate_value: true },
            /// System Control
            // Virtual schema indices: 2
            optional pub sys_ctl: SetAccessoryControlSysCtl => { bit: 1, key: "sysCtl", when: (eq(&loc_type, &0u64)), predicate_value: false },
            /// NMEA GPS Control
            // Virtual schema indices: 3
            optional pub nmea_gps_ctrl: SetAccessoryControlNmeaGpsCtrl => { bit: 2, key: "nmeaGpsCtrl", when: (eq(&loc_type, &1u64)), predicate_value: false },
        }
        steps {
            1 => required loc_type: SetAccessoryControlLocType => { bit: 0, key: "locType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional sys_ctl: SetAccessoryControlSysCtl => { bit: 1, key: "sysCtl", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&loc_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional nmea_gps_ctrl: SetAccessoryControlNmeaGpsCtrl => { bit: 2, key: "nmeaGpsCtrl", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&loc_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum GetAccessoryDataLocType: u8 {
        /// location assistance
        LocationAssistance = 2,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum GetAccessoryDataLocationAssistanceDataType: u8 {
        /// satellite ephemeris data maximum required refresh interval
        SatelliteEphemerisDataMaximumRequiredRefreshInterval = 3,
        /// satellite ephemeris data recommended refresh interval
        SatelliteEphemerisDataRecommendedRefreshInterval = 4,
    }
}

/// The specification lists no named values for this enum field.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GetAccessoryDataUnknownDataType(pub u8);
impl crate::__NamedDebug for GetAccessoryDataUnknownDataType {
    fn fmt_named(&self, name: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple(name).field(self).finish()
    }
}

impl crate::Iap1Decode for GetAccessoryDataUnknownDataType {
    fn decode(reader: &mut crate::Reader<'_>) -> Result<Self, crate::DecodeError> {
        Ok(Self(<u8 as crate::Iap1Decode>::decode(reader)?))
    }
}
impl crate::Iap1Encode for GetAccessoryDataUnknownDataType {
    fn encode(&self, writer: &mut crate::Writer<'_>) -> Result<(), crate::EncodeError> {
        crate::Iap1Encode::encode(&self.0, writer)
    }
}
impl crate::PredicateValue for GetAccessoryDataUnknownDataType {
    fn predicate_integer(&self) -> Option<i128> {
        Some(self.0 as i128)
    }
}

iap1_disjoint! {
    strings = LingoString;
    #[iap1_disjoint(predicate = integer)]
    pub enum GetAccessoryDataDataTypeDisjoint {
        #[iap1_disjoint(decode = decode_location_assistance_data_type, encode = encode_location_assistance_data_type)]
        /// Location Assistance Data Type
        LocationAssistanceDataType(GetAccessoryDataLocationAssistanceDataType),
        #[iap1_disjoint(decode = decode_unknown_data_type, encode = encode_unknown_data_type)]
        /// Unknown Data Type
        UnknownDataType(GetAccessoryDataUnknownDataType),
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0e, command = 0x0006,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetAccessoryData {
        fields {
            /// Capability Type
            // Virtual schema indices: 1
            required pub loc_type: GetAccessoryDataLocType => { bit: 0, key: "locType", when: (truthy(&true)), predicate_value: true },
            /// Location Assistance Data Type
            // Virtual schema indices: 2, 3
            required pub data_type: GetAccessoryDataDataTypeDisjoint => { bit: 1, key: "dataType", when: (eq(&loc_type, &2u64)) || (ne(&loc_type, &2u64)), predicate_value: false },
        }
        steps {
            1 => required loc_type: GetAccessoryDataLocType => { bit: 0, key: "locType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required data_type: GetAccessoryDataDataTypeDisjoint => { bit: 1, key: "dataType", wire: [disjoint(decode_location_assistance_data_type, encode_location_assistance_data_type, GetAccessoryDataLocationAssistanceDataType, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_location_assistance_data_type)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: eq(&loc_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            3 => required data_type: GetAccessoryDataDataTypeDisjoint => { bit: 1, key: "dataType", wire: [disjoint(decode_unknown_data_type, encode_unknown_data_type, GetAccessoryDataUnknownDataType, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_unknown_data_type)) }, child: &Iap1WireSpec::Scalar }, predicate_value: false, predicate: { expr: ne(&loc_type, &2u64), program: [FlatPredicateToken::Ne, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetAccessoryDataLocType: u8 {
        /// location assistance
        LocationAssistance = 2,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetAccessoryDataLocationAssistanceDataType: u8 {
        /// satellite ephemeris data maximum required refresh interval
        SatelliteEphemerisDataMaximumRequiredRefreshInterval = 3,
        /// satellite ephemeris data recommended refresh interval
        SatelliteEphemerisDataRecommendedRefreshInterval = 4,
    }
}

/// The specification lists no named values for this enum field.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RetAccessoryDataUnknownDataType(pub u8);
impl crate::__NamedDebug for RetAccessoryDataUnknownDataType {
    fn fmt_named(&self, name: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple(name).field(self).finish()
    }
}

impl crate::Iap1Decode for RetAccessoryDataUnknownDataType {
    fn decode(reader: &mut crate::Reader<'_>) -> Result<Self, crate::DecodeError> {
        Ok(Self(<u8 as crate::Iap1Decode>::decode(reader)?))
    }
}
impl crate::Iap1Encode for RetAccessoryDataUnknownDataType {
    fn encode(&self, writer: &mut crate::Writer<'_>) -> Result<(), crate::EncodeError> {
        crate::Iap1Encode::encode(&self.0, writer)
    }
}
impl crate::PredicateValue for RetAccessoryDataUnknownDataType {
    fn predicate_integer(&self) -> Option<i128> {
        Some(self.0 as i128)
    }
}

iap1_disjoint! {
    strings = LingoString;
    #[iap1_disjoint(predicate = integer)]
    pub enum RetAccessoryDataDataTypeDisjoint {
        #[iap1_disjoint(decode = decode_location_assistance_data_type, encode = encode_location_assistance_data_type)]
        /// Location Assistance Data Type
        LocationAssistanceDataType(RetAccessoryDataLocationAssistanceDataType),
        #[iap1_disjoint(decode = decode_unknown_data_type, encode = encode_unknown_data_type)]
        /// Unknown Data Type
        UnknownDataType(RetAccessoryDataUnknownDataType),
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0e, command = 0x0007,
        source = accessory,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetAccessoryData {
        fields {
            /// Capability Type
            // Virtual schema indices: 1
            required pub loc_type: RetAccessoryDataLocType => { bit: 0, key: "locType", when: (truthy(&true)), predicate_value: true },
            /// Location Assistance Data Type
            // Virtual schema indices: 2, 3
            required pub data_type: RetAccessoryDataDataTypeDisjoint => { bit: 1, key: "dataType", when: (eq(&loc_type, &2u64)) || (ne(&loc_type, &2u64)), predicate_value: true },
            /// Current Payload Section Index
            // Virtual schema indices: 4
            required pub sect_cur: u16 => { bit: 2, key: "sectCur", when: (truthy(&true)), predicate_value: true },
            /// Maximum Payload Section Index
            // Virtual schema indices: 5
            required pub sect_max: u16 => { bit: 3, key: "sectMax", when: (truthy(&true)), predicate_value: false },
            /// Total Size
            // Virtual schema indices: 6
            optional pub total_size: u32 => { bit: 4, key: "totalSize", when: (eq(&sect_cur, &0u64)), predicate_value: false },
            /// Maximum Required Refresh Interval
            // Virtual schema indices: 7
            optional pub max_required_refresh_interval: u32 => { bit: 5, key: "maxRequiredRefreshInterval", when: ((eq(&loc_type, &2u64)) && (eq(&data_type, &3u64))), predicate_value: false },
            /// Recommended Refresh Interval
            // Virtual schema indices: 8
            optional pub recommended_refresh_interval: u32 => { bit: 6, key: "recommendedRefreshInterval", when: ((eq(&loc_type, &2u64)) && (eq(&data_type, &4u64))), predicate_value: false },
        }
        steps {
            1 => required loc_type: RetAccessoryDataLocType => { bit: 0, key: "locType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required data_type: RetAccessoryDataDataTypeDisjoint => { bit: 1, key: "dataType", wire: [disjoint(decode_location_assistance_data_type, encode_location_assistance_data_type, RetAccessoryDataLocationAssistanceDataType, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_location_assistance_data_type)) }, child: &Iap1WireSpec::Scalar }, predicate_value: true, predicate: { expr: eq(&loc_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            3 => required data_type: RetAccessoryDataDataTypeDisjoint => { bit: 1, key: "dataType", wire: [disjoint(decode_unknown_data_type, encode_unknown_data_type, RetAccessoryDataUnknownDataType, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_unknown_data_type)) }, child: &Iap1WireSpec::Scalar }, predicate_value: true, predicate: { expr: ne(&loc_type, &2u64), program: [FlatPredicateToken::Ne, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            4 => required sect_cur: u16 => { bit: 2, key: "sectCur", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            5 => required sect_max: u16 => { bit: 3, key: "sectMax", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            6 => optional total_size: u32 => { bit: 4, key: "totalSize", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&sect_cur, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(2), FlatPredicateToken::Integer(0u8)] } },
            7 => optional max_required_refresh_interval: u32 => { bit: 5, key: "maxRequiredRefreshInterval", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&loc_type, &2u64)) && (eq(&data_type, &3u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(3u8)] } },
            8 => optional recommended_refresh_interval: u32 => { bit: 6, key: "recommendedRefreshInterval", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&loc_type, &2u64)) && (eq(&data_type, &4u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(4u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetAccessoryDataLocType: u8 {
        /// NMEA GPS location
        NMEAGPSLocation = 1,
        /// location assistance
        LocationAssistance = 2,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetAccessoryDataNMEAGPSLocationDataType: u8 {
        /// NMEA sentence filter list string
        NMEASentenceFilterListString = 0,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetAccessoryDataLocationAssistanceDataType: u8 {
        /// current point location data
        CurrentPointLocationData = 0,
        /// satellite ephemeris data
        SatelliteEphemerisData = 1,
        /// current GPS time on accessory
        CurrentGPSTimeOnAccessory = 5,
    }
}

/// The specification lists no named values for this enum field.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SetAccessoryDataUnknownDataType(pub u8);
impl crate::__NamedDebug for SetAccessoryDataUnknownDataType {
    fn fmt_named(&self, name: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple(name).field(self).finish()
    }
}

impl crate::Iap1Decode for SetAccessoryDataUnknownDataType {
    fn decode(reader: &mut crate::Reader<'_>) -> Result<Self, crate::DecodeError> {
        Ok(Self(<u8 as crate::Iap1Decode>::decode(reader)?))
    }
}
impl crate::Iap1Encode for SetAccessoryDataUnknownDataType {
    fn encode(&self, writer: &mut crate::Writer<'_>) -> Result<(), crate::EncodeError> {
        crate::Iap1Encode::encode(&self.0, writer)
    }
}
impl crate::PredicateValue for SetAccessoryDataUnknownDataType {
    fn predicate_integer(&self) -> Option<i128> {
        Some(self.0 as i128)
    }
}

iap1_disjoint! {
    strings = LingoString;
    #[iap1_disjoint(predicate = integer)]
    pub enum SetAccessoryDataDataTypeDisjoint {
        #[iap1_disjoint(decode = decode_nmeagps_location_data_type, encode = encode_nmeagps_location_data_type)]
        /// NMEA GPS Location Data Type
        NMEAGPSLocationDataType(SetAccessoryDataNMEAGPSLocationDataType),
        #[iap1_disjoint(decode = decode_location_assistance_data_type, encode = encode_location_assistance_data_type)]
        /// Location Assistance Data Type
        LocationAssistanceDataType(SetAccessoryDataLocationAssistanceDataType),
        #[iap1_disjoint(decode = decode_unknown_data_type, encode = encode_unknown_data_type)]
        /// Unknown Data Type
        UnknownDataType(SetAccessoryDataUnknownDataType),
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0e, command = 0x0008,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetAccessoryData {
        fields {
            /// Capability Type
            // Virtual schema indices: 1
            required pub loc_type: SetAccessoryDataLocType => { bit: 0, key: "locType", when: (truthy(&true)), predicate_value: true },
            /// NMEA GPS Location Data Type
            // Virtual schema indices: 2, 3, 4
            required pub data_type: SetAccessoryDataDataTypeDisjoint => { bit: 1, key: "dataType", when: (eq(&loc_type, &1u64)) || (eq(&loc_type, &2u64)) || ((ne(&loc_type, &1u64)) && (ne(&loc_type, &2u64))), predicate_value: true },
            /// Current Payload Section Index
            // Virtual schema indices: 5
            required pub sect_cur: u16 => { bit: 2, key: "sectCur", when: (truthy(&true)), predicate_value: true },
            /// Maximum Payload Section Index
            // Virtual schema indices: 6
            required pub sect_max: u16 => { bit: 3, key: "sectMax", when: (truthy(&true)), predicate_value: false },
            /// Total Size
            // Virtual schema indices: 7
            optional pub total_size: u32 => { bit: 4, key: "totalSize", when: (eq(&sect_cur, &0u64)), predicate_value: false },
            /// NMEA Sentence Filter List String
            // Virtual schema indices: 8
            optional pub filter_list_string: Vec<u8> => { bit: 5, key: "filterListString", when: ((eq(&loc_type, &1u64)) && (eq(&data_type, &0u64))), predicate_value: false },
            /// GPS Week Number
            // Virtual schema indices: 9, 15
            optional pub loc_week: u16 => { bit: 6, key: "locWeek", when: ((eq(&loc_type, &2u64)) && (eq(&data_type, &0u64))) || ((eq(&loc_type, &2u64)) && (eq(&data_type, &5u64))), predicate_value: false },
            /// GPS Time of Week
            // Virtual schema indices: 10, 16
            optional pub loc_time: u32 => { bit: 7, key: "locTime", when: ((eq(&loc_type, &2u64)) && (eq(&data_type, &0u64))) || ((eq(&loc_type, &2u64)) && (eq(&data_type, &5u64))), predicate_value: false },
            /// Latitude
            // Virtual schema indices: 11
            optional pub lat_deg_arc: i32 => { bit: 8, key: "latDegArc", when: ((eq(&loc_type, &2u64)) && (eq(&data_type, &0u64))), predicate_value: false },
            /// Longitude
            // Virtual schema indices: 12
            optional pub lon_dec_arc: i32 => { bit: 9, key: "lonDecArc", when: ((eq(&loc_type, &2u64)) && (eq(&data_type, &0u64))), predicate_value: false },
            /// Point Location Accuracy Radius
            // Virtual schema indices: 13
            optional pub loc_accuracy_radius: u16 => { bit: 10, key: "locAccuracyRadius", when: ((eq(&loc_type, &2u64)) && (eq(&data_type, &0u64))), predicate_value: false },
            /// Satellite Ephemeris Data
            // Virtual schema indices: 14
            optional pub eph_data: Vec<u8> => { bit: 11, key: "ephData", when: ((eq(&loc_type, &2u64)) && (eq(&data_type, &1u64))), predicate_value: false },
        }
        steps {
            1 => required loc_type: SetAccessoryDataLocType => { bit: 0, key: "locType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required data_type: SetAccessoryDataDataTypeDisjoint => { bit: 1, key: "dataType", wire: [disjoint(decode_nmeagps_location_data_type, encode_nmeagps_location_data_type, SetAccessoryDataNMEAGPSLocationDataType, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_nmeagps_location_data_type)) }, child: &Iap1WireSpec::Scalar }, predicate_value: true, predicate: { expr: eq(&loc_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            3 => required data_type: SetAccessoryDataDataTypeDisjoint => { bit: 1, key: "dataType", wire: [disjoint(decode_location_assistance_data_type, encode_location_assistance_data_type, SetAccessoryDataLocationAssistanceDataType, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_location_assistance_data_type)) }, child: &Iap1WireSpec::Scalar }, predicate_value: true, predicate: { expr: eq(&loc_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            4 => required data_type: SetAccessoryDataDataTypeDisjoint => { bit: 1, key: "dataType", wire: [disjoint(decode_unknown_data_type, encode_unknown_data_type, SetAccessoryDataUnknownDataType, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_unknown_data_type)) }, child: &Iap1WireSpec::Scalar }, predicate_value: true, predicate: { expr: (ne(&loc_type, &1u64)) && (ne(&loc_type, &2u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Ne, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8), FlatPredicateToken::Ne, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            5 => required sect_cur: u16 => { bit: 2, key: "sectCur", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            6 => required sect_max: u16 => { bit: 3, key: "sectMax", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            7 => optional total_size: u32 => { bit: 4, key: "totalSize", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&sect_cur, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(2), FlatPredicateToken::Integer(0u8)] } },
            8 => optional filter_list_string: Vec<u8> => { bit: 5, key: "filterListString", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: (eq(&loc_type, &1u64)) && (eq(&data_type, &0u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(0u8)] } },
            9 => optional loc_week: u16 => { bit: 6, key: "locWeek", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&loc_type, &2u64)) && (eq(&data_type, &0u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(0u8)] } },
            10 => optional loc_time: u32 => { bit: 7, key: "locTime", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&loc_type, &2u64)) && (eq(&data_type, &0u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(0u8)] } },
            11 => optional lat_deg_arc: i32 => { bit: 8, key: "latDegArc", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&loc_type, &2u64)) && (eq(&data_type, &0u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(0u8)] } },
            12 => optional lon_dec_arc: i32 => { bit: 9, key: "lonDecArc", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&loc_type, &2u64)) && (eq(&data_type, &0u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(0u8)] } },
            13 => optional loc_accuracy_radius: u16 => { bit: 10, key: "locAccuracyRadius", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&loc_type, &2u64)) && (eq(&data_type, &0u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(0u8)] } },
            14 => optional eph_data: Vec<u8> => { bit: 11, key: "ephData", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: (eq(&loc_type, &2u64)) && (eq(&data_type, &1u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(1u8)] } },
            15 => optional loc_week: u16 => { bit: 6, key: "locWeek", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&loc_type, &2u64)) && (eq(&data_type, &5u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(5u8)] } },
            16 => optional loc_time: u32 => { bit: 7, key: "locTime", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&loc_type, &2u64)) && (eq(&data_type, &5u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(5u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AsyncAccessoryDataLocType: u8 {
        /// NMEA GPS location
        NMEAGPSLocation = 1,
        /// location assistance
        LocationAssistance = 2,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AsyncAccessoryDataNMEAGPSLocationDataType: u8 {
        /// NMEA sentences
        NMEASentences = 128,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum AsyncAccessoryDataLocationAssistanceDataType: u8 {
        /// satellite ephemeris data
        SatelliteEphemerisData = 2,
        /// request to set current GPS time on accessory
        RequestToSetCurrentGPSTimeOnAccessory = 5,
    }
}

/// The specification lists no named values for this enum field.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AsyncAccessoryDataUnknownDataType(pub u8);
impl crate::__NamedDebug for AsyncAccessoryDataUnknownDataType {
    fn fmt_named(&self, name: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple(name).field(self).finish()
    }
}

impl crate::Iap1Decode for AsyncAccessoryDataUnknownDataType {
    fn decode(reader: &mut crate::Reader<'_>) -> Result<Self, crate::DecodeError> {
        Ok(Self(<u8 as crate::Iap1Decode>::decode(reader)?))
    }
}
impl crate::Iap1Encode for AsyncAccessoryDataUnknownDataType {
    fn encode(&self, writer: &mut crate::Writer<'_>) -> Result<(), crate::EncodeError> {
        crate::Iap1Encode::encode(&self.0, writer)
    }
}
impl crate::PredicateValue for AsyncAccessoryDataUnknownDataType {
    fn predicate_integer(&self) -> Option<i128> {
        Some(self.0 as i128)
    }
}

iap1_disjoint! {
    strings = LingoString;
    #[iap1_disjoint(predicate = integer)]
    pub enum AsyncAccessoryDataDataTypeDisjoint {
        #[iap1_disjoint(decode = decode_nmeagps_location_data_type, encode = encode_nmeagps_location_data_type)]
        /// NMEA GPS Location Data Type
        NMEAGPSLocationDataType(AsyncAccessoryDataNMEAGPSLocationDataType),
        #[iap1_disjoint(decode = decode_location_assistance_data_type, encode = encode_location_assistance_data_type)]
        /// Location Assistance Data Type
        LocationAssistanceDataType(AsyncAccessoryDataLocationAssistanceDataType),
        #[iap1_disjoint(decode = decode_unknown_data_type, encode = encode_unknown_data_type)]
        /// Unknown Data Type
        UnknownDataType(AsyncAccessoryDataUnknownDataType),
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0e, command = 0x0009,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct AsyncAccessoryData {
        fields {
            /// Capability Type
            // Virtual schema indices: 1
            required pub loc_type: AsyncAccessoryDataLocType => { bit: 0, key: "locType", when: (truthy(&true)), predicate_value: true },
            /// NMEA GPS Location Data Type
            // Virtual schema indices: 2, 3, 4
            required pub data_type: AsyncAccessoryDataDataTypeDisjoint => { bit: 1, key: "dataType", when: (eq(&loc_type, &1u64)) || (eq(&loc_type, &2u64)) || ((ne(&loc_type, &1u64)) && (ne(&loc_type, &2u64))), predicate_value: true },
            /// Current Payload Section Index
            // Virtual schema indices: 5
            required pub sect_cur: u16 => { bit: 2, key: "sectCur", when: (truthy(&true)), predicate_value: true },
            /// Maximum Payload Section Index
            // Virtual schema indices: 6
            required pub sect_max: u16 => { bit: 3, key: "sectMax", when: (truthy(&true)), predicate_value: false },
            /// Total Size
            // Virtual schema indices: 7
            optional pub total_size: u32 => { bit: 4, key: "totalSize", when: (eq(&sect_cur, &0u64)), predicate_value: false },
            /// NMEA Sentences
            // Virtual schema indices: 8
            optional pub nmea_sentences: Vec<u8> => { bit: 5, key: "nmeaSentences", when: ((eq(&loc_type, &1u64)) && (eq(&data_type, &128u64))), predicate_value: false },
            /// Satellite Ephemeris Data URL
            // Virtual schema indices: 9
            optional pub eph_data_url_string: Vec<u8> => { bit: 6, key: "ephDataURLString", when: ((eq(&loc_type, &2u64)) && (eq(&data_type, &2u64))), predicate_value: false },
        }
        steps {
            1 => required loc_type: AsyncAccessoryDataLocType => { bit: 0, key: "locType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required data_type: AsyncAccessoryDataDataTypeDisjoint => { bit: 1, key: "dataType", wire: [disjoint(decode_nmeagps_location_data_type, encode_nmeagps_location_data_type, AsyncAccessoryDataNMEAGPSLocationDataType, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_nmeagps_location_data_type)) }, child: &Iap1WireSpec::Scalar }, predicate_value: true, predicate: { expr: eq(&loc_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            3 => required data_type: AsyncAccessoryDataDataTypeDisjoint => { bit: 1, key: "dataType", wire: [disjoint(decode_location_assistance_data_type, encode_location_assistance_data_type, AsyncAccessoryDataLocationAssistanceDataType, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_location_assistance_data_type)) }, child: &Iap1WireSpec::Scalar }, predicate_value: true, predicate: { expr: eq(&loc_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            4 => required data_type: AsyncAccessoryDataDataTypeDisjoint => { bit: 1, key: "dataType", wire: [disjoint(decode_unknown_data_type, encode_unknown_data_type, AsyncAccessoryDataUnknownDataType, [scalar])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_unknown_data_type)) }, child: &Iap1WireSpec::Scalar }, predicate_value: true, predicate: { expr: (ne(&loc_type, &1u64)) && (ne(&loc_type, &2u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Ne, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8), FlatPredicateToken::Ne, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            5 => required sect_cur: u16 => { bit: 2, key: "sectCur", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            6 => required sect_max: u16 => { bit: 3, key: "sectMax", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            7 => optional total_size: u32 => { bit: 4, key: "totalSize", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&sect_cur, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(2), FlatPredicateToken::Integer(0u8)] } },
            8 => optional nmea_sentences: Vec<u8> => { bit: 5, key: "nmeaSentences", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: (eq(&loc_type, &1u64)) && (eq(&data_type, &128u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(128u8)] } },
            9 => optional eph_data_url_string: Vec<u8> => { bit: 6, key: "ephDataURLString", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: (eq(&loc_type, &2u64)) && (eq(&data_type, &2u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(2u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum IPodAckCommandResult: u8 {
        /// OK
        OK = 0,
        /// unknown database category or session ID
        UnknownDatabaseCategoryOrSessionID = 1,
        /// command failed
        CommandFailed = 2,
        /// Apple device out of resources
        AppleDeviceOutOfResources = 3,
        /// bad parameter
        BadParameter = 4,
        /// unknown ID
        UnknownID = 5,
        /// command pending
        CommandPending = 6,
        /// not authenticated
        NotAuthenticated = 7,
        /// bad authentication version
        BadAuthenticationVersion = 8,
        /// accessory power mode request failed
        AccessoryPowerModeRequestFailed = 9,
        /// certificate invalid
        CertificateInvalid = 10,
        /// certificate permissions invalid
        CertificatePermissionsInvalid = 11,
        /// file in use
        FileInUse = 12,
        /// invalid file handle
        InvalidFileHandle = 13,
        /// directory not empty
        DirectoryNotEmpty = 14,
        /// operation timed out
        OperationTimedOut = 15,
        /// command unavailable in this iPod mode
        CommandUnavailableInThisIPodMode = 16,
        /// Accessory Detect not grounded or invalid Accessory Identify Resistor detected
        AccessoryDetectNotGroundedOrInvalidAccessoryIdentifyResistorDetected = 17,
        /// selection not genius
        SelectionNotGenius = 18,
        /// multisection data section received successfully
        MultisectionDataSectionReceivedSuccessfully = 19,
        /// lingo busy
        LingoBusy = 20,
        /// maximum number of accessory connections already reached
        MaximumNumberOfAccessoryConnectionsAlreadyReached = 21,
        /// HID descriptor index already in use
        HIDDescriptorIndexAlreadyInUse = 22,
        /// dropped data (SessionWriteFailure)
        DroppedDataSessionWriteFailure = 23,
        /// attempt to enter iPod Out mode with incompatible video settings
        AttemptToEnterIPodOutModeWithIncompatibleVideoSettings = 24,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x0e, command = 0x0080,
        source = device,
        response = true,
        ack = true,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct IPodAck {
        fields {
            /// Command Result
            // Virtual schema indices: 1
            required pub command_result: IPodAckCommandResult => { bit: 0, key: "commandResult", when: (truthy(&true)), predicate_value: true },
            /// Command ID
            // Virtual schema indices: 2
            required pub acked_command_id: u8 => { bit: 1, key: "ackedCommandID", when: (truthy(&true)), predicate_value: false },
            /// Maximum Pending Wait
            // Virtual schema indices: 3
            optional pub maximum_pending_wait: u32 => { bit: 2, key: "maximumPendingWait", when: (eq(&command_result, &6u64)), predicate_value: false },
            /// Session ID
            // Virtual schema indices: 4
            optional pub session_id: u16 => { bit: 3, key: "sessionID", when: (eq(&command_result, &23u64)), predicate_value: false },
            /// Number of Bytes Dropped
            // Virtual schema indices: 5
            optional pub num_bytes_dropped: u32 => { bit: 4, key: "numBytesDropped", when: (eq(&command_result, &23u64)), predicate_value: false },
            /// Current Section Index
            // Virtual schema indices: 6
            optional pub sect_cur: u16 => { bit: 5, key: "sectCur", when: (eq(&command_result, &19u64)), predicate_value: false },
        }
        steps {
            1 => required command_result: IPodAckCommandResult => { bit: 0, key: "commandResult", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required acked_command_id: u8 => { bit: 1, key: "ackedCommandID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => optional maximum_pending_wait: u32 => { bit: 2, key: "maximumPendingWait", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&command_result, &6u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(6u8)] } },
            4 => optional session_id: u16 => { bit: 3, key: "sessionID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&command_result, &23u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(23u8)] } },
            5 => optional num_bytes_dropped: u32 => { bit: 4, key: "numBytesDropped", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&command_result, &23u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(23u8)] } },
            6 => optional sect_cur: u16 => { bit: 5, key: "sectCur", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&command_result, &19u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(19u8)] } },
        }
    }
}

iap1_registry! {
    0x0e;
    AccessoryAck => 0x0000,
    GetAccessoryCaps => 0x0001,
    RetAccessoryCaps => 0x0002,
    GetAccessoryControl => 0x0003,
    RetAccessoryControl => 0x0004,
    SetAccessoryControl => 0x0005,
    GetAccessoryData => 0x0006,
    RetAccessoryData => 0x0007,
    SetAccessoryData => 0x0008,
    AsyncAccessoryData => 0x0009,
    IPodAck => 0x0080,
}
