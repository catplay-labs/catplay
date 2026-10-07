mod shared;
pub use shared::*;

mod cipher_ring;
pub use cipher_ring::*;
mod cipher_fastchacha;
pub use cipher_fastchacha::*;

#[cfg(all(any(target_arch = "mips", target_arch = "riscv32"), feature = "fast_chacha"))]
pub type HomeKitCipher = HomeKitCipherFast;
#[cfg(not(all(any(target_arch = "mips", target_arch = "riscv32"), feature = "fast_chacha")))]
pub type HomeKitCipher = HomeKitCipherRing;
