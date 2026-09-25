use catplay_tracing::logger::setup_test_logger;
use catplay_util::ModSeq;
use log::debug;

use crate::{LSPPayload, LinkError, LinkEvent, LinkLayer, LinkStatus, Packet, PacketOrDetect, clock::MockClock};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[cfg(test)]
#[ctor::ctor]
pub fn init_logger() {
    setup_test_logger(true);
}

struct ClientsDuplexTest {
    client_queue: Arc<Mutex<VecDeque<LinkEvent>>>,
    server_queue: Arc<Mutex<VecDeque<LinkEvent>>>,
    pub client: LinkLayer,
    pub server: LinkLayer,
    pub client_recv: VecDeque<Vec<u8>>,
    pub server_recv: VecDeque<Vec<u8>>,
    pub clock: MockClock,
}

impl ClientsDuplexTest {
    fn new() -> ClientsDuplexTest {
        let client_queue: Arc<Mutex<VecDeque<LinkEvent>>> = Arc::new(Mutex::new(VecDeque::new()));
        let server_queue: Arc<Mutex<VecDeque<LinkEvent>>> = Arc::new(Mutex::new(VecDeque::new()));
        let client_recv = VecDeque::new();
        let server_recv = VecDeque::new();

        let client_cb = {
            let client_queue = client_queue.clone();
            move |ev| {
                client_queue.lock().unwrap().push_back(ev);
            }
        };
        let server_cb = {
            let server_queue = server_queue.clone();
            move |ev| {
                server_queue.lock().unwrap().push_back(ev);
            }
        };

        let clock = MockClock::new(Instant::now());

        let client = LinkLayer::with_clock(false, Box::new(clock.clone()), client_cb);
        let server = LinkLayer::with_clock(true, Box::new(clock.clone()), server_cb);

        ClientsDuplexTest {
            client_queue,
            server_queue,
            client,
            server,
            client_recv,
            server_recv,
            clock,
        }
    }

    fn sync(&mut self) {
        for _ in 0..10 {
            self.client.reconcile_all();
            self.server.reconcile_all();

            self.sync_client_once();
            self.sync_server_once();
        }
    }

    fn sync_client_once(&mut self) {
        for i in self.client_queue.lock().unwrap().drain(..) {
            match i {
                LinkEvent::Write(p) => self.server.read(p),
                LinkEvent::ReadCsm(p) => self.client_recv.push_back(p),
                _ => {}
            }
        }
    }

    fn sync_server_once(&mut self) {
        for i in self.server_queue.lock().unwrap().drain(..) {
            match i {
                LinkEvent::Write(p) => self.client.read(p),
                LinkEvent::ReadCsm(p) => self.server_recv.push_back(p),
                _ => {}
            }
        }
    }

    fn client_lose_ops(&mut self) {
        self.client.reconcile_all();
        let lost = self.client_queue.lock().unwrap().drain(..).len();
        debug!("Losing {lost} client ops!");
        assert!(lost > 0);
    }

    fn server_lose_ops(&mut self) {
        self.server.reconcile_all();
        let lost = self.server_queue.lock().unwrap().drain(..).len();
        debug!("Losing {lost} server ops!");
        assert!(lost > 0);
    }
}

#[test]
fn test_client_server_negotiation() {
    let mut test = ClientsDuplexTest::new();
    test.sync();

    assert_eq!(test.client.status(), &LinkStatus::Writable);
    assert_eq!(test.server.status(), &LinkStatus::Writable);
}

#[test]
fn test_transmit_once_sends_detect_while_detecting() {
    let mut test = ClientsDuplexTest::new();
    test.client_queue.lock().unwrap().clear();
    test.clock.advance(Duration::from_secs(1));

    assert_eq!(test.client.status(), &LinkStatus::Detecting);
    assert!(matches!(test.client.next_transmit(), Some(PacketOrDetect::Detect)));

    let queued = test.client_queue.lock().unwrap();
    assert!(queued.is_empty());
}

