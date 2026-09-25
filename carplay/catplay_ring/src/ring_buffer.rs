use std::{
    cell::UnsafeCell,
    fmt::{self, Debug},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use crate::{
    Bitmap,
    ring_storage::{RingStorage, mmap_aligned_capacity},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RingSegment<'a, T> {
    // Data properly read from the ring that has also advanced the timeline.
    Data(&'a [T]),

    /// Timeline jump - in an audio context this informs that samples for given time window
    /// were unable to be produced, and samples that will follow intend to stay on an accurate timeline.
    ///
    /// This can represent a different operation than forcefully writing silence samples multiplied by gap size to the ring.
    ///
    /// Note that adding timeline jumps to the ring larger than `capacity` will turn into overflow segments for the remainder.
    Gap(usize),

    /// Ring overflow - when the ring is too small to properly operate or reader and writer are too far ahead, and some samples were lost.
    ///
    /// This is basically a protection against playing "trash" data and a signal to play silence for N samples - when used in an audio context.
    Overflow(usize),

    /// Underflow - there wasn't enough data at the end of the ring to satisfy last N samples.
    ///
    /// The timeline IS still advanced by N samples. In an audio context, silence should be played,
    /// and when writer later catches up, the missing samples are intentionally dropped to keep
    /// wall-clock accurate playback.
    ///
    /// Note that reader can request a "short read" instead, in which case no Underflow segment will be generated, and the timeline will not be advanced by the underflow during that read call.
    Underflow(usize),
}

impl<'a, T> RingSegment<'a, T> {
    pub fn len(&self) -> usize {
        match self {
            RingSegment::Data(d) => d.len(),
            RingSegment::Gap(n) => *n,
            RingSegment::Overflow(n) => *n,
            RingSegment::Underflow(n) => *n,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_data(&self) -> bool {
        matches!(self, RingSegment::Data(_))
    }
}

impl<'a, T> fmt::Display for RingSegment<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RingSegment::Data(d) => write!(f, "Data({})", d.len()),
            RingSegment::Gap(n) => write!(f, "Gap({n})"),
            RingSegment::Overflow(n) => write!(f, "Overflow({n})"),
            RingSegment::Underflow(n) => write!(f, "Underflow({n})"),
        }
    }
}

/// Single-producer single-consumer (SPSC) mirrored ring buffer designed for audio streams.
///
/// The buffer preserves a **monotonic audio timeline** by treating `write_head` as
/// logical time (the number of samples that have passed through the system), while
/// physically storing only the last `capacity` samples of that timeline.
///
/// Overrun (data loss) is **not tracked explicitly**. Instead, the exact number of
/// lost samples is derived from the monotonic head distance:
///
/// ```text
/// lost = max(0, write_head - read_head - capacity)
/// ```
///
/// This allows precise reporting of XRUN conditions (overflow / underflow) without
/// breaking timeline continuity or requiring background maintenance tasks.
///
/// Loss is observed by the consumer when logical time must be advanced.
///
/// The gap is consumed progressively and deterministically:
/// regardless of how the consumer chunks reads or the size of the output buffers,
/// the same logical gap is emitted exactly once before audio data resumes.
///
/// Underflow advances time. Samples written later for that time are obsolete
/// and must be ignored to preserve a monotonic playback timeline.
///
/// The caller may zero-fill or otherwise handle missing portions of the output buffer
/// based on the offsets reported in [`XRunReport`].
pub struct RingBuffer<T: Copy + Default> {
    ring: UnsafeCell<RingStorage<T>>,

    capacity: usize,
    write_head: AtomicUsize, // producer only writes
    read_head: AtomicUsize,  // consumer only writes
    mask: usize,

    gaps: UnsafeCell<Bitmap>,
}

unsafe impl<T: Copy + Default> Sync for RingBuffer<T> {}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct XRunReport {
    // Samples that should have been read *before* valid data,
    // but were missing (read pointer had to skip ahead).
    pub overflow: usize,

    /// Samples that were actually copied from the ring buffer.
    pub copied: usize,

    /// Samples that were requested but could not be read
    /// because the buffer ran dry.
    pub underflow: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum WriteReport {
    Ok,
    Partial { written: usize },
}

impl XRunReport {
    pub fn zero_fill_overflow_and_underflow<T: Zeroable>(&self, out: &mut [T]) {
        fn fill_slice<T>(buf: &mut [T]) {
            unsafe {
                std::ptr::write_bytes(buf.as_mut_ptr(), 0, buf.len());
            }
        }

        fill_slice(self.overflow_slice_mut(out));
        fill_slice(self.underflow_slice_mut(out));
    }

    pub fn overflow_slice<'a, T>(&self, data: &'a [T]) -> &'a [T] {
        &data[..self.overflow]
    }

    pub fn overflow_slice_mut<'a, T>(&self, data: &'a mut [T]) -> &'a mut [T] {
        &mut data[..self.overflow]
    }

    pub fn underflow_slice<'a, T>(&self, data: &'a [T]) -> &'a [T] {
        &data[self.overflow + self.copied..self.overflow + self.copied + self.underflow]
    }

    pub fn underflow_slice_mut<'a, T>(&self, data: &'a mut [T]) -> &'a mut [T] {
        &mut data[self.overflow + self.copied..self.overflow + self.copied + self.underflow]
    }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe trait Zeroable: Default + Send + Copy + Debug + 'static {}
