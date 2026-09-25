use std::io;

use tokio::net::tcp::OwnedWriteHalf;
use tokio_util::codec::Encoder;

use crate::{BytesMutQueue, CItem, EncoderComposite, TcpSession};

pub trait TcpSink<S: TcpSession>: Send + Sync {
    /// Get a mutable reference to codec to change it's configuration.
    fn codec_mut(&mut self) -> &mut S::Codec;

    // Check if the socket hasn't entered unwritable status due to high TCP backlog.
    fn is_writable(&self) -> bool;

    /// Write a new object into sink.
    ///
    /// The object is guaranteed to be serialized into bytes within current context, with current state of the decoder.
    fn write(&mut self, item: CItem<S::Codec>) -> Result<(), S::Error>;

    /// Write a new object into sink.
    ///
    /// The object is guaranteed to be serialized into bytes within current context, with current state of the decoder.
    fn write_composite(&mut self, item: CItem<S::Codec>) -> Result<(), S::Error>
    where
        S::Codec: EncoderComposite<CItem<S::Codec>>,
        S::Error: From<<S::Codec as EncoderComposite<CItem<S::Codec>>>::Error>;
}

pub struct TcpSinkBuffer<'a, S: TcpSession> {
    pub composites: &'a mut BytesMutQueue,
    pub codec: &'a mut S::Codec,
}

impl<'a, S: TcpSession> TcpSinkBuffer<'a, S> {
    pub fn new(composites: &'a mut BytesMutQueue, codec: &'a mut S::Codec) -> Self {
        Self { composites, codec }
    }

