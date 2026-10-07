use crate::macros::iap1_global_registry;

pub mod strings;

iap1_global_registry! {
    strings = strings::LingoString;
    General(0x0, lingo_0x0),
    Microphone(0x1, lingo_0x1),
    SimpleRemote(0x2, lingo_0x2),
    DisplayRemote(0x3, lingo_0x3),
    ExtendedInterface(0x4, lingo_0x4),
    AccessoryPower(0x5, lingo_0x5),
    USBHostMode(0x6, lingo_0x6),
    RFTuner(0x7, lingo_0x7),
    AccessoryEqualizer(0x8, lingo_0x8),
    Sports(0x9, lingo_0x9),
    DigitalAudio(0xa, lingo_0xa),
    Test(0xb, lingo_0xb),
    Storage(0xc, lingo_0xc),
    IPodOut(0xd, lingo_0xd),
    Location(0xe, lingo_0xe),
}
