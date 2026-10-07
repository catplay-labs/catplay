use super::{DecodeContext, DecodeError, EncodeContext, EncodeError, FlatPredicateToken, PredicateProgram, Reader, Writer};
use catplay_reflect::{FieldAccess, FieldProject, FieldProjector, ProjectorCapability, ProjectorOptionalCapability, StringPool};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Iap1WireSpec {
    Scalar,
    Raw,
    CountedRemaining,
    CountedLiteral(usize),
    CountedField(u8),
    /// `variant` is a raw handle from the enclosing step context's string pool.
    Disjoint {
        variant: u16,
        child: &'static Iap1WireSpec,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Iap1FieldMode {
    Required,
    Optional,
    LengthOptional,
}

/// Variant names and wire values for an iAP1 enum.
/// `NAMES` and `VALUES` must have equal lengths and corresponding entries.
pub trait Iap1Enum<Storage: Copy + PartialEq + 'static> {
    type StringPool: StringPool;
    const NAMES: &'static [Self::StringPool];
    const VALUES: &'static [Storage];

    fn raw_value(&self) -> Storage;

    fn is_other(&self) -> bool {
        false
    }

    fn fmt_unknown(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("<unknown>")
    }
}

/// Shared wire decode and membership check for closed enums emitted by `iap1_enum!`.
#[doc(hidden)]
#[inline(never)]
pub fn __iap1_decode_enum_raw<Storage>(reader: &mut Reader<'_>, values: &[Storage]) -> Result<Storage, DecodeError>
where
    Storage: super::Iap1Decode + Copy + PartialEq + Into<u64>,
{
    let raw = Storage::decode(reader)?;
    if values.iter().any(|value| *value == raw) {
        Ok(raw)
    } else {
        Err(DecodeError::InvalidEnumValue { value: raw.into() })
    }
}

/// One wire-tag mapping used by the shared decoder for tagged token enums.
#[doc(hidden)]
pub struct Iap1TokenDecodeOp<T> {
    pub fid_type: u8,
    pub fid_subtype: u8,
    pub decode: for<'a> fn(&mut Reader<'a>) -> Result<T, DecodeError>,
}

/// Decode a tagged token enum using a small table of typed payload callbacks.
#[doc(hidden)]
#[inline(never)]
pub fn __iap1_decode_tagged_token<T, S: StringPool>(
    reader: &mut Reader<'_>,
    token_name: S,
    operations: &[Iap1TokenDecodeOp<T>],
) -> Result<T, DecodeError> {
    let (fid_type, fid_subtype, mut token) = reader.read_tagged_token()?;
    let operation = operations
        .iter()
        .find(|operation| operation.fid_type == fid_type && operation.fid_subtype == fid_subtype)
        .ok_or_else(|| DecodeError::UnknownToken {
            token: token_name.get(),
            fid_type,
            fid_subtype,
        })?;
    let value = (operation.decode)(&mut token)?;
    token.finish()?;
    Ok(value)
}

/// Local wrapper lets the crate share enum Debug formatting despite the
/// orphan rule preventing `Debug` from being blanket-implemented for `T`.
#[doc(hidden)]
pub struct Iap1EnumDebug<'a, T: ?Sized, Storage>(pub &'a T, pub core::marker::PhantomData<Storage>);

#[inline(never)]
fn fmt_iap1_enum<Storage: Copy + PartialEq, S: StringPool>(
    raw: Storage,
    names: &'static [S],
    values: &'static [Storage],
    formatter: &mut core::fmt::Formatter<'_>,
) -> Option<core::fmt::Result> {
    let mut index = 0;
    while index < names.len() && index < values.len() {
        if values[index] == raw {
            return Some(formatter.write_str(names[index].get()));
        }
        index += 1;
    }
    None
}

impl<Storage, T> core::fmt::Debug for Iap1EnumDebug<'_, T, Storage>
where
    Storage: Copy + PartialEq + 'static,
    T: Iap1Enum<Storage> + ?Sized,
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.0.is_other() {
            return self.0.fmt_unknown(formatter);
        }
        match fmt_iap1_enum(self.0.raw_value(), T::NAMES, T::VALUES, formatter) {
            Some(result) => result,
            None => self.0.fmt_unknown(formatter),
        }
    }
}

