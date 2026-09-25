use bytes::BytesMut;
use catplay_tokio::{BytesMutUtil, Decoder, Encoder};
use log::{debug, trace};

use crate::{
    cipher::AirPlayCipherSaltType,
    rtsp_frame::{RtspError, RtspResult},
};

use super::{AirPlayCipherCodec, RtspFrame, RtspFrameCodec};

pub struct AirPlayTransportCodec {
    pub frame_codec: RtspFrameCodec,
    pub cipher_codec: Option<AirPlayCipherCodec>,
    pub server: bool,
    // Number of leading bytes in `src` that are plaintext and visible to RTSP parser.
    plaintext_prefix_len: usize,
}

impl AirPlayTransportCodec {
    const DEFAULT_BUFFER_SIZE: usize = 8192;
    const MAX_BUFFER_SIZE: usize = 8192;

    pub fn new(server: bool) -> Self {
        Self {
            frame_codec: RtspFrameCodec::default(),
            cipher_codec: None,
            server,
            plaintext_prefix_len: 0,
        }
    }

    pub fn encrypt(&mut self, channel_type: AirPlayCipherSaltType, shared_secret: [u8; 32]) {
        self.cipher_codec
            .replace(AirPlayCipherCodec::new(shared_secret, channel_type, self.server));
    }

    pub fn decode(&mut self, src: &mut BytesMut) -> RtspResult<Option<Vec<RtspFrame>>> {
        trace!(
            "Decode buffer sizes src:{} plaintext_prefix:{}",
            src.capacity(),
            self.plaintext_prefix_len
        );

        if let Some(cipher) = self.cipher_codec.as_mut() {
            self.plaintext_prefix_len = cipher.decrypt_into_prefix_in_place(src, self.plaintext_prefix_len)?;
        } else {
            self.plaintext_prefix_len = src.len();
        }
        self.plaintext_prefix_len = self.plaintext_prefix_len.min(src.len());

        let mut frames = Vec::new();
        loop {
            let before = src.len();
            let decoded = self
                .frame_codec
                .decode_with_limit(src, self.plaintext_prefix_len)?;
            let consumed = before.saturating_sub(src.len());
            if consumed > 0 {
                // `decode_with_limit` may consume bytes and still return `None`
                // (e.g. header parsed, waiting for body). Keep plaintext window in sync.
                self.plaintext_prefix_len = self.plaintext_prefix_len.saturating_sub(consumed);
            }
            match decoded {
                Some(frame) => frames.push(frame),
                None => break,
            }
        }

        if src.is_empty() && src.capacity() > Self::MAX_BUFFER_SIZE {
            debug!("Trimmed src buffer");
            *src = BytesMut::with_capacity(Self::DEFAULT_BUFFER_SIZE);
            self.plaintext_prefix_len = 0;
        }

        if let Some(cipher) = self.cipher_codec.as_mut() {
            let want_plain = self.plaintext_prefix_len.max(1);
            let want_enc = cipher.estimate_encrypted_size(want_plain);
            BytesMutUtil::ensure_writable(src, want_enc);
        }

        if frames.is_empty() { Ok(None) } else { Ok(Some(frames)) }
    }

    pub fn encode(&mut self, items: Vec<RtspFrame>, dst: &mut BytesMut) -> RtspResult<()> {
        if items.is_empty() {
            return Ok(());
        }

        let plain_start = dst.len();
        for item in items {
            self.frame_codec.encode(item, dst)?;
        }

        trace!("Encode buffer sizes dst:{} ", dst.capacity());

        if let Some(cipher) = self.cipher_codec.as_mut() {
            let mut plaintext_tail = dst.split_off(plain_start);
            cipher.encode_in_place(&mut plaintext_tail)?;
            dst.unsplit(plaintext_tail);
        }

        Ok(())
    }
}

impl Decoder for AirPlayTransportCodec {
    type Item = Vec<RtspFrame>;
    type Error = RtspError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.decode(src)
    }
}

impl Encoder<Vec<RtspFrame>> for AirPlayTransportCodec {
    type Error = RtspError;

