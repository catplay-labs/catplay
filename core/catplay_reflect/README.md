# `catplay_reflect`

Small `no_std` support crate for declarative, typed-at-construction field
projection tables. It uses `macro_rules!` and does not require derive or proc
macros.

`field_project!` owns the struct declaration and emits one independent
`FieldProject<P, C>` table per `project` block. A table entry gets its concrete
field type from the generated mutable-field accessor and must satisfy the
selected projector's typed capability implementation.

Each table also creates a `static_objects!` pool for its contexts.
`FieldProjector::ContextPoolWidth` selects the raw handle width (`u16` by default,
or `u8` with `pool = u8`). Each table entry stores a pool-local raw handle.
`FieldProjector::ContextPoolObject` names the pool element type: use `C` and
pass context values for direct storage, or use `&'static C` and pass
`&Context { ... }` expressions to share identical static contexts across
tables. The generated pool preserves the supplied object representation.
Direct storage supports context initializers that cannot be statically
borrowed.
`FieldProjector::FieldOffsetWidth` selects `u32` or `u16` storage for the
projected field's byte offset. Use `u32` for large structs; `u16` rejects a
projected field whose offset exceeds 65,535 bytes. Existing projectors use
`u32`.
`HandleShared<Object, Raw>` selects a typed `static_objects!` pool in
its `new::<Pool>()` constructor and exposes a common runtime type across
different pools. `FieldTable` keeps its entries separately and stores this
pool descriptor by value. Its length uses the same `u8` or `u16` width as the
handle; `FieldOp::context()`
resolves the handle through it. Repeated field names are allowed in a
table: entries are numbered internally (`C1`, `C2`, ...).
The current `macro_rules!` counter supports up to 128 entries per table.

For `Option<T>` fields, a table entry may use `field: context => optional` or
`=> length_optional`. This selects `ProjectorOptionalCapability<T, C>` so a
projector can erase the option container at runtime while retaining a checked
`T` adapter. The callback receives a `FieldAccess` mode for shared versus
exclusive access. In shared mode it must not mutate or create mutable
references, so a read-only table pass never fabricates `&mut` from `&T`.

The capability is a local adapter type and can express a trait requirement on
the field type, for example `impl<F: CsmEncode> ProjectorCapability<F, u16> for
CsmEncodeCapability`. A different projector can select a different capability
and therefore a different bound for the same struct field. This arrangement
also permits generic capability implementations from downstream crates under
Rust's orphan rules.

The [Packet example](examples/packet.rs) shows the requested two-projector
syntax and a shared mock CSM encode implementation.

`FieldOp::apply` is unsafe because its caller must use the exact struct layout
used to generate the table. `FieldOp::apply_ref` is also unsafe and is only
valid when that projector's callback reads the field without mutating it. Do
not use the current implementation with `repr(packed)` structs.

## Static pools

The crate also exports `static_strings!`, `static_objects!`, and `StaticHandle`.
The macros build `no_std` pools at compile time: there is no allocation or
startup registration. Each pool has its own handle type, so safe code cannot
accidentally use a handle from one pool to read another. A handle is one byte
with `(u8)` or two bytes with `(u16)`; `Option<Handle>` has the same size.
Omitting the width selects `u16`.

### String pools

```rust
use catplay_reflect::static_strings;

static_strings! {
    pub struct FieldName(u8) {
        SCREEN = "screen",
        AUDIO = "audio",
        ALSO_AUDIO = concat!("au", "dio"),
        EMPTY = "",
    }
}

const AUDIO: FieldName = FieldName::require("audio");
const AUDIO_TEXT: &str = FieldName::AUDIO.as_str();

fn main() {
    assert_eq!(AUDIO, FieldName::AUDIO);
    assert_eq!(FieldName::AUDIO, FieldName::ALSO_AUDIO);
    assert_eq!(FieldName::lookup("audio"), Some(FieldName::AUDIO));
    assert_eq!(FieldName::from_raw(FieldName::AUDIO.to_raw()), Some(FieldName::AUDIO));
    assert_eq!(FieldName::len(), 4);        // Declarations, including aliases.
    assert_eq!(FieldName::unique_len(), 3); // Distinct stored strings.
    assert_eq!(AUDIO_TEXT, "audio");
}
```