#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct Iap1StepContext<S: StringPool> {
    pub bit: u8,
    pub key: S,
    pub mode: Iap1FieldMode,
    pub predicate_value: bool,
    pub predicate: &'static [FlatPredicateToken],
    pub wire: &'static Iap1WireSpec,
}

impl<S: StringPool> Iap1StepContext<S> {
    pub const fn new(
        bit: u8,
        key: S,
        mode: Iap1FieldMode,
        predicate_value: bool,
        predicate: &'static [FlatPredicateToken],
        wire: &'static Iap1WireSpec,
    ) -> Self {
        Self {
            bit,
            key,
            mode,
            predicate_value,
            predicate,
            wire,
        }
    }
}

pub struct Iap1Projector<S: StringPool>(core::marker::PhantomData<S>);
#[doc(hidden)]
pub struct Iap1ProjectCapability<S: StringPool>(core::marker::PhantomData<S>);

/// A field value that the common table executor can read and write.
#[doc(hidden)]
pub trait Iap1WireValue: Sized + core::fmt::Debug + Default {
    fn decode_wire(
        reader: &mut Reader<'_>,
        wire: Iap1WireSpec,
        values: &[Option<i128>; 128],
        key: &'static str,
    ) -> Result<Self, DecodeError>;

    fn encode_wire(
        &self,
        writer: &mut Writer<'_>,
        wire: Iap1WireSpec,
        values: &[Option<i128>; 128],
        key: &'static str,
    ) -> Result<bool, EncodeError>;

    fn predicate_integer(&self) -> Option<i128> {
        None
    }
}

impl<T> Iap1WireValue for T
where
    T: super::Iap1Decode + super::Iap1Encode + super::PredicateValue + core::fmt::Debug + Default,
{
    fn decode_wire(
        reader: &mut Reader<'_>,
        wire: Iap1WireSpec,
        _values: &[Option<i128>; 128],
        key: &'static str,
    ) -> Result<Self, DecodeError> {
        if wire != Iap1WireSpec::Scalar {
            return Err(DecodeError::InvalidFieldValue { field: key });
        }
        reader.with_field_fn(key, &T::decode)
    }

    fn encode_wire(
        &self,
        writer: &mut Writer<'_>,
        wire: Iap1WireSpec,
        _values: &[Option<i128>; 128],
        key: &'static str,
    ) -> Result<bool, EncodeError> {
        if wire != Iap1WireSpec::Scalar {
            return Err(EncodeError::InvalidFieldValue { field: key });
        }
        super::Iap1Encode::encode(self, writer)?;
        Ok(true)
    }

    fn predicate_integer(&self) -> Option<i128> {
        super::PredicateValue::predicate_integer(self)
    }
}

impl<T> Iap1WireValue for alloc::vec::Vec<T>
where
    T: super::Iap1Decode + super::Iap1Encode + core::fmt::Debug,
{
    fn decode_wire(
        reader: &mut Reader<'_>,
        wire: Iap1WireSpec,
        values: &[Option<i128>; 128],
        key: &'static str,
    ) -> Result<Self, DecodeError> {
        match decode_vec_opcode(wire, values, key)? {
            Iap1VecOpcode::Remaining => {
                reader.with_field_fn(key, &|reader| <Self as super::Iap1DecodeRemaining>::decode_remaining(reader, key))
            }
            Iap1VecOpcode::Counted(count) => {
                reader.with_field_fn(key, &|reader| <Self as super::Iap1DecodeCounted>::decode_counted(reader, count))
            }
        }
    }

    fn encode_wire(
        &self,
        writer: &mut Writer<'_>,
        wire: Iap1WireSpec,
        values: &[Option<i128>; 128],
        key: &'static str,
    ) -> Result<bool, EncodeError> {
        let count = encode_vec_opcode(wire, self.len(), values, key)?;
        super::Iap1EncodeCounted::encode_counted(self, writer, count, key)?;
        Ok(true)
    }
}

