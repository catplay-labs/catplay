#[cfg(test)]
mod tests {
    use catplay_csm::{decoder::*, msg::IdentificationInformation};

    #[test]
    fn test_ident_ids_round_trip() {
        let ids = [0x1D02, 0x4E0E, 0xAE00, 0x4E0A];
        let packed = IdentificationInformation::pack_ids(&ids);
        assert_eq!(packed.data, vec![0x1D, 0x02, 0x4E, 0x0E, 0xAE, 0x00, 0x4E, 0x0A]);
        assert_eq!(IdentificationInformation::unpack_ids(&packed), ids);
    }

    #[test]
    fn test_ident_has_id() {
        let packed = IdentificationInformation::pack_ids(&[0x1D02, 0x4E0E, 0xAE00]);
        assert!(IdentificationInformation::has_id(&packed.data, 0x1D02));
        assert!(IdentificationInformation::has_id(&packed.data, 0x4E0E));
        assert!(IdentificationInformation::has_id(&packed.data, 0xAE00));
        // must not match across the boundary of two IDs (0x02 0x4E)
        assert!(!IdentificationInformation::has_id(&packed.data, 0x024E));
        assert!(!IdentificationInformation::has_id(&packed.data, 0x4E0A));
        assert!(!IdentificationInformation::has_id(&[], 0x1D02));
        // a trailing odd byte is ignored
        assert!(!IdentificationInformation::has_id(&[0x1D], 0x1D00));
        assert_eq!(
            IdentificationInformation::unpack_ids(&CsmByteArray::new(vec![0x1D, 0x02, 0x4E])),
            vec![0x1D02]
        );
    }
}
