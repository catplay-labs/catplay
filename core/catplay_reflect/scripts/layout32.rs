// Compile this as a no_std rlib with rustc, not as a standalone executable:
// cargo build -p catplay_reflect --target armv7-unknown-linux-gnueabihf --lib
// rustc --edition=2021 --crate-type=rlib --target armv7-unknown-linux-gnueabihf \
//   --extern catplay_reflect=target/armv7-unknown-linux-gnueabihf/debug/libcatplay_reflect.rlib \
//   -L dependency=target/armv7-unknown-linux-gnueabihf/debug/deps \
//   core/catplay_reflect/scripts/layout32.rs -o /tmp/catplay_reflect_layout32.rlib
#![no_std]

use catplay_reflect::{StaticHandle, static_objects, static_strings};
use core::mem::size_of;

static_strings! { pub struct Name8(u8) { SCREEN = "screen", AUDIO = "audio" } }
static_strings! { pub struct Name16(u16) { SCREEN = "screen", AUDIO = "audio" } }
static_strings! { pub struct DefaultName { DEFAULT = "default" } }

pub struct Config {
    pub code: u16,
    pub name: Name8,
}

static_objects! {
    pub struct ConfigRef8(u8): Config {
        SCREEN = Config { code: 1, name: Name8::SCREEN },
        AUDIO = Config { code: 2, name: Name8::AUDIO },
    }
}

static_objects! {
    pub struct ConfigRef16(u16): Config {
        SCREEN = Config { code: 101, name: Name8::SCREEN },
        AUDIO = Config { code: 102, name: Name8::AUDIO },
    }
}

static_objects! {
    pub struct DefaultConfigRef: Config {
        DEFAULT = Config { code: 201, name: Name8::SCREEN },
    }
}

#[repr(transparent)]
pub struct Row<H: StaticHandle>(pub H);

const _: () = {
    assert!(size_of::<&str>() == 8);
    assert!(size_of::<&&str>() == 4);
    assert!(size_of::<&Config>() == 4);
    assert!(size_of::<Name8>() == 1);
    assert!(size_of::<Name16>() == 2);
    assert!(size_of::<Option<Name8>>() == 1);
    assert!(size_of::<Option<Name16>>() == 2);
    assert!(size_of::<ConfigRef8>() == 1);
    assert!(size_of::<ConfigRef16>() == 2);
    assert!(size_of::<Option<ConfigRef8>>() == 1);
    assert!(size_of::<Option<ConfigRef16>>() == 2);
    assert!(size_of::<DefaultName>() == 2);
    assert!(size_of::<DefaultConfigRef>() == 2);
    assert!(size_of::<Row<Name8>>() == 1);
    assert!(size_of::<Row<Name16>>() == 2);
    assert!(size_of::<Row<ConfigRef8>>() == 1);
    assert!(size_of::<Row<ConfigRef16>>() == 2);
    assert!(size_of::<[Row<Name8>; 7]>() == 7);
    assert!(size_of::<[Row<Name16>; 7]>() == 14);

    // u8 indexes an offset table; u16 directly stores a nonzero byte offset.
    assert!(Name8::AUDIO.to_raw() == 2);
    assert!(Name8::AUDIO.offset() == 7);
    assert!(Name8::index_bytes() == 4);
    assert!(Name8::storage_bytes() == 13);
    assert!(Name16::SCREEN.to_raw() == 1);
    assert!(Name16::AUDIO.to_raw() == 8);
    assert!(Name16::AUDIO.offset() == 8);
    assert!(Name16::AUDIO.index() == 1);
    assert!(Name16::from_raw(2).is_none());
    assert!(Name16::from_raw(8).is_some());
    assert!(Name16::index_bytes() == 0);
    assert!(Name16::storage_bytes() == 14);
    assert!(Name16::total_storage_bytes() == 14);
    assert!(DefaultName::DEFAULT.offset() == 1);
    assert!(DefaultName::index_bytes() == 0);
};

pub static NAMES8: [Row<Name8>; 2] = [Row(Name8::SCREEN), Row(Name8::AUDIO)];
pub static NAMES16: [Row<Name16>; 2] = [Row(Name16::SCREEN), Row(Name16::AUDIO)];
pub static CONFIGS8: [Row<ConfigRef8>; 2] = [Row(ConfigRef8::SCREEN), Row(ConfigRef8::AUDIO)];
pub static CONFIGS16: [Row<ConfigRef16>; 2] = [Row(ConfigRef16::SCREEN), Row(ConfigRef16::AUDIO)];

pub fn resolve<H: StaticHandle>(handle: H) -> &'static H::Target {
    handle.get()
}

fn restore<H: StaticHandle>(raw: H::Raw) -> Option<&'static H::Target> {
    H::from_raw(raw).map(resolve::<H>)
}

pub fn name8(raw: u8) -> Option<&'static str> {
    restore::<Name8>(raw)
}
pub fn name16(raw: u16) -> Option<&'static str> {
    restore::<Name16>(raw)
}
pub fn config8(raw: u8) -> Option<&'static Config> {
    restore::<ConfigRef8>(raw)
}
pub fn config16(raw: u16) -> Option<&'static Config> {
    restore::<ConfigRef16>(raw)
}