unsafe impl Zeroable for u8 {}
unsafe impl Zeroable for i8 {}
unsafe impl Zeroable for i16 {}
unsafe impl Zeroable for i32 {}
unsafe impl Zeroable for f32 {}

pub struct RingProducer<T: Copy + Default>(Arc<RingBuffer<T>>);
pub struct RingConsumer<T: Copy + Default>(Arc<RingBuffer<T>>);

#[derive(Debug, Clone, Copy)]
pub struct RingStats {
    pub capacity: usize,
    pub written: usize,
    pub consumed: usize,
    pub readable: usize,
    pub overflow: usize,
    pub underflow: usize,
}

impl RingStats {
    pub fn as_audio_stats(&self, sample_rate: usize, frame_size_in_samples: usize, start: Instant) -> RingStatsAudio {
        pub fn samples_to_ms(samples: usize, sample_rate: usize, frame_size_in_samples: usize) -> Duration {
            let total_nanos = (samples as u128) * 1_000_000_000u128 / (sample_rate as u128 * frame_size_in_samples as u128);
            Duration::from_nanos(total_nanos as u64)
        }

        RingStatsAudio {
            capacity: samples_to_ms(self.capacity, sample_rate as _, frame_size_in_samples as _),
            written: samples_to_ms(self.written, sample_rate as _, frame_size_in_samples as _),
            consumed: samples_to_ms(self.consumed, sample_rate as _, frame_size_in_samples as _),
            readable: samples_to_ms(self.readable, sample_rate as _, frame_size_in_samples as _),
            overflow: samples_to_ms(self.overflow, sample_rate as _, frame_size_in_samples as _),
            underflow: samples_to_ms(self.underflow, sample_rate as _, frame_size_in_samples as _),

            start_ago: Instant::now() - start,
            consumption_behind: (Instant::now() - start).saturating_sub(samples_to_ms(
                self.consumed,
                sample_rate as _,
                frame_size_in_samples as _,
            )),
            consumption_ahead: samples_to_ms(self.consumed, sample_rate as _, frame_size_in_samples as _)
                .saturating_sub(Instant::now() - start),
        }
    }
}

impl fmt::Display for RingStatsAudio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "consumed={:?}/{:?} buffer={:?}/{:?} overflow={:?} underflow={:?} cursor={}{:?}",
            self.consumed,
            self.written,
            self.readable,
            self.capacity,
            self.overflow,
            self.underflow,
            if !self.consumption_behind.is_zero() { "-" } else { "" },
            if !self.consumption_behind.is_zero() {
                self.consumption_behind
            } else {
                self.consumption_ahead
            }
        ))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RingStatsAudio {
    pub capacity: Duration,
    pub written: Duration,
    pub consumed: Duration,
    pub readable: Duration,
    pub overflow: Duration,
    pub underflow: Duration,

    pub start_ago: Duration,
    pub consumption_behind: Duration,
    pub consumption_ahead: Duration,
}

