use catplay_plist::{plist_bitflags, plist_enum, plist_enum_repr};

plist_enum_repr! {
    #[repr(u8)]
    pub enum StreamType {
        #[default]
        Invalid = 0,

        GeneralAudio = 96,

        MainAudio = 100,
        AltAudio = 101,
        MainHighAudio = 102,
        Screen = 110,

        // Modern CarPlay
        /*BufferedAudio = 103,
        AuxOutAudio = 106,
        AuxInAudio = 107,
        AltScreen = 111,
        MainAudioWithRedundancy = 104,
        AltAudioWithRedundancy = 105,
        AuxOutAudioWithRedundancy = 108,
        AuxInAudioWithRedundancy = 109*/
    }
}

impl StreamType {
    pub fn supports_legacy_audio_aes(&self) -> bool {
        matches!(
            self,
            StreamType::GeneralAudio | StreamType::MainHighAudio /* | StreamType::BufferedAudio */
        )
    }
}

plist_enum! {
    pub enum AudioType {
        /// Main Audio, Alt Audio
        #[default]
        Default,
        /// Main Audio
        Alert,
        /// Main Audio, Main High Audio
        Media,
        /// Main Audio
        Telephony,
        /// Main Audio
        SpeechRecognition,
        /// Main Audio, Alt Audio
        Compatibility
    }
}

