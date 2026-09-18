use std::fmt;

use crate::video::NalChunk;
use bitflags::bitflags;
use bytes::BytesMut;
use catplay_tokio::BytesMutUtil;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Value64 {
    pub u8: [u8; 8],
}

impl Value64 {
    pub const ZERO: Value64 = Value64::default();

    pub const fn default() -> Self {
        Self { u8: [0u8; 8] }
    }

    pub fn new(val: [u8; 8]) -> Self {
        Self { u8: val }
    }

    pub fn from_f32(a: f32, b: f32) -> Self {
        Self {
            u8: [a.to_le_bytes(), b.to_le_bytes()].concat().try_into().unwrap(),
        }
    }

    pub fn from_f32_floor(a: u32, b: u32) -> Self {
        Self::from_f32(a as f32, b as f32)
    }

    pub fn from_u32(a: u32, b: u32) -> Self {
        Self {
            u8: [a.to_le_bytes(), b.to_le_bytes()].concat().try_into().unwrap(),
        }
    }

    pub fn from_u64(a: u64) -> Self {
        Self { u8: a.to_le_bytes() }
    }

    pub fn as_f32(&self) -> (f32, f32) {
        (
            f32::from_le_bytes(self.u8[0..4].try_into().unwrap()),
            f32::from_le_bytes(self.u8[4..8].try_into().unwrap()),
        )
    }

    pub fn as_f32_floor(&self) -> (u32, u32) {
        let val = self.as_f32();
        (val.0.floor() as _, val.1.floor() as _)
    }

    pub fn as_u32(&self) -> (u32, u32) {
        (
            u32::from_le_bytes(self.u8[0..4].try_into().unwrap()),
            u32::from_le_bytes(self.u8[4..8].try_into().unwrap()),
        )
    }

    pub fn as_u64(&self) -> u64 {
        u64::from_le_bytes(self.u8)
    }

    pub fn serialize(&self) -> [u8; 8] {
        self.u8
    }

    pub fn debug(&self) -> String {
        if self.as_u64() == 0 {
            return "Value64(0)".into();
        }

        format!(
            "Value64(u64: {:?}, u32: {:?}, f32: {:?})",
            self.as_u64(),
            self.as_u32(),
            self.as_f32()
        )
    }
}

impl fmt::Debug for Value64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.debug())
    }
}

const SCREEN_FRAME_HEADER_LEN: usize = 128;
const _: [(); SCREEN_FRAME_HEADER_LEN] = [(); core::mem::size_of::<ScreenFrameHeader>()];

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ScreenFrameHeader {
    pub body_size: u32,
    pub opcode: ScreenOpCode,
    pub small_param: [u8; 3],
    pub params: [Value64; 15],
}

impl ScreenFrameHeader {
    #[inline]
    pub const fn size() -> usize {
        SCREEN_FRAME_HEADER_LEN
    }

    #[inline]
    pub fn from_bytes(buf: &[u8]) -> Option<ScreenFrameHeader> {
        const PARAMS_OFFSET: usize = 8;

        if buf.len() < SCREEN_FRAME_HEADER_LEN {
            return None;
        }

        let mut params = [Value64::default(); 15];
        for (i, p) in params.iter_mut().enumerate() {
            let start = PARAMS_OFFSET + i * 8;
            let end = start + 8;
            *p = Value64::new(buf[start..end].try_into().unwrap());
        }

        Some(ScreenFrameHeader {
            body_size: u32::from_le_bytes(buf[..4].try_into().unwrap()),
            opcode: ScreenOpCode::parse(buf[4]),
            small_param: [buf[5], buf[6], buf[7]],
            params,
        })
    }

