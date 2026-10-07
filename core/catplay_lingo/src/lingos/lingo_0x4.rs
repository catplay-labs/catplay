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

// Lingo 0x04: Extended Interface
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
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0001,
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
            required pub acked_command_id: u16 => { bit: 1, key: "ackedCommandID", when: (truthy(&true)), predicate_value: false },
            /// Maximum Pending Wait
            // Virtual schema indices: 3
            optional pub maximum_pending_wait: u32 => { bit: 2, key: "maximumPendingWait", when: (eq(&command_result, &6u64)), predicate_value: false },
            /// Session ID
            // Virtual schema indices: 4
            optional pub session_id: u16 => { bit: 3, key: "sessionID", when: (eq(&command_result, &23u64)), predicate_value: false },
            /// Number of Bytes Dropped
            // Virtual schema indices: 5
            optional pub num_bytes_dropped: u32 => { bit: 4, key: "numBytesDropped", when: (eq(&command_result, &23u64)), predicate_value: false },
        }
        steps {
            1 => required command_result: IPodAckCommandResult => { bit: 0, key: "commandResult", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required acked_command_id: u16 => { bit: 1, key: "ackedCommandID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => optional maximum_pending_wait: u32 => { bit: 2, key: "maximumPendingWait", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&command_result, &6u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(6u8)] } },
            4 => optional session_id: u16 => { bit: 3, key: "sessionID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&command_result, &23u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(23u8)] } },
            5 => optional num_bytes_dropped: u32 => { bit: 4, key: "numBytesDropped", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&command_result, &23u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(23u8)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0002,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetCurrentPlayingTrackChapterInfo {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0003,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturnCurrentPlayingTrackChapterInfo {
        fields {
            /// Current Chapter Index
            // Virtual schema indices: 1
            required pub current_chapter_index: i32 => { bit: 0, key: "currentChapterIndex", when: (truthy(&true)), predicate_value: false },
            /// Chapter Count
            // Virtual schema indices: 2
            required pub chapter_count: i32 => { bit: 1, key: "chapterCount", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required current_chapter_index: i32 => { bit: 0, key: "currentChapterIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required chapter_count: i32 => { bit: 1, key: "chapterCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0004,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetCurrentPlayingTrackChapter {
        fields {
            /// Chapter Index
            // Virtual schema indices: 1
            required pub chapter_index: u32 => { bit: 0, key: "chapterIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required chapter_index: u32 => { bit: 0, key: "chapterIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0005,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetCurrentPlayingTrackChapterPlayStatus {
        fields {
            /// Current Chapter Index
            // Virtual schema indices: 1
            required pub current_chapter_index: u32 => { bit: 0, key: "currentChapterIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required current_chapter_index: u32 => { bit: 0, key: "currentChapterIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0006,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturnCurrentPlayingTrackChapterPlayStatus {
        fields {
            /// Chapter Length
            // Virtual schema indices: 1
            required pub chapter_length: u32 => { bit: 0, key: "chapterLength", when: (truthy(&true)), predicate_value: false },
            /// Elapsed Time in Chapter
            // Virtual schema indices: 2
            required pub elapsed_time_in_chapter: u32 => { bit: 1, key: "elapsedTimeInChapter", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required chapter_length: u32 => { bit: 0, key: "chapterLength", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required elapsed_time_in_chapter: u32 => { bit: 1, key: "elapsedTimeInChapter", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0007,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetCurrentPlayingTrackChapterName {
        fields {
            /// Chapter Index
            // Virtual schema indices: 1
            required pub chapter_index: u32 => { bit: 0, key: "chapterIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required chapter_index: u32 => { bit: 0, key: "chapterIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0008,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturnCurrentPlayingTrackChapterName {
        fields {
            /// Chapter Name
            // Virtual schema indices: 1
            required pub chapter_name: String => { bit: 0, key: "chapterName", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required chapter_name: String => { bit: 0, key: "chapterName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0009,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetAudiobookSpeed {
        fields {
        }
        steps {
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum ReturnAudiobookSpeedAudiobookSpeedStatusCode: u8 {
        /// normal
        Normal = 0,
        /// faster (+1)
        Faster1 = 1,
        /// slower (-1)
        Slower1 = 255,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x000a,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturnAudiobookSpeed {
        fields {
            /// Audiobook Speed Status Code
            // Virtual schema indices: 1
            required pub audiobook_speed_status_code: ReturnAudiobookSpeedAudiobookSpeedStatusCode => { bit: 0, key: "audiobookSpeedStatusCode", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required audiobook_speed_status_code: ReturnAudiobookSpeedAudiobookSpeedStatusCode => { bit: 0, key: "audiobookSpeedStatusCode", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetAudiobookSpeedNewAudiobookSpeedCode: u8 {
        /// normal
        Normal = 0,
        /// faster (+1)
        Faster1 = 1,
        /// slower (-1)
        Slower1 = 255,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x000b,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetAudiobookSpeed {
        fields {
            /// Audiobook Speed Code
            // Virtual schema indices: 1
            required pub new_audiobook_speed_code: SetAudiobookSpeedNewAudiobookSpeedCode => { bit: 0, key: "newAudiobookSpeedCode", when: (truthy(&true)), predicate_value: false },
            /// Restore on Exit
            // Virtual schema indices: 2
            length_optional pub restore_on_exit: bool => { bit: 1, key: "restoreOnExit", when: (ctx.encoding() || (eq(&ctx.adjusted_payload_length, &2u64))), predicate_value: false },
        }
        steps {
            1 => required new_audiobook_speed_code: SetAudiobookSpeedNewAudiobookSpeedCode => { bit: 0, key: "newAudiobookSpeedCode", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => length_optional restore_on_exit: bool => { bit: 1, key: "restoreOnExit", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (eq(&ctx.adjusted_payload_length, &2u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Eq, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(2u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum GetIndexedPlayingTrackInfoTrackInfoType: u8 {
        /// capabilities and information
        CapabilitiesAndInformation = 0,
        /// podcast name
        PodcastName = 1,
        /// release date
        ReleaseDate = 2,
        /// description
        Description = 3,
        /// song lyrics
        SongLyrics = 4,
        /// genre
        Genre = 5,
        /// composer
        Composer = 6,
        /// artwork count
        ArtworkCount = 7,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x000c,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetIndexedPlayingTrackInfo {
        fields {
            /// Track Info Type
            // Virtual schema indices: 1
            required pub track_info_type: GetIndexedPlayingTrackInfoTrackInfoType => { bit: 0, key: "trackInfoType", when: (truthy(&true)), predicate_value: false },
            /// Track Index
            // Virtual schema indices: 2
            required pub track_index: u32 => { bit: 1, key: "trackIndex", when: (truthy(&true)), predicate_value: false },
            /// Chapter Index
            // Virtual schema indices: 3
            required pub chapter_index: u16 => { bit: 2, key: "chapterIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required track_info_type: GetIndexedPlayingTrackInfoTrackInfoType => { bit: 0, key: "trackInfoType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required track_index: u32 => { bit: 1, key: "trackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required chapter_index: u16 => { bit: 2, key: "chapterIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum ReturnIndexedPlayingTrackInfoTrackInfoType: u8 {
        /// capabilities and information
        CapabilitiesAndInformation = 0,
        /// podcast name
        PodcastName = 1,
        /// release date
        ReleaseDate = 2,
        /// description
        Description = 3,
        /// song lyrics
        SongLyrics = 4,
        /// genre
        Genre = 5,
        /// composer
        Composer = 6,
        /// artwork count
        ArtworkCount = 7,
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct ReturnIndexedPlayingTrackInfoTrackCapabilityBits: u32 {
        /// is audiobook
        const IS_AUDIOBOOK = 1 << 0;
        /// has chapters
        const HAS_CHAPTERS = 1 << 1;
        /// has album artwork
        const HAS_ALBUM_ARTWORK = 1 << 2;
        /// has song lyrics
        const HAS_SONG_LYRICS = 1 << 3;
        /// is a podcast episode
        const IS_A_PODCAST_EPISODE = 1 << 4;
        /// has release date
        const HAS_RELEASE_DATE = 1 << 5;
        /// has description
        const HAS_DESCRIPTION = 1 << 6;
        /// contains video
        const CONTAINS_VIDEO = 1 << 7;
        /// queued to play as video
        const QUEUED_TO_PLAY_AS_VIDEO = 1 << 8;
        /// capable of generating a Genius playlist
        const CAPABLE_OF_GENERATING_A_GENIUS_PLAYLIST = 1 << 13;
        /// iTunes U episode
        const I_TUNES_U_EPISODE = 1 << 14;
    }
}

iap1_disjoint! {
    strings = LingoString;
    #[iap1_disjoint(predicate = opaque)]
    pub enum ReturnIndexedPlayingTrackInfoTrackInformationDisjoint {
        #[iap1_disjoint(decode = decode_track_information, encode = encode_track_information)]
        /// Track Information
        TrackInformation(String),
        #[iap1_disjoint(decode = decode_track_information2, encode = encode_track_information2)]
        /// Track Information
        TrackInformation2(Vec<u8>),
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum ReturnIndexedPlayingTrackInfoWeekday: u8 {
        /// Sunday
        Sunday = 0,
        /// Monday
        Monday = 1,
        /// Tuesday
        Tuesday = 2,
        /// Wednesday
        Wednesday = 3,
        /// Thursday
        Thursday = 4,
        /// Friday
        Friday = 5,
        /// Saturday
        Saturday = 6,
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct ReturnIndexedPlayingTrackInfoPacketInformationBits: u8 {
        /// multiple packets
        const MULTIPLE_PACKETS = 1 << 0;
        /// last of multiple packets
        const LAST_OF_MULTIPLE_PACKETS = 1 << 1;
    }
}

iap1_record! {
    strings = LingoString;
    pub struct ReturnIndexedPlayingTrackInfoAvailableImages {
        fields {
            /// Format ID
            // Virtual schema indices: 1
            required pub format_id: u16 => { bit: 0, key: "formatID", when: (truthy(&true)), predicate_value: false },
            /// Number of Images in Format
            // Virtual schema indices: 2
            required pub image_count: u16 => { bit: 1, key: "imageCount", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required format_id: u16 => { bit: 0, key: "formatID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required image_count: u16 => { bit: 1, key: "imageCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x000d,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturnIndexedPlayingTrackInfo {
        fields {
            /// Track Info Type
            // Virtual schema indices: 1
            required pub track_info_type: ReturnIndexedPlayingTrackInfoTrackInfoType => { bit: 0, key: "trackInfoType", when: (truthy(&true)), predicate_value: true },
            /// Track Capability Bits
            // Virtual schema indices: 2
            optional pub track_capability_bits: ReturnIndexedPlayingTrackInfoTrackCapabilityBits => { bit: 1, key: "trackCapabilityBits", when: (eq(&track_info_type, &0u64)), predicate_value: false },
            /// Total Track Length
            // Virtual schema indices: 3
            optional pub total_track_length: u32 => { bit: 2, key: "totalTrackLength", when: (eq(&track_info_type, &0u64)), predicate_value: false },
            /// Chapter Count
            // Virtual schema indices: 4
            optional pub chapter_count: u16 => { bit: 3, key: "chapterCount", when: (eq(&track_info_type, &0u64)), predicate_value: false },
            /// Track Information
            // Virtual schema indices: 5, 15, 16
            optional pub track_information: ReturnIndexedPlayingTrackInfoTrackInformationDisjoint => { bit: 4, key: "trackInformation", when: (((eq(&track_info_type, &1u64)) || (eq(&track_info_type, &5u64))) || (eq(&track_info_type, &6u64))) || (((eq(&track_info_type, &3u64)) || (eq(&track_info_type, &4u64))) && (ne(&packet_information_bits, &0u64))) || (((eq(&track_info_type, &3u64)) || (eq(&track_info_type, &4u64))) && (eq(&packet_information_bits, &0u64))), predicate_value: false },
            /// Seconds (0-59)
            // Virtual schema indices: 6
            optional pub seconds: u8 => { bit: 5, key: "seconds", when: (eq(&track_info_type, &2u64)), predicate_value: false },
            /// Minutes (0-59)
            // Virtual schema indices: 7
            optional pub minutes: u8 => { bit: 6, key: "minutes", when: (eq(&track_info_type, &2u64)), predicate_value: false },
            /// Hours (0-23)
            // Virtual schema indices: 8
            optional pub hours: u8 => { bit: 7, key: "hours", when: (eq(&track_info_type, &2u64)), predicate_value: false },
            /// Day of the Month (1-31)
            // Virtual schema indices: 9
            optional pub day_of_month: u8 => { bit: 8, key: "dayOfMonth", when: (eq(&track_info_type, &2u64)), predicate_value: false },
            /// Month (1-12)
            // Virtual schema indices: 10
            optional pub month: u8 => { bit: 9, key: "month", when: (eq(&track_info_type, &2u64)), predicate_value: false },
            /// Year
            // Virtual schema indices: 11
            optional pub year: u16 => { bit: 10, key: "year", when: (eq(&track_info_type, &2u64)), predicate_value: false },
            /// Weekday
            // Virtual schema indices: 12
            optional pub weekday: ReturnIndexedPlayingTrackInfoWeekday => { bit: 11, key: "weekday", when: (eq(&track_info_type, &2u64)), predicate_value: false },
            /// Packet Information Bits
            // Virtual schema indices: 13
            optional pub packet_information_bits: ReturnIndexedPlayingTrackInfoPacketInformationBits => { bit: 12, key: "packetInformationBits", when: ((eq(&track_info_type, &3u64)) || (eq(&track_info_type, &4u64))), predicate_value: true },
            /// Packet Index
            // Virtual schema indices: 14
            optional pub packet_index: u16 => { bit: 13, key: "packetIndex", when: ((eq(&track_info_type, &3u64)) || (eq(&track_info_type, &4u64))), predicate_value: false },
            /// Available Images
            // Virtual schema indices: 17
            optional pub available_images: Vec<ReturnIndexedPlayingTrackInfoAvailableImages> => { bit: 14, key: "availableImages", when: (eq(&track_info_type, &7u64)), predicate_value: false },
        }
        steps {
            1 => required track_info_type: ReturnIndexedPlayingTrackInfoTrackInfoType => { bit: 0, key: "trackInfoType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional track_capability_bits: ReturnIndexedPlayingTrackInfoTrackCapabilityBits => { bit: 1, key: "trackCapabilityBits", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_info_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional total_track_length: u32 => { bit: 2, key: "totalTrackLength", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_info_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            4 => optional chapter_count: u16 => { bit: 3, key: "chapterCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_info_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            5 => optional track_information: ReturnIndexedPlayingTrackInfoTrackInformationDisjoint => { bit: 4, key: "trackInformation", wire: [disjoint(decode_track_information, encode_track_information, String, [raw_string])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_track_information)) }, child: &Iap1WireSpec::Raw }, predicate_value: false, predicate: { expr: ((eq(&track_info_type, &1u64)) || (eq(&track_info_type, &5u64))) || (eq(&track_info_type, &6u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Or, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(6u8)] } },
            6 => optional seconds: u8 => { bit: 5, key: "seconds", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_info_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            7 => optional minutes: u8 => { bit: 6, key: "minutes", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_info_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            8 => optional hours: u8 => { bit: 7, key: "hours", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_info_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            9 => optional day_of_month: u8 => { bit: 8, key: "dayOfMonth", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_info_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            10 => optional month: u8 => { bit: 9, key: "month", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_info_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            11 => optional year: u16 => { bit: 10, key: "year", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_info_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            12 => optional weekday: ReturnIndexedPlayingTrackInfoWeekday => { bit: 11, key: "weekday", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_info_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            13 => optional packet_information_bits: ReturnIndexedPlayingTrackInfoPacketInformationBits => { bit: 12, key: "packetInformationBits", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: (eq(&track_info_type, &3u64)) || (eq(&track_info_type, &4u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
            14 => optional packet_index: u16 => { bit: 13, key: "packetIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&track_info_type, &3u64)) || (eq(&track_info_type, &4u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
            15 => optional track_information: ReturnIndexedPlayingTrackInfoTrackInformationDisjoint => { bit: 4, key: "trackInformation", wire: [disjoint(decode_track_information2, encode_track_information2, Vec<u8>, [raw_bytes])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_track_information2)) }, child: &Iap1WireSpec::Raw }, predicate_value: false, predicate: { expr: ((eq(&track_info_type, &3u64)) || (eq(&track_info_type, &4u64))) && (ne(&packet_information_bits, &0u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Or, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8), FlatPredicateToken::Ne, FlatPredicateToken::Field(12), FlatPredicateToken::Integer(0u8)] } },
            16 => optional track_information: ReturnIndexedPlayingTrackInfoTrackInformationDisjoint => { bit: 4, key: "trackInformation", wire: [disjoint(decode_track_information, encode_track_information, String, [raw_string])], project_wire: Iap1WireSpec::Disjoint { variant: const { LingoString::require_raw(stringify!(decode_track_information)) }, child: &Iap1WireSpec::Raw }, predicate_value: false, predicate: { expr: ((eq(&track_info_type, &3u64)) || (eq(&track_info_type, &4u64))) && (eq(&packet_information_bits, &0u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Or, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(12), FlatPredicateToken::Integer(0u8)] } },
            17 => optional available_images: Vec<ReturnIndexedPlayingTrackInfoAvailableImages> => { bit: 14, key: "availableImages", wire: [counted_remaining], project_wire: Iap1WireSpec::CountedRemaining, predicate_value: false, predicate: { expr: eq(&track_info_type, &7u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(7u8)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x000e,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetArtworkFormats {
        fields {
        }
        steps {
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetArtworkFormatsArtworkFormatsPixelFormat: u8 {
        /// monochrome, 2 bits per pixel
        Monochrome2BitsPerPixel = 1,
        /// RGB 565 color, little-endian, 16 bpp
        RGB565ColorLittleEndian16Bpp = 2,
        /// RGB 565 color, big-endian, 16 bpp
        RGB565ColorBigEndian16Bpp = 3,
    }
}

iap1_record! {
    strings = LingoString;
    pub struct RetArtworkFormatsArtworkFormats {
        fields {
            /// Format ID
            // Virtual schema indices: 1
            required pub format_id: u16 => { bit: 0, key: "formatID", when: (truthy(&true)), predicate_value: false },
            /// Pixel Format
            // Virtual schema indices: 2
            required pub pixel_format: RetArtworkFormatsArtworkFormatsPixelFormat => { bit: 1, key: "pixelFormat", when: (truthy(&true)), predicate_value: false },
            /// Width
            // Virtual schema indices: 3
            required pub image_width: u16 => { bit: 2, key: "imageWidth", when: (truthy(&true)), predicate_value: false },
            /// Height
            // Virtual schema indices: 4
            required pub image_height: u16 => { bit: 3, key: "imageHeight", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required format_id: u16 => { bit: 0, key: "formatID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required pixel_format: RetArtworkFormatsArtworkFormatsPixelFormat => { bit: 1, key: "pixelFormat", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required image_width: u16 => { bit: 2, key: "imageWidth", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            4 => required image_height: u16 => { bit: 3, key: "imageHeight", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x000f,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetArtworkFormats {
        fields {
            /// Artwork Formats
            // Virtual schema indices: 1
            required pub artwork_formats: Vec<RetArtworkFormatsArtworkFormats> => { bit: 0, key: "artworkFormats", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required artwork_formats: Vec<RetArtworkFormatsArtworkFormats> => { bit: 0, key: "artworkFormats", wire: [counted_remaining], project_wire: Iap1WireSpec::CountedRemaining, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0010,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetTrackArtworkData {
        fields {
            /// Track Index
            // Virtual schema indices: 1
            required pub track_index: u32 => { bit: 0, key: "trackIndex", when: (truthy(&true)), predicate_value: false },
            /// Format ID
            // Virtual schema indices: 2
            required pub format_id: u16 => { bit: 1, key: "formatID", when: (truthy(&true)), predicate_value: false },
            /// Time Offset from Track Start
            // Virtual schema indices: 3
            required pub time_offset_from_track_start: u32 => { bit: 2, key: "timeOffsetFromTrackStart", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required track_index: u32 => { bit: 0, key: "trackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required format_id: u16 => { bit: 1, key: "formatID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required time_offset_from_track_start: u32 => { bit: 2, key: "timeOffsetFromTrackStart", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetTrackArtworkDataPixelFormat: u8 {
        /// monochrome, 2 bits per pixel
        Monochrome2BitsPerPixel = 1,
        /// RGB 565 color, little-endian, 16 bpp
        RGB565ColorLittleEndian16Bpp = 2,
        /// RGB 565 color, big-endian, 16 bpp
        RGB565ColorBigEndian16Bpp = 3,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0011,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetTrackArtworkData {
        fields {
            /// Descriptor Packet Index
            // Virtual schema indices: 1
            required pub descriptor_packet_index: u16 => { bit: 0, key: "descriptorPacketIndex", when: (truthy(&true)), predicate_value: true },
            /// Display Pixel Format
            // Virtual schema indices: 2
            optional pub pixel_format: RetTrackArtworkDataPixelFormat => { bit: 1, key: "pixelFormat", when: (eq(&descriptor_packet_index, &0u64)), predicate_value: false },
            /// Image Width
            // Virtual schema indices: 3
            optional pub image_width: u16 => { bit: 2, key: "imageWidth", when: (eq(&descriptor_packet_index, &0u64)), predicate_value: false },
            /// Image Height
            // Virtual schema indices: 4
            optional pub image_height: u16 => { bit: 3, key: "imageHeight", when: (eq(&descriptor_packet_index, &0u64)), predicate_value: false },
            /// Inset Rectangle, Top-Left Point, X Value
            // Virtual schema indices: 5
            optional pub x_value_of_top_left_point_of_inset_rectangle: u16 => { bit: 4, key: "xValueOfTopLeftPointOfInsetRectangle", when: (eq(&descriptor_packet_index, &0u64)), predicate_value: false },
            /// Inset Rectangle, Top-Left Point, Y Value
            // Virtual schema indices: 6
            optional pub y_value_of_top_left_point_of_inset_rectangle: u16 => { bit: 5, key: "yValueOfTopLeftPointOfInsetRectangle", when: (eq(&descriptor_packet_index, &0u64)), predicate_value: false },
            /// Inset Rectangle, Bottom-Right Point, X Value
            // Virtual schema indices: 7
            optional pub x_value_of_bottom_right_point_of_inset_rectangle: u16 => { bit: 6, key: "xValueOfBottomRightPointOfInsetRectangle", when: (eq(&descriptor_packet_index, &0u64)), predicate_value: false },
            /// Inset Rectangle, Bottom-Right Point, Y Value
            // Virtual schema indices: 8
            optional pub y_value_of_bottom_right_point_of_inset_rectangle: u16 => { bit: 7, key: "yValueOfBottomRightPointOfInsetRectangle", when: (eq(&descriptor_packet_index, &0u64)), predicate_value: false },
            /// Row Size
            // Virtual schema indices: 9
            optional pub row_size: u32 => { bit: 8, key: "rowSize", when: (eq(&descriptor_packet_index, &0u64)), predicate_value: false },
            /// Image Pixel Data
            // Virtual schema indices: 10
            required pub image_pixel_data: Vec<u8> => { bit: 9, key: "imagePixelData", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required descriptor_packet_index: u16 => { bit: 0, key: "descriptorPacketIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional pixel_format: RetTrackArtworkDataPixelFormat => { bit: 1, key: "pixelFormat", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&descriptor_packet_index, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional image_width: u16 => { bit: 2, key: "imageWidth", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&descriptor_packet_index, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            4 => optional image_height: u16 => { bit: 3, key: "imageHeight", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&descriptor_packet_index, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            5 => optional x_value_of_top_left_point_of_inset_rectangle: u16 => { bit: 4, key: "xValueOfTopLeftPointOfInsetRectangle", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&descriptor_packet_index, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            6 => optional y_value_of_top_left_point_of_inset_rectangle: u16 => { bit: 5, key: "yValueOfTopLeftPointOfInsetRectangle", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&descriptor_packet_index, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            7 => optional x_value_of_bottom_right_point_of_inset_rectangle: u16 => { bit: 6, key: "xValueOfBottomRightPointOfInsetRectangle", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&descriptor_packet_index, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            8 => optional y_value_of_bottom_right_point_of_inset_rectangle: u16 => { bit: 7, key: "yValueOfBottomRightPointOfInsetRectangle", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&descriptor_packet_index, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            9 => optional row_size: u32 => { bit: 8, key: "rowSize", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&descriptor_packet_index, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            10 => required image_pixel_data: Vec<u8> => { bit: 9, key: "imagePixelData", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0012,
        source = accessory,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct RequestProtocolVersion {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0013,
        source = device,
        response = true,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct ReturnProtocolVersion {
        fields {
            /// Protocol Major Version Number
            // Virtual schema indices: 1
            required pub protocol_major_version_number: u8 => { bit: 0, key: "protocolMajorVersionNumber", when: (truthy(&true)), predicate_value: false },
            /// Protocol Minor Version Number
            // Virtual schema indices: 2
            required pub protocol_minor_version_number: u8 => { bit: 1, key: "protocolMinorVersionNumber", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required protocol_major_version_number: u8 => { bit: 0, key: "protocolMajorVersionNumber", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required protocol_minor_version_number: u8 => { bit: 1, key: "protocolMinorVersionNumber", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0014,
        source = accessory,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct RequestiPodName {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0015,
        source = device,
        response = true,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct ReturniPodName {
        fields {
            /// iPod name
            // Virtual schema indices: 1
            required pub i_pod_name: String => { bit: 0, key: "iPodName", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required i_pod_name: String => { bit: 0, key: "iPodName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0016,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ResetDBSelection {
        fields {
        }
        steps {
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SelectDBRecordDatabaseCategoryType: u8 {
        /// top-level
        TopLevel = 0,
        /// playlist
        Playlist = 1,
        /// artist
        Artist = 2,
        /// album
        Album = 3,
        /// genre
        Genre = 4,
        /// track
        Track = 5,
        /// composer
        Composer = 6,
        /// audiobook
        Audiobook = 7,
        /// podcast
        Podcast = 8,
        /// nested playlist
        NestedPlaylist = 9,
        /// Genius Mixes
        GeniusMixes = 10,
        /// iTunes U
        ITunesU = 11,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0017,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SelectDBRecord {
        fields {
            /// Database Category Type
            // Virtual schema indices: 1
            required pub database_category_type: SelectDBRecordDatabaseCategoryType => { bit: 0, key: "databaseCategoryType", when: (truthy(&true)), predicate_value: false },
            /// Database Record Index
            // Virtual schema indices: 2
            required pub database_record_index: i32 => { bit: 1, key: "databaseRecordIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required database_category_type: SelectDBRecordDatabaseCategoryType => { bit: 0, key: "databaseCategoryType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required database_record_index: i32 => { bit: 1, key: "databaseRecordIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum GetNumberCategorizedDBRecordsDatabaseCategoryType: u8 {
        /// top-level
        TopLevel = 0,
        /// playlist
        Playlist = 1,
        /// artist
        Artist = 2,
        /// album
        Album = 3,
        /// genre
        Genre = 4,
        /// track
        Track = 5,
        /// composer
        Composer = 6,
        /// audiobook
        Audiobook = 7,
        /// podcast
        Podcast = 8,
        /// nested playlist
        NestedPlaylist = 9,
        /// Genius Mixes
        GeniusMixes = 10,
        /// iTunes U
        ITunesU = 11,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0018,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetNumberCategorizedDBRecords {
        fields {
            /// Database Category Type
            // Virtual schema indices: 1
            required pub database_category_type: GetNumberCategorizedDBRecordsDatabaseCategoryType => { bit: 0, key: "databaseCategoryType", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required database_category_type: GetNumberCategorizedDBRecordsDatabaseCategoryType => { bit: 0, key: "databaseCategoryType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0019,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturnNumberCategorizedDBRecords {
        fields {
            /// Database Record Count
            // Virtual schema indices: 1
            required pub database_record_count: u32 => { bit: 0, key: "databaseRecordCount", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required database_record_count: u32 => { bit: 0, key: "databaseRecordCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetrieveCategorizedDatabaseRecordsDatabaseCategoryType: u8 {
        /// top-level
        TopLevel = 0,
        /// playlist
        Playlist = 1,
        /// artist
        Artist = 2,
        /// album
        Album = 3,
        /// genre
        Genre = 4,
        /// track
        Track = 5,
        /// composer
        Composer = 6,
        /// audiobook
        Audiobook = 7,
        /// podcast
        Podcast = 8,
        /// nested playlist
        NestedPlaylist = 9,
        /// Genius Mixes
        GeniusMixes = 10,
        /// iTunes U
        ITunesU = 11,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x001a,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetrieveCategorizedDatabaseRecords {
        fields {
            /// Database Category Type
            // Virtual schema indices: 1
            required pub database_category_type: RetrieveCategorizedDatabaseRecordsDatabaseCategoryType => { bit: 0, key: "databaseCategoryType", when: (truthy(&true)), predicate_value: false },
            /// Database Record Start Index
            // Virtual schema indices: 2
            required pub database_record_start_index: i32 => { bit: 1, key: "databaseRecordStartIndex", when: (truthy(&true)), predicate_value: false },
            /// Database Record Read Count
            // Virtual schema indices: 3
            required pub database_record_read_count: i32 => { bit: 2, key: "databaseRecordReadCount", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required database_category_type: RetrieveCategorizedDatabaseRecordsDatabaseCategoryType => { bit: 0, key: "databaseCategoryType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required database_record_start_index: i32 => { bit: 1, key: "databaseRecordStartIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required database_record_read_count: i32 => { bit: 2, key: "databaseRecordReadCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x001b,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturnCategorizedDatabaseRecord {
        fields {
            /// Database Record Category Index
            // Virtual schema indices: 1
            required pub database_record_category_index: u32 => { bit: 0, key: "databaseRecordCategoryIndex", when: (truthy(&true)), predicate_value: false },
            /// Database Record
            // Virtual schema indices: 2
            required pub database_record: String => { bit: 1, key: "databaseRecord", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required database_record_category_index: u32 => { bit: 0, key: "databaseRecordCategoryIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required database_record: String => { bit: 1, key: "databaseRecord", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x001c,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetPlayStatus {
        fields {
        }
        steps {
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum ReturnPlayStatusPlayerState: u8 {
        /// stopped
        Stopped = 0,
        /// playing
        Playing = 1,
        /// paused
        Paused = 2,
        /// error
        Error = 255,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x001d,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturnPlayStatus {
        fields {
            /// Track Length
            // Virtual schema indices: 1
            required pub track_length: u32 => { bit: 0, key: "trackLength", when: (truthy(&true)), predicate_value: false },
            /// Track Position
            // Virtual schema indices: 2
            required pub track_position: u32 => { bit: 1, key: "trackPosition", when: (truthy(&true)), predicate_value: false },
            /// Player State
            // Virtual schema indices: 3
            required pub player_state: ReturnPlayStatusPlayerState => { bit: 2, key: "playerState", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required track_length: u32 => { bit: 0, key: "trackLength", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required track_position: u32 => { bit: 1, key: "trackPosition", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required player_state: ReturnPlayStatusPlayerState => { bit: 2, key: "playerState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x001e,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetCurrentPlayingTrackIndex {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x001f,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturnCurrentPlayingTrackIndex {
        fields {
            /// Playback Track Index
            // Virtual schema indices: 1
            required pub playback_track_index: i32 => { bit: 0, key: "playbackTrackIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required playback_track_index: i32 => { bit: 0, key: "playbackTrackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0020,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetIndexedPlayingTrackTitle {
        fields {
            /// Playback Track Index
            // Virtual schema indices: 1
            required pub playback_track_index: u32 => { bit: 0, key: "playbackTrackIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required playback_track_index: u32 => { bit: 0, key: "playbackTrackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0021,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturnIndexedPlayingTrackTitle {
        fields {
            /// Track Title
            // Virtual schema indices: 1
            required pub track_title: String => { bit: 0, key: "trackTitle", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required track_title: String => { bit: 0, key: "trackTitle", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0022,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetIndexedPlayingTrackArtistName {
        fields {
            /// Playback Track Index
            // Virtual schema indices: 1
            required pub playback_track_index: u32 => { bit: 0, key: "playbackTrackIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required playback_track_index: u32 => { bit: 0, key: "playbackTrackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0023,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturnIndexedPlayingTrackArtistName {
        fields {
            /// Artist Name
            // Virtual schema indices: 1
            required pub artist_name: String => { bit: 0, key: "artistName", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required artist_name: String => { bit: 0, key: "artistName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0024,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetIndexedPlayingTrackAlbumName {
        fields {
            /// Playback Track Index
            // Virtual schema indices: 1
            required pub playback_track_index: u32 => { bit: 0, key: "playbackTrackIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required playback_track_index: u32 => { bit: 0, key: "playbackTrackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0025,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturnIndexedPlayingTrackAlbumName {
        fields {
            /// Album Name
            // Virtual schema indices: 1
            required pub album_name: String => { bit: 0, key: "albumName", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required album_name: String => { bit: 0, key: "albumName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetPlayStatusChangeNotificationStatusEventChangeValue: u8 {
        /// disable all status event notifications
        DisableAllStatusEventNotifications = 0,
        /// enable play status notifications
        EnablePlayStatusNotifications = 1,
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct SetPlayStatusChangeNotificationStatusEventChangeMaskBits: u32 {
        /// basic play state changes
        const BASIC_PLAY_STATE_CHANGES = 1 << 0;
        /// extended play state changes
        const EXTENDED_PLAY_STATE_CHANGES = 1 << 1;
        /// track index
        const TRACK_INDEX = 1 << 2;
        /// track time offset (ms)
        const TRACK_TIME_OFFSET_MS = 1 << 3;
        /// track time offset (sec)
        const TRACK_TIME_OFFSET_SEC = 1 << 4;
        /// chapter index
        const CHAPTER_INDEX = 1 << 5;
        /// chapter time offset (ms)
        const CHAPTER_TIME_OFFSET_MS = 1 << 6;
        /// chapter time offset (sec)
        const CHAPTER_TIME_OFFSET_SEC = 1 << 7;
        /// track unique identifier
        const TRACK_UNIQUE_IDENTIFIER = 1 << 8;
        /// track media type (audio/video)
        const TRACK_MEDIA_TYPE_AUDIO_VIDEO = 1 << 9;
        /// track lyrics ready (if the track has lyrics)
        const TRACK_LYRICS_READY_IF_THE_TRACK_HAS_LYRICS = 1 << 10;
        /// track capabilities changed
        const TRACK_CAPABILITIES_CHANGED = 1 << 11;
        /// playback engine contents changed
        const PLAYBACK_ENGINE_CONTENTS_CHANGED = 1 << 12;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0026,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetPlayStatusChangeNotification {
        fields {
            /// Status Event Change Value
            // Virtual schema indices: 1
            length_optional pub status_event_change_value: SetPlayStatusChangeNotificationStatusEventChangeValue => { bit: 0, key: "statusEventChangeValue", when: (ctx.encoding() || (eq(&ctx.adjusted_payload_length, &1u64))), predicate_value: false },
            /// Status Event Change Mask Bits
            // Virtual schema indices: 2
            length_optional pub status_event_change_mask_bits: SetPlayStatusChangeNotificationStatusEventChangeMaskBits => { bit: 1, key: "statusEventChangeMaskBits", when: (ctx.encoding() || (eq(&ctx.adjusted_payload_length, &4u64))), predicate_value: false },
        }
        steps {
            1 => length_optional status_event_change_value: SetPlayStatusChangeNotificationStatusEventChangeValue => { bit: 0, key: "statusEventChangeValue", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (eq(&ctx.adjusted_payload_length, &1u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Eq, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(1u8)] } },
            2 => length_optional status_event_change_mask_bits: SetPlayStatusChangeNotificationStatusEventChangeMaskBits => { bit: 1, key: "statusEventChangeMaskBits", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (eq(&ctx.adjusted_payload_length, &4u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Eq, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(4u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum PlayStatusChangeNotificationPlayStatusType: u8 {
        /// playback stopped
        PlaybackStopped = 0,
        /// track index
        TrackIndex = 1,
        /// playback FFW seek stop
        PlaybackFFWSeekStop = 2,
        /// playback REW seek stop
        PlaybackREWSeekStop = 3,
        /// track time offset (ms)
        TrackTimeOffsetMs = 4,
        /// chapter index
        ChapterIndex = 5,
        /// playback status extended
        PlaybackStatusExtended = 6,
        /// track time offset (sec)
        TrackTimeOffsetSec = 7,
        /// chapter time offset (ms)
        ChapterTimeOffsetMs = 8,
        /// chapter time offset (sec)
        ChapterTimeOffsetSec = 9,
        /// track unique identifier
        TrackUniqueIdentifier = 10,
        /// track playback mode
        TrackPlaybackMode = 11,
        /// track lyrics ready
        TrackLyricsReady = 12,
        /// track capabilties changed
        TrackCapabiltiesChanged = 13,
        /// playback engine contents changed
        PlaybackEngineContentsChanged = 14,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum PlayStatusChangeNotificationPlayState: u8 {
        /// stopped
        Stopped = 2,
        /// FFW seek started
        FFWSeekStarted = 5,
        /// REW seek started
        REWSeekStarted = 6,
        /// FFW/REW seek stopped
        FFWREWSeekStopped = 7,
        /// playing
        Playing = 10,
        /// paused
        Paused = 11,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum PlayStatusChangeNotificationPlayMode: u8 {
        /// audio track
        AudioTrack = 0,
        /// video track
        VideoTrack = 1,
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct PlayStatusChangeNotificationTrackCaps: u32 {
        /// audiobook
        const AUDIOBOOK = 1 << 0;
        /// has chapters
        const HAS_CHAPTERS = 1 << 1;
        /// album artwork
        const ALBUM_ARTWORK = 1 << 2;
        /// lyrics
        const LYRICS = 1 << 3;
        /// podcast
        const PODCAST = 1 << 4;
        /// release date
        const RELEASE_DATE = 1 << 5;
        /// description
        const DESCRIPTION = 1 << 6;
        /// contains video
        const CONTAINS_VIDEO = 1 << 7;
        /// queued to play as video
        const QUEUED_TO_PLAY_AS_VIDEO = 1 << 8;
        /// capable of generating a Genius playlist
        const CAPABLE_OF_GENERATING_A_GENIUS_PLAYLIST = 1 << 13;
        /// iTunes U episode
        const I_TUNES_U_EPISODE = 1 << 14;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0027,
        source = device,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct PlayStatusChangeNotification {
        fields {
            /// New Play Status
            // Virtual schema indices: 1
            required pub play_status_type: PlayStatusChangeNotificationPlayStatusType => { bit: 0, key: "playStatusType", when: (truthy(&true)), predicate_value: true },
            /// Track Index
            // Virtual schema indices: 2
            optional pub track_index: u32 => { bit: 1, key: "trackIndex", when: (eq(&play_status_type, &1u64)), predicate_value: false },
            /// Track Time Offset
            // Virtual schema indices: 3
            optional pub track_offset_ms: u32 => { bit: 2, key: "trackOffsetMs", when: (eq(&play_status_type, &4u64)), predicate_value: false },
            /// Chapter Index
            // Virtual schema indices: 4
            optional pub chap_index: u32 => { bit: 3, key: "chapIndex", when: (eq(&play_status_type, &5u64)), predicate_value: false },
            /// Playback State
            // Virtual schema indices: 5
            optional pub play_state: PlayStatusChangeNotificationPlayState => { bit: 4, key: "playState", when: (eq(&play_status_type, &6u64)), predicate_value: false },
            /// Track Time Offset
            // Virtual schema indices: 6
            optional pub track_offset_sec: u32 => { bit: 5, key: "trackOffsetSec", when: (eq(&play_status_type, &7u64)), predicate_value: false },
            /// Chapter Time Offset
            // Virtual schema indices: 7
            optional pub chap_time_ms: u32 => { bit: 6, key: "chapTimeMs", when: (eq(&play_status_type, &8u64)), predicate_value: false },
            /// Chapter Time Offset
            // Virtual schema indices: 8
            optional pub chap_time_sec: u32 => { bit: 7, key: "chapTimeSec", when: (eq(&play_status_type, &9u64)), predicate_value: false },
            /// Track Unique Identifier
            // Virtual schema indices: 9
            optional pub track_uid: u64 => { bit: 8, key: "trackUID", when: (eq(&play_status_type, &10u64)), predicate_value: false },
            /// Track Playback Mode
            // Virtual schema indices: 10
            optional pub play_mode: PlayStatusChangeNotificationPlayMode => { bit: 9, key: "playMode", when: (eq(&play_status_type, &11u64)), predicate_value: false },
            /// Track Capabilities Bits
            // Virtual schema indices: 11
            optional pub track_caps: PlayStatusChangeNotificationTrackCaps => { bit: 10, key: "trackCaps", when: (eq(&play_status_type, &13u64)), predicate_value: false },
            /// Number of Tracks in New Playlist
            // Virtual schema indices: 12
            optional pub number_of_tracks_in_new_playlist: u32 => { bit: 11, key: "numberOfTracksInNewPlaylist", when: (eq(&play_status_type, &14u64)), predicate_value: false },
        }
        steps {
            1 => required play_status_type: PlayStatusChangeNotificationPlayStatusType => { bit: 0, key: "playStatusType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional track_index: u32 => { bit: 1, key: "trackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&play_status_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            3 => optional track_offset_ms: u32 => { bit: 2, key: "trackOffsetMs", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&play_status_type, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
            4 => optional chap_index: u32 => { bit: 3, key: "chapIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&play_status_type, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
            5 => optional play_state: PlayStatusChangeNotificationPlayState => { bit: 4, key: "playState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&play_status_type, &6u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(6u8)] } },
            6 => optional track_offset_sec: u32 => { bit: 5, key: "trackOffsetSec", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&play_status_type, &7u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(7u8)] } },
            7 => optional chap_time_ms: u32 => { bit: 6, key: "chapTimeMs", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&play_status_type, &8u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(8u8)] } },
            8 => optional chap_time_sec: u32 => { bit: 7, key: "chapTimeSec", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&play_status_type, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(9u8)] } },
            9 => optional track_uid: u64 => { bit: 8, key: "trackUID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&play_status_type, &10u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(10u8)] } },
            10 => optional play_mode: PlayStatusChangeNotificationPlayMode => { bit: 9, key: "playMode", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&play_status_type, &11u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(11u8)] } },
            11 => optional track_caps: PlayStatusChangeNotificationTrackCaps => { bit: 10, key: "trackCaps", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&play_status_type, &13u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(13u8)] } },
            12 => optional number_of_tracks_in_new_playlist: u32 => { bit: 11, key: "numberOfTracksInNewPlaylist", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&play_status_type, &14u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(14u8)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0028,
        source = accessory,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct PlayCurrentSelection {
        fields {
            /// Selection Track Record Index
            // Virtual schema indices: 1
            required pub selection_track_record_index: i32 => { bit: 0, key: "selectionTrackRecordIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required selection_track_record_index: i32 => { bit: 0, key: "selectionTrackRecordIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum PlayControlPlayControlCommandCode: u8 {
        /// toggle play/pause
        TogglePlayPause = 1,
        /// stop
        Stop = 2,
        /// next track
        #[deprecated]
        NextTrack = 3,
        /// previous track
        #[deprecated]
        PreviousTrack = 4,
        /// start FF
        StartFF = 5,
        /// start rew
        StartRew = 6,
        /// end FF/rew
        EndFFRew = 7,
        /// next
        Next = 8,
        /// previous
        Previous = 9,
        /// play
        Play = 10,
        /// pause
        Pause = 11,
        /// next chapter
        NextChapter = 12,
        /// previous chapter
        PreviousChapter = 13,
        /// resume playing built-in Music app
        ResumePlayingBuiltInMusicApp = 14,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0029,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct PlayControl {
        fields {
            /// Play Control Command Code
            // Virtual schema indices: 1
            required pub play_control_command_code: PlayControlPlayControlCommandCode => { bit: 0, key: "playControlCommandCode", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required play_control_command_code: PlayControlPlayControlCommandCode => { bit: 0, key: "playControlCommandCode", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x002a,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetTrackArtworkTimes {
        fields {
            /// Track Index
            // Virtual schema indices: 1
            required pub track_index: u32 => { bit: 0, key: "trackIndex", when: (truthy(&true)), predicate_value: false },
            /// Format ID
            // Virtual schema indices: 2
            required pub format_id: u16 => { bit: 1, key: "formatID", when: (truthy(&true)), predicate_value: false },
            /// Artwork Index
            // Virtual schema indices: 3
            required pub artwork_index: u16 => { bit: 2, key: "artworkIndex", when: (truthy(&true)), predicate_value: false },
            /// Artwork Count
            // Virtual schema indices: 4
            required pub artwork_count: u16 => { bit: 3, key: "artworkCount", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required track_index: u32 => { bit: 0, key: "trackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required format_id: u16 => { bit: 1, key: "formatID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required artwork_index: u16 => { bit: 2, key: "artworkIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            4 => required artwork_count: u16 => { bit: 3, key: "artworkCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_record! {
    strings = LingoString;
    pub struct RetTrackArtworkTimesTrackArtworkTimes {
        fields {
            /// Time Offset from Track Start
            // Virtual schema indices: 1
            required pub time_offset_from_track_start: u32 => { bit: 0, key: "timeOffsetFromTrackStart", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required time_offset_from_track_start: u32 => { bit: 0, key: "timeOffsetFromTrackStart", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x002b,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetTrackArtworkTimes {
        fields {
            /// Track Artwork Times
            // Virtual schema indices: 1
            required pub track_artwork_times: Vec<RetTrackArtworkTimesTrackArtworkTimes> => { bit: 0, key: "trackArtworkTimes", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required track_artwork_times: Vec<RetTrackArtworkTimesTrackArtworkTimes> => { bit: 0, key: "trackArtworkTimes", wire: [counted_remaining], project_wire: Iap1WireSpec::CountedRemaining, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x002c,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetShuffle {
        fields {
        }
        steps {
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum ReturnShuffleShuffleMode: u8 {
        /// off
        Off = 0,
        /// tracks
        Tracks = 1,
        /// albums
        Albums = 2,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x002d,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturnShuffle {
        fields {
            /// Shuffle Mode
            // Virtual schema indices: 1
            required pub shuffle_mode: ReturnShuffleShuffleMode => { bit: 0, key: "shuffleMode", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required shuffle_mode: ReturnShuffleShuffleMode => { bit: 0, key: "shuffleMode", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetShuffleShuffleMode: u8 {
        /// off
        Off = 0,
        /// tracks
        Tracks = 1,
        /// albums
        Albums = 2,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x002e,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetShuffle {
        fields {
            /// Shuffle Mode
            // Virtual schema indices: 1
            required pub shuffle_mode: SetShuffleShuffleMode => { bit: 0, key: "shuffleMode", when: (truthy(&true)), predicate_value: false },
            /// Restore on Exit
            // Virtual schema indices: 2
            length_optional pub restore_on_exit: bool => { bit: 1, key: "restoreOnExit", when: (ctx.encoding() || (eq(&ctx.adjusted_payload_length, &2u64))), predicate_value: false },
        }
        steps {
            1 => required shuffle_mode: SetShuffleShuffleMode => { bit: 0, key: "shuffleMode", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => length_optional restore_on_exit: bool => { bit: 1, key: "restoreOnExit", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (eq(&ctx.adjusted_payload_length, &2u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Eq, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(2u8)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x002f,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetRepeat {
        fields {
        }
        steps {
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum ReturnRepeatRepeatState: u8 {
        /// off
        Off = 0,
        /// one track
        OneTrack = 1,
        /// all tracks
        AllTracks = 2,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0030,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturnRepeat {
        fields {
            /// Repeat State
            // Virtual schema indices: 1
            required pub repeat_state: ReturnRepeatRepeatState => { bit: 0, key: "repeatState", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required repeat_state: ReturnRepeatRepeatState => { bit: 0, key: "repeatState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetRepeatRepeatState: u8 {
        /// off
        Off = 0,
        /// one track
        OneTrack = 1,
        /// all tracks
        AllTracks = 2,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0031,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetRepeat {
        fields {
            /// Repeat State
            // Virtual schema indices: 1
            required pub repeat_state: SetRepeatRepeatState => { bit: 0, key: "repeatState", when: (truthy(&true)), predicate_value: false },
            /// Restore on Exit
            // Virtual schema indices: 2
            length_optional pub restore_on_exit: bool => { bit: 1, key: "restoreOnExit", when: (ctx.encoding() || (eq(&ctx.adjusted_payload_length, &2u64))), predicate_value: false },
        }
        steps {
            1 => required repeat_state: SetRepeatRepeatState => { bit: 0, key: "repeatState", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => length_optional restore_on_exit: bool => { bit: 1, key: "restoreOnExit", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: ctx.encoding() || (eq(&ctx.adjusted_payload_length, &2u64)), program: [FlatPredicateToken::Or, FlatPredicateToken::Encoding, FlatPredicateToken::Eq, FlatPredicateToken::AdjustedPayloadLength, FlatPredicateToken::Integer(2u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SetDisplayImageDisplayPixelFormatCode: u8 {
        /// monochrome, 2 bits per pixel
        Monochrome2BitsPerPixel = 1,
        /// RGB 565 color, little-endian, 16 bpp
        RGB565ColorLittleEndian16Bpp = 2,
        /// RGB 565 color, big-endian, 16 bpp
        RGB565ColorBigEndian16Bpp = 3,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0032,
        source = accessory,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct SetDisplayImage {
        fields {
            /// Descriptor Packet Index
            // Virtual schema indices: 1
            required pub descriptor_packet_index: u16 => { bit: 0, key: "descriptorPacketIndex", when: (truthy(&true)), predicate_value: true },
            /// Display Pixel Format Code
            // Virtual schema indices: 2
            optional pub display_pixel_format_code: SetDisplayImageDisplayPixelFormatCode => { bit: 1, key: "displayPixelFormatCode", when: (eq(&descriptor_packet_index, &0u64)), predicate_value: false },
            /// Image Width
            // Virtual schema indices: 3
            optional pub image_width: u16 => { bit: 2, key: "imageWidth", when: (eq(&descriptor_packet_index, &0u64)), predicate_value: false },
            /// Image Height
            // Virtual schema indices: 4
            optional pub image_height: u16 => { bit: 3, key: "imageHeight", when: (eq(&descriptor_packet_index, &0u64)), predicate_value: false },
            /// Row Size (stride)
            // Virtual schema indices: 5
            optional pub row_stride: u32 => { bit: 4, key: "rowStride", when: (eq(&descriptor_packet_index, &0u64)), predicate_value: false },
            /// Display Image Pixel Data
            // Virtual schema indices: 6
            required pub display_image_pixel_data: Vec<u8> => { bit: 5, key: "displayImagePixelData", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required descriptor_packet_index: u16 => { bit: 0, key: "descriptorPacketIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional display_pixel_format_code: SetDisplayImageDisplayPixelFormatCode => { bit: 1, key: "displayPixelFormatCode", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&descriptor_packet_index, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional image_width: u16 => { bit: 2, key: "imageWidth", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&descriptor_packet_index, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            4 => optional image_height: u16 => { bit: 3, key: "imageHeight", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&descriptor_packet_index, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            5 => optional row_stride: u32 => { bit: 4, key: "rowStride", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&descriptor_packet_index, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            6 => required display_image_pixel_data: Vec<u8> => { bit: 5, key: "displayImagePixelData", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0033,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetMonoDisplayImageLimits {
        fields {
        }
        steps {
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum ReturnMonoDisplayImageLimitsDisplayPixelFormatCode: u8 {
        /// monochrome, 2 bits per pixel
        Monochrome2BitsPerPixel = 1,
        /// RGB 565 color, little-endian, 16 bpp
        RGB565ColorLittleEndian16Bpp = 2,
        /// RGB 565 color, big-endian, 16 bpp
        RGB565ColorBigEndian16Bpp = 3,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0034,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturnMonoDisplayImageLimits {
        fields {
            /// Maximum Image Width
            // Virtual schema indices: 1
            required pub max_image_width: u16 => { bit: 0, key: "maxImageWidth", when: (truthy(&true)), predicate_value: false },
            /// Maximum Image Height
            // Virtual schema indices: 2
            required pub max_image_height: u16 => { bit: 1, key: "maxImageHeight", when: (truthy(&true)), predicate_value: false },
            /// Display Pixel Format Code
            // Virtual schema indices: 3
            required pub display_pixel_format_code: ReturnMonoDisplayImageLimitsDisplayPixelFormatCode => { bit: 2, key: "displayPixelFormatCode", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required max_image_width: u16 => { bit: 0, key: "maxImageWidth", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required max_image_height: u16 => { bit: 1, key: "maxImageHeight", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required display_pixel_format_code: ReturnMonoDisplayImageLimitsDisplayPixelFormatCode => { bit: 2, key: "displayPixelFormatCode", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0035,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetNumPlayingTracks {
        fields {
        }
        steps {
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0036,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturnNumPlayingTracks {
        fields {
            /// Number of Tracks Playing
            // Virtual schema indices: 1
            required pub number_of_tracks_playing: u32 => { bit: 0, key: "numberOfTracksPlaying", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required number_of_tracks_playing: u32 => { bit: 0, key: "numberOfTracksPlaying", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0037,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct SetCurrentPlayingTrack {
        fields {
            /// New Current Playing Track Index
            // Virtual schema indices: 1
            required pub new_current_playing_track_index: u32 => { bit: 0, key: "newCurrentPlayingTrackIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required new_current_playing_track_index: u32 => { bit: 0, key: "newCurrentPlayingTrackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SelectSortDBRecordDatabaseCategoryType: u8 {
        /// top-level
        TopLevel = 0,
        /// playlist
        Playlist = 1,
        /// artist
        Artist = 2,
        /// album
        Album = 3,
        /// genre
        Genre = 4,
        /// track
        Track = 5,
        /// composer
        Composer = 6,
        /// audiobook
        Audiobook = 7,
        /// podcast
        Podcast = 8,
        /// nested playlist
        NestedPlaylist = 9,
        /// Genius Mixes
        GeniusMixes = 10,
        /// iTunes U
        ITunesU = 11,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum SelectSortDBRecordDatabaseSortType: u8 {
        /// genre
        Genre = 0,
        /// artist
        Artist = 1,
        /// composer
        Composer = 2,
        /// album
        Album = 3,
        /// name
        Name = 4,
        /// release date
        ReleaseDate = 6,
        /// series (video only)
        SeriesVideoOnly = 7,
        /// season (video only)
        SeasonVideoOnly = 8,
        /// episode (video only)
        EpisodeVideoOnly = 9,
        /// default
        Default = 255,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0038,
        source = accessory,
        response = false,
        ack = false,
        deprecated = true,
        transaction_id = permitted
    )]
    pub struct SelectSortDBRecord {
        fields {
            /// Database Category Type
            // Virtual schema indices: 1
            required pub database_category_type: SelectSortDBRecordDatabaseCategoryType => { bit: 0, key: "databaseCategoryType", when: (truthy(&true)), predicate_value: false },
            /// Category Record Index
            // Virtual schema indices: 2
            required pub category_record_index: u32 => { bit: 1, key: "categoryRecordIndex", when: (truthy(&true)), predicate_value: false },
            /// Database Sort Type
            // Virtual schema indices: 3
            required pub database_sort_type: SelectSortDBRecordDatabaseSortType => { bit: 2, key: "databaseSortType", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required database_category_type: SelectSortDBRecordDatabaseCategoryType => { bit: 0, key: "databaseCategoryType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required category_record_index: u32 => { bit: 1, key: "categoryRecordIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required database_sort_type: SelectSortDBRecordDatabaseSortType => { bit: 2, key: "databaseSortType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0039,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetColorDisplayImageLimits {
        fields {
        }
        steps {
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum ReturnColorDisplayImageLimitsColorDisplayImageLimitsDisplayPixelFormatCode: u8 {
        /// monochrome, 2 bits per pixel
        Monochrome2BitsPerPixel = 1,
        /// RGB 565 color, little-endian, 16 bpp
        RGB565ColorLittleEndian16Bpp = 2,
        /// RGB 565 color, big-endian, 16 bpp
        RGB565ColorBigEndian16Bpp = 3,
    }
}

iap1_record! {
    strings = LingoString;
    pub struct ReturnColorDisplayImageLimitsColorDisplayImageLimits {
        fields {
            /// Maximum Image Width
            // Virtual schema indices: 1
            required pub max_image_width: u16 => { bit: 0, key: "maxImageWidth", when: (truthy(&true)), predicate_value: false },
            /// Maximum Image Height
            // Virtual schema indices: 2
            required pub max_image_height: u16 => { bit: 1, key: "maxImageHeight", when: (truthy(&true)), predicate_value: false },
            /// Display Pixel Format Code
            // Virtual schema indices: 3
            required pub display_pixel_format_code: ReturnColorDisplayImageLimitsColorDisplayImageLimitsDisplayPixelFormatCode => { bit: 2, key: "displayPixelFormatCode", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required max_image_width: u16 => { bit: 0, key: "maxImageWidth", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required max_image_height: u16 => { bit: 1, key: "maxImageHeight", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required display_pixel_format_code: ReturnColorDisplayImageLimitsColorDisplayImageLimitsDisplayPixelFormatCode => { bit: 2, key: "displayPixelFormatCode", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x003a,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ReturnColorDisplayImageLimits {
        fields {
            /// Color Display Image Limits
            // Virtual schema indices: 1
            required pub color_display_image_limits: Vec<ReturnColorDisplayImageLimitsColorDisplayImageLimits> => { bit: 0, key: "colorDisplayImageLimits", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required color_display_image_limits: Vec<ReturnColorDisplayImageLimitsColorDisplayImageLimits> => { bit: 0, key: "colorDisplayImageLimits", wire: [counted_remaining], project_wire: Iap1WireSpec::CountedRemaining, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum ResetDBSelectionHierarchyHierarchySelection: u8 {
        /// audio
        Audio = 1,
        /// video
        Video = 2,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x003b,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct ResetDBSelectionHierarchy {
        fields {
            /// Hierarchy Selection
            // Virtual schema indices: 1
            required pub hierarchy_selection: ResetDBSelectionHierarchyHierarchySelection => { bit: 0, key: "hierarchySelection", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required hierarchy_selection: ResetDBSelectionHierarchyHierarchySelection => { bit: 0, key: "hierarchySelection", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum GetDBiTunesInfoITunesDBMetadataType: u8 {
        /// database UID
        DatabaseUID = 0,
        /// last iTunes sync date/time
        LastITunesSyncDateTime = 1,
        /// total audio track count
        TotalAudioTrackCount = 2,
        /// total video track count
        TotalVideoTrackCount = 3,
        /// total audiobook count
        TotalAudiobookCount = 4,
        /// total photo count
        TotalPhotoCount = 5,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x003c,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetDBiTunesInfo {
        fields {
            /// iTunes Database Metadata Type
            // Virtual schema indices: 1
            required pub i_tunes_db_metadata_type: GetDBiTunesInfoITunesDBMetadataType => { bit: 0, key: "iTunesDBMetadataType", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required i_tunes_db_metadata_type: GetDBiTunesInfoITunesDBMetadataType => { bit: 0, key: "iTunesDBMetadataType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetDBiTunesInfoITunesDBMetadataType: u8 {
        /// database UID
        DatabaseUID = 0,
        /// last iTunes sync date/time
        LastITunesSyncDateTime = 1,
        /// total audio track count
        TotalAudioTrackCount = 2,
        /// total video track count
        TotalVideoTrackCount = 3,
        /// total audiobook count
        TotalAudiobookCount = 4,
        /// total photo count
        TotalPhotoCount = 5,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x003d,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetDBiTunesInfo {
        fields {
            /// iTunes Database Metadata Type
            // Virtual schema indices: 1
            required pub i_tunes_db_metadata_type: RetDBiTunesInfoITunesDBMetadataType => { bit: 0, key: "iTunesDBMetadataType", when: (truthy(&true)), predicate_value: true },
            /// iTunes Database Unique 64-Bit Identifier
            // Virtual schema indices: 2
            optional pub i_tunes_dbuid: u64 => { bit: 1, key: "iTunesDBUID", when: (eq(&i_tunes_db_metadata_type, &0u64)), predicate_value: false },
            /// Seconds
            // Virtual schema indices: 3
            optional pub seconds: u8 => { bit: 2, key: "seconds", when: (eq(&i_tunes_db_metadata_type, &1u64)), predicate_value: false },
            /// Minute
            // Virtual schema indices: 4
            optional pub minute: u8 => { bit: 3, key: "minute", when: (eq(&i_tunes_db_metadata_type, &1u64)), predicate_value: false },
            /// Hour
            // Virtual schema indices: 5
            optional pub hour: u8 => { bit: 4, key: "hour", when: (eq(&i_tunes_db_metadata_type, &1u64)), predicate_value: false },
            /// Day
            // Virtual schema indices: 6
            optional pub day: u8 => { bit: 5, key: "day", when: (eq(&i_tunes_db_metadata_type, &1u64)), predicate_value: false },
            /// Month
            // Virtual schema indices: 7
            optional pub month: u8 => { bit: 6, key: "month", when: (eq(&i_tunes_db_metadata_type, &1u64)), predicate_value: false },
            /// Year
            // Virtual schema indices: 8
            optional pub year: u16 => { bit: 7, key: "year", when: (eq(&i_tunes_db_metadata_type, &1u64)), predicate_value: false },
            /// Total Audio Track Count
            // Virtual schema indices: 9
            optional pub total_audio_track_count: u32 => { bit: 8, key: "totalAudioTrackCount", when: (eq(&i_tunes_db_metadata_type, &2u64)), predicate_value: false },
            /// Total Video Track Count
            // Virtual schema indices: 10
            optional pub total_video_track_count: u32 => { bit: 9, key: "totalVideoTrackCount", when: (eq(&i_tunes_db_metadata_type, &3u64)), predicate_value: false },
            /// Total Audiobook Count
            // Virtual schema indices: 11
            optional pub total_audiobook_count: u32 => { bit: 10, key: "totalAudiobookCount", when: (eq(&i_tunes_db_metadata_type, &4u64)), predicate_value: false },
            /// Total Photo Count
            // Virtual schema indices: 12
            optional pub total_photo_count: u32 => { bit: 11, key: "totalPhotoCount", when: (eq(&i_tunes_db_metadata_type, &5u64)), predicate_value: false },
        }
        steps {
            1 => required i_tunes_db_metadata_type: RetDBiTunesInfoITunesDBMetadataType => { bit: 0, key: "iTunesDBMetadataType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional i_tunes_dbuid: u64 => { bit: 1, key: "iTunesDBUID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&i_tunes_db_metadata_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional seconds: u8 => { bit: 2, key: "seconds", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&i_tunes_db_metadata_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            4 => optional minute: u8 => { bit: 3, key: "minute", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&i_tunes_db_metadata_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            5 => optional hour: u8 => { bit: 4, key: "hour", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&i_tunes_db_metadata_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            6 => optional day: u8 => { bit: 5, key: "day", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&i_tunes_db_metadata_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            7 => optional month: u8 => { bit: 6, key: "month", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&i_tunes_db_metadata_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            8 => optional year: u16 => { bit: 7, key: "year", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&i_tunes_db_metadata_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            9 => optional total_audio_track_count: u32 => { bit: 8, key: "totalAudioTrackCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&i_tunes_db_metadata_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            10 => optional total_video_track_count: u32 => { bit: 9, key: "totalVideoTrackCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&i_tunes_db_metadata_type, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(3u8)] } },
            11 => optional total_audiobook_count: u32 => { bit: 10, key: "totalAudiobookCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&i_tunes_db_metadata_type, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(4u8)] } },
            12 => optional total_photo_count: u32 => { bit: 11, key: "totalPhotoCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&i_tunes_db_metadata_type, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(5u8)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct GetUIDTrackInfoTrackInformationTypeBits: u32, little_endian {
        /// capabilities
        const CAPABILITIES = 1 << 0;
        /// track name
        const TRACK_NAME = 1 << 1;
        /// artist name
        const ARTIST_NAME = 1 << 2;
        /// album name
        const ALBUM_NAME = 1 << 3;
        /// genre name
        const GENRE_NAME = 1 << 4;
        /// composer name
        const COMPOSER_NAME = 1 << 5;
        /// total track time duration
        const TOTAL_TRACK_TIME_DURATION = 1 << 6;
        /// unique track identifier
        const UNIQUE_TRACK_IDENTIFIER = 1 << 7;
        /// chapter count
        const CHAPTER_COUNT = 1 << 8;
        /// chapter times
        const CHAPTER_TIMES = 1 << 9;
        /// chapter names
        const CHAPTER_NAMES = 1 << 10;
        /// lyrics of currently playing song
        const LYRICS_OF_CURRENTLY_PLAYING_SONG = 1 << 11;
        /// description
        const DESCRIPTION = 1 << 12;
        /// album track index
        const ALBUM_TRACK_INDEX = 1 << 13;
        /// disc set album index
        const DISC_SET_ALBUM_INDEX = 1 << 14;
        /// play count
        const PLAY_COUNT = 1 << 15;
        /// skip count
        const SKIP_COUNT = 1 << 16;
        /// podcast release date
        const PODCAST_RELEASE_DATE = 1 << 17;
        /// last played date/time
        const LAST_PLAYED_DATE_TIME = 1 << 18;
        /// year (release date)
        const YEAR_RELEASE_DATE = 1 << 19;
        /// star rating
        const STAR_RATING = 1 << 20;
        /// series name
        const SERIES_NAME = 1 << 21;
        /// season number
        const SEASON_NUMBER = 1 << 22;
        /// track volume adjust
        const TRACK_VOLUME_ADJUST = 1 << 23;
        /// track EQ preset
        const TRACK_EQ_PRESET = 1 << 24;
        /// track sample rate
        const TRACK_SAMPLE_RATE = 1 << 25;
        /// bookmark offset
        const BOOKMARK_OFFSET = 1 << 26;
        /// start/stop time offset
        const START_STOP_TIME_OFFSET = 1 << 27;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x003e,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetUIDTrackInfo {
        fields {
            /// Unique Track Identifier
            // Virtual schema indices: 1
            required pub unique_track_identifier: u64 => { bit: 0, key: "uniqueTrackIdentifier", when: (truthy(&true)), predicate_value: false },
            /// Track Information Type Bits
            // Virtual schema indices: 2
            required pub track_information_type_bits: GetUIDTrackInfoTrackInformationTypeBits => { bit: 1, key: "trackInformationTypeBits", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required unique_track_identifier: u64 => { bit: 0, key: "uniqueTrackIdentifier", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required track_information_type_bits: GetUIDTrackInfoTrackInformationTypeBits => { bit: 1, key: "trackInformationTypeBits", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetUIDTrackInfoTrackInformationType: u8 {
        /// capabilities
        Capabilities = 0,
        /// track name
        TrackName = 1,
        /// artist name
        ArtistName = 2,
        /// album name
        AlbumName = 3,
        /// genre name
        GenreName = 4,
        /// composer name
        ComposerName = 5,
        /// total track time duration
        TotalTrackTimeDuration = 6,
        /// unique track identifier
        UniqueTrackIdentifier = 7,
        /// chapter count
        ChapterCount = 8,
        /// chapter times
        ChapterTimes = 9,
        /// chapter names
        ChapterNames = 10,
        /// lyrics of currently playing song
        LyricsOfCurrentlyPlayingSong = 11,
        /// description
        Description = 12,
        /// album track index
        AlbumTrackIndex = 13,
        /// disc set album index
        DiscSetAlbumIndex = 14,
        /// play count
        PlayCount = 15,
        /// skip count
        SkipCount = 16,
        /// podcast release date
        PodcastReleaseDate = 17,
        /// last played date/time
        LastPlayedDateTime = 18,
        /// year (release date)
        YearReleaseDate = 19,
        /// star rating
        StarRating = 20,
        /// series name
        SeriesName = 21,
        /// season number
        SeasonNumber = 22,
        /// track volume adjust
        TrackVolumeAdjust = 23,
        /// track EQ preset
        TrackEQPreset = 24,
        /// track sample rate
        TrackSampleRate = 25,
        /// bookmark offset
        BookmarkOffset = 26,
        /// start/stop time offset
        StartStopTimeOffset = 27,
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetUIDTrackInfoCapabilitiesBits: u32 {
        /// is audiobook
        const IS_AUDIOBOOK = 1 << 0;
        /// has chapters
        const HAS_CHAPTERS = 1 << 1;
        /// has artwork
        const HAS_ARTWORK = 1 << 2;
        /// has lyrics
        const HAS_LYRICS = 1 << 3;
        /// is podcast episode
        const IS_PODCAST_EPISODE = 1 << 4;
        /// has release date
        const HAS_RELEASE_DATE = 1 << 5;
        /// has description
        const HAS_DESCRIPTION = 1 << 6;
        /// is video
        const IS_VIDEO = 1 << 7;
        /// is queued as video
        const IS_QUEUED_AS_VIDEO = 1 << 8;
        /// capable of generating a Genius playlist
        const CAPABLE_OF_GENERATING_A_GENIUS_PLAYLIST = 1 << 13;
        /// iTunes U episode
        const I_TUNES_U_EPISODE = 1 << 14;
    }
}

iap1_record! {
    strings = LingoString;
    pub struct RetUIDTrackInfoChapterTimes {
        fields {
            /// Chapter Index
            // Virtual schema indices: 1
            required pub chapter_index: u16 => { bit: 0, key: "chapterIndex", when: (truthy(&true)), predicate_value: false },
            /// Chapter Offset
            // Virtual schema indices: 2
            required pub chapter_offset: u32 => { bit: 1, key: "chapterOffset", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required chapter_index: u16 => { bit: 0, key: "chapterIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required chapter_offset: u32 => { bit: 1, key: "chapterOffset", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_record! {
    strings = LingoString;
    pub struct RetUIDTrackInfoChapterNames {
        fields {
            /// Chapter Index
            // Virtual schema indices: 1
            required pub chapter_index: u16 => { bit: 0, key: "chapterIndex", when: (truthy(&true)), predicate_value: false },
            /// Chapter Name
            // Virtual schema indices: 2
            required pub chapter_name: String => { bit: 1, key: "chapterName", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required chapter_index: u16 => { bit: 0, key: "chapterIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required chapter_name: String => { bit: 1, key: "chapterName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x003f,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetUIDTrackInfo {
        fields {
            /// Unique Track Identifier
            // Virtual schema indices: 1
            required pub unique_track_identifier: u64 => { bit: 0, key: "uniqueTrackIdentifier", when: (truthy(&true)), predicate_value: false },
            /// Track Information Type
            // Virtual schema indices: 2
            required pub track_information_type: RetUIDTrackInfoTrackInformationType => { bit: 1, key: "trackInformationType", when: (truthy(&true)), predicate_value: true },
            /// Capabilities Bits
            // Virtual schema indices: 3
            optional pub capabilities_bits: RetUIDTrackInfoCapabilitiesBits => { bit: 2, key: "capabilitiesBits", when: (eq(&track_information_type, &0u64)), predicate_value: false },
            /// Track Name
            // Virtual schema indices: 4
            optional pub track_name: String => { bit: 3, key: "trackName", when: (eq(&track_information_type, &1u64)), predicate_value: false },
            /// Artist Name
            // Virtual schema indices: 5
            optional pub artist_name: String => { bit: 4, key: "artistName", when: (eq(&track_information_type, &2u64)), predicate_value: false },
            /// Album Name
            // Virtual schema indices: 6
            optional pub album_name: String => { bit: 5, key: "albumName", when: (eq(&track_information_type, &3u64)), predicate_value: false },
            /// Genre Name
            // Virtual schema indices: 7
            optional pub genre_name: String => { bit: 6, key: "genreName", when: (eq(&track_information_type, &4u64)), predicate_value: false },
            /// Composer Name
            // Virtual schema indices: 8
            optional pub composer_name: String => { bit: 7, key: "composerName", when: (eq(&track_information_type, &5u64)), predicate_value: false },
            /// Total Track Duration
            // Virtual schema indices: 9
            optional pub total_track_duration: u32 => { bit: 8, key: "totalTrackDuration", when: (eq(&track_information_type, &6u64)), predicate_value: false },
            /// iTunes Unique Track ID
            // Virtual schema indices: 10
            optional pub i_tunes_track_uid: u64 => { bit: 9, key: "iTunesTrackUID", when: (eq(&track_information_type, &7u64)), predicate_value: false },
            /// Chapter Count
            // Virtual schema indices: 11
            optional pub chapter_count: u16 => { bit: 10, key: "chapterCount", when: (eq(&track_information_type, &8u64)), predicate_value: false },
            /// Chapter Times
            // Virtual schema indices: 12
            optional pub chapter_times: Vec<RetUIDTrackInfoChapterTimes> => { bit: 11, key: "chapterTimes", when: (eq(&track_information_type, &9u64)), predicate_value: false },
            /// Chapter Names
            // Virtual schema indices: 13
            optional pub chapter_names: Vec<RetUIDTrackInfoChapterNames> => { bit: 12, key: "chapterNames", when: (eq(&track_information_type, &10u64)), predicate_value: false },
            /// Current Track Lyrics Section Index
            // Virtual schema indices: 14
            optional pub current_track_lyrics_section_index: u16 => { bit: 13, key: "currentTrackLyricsSectionIndex", when: (eq(&track_information_type, &11u64)), predicate_value: false },
            /// Maximum Track Lyrics Section Index
            // Virtual schema indices: 15
            optional pub maximum_track_lyrics_section_index: u16 => { bit: 14, key: "maximumTrackLyricsSectionIndex", when: (eq(&track_information_type, &11u64)), predicate_value: false },
            /// Track Lyrics String Data
            // Virtual schema indices: 16
            optional pub track_lyrics_string_data: Vec<u8> => { bit: 15, key: "trackLyricsStringData", when: (eq(&track_information_type, &11u64)), predicate_value: false },
            /// Description
            // Virtual schema indices: 17
            optional pub track_description: String => { bit: 16, key: "trackDescription", when: (eq(&track_information_type, &12u64)), predicate_value: false },
            /// Album Track Index
            // Virtual schema indices: 18
            optional pub album_track_index: u16 => { bit: 17, key: "albumTrackIndex", when: (eq(&track_information_type, &13u64)), predicate_value: false },
            /// Disc Set Album Index
            // Virtual schema indices: 19
            optional pub disc_set_album_index: u16 => { bit: 18, key: "discSetAlbumIndex", when: (eq(&track_information_type, &14u64)), predicate_value: false },
            /// Play Count
            // Virtual schema indices: 20
            optional pub play_count: u32 => { bit: 19, key: "playCount", when: (eq(&track_information_type, &15u64)), predicate_value: false },
            /// Skip Count
            // Virtual schema indices: 21
            optional pub skip_count: u32 => { bit: 20, key: "skipCount", when: (eq(&track_information_type, &16u64)), predicate_value: false },
            /// Seconds
            // Virtual schema indices: 22, 28
            optional pub seconds: u8 => { bit: 21, key: "seconds", when: (eq(&track_information_type, &17u64)) || (eq(&track_information_type, &18u64)), predicate_value: false },
            /// Minute
            // Virtual schema indices: 23, 29
            optional pub minute: u8 => { bit: 22, key: "minute", when: (eq(&track_information_type, &17u64)) || (eq(&track_information_type, &18u64)), predicate_value: false },
            /// Hour
            // Virtual schema indices: 24, 30
            optional pub hour: u8 => { bit: 23, key: "hour", when: (eq(&track_information_type, &17u64)) || (eq(&track_information_type, &18u64)), predicate_value: false },
            /// Day
            // Virtual schema indices: 25, 31
            optional pub day: u8 => { bit: 24, key: "day", when: (eq(&track_information_type, &17u64)) || (eq(&track_information_type, &18u64)), predicate_value: false },
            /// Month
            // Virtual schema indices: 26, 32
            optional pub month: u8 => { bit: 25, key: "month", when: (eq(&track_information_type, &17u64)) || (eq(&track_information_type, &18u64)), predicate_value: false },
            /// Year
            // Virtual schema indices: 27, 33, 34
            optional pub year: u16 => { bit: 26, key: "year", when: (eq(&track_information_type, &17u64)) || (eq(&track_information_type, &18u64)) || (eq(&track_information_type, &19u64)), predicate_value: false },
            /// Star Rating
            // Virtual schema indices: 35
            optional pub star_rating: u8 => { bit: 27, key: "starRating", when: (eq(&track_information_type, &20u64)), predicate_value: false },
            /// Series Name
            // Virtual schema indices: 36
            optional pub series_name: u8 => { bit: 28, key: "seriesName", when: (eq(&track_information_type, &21u64)), predicate_value: false },
            /// Season Number
            // Virtual schema indices: 37
            optional pub season_number: u16 => { bit: 29, key: "seasonNumber", when: (eq(&track_information_type, &22u64)), predicate_value: false },
            /// Track Volume Adjust
            // Virtual schema indices: 38
            optional pub track_volume_adjust: u8 => { bit: 30, key: "trackVolumeAdjust", when: (eq(&track_information_type, &23u64)), predicate_value: false },
            /// Track EQ Preset Index
            // Virtual schema indices: 39
            optional pub track_eq_preset_index: u16 => { bit: 31, key: "trackEQPresetIndex", when: (eq(&track_information_type, &24u64)), predicate_value: false },
            /// Data Sample Rate
            // Virtual schema indices: 40
            optional pub data_rate: u32 => { bit: 32, key: "dataRate", when: (eq(&track_information_type, &25u64)), predicate_value: false },
            /// Bookmark Offset from Start
            // Virtual schema indices: 41
            optional pub bookmark_offset: u32 => { bit: 33, key: "bookmark offset", when: (eq(&track_information_type, &26u64)), predicate_value: false },
            /// Start Time Offset
            // Virtual schema indices: 42
            optional pub start_time_offset: u32 => { bit: 34, key: "startTimeOffset", when: (eq(&track_information_type, &27u64)), predicate_value: false },
            /// Stop Time Offset
            // Virtual schema indices: 43
            optional pub stop_time_offset: u32 => { bit: 35, key: "stopTimeOffset", when: (eq(&track_information_type, &27u64)), predicate_value: false },
        }
        steps {
            1 => required unique_track_identifier: u64 => { bit: 0, key: "uniqueTrackIdentifier", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required track_information_type: RetUIDTrackInfoTrackInformationType => { bit: 1, key: "trackInformationType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => optional capabilities_bits: RetUIDTrackInfoCapabilitiesBits => { bit: 2, key: "capabilitiesBits", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(0u8)] } },
            4 => optional track_name: String => { bit: 3, key: "trackName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&track_information_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(1u8)] } },
            5 => optional artist_name: String => { bit: 4, key: "artistName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&track_information_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(2u8)] } },
            6 => optional album_name: String => { bit: 5, key: "albumName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&track_information_type, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(3u8)] } },
            7 => optional genre_name: String => { bit: 6, key: "genreName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&track_information_type, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(4u8)] } },
            8 => optional composer_name: String => { bit: 7, key: "composerName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&track_information_type, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(5u8)] } },
            9 => optional total_track_duration: u32 => { bit: 8, key: "totalTrackDuration", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &6u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(6u8)] } },
            10 => optional i_tunes_track_uid: u64 => { bit: 9, key: "iTunesTrackUID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &7u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(7u8)] } },
            11 => optional chapter_count: u16 => { bit: 10, key: "chapterCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &8u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(8u8)] } },
            12 => optional chapter_times: Vec<RetUIDTrackInfoChapterTimes> => { bit: 11, key: "chapterTimes", wire: [counted_remaining], project_wire: Iap1WireSpec::CountedRemaining, predicate_value: false, predicate: { expr: eq(&track_information_type, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(9u8)] } },
            13 => optional chapter_names: Vec<RetUIDTrackInfoChapterNames> => { bit: 12, key: "chapterNames", wire: [counted_remaining], project_wire: Iap1WireSpec::CountedRemaining, predicate_value: false, predicate: { expr: eq(&track_information_type, &10u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(10u8)] } },
            14 => optional current_track_lyrics_section_index: u16 => { bit: 13, key: "currentTrackLyricsSectionIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &11u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(11u8)] } },
            15 => optional maximum_track_lyrics_section_index: u16 => { bit: 14, key: "maximumTrackLyricsSectionIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &11u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(11u8)] } },
            16 => optional track_lyrics_string_data: Vec<u8> => { bit: 15, key: "trackLyricsStringData", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&track_information_type, &11u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(11u8)] } },
            17 => optional track_description: String => { bit: 16, key: "trackDescription", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&track_information_type, &12u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(12u8)] } },
            18 => optional album_track_index: u16 => { bit: 17, key: "albumTrackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &13u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(13u8)] } },
            19 => optional disc_set_album_index: u16 => { bit: 18, key: "discSetAlbumIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &14u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(14u8)] } },
            20 => optional play_count: u32 => { bit: 19, key: "playCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &15u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(15u8)] } },
            21 => optional skip_count: u32 => { bit: 20, key: "skipCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &16u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(16u8)] } },
            22 => optional seconds: u8 => { bit: 21, key: "seconds", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(17u8)] } },
            23 => optional minute: u8 => { bit: 22, key: "minute", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(17u8)] } },
            24 => optional hour: u8 => { bit: 23, key: "hour", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(17u8)] } },
            25 => optional day: u8 => { bit: 24, key: "day", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(17u8)] } },
            26 => optional month: u8 => { bit: 25, key: "month", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(17u8)] } },
            27 => optional year: u16 => { bit: 26, key: "year", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(17u8)] } },
            28 => optional seconds: u8 => { bit: 21, key: "seconds", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &18u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(18u8)] } },
            29 => optional minute: u8 => { bit: 22, key: "minute", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &18u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(18u8)] } },
            30 => optional hour: u8 => { bit: 23, key: "hour", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &18u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(18u8)] } },
            31 => optional day: u8 => { bit: 24, key: "day", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &18u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(18u8)] } },
            32 => optional month: u8 => { bit: 25, key: "month", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &18u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(18u8)] } },
            33 => optional year: u16 => { bit: 26, key: "year", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &18u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(18u8)] } },
            34 => optional year: u16 => { bit: 26, key: "year", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &19u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(19u8)] } },
            35 => optional star_rating: u8 => { bit: 27, key: "starRating", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &20u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(20u8)] } },
            36 => optional series_name: u8 => { bit: 28, key: "seriesName", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &21u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(21u8)] } },
            37 => optional season_number: u16 => { bit: 29, key: "seasonNumber", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &22u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(22u8)] } },
            38 => optional track_volume_adjust: u8 => { bit: 30, key: "trackVolumeAdjust", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &23u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(23u8)] } },
            39 => optional track_eq_preset_index: u16 => { bit: 31, key: "trackEQPresetIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &24u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(24u8)] } },
            40 => optional data_rate: u32 => { bit: 32, key: "dataRate", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &25u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(25u8)] } },
            41 => optional bookmark_offset: u32 => { bit: 33, key: "bookmark offset", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &26u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(26u8)] } },
            42 => optional start_time_offset: u32 => { bit: 34, key: "startTimeOffset", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &27u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(27u8)] } },
            43 => optional stop_time_offset: u32 => { bit: 35, key: "stopTimeOffset", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &27u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(27u8)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct GetDBTrackInfoTrackInformationTypeBits: u32, little_endian {
        /// capabilities
        const CAPABILITIES = 1 << 0;
        /// track name
        const TRACK_NAME = 1 << 1;
        /// artist name
        const ARTIST_NAME = 1 << 2;
        /// album name
        const ALBUM_NAME = 1 << 3;
        /// genre name
        const GENRE_NAME = 1 << 4;
        /// composer name
        const COMPOSER_NAME = 1 << 5;
        /// total track time duration
        const TOTAL_TRACK_TIME_DURATION = 1 << 6;
        /// unique track identifier
        const UNIQUE_TRACK_IDENTIFIER = 1 << 7;
        /// chapter count
        const CHAPTER_COUNT = 1 << 8;
        /// chapter times
        const CHAPTER_TIMES = 1 << 9;
        /// chapter names
        const CHAPTER_NAMES = 1 << 10;
        /// lyrics of currently playing song
        const LYRICS_OF_CURRENTLY_PLAYING_SONG = 1 << 11;
        /// description
        const DESCRIPTION = 1 << 12;
        /// album track index
        const ALBUM_TRACK_INDEX = 1 << 13;
        /// disc set album index
        const DISC_SET_ALBUM_INDEX = 1 << 14;
        /// play count
        const PLAY_COUNT = 1 << 15;
        /// skip count
        const SKIP_COUNT = 1 << 16;
        /// podcast release date
        const PODCAST_RELEASE_DATE = 1 << 17;
        /// last played date/time
        const LAST_PLAYED_DATE_TIME = 1 << 18;
        /// year (release date)
        const YEAR_RELEASE_DATE = 1 << 19;
        /// star rating
        const STAR_RATING = 1 << 20;
        /// series name
        const SERIES_NAME = 1 << 21;
        /// season number
        const SEASON_NUMBER = 1 << 22;
        /// track volume adjust
        const TRACK_VOLUME_ADJUST = 1 << 23;
        /// track EQ preset
        const TRACK_EQ_PRESET = 1 << 24;
        /// track sample rate
        const TRACK_SAMPLE_RATE = 1 << 25;
        /// bookmark offset
        const BOOKMARK_OFFSET = 1 << 26;
        /// start/stop time offset
        const START_STOP_TIME_OFFSET = 1 << 27;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0040,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetDBTrackInfo {
        fields {
            /// Track Database Start Index
            // Virtual schema indices: 1
            required pub track_database_start_index: u32 => { bit: 0, key: "trackDatabaseStartIndex", when: (truthy(&true)), predicate_value: false },
            /// Track Count (from Track Start Index)
            // Virtual schema indices: 2
            required pub track_count_from_start_index: i32 => { bit: 1, key: "trackCountFromStartIndex", when: (truthy(&true)), predicate_value: false },
            /// Track Information type Bits
            // Virtual schema indices: 3
            required pub track_information_type_bits: GetDBTrackInfoTrackInformationTypeBits => { bit: 2, key: "trackInformationTypeBits", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required track_database_start_index: u32 => { bit: 0, key: "trackDatabaseStartIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required track_count_from_start_index: i32 => { bit: 1, key: "trackCountFromStartIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required track_information_type_bits: GetDBTrackInfoTrackInformationTypeBits => { bit: 2, key: "trackInformationTypeBits", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetDBTrackInfoTrackInformationType: u8 {
        /// capabilities
        Capabilities = 0,
        /// track name
        TrackName = 1,
        /// artist name
        ArtistName = 2,
        /// album name
        AlbumName = 3,
        /// genre name
        GenreName = 4,
        /// composer name
        ComposerName = 5,
        /// total track time duration
        TotalTrackTimeDuration = 6,
        /// unique track identifier
        UniqueTrackIdentifier = 7,
        /// chapter count
        ChapterCount = 8,
        /// chapter times
        ChapterTimes = 9,
        /// chapter names
        ChapterNames = 10,
        /// lyrics of currently playing song
        LyricsOfCurrentlyPlayingSong = 11,
        /// description
        Description = 12,
        /// album track index
        AlbumTrackIndex = 13,
        /// disc set album index
        DiscSetAlbumIndex = 14,
        /// play count
        PlayCount = 15,
        /// skip count
        SkipCount = 16,
        /// podcast release date
        PodcastReleaseDate = 17,
        /// last played date/time
        LastPlayedDateTime = 18,
        /// year (release date)
        YearReleaseDate = 19,
        /// star rating
        StarRating = 20,
        /// series name
        SeriesName = 21,
        /// season number
        SeasonNumber = 22,
        /// track volume adjust
        TrackVolumeAdjust = 23,
        /// track EQ preset
        TrackEQPreset = 24,
        /// track sample rate
        TrackSampleRate = 25,
        /// bookmark offset
        BookmarkOffset = 26,
        /// start/stop time offset
        StartStopTimeOffset = 27,
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetDBTrackInfoCapabilitiesBits: u32 {
        /// is audiobook
        const IS_AUDIOBOOK = 1 << 0;
        /// has chapters
        const HAS_CHAPTERS = 1 << 1;
        /// has artwork
        const HAS_ARTWORK = 1 << 2;
        /// has lyrics
        const HAS_LYRICS = 1 << 3;
        /// is podcast episode
        const IS_PODCAST_EPISODE = 1 << 4;
        /// has release date
        const HAS_RELEASE_DATE = 1 << 5;
        /// has description
        const HAS_DESCRIPTION = 1 << 6;
        /// is video
        const IS_VIDEO = 1 << 7;
        /// is queued as video
        const IS_QUEUED_AS_VIDEO = 1 << 8;
        /// capable of generating a Genius playlist
        const CAPABLE_OF_GENERATING_A_GENIUS_PLAYLIST = 1 << 13;
        /// iTunes U episode
        const I_TUNES_U_EPISODE = 1 << 14;
    }
}

iap1_record! {
    strings = LingoString;
    pub struct RetDBTrackInfoChapterTimes {
        fields {
            /// Chapter Index
            // Virtual schema indices: 1
            required pub chapter_index: u16 => { bit: 0, key: "chapterIndex", when: (truthy(&true)), predicate_value: false },
            /// Chapter Offset
            // Virtual schema indices: 2
            required pub chapter_offset: u32 => { bit: 1, key: "chapterOffset", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required chapter_index: u16 => { bit: 0, key: "chapterIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required chapter_offset: u32 => { bit: 1, key: "chapterOffset", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_record! {
    strings = LingoString;
    pub struct RetDBTrackInfoChapterNames {
        fields {
            /// Chapter Index
            // Virtual schema indices: 1
            required pub chapter_index: u16 => { bit: 0, key: "chapterIndex", when: (truthy(&true)), predicate_value: false },
            /// Chapter Name
            // Virtual schema indices: 2
            required pub chapter_name: String => { bit: 1, key: "chapterName", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required chapter_index: u16 => { bit: 0, key: "chapterIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required chapter_name: String => { bit: 1, key: "chapterName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0041,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetDBTrackInfo {
        fields {
            /// Track Database Index
            // Virtual schema indices: 1
            required pub track_database_index: u32 => { bit: 0, key: "trackDatabaseIndex", when: (truthy(&true)), predicate_value: false },
            /// Track Information Type
            // Virtual schema indices: 2
            required pub track_information_type: RetDBTrackInfoTrackInformationType => { bit: 1, key: "trackInformationType", when: (truthy(&true)), predicate_value: true },
            /// Capabilities Bits
            // Virtual schema indices: 3
            optional pub capabilities_bits: RetDBTrackInfoCapabilitiesBits => { bit: 2, key: "capabilitiesBits", when: (eq(&track_information_type, &0u64)), predicate_value: false },
            /// Track Name
            // Virtual schema indices: 4
            optional pub track_name: String => { bit: 3, key: "trackName", when: (eq(&track_information_type, &1u64)), predicate_value: false },
            /// Artist Name
            // Virtual schema indices: 5
            optional pub artist_name: String => { bit: 4, key: "artistName", when: (eq(&track_information_type, &2u64)), predicate_value: false },
            /// Album Name
            // Virtual schema indices: 6
            optional pub album_name: String => { bit: 5, key: "albumName", when: (eq(&track_information_type, &3u64)), predicate_value: false },
            /// Genre Name
            // Virtual schema indices: 7
            optional pub genre_name: String => { bit: 6, key: "genreName", when: (eq(&track_information_type, &4u64)), predicate_value: false },
            /// Composer Name
            // Virtual schema indices: 8
            optional pub composer_name: String => { bit: 7, key: "composerName", when: (eq(&track_information_type, &5u64)), predicate_value: false },
            /// Total Track Duration
            // Virtual schema indices: 9
            optional pub total_track_duration: u32 => { bit: 8, key: "totalTrackDuration", when: (eq(&track_information_type, &6u64)), predicate_value: false },
            /// iTunes Unique Track ID
            // Virtual schema indices: 10
            optional pub i_tunes_track_uid: u64 => { bit: 9, key: "iTunesTrackUID", when: (eq(&track_information_type, &7u64)), predicate_value: false },
            /// Chapter Count
            // Virtual schema indices: 11
            optional pub chapter_count: u16 => { bit: 10, key: "chapterCount", when: (eq(&track_information_type, &8u64)), predicate_value: false },
            /// Chapter Times
            // Virtual schema indices: 12
            optional pub chapter_times: Vec<RetDBTrackInfoChapterTimes> => { bit: 11, key: "chapterTimes", when: (eq(&track_information_type, &9u64)), predicate_value: false },
            /// Chapter Names
            // Virtual schema indices: 13
            optional pub chapter_names: Vec<RetDBTrackInfoChapterNames> => { bit: 12, key: "chapterNames", when: (eq(&track_information_type, &10u64)), predicate_value: false },
            /// Current Track Lyrics Section index
            // Virtual schema indices: 14
            optional pub current_track_lyrics_section_index: u16 => { bit: 13, key: "currentTrackLyricsSectionIndex", when: (eq(&track_information_type, &11u64)), predicate_value: false },
            /// Maximum Track Lyrics Section index
            // Virtual schema indices: 15
            optional pub maximum_track_lyrics_section_index: u16 => { bit: 14, key: "maximumTrackLyricsSectionIndex", when: (eq(&track_information_type, &11u64)), predicate_value: false },
            /// Track Lyrics String Data
            // Virtual schema indices: 16
            optional pub track_lyrics_string_data: Vec<u8> => { bit: 15, key: "trackLyricsStringData", when: (eq(&track_information_type, &11u64)), predicate_value: false },
            /// Description
            // Virtual schema indices: 17
            optional pub track_description: String => { bit: 16, key: "trackDescription", when: (eq(&track_information_type, &12u64)), predicate_value: false },
            /// Album Track Index
            // Virtual schema indices: 18
            optional pub album_track_index: u16 => { bit: 17, key: "albumTrackIndex", when: (eq(&track_information_type, &13u64)), predicate_value: false },
            /// Disc Set Album Index
            // Virtual schema indices: 19
            optional pub disc_set_album_index: u16 => { bit: 18, key: "discSetAlbumIndex", when: (eq(&track_information_type, &14u64)), predicate_value: false },
            /// Play Count
            // Virtual schema indices: 20
            optional pub play_count: u32 => { bit: 19, key: "playCount", when: (eq(&track_information_type, &15u64)), predicate_value: false },
            /// Skip Count
            // Virtual schema indices: 21
            optional pub skip_count: u32 => { bit: 20, key: "skipCount", when: (eq(&track_information_type, &16u64)), predicate_value: false },
            /// Seconds
            // Virtual schema indices: 22, 28
            optional pub seconds: u8 => { bit: 21, key: "seconds", when: (eq(&track_information_type, &17u64)) || (eq(&track_information_type, &18u64)), predicate_value: false },
            /// Minute
            // Virtual schema indices: 23, 29
            optional pub minute: u8 => { bit: 22, key: "minute", when: (eq(&track_information_type, &17u64)) || (eq(&track_information_type, &18u64)), predicate_value: false },
            /// Hour
            // Virtual schema indices: 24, 30
            optional pub hour: u8 => { bit: 23, key: "hour", when: (eq(&track_information_type, &17u64)) || (eq(&track_information_type, &18u64)), predicate_value: false },
            /// Day
            // Virtual schema indices: 25, 31
            optional pub day: u8 => { bit: 24, key: "day", when: (eq(&track_information_type, &17u64)) || (eq(&track_information_type, &18u64)), predicate_value: false },
            /// Month
            // Virtual schema indices: 26, 32
            optional pub month: u8 => { bit: 25, key: "month", when: (eq(&track_information_type, &17u64)) || (eq(&track_information_type, &18u64)), predicate_value: false },
            /// Year
            // Virtual schema indices: 27, 33, 34
            optional pub year: u16 => { bit: 26, key: "year", when: (eq(&track_information_type, &17u64)) || (eq(&track_information_type, &18u64)) || (eq(&track_information_type, &19u64)), predicate_value: false },
            /// Star Rating
            // Virtual schema indices: 35
            optional pub star_rating: u8 => { bit: 27, key: "starRating", when: (eq(&track_information_type, &20u64)), predicate_value: false },
            /// Series Name
            // Virtual schema indices: 36
            optional pub series_name: u8 => { bit: 28, key: "seriesName", when: (eq(&track_information_type, &21u64)), predicate_value: false },
            /// Season Number
            // Virtual schema indices: 37
            optional pub season_number: u16 => { bit: 29, key: "seasonNumber", when: (eq(&track_information_type, &22u64)), predicate_value: false },
            /// Track Volume Adjust
            // Virtual schema indices: 38
            optional pub track_volume_adjust: u8 => { bit: 30, key: "trackVolumeAdjust", when: (eq(&track_information_type, &23u64)), predicate_value: false },
            /// Track EQ Preset Index
            // Virtual schema indices: 39
            optional pub track_eq_preset_index: u16 => { bit: 31, key: "trackEQPresetIndex", when: (eq(&track_information_type, &24u64)), predicate_value: false },
            /// Data Sample Rate
            // Virtual schema indices: 40
            optional pub data_rate: u32 => { bit: 32, key: "dataRate", when: (eq(&track_information_type, &25u64)), predicate_value: false },
            /// Bookmark Offset from Start
            // Virtual schema indices: 41
            optional pub bookmark_offset: u32 => { bit: 33, key: "bookmark offset", when: (eq(&track_information_type, &26u64)), predicate_value: false },
            /// Start Time Offset
            // Virtual schema indices: 42
            optional pub start_time_offset: u32 => { bit: 34, key: "startTimeOffset", when: (eq(&track_information_type, &27u64)), predicate_value: false },
            /// Stop Time Offset
            // Virtual schema indices: 43
            optional pub stop_time_offset: u32 => { bit: 35, key: "stopTimeOffset", when: (eq(&track_information_type, &27u64)), predicate_value: false },
        }
        steps {
            1 => required track_database_index: u32 => { bit: 0, key: "trackDatabaseIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required track_information_type: RetDBTrackInfoTrackInformationType => { bit: 1, key: "trackInformationType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => optional capabilities_bits: RetDBTrackInfoCapabilitiesBits => { bit: 2, key: "capabilitiesBits", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(0u8)] } },
            4 => optional track_name: String => { bit: 3, key: "trackName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&track_information_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(1u8)] } },
            5 => optional artist_name: String => { bit: 4, key: "artistName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&track_information_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(2u8)] } },
            6 => optional album_name: String => { bit: 5, key: "albumName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&track_information_type, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(3u8)] } },
            7 => optional genre_name: String => { bit: 6, key: "genreName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&track_information_type, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(4u8)] } },
            8 => optional composer_name: String => { bit: 7, key: "composerName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&track_information_type, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(5u8)] } },
            9 => optional total_track_duration: u32 => { bit: 8, key: "totalTrackDuration", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &6u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(6u8)] } },
            10 => optional i_tunes_track_uid: u64 => { bit: 9, key: "iTunesTrackUID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &7u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(7u8)] } },
            11 => optional chapter_count: u16 => { bit: 10, key: "chapterCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &8u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(8u8)] } },
            12 => optional chapter_times: Vec<RetDBTrackInfoChapterTimes> => { bit: 11, key: "chapterTimes", wire: [counted_remaining], project_wire: Iap1WireSpec::CountedRemaining, predicate_value: false, predicate: { expr: eq(&track_information_type, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(9u8)] } },
            13 => optional chapter_names: Vec<RetDBTrackInfoChapterNames> => { bit: 12, key: "chapterNames", wire: [counted_remaining], project_wire: Iap1WireSpec::CountedRemaining, predicate_value: false, predicate: { expr: eq(&track_information_type, &10u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(10u8)] } },
            14 => optional current_track_lyrics_section_index: u16 => { bit: 13, key: "currentTrackLyricsSectionIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &11u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(11u8)] } },
            15 => optional maximum_track_lyrics_section_index: u16 => { bit: 14, key: "maximumTrackLyricsSectionIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &11u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(11u8)] } },
            16 => optional track_lyrics_string_data: Vec<u8> => { bit: 15, key: "trackLyricsStringData", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&track_information_type, &11u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(11u8)] } },
            17 => optional track_description: String => { bit: 16, key: "trackDescription", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&track_information_type, &12u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(12u8)] } },
            18 => optional album_track_index: u16 => { bit: 17, key: "albumTrackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &13u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(13u8)] } },
            19 => optional disc_set_album_index: u16 => { bit: 18, key: "discSetAlbumIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &14u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(14u8)] } },
            20 => optional play_count: u32 => { bit: 19, key: "playCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &15u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(15u8)] } },
            21 => optional skip_count: u32 => { bit: 20, key: "skipCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &16u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(16u8)] } },
            22 => optional seconds: u8 => { bit: 21, key: "seconds", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(17u8)] } },
            23 => optional minute: u8 => { bit: 22, key: "minute", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(17u8)] } },
            24 => optional hour: u8 => { bit: 23, key: "hour", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(17u8)] } },
            25 => optional day: u8 => { bit: 24, key: "day", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(17u8)] } },
            26 => optional month: u8 => { bit: 25, key: "month", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(17u8)] } },
            27 => optional year: u16 => { bit: 26, key: "year", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(17u8)] } },
            28 => optional seconds: u8 => { bit: 21, key: "seconds", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &18u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(18u8)] } },
            29 => optional minute: u8 => { bit: 22, key: "minute", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &18u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(18u8)] } },
            30 => optional hour: u8 => { bit: 23, key: "hour", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &18u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(18u8)] } },
            31 => optional day: u8 => { bit: 24, key: "day", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &18u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(18u8)] } },
            32 => optional month: u8 => { bit: 25, key: "month", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &18u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(18u8)] } },
            33 => optional year: u16 => { bit: 26, key: "year", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &18u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(18u8)] } },
            34 => optional year: u16 => { bit: 26, key: "year", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &19u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(19u8)] } },
            35 => optional star_rating: u8 => { bit: 27, key: "starRating", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &20u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(20u8)] } },
            36 => optional series_name: u8 => { bit: 28, key: "seriesName", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &21u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(21u8)] } },
            37 => optional season_number: u16 => { bit: 29, key: "seasonNumber", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &22u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(22u8)] } },
            38 => optional track_volume_adjust: u8 => { bit: 30, key: "trackVolumeAdjust", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &23u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(23u8)] } },
            39 => optional track_eq_preset_index: u16 => { bit: 31, key: "trackEQPresetIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &24u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(24u8)] } },
            40 => optional data_rate: u32 => { bit: 32, key: "dataRate", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &25u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(25u8)] } },
            41 => optional bookmark_offset: u32 => { bit: 33, key: "bookmark offset", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &26u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(26u8)] } },
            42 => optional start_time_offset: u32 => { bit: 34, key: "startTimeOffset", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &27u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(27u8)] } },
            43 => optional stop_time_offset: u32 => { bit: 35, key: "stopTimeOffset", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &27u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(27u8)] } },
        }
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct GetPBTrackInfoTrackInformationTypeBits: u32, little_endian {
        /// capabilities
        const CAPABILITIES = 1 << 0;
        /// track name
        const TRACK_NAME = 1 << 1;
        /// artist name
        const ARTIST_NAME = 1 << 2;
        /// album name
        const ALBUM_NAME = 1 << 3;
        /// genre name
        const GENRE_NAME = 1 << 4;
        /// composer name
        const COMPOSER_NAME = 1 << 5;
        /// total track time duration
        const TOTAL_TRACK_TIME_DURATION = 1 << 6;
        /// unique track identifier
        const UNIQUE_TRACK_IDENTIFIER = 1 << 7;
        /// chapter count
        const CHAPTER_COUNT = 1 << 8;
        /// chapter times
        const CHAPTER_TIMES = 1 << 9;
        /// chapter names
        const CHAPTER_NAMES = 1 << 10;
        /// lyrics of currently playing song
        const LYRICS_OF_CURRENTLY_PLAYING_SONG = 1 << 11;
        /// description
        const DESCRIPTION = 1 << 12;
        /// album track index
        const ALBUM_TRACK_INDEX = 1 << 13;
        /// disc set album index
        const DISC_SET_ALBUM_INDEX = 1 << 14;
        /// play count
        const PLAY_COUNT = 1 << 15;
        /// skip count
        const SKIP_COUNT = 1 << 16;
        /// podcast release date
        const PODCAST_RELEASE_DATE = 1 << 17;
        /// last played date/time
        const LAST_PLAYED_DATE_TIME = 1 << 18;
        /// year (release date)
        const YEAR_RELEASE_DATE = 1 << 19;
        /// star rating
        const STAR_RATING = 1 << 20;
        /// series name
        const SERIES_NAME = 1 << 21;
        /// season number
        const SEASON_NUMBER = 1 << 22;
        /// track volume adjust
        const TRACK_VOLUME_ADJUST = 1 << 23;
        /// track EQ preset
        const TRACK_EQ_PRESET = 1 << 24;
        /// track sample rate
        const TRACK_SAMPLE_RATE = 1 << 25;
        /// bookmark offset
        const BOOKMARK_OFFSET = 1 << 26;
        /// start/stop time offset
        const START_STOP_TIME_OFFSET = 1 << 27;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0042,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetPBTrackInfo {
        fields {
            /// Track Playing Start Index
            // Virtual schema indices: 1
            required pub track_playing_start_index: u32 => { bit: 0, key: "trackPlayingStartIndex", when: (truthy(&true)), predicate_value: false },
            /// Track Count (from Track Start Index)
            // Virtual schema indices: 2
            required pub track_count_from_start_index: i32 => { bit: 1, key: "trackCountFromStartIndex", when: (truthy(&true)), predicate_value: false },
            /// Track Information Type Bits
            // Virtual schema indices: 3
            required pub track_information_type_bits: GetPBTrackInfoTrackInformationTypeBits => { bit: 2, key: "trackInformationTypeBits", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required track_playing_start_index: u32 => { bit: 0, key: "trackPlayingStartIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required track_count_from_start_index: i32 => { bit: 1, key: "trackCountFromStartIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required track_information_type_bits: GetPBTrackInfoTrackInformationTypeBits => { bit: 2, key: "trackInformationTypeBits", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetPBTrackInfoTrackInformationType: u8 {
        /// capabilities
        Capabilities = 0,
        /// track name
        TrackName = 1,
        /// artist name
        ArtistName = 2,
        /// album name
        AlbumName = 3,
        /// genre name
        GenreName = 4,
        /// composer name
        ComposerName = 5,
        /// total track time duration
        TotalTrackTimeDuration = 6,
        /// unique track identifier
        UniqueTrackIdentifier = 7,
        /// chapter count
        ChapterCount = 8,
        /// chapter times
        ChapterTimes = 9,
        /// chapter names
        ChapterNames = 10,
        /// lyrics of currently playing song
        LyricsOfCurrentlyPlayingSong = 11,
        /// description
        Description = 12,
        /// album track index
        AlbumTrackIndex = 13,
        /// disc set album index
        DiscSetAlbumIndex = 14,
        /// play count
        PlayCount = 15,
        /// skip count
        SkipCount = 16,
        /// podcast release date
        PodcastReleaseDate = 17,
        /// last played date/time
        LastPlayedDateTime = 18,
        /// year (release date)
        YearReleaseDate = 19,
        /// star rating
        StarRating = 20,
        /// series name
        SeriesName = 21,
        /// season number
        SeasonNumber = 22,
        /// track volume adjust
        TrackVolumeAdjust = 23,
        /// track EQ preset
        TrackEQPreset = 24,
        /// track sample rate
        TrackSampleRate = 25,
        /// bookmark offset
        BookmarkOffset = 26,
        /// start/stop time offset
        StartStopTimeOffset = 27,
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetPBTrackInfoCapabilitiesBits: u32 {
        /// is audiobook
        const IS_AUDIOBOOK = 1 << 0;
        /// has chapters
        const HAS_CHAPTERS = 1 << 1;
        /// has artwork
        const HAS_ARTWORK = 1 << 2;
        /// has lyrics
        const HAS_LYRICS = 1 << 3;
        /// is podcast episode
        const IS_PODCAST_EPISODE = 1 << 4;
        /// has release date
        const HAS_RELEASE_DATE = 1 << 5;
        /// has description
        const HAS_DESCRIPTION = 1 << 6;
        /// is video
        const IS_VIDEO = 1 << 7;
        /// is queued as video
        const IS_QUEUED_AS_VIDEO = 1 << 8;
        /// capable of generating a Genius playlist
        const CAPABLE_OF_GENERATING_A_GENIUS_PLAYLIST = 1 << 13;
        /// iTunes U episode
        const I_TUNES_U_EPISODE = 1 << 14;
    }
}

iap1_record! {
    strings = LingoString;
    pub struct RetPBTrackInfoChapterTimes {
        fields {
            /// Chapter Index
            // Virtual schema indices: 1
            required pub chapter_index: u16 => { bit: 0, key: "chapterIndex", when: (truthy(&true)), predicate_value: false },
            /// Chapter Offset
            // Virtual schema indices: 2
            required pub chapter_offset: u32 => { bit: 1, key: "chapterOffset", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required chapter_index: u16 => { bit: 0, key: "chapterIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required chapter_offset: u32 => { bit: 1, key: "chapterOffset", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_record! {
    strings = LingoString;
    pub struct RetPBTrackInfoChapterNames {
        fields {
            /// Chapter Index
            // Virtual schema indices: 1
            required pub chapter_index: u16 => { bit: 0, key: "chapterIndex", when: (truthy(&true)), predicate_value: false },
            /// Chapter Name
            // Virtual schema indices: 2
            required pub chapter_name: String => { bit: 1, key: "chapterName", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required chapter_index: u16 => { bit: 0, key: "chapterIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required chapter_name: String => { bit: 1, key: "chapterName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0043,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetPBTrackInfo {
        fields {
            /// Track Playback Index
            // Virtual schema indices: 1
            required pub track_playback_index: u32 => { bit: 0, key: "trackPlaybackIndex", when: (truthy(&true)), predicate_value: false },
            /// Track Information Type
            // Virtual schema indices: 2
            required pub track_information_type: RetPBTrackInfoTrackInformationType => { bit: 1, key: "trackInformationType", when: (truthy(&true)), predicate_value: true },
            /// Capabilities Bits
            // Virtual schema indices: 3
            optional pub capabilities_bits: RetPBTrackInfoCapabilitiesBits => { bit: 2, key: "capabilitiesBits", when: (eq(&track_information_type, &0u64)), predicate_value: false },
            /// Track Name
            // Virtual schema indices: 4
            optional pub track_name: String => { bit: 3, key: "trackName", when: (eq(&track_information_type, &1u64)), predicate_value: false },
            /// Artist Name
            // Virtual schema indices: 5
            optional pub artist_name: String => { bit: 4, key: "artistName", when: (eq(&track_information_type, &2u64)), predicate_value: false },
            /// Album Name
            // Virtual schema indices: 6
            optional pub album_name: String => { bit: 5, key: "albumName", when: (eq(&track_information_type, &3u64)), predicate_value: false },
            /// Genre Name
            // Virtual schema indices: 7
            optional pub genre_name: String => { bit: 6, key: "genreName", when: (eq(&track_information_type, &4u64)), predicate_value: false },
            /// Composer Name
            // Virtual schema indices: 8
            optional pub composer_name: String => { bit: 7, key: "composerName", when: (eq(&track_information_type, &5u64)), predicate_value: false },
            /// Total Track Duration
            // Virtual schema indices: 9
            optional pub total_track_duration: u32 => { bit: 8, key: "totalTrackDuration", when: (eq(&track_information_type, &6u64)), predicate_value: false },
            /// iTunes Unique Track ID
            // Virtual schema indices: 10
            optional pub i_tunes_track_uid: u64 => { bit: 9, key: "iTunesTrackUID", when: (eq(&track_information_type, &7u64)), predicate_value: false },
            /// Chapter Count
            // Virtual schema indices: 11
            optional pub chapter_count: u16 => { bit: 10, key: "chapterCount", when: (eq(&track_information_type, &8u64)), predicate_value: false },
            /// Chapter Times
            // Virtual schema indices: 12
            optional pub chapter_times: Vec<RetPBTrackInfoChapterTimes> => { bit: 11, key: "chapterTimes", when: (eq(&track_information_type, &9u64)), predicate_value: false },
            /// Chapter Names
            // Virtual schema indices: 13
            optional pub chapter_names: Vec<RetPBTrackInfoChapterNames> => { bit: 12, key: "chapterNames", when: (eq(&track_information_type, &10u64)), predicate_value: false },
            /// Current Track Lyrics Section Index
            // Virtual schema indices: 14
            optional pub current_track_lyrics_section_index: u16 => { bit: 13, key: "currentTrackLyricsSectionIndex", when: (eq(&track_information_type, &11u64)), predicate_value: false },
            /// Maximum Track Lyrics Section Index
            // Virtual schema indices: 15
            optional pub maximum_track_lyrics_section_index: u16 => { bit: 14, key: "maximumTrackLyricsSectionIndex", when: (eq(&track_information_type, &11u64)), predicate_value: false },
            /// Track Lyrics String Data
            // Virtual schema indices: 16
            optional pub track_lyrics_string_data: Vec<u8> => { bit: 15, key: "trackLyricsStringData", when: (eq(&track_information_type, &11u64)), predicate_value: false },
            /// Description
            // Virtual schema indices: 17
            optional pub track_description: String => { bit: 16, key: "trackDescription", when: (eq(&track_information_type, &12u64)), predicate_value: false },
            /// Album Track Index
            // Virtual schema indices: 18
            optional pub album_track_index: u16 => { bit: 17, key: "albumTrackIndex", when: (eq(&track_information_type, &13u64)), predicate_value: false },
            /// Disc Set Album Index
            // Virtual schema indices: 19
            optional pub disc_set_album_index: u16 => { bit: 18, key: "discSetAlbumIndex", when: (eq(&track_information_type, &14u64)), predicate_value: false },
            /// Play Count
            // Virtual schema indices: 20
            optional pub play_count: u32 => { bit: 19, key: "playCount", when: (eq(&track_information_type, &15u64)), predicate_value: false },
            /// Skip Count
            // Virtual schema indices: 21
            optional pub skip_count: u32 => { bit: 20, key: "skipCount", when: (eq(&track_information_type, &16u64)), predicate_value: false },
            /// Seconds
            // Virtual schema indices: 22, 28
            optional pub seconds: u8 => { bit: 21, key: "seconds", when: (eq(&track_information_type, &17u64)) || (eq(&track_information_type, &18u64)), predicate_value: false },
            /// Minute
            // Virtual schema indices: 23, 29
            optional pub minute: u8 => { bit: 22, key: "minute", when: (eq(&track_information_type, &17u64)) || (eq(&track_information_type, &18u64)), predicate_value: false },
            /// Hour
            // Virtual schema indices: 24, 30
            optional pub hour: u8 => { bit: 23, key: "hour", when: (eq(&track_information_type, &17u64)) || (eq(&track_information_type, &18u64)), predicate_value: false },
            /// Day
            // Virtual schema indices: 25, 31
            optional pub day: u8 => { bit: 24, key: "day", when: (eq(&track_information_type, &17u64)) || (eq(&track_information_type, &18u64)), predicate_value: false },
            /// Month
            // Virtual schema indices: 26, 32
            optional pub month: u8 => { bit: 25, key: "month", when: (eq(&track_information_type, &17u64)) || (eq(&track_information_type, &18u64)), predicate_value: false },
            /// Year
            // Virtual schema indices: 27, 33, 34
            optional pub year: u16 => { bit: 26, key: "year", when: (eq(&track_information_type, &17u64)) || (eq(&track_information_type, &18u64)) || (eq(&track_information_type, &19u64)), predicate_value: false },
            /// Star Rating
            // Virtual schema indices: 35
            optional pub star_rating: u8 => { bit: 27, key: "starRating", when: (eq(&track_information_type, &20u64)), predicate_value: false },
            /// Series Name
            // Virtual schema indices: 36
            optional pub series_name: u8 => { bit: 28, key: "seriesName", when: (eq(&track_information_type, &21u64)), predicate_value: false },
            /// Season Number
            // Virtual schema indices: 37
            optional pub season_number: u16 => { bit: 29, key: "seasonNumber", when: (eq(&track_information_type, &22u64)), predicate_value: false },
            /// Track Volume Adjust
            // Virtual schema indices: 38
            optional pub track_volume_adjust: u8 => { bit: 30, key: "trackVolumeAdjust", when: (eq(&track_information_type, &23u64)), predicate_value: false },
            /// Track EQ Preset Index
            // Virtual schema indices: 39
            optional pub track_eq_preset_index: u16 => { bit: 31, key: "trackEQPresetIndex", when: (eq(&track_information_type, &24u64)), predicate_value: false },
            /// Data Sample Rate
            // Virtual schema indices: 40
            optional pub data_rate: u32 => { bit: 32, key: "dataRate", when: (eq(&track_information_type, &25u64)), predicate_value: false },
            /// Bookmark Offset from Start
            // Virtual schema indices: 41
            optional pub bookmark_offset: u32 => { bit: 33, key: "bookmark offset", when: (eq(&track_information_type, &26u64)), predicate_value: false },
            /// Start Time Offset
            // Virtual schema indices: 42
            optional pub start_time_offset: u32 => { bit: 34, key: "startTimeOffset", when: (eq(&track_information_type, &27u64)), predicate_value: false },
            /// Stop Time Offset
            // Virtual schema indices: 43
            optional pub stop_time_offset: u32 => { bit: 35, key: "stopTimeOffset", when: (eq(&track_information_type, &27u64)), predicate_value: false },
        }
        steps {
            1 => required track_playback_index: u32 => { bit: 0, key: "trackPlaybackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required track_information_type: RetPBTrackInfoTrackInformationType => { bit: 1, key: "trackInformationType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => optional capabilities_bits: RetPBTrackInfoCapabilitiesBits => { bit: 2, key: "capabilitiesBits", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(0u8)] } },
            4 => optional track_name: String => { bit: 3, key: "trackName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&track_information_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(1u8)] } },
            5 => optional artist_name: String => { bit: 4, key: "artistName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&track_information_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(2u8)] } },
            6 => optional album_name: String => { bit: 5, key: "albumName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&track_information_type, &3u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(3u8)] } },
            7 => optional genre_name: String => { bit: 6, key: "genreName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&track_information_type, &4u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(4u8)] } },
            8 => optional composer_name: String => { bit: 7, key: "composerName", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&track_information_type, &5u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(5u8)] } },
            9 => optional total_track_duration: u32 => { bit: 8, key: "totalTrackDuration", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &6u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(6u8)] } },
            10 => optional i_tunes_track_uid: u64 => { bit: 9, key: "iTunesTrackUID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &7u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(7u8)] } },
            11 => optional chapter_count: u16 => { bit: 10, key: "chapterCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &8u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(8u8)] } },
            12 => optional chapter_times: Vec<RetPBTrackInfoChapterTimes> => { bit: 11, key: "chapterTimes", wire: [counted_remaining], project_wire: Iap1WireSpec::CountedRemaining, predicate_value: false, predicate: { expr: eq(&track_information_type, &9u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(9u8)] } },
            13 => optional chapter_names: Vec<RetPBTrackInfoChapterNames> => { bit: 12, key: "chapterNames", wire: [counted_remaining], project_wire: Iap1WireSpec::CountedRemaining, predicate_value: false, predicate: { expr: eq(&track_information_type, &10u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(10u8)] } },
            14 => optional current_track_lyrics_section_index: u16 => { bit: 13, key: "currentTrackLyricsSectionIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &11u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(11u8)] } },
            15 => optional maximum_track_lyrics_section_index: u16 => { bit: 14, key: "maximumTrackLyricsSectionIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &11u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(11u8)] } },
            16 => optional track_lyrics_string_data: Vec<u8> => { bit: 15, key: "trackLyricsStringData", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&track_information_type, &11u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(11u8)] } },
            17 => optional track_description: String => { bit: 16, key: "trackDescription", wire: [raw_string], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: eq(&track_information_type, &12u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(12u8)] } },
            18 => optional album_track_index: u16 => { bit: 17, key: "albumTrackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &13u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(13u8)] } },
            19 => optional disc_set_album_index: u16 => { bit: 18, key: "discSetAlbumIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &14u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(14u8)] } },
            20 => optional play_count: u32 => { bit: 19, key: "playCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &15u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(15u8)] } },
            21 => optional skip_count: u32 => { bit: 20, key: "skipCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &16u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(16u8)] } },
            22 => optional seconds: u8 => { bit: 21, key: "seconds", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(17u8)] } },
            23 => optional minute: u8 => { bit: 22, key: "minute", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(17u8)] } },
            24 => optional hour: u8 => { bit: 23, key: "hour", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(17u8)] } },
            25 => optional day: u8 => { bit: 24, key: "day", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(17u8)] } },
            26 => optional month: u8 => { bit: 25, key: "month", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(17u8)] } },
            27 => optional year: u16 => { bit: 26, key: "year", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &17u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(17u8)] } },
            28 => optional seconds: u8 => { bit: 21, key: "seconds", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &18u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(18u8)] } },
            29 => optional minute: u8 => { bit: 22, key: "minute", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &18u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(18u8)] } },
            30 => optional hour: u8 => { bit: 23, key: "hour", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &18u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(18u8)] } },
            31 => optional day: u8 => { bit: 24, key: "day", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &18u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(18u8)] } },
            32 => optional month: u8 => { bit: 25, key: "month", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &18u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(18u8)] } },
            33 => optional year: u16 => { bit: 26, key: "year", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &18u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(18u8)] } },
            34 => optional year: u16 => { bit: 26, key: "year", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &19u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(19u8)] } },
            35 => optional star_rating: u8 => { bit: 27, key: "starRating", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &20u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(20u8)] } },
            36 => optional series_name: u8 => { bit: 28, key: "seriesName", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &21u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(21u8)] } },
            37 => optional season_number: u16 => { bit: 29, key: "seasonNumber", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &22u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(22u8)] } },
            38 => optional track_volume_adjust: u8 => { bit: 30, key: "trackVolumeAdjust", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &23u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(23u8)] } },
            39 => optional track_eq_preset_index: u16 => { bit: 31, key: "trackEQPresetIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &24u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(24u8)] } },
            40 => optional data_rate: u32 => { bit: 32, key: "dataRate", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &25u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(25u8)] } },
            41 => optional bookmark_offset: u32 => { bit: 33, key: "bookmark offset", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &26u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(26u8)] } },
            42 => optional start_time_offset: u32 => { bit: 34, key: "startTimeOffset", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &27u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(27u8)] } },
            43 => optional stop_time_offset: u32 => { bit: 35, key: "stopTimeOffset", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_information_type, &27u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(1), FlatPredicateToken::Integer(27u8)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum CreateGeniusPlaylistIndexType: u8 {
        /// Database Engine
        DatabaseEngine = 0,
        /// Playback Engine
        PlaybackEngine = 1,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0044,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct CreateGeniusPlaylist {
        fields {
            /// Index Type
            // Virtual schema indices: 1
            required pub index_type: CreateGeniusPlaylistIndexType => { bit: 0, key: "indexType", when: (truthy(&true)), predicate_value: false },
            /// Track Index
            // Virtual schema indices: 2
            required pub track_index: u32 => { bit: 1, key: "trackIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required index_type: CreateGeniusPlaylistIndexType => { bit: 0, key: "indexType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required track_index: u32 => { bit: 1, key: "trackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0045,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RefreshGeniusPlaylist {
        fields {
            /// Playlist Index
            // Virtual schema indices: 1
            required pub playlist_index: u32 => { bit: 0, key: "playlistIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required playlist_index: u32 => { bit: 0, key: "playlistIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum IsGeniusAvailableForTrackIndexType: u8 {
        /// Database Engine
        DatabaseEngine = 0,
        /// Playback Engine
        PlaybackEngine = 1,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0047,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct IsGeniusAvailableForTrack {
        fields {
            /// Index Type
            // Virtual schema indices: 1
            required pub index_type: IsGeniusAvailableForTrackIndexType => { bit: 0, key: "indexType", when: (truthy(&true)), predicate_value: false },
            /// Track Index
            // Virtual schema indices: 2
            required pub track_index: u32 => { bit: 1, key: "trackIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required index_type: IsGeniusAvailableForTrackIndexType => { bit: 0, key: "indexType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required track_index: u32 => { bit: 1, key: "trackIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum GetPlaylistInfoInfoType: u8 {
        /// playlist information
        PlaylistInformation = 0,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0048,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetPlaylistInfo {
        fields {
            /// Info Type
            // Virtual schema indices: 1
            required pub info_type: GetPlaylistInfoInfoType => { bit: 0, key: "infoType", when: (truthy(&true)), predicate_value: false },
            /// DB Playlist Index
            // Virtual schema indices: 2
            required pub db_playlist_index: u32 => { bit: 1, key: "dbPlaylistIndex", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required info_type: GetPlaylistInfoInfoType => { bit: 0, key: "infoType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required db_playlist_index: u32 => { bit: 1, key: "dbPlaylistIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetPlaylistInfoInfoType: u8 {
        /// playlist information
        PlaylistInformation = 0,
    }
}

iap1_bitflags! {
    strings = LingoString;
    pub struct RetPlaylistInfoInfoTypeBits: u32 {
        /// is Genius playlist
        const IS_GENIUS_PLAYLIST = 1 << 0;
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x0049,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetPlaylistInfo {
        fields {
            /// Info Type
            // Virtual schema indices: 1
            required pub info_type: RetPlaylistInfoInfoType => { bit: 0, key: "infoType", when: (truthy(&true)), predicate_value: true },
            /// Info Type Bits
            // Virtual schema indices: 2
            optional pub info_type_bits: RetPlaylistInfoInfoTypeBits => { bit: 1, key: "infoTypeBits", when: (eq(&info_type, &0u64)), predicate_value: false },
        }
        steps {
            1 => required info_type: RetPlaylistInfoInfoType => { bit: 0, key: "infoType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional info_type_bits: RetPlaylistInfoInfoTypeBits => { bit: 1, key: "infoTypeBits", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&info_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
        }
    }
}

iap1_record! {
    strings = LingoString;
    pub struct PrepareUIDListTrackUIDs {
        fields {
            /// Track UID
            // Virtual schema indices: 1
            required pub track_uid: u64 => { bit: 0, key: "trackUID", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required track_uid: u64 => { bit: 0, key: "trackUID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x004a,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct PrepareUIDList {
        fields {
            /// Current Payload Section Index
            // Virtual schema indices: 1
            required pub sect_cur: u16 => { bit: 0, key: "sectCur", when: (truthy(&true)), predicate_value: false },
            /// Maximum Payload Section Index
            // Virtual schema indices: 2
            required pub sect_max: u16 => { bit: 1, key: "sectMax", when: (truthy(&true)), predicate_value: false },
            /// Track UIDs
            // Virtual schema indices: 3
            required pub track_ui_ds: Vec<PrepareUIDListTrackUIDs> => { bit: 2, key: "trackUIDs", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required sect_cur: u16 => { bit: 0, key: "sectCur", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required sect_max: u16 => { bit: 1, key: "sectMax", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => required track_ui_ds: Vec<PrepareUIDListTrackUIDs> => { bit: 2, key: "trackUIDs", wire: [counted_remaining], project_wire: Iap1WireSpec::CountedRemaining, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum PlayPreparedUIDListReserved: u8 {
        /// reserved
        Reserved = 0,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x004b,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct PlayPreparedUIDList {
        fields {
            /// Reserved
            // Virtual schema indices: 1
            required pub reserved: PlayPreparedUIDListReserved => { bit: 0, key: "reserved", when: (truthy(&true)), predicate_value: false },
            /// Track UID
            // Virtual schema indices: 2
            required pub track_uid: u64 => { bit: 1, key: "trackUID", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required reserved: PlayPreparedUIDListReserved => { bit: 0, key: "reserved", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required track_uid: u64 => { bit: 1, key: "trackUID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum GetArtworkTimesTrackIdentifierType: u8 {
        /// UID
        UID = 0,
        /// playback list index
        PlaybackListIndex = 1,
        /// database index
        DatabaseIndex = 2,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x004c,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetArtworkTimes {
        fields {
            /// Track Identifier Type
            // Virtual schema indices: 1
            required pub track_identifier_type: GetArtworkTimesTrackIdentifierType => { bit: 0, key: "trackIdentifierType", when: (truthy(&true)), predicate_value: true },
            /// Track Identifier (UID)
            // Virtual schema indices: 2
            optional pub track_identifier_uid: u64 => { bit: 1, key: "trackIdentifierUID", when: (eq(&track_identifier_type, &0u64)), predicate_value: false },
            /// Track Identifier (Playback List Index)
            // Virtual schema indices: 3
            optional pub track_identifier_playback_list: u32 => { bit: 2, key: "trackIdentifierPlaybackList", when: (eq(&track_identifier_type, &1u64)), predicate_value: false },
            /// Track Identifier (Database Index)
            // Virtual schema indices: 4
            optional pub track_identifier_database_index: u32 => { bit: 3, key: "trackIdentifierDatabaseIndex", when: (eq(&track_identifier_type, &2u64)), predicate_value: false },
            /// Format ID
            // Virtual schema indices: 5
            required pub format_id: u16 => { bit: 4, key: "formatID", when: (truthy(&true)), predicate_value: false },
            /// Artwork Index
            // Virtual schema indices: 6
            required pub artwork_index: u16 => { bit: 5, key: "artworkIndex", when: (truthy(&true)), predicate_value: false },
            /// Artwork Count
            // Virtual schema indices: 7
            required pub artwork_count: u16 => { bit: 6, key: "artworkCount", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required track_identifier_type: GetArtworkTimesTrackIdentifierType => { bit: 0, key: "trackIdentifierType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional track_identifier_uid: u64 => { bit: 1, key: "trackIdentifierUID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_identifier_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional track_identifier_playback_list: u32 => { bit: 2, key: "trackIdentifierPlaybackList", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_identifier_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            4 => optional track_identifier_database_index: u32 => { bit: 3, key: "trackIdentifierDatabaseIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_identifier_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            5 => required format_id: u16 => { bit: 4, key: "formatID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            6 => required artwork_index: u16 => { bit: 5, key: "artworkIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            7 => required artwork_count: u16 => { bit: 6, key: "artworkCount", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetArtworkTimesTrackIdentifierType: u8 {
        /// UID
        UID = 0,
        /// playback list index
        PlaybackListIndex = 1,
        /// database index
        DatabaseIndex = 2,
    }
}

iap1_record! {
    strings = LingoString;
    pub struct RetArtworkTimesTimeOffsets {
        fields {
            /// Time Offset
            // Virtual schema indices: 1
            required pub time_offset: u32 => { bit: 0, key: "timeOffset", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required time_offset: u32 => { bit: 0, key: "timeOffset", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x004d,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetArtworkTimes {
        fields {
            /// Track Identifier Type
            // Virtual schema indices: 1
            required pub track_identifier_type: RetArtworkTimesTrackIdentifierType => { bit: 0, key: "trackIdentifierType", when: (truthy(&true)), predicate_value: true },
            /// Track Identifier (UID)
            // Virtual schema indices: 2
            optional pub track_identifier_uid: u64 => { bit: 1, key: "trackIdentifierUID", when: (eq(&track_identifier_type, &0u64)), predicate_value: false },
            /// Track Identifier (Playback List Index)
            // Virtual schema indices: 3
            optional pub track_identifier_playback_list: u32 => { bit: 2, key: "trackIdentifierPlaybackList", when: (eq(&track_identifier_type, &1u64)), predicate_value: false },
            /// Track Identifier (Database Index)
            // Virtual schema indices: 4
            optional pub track_identifier_database_index: u32 => { bit: 3, key: "trackIdentifierDatabaseIndex", when: (eq(&track_identifier_type, &2u64)), predicate_value: false },
            /// Time Offsets
            // Virtual schema indices: 5
            required pub time_offsets: Vec<RetArtworkTimesTimeOffsets> => { bit: 4, key: "timeOffsets", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required track_identifier_type: RetArtworkTimesTrackIdentifierType => { bit: 0, key: "trackIdentifierType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional track_identifier_uid: u64 => { bit: 1, key: "trackIdentifierUID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_identifier_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional track_identifier_playback_list: u32 => { bit: 2, key: "trackIdentifierPlaybackList", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_identifier_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            4 => optional track_identifier_database_index: u32 => { bit: 3, key: "trackIdentifierDatabaseIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_identifier_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            5 => required time_offsets: Vec<RetArtworkTimesTimeOffsets> => { bit: 4, key: "timeOffsets", wire: [counted_remaining], project_wire: Iap1WireSpec::CountedRemaining, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum GetArtworkDataTrackIdentifierType: u8 {
        /// UID
        UID = 0,
        /// playback list index
        PlaybackListIndex = 1,
        /// database index
        DatabaseIndex = 2,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x004e,
        source = accessory,
        response = false,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct GetArtworkData {
        fields {
            /// Track Identifier Type
            // Virtual schema indices: 1
            required pub track_identifier_type: GetArtworkDataTrackIdentifierType => { bit: 0, key: "trackIdentifierType", when: (truthy(&true)), predicate_value: true },
            /// Track Identifier (UID)
            // Virtual schema indices: 2
            optional pub track_identifier_uid: u64 => { bit: 1, key: "trackIdentifierUID", when: (eq(&track_identifier_type, &0u64)), predicate_value: false },
            /// Track Identifier (Playback List Index)
            // Virtual schema indices: 3
            optional pub track_identifier_playback_list: u32 => { bit: 2, key: "trackIdentifierPlaybackList", when: (eq(&track_identifier_type, &1u64)), predicate_value: false },
            /// Track Identifier (Database Index)
            // Virtual schema indices: 4
            optional pub track_identifier_database_index: u32 => { bit: 3, key: "trackIdentifierDatabaseIndex", when: (eq(&track_identifier_type, &2u64)), predicate_value: false },
            /// Format ID
            // Virtual schema indices: 5
            required pub format_id: u16 => { bit: 4, key: "formatID", when: (truthy(&true)), predicate_value: false },
            /// Time Offset
            // Virtual schema indices: 6
            required pub time_offset: u32 => { bit: 5, key: "timeOffset", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required track_identifier_type: GetArtworkDataTrackIdentifierType => { bit: 0, key: "trackIdentifierType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => optional track_identifier_uid: u64 => { bit: 1, key: "trackIdentifierUID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_identifier_type, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            3 => optional track_identifier_playback_list: u32 => { bit: 2, key: "trackIdentifierPlaybackList", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_identifier_type, &1u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(1u8)] } },
            4 => optional track_identifier_database_index: u32 => { bit: 3, key: "trackIdentifierDatabaseIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&track_identifier_type, &2u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(2u8)] } },
            5 => required format_id: u16 => { bit: 4, key: "formatID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            6 => required time_offset: u32 => { bit: 5, key: "timeOffset", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetArtworkDataTrackIdentifierType: u8 {
        /// UID
        UID = 0,
        /// playback list index
        PlaybackListIndex = 1,
        /// database index
        DatabaseIndex = 2,
    }
}

iap1_enum! {
    strings = LingoString;
    pub enum RetArtworkDataPixelFormat: u8 {
        /// monochrome, 2 bits per pixel
        Monochrome2BitsPerPixel = 1,
        /// RGB 565 color, little-endian, 16 bpp
        RGB565ColorLittleEndian16Bpp = 2,
        /// RGB 565 color, big-endian, 16 bpp
        RGB565ColorBigEndian16Bpp = 3,
    }
}

iap1! {
    #[iap1(
        context = ctx, strings = LingoString, lingo = 0x04, command = 0x004f,
        source = device,
        response = true,
        ack = false,
        deprecated = false,
        transaction_id = permitted
    )]
    pub struct RetArtworkData {
        fields {
            /// Current Payload Section Index
            // Virtual schema indices: 1
            required pub sect_cur: u16 => { bit: 0, key: "sectCur", when: (truthy(&true)), predicate_value: true },
            /// Maximum Payload Section Index
            // Virtual schema indices: 2
            required pub sect_max: u16 => { bit: 1, key: "sectMax", when: (truthy(&true)), predicate_value: false },
            /// Track Identifier Type
            // Virtual schema indices: 3
            optional pub track_identifier_type: RetArtworkDataTrackIdentifierType => { bit: 2, key: "trackIdentifierType", when: (eq(&sect_cur, &0u64)), predicate_value: true },
            /// Track Identifier (UID)
            // Virtual schema indices: 4
            optional pub track_identifier_uid: u64 => { bit: 3, key: "trackIdentifierUID", when: ((eq(&sect_cur, &0u64)) && (eq(&track_identifier_type, &0u64))), predicate_value: false },
            /// Track Identifier (Playback List Index)
            // Virtual schema indices: 5
            optional pub track_identifier_playback_list: u32 => { bit: 4, key: "trackIdentifierPlaybackList", when: ((eq(&sect_cur, &0u64)) && (eq(&track_identifier_type, &1u64))), predicate_value: false },
            /// Track Identifier (Database Index)
            // Virtual schema indices: 6
            optional pub track_identifier_database_index: u32 => { bit: 5, key: "trackIdentifierDatabaseIndex", when: ((eq(&sect_cur, &0u64)) && (eq(&track_identifier_type, &2u64))), predicate_value: false },
            /// Display Pixel Format
            // Virtual schema indices: 7
            optional pub pixel_format: RetArtworkDataPixelFormat => { bit: 6, key: "pixelFormat", when: (eq(&sect_cur, &0u64)), predicate_value: false },
            /// Image Width
            // Virtual schema indices: 8
            optional pub image_width: u16 => { bit: 7, key: "imageWidth", when: (eq(&sect_cur, &0u64)), predicate_value: false },
            /// Inset Rectangle, Top-Left Point, X Value
            // Virtual schema indices: 9
            optional pub x_value_of_top_left_point_of_inset_rectangle: u16 => { bit: 8, key: "xValueOfTopLeftPointOfInsetRectangle", when: (eq(&sect_cur, &0u64)), predicate_value: false },
            /// Inset Rectangle, Top-Left Point, Y Value
            // Virtual schema indices: 10
            optional pub y_value_of_top_left_point_of_inset_rectangle: u16 => { bit: 9, key: "yValueOfTopLeftPointOfInsetRectangle", when: (eq(&sect_cur, &0u64)), predicate_value: false },
            /// Inset Rectangle, Bottom-Right Point, X Value
            // Virtual schema indices: 11
            optional pub x_value_of_bottom_right_point_of_inset_rectangle: u16 => { bit: 10, key: "xValueOfBottomRightPointOfInsetRectangle", when: (eq(&sect_cur, &0u64)), predicate_value: false },
            /// Inset Rectangle, Bottom-Right Point, Y Value
            // Virtual schema indices: 12
            optional pub y_value_of_bottom_right_point_of_inset_rectangle: u16 => { bit: 11, key: "yValueOfBottomRightPointOfInsetRectangle", when: (eq(&sect_cur, &0u64)), predicate_value: false },
            /// Row Size
            // Virtual schema indices: 13
            optional pub row_size: u32 => { bit: 12, key: "rowSize", when: (eq(&sect_cur, &0u64)), predicate_value: false },
            /// Display Image Pixel Data
            // Virtual schema indices: 14
            required pub display_image_pixel_data: Vec<u8> => { bit: 13, key: "displayImagePixelData", when: (truthy(&true)), predicate_value: false },
        }
        steps {
            1 => required sect_cur: u16 => { bit: 0, key: "sectCur", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            2 => required sect_max: u16 => { bit: 1, key: "sectMax", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
            3 => optional track_identifier_type: RetArtworkDataTrackIdentifierType => { bit: 2, key: "trackIdentifierType", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: true, predicate: { expr: eq(&sect_cur, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            4 => optional track_identifier_uid: u64 => { bit: 3, key: "trackIdentifierUID", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&sect_cur, &0u64)) && (eq(&track_identifier_type, &0u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(2), FlatPredicateToken::Integer(0u8)] } },
            5 => optional track_identifier_playback_list: u32 => { bit: 4, key: "trackIdentifierPlaybackList", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&sect_cur, &0u64)) && (eq(&track_identifier_type, &1u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(2), FlatPredicateToken::Integer(1u8)] } },
            6 => optional track_identifier_database_index: u32 => { bit: 5, key: "trackIdentifierDatabaseIndex", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: (eq(&sect_cur, &0u64)) && (eq(&track_identifier_type, &2u64)), program: [FlatPredicateToken::And, FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8), FlatPredicateToken::Eq, FlatPredicateToken::Field(2), FlatPredicateToken::Integer(2u8)] } },
            7 => optional pixel_format: RetArtworkDataPixelFormat => { bit: 6, key: "pixelFormat", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&sect_cur, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            8 => optional image_width: u16 => { bit: 7, key: "imageWidth", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&sect_cur, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            9 => optional x_value_of_top_left_point_of_inset_rectangle: u16 => { bit: 8, key: "xValueOfTopLeftPointOfInsetRectangle", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&sect_cur, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            10 => optional y_value_of_top_left_point_of_inset_rectangle: u16 => { bit: 9, key: "yValueOfTopLeftPointOfInsetRectangle", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&sect_cur, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            11 => optional x_value_of_bottom_right_point_of_inset_rectangle: u16 => { bit: 10, key: "xValueOfBottomRightPointOfInsetRectangle", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&sect_cur, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            12 => optional y_value_of_bottom_right_point_of_inset_rectangle: u16 => { bit: 11, key: "yValueOfBottomRightPointOfInsetRectangle", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&sect_cur, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            13 => optional row_size: u32 => { bit: 12, key: "rowSize", wire: [scalar], project_wire: Iap1WireSpec::Scalar, predicate_value: false, predicate: { expr: eq(&sect_cur, &0u64), program: [FlatPredicateToken::Eq, FlatPredicateToken::Field(0), FlatPredicateToken::Integer(0u8)] } },
            14 => required display_image_pixel_data: Vec<u8> => { bit: 13, key: "displayImagePixelData", wire: [raw_bytes], project_wire: Iap1WireSpec::Raw, predicate_value: false, predicate: { expr: truthy(&true), program: [FlatPredicateToken::Bool(true)] } },
        }
    }
}

iap1_registry! {
    0x04;
    IPodAck => 0x0001,
    GetCurrentPlayingTrackChapterInfo => 0x0002,
    ReturnCurrentPlayingTrackChapterInfo => 0x0003,
    SetCurrentPlayingTrackChapter => 0x0004,
    GetCurrentPlayingTrackChapterPlayStatus => 0x0005,
    ReturnCurrentPlayingTrackChapterPlayStatus => 0x0006,
    GetCurrentPlayingTrackChapterName => 0x0007,
    ReturnCurrentPlayingTrackChapterName => 0x0008,
    GetAudiobookSpeed => 0x0009,
    ReturnAudiobookSpeed => 0x000a,
    SetAudiobookSpeed => 0x000b,
    GetIndexedPlayingTrackInfo => 0x000c,
    ReturnIndexedPlayingTrackInfo => 0x000d,
    GetArtworkFormats => 0x000e,
    RetArtworkFormats => 0x000f,
    GetTrackArtworkData => 0x0010,
    RetTrackArtworkData => 0x0011,
    RequestProtocolVersion => 0x0012,
    ReturnProtocolVersion => 0x0013,
    RequestiPodName => 0x0014,
    ReturniPodName => 0x0015,
    ResetDBSelection => 0x0016,
    SelectDBRecord => 0x0017,
    GetNumberCategorizedDBRecords => 0x0018,
    ReturnNumberCategorizedDBRecords => 0x0019,
    RetrieveCategorizedDatabaseRecords => 0x001a,
    ReturnCategorizedDatabaseRecord => 0x001b,
    GetPlayStatus => 0x001c,
    ReturnPlayStatus => 0x001d,
    GetCurrentPlayingTrackIndex => 0x001e,
    ReturnCurrentPlayingTrackIndex => 0x001f,
    GetIndexedPlayingTrackTitle => 0x0020,
    ReturnIndexedPlayingTrackTitle => 0x0021,
    GetIndexedPlayingTrackArtistName => 0x0022,
    ReturnIndexedPlayingTrackArtistName => 0x0023,
    GetIndexedPlayingTrackAlbumName => 0x0024,
    ReturnIndexedPlayingTrackAlbumName => 0x0025,
    SetPlayStatusChangeNotification => 0x0026,
    PlayStatusChangeNotification => 0x0027,
    PlayCurrentSelection => 0x0028,
    PlayControl => 0x0029,
    GetTrackArtworkTimes => 0x002a,
    RetTrackArtworkTimes => 0x002b,
    GetShuffle => 0x002c,
    ReturnShuffle => 0x002d,
    SetShuffle => 0x002e,
    GetRepeat => 0x002f,
    ReturnRepeat => 0x0030,
    SetRepeat => 0x0031,
    SetDisplayImage => 0x0032,
    GetMonoDisplayImageLimits => 0x0033,
    ReturnMonoDisplayImageLimits => 0x0034,
    GetNumPlayingTracks => 0x0035,
    ReturnNumPlayingTracks => 0x0036,
    SetCurrentPlayingTrack => 0x0037,
    SelectSortDBRecord => 0x0038,
    GetColorDisplayImageLimits => 0x0039,
    ReturnColorDisplayImageLimits => 0x003a,
    ResetDBSelectionHierarchy => 0x003b,
    GetDBiTunesInfo => 0x003c,
    RetDBiTunesInfo => 0x003d,
    GetUIDTrackInfo => 0x003e,
    RetUIDTrackInfo => 0x003f,
    GetDBTrackInfo => 0x0040,
    RetDBTrackInfo => 0x0041,
    GetPBTrackInfo => 0x0042,
    RetPBTrackInfo => 0x0043,
    CreateGeniusPlaylist => 0x0044,
    RefreshGeniusPlaylist => 0x0045,
    IsGeniusAvailableForTrack => 0x0047,
    GetPlaylistInfo => 0x0048,
    RetPlaylistInfo => 0x0049,
    PrepareUIDList => 0x004a,
    PlayPreparedUIDList => 0x004b,
    GetArtworkTimes => 0x004c,
    RetArtworkTimes => 0x004d,
    GetArtworkData => 0x004e,
    RetArtworkData => 0x004f,
}
