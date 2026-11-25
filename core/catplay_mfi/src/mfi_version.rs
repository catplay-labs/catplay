/// MFi authentication generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MfiVersion {
    V2,
    V3,
}

/// Identifies an MFi generation from its certificate bytes. Unknown lengths
/// fall back to MFi 2.0 for compatibility with older devices.
pub fn detect_mfi_version(certificate: &[u8]) -> MfiVersion {
    match certificate.len() {
        607..=609 => MfiVersion::V3,
        _ => MfiVersion::V2,
    }
}

#[cfg(test)]
mod tests {
    use super::{MfiVersion, detect_mfi_version};

    #[test]
    fn detects_mfi_version_from_certificate_bytes() {
        for len in [607, 608, 609] {
            assert_eq!(detect_mfi_version(&vec![0; len]), MfiVersion::V3);
        }

        for len in [0, 1, 606, 610, 1280, 1281] {
            assert_eq!(detect_mfi_version(&vec![0; len]), MfiVersion::V2);
        }
    }
}