    #[inline]
    pub fn write(&self, buf: &mut BytesMut) {
        buf.reserve(SCREEN_FRAME_HEADER_LEN);

        buf.extend_from_slice(&u32::to_le_bytes(self.body_size));
        buf.extend_from_slice(&[self.opcode as u8]);
        buf.extend_from_slice(&self.small_param);
        for param in self.params {
            buf.extend_from_slice(&param.serialize());
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScreenOpCode {
    VideoFrame = 0,
    VideoConfig = 1,
    KeepAlive = 2,
    ForceKeyFrame = 3,
    Ignore = 4,
    KeepAliveWithBody = 5,

    #[default]
    Invalid = 255,
}

impl ScreenOpCode {
    fn parse(val: u8) -> ScreenOpCode {
        match val {
            x if x == ScreenOpCode::VideoFrame as u8 => ScreenOpCode::VideoFrame,
            x if x == ScreenOpCode::VideoConfig as u8 => ScreenOpCode::VideoConfig,
            x if x == ScreenOpCode::KeepAlive as u8 => ScreenOpCode::KeepAlive,
            x if x == ScreenOpCode::ForceKeyFrame as u8 => ScreenOpCode::ForceKeyFrame,
            x if x == ScreenOpCode::Ignore as u8 => ScreenOpCode::Ignore,
            x if x == ScreenOpCode::KeepAliveWithBody as u8 => ScreenOpCode::KeepAliveWithBody,
            _ => ScreenOpCode::Invalid,
        }
    }
}

bitflags! {
     #[derive(Debug)]
    pub struct ScreenFlag : u8 {
        const ShowHUD = 1 << 0;
        const RespectTimestamps = 1 << 1;
        const Encrypted = 1 << 2;
        const UseFormatDescription = 1 << 3;
        const Suspended = 1 << 5;
        const ClearScreen	 = 1 << 6;
        const InterestingFrame = 1 << 7;
        // const NoDisplaySleep = 1 << 8;
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScreenFrame {
    pub header: ScreenFrameHeader,
    pub header_buf: BytesMut,
    pub data: BytesMut,
    pub chacha_tag_buf: BytesMut,
    pub nal_offset_cache: Vec<NalChunk>,
}

impl ScreenFrame {
    pub fn keep_alive() -> Self {
        let mut frame = Self::default();
        frame.header.opcode = ScreenOpCode::KeepAlive;
        frame
    }

    /// Minimal `keepAliveSendStatsAsBody` heartbeat, not full statistics telemetry.
    /// Optional metric meanings are unknown, so send an empty dictionary rather than
    /// fabricated measurements. MHI2Q rejects zero-length bodies before opcode dispatch;
    /// its nonempty opcode-5 path refreshes activity without inspecting the plist.
    pub fn keep_alive_with_empty_stats() -> Self {
        // Canonical bplist00: empty dictionary object, one-byte offset table, trailer.
        // Precomputed to avoid fallible serialization and serializer allocations per tick.
        const EMPTY_DICT: &[u8; 42] = b"bplist00\xd0\x08\
            \x00\x00\x00\x00\x00\x00\x01\x01\
            \x00\x00\x00\x00\x00\x00\x00\x01\
            \x00\x00\x00\x00\x00\x00\x00\x00\
            \x00\x00\x00\x00\x00\x00\x00\x09";
        let mut frame = Self::default();
        frame.header.opcode = ScreenOpCode::KeepAliveWithBody;
        frame.header.body_size = EMPTY_DICT.len() as u32;
        // iOS places the body length in the second float of the last parameter.
        // Construct on the stack (Value64::from_f32 currently allocates a temporary Vec).
        let mut length = [0; 8];
        length[4..].copy_from_slice(&(EMPTY_DICT.len() as f32).to_le_bytes());
        frame.header.params[14] = Value64::new(length);
        frame.data.extend_from_slice(EMPTY_DICT);
        frame
    }

    pub fn new(header: ScreenFrameHeader, data: BytesMut) -> Self {
        Self {
            header,
            header_buf: BytesMut::with_capacity(ScreenFrameHeader::size()),
            data,
            chacha_tag_buf: BytesMut::with_capacity(16),
            nal_offset_cache: Vec::new(),
        }
    }

    pub fn with_buffers_chacha(header: ScreenFrameHeader, data: BytesMut, header_buf: BytesMut, chacha_tag_buf: BytesMut) -> Self {
        Self {
            header,
            header_buf,
            data,
            chacha_tag_buf,
            nal_offset_cache: Vec::new(),
        }
    }

    pub fn with_buffers(header: ScreenFrameHeader, data: BytesMut, header_buf: BytesMut) -> Self {
        Self {
            header,
            header_buf,
            data,
            chacha_tag_buf: BytesMut::with_capacity(16),
            nal_offset_cache: Vec::new(),
        }
    }

    pub fn write(&self, buf: &mut BytesMut) {
        buf.reserve(ScreenFrameHeader::size() + self.data.len());
        self.header.write(buf);
        buf.extend_from_slice(&self.data);
    }

    /// Creates a slice that allows viewing [ScreenFrame] as a `header_buf` + `data` + `chacha_tag_buf` combo,
    /// assuming they are adjacent _and_ `header_buf` is exactly 128 bytes and `chacha_tag_buf` is exactly 16 bytes or empty.
    ///
    /// This is assumed to be true for all frames originating from decoder, and can be used as a fast-path for proxying the frame in zero-copy mode.
    pub fn as_adjacent_slice_mut(&mut self) -> Option<&mut [u8]> {
        if self.header_buf.len() != ScreenFrameHeader::size() || (self.chacha_tag_buf.len() != 16 && !self.chacha_tag_buf.is_empty()) {
            return None;
        }

        BytesMutUtil::adjacent_slice_mut(&mut [&mut self.header_buf, &mut self.data, &mut self.chacha_tag_buf])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use catplay_plist::{Dictionary, PlistSerializable, Value};

    #[test]
    fn keep_alive_with_empty_stats_wire() {
        let frame = ScreenFrame::keep_alive_with_empty_stats();
        let mut wire = BytesMut::new();
        frame.write(&mut wire);
        assert_eq!(ScreenFrameHeader::size(), 128);
        assert_eq!(wire.len(), 128 + 42);
        let mut expected = [0; 128];
        expected[..4].copy_from_slice(&42u32.to_le_bytes());
        expected[4] = 5;
        expected[124..].copy_from_slice(&42f32.to_le_bytes());
        assert_eq!(&wire[..128], &expected);
        let header = ScreenFrameHeader::from_bytes(&wire).unwrap();
        assert_eq!(header.body_size as usize, wire.len() - 128);
        assert_eq!(header.params[14].as_f32(), (0.0, 42.0));
        assert_eq!(Value::pdecode(&wire[128..]).unwrap(), Value::Dictionary(Dictionary::new()));
        assert_eq!(Value::Dictionary(Dictionary::new()).pencode().unwrap(), frame.data);
    }

    #[test]
    fn keep_alive_legacy_wire_unchanged() {
        let mut wire = BytesMut::new();
        ScreenFrame::keep_alive().write(&mut wire);
        let mut expected = [0; 128];
        expected[4] = 2;
        assert_eq!(&wire[..], &expected);
    }

    #[test]
    fn adjacent_slice_mut_ignores_empty_buffers() {
        let mut backing = BytesMut::from(&b"abcdef"[..]);
        let mut left = backing.split_to(2);
        let mut empty = BytesMut::new();
        let mut right = backing.split_to(4);

        let slice = BytesMutUtil::adjacent_slice_mut(&mut [&mut left, &mut empty, &mut right]).unwrap();
        assert_eq!(slice, b"abcdef");

        slice[1] = b'B';
        slice[4] = b'E';
        assert_eq!(&left[..], b"aB");
        assert_eq!(&right[..], b"cdEf");
    }

    #[test]
    fn as_adjacent_slice_mut_ignores_empty_data() {
        let mut backing = BytesMut::from(&vec![0u8; ScreenFrameHeader::size() + 16][..]);
        let header_buf = backing.split_to(ScreenFrameHeader::size());
        let mut data = backing.split_to(16);
        let chacha_tag_buf = data.split_off(0);

        let mut frame = ScreenFrame::with_buffers_chacha(ScreenFrameHeader::default(), data, header_buf, chacha_tag_buf);
        let slice = frame.as_adjacent_slice_mut().unwrap();

        assert_eq!(slice.len(), ScreenFrameHeader::size() + 16);
    }
}
