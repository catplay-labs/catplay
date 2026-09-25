#![cfg(test)]

extern crate std;

use alloc::{collections::VecDeque, vec, vec::Vec};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::{
    FileTransferOp, FileTransferPayload, LinkEvent, LinkLayer, LinkStatus, Packet,
    clock::MockClock,
    files::{FileTransferEvent, FileTransferOutgoingSource, FileTransferReserved},
};

struct LinkLayerFileTest {
    client_queue: Arc<Mutex<VecDeque<LinkEvent>>>,
    server_queue: Arc<Mutex<VecDeque<LinkEvent>>>,
    client: LinkLayer,
    server: LinkLayer,
    client_files: VecDeque<(u8, FileTransferEvent)>,
    server_files: VecDeque<(u8, FileTransferEvent)>,
}

impl LinkLayerFileTest {
    fn new() -> Self {
        let client_queue = Arc::new(Mutex::new(VecDeque::new()));
        let server_queue = Arc::new(Mutex::new(VecDeque::new()));
        let clock = MockClock::new(Instant::now());

        let client = LinkLayer::with_clock(false, Box::new(clock.clone()), {
            let client_queue = client_queue.clone();
            move |ev| client_queue.lock().unwrap().push_back(ev)
        });
        let server = LinkLayer::with_clock(true, Box::new(clock), {
            let server_queue = server_queue.clone();
            move |ev| server_queue.lock().unwrap().push_back(ev)
        });

        Self {
            client_queue,
            server_queue,
            client,
            server,
            client_files: VecDeque::new(),
            server_files: VecDeque::new(),
        }
    }

    fn sync(&mut self) {
        for _ in 0..20 {
            self.client.reconcile();
            self.server.reconcile();
            self.drain_client_events();
            self.drain_server_events();
        }
    }

    fn drain_client_events(&mut self) {
        let events: Vec<_> = self.client_queue.lock().unwrap().drain(..).collect();

        for ev in events {
            match ev {
                LinkEvent::Write(packet) => self.server.read(packet),
                LinkEvent::FileTransfer(id, event) => self.client_files.push_back((id, event)),
                _ => {}
            }
        }
    }

    fn drain_server_events(&mut self) {
        let events: Vec<_> = self.server_queue.lock().unwrap().drain(..).collect();

        for ev in events {
            match ev {
                LinkEvent::Write(packet) => self.client.read(packet),
                LinkEvent::FileTransfer(id, event) => self.server_files.push_back((id, event)),
                _ => {}
            }
        }
    }
}

#[test]
fn two_link_layers_transfer_file_end_to_end() {
    let mut test = LinkLayerFileTest::new();
    test.sync();

    assert_eq!(test.client.status(), &LinkStatus::Writable);
    assert_eq!(test.server.status(), &LinkStatus::Writable);

    let file_id = test.client.files().reserve_file_tx().expect("file id").0;
    let file = (0..3500).map(|i| (i % 251) as u8).collect::<Vec<_>>();
    test.client.files().send_file(
        FileTransferReserved(file_id),
        0x1234,
        &[0xaa, 0xbb],
        FileTransferOutgoingSource::VecData(file.clone()),
    );

    let mut received = Vec::new();
    let mut setup_seen = false;
    let mut first_chunk_len = None;
    let mut sender_success = false;

    for _ in 0..50 {
        test.sync();

        while let Some((id, event)) = test.server_files.pop_front() {
            assert_eq!(id, file_id);
            match event {
                FileTransferEvent::Setup {
                    size,
                    file_type,
                    setup_data,
                } => {
                    assert_eq!(size, file.len() as u64);
                    assert_eq!(file_type, 0x1234);
                    assert_eq!(setup_data, vec![0xaa, 0xbb]);
                    setup_seen = true;
                }
                FileTransferEvent::Data { data, is_final_chunk } => {
                    first_chunk_len.get_or_insert(data.len());
                    received.extend_from_slice(&data);
                    if is_final_chunk {
                        assert_eq!(received, file);
                    }
                }
                other => panic!("unexpected receiver event: {other:?}"),
            }
        }

        while let Some((id, event)) = test.client_files.pop_front() {
            assert_eq!(id, file_id);
            if event == FileTransferEvent::Success {
                sender_success = true;
            }
        }

        if setup_seen && sender_success && received == file {
            break;
        }
    }

    assert!(setup_seen);
    assert_eq!(
        first_chunk_len,
        Some(2048 - Packet::PACKET_OVERHEAD - FileTransferPayload::HEADER_OVERHEAD)
    );
    assert_eq!(received, file);
    assert!(sender_success);
    assert!(matches!(test.client.status(), &LinkStatus::Writable | &LinkStatus::Unwritable));
    assert!(matches!(test.server.status(), &LinkStatus::Writable | &LinkStatus::Unwritable));
}

#[test]
fn setup_for_unknown_transfer_id_is_cancelled_and_reclaimed_end_to_end() {
    let mut test = LinkLayerFileTest::new();
    test.sync();

    assert_eq!(test.client.status(), &LinkStatus::Writable);
    assert_eq!(test.server.status(), &LinkStatus::Writable);

    let reserved = test
        .server
        .files()
        .reserve_file_tx()
        .expect("server file id");
    let file_id = reserved.0;
    let mut setup = Vec::new();
    setup.extend_from_slice(&4u64.to_be_bytes());
    setup.extend_from_slice(&0x4321u16.to_be_bytes());

    test.client
        .files()
        .enqueue_test_payload(FileTransferPayload {
            file_id,
            op: FileTransferOp::Setup,
            payload: setup,
        });

    let mut server_cancelled = false;
    let mut client_cancelled = false;

    for _ in 0..20 {
        test.sync();

        while let Some((id, event)) = test.server_files.pop_front() {
            assert_eq!(id, file_id);
            assert_eq!(event, FileTransferEvent::Cancel);
            server_cancelled = true;
        }

        while let Some((id, event)) = test.client_files.pop_front() {
            assert_eq!(id, file_id);
            assert_eq!(event, FileTransferEvent::Cancel);
            client_cancelled = true;
        }

        if server_cancelled && client_cancelled {
            break;
        }
    }

    assert!(server_cancelled);
    assert!(client_cancelled);

    let reclaimed = test
        .server
        .files()
        .reserve_file_tx()
        .expect("reclaimed server file id");
    assert_eq!(reclaimed.0, file_id);
}
