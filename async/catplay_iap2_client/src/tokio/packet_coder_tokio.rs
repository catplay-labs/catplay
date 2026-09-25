use bytes::BytesMut;
use catplay_iap2_link::{PacketCoder, PacketOrDetect};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

pub struct PacketCoderTokio {
    coder: PacketCoder,
}

impl PacketCoderTokio {
    pub fn new(coder: PacketCoder) -> Self {
        Self { coder }
    }
}

impl Decoder for PacketCoderTokio {
    type Item = PacketOrDetect;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<PacketOrDetect>, io::Error> {
        self.coder
            .decode(src)
            .map_err(|e| io::Error::other(format!("{:?}", e)))
    }
}

impl Encoder<PacketOrDetect> for PacketCoderTokio {
    type Error = io::Error;

    fn encode(&mut self, item: PacketOrDetect, dst: &mut BytesMut) -> Result<(), io::Error> {
        self.coder
            .encode(item, dst)
            .map_err(|e| io::Error::other(format!("{:?}", e)))
    }
}
