use std::{mem, time::Instant};

use bytes::{Buf, BytesMut};
use catplay_hap::aes::Aes128Ctr;
use catplay_hap::cipher::{HomeKitChaChaNonce, HomeKitCipher};
use catplay_tokio::{BytesMutUtil, Decoder, Encoder, EncoderComposite};
use log::{debug, trace};

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
    },
    AesCtr {
        read_cipher: Aes128Ctr,
        write_cipher: Aes128Ctr,
    },
}

pub struct ScreenFrameCodec {
    pub header: Option<(ScreenFrameHeader, Option<HomeKitChaChaNonce>, usize)>,
    crypto: Option<ScreenFrameCodecCrypto>,
    pub chunks_rx: usize,
}

impl ScreenFrameCodec {
    const MAX_PAYLOAD_SIZE: usize = 4 * 1024 * 1024; // 4MB
    const CHACHA_TAG_LEN: usize = 16;

    pub fn new(encryption: AirPlayStreamEncryption, stream_connection_id: u64, server: bool) -> Self {
        match encryption {
            // There was never support for unencrypted screen sessions.
            // ET = None maps to AES session where session_key before derivation is all zeroes.
            // AirPlayStreamEncryption::Unconfigured | AirPlayStreamEncryption::None => Self::unencrypted(),
            AirPlayStreamEncryption::Unconfigured | AirPlayStreamEncryption::None => Self::aes([0u8; 16], stream_connection_id),
            AirPlayStreamEncryption::Aes { key, .. } => Self::aes(key, stream_connection_id),
            AirPlayStreamEncryption::ChaCha { shared_secret } => Self::chacha(shared_secret, stream_connection_id, server),
        }
    }

    pub fn chacha(shared_secret: [u8; 32], stream_connection_id: u64, server: bool) -> Self {
        let (read_cipher, write_cipher) =
            create_chacha_ciphers(&shared_secret, AirPlayCipherSaltType::DataStream { stream_connection_id }, server);

        Self {
            header: None,
            crypto: Some(ScreenFrameCodecCrypto::Chacha {
                read_cipher,
                write_cipher,
                counter_rx: HomeKitChaChaNonce(0),
                counter_tx: HomeKitChaChaNonce(0),
            }),
            chunks_rx: 0,
        }
    }

    pub fn aes(session_key: [u8; 16], stream_connection_id: u64) -> Self {
        let (video_key, video_iv) = derive_aes_stream_keys(&session_key, stream_connection_id);
        let read_cipher = Aes128Ctr::new(&video_key, &video_iv);
        let write_cipher = Aes128Ctr::new(&video_key, &video_iv);

        Self {
            header: None,
            crypto: Some(ScreenFrameCodecCrypto::AesCtr { read_cipher, write_cipher }),
            chunks_rx: 0,
        }
    }

    pub fn unencrypted() -> Self {
        Self {
            header: None,
            crypto: None,
            chunks_rx: 0,
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
        let (read_cipher, counter_rx) = match self.crypto.as_mut().unwrap() {
            ScreenFrameCodecCrypto::Chacha {
                read_cipher, counter_rx, ..
            } => (read_cipher, counter_rx),
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
        let decrypted_chunk = read_cipher.decrypt_progressive(
            body_and_tag,
            header_raw,
            nonce,
            decrypted_payload_len,
            current_body_and_tag_len,
            body_size,
        )?;
        let crypto_took = Instant::now() - crypto_start;
        let decrypted_chunk_len = decrypted_chunk.len();

        debug!(
            "Decrypted video frame chunk {}/x of {chunk_len}/{body_size}b in {crypto_took:?} capacity={} len={} ptr={:?} decrypted_chunk={decrypted_chunk_len} final={is_final_chunk}",
            chunk_index,
            src.capacity(),
            src.len(),
            src.as_ptr()
        );

        Ok(decrypted_chunk_len)
    }

    fn decrypt_video_frame_chunk_aes(
        &mut self,
        src: &mut BytesMut,
        header_size: usize,
        decrypted_payload_len: usize,
        current_body_len: usize,
    ) -> usize {
        let read_cipher = match self.crypto.as_mut().unwrap() {
            ScreenFrameCodecCrypto::AesCtr { read_cipher, .. } => read_cipher,
            _ => unreachable!(),
        };
        let decrypted_chunk_len = current_body_len.saturating_sub(decrypted_payload_len);
        if decrypted_chunk_len > 0 {
            read_cipher.apply_keystream(&mut src[header_size + decrypted_payload_len..header_size + current_body_len]);
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
            Some(ScreenFrameCodecCrypto::AesCtr { .. }) => {
                Ok(self.decrypt_video_frame_chunk_aes(src, header_size, decrypted_payload_len, current_body_and_tag_len))
            }
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
                } else if video_is_encrypted {
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
                    let data = src.split_to(body_size);
                    Ok(Some(ScreenFrame::with_buffers(header, data, header_buf)))
                } else {
                    let header_buf = src.split_to(header_size);
                    let data = src.split_to(body_size);
                    Ok(Some(ScreenFrame::with_buffers(header, data, header_buf)))
                }
            }
        }
    }

    pub fn encode_composite(&mut self, mut frame: ScreenFrame, callback: &mut dyn FnMut(BytesMut)) -> RtspResult<()> {
        frame.header.body_size = frame.data.len() as u32;

        if frame.header.opcode == ScreenOpCode::VideoFrame
            && let Some(crypto) = self.crypto.as_mut()
        {
            match crypto {
                ScreenFrameCodecCrypto::Chacha {
                    write_cipher, counter_tx, ..
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
                    write_cipher.apply_keystream(&mut frame.data);
                    let crypto_took = Instant::now() - crypto_start;
                    debug!("Encrypted AES video frame of {}b in {crypto_took:?}", frame.data.len());

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
        screen::{ScreenFrame, ScreenFrameCodec, ScreenFrameHeader, ScreenOpCode, Value64},
        video::{AvccConfig, AvccConfigExtended},
    };

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

        let mut server = ScreenFrameCodec::aes(audio_key, stream_connection_id);
        let mut client = ScreenFrameCodec::aes(audio_key, stream_connection_id);

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

        let mut server = ScreenFrameCodec::aes(audio_key, stream_connection_id);
        let mut client = ScreenFrameCodec::aes(audio_key, stream_connection_id);

        let data: Vec<u8> = (0..8192).map(|i| ((i * 13) % 251) as u8).collect();
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
