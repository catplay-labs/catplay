use core::mem::MaybeUninit;

/// As we receive RTP datagrams, they start with 12 byte header followed by encrypted payload.
///
/// We would like the post-header payload start to be 16-byte or even 64-byte aligned for optimal decrypt speed, and [AlignedOffsetBuf] helps with that.
///
/// It also offers stack allocation for small RTP MTU-sized buffers as well as a utility to build a collection of slices for `recvmmsg` syscall.
#[repr(C)]
#[repr(align(64))]
#[derive(Clone, Copy)]
pub struct AlignedOffsetBuf<const N: usize, const PAD: usize> {
    _pad: MaybeUninit<[u8; PAD]>,
    buf: MaybeUninit<[u8; N]>,
    written: usize,
}

impl<const N: usize, const PAD: usize> Default for AlignedOffsetBuf<N, PAD> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::missing_safety_doc)]
impl<const N: usize, const PAD: usize> AlignedOffsetBuf<N, PAD> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            _pad: MaybeUninit::uninit(),
            buf: MaybeUninit::uninit(),
            written: 0,
        }
    }

    #[inline(always)]
    pub const fn zeroed() -> Self {
        Self {
            _pad: MaybeUninit::zeroed(),
            buf: MaybeUninit::zeroed(),
            written: 0,
        }
    }

    #[inline(always)]
    pub const fn written(&self) -> usize {
        self.written
    }

    #[inline(always)]
    pub const fn max_size(&self) -> usize {
        N
    }

    #[inline(always)]
    pub const fn as_ptr(&self) -> *const u8 {
        self.buf.as_ptr() as *const u8
    }

    #[inline(always)]
    pub const fn as_mut_ptr(&mut self) -> *mut u8 {
        self.buf.as_mut_ptr() as *mut u8
    }

    #[inline(always)]
    pub unsafe fn assume_written(&mut self, written: usize) {
        // SAFETY: even without assert it will panic at runtime because of [..self.written] later
        debug_assert!(written <= N, "assume_written: {written} > buffer size of {N}");
        self.written = written;
    }

    #[inline(always)]
    pub const unsafe fn uninit_slice(&self) -> &[u8] {
        unsafe { self.buf.assume_init_ref() }
    }

    #[inline(always)]
    pub const unsafe fn uninit_slice_mut(&mut self) -> &mut [u8] {
        unsafe { self.buf.assume_init_mut() }
    }

    #[inline(always)]
    pub fn new_batch<const BATCH: usize>() -> [Self; BATCH] {
        core::array::from_fn(|_| Self::new())
    }

    #[inline(always)]
    pub fn new_batch_zeroed<const BATCH: usize>() -> [Self; BATCH] {
        core::array::from_fn(|_| Self::zeroed())
    }

    #[inline(always)]
    pub fn new_batch_alloc(batch: usize) -> Box<[Self]> {
        vec![Self::new(); batch].into_boxed_slice()
    }

    #[inline(always)]
    pub fn new_batch_zeroed_alloc(batch: usize) -> Box<[Self]> {
        vec![Self::zeroed(); batch].into_boxed_slice()
    }

    #[inline(always)]
    pub unsafe fn build_recv_slices<'a, const BATCH: usize>(batch: impl IntoIterator<Item = &'a mut Self>) -> [&'a mut [u8]; BATCH] {
        let mut tmp: [&'a mut [u8]; BATCH] = core::array::from_fn(|_| &mut [][..]);
        for (slot, b) in tmp.iter_mut().zip(batch) {
            *slot = unsafe { b.uninit_slice_mut() };
        }
        tmp
    }

    #[inline(always)]
    pub unsafe fn build_recv_slices_from_slice<'a, const BATCH: usize>(batch: &'a mut [Self]) -> [&'a mut [u8]; BATCH] {
        assert_eq!(batch.len(), BATCH);
        unsafe { Self::build_recv_slices(batch) }
    }

    #[inline(always)]
    pub fn build_send_slices<'a, const BATCH: usize>(batch: impl IntoIterator<Item = &'a Self>) -> [&'a [u8]; BATCH] {
        let mut tmp: [&'a [u8]; BATCH] = [&[]; BATCH];
        for (slot, b) in tmp.iter_mut().zip(batch) {
            *slot = b.as_ref();
        }
        tmp
    }
}

impl<const N: usize, const PAD: usize> AsRef<[u8]> for AlignedOffsetBuf<N, PAD> {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        unsafe { &self.buf.assume_init_ref()[..self.written] }
    }
}

impl<const N: usize, const PAD: usize> AsMut<[u8]> for AlignedOffsetBuf<N, PAD> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { &mut self.buf.assume_init_mut()[..self.written] }
    }
}

pub const fn calc_buffer_pad(offset: usize) -> usize {
    const ALIGN: usize = 64;

    let rem = offset % ALIGN;
    if rem == 0 { 0 } else { ALIGN - rem }
}

