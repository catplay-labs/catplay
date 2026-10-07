use alloc::vec::Vec;
use log::debug;

use crate::{FileTransferOp, FileTransferPayload, files::FileTransferEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileTransferState {
    Sending,
    Pause,
}

#[derive(Debug, Clone, Copy)]
struct FileTransferStatus {
    #[allow(unused)]
    file_id: u8,
    total: usize,
    progress: usize,
    state: FileTransferState,
}

pub struct FileTransferReceiver {
    server: bool,
    incoming: [Option<FileTransferStatus>; 255],
}

impl FileTransferReceiver {
    pub fn new(server: bool) -> Self {
        Self {
            server,
            incoming: [None; 255],
        }
    }

    pub fn feed(&mut self, dg: FileTransferPayload) -> (Option<FileTransferPayload>, Option<FileTransferEvent>) {
        let file_id = dg.file_id;
        let Some(ev) = FileTransferEvent::try_from(&dg).ok() else {
            return (None, None);
        };

        debug!("File transfer event {ev:?} for incoming id {file_id}");

        let valid_range = match self.server {
            false => 128..255,
            true => 0..127,
        };

        if !valid_range.contains(&file_id) {
            return (None, None);
        };

        let mut notify_remote: Option<_> = None;
        let mut notify_local: Option<_> = None;

        let old_status = self.incoming.get(file_id as usize).unwrap();

        let new_status = match ev.clone() {
            FileTransferEvent::Setup { size, .. } => {
                if old_status.is_some() {
                    return (None, None);
                }

                let notification = FileTransferPayload {
                    file_id,
                    op: FileTransferOp::Start,
                    payload: Vec::new(),
                };

                notify_local.replace(ev);
                notify_remote.replace(notification);

                Some(FileTransferStatus {
                    file_id,
                    total: size as _,
                    progress: 0,
                    state: FileTransferState::Sending,
                })
            }
            FileTransferEvent::Data { data, is_final_chunk } => {
                let Some(old_status) = old_status else {
                    return (None, None);
                };

                let data_len = data.len();
                notify_local.replace(FileTransferEvent::Data { data, is_final_chunk });

                if is_final_chunk {
                    let notification = FileTransferPayload {
                        file_id,
                        op: FileTransferOp::Success,
                        payload: Vec::new(),
                    };
                    notify_remote.replace(notification);

                    None
                } else {
                    Some(FileTransferStatus {
                        file_id,
                        progress: old_status.progress + data_len,
                        total: old_status.total,
                        state: FileTransferState::Sending,
                    })
                }
            }
            FileTransferEvent::Start => {
                let Some(old_status) = old_status else {
                    return (None, None);
                };

                if old_status.state != FileTransferState::Pause {
                    return (None, None);
                }

                notify_local.replace(ev);

                Some(FileTransferStatus {
                    file_id,
                    progress: old_status.progress,
                    total: old_status.total,
                    state: FileTransferState::Sending,
                })
            }
            FileTransferEvent::Pause => {
                let Some(old_status) = old_status else {
                    return (None, None);
                };

                if old_status.state != FileTransferState::Sending {
                    return (None, None);
                }

                notify_local.replace(ev);

                Some(FileTransferStatus {
                    file_id,
                    progress: old_status.progress,
                    total: old_status.total,
                    state: FileTransferState::Pause,
                })
            }

            FileTransferEvent::Cancel => {
                notify_local.replace(ev);

                None
            }
            _ => return (None, None),
        };

        if let Some(ev) = notify_local.as_ref() {
            debug!("Notifying local: {ev:?} for {file_id}");
        }

        if let Some(ev) = notify_remote.as_ref() {
            debug!("Notifying remote: {ev:?} for {file_id}");
        }

        debug!("Updated file transfer status: {old_status:?} to {new_status:?}");
        self.incoming[file_id as usize] = new_status;
        (notify_remote, notify_local)
    }
}