impl Iap1WireValue for alloc::string::String {
    fn decode_wire(
        reader: &mut Reader<'_>,
        wire: Iap1WireSpec,
        values: &[Option<i128>; 128],
        key: &'static str,
    ) -> Result<Self, DecodeError> {
        match wire {
            Iap1WireSpec::Raw | Iap1WireSpec::CountedRemaining => {
                reader.with_field_fn(key, &|reader| <Self as super::Iap1DecodeRemaining>::decode_remaining(reader, key))
            }
            Iap1WireSpec::CountedLiteral(count) => {
                reader.with_field_fn(key, &|reader| <Self as super::Iap1DecodeCounted>::decode_counted(reader, count))
            }
            Iap1WireSpec::CountedField(slot) => {
                let count = decode_count(values, slot, key)?;
                reader.with_field_fn(key, &|reader| <Self as super::Iap1DecodeCounted>::decode_counted(reader, count))
            }
            _ => Err(DecodeError::InvalidFieldValue { field: key }),
        }
    }

    fn encode_wire(
        &self,
        writer: &mut Writer<'_>,
        wire: Iap1WireSpec,
        values: &[Option<i128>; 128],
        key: &'static str,
    ) -> Result<bool, EncodeError> {
        let count = match wire {
            Iap1WireSpec::Raw | Iap1WireSpec::CountedRemaining => self.len() + 1,
            Iap1WireSpec::CountedLiteral(count) => count,
            Iap1WireSpec::CountedField(slot) => encode_count(values, slot, key)?,
            _ => return Err(EncodeError::InvalidFieldValue { field: key }),
        };
        super::Iap1EncodeCounted::encode_counted(self, writer, count, key)?;
        Ok(true)
    }
}

fn decode_count(values: &[Option<i128>; 128], slot: u8, key: &'static str) -> Result<usize, DecodeError> {
    values[usize::from(slot)]
        .and_then(|value| usize::try_from(value).ok())
        .ok_or(DecodeError::InvalidFieldValue { field: key })
}

fn encode_count(values: &[Option<i128>; 128], slot: u8, key: &'static str) -> Result<usize, EncodeError> {
    values[usize::from(slot)]
        .and_then(|value| usize::try_from(value).ok())
        .ok_or(EncodeError::InvalidFieldValue { field: key })
}

#[derive(Clone, Copy)]
enum Iap1VecOpcode {
    Remaining,
    Counted(usize),
}

#[inline(never)]
fn decode_vec_opcode(wire: Iap1WireSpec, values: &[Option<i128>; 128], key: &'static str) -> Result<Iap1VecOpcode, DecodeError> {
    match wire {
        Iap1WireSpec::Raw | Iap1WireSpec::CountedRemaining => Ok(Iap1VecOpcode::Remaining),
        Iap1WireSpec::CountedLiteral(count) => Ok(Iap1VecOpcode::Counted(count)),
        Iap1WireSpec::CountedField(slot) => Ok(Iap1VecOpcode::Counted(decode_count(values, slot, key)?)),
        _ => Err(DecodeError::InvalidFieldValue { field: key }),
    }
}

#[inline(never)]
fn encode_vec_opcode(wire: Iap1WireSpec, len: usize, values: &[Option<i128>; 128], key: &'static str) -> Result<usize, EncodeError> {
    match wire {
        Iap1WireSpec::Raw | Iap1WireSpec::CountedRemaining => Ok(len),
        Iap1WireSpec::CountedLiteral(count) => Ok(count),
        Iap1WireSpec::CountedField(slot) => encode_count(values, slot, key),
        _ => Err(EncodeError::InvalidFieldValue { field: key }),
    }
}

