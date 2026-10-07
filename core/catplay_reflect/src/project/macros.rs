/// Declares a named-field struct and one or more independent field tables.
///
/// Every table entry is checked against the selected field's concrete type and
/// its projector capability implementation.
#[macro_export]
macro_rules! field_project {
    (
        $(#[$struct_attr:meta])*
        $vis:vis struct $name:ident {
            $($(#[$field_attr:meta])* $field_vis:vis $field:ident : $field_ty:ty),* $(,)?
        }
        $(
            project $projector:ty, context = $context_ty:ty $(, pool = $pool:ident)? {
                $($(#[$($entry_attr:tt)*])* $projected_field:ident : $context:expr $(=> $field_mode:ident)?),* $(,)?
            }
        )*
    ) => {
        $(#[$struct_attr])*
        $vis struct $name {
            $($(#[$field_attr])* $field_vis $field: $field_ty),*
        }

        $(
            $crate::__field_project_numbered! {
                @start [$vis] [$name] [$context_ty] [$projector] [$($pool)?]
                $(([$projected_field] [$context] [$(#[$($entry_attr)*])*] [$($field_mode)?])),*
            }
        )*
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __field_project_numbered {
    (@start [$vis:vis] [$name:ident] [$context_ty:ty] [$projector:ty] [$($pool:ident)?] $($entries:tt)*) => {
        // macro_rules! cannot construct identifiers from integers. Reserve
        // pool-local names and consume one for each declared operation.
        $crate::__field_project_numbered! {
            @assign [$vis] [$name] [$context_ty] [$projector] [$($pool)?] []
            [
            C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 C11 C12 C13 C14 C15 C16
            C17 C18 C19 C20 C21 C22 C23 C24 C25 C26 C27 C28 C29 C30 C31 C32
            C33 C34 C35 C36 C37 C38 C39 C40 C41 C42 C43 C44 C45 C46 C47 C48
            C49 C50 C51 C52 C53 C54 C55 C56 C57 C58 C59 C60 C61 C62 C63 C64
            C65 C66 C67 C68 C69 C70 C71 C72 C73 C74 C75 C76 C77 C78 C79 C80
            C81 C82 C83 C84 C85 C86 C87 C88 C89 C90 C91 C92 C93 C94 C95 C96
            C97 C98 C99 C100 C101 C102 C103 C104 C105 C106 C107 C108 C109 C110 C111 C112
            C113 C114 C115 C116 C117 C118 C119 C120 C121 C122 C123 C124 C125 C126 C127 C128
            ]
            [$($entries)*]
        }
    };
    (
        @assign [$vis:vis] [$name:ident] [$context_ty:ty] [$projector:ty] [$($pool:ident)?]
        [$($assigned:tt)*] [$n1:ident $n2:ident $n3:ident $n4:ident $($names:ident)*]
        [
            ([$f1:ident] [$c1:expr] [$($a1:tt)*] [$($m1:ident)?]),
            ([$f2:ident] [$c2:expr] [$($a2:tt)*] [$($m2:ident)?]),
            ([$f3:ident] [$c3:expr] [$($a3:tt)*] [$($m3:ident)?]),
            ([$f4:ident] [$c4:expr] [$($a4:tt)*] [$($m4:ident)?])
            $(, $remaining:tt)* $(,)?
        ]
    ) => {
        $crate::__field_project_numbered! {
            @assign [$vis] [$name] [$context_ty] [$projector] [$($pool)?]
            [
                $($assigned)*
                ([$n1] [$f1] [$c1] [$($a1)*] [$($m1)?]),
                ([$n2] [$f2] [$c2] [$($a2)*] [$($m2)?]),
                ([$n3] [$f3] [$c3] [$($a3)*] [$($m3)?]),
                ([$n4] [$f4] [$c4] [$($a4)*] [$($m4)?]),
            ]
            [$($names)*]
            [$($remaining),*]
        }
    };
    (
        @assign [$vis:vis] [$name:ident] [$context_ty:ty] [$projector:ty] [$($pool:ident)?]
        [$($assigned:tt)*] [$next:ident $($names:ident)*]
        [([$field:ident] [$context:expr] [$($attr:tt)*] [$($mode:ident)?]) $(, $remaining:tt)* $(,)?]
    ) => {
        $crate::__field_project_numbered! {
            @assign [$vis] [$name] [$context_ty] [$projector] [$($pool)?]
            [$($assigned)* ([$next] [$field] [$context] [$($attr)*] [$($mode)?]),]
            [$($names)*]
            [$($remaining),*]
        }
    };
    (
        @assign [$vis:vis] [$name:ident] [$context_ty:ty] [$projector:ty] [$($pool:ident)?]
        [$($assigned:tt)*] [$($names:ident)*] []
    ) => {
        $crate::__field_project_pool_numbered! {
            [$vis] [$name] [$context_ty] [$projector] [$($pool)?] $($assigned)*
        }
    };
    (@assign [$vis:vis] [$name:ident] [$context_ty:ty] [$projector:ty] [$($pool:ident)?] [$($assigned:tt)*] [] [$($remaining:tt)+]) => {
        ::core::compile_error!("field_project!: more than 128 context entries");
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __field_project_pool_numbered {
    (
        [$vis:vis] [$name:ident] [$context_ty:ty] [$projector:ty] [] $($entries:tt)*
    ) => {
        $crate::__field_project_pool_numbered! { [$vis] [$name] [$context_ty] [$projector] [u16] $($entries)* }
    };
    (
        [$vis:vis] [$name:ident] [$context_ty:ty] [$projector:ty] [$pool:ident]
        $(([$pool_entry:ident] [$field:ident] [$context:expr] [$($entry_attr:tt)*] [$($field_mode:ident)?])),* $(,)?
    ) => {
        const _: () = {
            $crate::static_objects! {
                struct __FieldContextPool($pool): <$projector as $crate::FieldProjector<$context_ty>>::ContextPoolObject {
                    $($pool_entry = $context),*
                }
            }

            unsafe impl $crate::FieldProject<$projector, $context_ty> for $name {
                const FIELDS: &'static $crate::FieldTable<$context_ty, $projector> =
                    &$crate::FieldTable::new::<__FieldContextPool>(
                        &[
                            $(
                                $crate::__field_project_op! {
                                    $name, $context_ty, $projector, $field,
                                    __FieldContextPool::$pool_entry.to_raw();
                                    [$($entry_attr)*];
                                    $($field_mode)?
                                }
                            ),*
                        ],
                    );
            }
        };
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __field_project_op {
    ($name:ident, $context_ty:ty, $projector:ty, $field:ident, $context:expr; [#[csm_count(optional)]]; $($field_mode:ident)?) => {
        $crate::FieldEntry::<$context_ty, $projector>::from_optional_offset::<$name, _, { ::core::mem::offset_of!($name, $field) }>(
            // SAFETY: offset_of! and the accessor select this exact Option<T> field.
            unsafe { $crate::FieldOffset::<$name, _>::from_offset(
                ::core::mem::offset_of!($name, $field),
                |value: &mut $name| &mut value.$field,
            ) },
            $context,
        )
    };
    ($name:ident, $context_ty:ty, $projector:ty, $field:ident, $context:expr; [#[csm_count($count:ident)]]; $($field_mode:ident)?) => {
        $crate::__field_project_op!($name, $context_ty, $projector, $field, $context; $($field_mode)? )
    };
    ($name:ident, $context_ty:ty, $projector:ty, $field:ident, $context:expr; []; $($field_mode:ident)?) => {
        $crate::__field_project_op!($name, $context_ty, $projector, $field, $context; $($field_mode)? )
    };
    ($name:ident, $context_ty:ty, $projector:ty, $field:ident, $context:expr; optional) => {
        $crate::FieldEntry::<$context_ty, $projector>::from_optional_offset::<$name, _, { ::core::mem::offset_of!($name, $field) }>(
            // SAFETY: offset_of! and the accessor select this exact Option<T> field.
            unsafe { $crate::FieldOffset::<$name, _>::from_offset(
                ::core::mem::offset_of!($name, $field),
                |value: &mut $name| &mut value.$field,
            ) },
            $context,
        )
    };
    ($name:ident, $context_ty:ty, $projector:ty, $field:ident, $context:expr; length_optional) => {
        $crate::__field_project_op!($name, $context_ty, $projector, $field, $context; optional)
    };
    ($name:ident, $context_ty:ty, $projector:ty, $field:ident, $context:expr; required) => {
        $crate::__field_project_op!($name, $context_ty, $projector, $field, $context; value)
    };
    ($name:ident, $context_ty:ty, $projector:ty, $field:ident, $context:expr; value) => {
        $crate::FieldEntry::<$context_ty, $projector>::from_offset::<$name, _, { ::core::mem::offset_of!($name, $field) }>(
            // SAFETY: offset_of! and the accessor select this exact field.
            unsafe { $crate::FieldOffset::<$name, _>::from_offset(
                ::core::mem::offset_of!($name, $field),
                |value: &mut $name| &mut value.$field,
            ) },
            $context,
        )
    };
    ($name:ident, $context_ty:ty, $projector:ty, $field:ident, $context:expr;) => {
        $crate::__field_project_op!($name, $context_ty, $projector, $field, $context; value)
    };
}
