use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use bytes::BytesMut;
use catplay_csm::decoder::{AsCsmPacket, CsmPacketBox, CsmPacketRegistry};
use catplay_iap2_link::{
    LinkEvent, LinkLayer, LinkStatus, PacketCoder, PacketFrame,
    files::{FileTransferEvent, FileTransferOutgoingSource, FileTransferReserved},
};
use catplay_lingo::{
    link_layer::LinkLayer as LingoLinkLayer,
    wire::{Frame as LingoFrame, Outgoing},
};
use catplay_util::{AsyncShutdown, EventReconciler, EventSleeper, deadline_maybe, mpsc, notify::Notify, oneshot};
use log::{debug, trace, warn};

use crate::{
    CsmClient, CsmClientHandle, CsmClientHandleRef, CsmFileTransferEvent, CsmRemote, CsmSession, CsmSessionBox, CsmSessionError,
    CsmSessionResult, CsmSessionStatus,
};

/// Async Tokio-based client for iAP2.
#[derive(EventSleeper, EventReconciler)]
#[reconcile_error(CsmSessionError)]
#[reconcile_func(reconcile)]
#[sleep(deadline_maybe(self.deadline))]
#[allow(unused)]
pub struct AsyncClient {
    files: BTreeMap<u8, InflightFileTransfer>,
    status: Arc<Mutex<CsmSessionStatus>>,
    handle: CsmClientHandleRef,

    server: bool,
    remote: CsmRemote,
    link: Arc<Mutex<LinkLayer>>,
    lingo: Arc<Mutex<LingoLinkLayer>>,

    coder: PacketCoder,

    rx_buf: BytesMut,

    send: mpsc::UnboundedSender<(PacketFrame<Vec<u8>>, oneshot::Sender<()>)>,
    session: Box<dyn CsmSession>,

    had_init: bool,
    tx_counter: u64,

    #[reconcile_pop]
    error: Option<CsmSessionError>,
    #[sleep]
    notify: catplay_util::notify::Notify,

    deadline: Option<Duration>,

    #[sleep]
    link_events_rx: mpsc::UnboundedReceiver<LinkEvent>,

    /// Oneshot acts here as a tx permit - until dropped by the remote end,
    /// no further iAP2 packets will be generated for the drain.
    #[sleep]
    pending_tx: Option<oneshot::PeekReceiver<()>>,
}

pub struct AsyncHandle {
    server: bool,
    remote: CsmRemote,
    notify: Notify,
    status: Arc<Mutex<CsmSessionStatus>>,
    link: Arc<Mutex<LinkLayer>>,
    lingo: Arc<Mutex<LingoLinkLayer>>,
}

impl CsmClientHandle for AsyncHandle {
    fn disconnect(&self) {
        // let mut link = self.link.lock().unwrap();
        // debug!("Disconnect was called on the handle");
        // link.close();
        // self.notify.notify_one();
    }

    fn is_server(&self) -> bool {
        self.server
    }

    fn is_closed(&self) -> bool {
        // let status = *self.link.lock().unwrap().status();
        // matches!(status, LinkStatus::Error(_))
        false
    }

    fn remote(&self) -> &CsmRemote {
        &self.remote
    }

    fn status(&self) -> CsmSessionStatus {
        self.status.lock().unwrap().clone()
    }

    fn is_downgrade(&self) -> bool {
        let status = *self.link.lock().unwrap().status();
        matches!(status, LinkStatus::Downgrade)
    }
    fn is_writable(&self) -> bool {
        // let status = *self.link.lock().unwrap().status();
        // matches!(status, LinkStatus::Writable)
        true
    }

