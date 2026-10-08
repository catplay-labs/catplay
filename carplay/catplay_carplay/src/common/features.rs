use catplay_plist::plist_bitflags;

plist_bitflags! {
    pub struct AirPlayFeature: u64 {
        /// Screen mirroring.
        const SCREEN                       = 1 << 7;
        /// Can rotate during screen mirroring.
        const ROTATE                       = 1 << 8;
        /// Audio.
        const AUDIO                        = 1 << 9;
        /// Redundant audio packets to handle loss.
        const REDUNDANT_AUDIO              = 1 << 11;
        /// Artwork meta data for audio streams.
        const AUDIO_METADATA_ARTWORK       = 1 << 15;
        /// Progress meta data for audio streams.
        const AUDIO_METADATA_PROGRESS      = 1 << 16;
        /// Textual meta data for audio streams.
        const AUDIO_METADATA_TEXT          = 1 << 17;
        /// Uncompressed PCM audio data.
        const AUDIO_PCM                    = 1 << 18;
        /// ALAC audio compression. **[non-CarPlay flag]**
        const AUDIO_ALAC                = 1 << 19;
        /// AAC-LC audio compression.
        const AUDIO_AAC_LC                 = 1 << 20;
        /// AAC_ELD audio compression. **[non-CarPlay flag]**
        const AUDIO_AAC_ELD                = 1 << 21;

        /// Unencrypted audio.
        const AUDIO_UNENCRYPTED            = 1 << 22;
        /// Audio: 128-bit AES key encrypted with MFi-SAPv1.
        const AUDIO_AES_128_MFI_SAP_V1     = 1 << 26;
        /// Pairing support.
        const PAIRING                      = 1 << 27;
        /// Unified meta data in a single plist request.
        const UNIFIED_METADATA             = 1 << 29;
        /// _airplay._tcp and _raop._tcp are unified (_raop._tcp for compatibility only).
        const UNIFIED_BONJOUR              = 1 << 30;
        /// Reserved to avoid client bugs treating this as a sign bit.
        const RESERVED                     = 1 << 31;
        /// Car support.
        const CAR                          = 1 << 32;
        /// Car initiates the connection through CarPlayStartSession or CarPlayControl.
        const CARPLAY_CONTROL              = 1 << 37;
        /// Control channel encryption.
        const HK_PAIRING_AND_ENCRYPT       = 1 << 38;
        /// Support for Buffered audio playback
        const BUFFERED_AUDIO = 1 << 40;
        /// Supports IEEE 1588 clock.
        const SUPPORTS_1588_CLOCK          = 1 << 41;
        /// Screen multi-codec support (HEVC). **[non-CarPlay flag]**
        const SUPPORTS_SCREEN_MULTI_CODEC  = 1 << 42;
        /// Supports system pairing.
        const SYSTEM_PAIRING               = 1 << 43;
        /// Supports hardware timestamps for IEEE 1588 clock.
        const SUPPORTS_HARDWARE_1588_CLOCK = 1 << 45;
        /// Supports HK pairing and access control.
        const SUPPORTS_HK_PAIRING_AND_ACCESS_CONTROL = 1 << 46;
        /// Supports HK peer management.
        const SUPPORTS_HK_PEER_MANAGEMENT  = 1 << 47;
        /// Supports transient pairing.
        const TRANSIENT_PAIRING             = 1 << 48;
        /// Supports extended message format for setting 1588 clock peers.
        const SUPPORTS_SET_PEERS_EXTENDED_MESSAGE = 1 << 52;

        // FairPlay

        /// Authentication type 4. FairPlay authentication. **[non-CarPlay flag]**
        const AUTHENTICATION4              = 1 << 14;
        /// Video protected with FairPlay DRM. **[non-CarPlay flag]**
        const VIDEO_FAIRPLAY               = 1 << 2;
        /// FairPlay secure auth supported. **[non-CarPlay flag]**
        const FPSAP_V2PT5_AES_GCM          = 1 << 12;

    }
}

plist_bitflags! {
    pub struct AirPlayStatus: u8 {
        /// Problem has been detected.
        const PROBLEM                       = 1 << 0;
        /// Device is not configured.
        const UNCONFIGURED                  = 1 << 1;
        /// Audio cable is attached.
        const AUDIO                        = 1 << 2;
    }
}

plist_bitflags! {
    pub struct AirPlayCompressionType: u8 {
        /// Uncompressed PCM.
        const PCM                       = 1 << 0;
        /// Apple Lossless (ALAC)
        const ALAC = 1 << 1;
        /// AAC Low Complexity (AAC-LC).
        const AAC_LC                  = 1 << 2;
        /// AAC Enhanced Low Delay (AAC-ELD).
        const AAC_ELD                        = 1 << 3;
        /// H.264 video.
        const H264                        = 1 << 4;
        /// Opus
        const OPUS                        = 1 << 5;
    }
}

plist_bitflags! {
    pub struct AirPlayEncryptionType: u8 {
        /// If set, encryption is not required.
        const NONE                       = 1 << 0;
        /// If set, 128-bit AES key encrypted with MFi-SAPv1 is supported.
        const MFI_SAPv1                  = 1 << 4;
        /// If set, 128-bit AES key encrypted with FairPlay is supported.
        const FAIRPLAY = 1 << 5;
    }
}
