use alloc::{vec, vec::Vec};
use log::debug;

use crate::{FileTransferOp, FileTransferPayload, files::FileTransferEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileTransferState {
    PendingSetup,

    Sending,
    Pause,

    WaitingForSuccess,

    // Final states
    Success,
    Failure,
    Cancelled,
}

#[derive(Debug)]
pub enum FileTransferOutgoingSource {
    VecData(Vec<u8>),
}

impl FileTransferOutgoingSource {
    fn get_chunk(&self, offset: usize, max_len: usize) -> (&[u8], bool) {
        match self {
            FileTransferOutgoingSource::VecData(v) => {
                if offset >= v.len() {
                    return (&[], true);
                }
                let rem = &v[offset..];
                let take = rem.len().min(max_len);
                let slice = &rem[..take];
                let eof = offset + take >= v.len();
                (slice, eof)
            }
        }
    }

    fn size(&self) -> Option<u64> {
        match self {
            FileTransferOutgoingSource::VecData(v) => Some(v.len() as _),
        }
    }
}

#[derive(Debug)]
pub(crate) struct FileTransferOutgoing {
    // Setup data
    pub file_size: u64,
    pub file_type: u16,
    pub setup_data: Vec<u8>,

    pub file_id: u8,
    pub total: usize,
    pub progress: usize,
    pub source: FileTransferOutgoingSource,
    pub state: FileTransferState,
}

impl FileTransferOutgoing {
    pub fn poll_send(&mut self, chunk_size: usize) -> Option<FileTransferPayload> {
        if self.state != FileTransferState::Sending {
            return None;
        }

        let offset = self.progress;

        let (slice, eof) = self.source.get_chunk(offset, chunk_size);

        let written = slice.len();
        self.progress += written;

        let is_last = eof || (self.total > 0 && self.progress >= self.total);
        let is_first = offset == 0;

        let op = match (is_first, is_last) {
            (true, true) => FileTransferOp::FirstAndOnlyData,
            (true, false) => FileTransferOp::FirstData,
            (false, true) => FileTransferOp::LastData,
            (false, false) => FileTransferOp::Data,
        };

        if is_last {
            self.state = FileTransferState::WaitingForSuccess;
        }

        let ev = FileTransferPayload {
            file_id: self.file_id,
            op,
            payload: slice.to_vec(),
        };

        debug!("Sending file chunk to remote: {ev:?}");
        Some(ev)
    }

    pub fn feed_event(&mut self, ev: FileTransferEvent) {
        self.state = match (ev, self.state) {
            (FileTransferEvent::Start, FileTransferState::Pause) => FileTransferState::Sending,
            (FileTransferEvent::Pause, FileTransferState::Sending) => FileTransferState::Pause,

            (FileTransferEvent::Success, _) => FileTransferState::Success,
            (FileTransferEvent::Failure, _) => FileTransferState::Failure,
            (FileTransferEvent::Cancel, _) => FileTransferState::Cancelled,
            _ => self.state,
        }
    }

    pub fn local_event(&mut self, old_state: FileTransferState, new_state: FileTransferState) -> Option<FileTransferEvent> {
        if old_state == new_state {
            return None;
        }

        Some(match (old_state, new_state) {
            (_, FileTransferState::Success) => FileTransferEvent::Success,
            (_, FileTransferState::Failure) => FileTransferEvent::Failure,
            (_, FileTransferState::Cancelled) => FileTransferEvent::Cancel,
            _ => return None,
        })
    }

    pub fn setup(&mut self) -> Option<FileTransferPayload> {
        if self.state != FileTransferState::PendingSetup {
            return None;
        }

        self.state = FileTransferState::Pause;

        let mut setup_payload = Vec::with_capacity(10 + self.setup_data.len());
        setup_payload.extend_from_slice(&self.file_size.to_be_bytes());
        setup_payload.extend_from_slice(&self.file_type.to_be_bytes());
        setup_payload.extend_from_slice(&self.setup_data);

        let setup_payload = FileTransferPayload {
            file_id: self.file_id,
            op: FileTransferOp::Setup,
            payload: setup_payload,
        };

        Some(setup_payload)
    }

    pub fn cancel(&mut self) -> Option<FileTransferPayload> {
        match self.state {
            FileTransferState::Cancelled | FileTransferState::Success | FileTransferState::Failure => None,
            _ => {
                self.state = FileTransferState::Cancelled;
                Some(FileTransferPayload {
                    file_id: self.file_id,
                    op: FileTransferOp::Cancel,
                    payload: vec![],
                })
            }
        }
    }

    pub fn should_reclaim_id(&self) -> bool {
        matches!(
            self.state,
            FileTransferState::Cancelled | FileTransferState::Success | FileTransferState::Failure
        )
    }
}

pub struct FileTransferTransmitter {
    server: bool,
    taken_ids: [bool; 255],
    pub(crate) outgoing: [Option<FileTransferOutgoing>; 255],
}

pub struct FileTransferReserved(pub u8);
pub struct FileTransferCancellable(pub u8);

impl FileTransferTransmitter {
    pub fn new(server: bool) -> Self {
        Self {
            server,
            taken_ids: [false; 255],
            outgoing: [const { None }; 255],
        }
    }

    pub fn reserve(&mut self) -> Option<FileTransferReserved> {
        let valid_range = if self.server { 128u8..255 } else { 0u8..127 };

        for i in valid_range.clone() {
            if !self.taken_ids[i as usize] {
                self.taken_ids[i as usize] = true;
                return Some(FileTransferReserved(i));
            }
        }

        None
    }

    pub fn cancel_reserved(&mut self, id: FileTransferReserved) {
        self.taken_ids[id.0 as usize] = false;
    }

    pub fn cancel(&mut self, id: FileTransferCancellable) -> Option<FileTransferPayload> {
        let session = self.outgoing.get_mut(id.0 as usize).unwrap();

        if let Some(session) = session {
            let ret = session.cancel();
            self.reconcile(id.0);

            ret
        } else {
            None
        }
    }

    pub fn feed(&mut self, ev: FileTransferPayload) -> (Option<FileTransferPayload>, Option<FileTransferEvent>) {
        let session_ref = self.outgoing.get_mut(ev.file_id as usize).unwrap();

        let valid_range = if self.server { 128u8..255 } else { 0u8..127 };
        if !valid_range.contains(&ev.file_id) {
            return (None, None);
        }

        debug!("Feeding TX event: {ev:?}");
        let file_id = ev.file_id;
        if let Some(session) = session_ref {
            let Ok(ev) = FileTransferEvent::try_from(&ev) else {
                return (None, None);
            };

            let old_state = session.state;
            session.feed_event(ev);
            let new_state = session.state;
            let lev = session.local_event(old_state, new_state);
            self.reconcile(file_id);

            return (None, lev);
        }

        if ev.op == FileTransferOp::Setup {
            self.taken_ids[file_id as usize] = false;
            return (
                Some(FileTransferPayload {
                    file_id,
                    op: FileTransferOp::Cancel,
                    payload: vec![],
                }),
                Some(FileTransferEvent::Cancel),
            );
        }

        (None, None)
    }

    fn reconcile(&mut self, id: u8) {
        let session_ref = self.outgoing.get_mut(id as usize).unwrap();
        if let Some(session) = session_ref
            && session.should_reclaim_id()
        {
            debug!("Unlinking finished file transfer session {}", session.file_id);
            self.taken_ids[session.file_id as usize] = false;
            session_ref.take();
        }
    }

    #[allow(clippy::result_unit_err)]
    pub fn setup(
        &mut self,
        id: FileTransferReserved,
        file_type: u16,
        setup_data: &[u8],
        source: FileTransferOutgoingSource,
    ) -> Result<(FileTransferCancellable, FileTransferPayload), ()> {
        let session = self.outgoing.get_mut(id.0 as usize).unwrap();
        if session.is_some() {
            return Err(());
        }

        let mut new_session = FileTransferOutgoing {
            file_size: source.size().unwrap_or(0),
            file_type,
            setup_data: setup_data.to_vec(),
            file_id: id.0,
            total: source.size().unwrap_or(0) as _,
            progress: 0,
            source,
            state: FileTransferState::PendingSetup,
        };

        let setup = new_session.setup().expect("first setup");
        session.replace(new_session);

        self.reconcile(id.0);
        Ok((FileTransferCancellable(id.0), setup))
    }

    pub fn wants_tx(&self) -> bool {
        // Any transfer in Sending state means _at least_ one more Some(...) result from poll()
        for slot in self.outgoing.iter().flatten() {
            if slot.state == FileTransferState::Sending {
                return true;
            }
        }

        false
    }

    pub fn poll(&mut self, chunk_size: usize) -> Option<FileTransferPayload> {
        for slot in self.outgoing.iter_mut().flatten() {
            if slot.state == FileTransferState::Sending
                && let Some(v) = slot.poll_send(chunk_size)
            {
                //  debug!("Send a data chunk!");
                return Some(v);
            }
        }

        //   debug!("Nothing to poll");
        None
    }
}
