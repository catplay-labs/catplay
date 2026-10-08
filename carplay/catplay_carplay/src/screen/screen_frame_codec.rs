use std::{mem, time::Instant};

use bitflags::bitflags;
use bytes::{Buf, BytesMut};
use catplay_hap::aes::Aes128Ctr;
use catplay_hap::cipher::HomeKitChaChaNonce;
#[cfg(not(test))]
use catplay_hap::cipher::HomeKitCipher;
// Exercise the real prefetch path on the host too; production backend selection
// is unchanged. Key derivation and direction still use the shared factory.
#[cfg(test)]
use catplay_hap::cipher::HomeKitCipherFast as HomeKitCipher;
use catplay_tokio::{BytesMutUtil, Decoder, Encoder, EncoderComposite};
use log::{debug, trace};

#[cfg(test)]
use crate::cipher::create_chacha_ciphers_with;
use crate::{
    cipher::{AirPlayCipherSaltType, AirPlayStreamEncryption, create_chacha_ciphers, derive_aes_stream_keys},
    rtsp_frame::{RtspError, RtspResult},
    screen::{ScreenFrame, ScreenFrameHeader, ScreenOpCode},
};

#[allow(clippy::large_enum_variant)]
enum ScreenFrameCodecCrypto {
    Chacha {
        read_cipher: HomeKitCipher,
        write_cipher: HomeKitCipher,
        counter_rx: HomeKitChaChaNonce,
        counter_tx: HomeKitChaChaNonce,
        frame_size_rx: FrameSizeGuesstimator,
        frame_size_tx: FrameSizeGuesstimator,
    },
    AesCtr {
        read_cipher: Aes128Ctr,
        write_cipher: Aes128Ctr,
    },
}

#[derive(Default, Clone, Copy)]
struct FrameSizeGuesstimator {
    mean: Option<usize>,
    deviation: usize,
    peak: usize,

    min_seen: usize,
    max_seen: usize,
    unlimited_budget: bool,
}

impl FrameSizeGuesstimator {
    pub fn unlimited_budget() -> Self {
        Self {
            unlimited_budget: true,
            ..Default::default()
        }
    }

    fn observe(&mut self, frame_size: usize) {
        match self.mean {
            None => {
                self.mean = Some(frame_size);
                self.peak = frame_size;
                self.min_seen = frame_size;
                self.max_seen = frame_size;
            }

            Some(previous_mean) => {
                self.min_seen = self.min_seen.min(frame_size);
                self.max_seen = self.max_seen.max(frame_size);

                // How surprising was this frame relative to what we expected?
                let error = frame_size.abs_diff(previous_mean);

                // Mean EWMA, alpha = 1/4.
                self.mean = Some(previous_mean.saturating_mul(3).saturating_add(frame_size) / 4);

                // EWMA of prediction error, alpha = 1/4.
                self.deviation = self.deviation.saturating_mul(3).saturating_add(error) / 4;

                // Recent peak decays by 12.5% per frame.
                //
                // A burst immediately raises it, while an old large frame
                // disappears relatively quickly from the prediction.
                let decayed_peak = self.peak.saturating_mul(7) / 8;
                self.peak = frame_size.max(decayed_peak);
            }
        }
    }

    fn prefetch_next(&mut self, frame_size: usize) -> usize {
        self.observe(frame_size);

        let mean = self.mean.unwrap();

        // Baseline plus uncertainty.
        let statistical = mean.saturating_add(self.deviation.saturating_mul(2));

        // Protect against short bursts whose mean has not caught up yet.
        let mut estimate = statistical.max(self.peak);

        if self.unlimited_budget {
            // Continuously prefetch until next frame (from a deprioritized thread)
            // to increase cache hit rate at the cost of zero thermal efficiency
            // and increased latency for high-priority threads.
            estimate = usize::MAX;
        }

        // Never speculate outside sizes actually observed in this session.
        estimate.clamp(self.min_seen, self.max_seen)
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    #[allow(non_upper_case_globals)]
    pub struct ScreenFrameCodecFlags: u8 {
        const PreferPolySkip = 1 << 0;
        const PreferPrefetchAes = 1 << 1;
        const PreferPrefetchChacha = 1 << 2;
        const PreferPrefetchChachaBurning = 1 << 3;
    }
}

impl ScreenFrameCodecFlags {
    pub fn crate_defaults() -> Self {
        let mut flags = ScreenFrameCodecFlags::empty();
        flags.set(
            ScreenFrameCodecFlags::PreferPolySkip,
            cfg!(feature = "screen_skip_poly_best_effort"),
        );
        flags.set(ScreenFrameCodecFlags::PreferPrefetchAes, cfg!(feature = "screen_prefetch_aes"));
        flags.set(
            ScreenFrameCodecFlags::PreferPrefetchChacha,
            cfg!(feature = "screen_prefetch_chacha"),
        );
        flags.set(
            ScreenFrameCodecFlags::PreferPrefetchChachaBurning,
            cfg!(feature = "screen_prefetch_chacha_burning"),
        );
        flags
    }
}

pub struct ScreenFrameCodec {
    pub header: Option<(ScreenFrameHeader, Option<HomeKitChaChaNonce>, usize)>,
    crypto: Option<ScreenFrameCodecCrypto>,
    pub chunks_rx: usize,
    flags: ScreenFrameCodecFlags,
}

impl ScreenFrameCodec {
    const MAX_PAYLOAD_SIZE: usize = 4 * 1024 * 1024; // 4MB
    const CHACHA_TAG_LEN: usize = 16;

    /// An estimated maximum frame size; a bigger buffer does not result in CPU overhead, only reserves memory.
    /// After frame is decoded we only refill the amount equal to the size of decoded frame.
    const AES_PREFETCH_SIZE: usize = 512 * 1024;

    pub fn new(encryption: AirPlayStreamEncryption, stream_connection_id: u64, server: bool) -> Self {
        Self::new_with_flags(encryption, stream_connection_id, server, ScreenFrameCodecFlags::empty())
    }

