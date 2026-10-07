mod cbc;
mod ctr;
mod ctr_kernel;
#[cfg(feature = "openssl")]
mod ctr_openssl;
mod ctr_prefetch;
mod ctr_soft;

pub use crate::prefetch::{PrefetchResult, PrefetchStats, PrefetchStatus};
pub use cbc::Aes128Cbc;
pub use ctr::Aes128Ctr;
pub use ctr_kernel::{Aes128CtrKernelStream, AfAlgError};
#[cfg(feature = "openssl")]
pub use ctr_openssl::Aes128CtrOpenSsl;
pub use ctr_prefetch::{Aes128CtrEngine, Aes128CtrPrefetch, DEFAULT_PREFETCH_CAPACITY, PrefetchError};
pub use ctr_soft::Aes128CtrSoft;

#[cfg(all(target_arch = "mips", feature = "mips_prefers_af_alg"))]
pub(crate) use ctr_kernel::Aes128CtrKernelStream as Aes128CtrBackend;
#[cfg(all(not(all(target_arch = "mips", feature = "mips_prefers_af_alg")), feature = "openssl"))]
pub(crate) use ctr_openssl::Aes128CtrOpenSsl as Aes128CtrBackend;
#[cfg(all(not(all(target_arch = "mips", feature = "mips_prefers_af_alg")), not(feature = "openssl")))]
pub(crate) use ctr_soft::Aes128CtrSoft as Aes128CtrBackend;
