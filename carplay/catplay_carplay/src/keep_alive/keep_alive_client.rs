use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use catplay_tokio::{UdpSession, UdpSocketPeer};
use catplay_util::{EventSleeper, EventToken};
use log::debug;
use tokio::time::sleep_until;

use crate::rtsp_frame::RtspError;

pub struct KeepAliveClient {
    start: Option<Instant>,

    pending_send: bool,
    last_sent: Option<Instant>,
}

impl KeepAliveClient {
    const PING_INTERVAL: Duration = Duration::from_millis(10000);

    pub fn new() -> Self {
        Self {
            start: None,
            pending_send: false,
            last_sent: None,
        }
    }

    pub fn send_ping(&mut self, sink: &dyn UdpSocketPeer<Self>) {
        let mut buf = [0u8; 4];
        rand::fill(&mut buf);

        match sink.send(&buf) {
            Ok(_) => {}
            Err(err) => {
                debug!("Failed to send keep alive ping: {err:?}");
            }
        }
    }
}

impl UdpSession for KeepAliveClient {
    type Error = RtspError;

    fn on_eof(&mut self, error: Option<Self::Error>) {
        debug!("EOF on keep alive socket! Error: {error:?}");
    }

    fn on_connect(&mut self, peer: SocketAddr, _sink: &dyn UdpSocketPeer<Self>) -> Result<(), Self::Error> {
        self.start.replace(Instant::now());
        debug!("Connected to {peer}");
        Ok(())
    }

    fn reconcile(&mut self, peer: Option<&dyn UdpSocketPeer<Self>>) -> Result<(), Self::Error> {
        // We can send keep alives as the transmitter, but the protocol does not send the keep alives back in response

        if self.pending_send {
            self.pending_send = false;
            self.last_sent.replace(Instant::now());

            if let Some(peer) = peer {
                self.send_ping(peer);
            }
        }

        Ok(())
    }

    fn on_datagram(&mut self, _data: &mut [u8], _peer: SocketAddr, _sink: &dyn UdpSocketPeer<Self>) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl EventSleeper for KeepAliveClient {
    async fn sleep(&mut self) -> Option<EventToken> {
        self.start?;

        let sleep_until_time = match self.last_sent {
            Some(prev) => prev + Self::PING_INTERVAL,
            None => Instant::now(),
        };

        sleep_until(sleep_until_time.into()).await;
        self.pending_send = true;
        Some(EventToken(1))
    }
}