plist_bitflags! {
    pub struct AudioFormat: u128 {
        // 0, 1: reserved

        /// PCM, 8000 Hz, 16-Bit, Mono.
        const PCM_8000_MONO                 = 1u128 <<  2;
        /// PCM, 8000 Hz, 16-Bit, Stereo.
        const PCM_8000_STEREO               = 1u128 <<  3;
        /// PCM, 16000 Hz, 16-Bit, Mono.
        const PCM_16000_MONO                = 1u128 <<  4;
        /// PCM, 16000 Hz, 16-Bit, Stereo.
        const PCM_16000_STEREO              = 1u128 <<  5;
        /// PCM, 24000 Hz, 16-Bit, Mono.
        const PCM_24000_MONO                = 1u128 <<  6;
        /// PCM, 24000 Hz, 16-Bit, Stereo.
        const PCM_24000_STEREO              = 1u128 <<  7;
        /// PCM, 32000 Hz, 16-Bit, Mono.
        const PCM_32000_MONO                = 1u128 <<  8;
        /// PCM, 32000 Hz, 16-Bit, Stereo.
        const PCM_32000_STEREO              = 1u128 <<  9;
        /// PCM, 44100 Hz, 16-Bit, Mono.
        const PCM_44100_MONO                = 1u128 << 10;
        /// PCM, 44100 Hz, 16-Bit, Stereo.
        const PCM_44100_STEREO              = 1u128 << 11;
        /// PCM, 44100 Hz, 24-Bit, Mono.
        const PCM_44100_24_MONO             = 1u128 << 12;
        /// PCM, 44100 Hz, 24-Bit, Stereo.
        const PCM_44100_24_STEREO           = 1u128 << 13;
        /// PCM, 48000 Hz, 16-Bit, Mono.
        const PCM_48000_MONO                = 1u128 << 14;
        /// PCM, 48000 Hz, 16-Bit, Stereo.
        const PCM_48000_STEREO              = 1u128 << 15;
        /// PCM, 48000 Hz, 24-Bit, Mono.
        const PCM_48000_24_MONO             = 1u128 << 16;
        /// PCM, 48000 Hz, 24-Bit, Stereo.
        const PCM_48000_24_STEREO           = 1u128 << 17;

        /// ALAC, 44100 Hz, 16-Bit, Stereo.
        const ALAC_44100_16_STEREO          = 1u128 << 18;
        /// ALAC, 44100 Hz, 24-Bit, Stereo.
        const ALAC_44100_24_STEREO          = 1u128 << 19;
        /// ALAC, 48000 Hz, 16-Bit, Stereo.
        const ALAC_48000_16_STEREO          = 1u128 << 20;
        /// ALAC, 48000 Hz, 24-Bit, Stereo.
        const ALAC_48000_24_STEREO          = 1u128 << 21;

        /// AAC-LC, 44100 Hz, Stereo.
        const AAC_LC_44100_STEREO           = 1u128 << 22;
        /// AAC-LC, 48000 Hz, Stereo.
        const AAC_LC_48000_STEREO           = 1u128 << 23;

        /// AAC-ELD, 44100 Hz, Stereo.
        const AAC_ELD_44100_STEREO          = 1u128 << 24;
        /// AAC-ELD, 48000 Hz, Stereo.
        const AAC_ELD_48000_STEREO          = 1u128 << 25;
        /// AAC-ELD, 16000 Hz, Mono.
        const AAC_ELD_16000_MONO            = 1u128 << 26;
        /// AAC-ELD, 24000 Hz, Mono.
        const AAC_ELD_24000_MONO            = 1u128 << 27;

        /// OPUS, 16000 Hz, Mono.
        const OPUS_16000_MONO               = 1u128 << 28;
        /// OPUS, 24000 Hz, Mono.
        const OPUS_24000_MONO               = 1u128 << 29;
        /// OPUS, 48000 Hz, Mono.
        const OPUS_48000_MONO               = 1u128 << 30;

        /// AAC-ELD, 44100 Hz, Mono.
        const AAC_ELD_44100_MONO            = 1u128 << 31;
        /// AAC-ELD, 48000 Hz, Mono.
        const AAC_ELD_48000_MONO            = 1u128 << 32;

        /// QC3, 48000 Hz, 5.1.2.
        const QC3_48000_5_1_2               = 1u128 << 33;
        /// QC3, 48000 Hz, 7.1.4.
        const QC3_48000_7_1_4               = 1u128 << 34;
        /// QC3, 48000 Hz, 9.1.6.
        const QC3_48000_9_1_6               = 1u128 << 35;

        // 36, 37: unknown

        /// PCM, 48000 Hz, 16-Bit, 5.1.2.
        const PCM_48000_5_1_2               = 1u128 << 38;

        /// AAC-LC, 48000 Hz, 5.1.
        const AAC_LC_48000_5_1              = 1u128 << 39;
        /// AAC-LC, 48000 Hz, 5.1.2.
        const AAC_LC_48000_5_1_2            = 1u128 << 40;

        /// AAC-ELD, 48000 Hz, 5.1.
        const AAC_ELD_48000_5_1             = 1u128 << 41;
        /// AAC-ELD, 48000 Hz, 5.1.2.
        const AAC_ELD_48000_5_1_2           = 1u128 << 42;
        /// AAC-ELD, 32000 Hz, Mono.
        const AAC_ELD_32000_MONO            = 1u128 << 43;

        /// PCM, 48000 Hz, 16-Bit, 5.1.
        const PCM_48000_5_1                 = 1u128 << 44;
        /// PCM, 48000 Hz, Float32, Mono.
        const PCM_48000_F32_MONO            = 1u128 << 45;
        /// PCM, 48000 Hz, Float32, Stereo.
        const PCM_48000_F32_STEREO          = 1u128 << 46;
        /// PCM, 48000 Hz, Float32, 5.1.
        const PCM_48000_F32_5_1             = 1u128 << 47;
        /// PCM, 48000 Hz, Float32, 5.1.2.
        const PCM_48000_F32_5_1_2           = 1u128 << 48;

        /// DDPLUS, 48000 Hz, Stereo.
        const DDPLUS_48000_STEREO           = 1u128 << 49;
        /// DDPLUS, 48000 Hz, 5.1.
        const DDPLUS_48000_5_1              = 1u128 << 50;
        /// DDPLUS, 48000 Hz, 5.1.2.
        const DDPLUS_48000_5_1_2            = 1u128 << 51;
        /// DDPLUS, 48000 Hz, 7.1.4.
        const DDPLUS_48000_7_1_4            = 1u128 << 52;
        /// DDPLUS, 48000 Hz, 9.1.6.
        const DDPLUS_48000_9_1_6            = 1u128 << 53;

        /// QAAC, 48000 Hz, Stereo.
        const QAAC_48000_STEREO             = 1u128 << 54;
        /// QAAC, 48000 Hz, 5.1.
        const QAAC_48000_5_1                = 1u128 << 55;
        /// QAAC, 48000 Hz, 5.1.2.
        const QAAC_48000_5_1_2              = 1u128 << 56;

        /// QAACHE, 48000 Hz, Stereo.
        const QAACHE_48000_STEREO           = 1u128 << 57;
        /// QAACHE, 48000 Hz, 5.1.
        const QAACHE_48000_5_1              = 1u128 << 58;
        // 59: unknown
        /// QAACHE, 48000 Hz, 5.1.2.
        const QAACHE_48000_5_1_2            = 1u128 << 60;

        /// QLAC, 48000 Hz, 24-Bit, Stereo.
        const QLAC_48000_24_STEREO          = 1u128 << 61;

        /// QC3, 48000 Hz, Stereo.
        const QC3_48000_STEREO              = 1u128 << 62;
        /// QC3, 48000 Hz, 5.1.
        const QC3_48000_5_1                 = 1u128 << 63;

        /// APAC, 48000 Hz, Stereo.
        const APAC_48000_STEREO             = 1u128 << 64;
        /// APAC, 48000 Hz, 5.1.
        const APAC_48000_5_1                = 1u128 << 65;
        /// APAC, 48000 Hz, 5.1.2.
        const APAC_48000_5_1_2              = 1u128 << 66;
        /// APAC, 48000 Hz, 7.1.
        const APAC_48000_7_1                = 1u128 << 67;
        /// APAC, 48000 Hz, 7.1.4.
        const APAC_48000_7_1_4              = 1u128 << 68;

        /// PCM, 48000 Hz, Float32, 7.1.4.
        const PCM_48000_F32_7_1_4           = 1u128 << 69;

        /// QAAC, 44100 Hz, Stereo.
        const QAAC_44100_STEREO             = 1u128 << 70;

        /// QAACHE, 44100 Hz, Stereo.
        const QAACHE_44100_STEREO           = 1u128 << 71;

        /// QAACHEV2, 44100 Hz, Stereo.
        const QAACHEV2_44100_STEREO         = 1u128 << 72;

        /// QLAC, 44100 Hz, 24-Bit, Stereo.
        const QLAC_44100_24_STEREO          = 1u128 << 73;

        /// MP3, 44100 Hz, Stereo.
        const MP3_44100_STEREO              = 1u128 << 74;
        /// MP3, 48000 Hz, Stereo.
        const MP3_48000_STEREO              = 1u128 << 75;

        /// APAC, 48000 Hz, 5.1.4.
        const APAC_48000_5_1_4              = 1u128 << 76;
        /// APAC, 48000 Hz, 7.1.2.
        const APAC_48000_7_1_2              = 1u128 << 77;

        /// PCM, 48000 Hz, 16-Bit, 7.1.
        const PCM_48000_7_1                 = 1u128 << 78;
        /// PCM, 48000 Hz, Float32, 7.1.
        const PCM_48000_F32_7_1             = 1u128 << 79;
        /// PCM, 48000 Hz, 16-Bit, 5.1.4.
        const PCM_48000_5_1_4               = 1u128 << 80;
        /// PCM, 48000 Hz, Float32, 5.1.4.
        const PCM_48000_F32_5_1_4           = 1u128 << 81;
        /// PCM, 48000 Hz, 16-Bit, 7.1.2.
        const PCM_48000_7_1_2               = 1u128 << 82;
        /// PCM, 48000 Hz, Float32, 7.1.2.
        const PCM_48000_F32_7_1_2           = 1u128 << 83;
        /// PCM, 48000 Hz, 16-Bit, 7.1.4.
        const PCM_48000_7_1_4               = 1u128 << 84;

        /// ALAC, 44100 Hz, 20-Bit, Stereo.
        const ALAC_44100_20_STEREO          = 1u128 << 85;
        /// ALAC, 48000 Hz, 20-Bit, Stereo.
        const ALAC_48000_20_STEREO          = 1u128 << 86;

        /// QAC3, 48000 Hz, 5.1.
        const QAC3_48000_5_1                = 1u128 << 87;

        /// QEC3, 48000 Hz, 7.1.
        const QEC3_48000_7_1                = 1u128 << 88;

        /// PAAC, 44100 Hz, Stereo.
        const PAAC_44100_STEREO             = 1u128 << 89;

        /// AAC-LC, 48000 Hz, 7.1.
        const AAC_LC_48000_7_1              = 1u128 << 90;

        /// EAC3, 48000 Hz, 5.1.
        const EAC3_48000_5_1                = 1u128 << 91;

        /// APAC, 48000 Hz, 9.1.6.
        const APAC_48000_9_1_6              = 1u128 << 92;

        /// PCM, 48000 Hz, 16-Bit, 9.1.6.
        const PCM_48000_9_1_6               = 1u128 << 93;
        /// PCM, 48000 Hz, Float32, 9.1.6.
        const PCM_48000_F32_9_1_6           = 1u128 << 94;
    }
}

