use std::{
    io, mem,
    ptr::{self, NonNull},
    slice,
};

const MAGIC: u64 = 0x4350_5245_434c_414d;
const VERSION: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct ReclaimableVecHeader {
    magic: u64,
    version: u32,
    header_len: u32,
    len: u64,
    capacity: u64,
    data_crc32: u32,
    header_crc32: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum ReclaimableVecError {
    #[error("Gone")]
    Gone,
    #[error("buffer capacity overflow")]
    CapacityOverflow,
    #[error("not enough capacity")]
    NotEnoughCapacity,
    #[error("mmap failed: {0}")]
    Mmap(io::Error),
    #[error("munmap failed: {0}")]
    Munmap(io::Error),
    #[error("madvise failed: {0}")]
    Madvise(io::Error),
}

pub struct ReclaimableVec {
    ptr: NonNull<u8>,
    map_len: usize,
    data_offset: usize,
    capacity: usize,
    len: usize,
}

pub struct ReclaimableVecBuilder {
    ptr: NonNull<u8>,
    map_len: usize,
    data_offset: usize,
    capacity: usize,
    len: usize,
}

unsafe impl Send for ReclaimableVec {}
unsafe impl Send for ReclaimableVecBuilder {}

impl ReclaimableVec {
    pub fn new() -> Result<Self, ReclaimableVecError> {
        ReclaimableVecBuilder::new()?.build()
    }

    pub fn with_capacity(capacity: usize) -> Result<Self, ReclaimableVecError> {
        ReclaimableVecBuilder::with_capacity(capacity)?.build()
    }

    pub fn from_slice(data: &[u8]) -> Result<Self, ReclaimableVecError> {
        ReclaimableVecBuilder::from_slice(data)?.build()
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn copy_to_vec(&self) -> Result<Vec<u8>, ReclaimableVecError> {
        let header = self.read_valid_header()?;
        let len = usize::try_from(header.len).map_err(|_| ReclaimableVecError::Gone)?;
        let capacity = usize::try_from(header.capacity).map_err(|_| ReclaimableVecError::Gone)?;
        if len > capacity || capacity != self.capacity || len != self.len {
            return Err(ReclaimableVecError::Gone);
        }

        let mut out = vec![0u8; len];
        unsafe {
            ptr::copy_nonoverlapping(self.data_ptr(), out.as_mut_ptr(), len);
        }

        if crc32(&out) != header.data_crc32 {
            return Err(ReclaimableVecError::Gone);
        }

        Ok(out)
    }

    pub fn validate(&self) -> Result<(), ReclaimableVecError> {
        let header = self.read_valid_header()?;
        let len = usize::try_from(header.len).map_err(|_| ReclaimableVecError::Gone)?;
        let capacity = usize::try_from(header.capacity).map_err(|_| ReclaimableVecError::Gone)?;
        if len > capacity || capacity != self.capacity || len != self.len {
            return Err(ReclaimableVecError::Gone);
        }

        if crc32(unsafe { slice::from_raw_parts(self.data_ptr(), len) }) != header.data_crc32 {
            return Err(ReclaimableVecError::Gone);
        }

        Ok(())
    }

    #[inline(always)]
    pub(crate) fn as_ptr(&self) -> *const u8 {
        self.data_ptr()
    }

    fn read_valid_header(&self) -> Result<ReclaimableVecHeader, ReclaimableVecError> {
        let header = unsafe { ptr::read_unaligned(self.ptr.as_ptr().cast::<ReclaimableVecHeader>()) };

        if header.magic != MAGIC
            || header.version != VERSION
            || header.header_len as usize != mem::size_of::<ReclaimableVecHeader>()
            || header_crc32(&header) != header.header_crc32
        {
            return Err(ReclaimableVecError::Gone);
        }

        Ok(header)
    }

    #[inline(always)]
    fn data_ptr(&self) -> *mut u8 {
        unsafe { self.ptr.as_ptr().add(self.data_offset) }
    }

    #[cfg(test)]
    fn corrupt_header_for_test(&self) {
        unsafe {
            *self.ptr.as_ptr() = 0;
        }
    }

    #[cfg(test)]
    fn corrupt_data_for_test(&self) {
        if self.len > 0 {
            unsafe {
                *self.data_ptr() ^= 0xff;
            }
        }
    }

    #[cfg(test)]
    fn force_reclaim(&self) -> Result<(), ReclaimableVecError> {
        madvise_reclaimable(self.ptr, self.map_len, true)
    }
}

impl Drop for ReclaimableVec {
    fn drop(&mut self) {
        munmap_or_panic(self.ptr, self.map_len);
    }
}

impl ReclaimableVecBuilder {
    pub fn new() -> Result<Self, ReclaimableVecError> {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Result<Self, ReclaimableVecError> {
        let data_offset = data_offset();
        let wanted_len = data_offset
            .checked_add(capacity)
            .ok_or(ReclaimableVecError::CapacityOverflow)?;
        let page_size = page_size().ok_or_else(|| ReclaimableVecError::Mmap(io::Error::last_os_error()))?;
        let map_len = round_up(wanted_len.max(1), page_size).ok_or(ReclaimableVecError::CapacityOverflow)?;

        let raw = unsafe {
            libc::mmap(
                ptr::null_mut(),
                map_len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANON,
                -1,
                0,
            )
        };
        if raw == libc::MAP_FAILED {
            return Err(ReclaimableVecError::Mmap(io::Error::last_os_error()));
        }

        let mut builder = Self {
            ptr: unsafe { NonNull::new_unchecked(raw.cast::<u8>()) },
            map_len,
            data_offset,
            capacity,
            len: 0,
        };
        builder.write_header(0);

        Ok(builder)
    }

    pub fn from_slice(data: &[u8]) -> Result<Self, ReclaimableVecError> {
        let mut builder = Self::with_capacity(data.len())?;
        builder.extend_from_slice(data)?;
        Ok(builder)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.write_header(0);
    }

    #[inline(always)]
    pub fn as_mut_capacity_slice(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.data_ptr(), self.capacity) }
    }

    #[inline(always)]
    pub(crate) fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data_ptr()
    }

    pub fn set_len(&mut self, len: usize) -> Result<(), ReclaimableVecError> {
        if len > self.capacity {
            return Err(ReclaimableVecError::NotEnoughCapacity);
        }

        self.len = len;
        self.refresh_header_crc();
        Ok(())
    }

    pub fn push(&mut self, byte: u8) -> Result<(), ReclaimableVecError> {
        if self.len == self.capacity {
            return Err(ReclaimableVecError::NotEnoughCapacity);
        }

        unsafe {
            *self.data_ptr().add(self.len) = byte;
        }
        self.len += 1;
        self.refresh_header_crc();
        Ok(())
    }

    pub fn extend_from_slice(&mut self, data: &[u8]) -> Result<(), ReclaimableVecError> {
        let new_len = self
            .len
            .checked_add(data.len())
            .ok_or(ReclaimableVecError::CapacityOverflow)?;
        if new_len > self.capacity {
            return Err(ReclaimableVecError::NotEnoughCapacity);
        }

        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), self.data_ptr().add(self.len), data.len());
        }
        self.len = new_len;
        self.refresh_header_crc();
        Ok(())
    }

    pub fn set_from_slice(&mut self, data: &[u8]) -> Result<(), ReclaimableVecError> {
        if data.len() > self.capacity {
            return Err(ReclaimableVecError::NotEnoughCapacity);
        }

        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), self.data_ptr(), data.len());
        }
        self.len = data.len();
        self.refresh_header_crc();
        Ok(())
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data_ptr(), self.len) }
    }

    pub fn build(mut self) -> Result<ReclaimableVec, ReclaimableVecError> {
        self.refresh_header_crc();
        madvise_reclaimable(self.ptr, self.map_len, false)?;

        let vec = ReclaimableVec {
            ptr: self.ptr,
            map_len: self.map_len,
            data_offset: self.data_offset,
            capacity: self.capacity,
            len: self.len,
        };
        mem::forget(self);

        Ok(vec)
    }

    fn refresh_header_crc(&mut self) {
        let crc = crc32(self.as_slice());
        self.write_header(crc);
    }

    fn write_header(&mut self, data_crc32: u32) {
        let mut header = ReclaimableVecHeader {
            magic: MAGIC,
            version: VERSION,
            header_len: mem::size_of::<ReclaimableVecHeader>() as u32,
            len: self.len as u64,
            capacity: self.capacity as u64,
            data_crc32,
            header_crc32: 0,
        };
        header.header_crc32 = header_crc32(&header);

        unsafe {
            ptr::write_unaligned(self.ptr.as_ptr().cast::<ReclaimableVecHeader>(), header);
        }
    }

    #[inline(always)]
    fn data_ptr(&self) -> *mut u8 {
        unsafe { self.ptr.as_ptr().add(self.data_offset) }
    }
}

