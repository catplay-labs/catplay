use std::{
    io::{self, ErrorKind},
    marker::PhantomData,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    os::fd::AsRawFd,
    sync::{Arc, Mutex},
};

use crate::{AlignedOffsetBuf, UdpSender, UdpSession, UdpSocketPeer};
use log::{debug, trace};
use std::net::UdpSocket as StdUdpSocket;
use tokio::{
    io::unix::AsyncFd,
    select,
    sync::{Mutex as TokioMutex, Notify, mpsc, watch},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

const DATAGRAM_BATCH_SIZE: usize = 64;

pub(crate) struct Inner<F: UdpSession> {
    pub(crate) cancel_token: CancellationToken,

    pub(crate) socket: Arc<StdUdpSocket>,
    pub(crate) connect_event_pending: Notify,
    pub(crate) icmp_error: mpsc::Sender<io::Error>,
    pub(crate) set_peer: Mutex<Option<SocketAddr>>,
    pub(crate) session: TokioMutex<Option<TokioMutex<F>>>,

    pub(crate) eof_reason: watch::Sender<Result<(), Option<F::Error>>>,

    pub(crate) task: Mutex<Option<JoinHandle<()>>>,
}

pub struct UdpHelperTask<const DATAGRAM_SIZE: usize, const PAD: usize, F: UdpSession> {
    pub(crate) inner: Arc<Inner<F>>,
    icmp_error: mpsc::Receiver<io::Error>,
    fd: AsyncFd<i32>,
    peer_sink: Option<UdpHelperSink<F>>,
    batch: Box<[AlignedOffsetBuf<DATAGRAM_SIZE, PAD>]>,
}

enum UdpSleepOutcome<E> {
    Continue,
    Cancelled,
    Error(E),
}

enum UdpReconcileOutcome {
    Continue,
    Cancelled,
}

#[derive(Clone)]
pub struct UdpHelperSink<F: UdpSession> {
    icmp_error: mpsc::Sender<io::Error>,
    socket: Arc<StdUdpSocket>,
    peer: Option<SocketAddr>,
    _phantom: PhantomData<F>,
}

impl<F: UdpSession> UdpHelperSink<F> {
    fn new(inner: Arc<Inner<F>>, peer: Option<SocketAddr>) -> Self {
        Self {
            icmp_error: inner.icmp_error.clone(),
            socket: inner.socket.clone(),
            peer,
            _phantom: PhantomData,
        }
    }

    fn icmp_error_spy(&self, err: &io::Error) {
        debug!("Observed OOB send error on UDP socket: {err:?}");

        match err.kind() {
            ErrorKind::ConnectionRefused
            | ErrorKind::HostUnreachable
            | ErrorKind::NetworkUnreachable
            | ErrorKind::AddrNotAvailable
            | ErrorKind::NetworkDown => {
                if self
                    .icmp_error
                    .try_send(io::Error::new(err.kind(), "ICMP error during sendmsg()"))
                    .is_err()
                {
                    debug!("Dropped duplicate ICMP error");
                }
            }
            _ => {}
        }
    }
}

impl<F: UdpSession> UdpSocketPeer<F> for UdpHelperSink<F> {
    fn send(&self, data: &[u8]) -> Result<(), F::Error> {
        let _bytes_written = match self.peer {
            None => self.socket.send(data),
            Some(peer) => self.socket.send_to(data, peer),
        }
        .inspect_err(|err| self.icmp_error_spy(err))?;

        Ok(())
    }

    fn send_multiple(&self, data: &[&[u8]]) -> Result<usize, F::Error> {
        let n = self
            .socket
            .sendmmsg(data, self.peer)
            .inspect_err(|err| self.icmp_error_spy(err))?;
        Ok(n)
    }
}

impl<const DATAGRAM_SIZE: usize, const PAD: usize, F: UdpSession> UdpHelperTask<DATAGRAM_SIZE, PAD, F> {
    fn new(inner: Arc<Inner<F>>, icmp_error: mpsc::Receiver<io::Error>) -> io::Result<Self> {
        let fd = AsyncFd::new(inner.socket.as_raw_fd())?;
        Ok(Self {
            inner,
            icmp_error,
            fd,
            peer_sink: None,
            batch: AlignedOffsetBuf::<DATAGRAM_SIZE, PAD>::new_batch_alloc(DATAGRAM_BATCH_SIZE),
        })
    }

    fn bind_inner(addr: SocketAddr, session: F) -> Result<(Arc<Inner<F>>, mpsc::Receiver<io::Error>, SocketAddr), F::Error> {
        let socket = StdUdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        let local = socket.local_addr()?;

        let icmp = mpsc::channel(1);

        let inner = Arc::new(Inner {
            socket: Arc::new(socket),
            connect_event_pending: Notify::new(),
            icmp_error: icmp.0,
            set_peer: Mutex::new(None),
            session: TokioMutex::new(Some(TokioMutex::new(session))),
            cancel_token: CancellationToken::new(),
            eof_reason: watch::channel(Ok(())).0,
            task: Mutex::new(None),
        });

        Ok((inner, icmp.1, local))
    }

    pub fn bind(addr: SocketAddr, session: F) -> Result<(Self, SocketAddr), F::Error> {
        let (inner, icmp_error, local) = Self::bind_inner(addr, session)?;
        Ok((Self::new(inner, icmp_error)?, local))
    }

    pub fn connect(addr: SocketAddr, session: F) -> Result<(Self, SocketAddr, UdpHelperSink<F>), F::Error> {
        let bind_addr = if addr.is_ipv6() {
            SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0).into()
        } else {
            SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0).into()
        };

        let (task, local_addr) = Self::bind(bind_addr, session)?;
        let sink = task.connect_finish(addr)?;
        Ok((task, local_addr, sink))
    }

    #[allow(clippy::type_complexity)]
    pub fn local_pair(session1: F, session2: F) -> Result<((Self, UdpHelperSink<F>), (Self, UdpHelperSink<F>)), F::Error> {
        let bind_all = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0).into();
        let (task1, local) = Self::bind(bind_all, session1)?;
        let (task2, local, task2_sink) = Self::connect(local, session2)?;
        let task1_sink = task1.connect_finish(local)?;

        Ok(((task1, task1_sink), (task2, task2_sink)))
    }

    pub fn connect_finish(&self, remote: SocketAddr) -> Result<UdpHelperSink<F>, F::Error> {
        self.inner.socket.connect(remote)?;
        Self::finish_peer_inner(&self.inner, remote)
    }

    pub fn pseudo_connect(&self, remote: SocketAddr) -> Result<UdpHelperSink<F>, F::Error> {
        Self::finish_peer_inner(&self.inner, remote)
    }

    pub(crate) fn finish_peer_inner(inner: &Arc<Inner<F>>, remote: SocketAddr) -> Result<UdpHelperSink<F>, F::Error> {
        inner.set_peer.lock().unwrap().replace(remote);
        inner.connect_event_pending.notify_one();
        Ok(UdpHelperSink::new(inner.clone(), Some(remote)))
    }

    pub(crate) async fn run(&mut self) {
        let inner = self.inner.clone();
        let mut session_holder = inner.session.lock().await;
        let mut session = session_holder.as_mut().unwrap().lock().await;

        let err = self.work(&mut session).await.err();
        debug!("Reached EOF status: error={err:?}");
        session.on_eof(err.clone());
        let _ = inner.eof_reason.send_replace(Err(err));
    }

    async fn work(&mut self, session: &mut F) -> Result<(), F::Error> {
        if self.inner.cancel_token.is_cancelled() {
            debug!("Returning, because cancel token is active");
            return Ok(());
        }

        loop {
            match self.reconcile(session)? {
                UdpReconcileOutcome::Continue => {}
                UdpReconcileOutcome::Cancelled => {
                    debug!("Returning, because cancel token is active");
                    return Ok(());
                }
            }

            match self.sleep(session).await {
                UdpSleepOutcome::Continue => {}
                UdpSleepOutcome::Cancelled => {
                    debug!("Returning on activated cancel token");
                    return Ok(());
                }
                UdpSleepOutcome::Error(err) => return Err(err),
            }
        }
    }

    fn reconcile(&mut self, session: &mut F) -> Result<UdpReconcileOutcome, F::Error> {
        if self.inner.cancel_token.is_cancelled() {
            return Ok(UdpReconcileOutcome::Cancelled);
        }

        session.reconcile(self.peer_sink.as_ref().map(|r| r as &dyn UdpSocketPeer<F>))?;
        Ok(UdpReconcileOutcome::Continue)
    }

    async fn sleep(&mut self, session: &mut F) -> UdpSleepOutcome<F::Error> {
        select! {
            Some(token) = session.sleep() => {
                trace!("Session wakeup {token:?}");
                UdpSleepOutcome::Continue
            }
            _ = self.inner.cancel_token.cancelled() => UdpSleepOutcome::Cancelled,
            _ = self.inner.connect_event_pending.notified() => {
                let new_peer = self.inner.set_peer.lock().unwrap().expect("missing peer");
                debug!("Connected to peer {new_peer:?}");

                let sink = UdpHelperSink::new(self.inner.clone(), Some(new_peer));
                match session.on_connect(new_peer, &sink) {
                    Ok(()) => {
                        self.peer_sink.replace(sink);
                        UdpSleepOutcome::Continue
                    }
                    Err(err) => UdpSleepOutcome::Error(err),
                }
            }
            Some(err) = self.icmp_error.recv() => {
                debug!("Returning on ICMP error: {err:?}");
                UdpSleepOutcome::Error(err.into())
            }
            ret = self.fd.readable() => {
                let mut guard = match ret {
                    Err(err) => {
                        debug!("Returing on AsyncFd readable() err: {err:?}");
                        return UdpSleepOutcome::Error(err.into());
                    },
                    Ok(v) => v
                };

                let mut error_pending: Option<_> = None;
                let inner = self.inner.clone();
                let fd = &self.fd;
                let mut bufs = unsafe {
                    AlignedOffsetBuf::build_recv_slices_from_slice::<DATAGRAM_BATCH_SIZE>(&mut self.batch)
                };

                while error_pending.is_none() && match fd.recvmmsg(&mut bufs, &mut |data, peer| {
                    let Some(peer) = peer else {
                        debug!("Datagram dropped due to invalid peer");
                        return;
                    };

                    if error_pending.is_some() {
                        return;
                    }

                    let sink = UdpHelperSink::new(inner.clone(), Some(peer));
                    if let Err(err) = session.on_datagram(data, peer, &sink) && error_pending.is_none() {
                        debug!("Error pending: {err:?}");
                        error_pending.replace(err);
                    }
                }) {
                    Ok(more) => more,
                    Err(err) => {
                        debug!("Returning on recvmmsg err: {err:?}");
                        return UdpSleepOutcome::Error(err.into());
                    }
                } {}

                guard.clear_ready();

                match error_pending {
                    Some(err) => UdpSleepOutcome::Error(err),
                    None => UdpSleepOutcome::Continue,
                }
            }
        }
    }
}