impl AudioFormat {
    pub const fn debug_label(&self) -> &'static str {
        let bits = self.bits();

        if bits == 0 {
            return "EMPTY";
        }
        if bits.count_ones() != 1 {
            return "MULTIPLE";
        }

        match bits.trailing_zeros() {
            2 => "PCM/8000/16/1",
            3 => "PCM/8000/16/2",
            4 => "PCM/16000/16/1",
            5 => "PCM/16000/16/2",
            6 => "PCM/24000/16/1",
            7 => "PCM/24000/16/2",
            8 => "PCM/32000/16/1",
            9 => "PCM/32000/16/2",
            10 => "PCM/44100/16/1",
            11 => "PCM/44100/16/2",
            12 => "PCM/44100/24/1",
            13 => "PCM/44100/24/2",
            14 => "PCM/48000/16/1",
            15 => "PCM/48000/16/2",
            16 => "PCM/48000/24/1",
            17 => "PCM/48000/24/2",
            18 => "ALAC/44100/16/2",
            19 => "ALAC/44100/24/2",
            20 => "ALAC/48000/16/2",
            21 => "ALAC/48000/24/2",
            22 => "AAC-LC/44100/2",
            23 => "AAC-LC/48000/2",
            24 => "AAC-ELD/44100/2",
            25 => "AAC-ELD/48000/2",
            26 => "AAC-ELD/16000/1",
            27 => "AAC-ELD/24000/1",
            28 => "OPUS/16000/1",
            29 => "OPUS/24000/1",
            30 => "OPUS/48000/1",
            31 => "AAC-ELD/44100/1",
            32 => "AAC-ELD/48000/1",
            33 => "QC3/48000/5.1.2",
            34 => "QC3/48000/7.1.4",
            35 => "QC3/48000/9.1.6",
            38 => "PCM/48000/16/5.1.2",
            39 => "AAC_LC/48000/5.1",
            40 => "AAC_LC/48000/5.1.2",
            41 => "AAC_ELD/48000/5.1",
            42 => "AAC_ELD/48000/5.1.2",
            43 => "AAC-ELD/32000/1",
            44 => "PCM/48000/16/5.1",
            45 => "PCM/48000/32f/1",
            46 => "PCM/48000/32f/2",
            47 => "PCM/48000/32f/5.1",
            48 => "PCM/48000/32f/5.1.2",
            49 => "DDPLUS/48000/2",
            50 => "DDPLUS/48000/5.1",
            51 => "DDPLUS/48000/5.1.2",
            52 => "DDPLUS/48000/7.1.4",
            53 => "DDPLUS/48000/9.1.6",
            54 => "QAAC/48000/2",
            55 => "QAAC/48000/5.1",
            56 => "QAAC/48000/5.1.2",
            57 => "QAACHE/48000/2",
            58 => "QAACHE/48000/5.1",
            60 => "QAACHE/48000/5.1.2",
            61 => "QLAC/48000/24/2",
            62 => "QC3/48000/2",
            63 => "QC3/48000/5.1",
            64 => "APAC/48000/2",
            65 => "APAC/48000/5.1",
            66 => "APAC/48000/5.1.2",
            67 => "APAC/48000/7.1",
            68 => "APAC/48000/7.1.4",
            69 => "PCM/48000/32f/7.1.4",
            70 => "QAAC/44100/2",
            71 => "QAACHE/44100/2",
            72 => "QAACHEV2/44100/2",
            73 => "QLAC/44100/24/2",
            74 => "MP3/44100/2",
            75 => "MP3/48000/2",
            76 => "APAC/48000/5.1.4",
            77 => "APAC/48000/7.1.2",
            78 => "PCM/48000/16/7.1",
            79 => "PCM/48000/32f/7.1",
            80 => "PCM/48000/16/5.1.4",
            81 => "PCM/48000/32f/5.1.4",
            82 => "PCM/48000/16/7.1.2",
            83 => "PCM/48000/32f/7.1.2",
            84 => "PCM/48000/16/7.1.4",
            85 => "ALAC/44100/20/2",
            86 => "ALAC/48000/20/2",
            87 => "QAC3/48000/5.1",
            88 => "QEC3/48000/7.1",
            89 => "PAAC/44100/2",
            90 => "AAC_LC/48000/7.1",
            91 => "EAC3/48000/5.1",
            92 => "APAC/48000/9.1.6",
            93 => "PCM/48000/16/9.1.6",
            94 => "PCM/48000/32f/9.1.6",
            _ => "UNKNOWN",
        }
    }
}