impl Drop for ReclaimableVecBuilder {
    fn drop(&mut self) {
        munmap_or_panic(self.ptr, self.map_len);
    }
}

fn madvise_reclaimable(ptr: NonNull<u8>, map_len: usize, force: bool) -> Result<(), ReclaimableVecError> {
    #[cfg(any(target_os = "android", target_os = "linux"))]
    const ADVICE: libc::c_int = libc::MADV_FREE;
    #[cfg(not(any(target_os = "android", target_os = "linux")))]
    const ADVICE: libc::c_int = libc::MADV_DONTNEED;

    let rc = unsafe {
        libc::madvise(
            ptr.as_ptr().cast::<libc::c_void>(),
            map_len,
            if force { libc::MADV_DONTNEED } else { ADVICE },
        )
    };
    if rc != 0 {
        return Err(ReclaimableVecError::Madvise(io::Error::last_os_error()));
    }

    Ok(())
}

fn munmap_or_panic(ptr: NonNull<u8>, map_len: usize) {
    let rc = unsafe { libc::munmap(ptr.as_ptr().cast::<libc::c_void>(), map_len) };
    if rc != 0 && !std::thread::panicking() {
        panic!("{}", ReclaimableVecError::Munmap(io::Error::last_os_error()));
    }
}

fn data_offset() -> usize {
    round_up(mem::size_of::<ReclaimableVecHeader>(), mem::align_of::<u64>()).expect("static header size must align")
}

