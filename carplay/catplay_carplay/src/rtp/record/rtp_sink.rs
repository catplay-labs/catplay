use std::num::NonZeroUsize;

use log::{debug, trace, warn};

use crate::audio::{AudioSink, RingProducer, WriteReport, Zeroable};

pub struct RtpSink<S: Zeroable> {
    ring: RingProducer<S>,
    samples_per_packet: NonZeroUsize,
    written_samples: usize,
    notify: Box<dyn FnMut() + Send>,
}

impl<S: Zeroable> RtpSink<S> {
    pub fn new(ring: RingProducer<S>, samples_per_packet: NonZeroUsize, notify: impl FnMut() + Send + 'static) -> Self {
        Self {
            ring,
            samples_per_packet,
            notify: Box::new(notify),
            written_samples: 0,
        }
    }
}

impl<S: Zeroable> AudioSink for RtpSink<S> {
    type Sample = S;

    fn write(&mut self, slice: &[Self::Sample]) {
        let written = match self.ring.write(slice) {
            WriteReport::Ok => slice.len(),
            WriteReport::Partial { written } => written,
        };

        if written < slice.len() {
            warn!("RTP recorder: partial write {written}/{}", slice.len());
        } else {
            debug!("RTP recorder: {written} samples");
        }

        let before = self.written_samples;
        self.written_samples += written;

        if before / self.samples_per_packet < self.written_samples / self.samples_per_packet {
            trace!("RTP recorder: Waking up encoder");
            (self.notify)();
        }
    }

    fn writable(&mut self) -> usize {
        // writable_slice exposes an overwrite window, not free capacity. Use the
        // logical backlog, including overflow that the reader has not committed.
        // stat acquires read_head; this sole producer's write_head is stable here.
        // A concurrent consumer can only free more space, so a stale snapshot is
        // conservative and needs no additional lock or ring API changes.
        let stat = self.ring.stat();
        stat.capacity.saturating_sub(stat.written.saturating_sub(stat.consumed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::RingBuffer;

    #[test]
    fn writable_tracks_full_partial_and_drained_ring() {
        let (ring, mut consumer) = RingBuffer::<i16>::spsc(1024);
        let capacity = consumer.capacity();
        let mut sink = RtpSink::new(ring, NonZeroUsize::new(352).unwrap(), || {});
        assert_eq!(sink.writable(), capacity);
        sink.write(&vec![7; capacity - 2]);
        assert_eq!(sink.writable(), 2);
        sink.write(&[8, 9]);
        assert_eq!(sink.writable(), 0);
        let mut out = [0; 352];
        consumer.read(&mut out).unwrap();
        assert_eq!(out, [7; 352]);
        assert_eq!(sink.writable(), 352);
        sink.write(&[10; 352]);
        assert_eq!(sink.writable(), 0);
        assert_eq!(consumer.stat().overflow, 0);
    }

    #[test]
    fn writable_saturates_until_logical_overflow_is_consumed() {
        let (mut ring, mut consumer) = RingBuffer::<i16>::spsc(1024);
        let capacity = consumer.capacity();
        // The general ring API must retain its overwrite/timeline semantics.
        ring.write_default(capacity + 20);
        let mut sink = RtpSink::new(ring, NonZeroUsize::new(352).unwrap(), || {});
        assert_eq!(sink.writable(), 0);
        consumer.read_commit(10);
        assert_eq!(sink.writable(), 0);
        consumer.read_commit(10);
        assert_eq!(sink.writable(), 0);
        consumer.read_commit(2);
        assert_eq!(sink.writable(), 2);
        consumer.read_commit(capacity);
        assert_eq!(sink.writable(), capacity, "reader ahead must not underflow occupancy");
    }
}
