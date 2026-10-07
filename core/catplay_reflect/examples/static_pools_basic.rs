use catplay_reflect::{StaticHandle, static_objects, static_strings};
use core::mem::size_of;

static_strings! {
    /// A one-byte name used by protocol metadata.
    pub struct Name(u8) {
        SCREEN = "screen",
        AUDIO = "audio",
        MICROSECONDS = "μs",
        ALSO_AUDIO = "audio",
        EMPTY = "",
    }
}

static_strings! {
    /// The same contents addressed by direct two-byte offsets.
    pub struct WideName(u16) {
        SCREEN = "screen",
        AUDIO = "audio",
        MICROSECONDS = "μs",
        ALSO_AUDIO = "audio",
        EMPTY = "",
    }
}

// The objects do not need Copy, Clone, Debug, Eq or Hash.
pub struct Command {
    pub opcode: u16,
    pub name: Name,
    pub timeout_ms: u16,
}

static_objects! {
    /// A one-byte reference to a statically declared command.
    pub struct CommandRef(u8): Command {
        START_SCREEN = Command { opcode: 1, name: Name::SCREEN, timeout_ms: 100 },
        START_AUDIO = Command { opcode: 2, name: Name::AUDIO, timeout_ms: 32 },
    }
}

static_objects! {
    /// The same target type in an independent pool with two-byte handles.
    pub struct FallbackCommandRef(u16): Command {
        START_SCREEN = Command { opcode: 101, name: Name::SCREEN, timeout_ms: 500 },
    }
}

// H identifies the pool at compile time. No base pointer or resolver is stored
// in a row, and the transparent wrapper has the exact size of its handle.
#[repr(transparent)]
#[derive(Clone, Copy)]
struct Row<H: StaticHandle> {
    handle: H,
}

static COMMANDS: [Row<CommandRef>; 3] = [
    Row {
        handle: CommandRef::START_SCREEN,
    },
    Row {
        handle: CommandRef::START_AUDIO,
    },
    Row {
        handle: CommandRef::START_AUDIO,
    },
];

static FALLBACK_COMMANDS: [Row<FallbackCommandRef>; 1] = [Row {
    handle: FallbackCommandRef::START_SCREEN,
}];

// The terminator scan for this reference runs during const evaluation.
const AUDIO_NAME: &str = Name::AUDIO.as_str();
const FOUND_AUDIO: Option<Name> = Name::lookup("audio");
const AUDIO_COMMAND: &Command = CommandRef::START_AUDIO.get();

// One function works with both unsized str and sized Command targets.
fn resolve<H: StaticHandle>(handle: H) -> &'static H::Target {
    handle.get()
}

// Monomorphization selects the correct pool for each H, even when the target
// type is the same. Both one-byte and two-byte command handles work here.
fn print_commands<H: StaticHandle<Target = Command>>(rows: &[Row<H>]) {
    for row in rows {
        let command = resolve(row.handle);
        println!("{}: opcode={}, timeout={}ms", command.name, command.opcode, command.timeout_ms);
    }
}

fn restore<H: StaticHandle>(raw: H::Raw) -> Option<&'static H::Target> {
    H::from_raw(raw).map(resolve::<H>)
}

fn main() {
    assert_eq!(Name::AUDIO, Name::ALSO_AUDIO);
    assert_eq!(FOUND_AUDIO, Some(Name::AUDIO));
    assert_eq!(Name::AUDIO.to_raw(), 2);
    assert_eq!(Name::AUDIO.offset(), 7); // Immediately after the "screen\0" record.
    assert_eq!(WideName::AUDIO.to_raw(), 8); // Prefix byte + "screen\0".
    assert_eq!(WideName::AUDIO.offset(), WideName::AUDIO.to_raw());
    assert_eq!(WideName::AUDIO.index(), 1); // Ordinal lookup scans preceding records.
    assert_eq!(WideName::AUDIO, WideName::ALSO_AUDIO);
    assert_eq!(WideName::index_bytes(), 0);
    assert_eq!(size_of::<Name>(), 1);
    assert_eq!(size_of::<Option<Name>>(), 1);
    assert_eq!(size_of::<WideName>(), 2);
    assert_eq!(size_of::<Option<WideName>>(), 2);
    assert_eq!(size_of::<Row<CommandRef>>(), 1);
    assert_eq!(size_of::<Row<FallbackCommandRef>>(), 2);
    assert_eq!(size_of::<Option<CommandRef>>(), 1);
    assert_eq!(size_of::<Option<FallbackCommandRef>>(), 2);

    println!(
        "Name: {} B, Option<Name>: {} B; WideName: {} B",
        size_of::<Name>(),
        size_of::<Option<Name>>(),
        size_of::<WideName>()
    );
    println!(
        "u8 strings: {} record bytes + {} index bytes = {} bytes; {} unique strings",
        Name::storage_bytes(),
        Name::index_bytes(),
        Name::total_storage_bytes(),
        Name::unique_len()
    );
    println!(
        "u16 strings: {} bytes including the prefix; {} index bytes",
        WideName::storage_bytes(),
        WideName::index_bytes()
    );
    println!(
        "Generic rows: {} B / {} B per row for the same Command target",
        size_of::<Row<CommandRef>>(),
        size_of::<Row<FallbackCommandRef>>()
    );
    println!("const references: {AUDIO_NAME}, opcode={}", AUDIO_COMMAND.opcode);
    println!(
        "generic string access: {} / {}; raw ID / offset: {} / {}",
        resolve(Name::AUDIO),
        resolve(WideName::AUDIO),
        Name::AUDIO.to_raw(),
        WideName::AUDIO.to_raw()
    );
    assert_eq!(restore::<Name>(Name::AUDIO.to_raw()), Some("audio"));
    assert_eq!(restore::<WideName>(WideName::AUDIO.to_raw()), Some("audio"));
    assert_eq!(restore::<Name>(Name::EMPTY.to_raw()), Some(""));
    assert_eq!(restore::<WideName>(WideName::EMPTY.to_raw()), Some(""));
    assert_eq!(restore::<Name>(0), None);
    assert_eq!(restore::<WideName>(0), None);
    assert_eq!(restore::<CommandRef>(1).unwrap().opcode, 1);
    assert_eq!(restore::<FallbackCommandRef>(1).unwrap().opcode, 101);
    print_commands(&COMMANDS);
    print_commands(&FALLBACK_COMMANDS);
}