    fn send(&self, packet: &dyn AsCsmPacket) -> CsmSessionResult<()> {
        // let mut link = self.link.lock().unwrap();
        let registry = CsmPacketRegistry::static_registry();
        // if let LinkStatus::Error(err) = link.status() {
        //     debug!("Rejecting CSM write because link is in error state: {err:?}");
        //     return Err((*err).into());
        // }

        #[cfg(debug_assertions)]
        debug!("Writing CSM: -> {:?}", packet.as_csm());

        match registry.encode(packet) {
            Ok(d) => {
                #[cfg(debug_assertions)]
                trace!("Writing CSM {:?} bytes {:?}", packet.as_csm(), d);
                self.link.lock().unwrap().csm().enqueue_tx(d);
            }
            Err(err) => {
                debug!("Rejecting write of unserializable CSM packet!: {err:?}");
                self.disconnect();
                return Err(crate::CsmSessionError::AttemptToSerializeUnknownPacket);
            }
        }

        self.notify.notify();
        Ok(())
    }

    fn send_all(&self, packets: &[CsmPacketBox]) -> CsmSessionResult<()> {
        for packet in packets.iter() {
            self.send(packet)?;
        }
        Ok(())
    }

    fn send_lingo(&self, outgoing: Outgoing) -> CsmSessionResult<()> {
        let mut link = self.link.lock().unwrap();
        if !link.is_downgrade() {
            return Err("iAP1 message requires a downgraded link".into());
        }
        let packet = self.lingo.lock().unwrap().send(outgoing)?;
        link.lingo().enqueue_tx(packet.encode_body()?);
        drop(link);
        self.notify.notify();
        Ok(())
    }

    fn finish_lingo_transaction(&self, id: u16) {
        self.lingo
            .lock()
            .unwrap()
            .transactions_mut()
            .finish_transaction(id);
    }

    fn pending_lingo_transaction_count(&self) -> Option<usize> {
        Some(self.lingo.lock().unwrap().transactions().pending_count())
    }

    fn send_file_reserve(&self) -> Option<u8> {
        self.link
            .lock()
            .unwrap()
            .files()
            .reserve_file_tx()
            .map(|t| t.0)
    }

    fn send_file(&self, file_id: u8, file_type: u16, setup_data: &[u8], source: Vec<u8>) {
        self.link.lock().unwrap().files().send_file(
            FileTransferReserved(file_id),
            file_type,
            setup_data,
            FileTransferOutgoingSource::VecData(source),
        );
        self.notify.notify();
    }
}

impl CsmClient for AsyncClient {
    fn status(&self) -> CsmSessionStatus {
        self.status.lock().unwrap().clone()
    }

    fn handle(&self) -> CsmClientHandleRef {
        self.handle.clone()
    }
}

impl AsyncShutdown for AsyncClient {}

#[derive(EventSleeper)]
pub struct AsyncClientDrain {
    #[sleep]
    receiver: mpsc::UnboundedReceiver<(PacketFrame<Vec<u8>>, oneshot::Sender<()>)>,
}

impl AsyncClientDrain {
    fn new(receiver: mpsc::UnboundedReceiver<(PacketFrame<Vec<u8>>, oneshot::Sender<()>)>) -> Self {
        Self { receiver }
    }

    pub fn take(&mut self) -> Option<PacketFrame<Vec<u8>>> {
        self.receiver.take().map(|(packet, _os)| packet)
    }
}

struct InflightFileTransfer {
    size: u64,
    file_type: u16,
    setup_data: Vec<u8>,
    data: Vec<u8>,
}

impl AsyncClient {
    pub fn new(server: bool, remote: CsmRemote, session: CsmSessionBox) -> (Self, AsyncClientDrain) {
        let (link_events_tx, link_events_rx) = mpsc::unbounded();
        let notify = Notify::new();

        let link = Arc::new(Mutex::new(LinkLayer::new(server, move |ev| {
            let _ = link_events_tx.unbounded_send(ev);
        })));
        let lingo = Arc::new(Mutex::new(LingoLinkLayer::with_role(server)));

        let tx = mpsc::unbounded();

        let status = Arc::new(Mutex::new(Default::default()));
        let handle = Arc::new(AsyncHandle {
            server,
            remote,
            notify: notify.clone(),
            status: status.clone(),
            link: link.clone(),
            lingo: lingo.clone(),
        });

        (
            Self {
                status: status.clone(),
                handle,
                coder: PacketCoder::new(false),
                error: None,
                had_init: false,
                link,
                lingo,
                remote,
                server,
                session,
                rx_buf: BytesMut::new(),
                tx_counter: 0,
                send: tx.0,
                notify,
                deadline: None,
                link_events_rx,
                pending_tx: None,
                files: Default::default(),
            },
            AsyncClientDrain::new(tx.1),
        )
    }