impl<T: Copy + Default> RingProducer<T> {
    pub fn stat(&mut self) -> RingStats {
        self.0.stat()
    }

    pub fn capacity(&mut self) -> usize {
        self.0.capacity()
    }

    pub fn peek_overflow(&mut self) -> usize {
        self.0.peek_overflow()
    }

    pub fn write(&mut self, input: &[T]) -> WriteReport {
        self.0.write(input)
    }

    pub fn write_default(&mut self, repeat: usize) {
        self.0.write_default(repeat);
    }

    pub fn writable_slice(&self) -> &mut [T] {
        self.0.writable_slice()
    }

    pub fn write_commit(&self, written: usize) {
        self.0.write_commit(written);
    }
}

impl<T: Copy + Default> RingConsumer<T> {
    pub fn stat(&mut self) -> RingStats {
        self.0.stat()
    }

    pub fn capacity(&mut self) -> usize {
        self.0.capacity()
    }

    pub fn read(&mut self, out: &mut [T]) -> Result<(), XRunReport> {
        self.0.read(out)
    }

    pub fn readable_slice(&self) -> (usize, &[T]) {
        self.0.readable_slice()
    }

    pub fn read_commit(&self, n: usize) {
        self.0.read_commit(n)
    }

    pub fn read_segments<'a, F>(&'a self, requested: usize, emit_underflow: bool, emit: F)
    where
        F: FnMut(RingSegment<'a, T>),
    {
        self.0.read_segments(requested, emit_underflow, emit)
    }
}

impl<T: Copy + Default> RingBuffer<T> {
    #[inline]
    fn logical_to_ring_index(&self, logical: usize) -> usize {
        logical & self.mask
    }

    fn new_unsplit(size: usize) -> Self {
        let size: usize = size.next_power_of_two().max(1);
        assert!(size > 0);

        Self {
            ring: UnsafeCell::new(RingStorage::new(size)),
            write_head: AtomicUsize::new(0),
            read_head: AtomicUsize::new(0),

            gaps: UnsafeCell::new(Bitmap::new(size)),
            capacity: size,
            mask: size - 1,
        }
    }