Entries may be any const-evaluable `&'static str` expression. Equal strings
within one pool share a record and a handle. Empty strings and UTF-8 are
supported; an embedded NUL is rejected at compile time because NUL terminates
each stored record. `lookup(&str)` searches exact contents without inserting
anything; a query containing NUL returns `None`. Use `require` (or `resolve`
with `expect`) in a `const` expression to make a missing declaration a compile
error. An ordinary runtime call to `lookup` scans the pool, while `as_str()`
scans only its selected record up to the terminator. Both are `const fn`, so
their work happens during compilation when used in a const context.

For `(u8)`, `to_raw()` is a dense, nonzero ID and a two-byte offset table maps
each unique ID to its record. For `(u16)`, `to_raw()` is the record's nonzero
byte offset; the pool reserves offset zero and needs no offset table. Thus
`index()` is constant time for `u8`, but scans preceding records for `u16`.
`from_raw()` validates a value against the chosen pool. `offset()` returns
the record's byte offset for either width.

### Object pools

```rust
use catplay_reflect::static_objects;

struct Command { opcode: u16, timeout_ms: u16 }

static_objects! {
    struct CommandRef(u8): Command {
        PLAY = Command { opcode: 1, timeout_ms: 100 },
        STOP = Command { opcode: 2, timeout_ms: 250 },
    }
}

const PLAY: &Command = CommandRef::PLAY.get();

fn main() {
    assert_eq!(PLAY.opcode, 1);
    assert_eq!(CommandRef::from_raw(2), Some(CommandRef::STOP));
    assert_eq!(core::mem::size_of::<Option<CommandRef>>(), 1);
}
```

Objects are stored directly in a static array, with their natural alignment.
Their type must be `Sized + Sync`, and initializers must be valid for a static;
the objects do not need `Copy`, `Clone`, or `Debug`. `get()` returns a shared
reference without copying the object. Object handles encode the declaration
index plus one. Equal objects remain separate entries. A pool of references
stores full references, so declare object values when that is what you want
the pool to own.

### Choosing a width and using raw handles

| Pool | `(u8)` | `(u16)` or omitted width |
| --- | --- | --- |
| String handle | Dense ID, at most 255 unique strings | Direct byte offset; no index table |
| Object handle | At most 255 objects | At most 65,535 objects |
| `Option<Handle>` | 1 byte | 2 bytes |

String pools occupy at most 65,536 bytes of UTF-8 records and terminators,
including the reserved prefix byte for `(u16)`. A single string may contain
at most 65,535 UTF-8 bytes with `(u8)`, or 65,534 with `(u16)`. The `(u8)`
string pool additionally stores two offset bytes per unique string. Object
pools have no 64 KiB byte limit; their limits count objects instead. The
macros check these limits during compilation. `storage_bytes()`,
`index_bytes()`, and `total_storage_bytes()` report string-pool data costs;
they exclude linker padding and the optional handle table returned by
`all()`.

Raw values are local to their pool and declaration layout. Reordering or
editing entries can change them, and a string `(u16)` raw value means an
offset rather than the ID used by an object `(u16)` handle. Do not persist
raw values as unversioned protocol or file-format identifiers. Keep the typed
handle where possible; use `from_raw()` when accepting a value from elsewhere.

`StaticHandle` lets generic code resolve either kind of handle through
`get()`, `to_raw()`, and `from_raw()`. The concrete handle type selects the
pool at compile time; an individual handle stores no base pointer or
resolver. If one runtime table must select an object pool once and store
many raw handles, `HandleShared<Object, Raw>` provides a shared pool
descriptor; `FieldTable` uses this pattern for contexts. See
[`examples/static_pools_basic.rs`](examples/static_pools_basic.rs) for both
widths, generic access, and raw-handle validation.

To check the examples and pool behavior:

```sh
cargo test -p catplay_reflect
cargo run -p catplay_reflect --example static_pools_basic
```