    pub fn new_with_flags(
        encryption: AirPlayStreamEncryption,
        stream_connection_id: u64,
        server: bool,
        flags: ScreenFrameCodecFlags,
    ) -> Self {
        match encryption {
            // There was never support for unencrypted screen sessions.
            // ET = None maps to AES session where session_key before derivation is all zeroes.
            // AirPlayStreamEncryption::Unconfigured | AirPlayStreamEncryption::None => Self::unencrypted(),
            AirPlayStreamEncryption::Unconfigured | AirPlayStreamEncryption::None => {
                Self::aes_with_flags([0u8; 16], stream_connection_id, server, flags)
            }
            AirPlayStreamEncryption::Aes { key, .. } => Self::aes_with_flags(key, stream_connection_id, server, flags),
            AirPlayStreamEncryption::ChaCha { shared_secret } => {
                Self::chacha_with_flags(shared_secret, stream_connection_id, server, flags)
            }
        }
    }

    pub fn chacha(shared_secret: [u8; 32], stream_connection_id: u64, server: bool) -> Self {
        Self::chacha_with_flags(shared_secret, stream_connection_id, server, ScreenFrameCodecFlags::empty())
    }

    pub fn chacha_with_flags(shared_secret: [u8; 32], stream_connection_id: u64, server: bool, flags: ScreenFrameCodecFlags) -> Self {
        #[cfg(not(test))]
        let (mut read_cipher, write_cipher) =
            crate::cipher::create_chacha_ciphers(&shared_secret, AirPlayCipherSaltType::DataStream { stream_connection_id }, server);

        #[cfg(test)]
        let (mut read_cipher, write_cipher) = create_chacha_ciphers_with(
            &shared_secret,
            AirPlayCipherSaltType::DataStream { stream_connection_id },
            server,
            HomeKitCipher::new,
        );

        if flags.contains(ScreenFrameCodecFlags::PreferPolySkip) {
            read_cipher.set_prefer_poly_skip();
        }

        let guesstimator = if flags.contains(ScreenFrameCodecFlags::PreferPrefetchChachaBurning) {
            FrameSizeGuesstimator::unlimited_budget()
        } else {
            FrameSizeGuesstimator::default()
        };

        Self {
            header: None,
            crypto: Some(ScreenFrameCodecCrypto::Chacha {
                read_cipher,
                write_cipher,
                counter_rx: HomeKitChaChaNonce(0),
                counter_tx: HomeKitChaChaNonce(0),
                frame_size_rx: guesstimator,
                frame_size_tx: guesstimator,
            }),
            chunks_rx: 0,
            flags,
        }
    }

    pub fn aes(session_key: [u8; 16], stream_connection_id: u64, server: bool) -> Self {
        Self::aes_with_flags(session_key, stream_connection_id, server, ScreenFrameCodecFlags::empty())
    }

    pub fn aes_with_flags(session_key: [u8; 16], stream_connection_id: u64, server: bool, flags: ScreenFrameCodecFlags) -> Self {
        let (video_key, video_iv) = derive_aes_stream_keys(&session_key, stream_connection_id);
        let read_cipher = Aes128Ctr::new(
            &video_key,
            &video_iv,
            if server && flags.contains(ScreenFrameCodecFlags::PreferPrefetchAes) {
                Self::AES_PREFETCH_SIZE
            } else {
                0
            },
        );
        let write_cipher = Aes128Ctr::new(
            &video_key,
            &video_iv,
            if !server && flags.contains(ScreenFrameCodecFlags::PreferPrefetchAes) {
                Self::AES_PREFETCH_SIZE
            } else {
                0
            },
        );

        Self {
            header: None,
            crypto: Some(ScreenFrameCodecCrypto::AesCtr { read_cipher, write_cipher }),
            chunks_rx: 0,
            flags,
        }
    }

    pub fn unencrypted() -> Self {
        Self {
            header: None,
            crypto: None,
            chunks_rx: 0,
            flags: ScreenFrameCodecFlags::empty(),
        }
    }

    fn is_chacha(&self) -> bool {
        matches!(self.crypto, Some(ScreenFrameCodecCrypto::Chacha { .. }))
    }

    #[allow(clippy::too_many_arguments)]
    fn decrypt_video_frame_chunk_chacha(
        &mut self,
        src: &mut BytesMut,
        chunk_index: usize,
        header_size: usize,
        body_size: usize,
        decrypt_nonce: &mut Option<HomeKitChaChaNonce>,
        decrypted_payload_len: usize,
        current_body_and_tag_len: usize,
    ) -> RtspResult<usize> {
        let (read_cipher, counter_rx, frame_size_rx) = match self.crypto.as_mut().unwrap() {
            ScreenFrameCodecCrypto::Chacha {
                read_cipher,
                counter_rx,
                frame_size_rx,
                ..
            } => (read_cipher, counter_rx, frame_size_rx),
            _ => unreachable!(),
        };

        let nonce = if let Some(nonce) = *decrypt_nonce {
            nonce
        } else {
            let nonce = counter_rx.advance()?;
            *decrypt_nonce = Some(nonce);
            nonce
        };

        let chunk_len = current_body_and_tag_len.saturating_sub(decrypted_payload_len);
        let is_final_chunk = current_body_and_tag_len == body_size;
        let (header_raw, body_and_tail) = src.split_at_mut(header_size);
        let body_and_tag = &mut body_and_tail[..current_body_and_tag_len];
        let crypto_start = Instant::now();
        let result = read_cipher.decrypt_progressive(
            body_and_tag,
            header_raw,
            nonce,
            decrypted_payload_len,
            current_body_and_tag_len,
            body_size,
        );
        let crypto_took = Instant::now() - crypto_start;
        let decrypted_chunk_len = match result {
            Ok(chunk) => chunk.len(),
            Err(error) => {
                return Err(error.into());
            }
        };

        debug!(
            "Decrypted ChaCha video frame chunk {}/x of {chunk_len}/{body_size}b in {crypto_took:?} capacity={} len={} ptr={:?} decrypted_chunk={decrypted_chunk_len} final={is_final_chunk}",
            chunk_index,
            src.capacity(),
            src.len(),
            src.as_ptr()
        );

        if is_final_chunk
            && self
                .flags
                .contains(ScreenFrameCodecFlags::PreferPrefetchChacha)
        {
            let payload_len = body_size - Self::CHACHA_TAG_LEN;
            Self::reset_chacha_prefetch(read_cipher, *counter_rx, frame_size_rx, payload_len, "Decrypted");
        }

        Ok(decrypted_chunk_len)
    }

