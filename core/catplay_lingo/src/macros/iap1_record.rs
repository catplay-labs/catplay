/// A nested record or token payload defined by the normalized iAP1 operation table.
#[macro_export]
macro_rules! iap1_record {
    (
        strings = $strings:path;
        $(#[$meta:meta])* $vis:vis struct $name:ident {
            fields {
                $( $(#[$field_meta:meta])* $mode:ident $field_vis:vis $field:ident: $ty:ty => {
                    bit: $bit:literal, key: $key:literal, when: $active:expr,
                    predicate_value: $field_predicate_value:tt
                }),* $(,)?
            }
            steps {
                $( $index:literal => $step_mode:ident $slot:ident: $step_ty:ty => {
                    bit: $step_bit:literal, key: $step_key:literal, wire: [$($wire:tt)*],
                    project_wire: $project_wire:expr,
                    predicate_value: $step_predicate_value:tt,
                    predicate: { expr: $predicate:expr, program: [$($program:expr),* $(,)?] }
                }),* $(,)?
            }
        }
    ) => {
        $crate::__iap1_project_struct! { strings = $strings; $(#[$meta])* $vis struct $name {
            fields {
                $( $(#[$field_meta])* $mode $field_vis $field: $ty => {
                    bit: $bit, key: $key, when: $active,
                    predicate_value: $field_predicate_value
                }),*
            }
            steps {
                $( $index => $step_mode $slot: $step_ty => {
                    bit: $step_bit, key: $step_key, wire: [$($wire)*],
                    project_wire: $project_wire,
                    predicate_value: $step_predicate_value,
                    predicate: { expr: $predicate, program: [$($program),*] }
                }),*
            }
        } }
        impl $crate::Iap1Decode for $name {
            fn decode(reader: &mut $crate::Reader<'_>) -> Result<Self, $crate::DecodeError> {
                let mut value = Self::default();
                $crate::decode_projected_fields(
                    &mut value,
                    <$name as $crate::__field_project::FieldProject<
                        $crate::Iap1Projector<$strings>,
                        $crate::Iap1StepContext<$strings>,
                    >>::fields(),
                    <$name as $crate::Iap1StructName>::STRUCT_NAME,
                    &$crate::DecodeContext::default(),
                    reader,
                )?;
                Ok(value)
            }
        }
        impl $crate::Iap1Encode for $name {
            fn encode(&self, writer: &mut $crate::Writer<'_>) -> Result<(), $crate::EncodeError> {
                // SAFETY: this generated table is the FieldProject table for `Self`.
                unsafe {
                    $crate::encode_projected_fields_raw(
                        (self as *const Self).cast::<u8>(),
                        <$name as $crate::__field_project::FieldProject<
                            $crate::Iap1Projector<$strings>,
                            $crate::Iap1StepContext<$strings>,
                        >>::fields(),
                        &$crate::EncodeContext::default(),
                        writer,
                        Self::__FIELD_BITS,
                        false,
                    )
                }
            }
        }
    };
}