    fn encode(&mut self, item: Vec<RtspFrame>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.encode(item, dst)
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use catplay_tracing::hexdump::pretty_hexdump_limited;

    use crate::cipher::AirPlayCipherSaltType;
    use crate::rtsp_frame::{HttpStatus, RtspResponse};
    use crate::rtsp_transport::{AirPlayCipherCodec, RtspFrame};

    use super::AirPlayTransportCodec;

    fn encode_legacy_1024_chunks(shared_secret: [u8; 32], plaintext: &[u8]) -> BytesMut {
        let mut legacy_encoder = AirPlayCipherCodec::new(shared_secret, AirPlayCipherSaltType::Control, true);
        let mut encrypted = BytesMut::new();

        for chunk in plaintext.chunks(1024) {
            legacy_encoder
                .encode(BytesMut::from(chunk), &mut encrypted)
                .expect("legacy chunk encryption should succeed");
        }

        encrypted
    }

    #[test]
    fn encrypt_keeps_existing_plaintext_prefix() {
        let shared_secret = [7u8; 32];
        let mut codec = AirPlayTransportCodec::new(false);
        codec.plaintext_prefix_len = 17;

        codec.encrypt(AirPlayCipherSaltType::Control, shared_secret);

        assert_eq!(codec.plaintext_prefix_len, 17);
    }

    #[test]
    fn decodes_fragmented_legacy_1024_stream_on_transport_codec() {
        let shared_secret = [9u8; 32];

        // Build a single large RTSP response to mirror /info-ish payload sizes.
        let mut plain_transport = AirPlayTransportCodec::new(true);
        let mut plaintext_rtsp = BytesMut::new();
        let mut response = RtspResponse::new(Some(4), HttpStatus::Ok);
        response.payload = BytesMut::from(&vec![b'R'; 28_838][..]);
        plain_transport
            .encode(vec![RtspFrame::Response(response)], &mut plaintext_rtsp)
            .expect("plain transport encode should succeed");
        assert!(
            plaintext_rtsp.starts_with(b"RTSP/1.0"),
            "plaintext RTSP frame should start with status line, got: {}",
            pretty_hexdump_limited(&plaintext_rtsp[..plaintext_rtsp.len().min(128)], 256)
        );

        // Legacy wire format: independent encrypted chunks of at most 1024 plaintext bytes.
        let encrypted_stream = encode_legacy_1024_chunks(shared_secret, &plaintext_rtsp);
        {
            let mut decoder = AirPlayCipherCodec::new(shared_secret, AirPlayCipherSaltType::Control, false);
            let mut tmp = encrypted_stream.clone();
            let first = decoder
                .decode(&mut tmp)
                .expect("first cipher decode should succeed")
                .expect("first cipher packet should exist");
            assert!(
                first.starts_with(b"RTSP/1.0"),
                "first decrypted legacy chunk should start with RTSP status line, got: {}",
                pretty_hexdump_limited(&first[..first.len().min(128)], 256)
            );
        }

        // Decode through transport codec with encrypted mode enabled.
        let mut encrypted_transport = AirPlayTransportCodec::new(false);
        encrypted_transport.encrypt(AirPlayCipherSaltType::Control, shared_secret);

        let mut input_buf = BytesMut::new();
        let mut output_frames = Vec::new();
        let mut cursor = 0usize;
        let fragment_sizes = [1usize, 2, 3, 5, 8, 13, 21, 55, 89, 233, 377, 610];
        let mut frag_idx = 0usize;

        while cursor < encrypted_stream.len() {
            let take = fragment_sizes[frag_idx % fragment_sizes.len()].min(encrypted_stream.len() - cursor);
            input_buf.extend_from_slice(&encrypted_stream[cursor..cursor + take]);
            cursor += take;
            frag_idx += 1;

            match encrypted_transport.decode(&mut input_buf) {
                Ok(Some(frames)) => output_frames.extend(frames),
                Ok(None) => {}
                Err(err) => {
                    let prefix = encrypted_transport
                        .plaintext_prefix_len
                        .min(input_buf.len());
                    let dump_start = prefix.saturating_sub(32);
                    let dump_end = (prefix + 96).min(input_buf.len());
                    panic!(
                        "transport decode failed with unexpected error at cursor={} take={} input_len={} prefix={} window=[{}..{}] err={:?}\nhead:\n{}\nwindow:\n{}",
                        cursor,
                        take,
                        input_buf.len(),
                        prefix,
                        dump_start,
                        dump_end,
                        err,
                        pretty_hexdump_limited(&input_buf[..input_buf.len().min(128)], 512),
                        pretty_hexdump_limited(&input_buf[dump_start..dump_end], 512)
                    );
                }
            }
        }

        while let Some(frames) = encrypted_transport
            .decode(&mut input_buf)
            .expect("transport decode should succeed")
        {
            output_frames.extend(frames);
        }

        assert!(input_buf.is_empty(), "all encrypted input should be consumed");
        assert_eq!(output_frames.len(), 1);

        let RtspFrame::Response(decoded) = output_frames.pop().expect("missing decoded frame") else {
            panic!("expected response frame");
        };

        assert_eq!(decoded.cseq, Some(4));
        assert_eq!(decoded.payload.len(), 28_838);
        assert!(decoded.payload.iter().all(|b| *b == b'R'));
    }
}