#[doc(hidden)]
pub trait Iap1ProjectedField<S: StringPool>: core::fmt::Debug {
    fn decode_step(&mut self, reader: &mut Reader<'_>, values: &[Option<i128>; 128], step: &Iap1StepContext<S>) -> Result<(), DecodeError>;
    fn encode_step(&self, writer: &mut Writer<'_>, values: &[Option<i128>; 128], step: &Iap1StepContext<S>) -> Result<bool, EncodeError>;
    fn predicate_integer(&self) -> Option<i128>;
    fn is_present(&self) -> bool;
}

impl<S: StringPool, T: Iap1WireValue> Iap1ProjectedField<S> for T {
    fn decode_step(&mut self, reader: &mut Reader<'_>, values: &[Option<i128>; 128], step: &Iap1StepContext<S>) -> Result<(), DecodeError> {
        *self = Self::decode_wire(reader, *step.wire, values, step.key.get())?;
        Ok(())
    }

    fn encode_step(&self, writer: &mut Writer<'_>, values: &[Option<i128>; 128], step: &Iap1StepContext<S>) -> Result<bool, EncodeError> {
        self.encode_wire(writer, *step.wire, values, step.key.get())
    }

    fn predicate_integer(&self) -> Option<i128> {
        Iap1WireValue::predicate_integer(self)
    }

    fn is_present(&self) -> bool {
        true
    }
}

impl<S: StringPool, T: Iap1WireValue> Iap1ProjectedField<S> for Option<T> {
    fn decode_step(&mut self, reader: &mut Reader<'_>, values: &[Option<i128>; 128], step: &Iap1StepContext<S>) -> Result<(), DecodeError> {
        if self.is_some() {
            return Err(DecodeError::InvalidFieldValue { field: step.key.get() });
        }
        *self = Some(T::decode_wire(reader, *step.wire, values, step.key.get())?);
        Ok(())
    }

    fn encode_step(&self, writer: &mut Writer<'_>, values: &[Option<i128>; 128], step: &Iap1StepContext<S>) -> Result<bool, EncodeError> {
        let Some(value) = self.as_ref() else { return Ok(false) };
        value.encode_wire(writer, *step.wire, values, step.key.get())
    }

    fn predicate_integer(&self) -> Option<i128> {
        self.as_ref().and_then(Iap1WireValue::predicate_integer)
    }

    fn is_present(&self) -> bool {
        self.is_some()
    }
}

pub enum Iap1Request {
    Decode {
        reader: *mut Reader<'static>,
        context: *const DecodeContext,
        values: *mut [Option<i128>; 128],
        seen: *mut u128,
        error: *mut Option<DecodeError>,
    },
    Collect {
        values: *mut [Option<i128>; 128],
    },
    Validate {
        active: u128,
        seen: *mut u128,
        error: *mut Option<EncodeError>,
    },
    Encode {
        writer: *mut Writer<'static>,
        context: *const EncodeContext,
        values: *const [Option<i128>; 128],
        written: *mut u128,
        length_selected: bool,
        error: *mut Option<EncodeError>,
    },
    Debug {
        formatter: *mut core::fmt::DebugStruct<'static, 'static>,
        seen: *mut u128,
    },
}

impl<S: StringPool> FieldProjector<Iap1StepContext<S>> for Iap1Projector<S> {
    type Request = Iap1Request;
    type Capability = Iap1ProjectCapability<S>;
    type ContextPoolWidth = u8;
    type ContextPoolObject = &'static Iap1StepContext<S>;
    type FieldOffsetWidth = u16;
}

enum ErasedField<'a, S: StringPool> {
    Mutable(&'a mut dyn Iap1ProjectedField<S>),
    Shared(&'a dyn Iap1ProjectedField<S>),
}

impl<S: StringPool> ErasedField<'_, S> {
    fn as_ref(&self) -> &dyn Iap1ProjectedField<S> {
        match self {
            Self::Mutable(value) => &**value,
            Self::Shared(value) => *value,
        }
    }

    fn as_mut(&mut self) -> Option<&mut dyn Iap1ProjectedField<S>> {
        match self {
            Self::Mutable(value) => Some(&mut **value),
            Self::Shared(_) => None,
        }
    }
}

fn apply_field<S: StringPool>(mut field: ErasedField<'_, S>, step: &Iap1StepContext<S>, request: &mut Iap1Request) {
    match request {
        Iap1Request::Decode {
            reader,
            context,
            values,
            seen,
            error,
        } => {
            if unsafe { (**error).is_some() } {
                return;
            }
            // SAFETY: the context pointer remains valid until decode returns.
            let context = unsafe { &**context };
            let values_ref = unsafe { &**values };
            if !PredicateProgram::new(step.predicate).eval_decode(values_ref, context) {
                return;
            }
            let mask = 1u128 << step.bit;
            let already_seen = unsafe { **seen & mask != 0 };
            if already_seen {
                unsafe {
                    **error = Some(DecodeError::InvalidFieldValue { field: step.key.get() });
                }
                return;
            }
            unsafe {
                **seen |= mask;
            }
            // SAFETY: FieldProject generated this operation for the live field.
            let reader = unsafe { &mut **reader };
            let values_ref = unsafe { &**values };
            let Some(value) = field.as_mut() else { return };
            if let Err(failure) = value.decode_step(reader, values_ref, step) {
                unsafe {
                    **error = Some(failure);
                }
            } else if step.predicate_value {
                if let Iap1Request::Decode { values, .. } = request {
                    unsafe {
                        (**values)[usize::from(step.bit)] = value.predicate_integer();
                    }
                }
            }
        }
        Iap1Request::Collect { values } => {
            if step.predicate_value {
                // SAFETY: FieldProject generated this operation for the live field.
                let value = field.as_ref();
                unsafe {
                    (**values)[usize::from(step.bit)] = value.predicate_integer();
                }
            }
        }
        Iap1Request::Validate { active, seen, error } => {
            if unsafe { (**error).is_some() } {
                return;
            }
            let mask = 1u128 << step.bit;
            if unsafe { **seen & mask != 0 } {
                return;
            }
            unsafe {
                **seen |= mask;
            }
            // SAFETY: FieldProject generated this operation for the live field.
            let value = field.as_ref();
            let active = *active & mask != 0;
            match step.mode {
                Iap1FieldMode::Required => {}
                Iap1FieldMode::Optional => match (active, value.is_present()) {
                    (true, false) => unsafe {
                        **error = Some(EncodeError::MissingConditionalField { field: step.key.get() });
                    },
                    (false, true) => unsafe {
                        **error = Some(EncodeError::UnexpectedConditionalField { field: step.key.get() });
                    },
                    _ => {}
                },
                Iap1FieldMode::LengthOptional if !active && value.is_present() => unsafe {
                    **error = Some(EncodeError::UnexpectedConditionalField { field: step.key.get() });
                },
                Iap1FieldMode::LengthOptional => {}
            }
        }
        Iap1Request::Encode {
            writer,
            context,
            values,
            written,
            length_selected,
            error,
            ..
        } => {
            if unsafe { (**error).is_some() } {
                return;
            }
            let context = unsafe { &**context };
            let values_ref = unsafe { &**values };
            if !PredicateProgram::new(step.predicate).eval_encode(values_ref, context) {
                return;
            }
            let mask = 1u128 << step.bit;
            if *length_selected && unsafe { **written & mask != 0 } {
                return;
            }
            // SAFETY: FieldProject generated this operation for the live field.
            let writer = unsafe { &mut **writer };
            let value = field.as_ref();
            match value.encode_step(writer, values_ref, step) {
                Ok(true) => unsafe {
                    **written |= mask;
                },
                Ok(false) if !*length_selected => unsafe {
                    **error = Some(EncodeError::InvalidFieldValue { field: step.key.get() });
                },
                Ok(false) => {}
                Err(failure) => unsafe {
                    **error = Some(failure);
                },
            }
        }
        Iap1Request::Debug { formatter, seen } => {
            let mask = 1u128 << step.bit;
            if unsafe { **seen & mask != 0 } {
                return;
            }
            unsafe {
                **seen |= mask;
            }
            // SAFETY: both pointers stay live through the formatter callback.
            let formatter = unsafe { &mut **formatter };
            formatter.field(step.key.get(), field.as_ref());
        }
    }
}

impl<S: StringPool, F: Iap1ProjectedField<S>> ProjectorCapability<F, Iap1StepContext<S>> for Iap1ProjectCapability<S> {
    type Request = Iap1Request;

    unsafe fn project(field: *mut F, access: FieldAccess, step: &Iap1StepContext<S>, request: &mut Self::Request) {
        let field = match access {
            FieldAccess::Mutable => {
                // SAFETY: mutable mode originates from FieldOp::apply with an exclusive reference.
                ErasedField::Mutable(unsafe { &mut *field })
            }
            FieldAccess::ReadOnly => {
                // SAFETY: read-only mode originates from apply_ref; no mutable reference is formed.
                ErasedField::Shared(unsafe { &*field })
            }
        };
        apply_field(field, step, request);
    }
}

impl<S: StringPool, T: Iap1WireValue> ProjectorOptionalCapability<T, Iap1StepContext<S>> for Iap1ProjectCapability<S> {
    type Request = Iap1Request;

    unsafe fn project_option(field: *mut Option<T>, access: FieldAccess, step: &Iap1StepContext<S>, request: &mut Self::Request) {
        let field = match access {
            FieldAccess::Mutable => {
                // SAFETY: mutable mode originates from FieldOp::apply with an exclusive reference.
                ErasedField::Mutable(unsafe { &mut *field })
            }
            FieldAccess::ReadOnly => {
                // SAFETY: read-only mode originates from apply_ref; no mutable reference is formed.
                ErasedField::Shared(unsafe { &*field })
            }
        };
        apply_field(field, step, request);
    }
}

#[doc(hidden)]
pub type Iap1Fields<S> = catplay_reflect::FieldTable<Iap1StepContext<S>, Iap1Projector<S>>;

fn apply_projected_mut<S: StringPool>(base: *mut u8, request: &mut Iap1Request, fields: &Iap1Fields<S>) {
    for op in fields {
        // SAFETY: caller pairs this project table with its live struct base.
        unsafe { op.apply_raw(base, request) };
    }
}

fn apply_projected_ref<S: StringPool>(base: *const u8, request: &mut Iap1Request, fields: &Iap1Fields<S>) {
    for op in fields {
        // SAFETY: caller pairs this project table with its live struct base; requests only read.
        unsafe { op.apply_ref_raw(base, request) };
    }
}

/// Decode a complete message through its `FieldProject` table.
pub fn decode_projected_payload<S: StringPool, T>(
    value: &mut T,
    fields: &Iap1Fields<S>,
    type_name: &'static str,
    context: &DecodeContext,
    payload: &[u8],
) -> Result<(), DecodeError>
where
    T: FieldProject<Iap1Projector<S>, Iap1StepContext<S>>,
{
    let mut reader = Reader::new(payload);
    let base = (value as *mut T).cast::<u8>();
    reader.with_root_fn(type_name, &|reader| {
        decode_fields(base, fields, context, reader)?;
        reader.check_finished()
    })
}

/// Shared decoder entrypoint for a typed wrapper that already owns a `Reader`.
pub fn decode_projected_fields<S: StringPool, T>(
    value: &mut T,
    fields: &Iap1Fields<S>,
    type_name: &'static str,
    context: &DecodeContext,
    reader: &mut Reader<'_>,
) -> Result<(), DecodeError>
where
    T: FieldProject<Iap1Projector<S>, Iap1StepContext<S>>,
{
    let base = (value as *mut T).cast::<u8>();
    reader.with_root_fn(type_name, &|reader| decode_fields(base, fields, context, reader))
}

fn decode_fields<S: StringPool>(
    base: *mut u8,
    fields: &Iap1Fields<S>,
    context: &DecodeContext,
    reader: &mut Reader<'_>,
) -> Result<(), DecodeError> {
    let mut values = [None; 128];
    let mut seen = 0u128;
    let mut error = None;
    let reader = (reader as *mut Reader<'_>).cast::<Reader<'static>>();
    let context_ptr = context as *const DecodeContext;
    let mut request = Iap1Request::Decode {
        reader,
        context: context_ptr,
        values: &mut values,
        seen: &mut seen,
        error: &mut error,
    };
    apply_projected_mut(base, &mut request, fields);
    if let Some(error) = error {
        return Err(error);
    }
    for op in fields {
        let step = op.context();
        let mask = 1u128 << step.bit;
        if step.mode == Iap1FieldMode::Required && PredicateProgram::new(step.predicate).eval_decode(&values, context) && seen & mask == 0 {
            return Err(DecodeError::InvalidFieldValue { field: step.key.get() });
        }
    }
    Ok(())
}

#[inline(never)]
pub fn encode_projected_fields<S: StringPool, T>(
    value: &T,
    fields: &Iap1Fields<S>,
    context: &EncodeContext,
    writer: &mut Writer<'_>,
    field_bits: &[u32],
    length_selected: bool,
) -> Result<(), EncodeError>
where
    T: FieldProject<Iap1Projector<S>, Iap1StepContext<S>>,
{
    let base = (value as *const T).cast::<u8>();
    // SAFETY: `value` is a live `T`, and `T` is bound to this field table.
    unsafe { encode_projected_fields_raw(base, fields, context, writer, field_bits, length_selected) }
}

/// Type-erased entrypoint used by generated records after they bind their field table.
///
/// # Safety
/// `base` must point to a live value matching `fields` for the duration of the call.
#[doc(hidden)]
#[inline(never)]
pub unsafe fn encode_projected_fields_raw<S: StringPool>(
    base: *const u8,
    fields: &Iap1Fields<S>,
    context: &EncodeContext,
    writer: &mut Writer<'_>,
    field_bits: &[u32],
    length_selected: bool,
) -> Result<(), EncodeError> {
    encode_fields(base, fields, context, writer, field_bits, length_selected)
}

/// Encode one complete message, including length-selected round-trip validation.
pub fn encode_projected_message<S: StringPool, T>(
    value: &T,
    fields: &Iap1Fields<S>,
    type_name: &'static str,
    field_bits: &[u32],
    length_selected: bool,
    context: &EncodeContext,
    writer: &mut Writer<'_>,
) -> Result<(), EncodeError>
where
    T: Default + PartialEq + FieldProject<Iap1Projector<S>, Iap1StepContext<S>>,
{
    if !length_selected {
        // SAFETY: `value` is a live `T`, and the bound pairs `T` with `fields`.
        return unsafe { encode_projected_fields_raw((value as *const T).cast::<u8>(), fields, context, writer, field_bits, false) };
    }

    let mut payload = alloc::vec::Vec::new();
    encode_fields(
        (value as *const T).cast::<u8>(),
        fields,
        context,
        &mut Writer::new(&mut payload),
        field_bits,
        true,
    )?;
    let mut decoded = T::default();
    decode_projected_payload(
        &mut decoded,
        fields,
        type_name,
        &DecodeContext {
            adjusted_payload_length: payload.len(),
            has_transaction_id: context.has_transaction_id,
        },
        &payload,
    )
    .map_err(EncodeError::RoundTripDecode)?;
    if &decoded != value {
        return Err(EncodeError::RoundTripMismatch);
    }
    writer.write_bytes(&payload);
    Ok(())
}

fn encode_fields<S: StringPool>(
    base: *const u8,
    fields: &Iap1Fields<S>,
    context: &EncodeContext,
    writer: &mut Writer<'_>,
    field_bits: &[u32],
    length_selected: bool,
) -> Result<(), EncodeError> {
    let mut values = [None; 128];
    let mut request = Iap1Request::Collect { values: &mut values };
    apply_projected_ref(base, &mut request, fields);
    let mut active = 0u128;
    for op in fields {
        let step = op.context();
        if PredicateProgram::new(step.predicate).eval_encode(&values, context) {
            active |= 1u128 << step.bit;
        }
    }
    let mut validated = 0u128;
    let mut error = None;
    let mut validation = Iap1Request::Validate {
        active,
        seen: &mut validated,
        error: &mut error,
    };
    apply_projected_ref(base, &mut validation, fields);
    if let Some(error) = error {
        return Err(error);
    }

    let writer_ptr = (writer as *mut Writer<'_>).cast::<Writer<'static>>();
    let context_ptr = context as *const EncodeContext;
    let mut written = 0u128;
    let mut request = Iap1Request::Encode {
        writer: writer_ptr,
        context: context_ptr,
        values: &values,
        written: &mut written,
        length_selected,
        error: &mut error,
    };
    if length_selected {
        for &bit in field_bits {
            for op in fields.iter().filter(|op| op.context().bit == bit as u8) {
                // SAFETY: this table is generated for T and encoding only reads fields.
                unsafe { op.apply_ref_raw(base, &mut request) };
                if error.is_some() {
                    break;
                }
            }
            if error.is_some() {
                break;
            }
        }
    } else {
        for op in fields {
            // SAFETY: this table is generated for T and encoding only reads fields.
            unsafe { op.apply_ref_raw(base, &mut request) };
            if error.is_some() {
                break;
            }
        }
    }
    error.map_or(Ok(()), Err)
}

pub fn debug_projected_fields<S: StringPool, T>(value: &T, name: &str, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
where
    T: FieldProject<Iap1Projector<S>, Iap1StepContext<S>>,
{
    let fields = T::fields();
    let base = (value as *const T).cast::<u8>();
    debug_fields(base, fields, name, formatter)
}

fn debug_fields<S: StringPool>(
    base: *const u8,
    fields: &Iap1Fields<S>,
    name: &str,
    formatter: &mut core::fmt::Formatter<'_>,
) -> core::fmt::Result {
    let mut debug = formatter.debug_struct(name);
    let formatter = (&mut debug as *mut core::fmt::DebugStruct<'_, '_>).cast::<core::fmt::DebugStruct<'static, 'static>>();
    let mut seen = 0u128;
    let mut request = Iap1Request::Debug {
        formatter,
        seen: &mut seen,
    };
    apply_projected_ref(base, &mut request, fields);
    debug.finish()
}