#[cfg(test)]
mod tests {
    use crate::{AlignedOffsetBuf, calc_buffer_pad};

    #[test]
    fn test_align() {
        type CryptoBuf<const N: usize> = AlignedOffsetBuf<N, { calc_buffer_pad(12) }>;

        let a = CryptoBuf::<1500>::new();
        unsafe {
            assert!(a.as_ref().as_ptr().is_aligned());
            assert!((a.as_ptr().add(12) as usize).is_multiple_of(64));
            assert!(!(a.as_ptr().add(13) as usize).is_multiple_of(64));
        }
    }

    #[test]
    fn test_writes() {
        type CryptoBuf<const N: usize> = AlignedOffsetBuf<N, { calc_buffer_pad(12) }>;

        let mut a = CryptoBuf::<1500>::new();
        assert_eq!(a.as_ref().len(), 0);
        assert_eq!(a.as_mut().len(), 0);

        unsafe {
            a.assume_written(10);
        }

        assert_eq!(a.as_ref().len(), 10);
        assert_eq!(a.as_mut().len(), 10);

        unsafe {
            a.assume_written(1500);
        }

        assert_eq!(a.as_ref().len(), 1500);
        assert_eq!(a.as_mut().len(), 1500);

        let slice = a.as_mut();
        slice[0] = 0x42;
        assert_eq!(slice[0], 0x42);
        slice[1499] = 0x43;
        assert_eq!(slice[1499], 0x43);
    }

    #[test]
    fn test_overflow_boundary() {
        type CryptoBuf<const N: usize> = AlignedOffsetBuf<N, { calc_buffer_pad(12) }>;

        let mut a = CryptoBuf::<1500>::new();

        unsafe {
            a.assume_written(1500);
        }
    }

    #[test]
    #[should_panic]
    fn test_overflow_panic() {
        type CryptoBuf<const N: usize> = AlignedOffsetBuf<N, { calc_buffer_pad(12) }>;

        let mut a = CryptoBuf::<1500>::new();

        unsafe {
            a.assume_written(1501);
        }
    }

    #[test]
    fn test_asref_asmut() {
        type CryptoBuf<const N: usize> = AlignedOffsetBuf<N, { calc_buffer_pad(12) }>;

        let mut a = CryptoBuf::<1500>::new();
        assert_eq!(a.as_ref().len(), 0);
        assert_eq!(a.as_mut().len(), 0);

        unsafe {
            a.assume_written(10);
        }

        assert_eq!(a.as_ref().len(), 10);
        assert_eq!(a.as_mut().len(), 10);
    }

    #[test]
    fn test_batch_init() {
        type CryptoBuf<const N: usize> = AlignedOffsetBuf<N, { calc_buffer_pad(12) }>;

        let _batch1 = CryptoBuf::<1500>::new_batch::<16>();
        let mut _batch2 = CryptoBuf::<1500>::new_batch_zeroed::<16>();
        let _bufs = unsafe { AlignedOffsetBuf::build_recv_slices::<16>(&mut _batch2) };
        assert_eq!(_bufs[0].len(), 1500);
        assert_eq!(_bufs.len(), 16);
    }

    #[test]
    fn test_batch_recv_partial_iterator() {
        type CryptoBuf = AlignedOffsetBuf<1500, { calc_buffer_pad(12) }>;

        let mut batch = CryptoBuf::new_batch_zeroed::<2>();
        let slices = unsafe { CryptoBuf::build_recv_slices::<16>(batch.iter_mut().take(1)) };
        assert_eq!(slices[0].len(), 1500);
        assert!(slices[1..].iter().all(|slice| slice.is_empty()));
        slices[0][123] = 33;
        unsafe { batch[0].assume_written(124) };
        assert_eq!(batch[0].as_ref()[123], 33);
    }

    #[test]
    fn test_batch_send_init() {
        type CryptoBuf<const N: usize> = AlignedOffsetBuf<N, { calc_buffer_pad(12) }>;

        let mut _batch = CryptoBuf::<1500>::new_batch::<16>();
        {
            let _bufs = unsafe { AlignedOffsetBuf::build_recv_slices::<16>(&mut _batch) };
            _bufs[0][123] = 33;
        }
        unsafe { _batch[0].assume_written(124) }
        {
            let _bufs = AlignedOffsetBuf::build_send_slices::<16>(&_batch);
            assert_eq!(_bufs[0].len(), 124);
            assert_eq!(_bufs[0][123], 33);
        }
        let slices = AlignedOffsetBuf::build_send_slices::<16>(_batch.iter().take(1));
        assert_eq!(slices[0].len(), 124);
        assert_eq!(slices[0][123], 33);
        assert!(slices[1..].iter().all(|slice| slice.is_empty()));
    }
}
