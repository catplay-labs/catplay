//! Support for macro expansions in downstream crates.

use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::num::{NonZeroU8, NonZeroU16};

/// An opaque ID branded with its generated pool type.
///
/// The carrier lives in this crate so callers cannot forge a handle by writing
/// the tuple field of a macro-generated type in their own module. A carrier
/// from a different pool has a different type. No safe constructor exposes an
/// unchecked ID.
macro_rules! define_index {
    ($name:ident, $raw:ty, $nonzero:ty) => {
        #[repr(transparent)]
        pub struct $name<P> {
            raw: $nonzero,
            marker: PhantomData<fn() -> P>,
        }

        impl<P> $name<P> {
            /// Construct an ID after the generated pool has validated membership.
            ///
            /// # Safety
            ///
            /// `raw` must be a valid encoding for pool `P`: an in-bounds
            /// object index plus one, unique-string index plus one for a u8
            /// string pool, or record offset for a u16 string pool.
            /// A nonzero integer alone does not establish this invariant.
            pub const unsafe fn new_unchecked(raw: $raw) -> Self {
                Self {
                    raw: <$nonzero>::new(raw).expect("catplay_reflect: a raw ID must be nonzero"),
                    marker: PhantomData,
                }
            }

            pub const fn get(self) -> $raw {
                self.raw.get()
            }
        }

        // Manual implementations deliberately impose no trait bounds on P.
        impl<P> Copy for $name<P> {}
        impl<P> Clone for $name<P> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<P> PartialEq for $name<P> {
            fn eq(&self, other: &Self) -> bool {
                self.raw == other.raw
            }
        }
        impl<P> Eq for $name<P> {}
        impl<P> Hash for $name<P> {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.raw.hash(state);
            }
        }
    };
}

define_index!(Index8, u8, NonZeroU8);
define_index!(Index16, u16, NonZeroU16);

/// Compatibility name for the original two-byte internal carrier.
pub type Index<P> = Index16<P>;
