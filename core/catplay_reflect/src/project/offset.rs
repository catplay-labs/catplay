use core::marker::PhantomData;

/// Describes a typed field location inside `S`.
///
/// The field type marker is checked through an accessor function. The offset is
/// erased only after this check, when the field is added to a `FieldOp`.
pub struct FieldOffset<S, F> {
    offset: usize,
    _types: PhantomData<fn(&mut S) -> &mut F>,
}

impl<S, F> FieldOffset<S, F> {
    /// Construct a field offset after checking that `accessor` selects an `F`.
    ///
    /// # Safety
    /// `offset` must be the byte offset of the field selected by `accessor` in
    /// every valid value of `S`. `S` must not have a packed representation if
    /// the projector obtains a reference to the field.
    pub const unsafe fn from_offset(offset: usize, accessor: for<'a> fn(&'a mut S) -> &'a mut F) -> Self {
        let _ = accessor;
        Self {
            offset,
            _types: PhantomData,
        }
    }

    pub const fn byte_offset(&self) -> usize {
        self.offset
    }
}
mod offset_width {
    pub trait Sealed {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
}

/// Storage width for byte offsets in a [`crate::FieldEntry`].
///
/// Only `u16` and `u32` are supported. The `const` constructors use
/// [`ConstOffset`] to encode an `offset_of!` value at compile time.
pub trait OffsetWidth: Copy + offset_width::Sealed {
    fn as_usize(self) -> usize;
}

impl OffsetWidth for u16 {
    fn as_usize(self) -> usize {
        usize::from(self)
    }
}

impl OffsetWidth for u32 {
    fn as_usize(self) -> usize {
        self as usize
    }
}

/// Compile-time encoding used by `field_project!` constructors.
#[doc(hidden)]
pub trait ConstOffset<const OFFSET: usize>: OffsetWidth {
    const VALUE: Self;
}

impl<const OFFSET: usize> ConstOffset<OFFSET> for u16 {
    const VALUE: Self = {
        assert!(OFFSET <= u16::MAX as usize, "field_project!: field offset exceeds u16");
        OFFSET as u16
    };
}

impl<const OFFSET: usize> ConstOffset<OFFSET> for u32 {
    const VALUE: Self = {
        assert!(OFFSET <= u32::MAX as usize, "field_project!: field offset exceeds u32");
        OFFSET as u32
    };
}
