use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use catplay_tokio::{UdpSession, UdpSocketPeer};
use catplay_util::{EventSleeper, EventToken, sleep};
use log::debug;

use crate::rtsp_frame::RtspError;

pub struct KeepAliveServer {
    last_received: Option<Instant>,
}

impl KeepAliveServer {
    pub fn new() -> Self {
        Self { last_received: None }
    }
}

#[async_trait]
impl UdpSession for KeepAliveServer {
    type Error = RtspError;

    fn on_datagram(&mut self, data: &mut [u8], _peer: SocketAddr, _sink: &dyn UdpSocketPeer<Self>) -> Result<(), Self::Error> {
        if data.len() > 8 {
            debug!("Ignoring keep-alive, too big at {}", data.len());
            return Ok(());
        }

        // No response to keep alive is expected in this protocol
        // TODO: use it for tracking session timeout after idle transition
        debug!("Got keepalive: {data:?}");
        self.last_received.replace(Instant::now());
        Ok(())
    }

    fn reconcile(&mut self, _peer: Option<&dyn UdpSocketPeer<Self>>) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl EventSleeper for KeepAliveServer {
    async fn sleep(&mut self) -> Option<EventToken> {
        sleep(Duration::from_millis(1000)).await;
        Some(EventToken(1))
    }
}