    fn decrypt_video_frame_chunk_aes(
        &mut self,
        src: &mut BytesMut,
        chunk_index: usize,
        header_size: usize,
        body_size: usize,
        decrypted_payload_len: usize,
        current_body_len: usize,
    ) -> usize {
        let read_cipher = match self.crypto.as_mut().unwrap() {
            ScreenFrameCodecCrypto::AesCtr { read_cipher, .. } => read_cipher,
            _ => unreachable!(),
        };
        let decrypted_chunk_len = current_body_len.saturating_sub(decrypted_payload_len);
        if decrypted_chunk_len > 0 {
            read_cipher
                .apply_keystream(&mut src[header_size + decrypted_payload_len..header_size + current_body_len])
                .expect("AES-CTR video decryption failed");
            debug!("Decrypted AES-CTR video chunk {chunk_index}/x of {decrypted_chunk_len}/{body_size}b");
        }
        decrypted_chunk_len
    }

    #[allow(clippy::too_many_arguments)]
    fn decrypt_video_frame_chunk(
        &mut self,
        src: &mut BytesMut,
        chunk_index: usize,
        header_size: usize,
        body_size: usize,
        decrypt_nonce: &mut Option<HomeKitChaChaNonce>,
        decrypted_payload_len: usize,
        current_body_and_tag_len: usize,
    ) -> RtspResult<usize> {
        match self.crypto.as_ref() {
            Some(ScreenFrameCodecCrypto::Chacha { .. }) => self.decrypt_video_frame_chunk_chacha(
                src,
                chunk_index,
                header_size,
                body_size,
                decrypt_nonce,
                decrypted_payload_len,
                current_body_and_tag_len,
            ),
            Some(ScreenFrameCodecCrypto::AesCtr { .. }) => Ok(self.decrypt_video_frame_chunk_aes(
                src,
                chunk_index,
                header_size,
                body_size,
                decrypted_payload_len,
                current_body_and_tag_len,
            )),
            None => Ok(0),
        }
    }

    pub fn decode(&mut self, src: &mut BytesMut) -> RtspResult<Option<ScreenFrame>> {
        match self.header.take() {
            None => {
                {
                    // If the buffer is empty after decoding a frame
                    // (which is expected because they only come in ~16ms intervals, unless a network lag results in a burst)
                    // and the buffer is warm enough for the post-header reserve(...) to be no-op
                    // then we can align the buffer for optimal decrypt speed
                    if src.is_empty() {
                        BytesMutUtil::ensure_writable(src, 256);

                        let align = src.as_ptr().align_offset(64);
                        if align != usize::MAX && align != 0 {
                            trace!("Adding {align} align before recv");
                            src.resize(align, 0);
                            src.advance(align);
                        } else {
                            trace!("No need to align ptr={:?}", src.as_ptr());
                        }
                    }
                }

                let header_size = ScreenFrameHeader::size();
                if src.len() < header_size {
                    BytesMutUtil::ensure_writable(src, header_size);
                    return Ok(None);
                }
                let decoded = ScreenFrameHeader::from_bytes(&src[..header_size]).unwrap();

                self.chunks_rx = 0;
                self.header.replace((decoded, None, 0));
                self.decode(src)
            }
            Some((mut header, mut decrypt_nonce, mut decrypted_payload_len)) => {
                if header.body_size > Self::MAX_PAYLOAD_SIZE as _ {
                    return Err(RtspError::PayloadTooBig(header.body_size as _, Self::MAX_PAYLOAD_SIZE));
                }

                let header_size = ScreenFrameHeader::size();
                let body_size = header.body_size as usize;
                let frame_size = header_size + body_size;

                self.chunks_rx += 1;
                let video_is_encrypted = header.opcode == ScreenOpCode::VideoFrame && self.crypto.is_some();
                if src.len() < frame_size {
                    BytesMutUtil::ensure_writable(src, frame_size);

                    if video_is_encrypted {
                        let current_body_and_tag_len = src.len().saturating_sub(header_size);
                        // Bigger frames may get delivered over anywhere between 1-10 calls to decode(...).
                        // Progressive decryption avoids wasting CPU time while waiting for next chunk of data to arrive.
                        let decrypted_chunk_len = self.decrypt_video_frame_chunk(
                            src,
                            self.chunks_rx,
                            header_size,
                            body_size,
                            &mut decrypt_nonce,
                            decrypted_payload_len,
                            current_body_and_tag_len,
                        )?;
                        decrypted_payload_len += decrypted_chunk_len;
                    }

                    self.header
                        .replace((header, decrypt_nonce, decrypted_payload_len));
                    return Ok(None);
                }

                self.header = None;

                let chunks_rx = mem::take(&mut self.chunks_rx);
                if video_is_encrypted && self.is_chacha() {
                    self.decrypt_video_frame_chunk(
                        src,
                        chunks_rx,
                        header_size,
                        body_size,
                        &mut decrypt_nonce,
                        decrypted_payload_len,
                        body_size,
                    )?;

                    let header_buf = src.split_to(header_size);
                    let mut data_and_tag = src.split_to(body_size);
                    let chacha_tag_buf = data_and_tag.split_off(body_size.saturating_sub(Self::CHACHA_TAG_LEN));
                    header.body_size = body_size.saturating_sub(Self::CHACHA_TAG_LEN) as u32;
                    Ok(Some(ScreenFrame::with_buffers_chacha(
                        header,
                        data_and_tag,
                        header_buf,
                        chacha_tag_buf,
                    )))
                } else {
                    if video_is_encrypted {
                        self.decrypt_video_frame_chunk(
                            src,
                            chunks_rx,
                            header_size,
                            body_size,
                            &mut decrypt_nonce,
                            decrypted_payload_len,
                            body_size,
                        )?;
                        if self
                            .flags
                            .contains(ScreenFrameCodecFlags::PreferPrefetchAes)
                            && let Some(ScreenFrameCodecCrypto::AesCtr { read_cipher, .. }) = self.crypto.as_mut()
                        {
                            debug!("Decrypted AES-CTR video frame: {}", read_cipher.finish_prefetch_cycle());
                        }
                    }
                    let header_buf = src.split_to(header_size);
                    let data = src.split_to(body_size);
                    Ok(Some(ScreenFrame::with_buffers(header, data, header_buf)))
                }
            }
        }
    }

