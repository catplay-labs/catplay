use bytes::{Buf, BytesMut};
use std::{collections::VecDeque, io::IoSlice};

pub struct BytesMutQueue {
    pub(crate) queue: VecDeque<BytesMut>,
    pub(crate) tail: BytesMut,
    pub(crate) out: Vec<IoSlice<'static>>,
    policy: TailPolicy,
}

struct TailPolicy {
    retain_capacity: usize,
    shrink_threshold: usize,
}

impl Default for TailPolicy {
    fn default() -> Self {
        Self {
            retain_capacity: 8 * 1024,
            shrink_threshold: 256 * 1024,
        }
    }
}

impl BytesMutQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            tail: BytesMut::new(),
            out: Vec::new(),
            policy: TailPolicy::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.queue.len() + usize::from(!self.tail.is_empty())
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty() && self.tail.is_empty()
    }

    pub fn tail_mut(&mut self) -> &mut BytesMut {
        &mut self.tail
    }

    pub fn push(&mut self, buf: BytesMut) {
        if buf.is_empty() {
            return;
        }

        if !self.tail.is_empty() {
            /* self.tail.extend_from_slice(&buf); */
            let old_tail = std::mem::replace(&mut self.tail, buf);
            self.queue.push_back(old_tail);
            return;
        }

        self.queue.push_back(buf);
    }

    pub fn first(&self) -> Option<&[u8]> {
        if let Some(front) = self.queue.front() {
            return Some(front.as_ref());
        }

        if !self.tail.is_empty() {
            return Some(self.tail.as_ref());
        }

        None
    }

    pub fn as_iovecs(&mut self) -> &[IoSlice<'_>] {
        self.out.clear();
        if self.is_empty() {
            return self.out.as_slice();
        }

        let mut cur_ptr: *const u8 = std::ptr::null();
        let mut cur_len = 0usize;
        let mut has_cur = false;

        let mut push_span = |ptr: *const u8, len: usize, out: &mut Vec<IoSlice<'static>>| {
            if len == 0 {
                return;
            }

            if !has_cur {
                cur_ptr = ptr;
                cur_len = len;
                has_cur = true;
                return;
            }

            let expected = unsafe { cur_ptr.add(cur_len) };
            if expected == ptr {
                cur_len += len;
                return;
            }

            let slice = unsafe { std::slice::from_raw_parts(cur_ptr, cur_len) };
            let slice_static: &'static [u8] = unsafe { std::mem::transmute(slice) };
            out.push(IoSlice::new(slice_static));
            cur_ptr = ptr;
            cur_len = len;
        };

        for b in self.queue.make_contiguous() {
            push_span(b.as_ptr(), b.len(), &mut self.out);
        }

        if !self.tail.is_empty() {
            push_span(self.tail.as_ptr(), self.tail.len(), &mut self.out);
        }

        if has_cur {
            let slice = unsafe { std::slice::from_raw_parts(cur_ptr, cur_len) };
            let slice_static: &'static [u8] = unsafe { std::mem::transmute(slice) };
            self.out.push(IoSlice::new(slice_static));
        }

        self.out.as_slice()
    }

    pub fn queued_bytes(&self) -> usize {
        self.queue.iter().map(BytesMut::len).sum::<usize>() + self.tail.len()
    }

    pub fn drain_written(&mut self, bytes: usize) {
        let mut left = bytes;
        while left > 0 {
            let Some(front) = self.queue.front_mut() else {
                break;
            };

            let front_len = front.len();
            if front_len <= left {
                left -= front_len;
                self.queue.pop_front();
                continue;
            }

            front.advance(left);
            left = 0;
            break;
        }

        if left == 0 || self.tail.is_empty() {
            self.shrink_empty_tail_if_needed();
            return;
        }

        if left >= self.tail.len() {
            self.tail.clear();
        } else {
            self.tail.advance(left);
        }

        self.shrink_empty_tail_if_needed();
    }

    fn shrink_empty_tail_if_needed(&mut self) {
        if self.tail.is_empty() && self.tail.capacity() > self.policy.shrink_threshold {
            self.tail = BytesMut::with_capacity(self.policy.retain_capacity);
        }
    }
}

impl Default for BytesMutQueue {
    fn default() -> Self {
        Self::new()
    }
}
