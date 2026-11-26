use catplay_plist::{plist_bitflags, plist_enum, plist_enum_repr};

plist_bitflags! {
    pub struct DisplayFeature: u8 {
        /// Supports interacting via knobs.
        const KNOBS     = 1 << 1;
        /// Supports interacting via low-fidelity touch.
        const LOWFI_TOUCH     = 1 << 2;
        /// Supports interacting via high-fidelity touch.
        const HIGHFI_TOUCH     = 1 << 3;
        /// Supports interacting via touchpad.
        const TOUCHPAD     = 1 << 4;
    }
}

plist_enum_repr! {
    #[repr(u8)]
    pub enum PrimaryInputDevice {
        /// Legacy head units report zero without a known input type.
        Unknown = 0,
        /// Accessory uses touchscreen as primary input.
        #[default]
        TouchScreen = 1,
        /// Accessory uses touchpad as primary input.
        Touchpad = 2,
        /// Accessory uses knob as primary input.
        Knob = 3,
    }
}

plist_enum! {
    pub enum ExtendedFeature {
        /// Specifies that the accessory supports enhanced Car UI requests (such as AC_BACK handling)
        #[serde(rename = "enhancedRequestCarUI")]
        EnhancedRequestCarUI,
        #[serde(rename = "vocoderInfo")]
        VocoderInfo,
        /// Specifies that the accessory supports accurate audio timestamps
        #[serde(rename = "ProvidesHighAccuracyTimeStamps")]
        HighAccuracyTimeStamps,

        #[serde(other)]
        #[default]
        Invalid,
    }
}

plist_enum! {
    pub enum ControllerFeature {
        UiContext,
        ViewAreas,
        CornerMasks,
        FocusTransfer,
        #[serde(rename = "h.264Level5.1")]
        H264Level51,
        MainBuffered,
        AltScreen,
        EnhancedSiri,
        Hevc,
        SessionManagement,
        LogTransfer,
        #[serde(rename = "iAPChannel")]
        IApChannel,

        VehicleStateProtocol,
        VideoPlayback,

        #[serde(other)]
        #[default]
        Invalid,
    }
}

plist_enum! {
    pub enum LimitedUIElement {
        SoftKeyboard,
        SoftPhoneKeypad,
        NonMusicLists,
        MusicLists,
        JapanMaps,

        #[serde(other)]
        #[default]
        Invalid,
    }
}
