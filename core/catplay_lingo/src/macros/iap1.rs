/// Defines one iAP1 payload type and its static codec from normalized schema.
/// `fields` declare storage; `steps` define the ordered wire operations.
/// The former field-attribute syntax is not accepted:
///
/// ```compile_fail
/// catplay_lingo::iap1! {
///     #[iap1(context = ctx, lingo = 0, command = 1, source = device,
///         response = false, ack = false, deprecated = false,
///         transaction_id = permitted)]
///     pub struct Legacy {
///         #[iap1_field(key = "value", wire = [scalar])]
///         pub value: u8,
///     }
/// }
/// ```
#[macro_export]
macro_rules! iap1 {
    (
        #[iap1(context = $_ctx:ident, strings = $strings:path, lingo = $lingo:expr, command = $command:expr,
               source = $source:ident, response = $response:expr, ack = $ack:expr,
               deprecated = $deprecated:expr, transaction_id = $transaction_id:ident)]
        $(#[$meta:meta])* $vis:vis struct $name:ident {
            fields { $($fields:tt)* }
            steps { $($steps:tt)* }
        }
    ) => {
        $crate::__iap1_project_struct! { strings = $strings; $(#[$meta])* $vis struct $name {
            fields { $($fields)* } steps { $($steps)* }
        } }
        impl $crate::Iap1Message for $name {
            const META: $crate::Iap1MessageMetadata = $crate::Iap1MessageMetadata {
                lingo: $lingo, command: $command,
                source: $crate::__iap1_source!($source), response: $response, ack: $ack,
                deprecated: $deprecated,
                transaction_id: $crate::__iap1_transaction_policy!($transaction_id),
            };
            const KEY: $crate::CommandKey = $crate::CommandKey {
                lingo: $lingo, command: $command,
            };
            const FIELD_BITS: &'static [u32] = Self::__FIELD_BITS;
            const LENGTH_SELECTED: bool = Self::__LENGTH_SELECTED;
        }
    };
}
