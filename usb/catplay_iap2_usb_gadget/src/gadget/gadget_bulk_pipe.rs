use std::io;

use bytes::BytesMut;
use log::debug;
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream, ReadHalf, WriteHalf};
use usb_gadget::function::custom::{EndpointReceiver, EndpointSender};

use catplay_util::{AsyncShutdown, EventReconciler, EventSleeper, EventToken, event_select, filling_slot};

pub struct GadgetBulkPipe {
    rx: EndpointReceiver,
    tx: EndpointSender,
    read_half: Option<ReadHalf<DuplexStream>>,
    write_half: Option<WriteHalf<DuplexStream>>,
    pending_usb: Option<io::Result<BytesMut>>,
    pending_user: Option<io::Result<BytesMut>>,
    rx_buffer: usize,
    tx_buffer: usize,
    rx_read_buf: Option<BytesMut>,
    tx_read_buf: Vec<u8>,
}

impl GadgetBulkPipe {
    pub fn new(rx: EndpointReceiver, tx: EndpointSender, rx_buffer: usize, tx_buffer: usize) -> io::Result<Self> {
        let handle = Self {
            rx,
            tx,
            read_half: None,
            write_half: None,
            pending_usb: None,
            pending_user: None,
            rx_buffer,
            tx_buffer,
            rx_read_buf: Some(BytesMut::with_capacity(rx_buffer)),
            tx_read_buf: vec![0u8; tx_buffer],
        };

        Ok(handle)
    }

    pub fn stop(&mut self) {
        debug!("Stopping pipe");
        self.read_half = None;
        self.write_half = None;
        self.pending_usb = None;
        self.pending_user = None;
    }

    pub async fn start(&mut self) -> DuplexStream {
        debug!("Starting");
        self.stop();

        debug!("Starting reset");
        let _ = self.tx.cancel();
        let _ = self.rx.cancel();

        let (pipe, user) = tokio::io::duplex(self.rx_buffer.max(self.tx_buffer));
        let (read_half, write_half) = tokio::io::split(pipe);

        self.read_half = Some(read_half);
        self.write_half = Some(write_half);

        user
    }
}

impl AsyncShutdown for GadgetBulkPipe {
    async fn shutdown(&mut self) {
        self.stop();

        let _ = self.tx.cancel();
        let _ = self.rx.cancel();
    }
}

impl EventSleeper for GadgetBulkPipe {
    async fn sleep(&mut self) -> Option<EventToken> {
        if self.pending_usb.is_some() || self.pending_user.is_some() {
            return Some(EventToken(0));
        }

        event_select!(
            filling_slot(
                &mut self.pending_usb,
                Self::read_usb(self.write_half.is_some(), &mut self.rx, &mut self.rx_read_buf, self.rx_buffer)
            ),
            filling_slot(
                &mut self.pending_user,
                Self::read_user(self.read_half.as_mut(), &mut self.tx_read_buf)
            )
        )
    }
}

impl GadgetBulkPipe {
    async fn read_usb(
        active: bool,
        rx: &mut EndpointReceiver,
        rx_read_buf: &mut Option<BytesMut>,
        rx_buffer: usize,
    ) -> Option<io::Result<BytesMut>> {
        if !active {
            return None;
        }

        loop {
            let rx_buf = rx_read_buf
                .take()
                .unwrap_or_else(|| BytesMut::with_capacity(rx_buffer));

            match rx.recv_async(rx_buf).await {
                Ok(Some(rx_buf)) => return Some(Ok(rx_buf)),
                Ok(None) => debug!("USB receive buffer armed, waiting for data"),
                Err(err) => {
                    debug!("Observed rx.recv_async() pipe err: {}", err);
                    return Some(Err(err));
                }
            }
        }
    }

    async fn read_user(read_half: Option<&mut ReadHalf<DuplexStream>>, tx_buf: &mut [u8]) -> Option<io::Result<BytesMut>> {
        let read_half = read_half?;

        let ret = read_half
            .read(tx_buf)
            .await
            .inspect_err(|e| debug!("Observed read_half.read() pipe err: {}", e))
            .map(|n| BytesMut::from(&tx_buf[..n]));

        Some(ret)
    }

    async fn reconcile_usb(&mut self, mut rx_buf: BytesMut) -> io::Result<()> {
        if let Some(write_half) = self.write_half.as_mut() {
            write_half
                .write_all(&rx_buf[..])
                .await
                .inspect_err(|e| debug!("Observed write_half.write_all() pipe err: {}", e))?;
        }

        rx_buf.clear();
        self.rx_read_buf = Some(rx_buf);

        Ok(())
    }

    async fn reconcile_user(&mut self, tx_buf: BytesMut) -> io::Result<()> {
        if tx_buf.is_empty() {
            debug!("Observed write pipe EOF");
            self.read_half = None;
            return Ok(());
        }

        self.tx
            .send_async(tx_buf.into())
            .await
            .inspect_err(|e| debug!("Observed tx.send_async pipe err: {}", e))
    }
}

impl EventReconciler for GadgetBulkPipe {
    type Error = io::Error;

    async fn reconcile(&mut self) -> Result<(), Self::Error> {
        if let Some(ret) = self.pending_usb.take() {
            match ret {
                Ok(rx_buf) => self.reconcile_usb(rx_buf).await?,
                Err(err) => {
                    self.stop();
                    return Err(err);
                }
            };
        }

        if let Some(ret) = self.pending_user.take() {
            match ret {
                Ok(tx_buf) => self.reconcile_user(tx_buf).await?,
                Err(err) => {
                    self.stop();
                    return Err(err);
                }
            };
        }

        Ok(())
    }
}

impl Drop for GadgetBulkPipe {
    fn drop(&mut self) {
        self.stop();
    }
}