    pub fn session_mut(&mut self) -> &dyn CsmSession {
        &mut self.session
    }

    /// Switch to iAP1 before the first iAP2 DETECT is transmitted.
    /// Call this immediately after construction, before reconciling the HID session.
    pub fn downgrade(&mut self) -> CsmSessionResult<()> {
        self.link.lock().unwrap().downgrade()?;
        self.notify.notify();
        Ok(())
    }

    pub fn read_frame_buf(&mut self, buf: &[u8]) {
        self.rx_buf.extend_from_slice(buf);

        while let Ok(Some(packet)) = self.coder.decode(&mut self.rx_buf) {
            #[cfg(debug_assertions)]
            debug!("Forwarding packet to link handler: <- {packet:?}");
            self.link.lock().unwrap().read(packet);
            self.notify.notify();
        }
    }

    pub fn read_frame(&mut self, frame: &PacketFrame<Vec<u8>>) {
        self.read_frame_buf(frame.as_ref());
    }

    fn is_tx_allowed(&mut self) -> bool {
        match self.pending_tx.as_mut() {
            Some(v) => {
                if v.peek().is_some() {
                    self.pending_tx = None;
                    true
                } else {
                    false
                }
            }
            None => true,
        }
    }

    fn on_file_event(&mut self, id: u8, file: FileTransferEvent) -> Option<(u8, CsmFileTransferEvent)> {
        match file {
            FileTransferEvent::Setup {
                size,
                file_type,
                setup_data,
            } => {
                if size > isize::MAX as u64 {
                    // Not valid for buffer allocation at all.
                    return None;
                }

                // In unexpected case we already have an inflight transfer at that id, override it.
                self.files.insert(
                    id,
                    InflightFileTransfer {
                        size,
                        file_type,
                        setup_data: setup_data.clone(),
                        data: Vec::with_capacity(size as _),
                    },
                );

                // TODO: don't blindly accept file transfers as this could cause uncontrolled memory usage

                Some((
                    id,
                    CsmFileTransferEvent::Offer {
                        size,
                        file_type,
                        setup_data,
                    },
                ))
            }
            FileTransferEvent::Data { data, is_final_chunk } => {
                let mut complete = false;
                if let Some(transfer) = self.files.get_mut(&id) {
                    let missing = transfer.size.saturating_sub(transfer.data.len() as _);
                    let missing_chunk_size = missing.min(data.len() as _) as usize;
                    let missing_chunk = &data[0..missing_chunk_size];
                    transfer.data.extend_from_slice(missing_chunk);

                    if missing == 0 || is_final_chunk {
                        // Transfer complete
                        complete = true;
                    }
                }

                match complete {
                    true => {
                        let transfer = self.files.remove(&id).unwrap();
                        Some((
                            id,
                            CsmFileTransferEvent::Completed {
                                size: transfer.data.len() as _,
                                data: transfer.data,
                                file_type: transfer.file_type,
                                setup_data: transfer.setup_data,
                            },
                        ))
                    }
                    false => None,
                }
            }
            FileTransferEvent::Cancel => {
                // Mark transfer as failed
                match self.files.remove(&id).is_some() {
                    true => Some((id, CsmFileTransferEvent::Cancelled {})),
                    false => None,
                }
            }
            _ => None,
        }
    }