    fn reset_chacha_prefetch(
        cipher: &mut HomeKitCipher,
        next_nonce: HomeKitChaChaNonce,
        frame_size_ewma: &mut FrameSizeGuesstimator,
        payload_len: usize,
        direction: &str,
    ) {
        let next_size = frame_size_ewma.prefetch_next(payload_len);
        if let Some(stats) = cipher.reset_prefetch(next_nonce, next_size) {
            debug!(
                "{direction} ChaCha video frame of {payload_len}b: nonce={} {} next_target={next_size}",
                stats.nonce, stats,
            );
        }
    }

    pub fn encode_composite(&mut self, mut frame: ScreenFrame, callback: &mut dyn FnMut(BytesMut)) -> RtspResult<()> {
        frame.header.body_size = frame.data.len() as u32;

        if frame.header.opcode == ScreenOpCode::VideoFrame
            && let Some(crypto) = self.crypto.as_mut()
        {
            match crypto {
                ScreenFrameCodecCrypto::Chacha {
                    write_cipher,
                    counter_tx,
                    frame_size_tx,
                    ..
                } => {
                    let nonce = counter_tx.advance()?;

                    frame.header.body_size += Self::CHACHA_TAG_LEN as u32;

                    let mut header = frame.header_buf;
                    header.clear();
                    frame.header.write(&mut header);

                    let crypto_start = Instant::now();
                    let tag = write_cipher.encrypt(&mut frame.data, &header, nonce)?;
                    let crypto_took = Instant::now() - crypto_start;

                    debug!("Encrypted ChaCha video frame of {}b in {crypto_took:?}", frame.data.len());
                    if self
                        .flags
                        .contains(ScreenFrameCodecFlags::PreferPrefetchChacha)
                    {
                        Self::reset_chacha_prefetch(write_cipher, *counter_tx, frame_size_tx, frame.data.len(), "Encrypted");
                    }

                    let mut tag_bm = frame.chacha_tag_buf;
                    tag_bm.clear();
                    tag_bm.extend_from_slice(tag.as_ref());

                    callback(header);
                    callback(frame.data);
                    callback(tag_bm);
                    return Ok(());
                }
                ScreenFrameCodecCrypto::AesCtr { write_cipher, .. } => {
                    let mut header = frame.header_buf;
                    header.clear();
                    frame.header.write(&mut header);

                    let crypto_start = Instant::now();
                    write_cipher
                        .apply_keystream(&mut frame.data)
                        .expect("AES-CTR video encryption failed");
                    let crypto_took = Instant::now() - crypto_start;
                    if self
                        .flags
                        .contains(ScreenFrameCodecFlags::PreferPrefetchAes)
                    {
                        debug!(
                            "Encrypted AES video frame of {}b in {crypto_took:?}: {}",
                            frame.data.len(),
                            write_cipher.finish_prefetch_cycle(),
                        );
                    } else {
                        debug!("Encrypted AES video frame of {}b in {crypto_took:?}", frame.data.len());
                    }

                    callback(header);
                    callback(frame.data);
                    return Ok(());
                }
            }
        }

        // a basic non-video frame
        let mut buf = BytesMut::new();
        frame.write(&mut buf);
        callback(buf);

        Ok(())
    }

    pub fn encode(&mut self, frame: ScreenFrame, dst: &mut BytesMut) -> RtspResult<()> {
        self.encode_forced(frame, dst)
    }
}

impl Decoder for ScreenFrameCodec {
    type Item = ScreenFrame;
    type Error = RtspError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.decode(src)
    }
}

impl EncoderComposite<ScreenFrame> for ScreenFrameCodec {
    type Error = RtspError;

    fn encode_composite(&mut self, item: ScreenFrame, callback: &mut dyn FnMut(BytesMut)) -> Result<(), Self::Error> {
        self.encode_composite(item, callback)
    }
}

impl Encoder<ScreenFrame> for ScreenFrameCodec {
    type Error = RtspError;

    fn encode(&mut self, item: ScreenFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.encode(item, dst)
    }
}

#[cfg(test)]
mod tests {

    use std::time::{Duration, Instant};

    use bytes::BytesMut;
    use catplay_tracing::logger::setup_test_logger;

    use crate::{
        clock::MediaClockSession,
        screen::{ScreenFrame, ScreenFrameCodec, ScreenFrameCodecFlags, ScreenFrameHeader, ScreenOpCode, Value64},
        video::{AvccConfig, AvccConfigExtended},
    };

    use super::FrameSizeGuesstimator;