    pub(crate) fn flush_queue(stream: &OwnedWriteHalf, composites: &mut BytesMutQueue) -> io::Result<()> {
        while !composites.is_empty() {
            let result = if composites.len() == 1 {
                match composites.first() {
                    Some(first) => stream.try_write(first),
                    None => Ok(0),
                }
            } else {
                stream.try_write_vectored(composites.as_iovecs())
            };

            match result {
                Ok(n) if n > 0 => composites.drain_written(n),
                Ok(_) => break,
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}

impl<'a, S: TcpSession> TcpSink<S> for TcpSinkBuffer<'a, S> {
    fn codec_mut(&mut self) -> &mut S::Codec {
        self.codec
    }

    fn is_writable(&self) -> bool {
        // TODO: track unwritable status
        true
    }

    fn write(&mut self, item: CItem<S::Codec>) -> Result<(), S::Error> {
        self.codec.encode(item, self.composites.tail_mut())?;
        Ok(())
    }

    fn write_composite(&mut self, item: CItem<S::Codec>) -> Result<(), S::Error>
    where
        S::Codec: EncoderComposite<CItem<S::Codec>>,
        S::Error: From<<S::Codec as EncoderComposite<CItem<S::Codec>>>::Error>,
    {
        self.codec
            .encode_composite(item, &mut |b| self.composites.push(b))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fmt,
        io::{self, ErrorKind},
    };

    use async_trait::async_trait;
    use bytes::BytesMut;
    use catplay_util::{AsyncShutdown, EventSleeper};
    use tokio_util::codec::{Decoder, Encoder};

    use crate::{EncoderComposite, TcpSession, TcpSink};

    use super::{BytesMutQueue, TcpSinkBuffer};

    fn bytes(byte: u8, len: usize) -> BytesMut {
        BytesMut::from(vec![byte; len].as_slice())
    }

    fn collect_iovecs(queue: &mut BytesMutQueue) -> Vec<u8> {
        queue
            .as_iovecs()
            .iter()
            .flat_map(|slice| slice.iter().copied())
            .collect()
    }

    #[derive(Clone, Debug)]
    struct TestError {
        kind: ErrorKind,
    }

    impl From<io::Error> for TestError {
        fn from(value: io::Error) -> Self {
            Self { kind: value.kind() }
        }
    }

    impl From<TestError> for io::Error {
        fn from(value: TestError) -> Self {
            io::Error::new(value.kind, "test error")
        }
    }

    #[derive(Default)]
    struct TestCodec;

    impl Decoder for TestCodec {
        type Item = Vec<u8>;
        type Error = TestError;

        fn decode(&mut self, _src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
            Ok(None)
        }
    }

    impl Encoder<Vec<u8>> for TestCodec {
        type Error = TestError;

        fn encode(&mut self, item: Vec<u8>, dst: &mut BytesMut) -> Result<(), Self::Error> {
            dst.extend_from_slice(&item);
            Ok(())
        }
    }

    impl EncoderComposite<Vec<u8>> for TestCodec {
        type Error = TestError;

        fn encode_composite(&mut self, item: Vec<u8>, callback: &mut dyn FnMut(BytesMut)) -> Result<(), Self::Error> {
            let split = item.len() / 2;
            callback(BytesMut::from(&item[..split]));
            callback(BytesMut::from(&item[split..]));
            Ok(())
        }
    }

    struct TestSession;

    impl AsyncShutdown for TestSession {}
    impl EventSleeper for TestSession {}

    #[async_trait]
    impl TcpSession for TestSession {
        type Codec = TestCodec;
        type Error = TestError;

        fn init_codec(&mut self) -> Result<Self::Codec, Self::Error> {
            Ok(TestCodec)
        }

        async fn on_msg(&mut self, _sink: &mut dyn TcpSink<Self>, _msg: Vec<u8>) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}", self.kind)
        }
    }

    fn mixed_write_and_composite_queue() -> BytesMutQueue {
        let mut queue = BytesMutQueue::new();
        let mut codec = TestCodec;
        let mut sink = TcpSinkBuffer::<TestSession>::new(&mut queue, &mut codec);

        sink.write(vec![0x01, 0x02, 0x03]).unwrap();
        sink.write_composite(vec![0x10, 0x11, 0x12, 0x13]).unwrap();
        sink.write(vec![0x20, 0x21]).unwrap();

        queue
    }

    #[test]
    fn drain_written_partial_front_preserves_tail() {
        let mut queue = BytesMutQueue::new();
        queue.tail_mut().extend_from_slice(&[0xcc; 5]);
        queue.push(bytes(0xaa, 3));
        queue.push(bytes(0xdd, 10));
        queue.push(bytes(0xee, 16));

        queue.drain_written(5 + 3 + 4);

        assert_eq!(queue.queue.len(), 1);
        assert_eq!(queue.queue.front().unwrap().as_ref(), &[0xdd; 6]);
        assert_eq!(queue.tail.as_ref(), &[0xee; 16]);
        assert_eq!(queue.queued_bytes(), 22);
    }

    #[test]
    fn drain_written_partial_two_buffers_preserves_remaining_stream() {
        let cases = [
            (
                3,
                vec![0xaa; 2]
                    .into_iter()
                    .chain(vec![0xbb; 8])
                    .collect::<Vec<_>>(),
            ),
            (5 + 4, vec![0xbb; 4]),
        ];

        for (written, expected_remaining) in cases {
            let mut queue = BytesMutQueue::new();
            queue.push(bytes(0xaa, 5));
            queue.push(bytes(0xbb, 8));

            queue.drain_written(written);

            assert_eq!(collect_iovecs(&mut queue), expected_remaining);
            assert_eq!(queue.queued_bytes(), 13 - written);
        }
    }

    #[test]
    fn mixed_write_and_write_composite_survives_partial_writes() {
        let expected = vec![0x01, 0x02, 0x03, 0x10, 0x11, 0x12, 0x13, 0x20, 0x21];

        for written in 0..=expected.len() {
            let mut queue = mixed_write_and_composite_queue();
            assert_eq!(collect_iovecs(&mut queue), expected);

            // Simulates try_write/try_write_vectored reporting WouldBlock before any bytes are accepted.
            assert_eq!(collect_iovecs(&mut queue), expected);

            queue.drain_written(written);

            assert_eq!(collect_iovecs(&mut queue), expected[written..]);
            assert_eq!(queue.queued_bytes(), expected.len() - written);
        }
    }

    #[test]
    fn mixed_write_and_write_composite_survives_incremental_writes_with_wouldblock_gaps() {
        let expected = vec![0x01, 0x02, 0x03, 0x10, 0x11, 0x12, 0x13, 0x20, 0x21];
        let mut queue = mixed_write_and_composite_queue();

        for written in [1, 0, 2, 0, 1, 3, 0, 2] {
            let before = expected.len() - queue.queued_bytes();
            if written == 0 {
                assert_eq!(collect_iovecs(&mut queue), expected[before..]);
            } else {
                queue.drain_written(written);
                let after = before + written;
                assert_eq!(collect_iovecs(&mut queue), expected[after..]);
            }
        }

        assert!(queue.is_empty());
    }
}