fn header_crc32(header: &ReclaimableVecHeader) -> u32 {
    let len = mem::size_of::<ReclaimableVecHeader>() - mem::size_of::<u32>();
    let bytes = unsafe { slice::from_raw_parts((header as *const ReclaimableVecHeader).cast::<u8>(), len) };
    crc32(bytes)
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(bytes);
    hasher.finalize()
}

fn round_up(value: usize, align: usize) -> Option<usize> {
    let rem = value % align;
    if rem == 0 { Some(value) } else { value.checked_add(align - rem) }
}

fn page_size() -> Option<usize> {
    let value = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    (value > 0).then_some(value as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copies_content_out_when_crc_matches() {
        let vec = ReclaimableVec::from_slice(b"catplay").unwrap();

        assert_eq!(vec.copy_to_vec().unwrap(), b"catplay");
    }

    #[test]
    fn detects_gone_header_before_copying_content() {
        let vec = ReclaimableVec::from_slice(b"catplay").unwrap();
        vec.corrupt_header_for_test();

        assert!(matches!(vec.copy_to_vec(), Err(ReclaimableVecError::Gone)));
    }

    #[test]
    fn detects_gone_content_after_copying_content() {
        let vec = ReclaimableVec::from_slice(b"catplay").unwrap();
        vec.corrupt_data_for_test();

        assert!(matches!(vec.copy_to_vec(), Err(ReclaimableVecError::Gone)));
    }

    #[test]
    fn rejects_growth_past_capacity() {
        let mut vec = ReclaimableVecBuilder::with_capacity(2).unwrap();

        assert!(vec.extend_from_slice(b"abc").is_err());
    }

    #[test]
    fn builder_can_fill_mmap_without_intermediate_copy() {
        let mut builder = ReclaimableVecBuilder::with_capacity(7).unwrap();
        builder.as_mut_capacity_slice().copy_from_slice(b"catplay");
        builder.set_len(7).unwrap();

        let vec = builder.build().unwrap();
        assert_eq!(vec.copy_to_vec().unwrap(), b"catplay");
    }

    #[test]
    fn test_force_reclaim() {
        let mut builder = ReclaimableVecBuilder::with_capacity(7).unwrap();
        builder.as_mut_capacity_slice().copy_from_slice(b"catplay");
        builder.set_len(7).unwrap();

        let vec = builder.build().unwrap();

        vec.force_reclaim().unwrap();
        // sleep(Duration::from_secs(2));
        assert!(vec.copy_to_vec().is_err());
    }
}
