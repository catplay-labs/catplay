use log::debug;

use crate::{LinkEvent, LinkLayer, clock::MockClock};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Instant,
};

pub(super) struct ClientsDuplexTest {
    pub(super) client_queue: Arc<Mutex<VecDeque<LinkEvent>>>,
    pub(super) server_queue: Arc<Mutex<VecDeque<LinkEvent>>>,
    pub(super) client: LinkLayer,
    pub(super) server: LinkLayer,
    pub(super) client_recv: VecDeque<Vec<u8>>,
    pub(super) server_recv: VecDeque<Vec<u8>>,
    pub(super) clock: MockClock,
}

impl ClientsDuplexTest {
    pub(super) fn new() -> ClientsDuplexTest {
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

    pub(super) fn sync(&mut self) {
        for _ in 0..10 {
            self.client.reconcile(usize::MAX);
            self.server.reconcile(usize::MAX);

            self.sync_client_once();
            self.sync_server_once();
        }
    }

    pub(super) fn sync_client_once(&mut self) {
        for i in self.client_queue.lock().unwrap().drain(..) {
            match i {
                LinkEvent::Write(p) => self.server.read(p),
                LinkEvent::ReadCsm(p) => self.client_recv.push_back(p),
                _ => {}
            }
        }
    }

    pub(super) fn sync_server_once(&mut self) {
        for i in self.server_queue.lock().unwrap().drain(..) {
            match i {
                LinkEvent::Write(p) => self.client.read(p),
                LinkEvent::ReadCsm(p) => self.server_recv.push_back(p),
                _ => {}
            }
        }
    }

    pub(super) fn client_lose_ops(&mut self) {
        self.client.reconcile(usize::MAX);
        let lost = self.client_queue.lock().unwrap().drain(..).len();
        debug!("Losing {lost} client ops!");
        assert!(lost > 0);
    }

    pub(super) fn server_lose_ops(&mut self) {
        self.server.reconcile(usize::MAX);
        let lost = self.server_queue.lock().unwrap().drain(..).len();
        debug!("Losing {lost} server ops!");
        assert!(lost > 0);
    }
}