    #[test]
    fn test_frame_size_ewma_and_chacha_prefetch_bounds() {
        let mut sizes = FrameSizeGuesstimator::default();
        assert_eq!(sizes.prefetch_next(8 * 1024), 8 * 1024);
        assert_eq!(sizes.mean, Some(8 * 1024));
        assert_eq!(sizes.prefetch_next(128 * 1024), 128 * 1024);
        assert_eq!(sizes.mean, Some(38 * 1024));
        assert_eq!(sizes.prefetch_next(128 * 1024), 128 * 1024);
        assert_eq!(sizes.mean, Some(60 * 1024 + 512));
        assert_eq!(sizes.prefetch_next(4 * 1024 * 1024), 4 * 1024 * 1024);

        let mut large = FrameSizeGuesstimator::default();
        assert_eq!(large.prefetch_next(256 * 1024), 256 * 1024);
        assert_eq!(large.prefetch_next(0), 256 * 1024);

        let mut burning = FrameSizeGuesstimator::unlimited_budget();
        assert_eq!(burning.prefetch_next(8 * 1024), 8 * 1024);
        assert_eq!(burning.prefetch_next(128 * 1024), 128 * 1024);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let shared_secret: [u8; 32] = *b"12345678123456781234567812345678";
        let mut shared_secret2: [u8; 32] = *b"12345678123456781234567812345678";
        shared_secret2.reverse();

        let stream_connection_id: u64 = 12345678;

        let mut server = ScreenFrameCodec::chacha(shared_secret, stream_connection_id, true);
        let data = vec![1, 1, 2, 2, 3, 3, 4, 4];
        let mut bm = BytesMut::new();
        bm.extend_from_slice(&data);

        let frame = ScreenFrame::new(
            ScreenFrameHeader {
                opcode: ScreenOpCode::VideoFrame,
                body_size: 0,
                small_param: [0u8; 3],
                params: [Value64::default(); 15],
            },
            bm,
        );

        let mut dst = BytesMut::new();
        server.encode(frame, &mut dst).unwrap();

        let peek = ScreenFrameHeader::from_bytes(&dst).unwrap();
        assert_eq!(peek.body_size as usize, data.len() + 16);
        println!("{:?}", peek);

        let mut client = ScreenFrameCodec::chacha(shared_secret, stream_connection_id, false);
        let mut ret = client.decode(&mut dst).unwrap().unwrap();
        assert_eq!(ret.header.body_size as usize, data.len());
        assert_eq!(ret.data, data);

        // For decoded frames we expect header_buf|data|chacha_tag_buf to be adjacent in case we want to use .unsplit() fast-path when proxying the frame
        unsafe {
            assert_eq!(ret.header_buf.as_ptr().add(ret.header_buf.len()), ret.data.as_ptr());
            assert_eq!(ret.data.as_ptr().add(ret.data.len()), ret.chacha_tag_buf.as_ptr());

            assert_eq!(ret.chacha_tag_buf.len(), 16);
            assert_eq!(ret.header_buf.len(), 128);
            assert!(ret.as_adjacent_slice_mut().is_some());
        }

        // Simulate proxying process (changed chacha secret)
        let mut server = ScreenFrameCodec::chacha(shared_secret2, stream_connection_id, true);
        let mut client = ScreenFrameCodec::chacha(shared_secret2, stream_connection_id, false);
        let mut dst = BytesMut::new();
        server.encode(ret, &mut dst).unwrap();
        let ret = client.decode(&mut dst).unwrap().unwrap();

        assert_eq!(ret.header.body_size as usize, data.len());
        assert_eq!(ret.data, data);

        println!("{:?}", ret);
    }

    #[test]
    fn test_encrypt_decrypt_progressive_three_decode_calls() {
        setup_test_logger(true);

        let shared_secret: [u8; 32] = *b"12345678123456781234567812345678";
        let stream_connection_id: u64 = 12345678;

        let mut server = ScreenFrameCodec::chacha(shared_secret, stream_connection_id, true);
        let data: Vec<u8> = (0..8192).map(|i| ((i * 31) % 251) as u8).collect();
        let mut bm = BytesMut::new();
        bm.extend_from_slice(&data);

        let frame = ScreenFrame::new(
            ScreenFrameHeader {
                opcode: ScreenOpCode::VideoFrame,
                body_size: 0,
                small_param: [0u8; 3],
                params: [Value64::default(); 15],
            },
            bm,
        );

        let mut encoded = BytesMut::new();
        server.encode(frame, &mut encoded).unwrap();

        let header_size = ScreenFrameHeader::size();
        let encrypted_body_size = ScreenFrameHeader::from_bytes(&encoded).unwrap().body_size as usize;
        let frame_size = header_size + encrypted_body_size;
        assert_eq!(frame_size, encoded.len());

        // Force 3-step decode:
        // 1) header + tiny encrypted prefix (<16),
        // 2) middle chunk (progressive decrypt),
        // 3) final tail (tag verification + final decrypt).
        let first_chunk_end = header_size + 8;
        let final_tail_len = 32;
        let second_chunk_end = frame_size - final_tail_len;
        assert!(first_chunk_end < second_chunk_end);

        let mut client = ScreenFrameCodec::chacha(shared_secret, stream_connection_id, false);
        let mut rx = BytesMut::new();

        rx.extend_from_slice(&encoded[..first_chunk_end]);
        let ret = client.decode(&mut rx).unwrap();
        assert!(ret.is_none());
        let state_after_first = client.header.as_ref().unwrap();
        assert!(state_after_first.1.is_some());
        assert_eq!(state_after_first.2, 0);

        rx.extend_from_slice(&encoded[first_chunk_end..second_chunk_end]);
        let ret = client.decode(&mut rx).unwrap();
        assert!(ret.is_none());
        let state_after_second = client.header.as_ref().unwrap();
        assert!(state_after_second.1.is_some());
        assert!(state_after_second.2 > 0);

        rx.extend_from_slice(&encoded[second_chunk_end..]);
        let mut ret = client.decode(&mut rx).unwrap().unwrap();
        assert_eq!(ret.header.body_size as usize, data.len());
        assert_eq!(ret.data, data);

        // For decoded frames we expect header_buf|data|chacha_tag_buf to be adjacent in case we want to use .unsplit() fast-path when proxying the frame
        unsafe {
            assert_eq!(ret.header_buf.as_ptr().add(ret.header_buf.len()), ret.data.as_ptr());
            assert_eq!(ret.data.as_ptr().add(ret.data.len()), ret.chacha_tag_buf.as_ptr());

            assert_eq!(ret.chacha_tag_buf.len(), 16);
            assert_eq!(ret.header_buf.len(), 128);

            assert!(ret.as_adjacent_slice_mut().is_some());
        }
    }

