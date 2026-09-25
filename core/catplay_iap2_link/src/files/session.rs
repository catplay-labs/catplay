use alloc::{collections::VecDeque, vec::Vec};
use log::debug;

use crate::{
    FileTransferPayload, LinkSession, Packet, PayloadDecodable,
    files::{FileTransferEvent, FileTransferOutgoingSource, FileTransferReceiver, FileTransferReserved, FileTransferTransmitter},
};

pub struct FileSession {
    tx: FileTransferTransmitter,
    rx: FileTransferReceiver,
    server: bool,

    queue_tx: VecDeque<FileTransferPayload>,
    local_events: VecDeque<(u8, FileTransferEvent)>,
}

impl FileSession {
    pub fn new(server: bool) -> Self {
        Self {
            tx: FileTransferTransmitter::new(server),
            rx: FileTransferReceiver::new(server),
            queue_tx: VecDeque::new(),
            local_events: VecDeque::new(),
            server,
        }
    }

    pub fn reserve_file_tx(&mut self) -> Option<FileTransferReserved> {
        self.tx.reserve()
    }

    pub fn cancel_file_tx(&mut self, f: FileTransferReserved) {
        self.tx.cancel_reserved(f);
    }

    pub fn send_file(&mut self, id: FileTransferReserved, file_type: u16, setup_data: &[u8], source: FileTransferOutgoingSource) {
        let ret = self
            .tx
            .setup(id, file_type, setup_data, source)
            .expect("duplicate send_file");

        self.queue_tx.push_back(ret.1);
    }

    pub fn dequeue_local_event(&mut self) -> Option<(u8, FileTransferEvent)> {
        self.local_events.pop_front()
    }

    #[cfg(test)]
    pub fn enqueue_test_payload(&mut self, payload: FileTransferPayload) {
        self.queue_tx.push_back(payload);
    }
}

impl LinkSession for FileSession {
    fn dequeue_tx(&mut self, max_payload_len: usize) -> Option<Vec<u8>> {
        if let Some(v) = self.queue_tx.pop_front().map(|i| {
            debug!("Local TX/RX file response: {i:?} [server={}]", self.server);
            let mut v = Vec::new();
            i.to_bytes(&mut v);
            v
        }) {
            return Some(v);
        }

        let chunk_size = max_payload_len.saturating_sub(FileTransferPayload::HEADER_OVERHEAD);
        if chunk_size == 0 {
            return None;
        }

        if let Some(tx) = self.tx.poll(chunk_size) {
            debug!("Local TX file response #2: {tx:?} [server={}]", self.server);
            let mut buf = Vec::new();
            tx.to_bytes(&mut buf);
            return Some(buf);
        }

        None
    }

    fn enqueue_rx(&mut self, packet: &Packet) {
        let f = FileTransferPayload::from_packet(packet);
        let Some(f) = f else {
            debug!("Ignoring invalid FileTransferPayload");
            return;
        };

        let file_id = f.file_id;

        debug!("Feeding: {f:?} [server={}]", self.server);

        // Feed RX
        {
            let (resp, local) = self.rx.feed(f.clone());

            if let Some(resp) = resp {
                debug!("Local RX file response(adding to queue): {resp:?} [server={}]", self.server);
                self.queue_tx.push_back(resp);
            }

            if let Some(local) = local {
                debug!("Local RX file event(adding to queue): {local:?} [server={}]", self.server);
                self.local_events.push_back((file_id, local));
            }
        }

        // Feed TX
        {
            let (resp, local) = self.tx.feed(f);

            if let Some(resp) = resp {
                debug!("Local TX file response(adding to queue): {resp:?} [server={}]", self.server);
                self.queue_tx.push_back(resp);
            }

            if let Some(local) = local {
                debug!("Local TX file event(adding to queue): {local:?} [server={}]", self.server);
                self.local_events.push_back((file_id, local));
            }
        }
    }

    fn has_tx_pending(&self) -> bool {
        !self.queue_tx.is_empty() || self.tx.wants_tx()
    }
}