    pub fn read_segments<'a, F>(&'a self, requested: usize, emit_underflow: bool, mut emit: F)
    where
        F: FnMut(RingSegment<'a, T>),
    {
        if requested == 0 {
            return;
        }

        let read = self.read_head.load(Ordering::Relaxed);
        let write = self.write_head.load(Ordering::Acquire);

        let lost = write.saturating_sub(self.capacity).saturating_sub(read);
        let overflow = lost.min(requested);

        if overflow > 0 {
            emit(RingSegment::Overflow(overflow));
        }

        let cursor = read + overflow;
        let mut remaining = requested - overflow;

        // Timeline available after applying overflow; bounded by ring capacity.
        let available_timeline = write.saturating_sub(cursor).min(self.capacity);
        let readable = available_timeline.min(remaining);

        let gaps = unsafe { &mut *self.gaps.get() };
        let ring = unsafe { (&*self.ring.get()).as_slice() };
        let cap = self.capacity;

        // Search gap in logical timeline [start_abs, start_abs + len), handling wrap across bitmap boundary.
        let mut search_gap_abs = |start_abs: usize, len: usize| -> Option<(usize, usize)> {
            if len == 0 {
                return None;
            }

            let start = self.logical_to_ring_index(start_abs);
            let first_len = len.min(cap - start);

            if let Some((gap_start_mod, mut gap_len)) = gaps.search_next_gap(start, first_len) {
                let abs_start = start_abs + (gap_start_mod - start);

                // Merge contiguous run if gap starts near end and continues at bitmap index 0.
                if gap_start_mod + gap_len == cap
                    && len > first_len
                    && let Some((gap2_start, gap2_len)) = gaps.search_next_gap(0, len - first_len)
                    && gap2_start == 0
                {
                    gap_len += gap2_len;
                }

                return Some((abs_start, gap_len));
            }

            if len > first_len
                && let Some((gap_start_mod, gap_len)) = gaps.search_next_gap(0, len - first_len)
            {
                let abs_start = start_abs + first_len + gap_start_mod;
                return Some((abs_start, gap_len));
            }

            None
        };

        let mut processed = 0usize;
        while processed < readable {
            let span_len = readable - processed;
            let abs = cursor + processed;

            if let Some((gap_start, gap_len)) = search_gap_abs(abs, span_len) {
                if gap_start > abs {
                    let data_len = gap_start - abs;
                    let head = self.logical_to_ring_index(abs);
                    emit(RingSegment::Data(&ring[head..head + data_len]));

                    processed += data_len;
                    remaining -= data_len;
                    continue;
                }

                let take = gap_len.min(span_len);
                emit(RingSegment::Gap(take));
                processed += take;
                remaining -= take;
            } else {
                let data_len = span_len;
                let head = self.logical_to_ring_index(abs);
                emit(RingSegment::Data(&ring[head..head + data_len]));

                processed += data_len;
                remaining -= data_len;
            }
        }

        // Everything that remains is a true underflow.
        if remaining > 0 && emit_underflow {
            emit(RingSegment::Underflow(remaining));
        }

        let advance = overflow + processed + if emit_underflow { remaining } else { 0 };

        if advance > 0 {
            self.read_commit(advance);
        }
    }

    pub fn advance_timeline(&self, size: usize) {
        let cap = self.capacity;
        if size == 0 {
            return;
        }

        let mark = size.min(cap);

        let write_head = self.write_head.load(Ordering::Relaxed);
        let start = self.logical_to_ring_index(write_head);

        let gaps = unsafe { &mut *self.gaps.get() };
        let first_len = (cap - start).min(mark);
        let second_len = mark - first_len;

        gaps.mark_region(start, first_len);
        gaps.mark_region(0, second_len);

        self.write_commit_advance(0, size);
    }

    pub fn spsc(size: usize) -> (RingProducer<T>, RingConsumer<T>) {
        let mut size = size.max(1).next_power_of_two();

        #[cfg(unix)]
        if let Some(aligned) = mmap_aligned_capacity::<T>(size) {
            size = aligned;
        }

        Self::new_unsplit(size).split()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn write_default(&self, repeat: usize) {
        if repeat == 0 {
            return;
        }

        let dst = self.writable_slice();
        let written_phys = repeat.min(dst.len());
        dst[..written_phys].fill(T::default());
        self.write_commit_advance(written_phys, repeat);
    }

    pub fn readable_slice(&self) -> (usize, &[T]) {
        let read = self.read_head.load(Ordering::Relaxed);
        let write = self.write_head.load(Ordering::Acquire);

        let lost = write.saturating_sub(self.capacity).saturating_sub(read);

        let overflow = lost;

        let available = write.saturating_sub(read + overflow);

        if available == 0 {
            return (overflow, &[]);
        }

        let to_read = available.min(self.capacity);

        let head = self.logical_to_ring_index(read + overflow);

        unsafe {
            let ring = (&*self.ring.get()).as_slice();
            let slice = &ring[head..head + to_read];
            (overflow, slice)
        }
    }

    pub fn read(&self, out: &mut [T]) -> Result<(), XRunReport> {
        let requested = out.len();
        if requested == 0 {
            return Ok(());
        }

        let mut copied = 0usize;
        let mut overflow = 0usize;
        let mut underflow = 0usize;
        let mut cursor = 0usize;

        self.read_segments(requested, true, |seg| match seg {
            RingSegment::Data(data) => {
                let len = data.len();
                out[cursor..cursor + len].copy_from_slice(data);
                copied += len;
                cursor += len;
            }
            RingSegment::Overflow(n) => {
                overflow += n;
                cursor += n;
            }
            RingSegment::Gap(n) | RingSegment::Underflow(n) => {
                // `read()` reports timeline holes as underflow so callers can fill silence.
                underflow += n;
                cursor += n;
            }
        });

        #[cfg(test)]
        {
            debug_assert_eq!(overflow + copied + underflow, requested);
            debug_assert_eq!(cursor, requested);
        }

        if overflow == 0 && underflow == 0 {
            Ok(())
        } else {
            Err(XRunReport {
                copied,
                overflow,
                underflow,
            })
        }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn writable_slice(&self) -> &mut [T] {
        let cap = self.capacity;
        let write = self.write_head.load(Ordering::Relaxed);
        let head = self.logical_to_ring_index(write);

        unsafe {
            let ring = &mut *self.ring.get();
            &mut ring.as_mut_slice()[head..head + cap]
        }
    }

    #[inline]
    pub fn write_commit(&self, written: usize) {
        self.write_commit_advance(written, written)
    }

    pub fn write_commit_advance(&self, written_phys: usize, advance: usize) {
        let cap = self.capacity;
        debug_assert!(written_phys <= cap);

        let old = self.write_head.load(Ordering::Relaxed);
        let head = self.logical_to_ring_index(old);

        if written_phys > 0 {
            unsafe {
                let ring = &mut *self.ring.get();
                ring.sync_mirror_range(cap, head, written_phys);
            }
        }

        let first_len = (cap - head).min(written_phys);
        let second_len = written_phys - first_len;
        let gaps = unsafe { &mut *self.gaps.get() };
        gaps.unmark_region(head, first_len);
        gaps.unmark_region(0, second_len);

        let _old_head = self.write_head.fetch_add(advance, Ordering::Release);
    }

    pub fn read_commit(&self, read: usize) {
        if read == 0 {
            return;
        }

        let _old_head = self.read_head.fetch_add(read, Ordering::Release);
    }

    pub fn write(&self, input: &[T]) -> WriteReport {
        let cap = self.capacity;
        let total = input.len();
        if total == 0 {
            return WriteReport::Ok;
        }

        let phys = total.min(cap);
        let src = &input[total - phys..];

        let dst = self.writable_slice();
        dst[..phys].copy_from_slice(src);

        self.write_commit_advance(phys, total);

        if phys < total {
            WriteReport::Partial { written: phys }
        } else {
            WriteReport::Ok
        }
    }

    pub fn stat(&self) -> RingStats {
        let consumed = self.read_head.load(Ordering::Acquire);
        let written = self.write_head.load(Ordering::Relaxed);

        let readable = written.saturating_sub(consumed);
        let capacity = self.capacity;

        let overflow = written.saturating_sub(capacity).saturating_sub(consumed);
        let underflow = consumed.saturating_sub(written);

        RingStats {
            consumed,
            written,

            readable,

            capacity,

            overflow,
            underflow,
        }
    }

    pub fn total_consumed(&self) -> usize {
        self.read_head.load(Ordering::Relaxed)
    }

    pub fn total_written(&self) -> usize {
        self.write_head.load(Ordering::Relaxed)
    }

    pub fn peek_overflow(&self) -> usize {
        let read = self.read_head.load(Ordering::Relaxed);
        let write = self.write_head.load(Ordering::Acquire);

        write.saturating_sub(self.capacity).saturating_sub(read)
    }

    pub fn split(self) -> (RingProducer<T>, RingConsumer<T>) {
        let arc = Arc::new(self);
        (RingProducer(arc.clone()), RingConsumer(arc))
    }

    #[cfg(all(test, unix))]
    fn uses_kernel_mirror(&self) -> bool {
        matches!(unsafe { &*self.ring.get() }, RingStorage::MirroredMmap(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect_segments<'a, T: Zeroable>(rb: &'a RingBuffer<T>, requested: usize) -> Vec<RingSegment<'a, T>> {
        let mut segs = Vec::new();

        rb.read_segments(requested, true, |seg| {
            segs.push(seg);
        });

        #[cfg(debug_assertions)]
        {
            let consumed: usize = segs
                .iter()
                .map(|s| match s {
                    RingSegment::Data(d) => d.len(),
                    RingSegment::Gap(n) => *n,
                    RingSegment::Overflow(n) => *n,
                    RingSegment::Underflow(n) => *n,
                })
                .sum();

            debug_assert_eq!(
                consumed, requested,
                "collect_segments: timeline mismatch: consumed={}, requested={}, segs={:?}",
                consumed, requested, segs
            );
        }

        segs
    }

    fn collect_segments_no_underflow<'a, T: Zeroable>(rb: &'a RingBuffer<T>, requested: usize) -> Vec<RingSegment<'a, T>> {
        let mut segs = Vec::new();
        rb.read_segments(requested, false, |seg| segs.push(seg));
        segs
    }

    #[test]
    fn test_basic_read_write_segments() {
        let rb = RingBuffer::<i16>::new_unsplit(8);

        let input = [1, 2, 3, 4];
        assert_eq!(rb.write(&input), WriteReport::Ok);
        assert_eq!(collect_segments(&rb, 4), vec![RingSegment::Data(&[1, 2, 3, 4])]);
    }

    #[test]
    fn test_read_write_exact_capacity_segments() {
        let rb = RingBuffer::<i16>::new_unsplit(8);

        let input = [1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(rb.write(&input), WriteReport::Ok);
        assert_eq!(collect_segments(&rb, 8), vec![RingSegment::Data(&[1, 2, 3, 4, 5, 6, 7, 8])]);
    }
    #[test]
    fn test_underflow_segments() {
        let rb = RingBuffer::<i16>::new_unsplit(8);

        assert_eq!(rb.write(&[1, 2, 3, 4, 5]), WriteReport::Ok);
        assert_eq!(collect_segments(&rb, 3), vec![RingSegment::Data(&[1, 2, 3])]);
        assert_eq!(
            collect_segments(&rb, 4),
            vec![RingSegment::Data(&[4, 5]), RingSegment::Underflow(2)]
        );

        assert_eq!(rb.write(&[9, 8, 7]), WriteReport::Ok);
        assert_eq!(collect_segments(&rb, 1), vec![RingSegment::Data(&[7])]);
    }

    #[test]
    fn test_overflow_multiple_wraps_segments() {
        let rb = RingBuffer::<i16>::new_unsplit(2);

        assert_eq!(rb.write(&[1, 2, 3, 4, 5, 6, 7, 8]), WriteReport::Partial { written: 2 });
        assert_eq!(collect_segments(&rb, 8), vec![RingSegment::Overflow(6), RingSegment::Data(&[7, 8])]);
    }

    #[test]
    fn test_overflow_multiple_wraps_split_read_segments() {
        let rb = RingBuffer::<i16>::new_unsplit(2);

        assert_eq!(rb.write(&[1, 2, 3, 4, 5, 6, 7, 8]), WriteReport::Partial { written: 2 });
        assert_eq!(collect_segments(&rb, 4), vec![RingSegment::Overflow(4)]);
        assert_eq!(collect_segments(&rb, 4), vec![RingSegment::Overflow(2), RingSegment::Data(&[7, 8])]);
    }

    #[test]
    fn test_overflow_underflow_mix_segments() {
        let rb = RingBuffer::<i16>::new_unsplit(2);

        assert_eq!(rb.write(&[1, 2, 3, 4, 5, 6, 7, 8]), WriteReport::Partial { written: 2 });
        assert_eq!(collect_segments(&rb, 4), vec![RingSegment::Overflow(4)]);
        assert_eq!(
            collect_segments(&rb, 5),
            vec![RingSegment::Overflow(2), RingSegment::Data(&[7, 8]), RingSegment::Underflow(1)]
        );
        // [1] goes into overflow debt repayment
        assert_eq!(rb.write(&[1, 2]), WriteReport::Ok);
        assert_eq!(collect_segments(&rb, 1), vec![RingSegment::Data(&[2])]);
    }

    #[test]
    fn empty_read_returns_zero_segments() {
        let rb = RingBuffer::<i16>::new_unsplit(4);

        assert_eq!(collect_segments(&rb, 0), Vec::<RingSegment<'_, i16>>::new());
    }
    #[test]
    fn test_capacity_helpers() {
        let rb = RingBuffer::<i16>::new_unsplit(8);

        rb.write(&[1, 2]);
        assert_eq!(rb.capacity(), 8);
    }

    #[test]
    fn test_zero_fill_with_underflow() {
        let rb = RingBuffer::<i16>::new_unsplit(2);
        assert_eq!(rb.write(&[1, 2, 3, 4, 5, 6, 7, 8]), WriteReport::Partial { written: 2 });

        let mut out = [0; 4];
        assert_eq!(
            rb.read(&mut out).unwrap_err(),
            XRunReport {
                copied: 0,
                underflow: 0,
                overflow: 4
            }
        );
        assert_eq!(out, [0, 0, 0, 0]);

        let mut out = [9; 7];
        let report = XRunReport {
            copied: 2,
            underflow: 3,
            overflow: 2,
        };
        assert_eq!(rb.read(&mut out).unwrap_err(), report);
        assert_eq!(out, [9, 9, 7, 8, 9, 9, 9]);

        report.zero_fill_overflow_and_underflow(&mut out);
        assert_eq!(&out[..], &[0, 0, 7, 8, 0, 0, 0]);
    }

    #[test]
    fn test_read_maps_gap_to_underflow_for_zero_fill() {
        let rb = RingBuffer::<i16>::new_unsplit(8);
        rb.write(&[1, 2, 3, 4]);
        rb.advance_timeline(3);

        let mut out = [9; 7];
        let report = rb.read(&mut out).unwrap_err();

        assert_eq!(
            report,
            XRunReport {
                copied: 4,
                overflow: 0,
                underflow: 3,
            }
        );
        assert_eq!(out, [1, 2, 3, 4, 9, 9, 9]);

        report.zero_fill_overflow_and_underflow(&mut out);
        assert_eq!(out, [1, 2, 3, 4, 0, 0, 0]);
    }

    #[test]
    fn test_total_consumed_written_segments() {
        let rb = RingBuffer::<i16>::new_unsplit(2);

        assert_eq!(rb.write(&[1, 2, 3, 4]), WriteReport::Partial { written: 2 });

        assert_eq!(
            collect_segments(&rb, 5),
            vec![RingSegment::Overflow(2), RingSegment::Data(&[3, 4]), RingSegment::Underflow(1)]
        );

        assert_eq!(rb.total_consumed(), 5);
        assert_eq!(rb.total_written(), 4);
    }

    #[test]
    fn test_write_default_segments() {
        const COPY_SIZE: usize = 2050;

        let rb = RingBuffer::<i16>::new_unsplit(4096);
        rb.write_default(COPY_SIZE);

        assert_eq!(
            collect_segments(&rb, COPY_SIZE + 1),
            vec![RingSegment::Data(&[0; COPY_SIZE]), RingSegment::Underflow(1)]
        );
    }

    #[test]
    fn test_write_default_wrap_segments() {
        const CAP: usize = 1024;
        const COPY_SIZE: usize = CAP + 100;

        let rb = RingBuffer::<i16>::new_unsplit(CAP);
        rb.write_default(COPY_SIZE);

        assert_eq!(
            collect_segments(&rb, COPY_SIZE),
            vec![RingSegment::Overflow(100), RingSegment::Data(&[0; CAP])]
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_kernel_mirror_wraps_contiguously() {
        const CAP: usize = 2048;

        let rb = RingBuffer::<i16>::new_unsplit(CAP);
        assert!(rb.uses_kernel_mirror());

        let prefix = vec![11; CAP - 2];
        assert_eq!(rb.write(&prefix), WriteReport::Ok);

        let mut discard = vec![0; CAP - 2];
        assert_eq!(rb.read(&mut discard), Ok(()));

        assert_eq!(rb.write(&[1, 2, 3, 4]), WriteReport::Ok);
        assert_eq!(collect_segments(&rb, 4), vec![RingSegment::Data(&[1, 2, 3, 4])]);
    }

    #[cfg(unix)]
    #[test]
    fn test_spsc_rounds_capacity_for_kernel_mirror() {
        let (mut producer, _consumer) = RingBuffer::<i16>::spsc(8);

        assert_eq!(producer.capacity(), 2048);
        assert!(producer.0.uses_kernel_mirror());
    }

    #[test]
    fn test_advance_timeline_produces_gap_segments() {
        let rb = RingBuffer::<i16>::new_unsplit(8);

        rb.write(&[1, 2, 3, 4]);
        rb.advance_timeline(3);

        assert_eq!(
            collect_segments(&rb, 6),
            vec![RingSegment::Data(&[1, 2, 3, 4]), RingSegment::Gap(2),]
        );
        assert_eq!(collect_segments(&rb, 2), vec![RingSegment::Gap(1), RingSegment::Underflow(1)]);
    }

    #[test]
    fn test_advance_timeline_produces_gap_middle() {
        let rb = RingBuffer::<i16>::new_unsplit(16);

        rb.write(&[1, 2, 3, 4]);
        rb.advance_timeline(3);
        rb.write(&[5]);
        rb.advance_timeline(2);
        rb.write(&[6, 7]);

        assert_eq!(
            collect_segments(&rb, 12),
            vec![
                RingSegment::Data(&[1, 2, 3, 4]),
                RingSegment::Gap(3),
                RingSegment::Data(&[5]),
                RingSegment::Gap(2),
                RingSegment::Data(&[6, 7]),
            ]
        );
    }

    #[test]
    fn test_gap_sequence_is_stable_across_split_reads() {
        let rb = RingBuffer::<i16>::new_unsplit(16);

        rb.write(&[1, 2, 3, 4]);
        rb.advance_timeline(3);
        rb.write(&[5]);
        rb.advance_timeline(2);
        rb.write(&[6, 7]);

        assert_eq!(
            collect_segments(&rb, 5),
            vec![RingSegment::Data(&[1, 2, 3, 4]), RingSegment::Gap(1),]
        );

        assert_eq!(
            collect_segments(&rb, 7),
            vec![
                RingSegment::Gap(2),
                RingSegment::Data(&[5]),
                RingSegment::Gap(2),
                RingSegment::Data(&[6, 7]),
            ]
        );
    }

    #[test]
    fn test_advance_timeline_larger_than_capacity_turns_into_overflow_and_gap() {
        let rb = RingBuffer::<i16>::new_unsplit(8);

        rb.advance_timeline(20);

        assert_eq!(collect_segments(&rb, 20), vec![RingSegment::Overflow(12), RingSegment::Gap(8),]);
    }

    #[test]
    fn test_read_segments_without_underflow_emission_does_not_advance_missing_timeline() {
        let rb = RingBuffer::<i16>::new_unsplit(8);

        rb.write(&[1, 2]);
        assert_eq!(collect_segments_no_underflow(&rb, 4), vec![RingSegment::Data(&[1, 2])]);
        assert_eq!(rb.total_consumed(), 2);

        rb.write(&[3, 4]);
        assert_eq!(collect_segments(&rb, 2), vec![RingSegment::Data(&[3, 4])]);
    }

    #[test]
    fn test_gap_read_after_timeline_wrap() {
        let rb = RingBuffer::<i16>::new_unsplit(8);

        rb.write(&[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(collect_segments(&rb, 8), vec![RingSegment::Data(&[1, 2, 3, 4, 5, 6, 7, 8]),]);

        rb.advance_timeline(2);
        rb.write(&[9, 10, 11, 12, 13, 14]);

        assert_eq!(
            collect_segments(&rb, 8),
            vec![RingSegment::Gap(2), RingSegment::Data(&[9, 10, 11, 12, 13, 14]),]
        );
    }

    #[test]
    fn test_overwritten_gap_bits_do_not_create_phantom_gaps() {
        let rb = RingBuffer::<i16>::new_unsplit(8);

        rb.write(&[1, 2, 3, 4]);
        rb.advance_timeline(2);
        rb.write(&[5, 6]);
        assert_eq!(
            collect_segments(&rb, 8),
            vec![RingSegment::Data(&[1, 2, 3, 4]), RingSegment::Gap(2), RingSegment::Data(&[5, 6]),]
        );

        rb.write(&[7, 8, 9, 10, 11, 12, 13, 14]);
        assert_eq!(collect_segments(&rb, 8), vec![RingSegment::Data(&[7, 8, 9, 10, 11, 12, 13, 14]),]);
    }
}
