use crate::common::{AirPlayFeature, AirPlayStatus};

use catplay_bonjour::{BonjourEntryType, get_bonjour_txt_optional};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AirPlayBonjourEntry {
    /// SDK version
    pub srcvers: String,
    /// Homekit device id (for pairing fast-path on reconnect)
    pub pi: Option<Uuid>,
    /// HomeKit pairing key in hex (optional addition)
    pub pk: String,

    pub protovers: String,
    pub model: String,
    pub flags: AirPlayStatus,
    pub features: AirPlayFeature,

    /// MAC address on the Wi-Fi interface of the server (AA:BB:CC:EE:DD)
    pub deviceid: String,

    // Non-CarPlay additions
    /// Vodka Version(FairPlay); valid value is 2
    pub vv: String,
    /// pw=false means no password required for pairing
    pub pw: String,
}

fn airplay_features_bonjour(features: AirPlayFeature) -> String {
    let low = (features.bits() & 0xFFFF_FFFF) as u32;
    let high = (features.bits() >> 32) as u32;
    format!("0x{low:08X},0x{high:X}")
}

fn parse_airplay_features_bonjour(s: &str) -> Option<AirPlayFeature> {
    let mut parts = s.split(',').map(str::trim);

    let low_str = parts.next()?;
    let high_str = parts.next()?;

    let low = u32::from_str_radix(low_str.trim_start_matches("0x"), 16).ok()?;
    let high = u32::from_str_radix(high_str.trim_start_matches("0x"), 16).ok()?;
    let val = ((high as u64) << 32) | (low as u64);

    Some(AirPlayFeature::from_bits_truncate(val))
}

fn airplay_flags_bonjour(flags: AirPlayStatus) -> String {
    let flags = flags.bits();
    format!("0x{flags:02X}")
}

fn parse_airplay_flags_bonjour(s: &str) -> Option<AirPlayStatus> {
    u8::from_str_radix(s.trim_start_matches("0x"), 16)
        .map(AirPlayStatus::from_bits_retain)
        .ok()
}

impl BonjourEntryType for AirPlayBonjourEntry {
    const SERVICE_TYPE: &'static str = "_airplay._tcp.local.";
    type Entry = AirPlayBonjourEntry;

    fn from_props(data: &HashMap<String, String>) -> AirPlayBonjourEntry {
        Self {
            srcvers: get_bonjour_txt_optional(data, "srcvers"),
            pi: Uuid::parse_str(&get_bonjour_txt_optional(data, "pi")).ok(),
            pk: get_bonjour_txt_optional(data, "pk"),

            protovers: get_bonjour_txt_optional(data, "protovers"),
            model: get_bonjour_txt_optional(data, "model"),
            deviceid: get_bonjour_txt_optional(data, "deviceid"),

            flags: parse_airplay_flags_bonjour(&get_bonjour_txt_optional(data, "flags")).unwrap_or(AirPlayStatus::empty()),
            features: parse_airplay_features_bonjour(&get_bonjour_txt_optional(data, "features")).unwrap_or(AirPlayFeature::empty()),

            vv: get_bonjour_txt_optional(data, "vv"),
            pw: get_bonjour_txt_optional(data, "pw"),
        }
    }

    fn to_props(data: AirPlayBonjourEntry, output: &mut HashMap<String, String>) {
        output.insert("srcvers".into(), data.srcvers.clone());
        if let Some(pi) = data.pi {
            output.insert("pi".into(), pi.to_string());
        }
        if !data.pk.is_empty() {
            output.insert("pk".into(), data.pk.clone());
        }

        output.insert("protovers".into(), data.protovers.clone());
        output.insert("model".into(), data.model.clone());
        output.insert("flags".into(), airplay_flags_bonjour(data.flags));
        output.insert("features".into(), airplay_features_bonjour(data.features));
        output.insert("deviceid".into(), data.deviceid.clone());

        if !data.vv.is_empty() {
            output.insert("vv".into(), data.vv.clone());
        }
        if !data.pw.is_empty() {
            output.insert("pw".into(), data.pw.clone());
        }
    }
}

impl AirPlayBonjourEntry {
    pub fn to_rtsp_info_string(&self) -> Vec<u8> {
        let mut out = HashMap::new();
        let mut s = Vec::new();
        Self::to_props(self.clone(), &mut out);

        for (k, v) in out {
            let chunk = format!("{k}={v}");
            let bytes = chunk.as_bytes();
            let len = bytes.len().min(u8::MAX as usize) as u8;
            s.push(len);
            s.extend_from_slice(&bytes[..len as usize]);
        }
        s
    }
}