#[test]
fn test_transmit_once_sends_syn_while_negotiating() {
    let mut test = ClientsDuplexTest::new();
    test.client_queue.lock().unwrap().clear();
    test.client.change_state(LinkStatus::Negotiating);

    let syn = match test.client.next_transmit() {
        Some(PacketOrDetect::Packet(packet)) => packet,
        _ => panic!("expected queued SYN"),
    };

    assert!(syn.header.control.is_syn());
    assert_eq!(
        test.client_queue
            .lock()
            .unwrap()
            .iter()
            .filter(|ev| matches!(ev, LinkEvent::Write(_)))
            .count(),
        0
    );
}

#[test]
fn test_single_csm() {
    let mut test = ClientsDuplexTest::new();
    test.client.csm().enqueue_tx(vec![1, 2, 3, 4]);
    test.server.csm().enqueue_tx(vec![4, 3, 2, 1]);
    test.sync();

    assert_eq!(test.client.status(), &LinkStatus::Writable);
    assert_eq!(test.server.status(), &LinkStatus::Writable);

    assert_eq!(test.client_recv.pop_front(), Some(vec![4, 3, 2, 1]));
    assert_eq!(test.server_recv.pop_front(), Some(vec![1, 2, 3, 4]));
}

#[test]
fn test_long_session() {
    const N: usize = 1024;

    let mut test = ClientsDuplexTest::new();
    test.sync();

    assert_eq!(test.client.status(), &LinkStatus::Writable);
    assert_eq!(test.server.status(), &LinkStatus::Writable);

    for _ in 0..N {
        test.client.csm().enqueue_tx(vec![1, 2, 3, 4]);
        test.server.csm().enqueue_tx(vec![4, 3, 2, 1]);
    }

    test.server.reconcile_all();
    test.client.reconcile_all();

    assert_eq!(test.client.status(), &LinkStatus::Unwritable);
    assert_eq!(test.server.status(), &LinkStatus::Unwritable);

    for _ in 0..N {
        test.sync();
    }

    assert_eq!(test.client.status(), &LinkStatus::Writable);
    assert_eq!(test.server.status(), &LinkStatus::Writable);

    assert_eq!(test.client_recv.len(), N);
    assert_eq!(test.server_recv.len(), N);
}

#[test]
fn test_server_lost_detect() {
    let mut test = ClientsDuplexTest::new();
    test.client_lose_ops();
    test.sync();

    assert_eq!(test.client.status(), &LinkStatus::Detecting);
    assert_eq!(test.server.status(), &LinkStatus::Detecting);

    test.clock.advance(Duration::from_secs(2));
    test.sync();

    assert_eq!(test.client.status(), &LinkStatus::Writable);
    assert_eq!(test.server.status(), &LinkStatus::Writable);
}

#[test]
fn test_client_lost_detect() {
    let mut test = ClientsDuplexTest::new();
    test.client.reconcile_all();
    test.sync_server_once();
    debug!("Stage 1");
    test.sync_client_once();
    debug!("Stage 2");
    test.server_lose_ops();
    debug!("Stage 3");
    test.sync();
    debug!("Stage 4");

    assert_eq!(test.client.status(), &LinkStatus::Detecting);
    assert_eq!(test.server.status(), &LinkStatus::Detecting);

    test.clock.advance(Duration::from_secs(2));
    test.sync();

    assert_eq!(test.client.status(), &LinkStatus::Writable);
    assert_eq!(test.server.status(), &LinkStatus::Writable);
}

#[test]

fn test_client_retransmit_timeout() {
    let mut test = ClientsDuplexTest::new();
    test.sync();

    assert_eq!(test.client.status(), &LinkStatus::Writable);
    assert_eq!(test.server.status(), &LinkStatus::Writable);
    test.client.csm().enqueue_tx(vec![1, 2, 3, 4]);

    for _ in 0..100 {
        test.clock.advance(Duration::from_secs(2));

        test.sync_client_once();
        test.client.reconcile_all();
        if let LinkStatus::Error(_) = test.client.status() {
            break;
        }

        test.client_lose_ops();
    }

    assert_eq!(test.client.status(), &LinkStatus::Error(LinkError::RetransmissionsExceeded));
    assert_eq!(test.server.status(), &LinkStatus::Writable);
}

