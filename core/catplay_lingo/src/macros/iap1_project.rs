#[doc(hidden)]
#[macro_export]
macro_rules! __iap1_project_struct {
    (
        strings = $strings:path;
        $(#[$meta:meta])* $vis:vis struct $name:ident {
            fields {
                $( $(#[$field_meta:meta])* $mode:ident $field_vis:vis $field:ident: $ty:ty => {
                bit: $bit:literal, key: $key:literal, when: $active:expr,
                predicate_value: $predicate_value:tt
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
        $crate::__field_project::field_project! {
            #[derive(Clone, PartialEq, Eq, Default)]
            $(#[$meta])*
            $vis struct $name {
                $( $(#[$field_meta])* $field_vis $field: $crate::__iap1_field_type!($mode, $ty), )*
            }
            project $crate::Iap1Projector<$strings>, context = $crate::Iap1StepContext<$strings>, pool = u8 {
                $($slot: &$crate::Iap1StepContext::<$strings>::new(
                    $step_bit,
                    const { <$strings>::require($step_key) },
                    $crate::__iap1_project_mode!($step_mode),
                    $step_predicate_value,
                    &[$($program),*],
                    &($project_wire),
                ) => $step_mode,)*
            }
        }

        impl $crate::Iap1StructName for $name {
            type StringPool = $strings;
            const STRUCT_NAME_HANDLE: $strings =
                const { <$strings>::require(stringify!($name)) };
            const STRUCT_NAME: &'static str = Self::STRUCT_NAME_HANDLE.as_str();
        }

        impl ::core::fmt::Debug for $name {
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                $crate::Iap1StructName::fmt_projected_debug(self, formatter)
            }
        }

        impl $crate::__NamedDebug for $name {
            fn fmt_named(&self, name: &str, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                $crate::debug_projected_fields(self, name, formatter)
            }
        }

        const _: () = {
            assert!((&[$($key),*] as &[&str]).len() <= 128, "iAP1 structs support at most 128 fields");
            $(assert!($bit < 128, "field bit must be below 128");)*
            $(assert!($step_bit < 128, "step field bit must be below 128");)*
        };

        impl $name {
            const __LENGTH_SELECTED: bool = false $(|| $crate::__iap1_length_selected!($mode))*;
            const __FIELD_BITS: &'static [u32] = &[$($bit),*];
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __iap1_length_selected {
    (length_optional) => {
        true
    };
    ($mode:ident) => {
        false
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __iap1_project_mode {
    (required) => {
        $crate::Iap1FieldMode::Required
    };
    (optional) => {
        $crate::Iap1FieldMode::Optional
    };
    (length_optional) => {
        $crate::Iap1FieldMode::LengthOptional
    };
}