    async fn reconcile(&mut self) -> CsmSessionResult<bool> {
        let mut produced_events: bool = false;

        if let Some(p) = { self.link_events_rx.take() } {
            produced_events = true;
            match p {
                LinkEvent::ReadLegacy(body) => {
                    let received = {
                        let mut lingo = self.lingo.lock().unwrap();
                        match lingo.on_frame(&LingoFrame { body: &body }) {
                            Ok(packet) => {
                                let pending = packet
                                    .transaction_id
                                    .and_then(|id| lingo.transactions().pending_command(id));
                                Some((packet, pending))
                            }
                            Err(error) => {
                                warn!("Undecoded iAP1 body {body:02x?}: {error}");
                                None
                            }
                        }
                    };
                    if let Some((packet, pending)) = received {
                        trace!("Received iAP1 message: {:?}", packet.message);
                        self.session
                            .on_legacy_message(packet.message, packet.transaction_id, pending, self.handle.clone())
                            .await?;
                    }
                }
                LinkEvent::ReadCsm(value) => {
                    trace!("Received CSM bytes: {value:?}");

                    let registry = CsmPacketRegistry::static_registry();
                    let Some(pkt) = registry.decode(&value) else {
                        return Err(CsmSessionError::ReceivedCorruptedCsmPayload);
                    };

                    #[cfg(debug_assertions)]
                    debug!("Received CSM: <- {pkt:?}");
                    self.session.respond(pkt, self.handle.clone()).await?;
                }
                LinkEvent::Write(packet) => {
                    #[cfg(debug_assertions)]
                    trace!("Forwarding packet from link handler: -> {packet:?}");

                    let mut b = BytesMut::new();
                    self.coder.encode(packet, &mut b)?;

                    let (os_tx, os_rx) = oneshot::channel();
                    let frame = PacketFrame::new(Some(self.tx_counter), b.into());
                    if self.send.unbounded_send((frame, os_tx)).is_err() {
                        return Err(CsmSessionError::DrainClosed)?;
                    };
                    self.pending_tx.replace(os_rx);

                    self.tx_counter += 1;
                }
                LinkEvent::Status(link_status) => {
                    debug!("New link status: {link_status:?}");
                    *self.status.lock().unwrap() = link_status.into();
                    match link_status {
                        LinkStatus::Writable | LinkStatus::Downgrade if !self.had_init => {
                            self.had_init = true;
                            debug!("Session starting in {link_status:?} mode");
                            self.session.start(self.handle.clone()).await?;
                        }
                        LinkStatus::Error(err) => {
                            debug!("Observed link error: {err:?}");
                            return Err(CsmSessionError::Link(err));
                        }
                        _ => {}
                    }

                    self.notify.notify();
                }
                LinkEvent::FileTransfer(id, file) => {
                    trace!("File event @ {id}: {file:?}");
                    if let Some((id, local_event)) = self.on_file_event(id, file) {
                        match &local_event {
                            CsmFileTransferEvent::Offer {
                                size,
                                file_type,
                                setup_data,
                            } => debug!("File offer @ {id}: size={size} file_type={file_type} setup_data={setup_data:?}"),
                            CsmFileTransferEvent::Completed { size, file_type, .. } => {
                                debug!("File completed @ {id}: size={size} file_type={file_type}")
                            }
                            CsmFileTransferEvent::Cancelled => debug!("File cancelled @ {id}"),
                        }
                        self.session
                            .on_file_event(id, local_event, self.handle.clone())
                            .await?;
                    }
                }
            }
        }

        let allows_tx = self.is_tx_allowed();

        let mut link = self.link.lock().unwrap();
        let may_have_more = link.reconcile(usize::from(allows_tx));

        // If the session is in final state, return that as an error here.
        if let CsmSessionStatus::Error(err) = self.status.lock().unwrap().clone() {
            return Err(err);
        }

        // Don't schedule wake up for retransmissions if we can't transmit, to prevent infinite CPU spin
        self.deadline = link.sleep().filter(|_| allows_tx);
        Ok(produced_events || (allows_tx && may_have_more))
    }
}

impl Drop for AsyncClient {
    fn drop(&mut self) {
        debug!("AsyncClient was dropped!");
        self.handle.disconnect();
    }
}