    #[test]
    fn test_decode_realigns_empty_buffer_before_second_frame() {
        setup_test_logger(true);
        const WARMED_UP_CAPACITY: usize = 64 * 1024;
        // Simulate reading a screen frame with a random, unaligned length
        const WARMUP_FRAME_BODY_LEN: usize = WARMED_UP_CAPACITY - (8 * 1024) - 111;
        const SECOND_FRAME_BODY_LEN: usize = 512;

        let shared_secret: [u8; 32] = *b"12345678123456781234567812345678";
        let stream_connection_id: u64 = 12345678;

        let mut server = ScreenFrameCodec::chacha(shared_secret, stream_connection_id, true);
        let mut client = ScreenFrameCodec::chacha(shared_secret, stream_connection_id, false);

        let warmup_data: Vec<u8> = (0..WARMUP_FRAME_BODY_LEN)
            .map(|i| ((i * 31) % 251) as u8)
            .collect();
        let data2: Vec<u8> = (0..SECOND_FRAME_BODY_LEN)
            .map(|i| ((i * 17) % 251) as u8)
            .collect();

        let make_frame = |data: &[u8]| {
            let mut bm = BytesMut::new();
            bm.extend_from_slice(data);
            ScreenFrame::new(
                ScreenFrameHeader {
                    opcode: ScreenOpCode::VideoFrame,
                    body_size: 0,
                    small_param: [0u8; 3],
                    params: [Value64::default(); 15],
                },
                bm,
            )
        };

        let mut rx = BytesMut::new();

        // Warm up the RX buffer to a realistic large-frame capacity before testing the empty-buffer realign path.
        rx.reserve(WARMED_UP_CAPACITY);
        assert!(rx.capacity() >= WARMED_UP_CAPACITY);

        let mut encoded1 = BytesMut::new();
        server
            .encode(make_frame(&warmup_data), &mut encoded1)
            .unwrap();

        rx.extend_from_slice(&encoded1);
        let ret1 = client.decode(&mut rx).unwrap().unwrap();
        assert_eq!(ret1.data, warmup_data);
        assert!(rx.is_empty());
        assert!(rx.capacity() > 0);

        // Trigger the empty-buffer realign branch before any new frame bytes are appended.
        let ret = client.decode(&mut rx).unwrap();
        assert!(ret.is_none());
        assert!(rx.is_empty());
        assert_eq!(
            rx.as_ptr().align_offset(64),
            0,
            "empty warmed-up buffer should be 64B-aligned after realign"
        );
        assert!(rx.capacity() > 0);

        let mut encoded2 = BytesMut::new();
        server.encode(make_frame(&data2), &mut encoded2).unwrap();
        rx.extend_from_slice(&encoded2);
        let ret2 = client.decode(&mut rx).unwrap().unwrap();

        assert_eq!(ret2.header.body_size as usize, data2.len());
        assert_eq!(ret2.data, data2);
        assert!(rx.is_empty());
    }

    #[test]
    fn test_aes_roundtrip() {
        let audio_key = *b"1234567890abcdef";
        let stream_connection_id: u64 = 0x1234_5678_9abc_def0;

        let mut server = ScreenFrameCodec::aes(audio_key, stream_connection_id, false);
        let mut client = ScreenFrameCodec::aes(audio_key, stream_connection_id, true);

        let data: Vec<u8> = (0..4096).map(|i| ((i * 17) % 251) as u8).collect();
        let frame = ScreenFrame::new(
            ScreenFrameHeader {
                opcode: ScreenOpCode::VideoFrame,
                body_size: 0,
                small_param: [0u8; 3],
                params: [Value64::default(); 15],
            },
            BytesMut::from(&data[..]),
        );

        let mut encoded = BytesMut::new();
        server.encode(frame, &mut encoded).unwrap();

        let decoded = client.decode(&mut encoded).unwrap().unwrap();
        assert_eq!(decoded.header.body_size as usize, data.len());
        assert_eq!(decoded.data.as_ref(), data.as_slice());
        assert!(decoded.chacha_tag_buf.is_empty());
    }

    #[test]
    fn test_aes_progressive_three_decode_calls() {
        let audio_key = *b"1234567890abcdef";
        let stream_connection_id: u64 = 0x1234_5678_9abc_def0;

        let mut server = ScreenFrameCodec::aes(audio_key, stream_connection_id, false);
        let mut client = ScreenFrameCodec::aes(audio_key, stream_connection_id, true);

        let data: Vec<u8> = (0..4096 * 1024).map(|i| ((i * 13) % 251) as u8).collect();
        let frame = ScreenFrame::new(
            ScreenFrameHeader {
                opcode: ScreenOpCode::VideoFrame,
                body_size: 0,
                small_param: [0u8; 3],
                params: [Value64::default(); 15],
            },
            BytesMut::from(&data[..]),
        );

        let mut encoded = BytesMut::new();
        server.encode(frame, &mut encoded).unwrap();

        let header_size = ScreenFrameHeader::size();
        let encrypted_body_size = ScreenFrameHeader::from_bytes(&encoded).unwrap().body_size as usize;
        let frame_size = header_size + encrypted_body_size;
        assert_eq!(frame_size, encoded.len());

        let first_chunk_end = header_size + 64;
        let final_tail_len = 256;
        let second_chunk_end = frame_size - final_tail_len;
        assert!(first_chunk_end < second_chunk_end);

        let mut rx = BytesMut::new();

        rx.extend_from_slice(&encoded[..first_chunk_end]);
        let ret = client.decode(&mut rx).unwrap();
        assert!(ret.is_none());
        let state_after_first = client.header.as_ref().unwrap();
        assert!(state_after_first.1.is_none());
        assert!(state_after_first.2 > 0);
        let first_decrypted_payload_len = state_after_first.2;

        rx.extend_from_slice(&encoded[first_chunk_end..second_chunk_end]);
        let ret = client.decode(&mut rx).unwrap();
        assert!(ret.is_none());
        let state_after_second = client.header.as_ref().unwrap();
        assert!(state_after_second.1.is_none());
        assert!(state_after_second.2 > first_decrypted_payload_len);

        rx.extend_from_slice(&encoded[second_chunk_end..]);
        let decoded = client.decode(&mut rx).unwrap().unwrap();
        assert_eq!(decoded.header.body_size as usize, data.len());
        assert_eq!(decoded.data.as_ref(), data.as_slice());
        assert!(decoded.chacha_tag_buf.is_empty());
    }