#[test]
fn test_server_retransmit_timeout() {
    let mut test = ClientsDuplexTest::new();
    test.sync();

    assert_eq!(test.client.status(), &LinkStatus::Writable);
    assert_eq!(test.server.status(), &LinkStatus::Writable);
    test.server.csm().enqueue_tx(vec![1, 2, 3, 4]);

    for _ in 0..100 {
        test.clock.advance(Duration::from_secs(2));

        test.sync_server_once();
        test.server.reconcile_all();
        if let LinkStatus::Error(_) = test.server.status() {
            break;
        }

        test.server_lose_ops();
    }

    assert_eq!(test.server.status(), &LinkStatus::Error(LinkError::RetransmissionsExceeded));
    assert_eq!(test.client.status(), &LinkStatus::Writable);
}

#[test]
fn test_eak_enters_recovery_and_ack_exits() {
    let mut test = ClientsDuplexTest::new();
    test.sync();

    test.client.csm().enqueue_tx(vec![1, 2, 3, 4]);
    test.client.reconcile_all();

    assert_eq!(test.client.status(), &LinkStatus::Writable);
    assert_eq!(test.client.retransmit_map.len(), 1);

    let missing_seq = *test.client.retransmit_map.keys().next().unwrap();
    let eak = Packet::new_eak(
        test.server.own_seq,
        test.server
            .peer_seq
            .expect("server peer_seq should be known after negotiation"),
        LSPPayload::SESSION_ID_CONTROL,
        vec![missing_seq],
    );
    test.client.read(eak.into());

    assert_eq!(test.client.status(), &LinkStatus::Recovery);
    assert!(test.client.eak.is_active());

    let ack = Packet::new_ack(
        test.server.own_seq,
        test.client.own_seq - ModSeq(1),
        LSPPayload::SESSION_ID_CONTROL,
        None,
    );
    test.client.read(ack.into());

    assert_eq!(test.client.status(), &LinkStatus::Writable);
    assert!(!test.client.eak.is_active());
    assert!(test.client.retransmit_map.is_empty());
}

#[test]
fn test_transmit_once_retransmits_missing_packet_in_recovery() {
    let mut test = ClientsDuplexTest::new();
    test.sync();

    test.client.csm().enqueue_tx(vec![1, 2, 3, 4]);
    test.client.reconcile_all();

    let missing_seq = *test.client.retransmit_map.keys().next().unwrap();
    test.client_queue.lock().unwrap().clear();

    let eak = Packet::new_eak(
        test.server.own_seq,
        test.server
            .peer_seq
            .expect("server peer_seq should be known after negotiation"),
        LSPPayload::SESSION_ID_CONTROL,
        vec![missing_seq],
    );
    test.client.read(eak.into());

    let queued = test.client_queue.lock().unwrap();
    assert_eq!(
        queued
            .iter()
            .filter(|ev| matches!(ev, LinkEvent::Write(_)))
            .count(),
        0
    );
    drop(queued);

    assert_eq!(test.client.status(), &LinkStatus::Recovery);
    let retransmit = match test.client.next_transmit() {
        Some(PacketOrDetect::Packet(packet)) => packet,
        _ => panic!("expected queued retransmission"),
    };

    assert_eq!(retransmit.header.seq.0, missing_seq);
    assert_eq!(
        test.client_queue
            .lock()
            .unwrap()
            .iter()
            .filter(|ev| matches!(ev, LinkEvent::Write(_)))
            .count(),
        0
    );
}

#[test]
fn test_transmit_once_retransmits_timed_out_packet() {
    let mut test = ClientsDuplexTest::new();
    test.sync();

    test.client.csm().enqueue_tx(vec![1, 2, 3, 4]);
    test.client.reconcile_all();

    let timed_out_seq = *test.client.retransmit_map.keys().next().unwrap();
    test.client_queue.lock().unwrap().clear();
    test.clock.advance(Duration::from_secs(3));

    assert_eq!(test.client.status(), &LinkStatus::Writable);
    let retransmit = match test.client.next_transmit() {
        Some(PacketOrDetect::Packet(packet)) => packet,
        _ => panic!("expected queued retransmission"),
    };

    assert_eq!(retransmit.header.seq.0, timed_out_seq);
    assert_eq!(
        test.client_queue
            .lock()
            .unwrap()
            .iter()
            .filter(|ev| matches!(ev, LinkEvent::Write(_)))
            .count(),
        0
    );
}

// TODO more test scenarios:
// - SYN retransmit
// - packet retransmit