    #[test]
    fn test_aes_progressive_three_decode_calls_with_prefetch() {
        setup_test_logger(false);

        // Prefetch starts empty and runs asynchronously. Wait for observable cache
        // readiness rather than racing the worker with back-to-back decode calls.
        let wait_for_prefetch = |codec: &ScreenFrameCodec, chunk_len: usize| {
            let Some(super::ScreenFrameCodecCrypto::AesCtr { read_cipher, .. }) = &codec.crypto else {
                unreachable!();
            };
            let minimum = chunk_len.min(read_cipher.status().requested_bytes);
            assert!(minimum > 0, "AES prefetch must be enabled for this PoC");
            let deadline = Instant::now() + Duration::from_secs(5);
            while read_cipher.status().available_bytes < minimum {
                assert!(Instant::now() < deadline, "AES prefetch worker did not fill the cache");
                std::thread::sleep(Duration::from_millis(1));
            }
        };

        let audio_key = *b"1234567890abcdef";
        let stream_connection_id: u64 = 0x1234_5678_9abc_def0;

        let mut server = ScreenFrameCodec::aes_with_flags(audio_key, stream_connection_id, false, ScreenFrameCodecFlags::PreferPrefetchAes);
        let mut client = ScreenFrameCodec::aes_with_flags(audio_key, stream_connection_id, true, ScreenFrameCodecFlags::PreferPrefetchAes);

        let data: Vec<u8> = (0..4096 * 1024).map(|i| ((i * 13) % 251) as u8).collect();
        let frame = ScreenFrame::new(
            ScreenFrameHeader {
                opcode: ScreenOpCode::VideoFrame,
                body_size: 0,
                small_param: [0u8; 3],
                params: [Value64::default(); 15],
            },
            BytesMut::from(&data[..]),
        );

        let mut encoded = BytesMut::new();
        server.encode(frame, &mut encoded).unwrap();

        let header_size = ScreenFrameHeader::size();
        let encrypted_body_size = ScreenFrameHeader::from_bytes(&encoded).unwrap().body_size as usize;
        let frame_size = header_size + encrypted_body_size;
        assert_eq!(frame_size, encoded.len());

        let first_chunk_end = header_size + 64;
        let final_tail_len = 256;
        let second_chunk_end = frame_size - final_tail_len;
        assert!(first_chunk_end < second_chunk_end);

        let mut rx = BytesMut::new();

        wait_for_prefetch(&client, first_chunk_end - header_size);
        rx.extend_from_slice(&encoded[..first_chunk_end]);
        let ret = client.decode(&mut rx).unwrap();
        assert!(ret.is_none());
        let state_after_first = client.header.as_ref().unwrap();
        assert!(state_after_first.1.is_none());
        assert!(state_after_first.2 > 0);
        let first_decrypted_payload_len = state_after_first.2;

        wait_for_prefetch(&client, second_chunk_end - first_chunk_end);
        rx.extend_from_slice(&encoded[first_chunk_end..second_chunk_end]);
        let ret = client.decode(&mut rx).unwrap();
        assert!(ret.is_none());
        let state_after_second = client.header.as_ref().unwrap();
        assert!(state_after_second.1.is_none());
        assert!(state_after_second.2 > first_decrypted_payload_len);

        wait_for_prefetch(&client, final_tail_len);
        rx.extend_from_slice(&encoded[second_chunk_end..]);
        let decoded = client.decode(&mut rx).unwrap().unwrap();
        assert_eq!(decoded.header.body_size as usize, data.len());
        assert_eq!(decoded.data.as_ref(), data.as_slice());
        assert!(decoded.chacha_tag_buf.is_empty());
    }

    #[test]
    fn test_chacha_prefetch_uses_ewma_in_both_directions() {
        let mut tx = ScreenFrameCodec::chacha_with_flags([0x24; 32], 19, false, ScreenFrameCodecFlags::PreferPrefetchChacha);
        let mut rx = ScreenFrameCodec::chacha_with_flags([0x24; 32], 19, true, ScreenFrameCodecFlags::PreferPrefetchChacha);

        for (frame_len, expected_target) in [(128 * 1024, 128 * 1024), (64 * 1024, 128 * 1024)] {
            let frame = ScreenFrame::new(
                ScreenFrameHeader {
                    opcode: ScreenOpCode::VideoFrame,
                    body_size: 0,
                    small_param: [0; 3],
                    params: [Value64::default(); 15],
                },
                BytesMut::from(&vec![0x5a; frame_len][..]),
            );
            let mut wire = BytesMut::new();
            tx.encode(frame, &mut wire).unwrap();
            assert_eq!(rx.decode(&mut wire).unwrap().unwrap().data.len(), frame_len);

            let Some(super::ScreenFrameCodecCrypto::Chacha {
                write_cipher,
                frame_size_tx,
                ..
            }) = &tx.crypto
            else {
                unreachable!()
            };
            let Some(super::ScreenFrameCodecCrypto::Chacha {
                read_cipher,
                frame_size_rx,
                ..
            }) = &rx.crypto
            else {
                unreachable!()
            };

            assert_eq!(frame_size_tx.max_seen, frame_size_rx.max_seen);
            assert_eq!(write_cipher.prefetch_status().unwrap().requested_bytes, expected_target);
            assert_eq!(read_cipher.prefetch_status().unwrap().requested_bytes, expected_target);
        }
    }

    #[test]
    fn test_chacha_prefetch_next_frame_progressive() {
        setup_test_logger(false);

        let mut tx = ScreenFrameCodec::chacha_with_flags([0x42; 32], 17, false, ScreenFrameCodecFlags::PreferPrefetchChacha);
        let mut rx = ScreenFrameCodec::chacha_with_flags([0x42; 32], 17, true, ScreenFrameCodecFlags::PreferPrefetchChacha);
        let data: Vec<u8> = (0..8193).map(|i| (i * 31) as u8).collect();
        let make_frame = || {
            ScreenFrame::new(
                ScreenFrameHeader {
                    opcode: ScreenOpCode::VideoFrame,
                    body_size: 0,
                    small_param: [0; 3],
                    params: [Value64::default(); 15],
                },
                BytesMut::from(&data[..]),
            )
        };
        let status = |codec: &ScreenFrameCodec| {
            let Some(super::ScreenFrameCodecCrypto::Chacha {
                read_cipher, counter_rx, ..
            }) = &codec.crypto
            else {
                unreachable!()
            };
            (counter_rx.0, read_cipher.prefetch_status())
        };
        let tx_status = |codec: &ScreenFrameCodec| {
            let Some(super::ScreenFrameCodecCrypto::Chacha {
                write_cipher, counter_tx, ..
            }) = &codec.crypto
            else {
                unreachable!()
            };
            (counter_tx.0, write_cipher.prefetch_status())
        };
        assert!(tx_status(&tx).1.is_none());
        assert!(status(&rx).1.is_none());
        let mut wire = BytesMut::new();
        tx.encode(make_frame(), &mut wire).unwrap();
        assert_eq!(rx.decode(&mut wire).unwrap().unwrap().data.as_ref(), &data);
        let expected_target = data.len();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let (next_nonce, target) = status(&rx);
            let target = target.unwrap();
            assert_eq!((next_nonce, target.nonce, target.requested_bytes), (1, 1, expected_target));
            let (tx_nonce, tx_target) = tx_status(&tx);
            let tx_target = tx_target.unwrap();
            assert_eq!((tx_nonce, tx_target.nonce, tx_target.requested_bytes), (1, 1, expected_target));
            if target.available_bytes == expected_target && tx_target.available_bytes == expected_target {
                break;
            }
            assert!(Instant::now() < deadline, "ChaCha worker did not fill cache");
            std::thread::sleep(Duration::from_millis(1));
        }
        // A non-video frame neither consumes the predicted nonce nor resets it.
        tx.encode(
            ScreenFrame::new(
                ScreenFrameHeader {
                    opcode: ScreenOpCode::VideoConfig,
                    body_size: 0,
                    small_param: [0; 3],
                    params: [Value64::default(); 15],
                },
                BytesMut::new(),
            ),
            &mut wire,
        )
        .unwrap();
        assert_eq!(rx.decode(&mut wire).unwrap().unwrap().header.opcode, ScreenOpCode::VideoConfig);
        assert_eq!(status(&rx).0, 1);
        assert_eq!(status(&rx).1.unwrap().available_bytes, expected_target);
        assert_eq!(tx_status(&tx).0, 1);
        assert_eq!(tx_status(&tx).1.unwrap().available_bytes, expected_target);
        tx.encode(make_frame(), &mut wire).unwrap();
        let (tx_nonce, tx_target) = tx_status(&tx);
        let tx_target = tx_target.unwrap();
        assert_eq!((tx_nonce, tx_target.nonce, tx_target.requested_bytes), (2, 2, expected_target));
        let header_size = ScreenFrameHeader::size();
        let mut input = wire.split_to(header_size + 8);
        assert!(rx.decode(&mut input).unwrap().is_none());
        assert_eq!(status(&rx).0, 2);
        assert_eq!(status(&rx).1.unwrap().nonce, 1);
        // Leave only the tag for the final decode, exercising an unaligned payload.
        input.extend_from_slice(&wire.split_to(wire.len() - 16));
        assert!(rx.decode(&mut input).unwrap().is_none());
        let mid = status(&rx).1.unwrap();
        assert_eq!(mid.nonce, 1);
        assert!(mid.used_bytes > 0);
        assert_eq!(mid.used_bytes, rx.header.as_ref().unwrap().2);
        input.extend_from_slice(&wire);
        let mut decoded = rx.decode(&mut input).unwrap().unwrap();
        assert_eq!(decoded.data.as_ref(), &data);
        assert!(decoded.as_adjacent_slice_mut().is_some());
        let (next_nonce, next) = status(&rx);
        let next = next.unwrap();
        assert_eq!(
            (next_nonce, next.nonce, next.requested_bytes, next.used_bytes),
            (2, 2, expected_target, 0)
        );
    }

    #[test]
    fn test_unencrypted_config_and_video_frame_roundtrip() {
        let mut server = ScreenFrameCodec::unencrypted();
        let mut client = ScreenFrameCodec::unencrypted();
        let clock = MediaClockSession::new();

        let config = AvccConfigExtended {
            hevc: false,
            avcc: AvccConfig {
                nal_size_len: 4,
                sps_pps: vec![
                    0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0xE0, 0x1E, 0x89, 0x8B, 0x00, 0x00, 0x00, 0x01, 0x68, 0xCE,
                ],
            },
            video_latency: Duration::from_millis(42),
            width: 800,
            height: 480,
            respect_timestamps: false,
        };

        let mut encoded = BytesMut::new();
        server
            .encode(ScreenFrame::config(&config), &mut encoded)
            .unwrap();
        let decoded_config_frame = client.decode(&mut encoded).unwrap().unwrap();
        assert_eq!(decoded_config_frame.header.opcode, ScreenOpCode::VideoConfig);
        assert_eq!(
            decoded_config_frame
                .config_decode(config.video_latency)
                .unwrap(),
            config
        );

        let annexb = BytesMut::from(&[0x00, 0x00, 0x00, 0x01, 0x65, 0x88, 0x84, 0x21][..]);
        let pts = Instant::now();
        let video_frame = ScreenFrame::video_annexb(annexb.clone(), config.avcc.nal_size_len, pts, &clock).unwrap();

        let mut encoded = BytesMut::new();
        server.encode(video_frame, &mut encoded).unwrap();
        let decoded_video_frame = client.decode(&mut encoded).unwrap().unwrap();
        assert_eq!(decoded_video_frame.header.opcode, ScreenOpCode::VideoFrame);
        assert!(decoded_video_frame.chacha_tag_buf.is_empty());

        let decoded_video = decoded_video_frame
            .video_decode(&config, &clock)
            .unwrap()
            .unwrap();
        assert_eq!(decoded_video.width, config.width);
        assert_eq!(decoded_video.height, config.height);
        assert_eq!(decoded_video.data, annexb);
        assert_eq!(decoded_video.config.as_ref(), Some(&config));
        assert_eq!(decoded_video.is_keyframe, Some(true));
    }
}
