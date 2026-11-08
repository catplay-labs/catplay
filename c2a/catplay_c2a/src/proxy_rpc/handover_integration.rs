//! Exercise the real manager and transmitter with a local TCP control peer.
//! Pairing, media SETUP and a vehicle's RTSP policy are outside this fixture.
use super::*;

use async_trait::async_trait;
use catplay_carplay::{
    carplay_rx::AirPlayReceiverHandle,
    carplay_tx::{AirPlayRtspClient, AirPlayTransmitterBootstrapStreams, AirPlayTransmitterImpl, AirPlayTransmitterProxy},
    clock::{MediaClockProxy, MediaClockSession},
    events::CommandQueue,
    modes::{AirPlayModeState, AppState, ChangeModes, Resource},
    msg::{ControllerFeature, Display, HidDevice},
    rtsp_frame::{RtspDrain, RtspFuture, RtspMethod, RtspQueue, RtspRequest},
    rtsp_session::{RtspTransmitter, RtspTransmitterCallback, RtspTransmitterEvent, RtspTransmitterHandle},
};
use catplay_csm::decoder::CsmPacketBox;
use catplay_hap::HomekitStorageFile;
use catplay_tokio::TcpHelper;
use catplay_util::{AsyncShutdown, EventReconciler, EventSink, EventSleeper};
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

struct Init(Option<oneshot::Sender<(RtspQueue, RtspTransmitterHandle)>>);
impl EventSleeper for Init {}
impl EventReconciler for Init {
    type Error = RtspError;
}
impl AsyncShutdown for Init {}
impl RtspTransmitterCallback for Init {}
#[async_trait]
impl EventSink<RtspTransmitterEvent, ()> for Init {
    async fn on_event<'a>(&mut self, event: RtspTransmitterEvent)
    where
        RtspTransmitterEvent: 'a,
    {
        if let RtspTransmitterEvent::Init { queue, handle } = event {
            self.0.take().unwrap().send((queue, handle)).ok().unwrap();
        }
    }
}

struct Phone {
    clock: MediaClockProxy,
    closed: AtomicBool,
    commands: RtspQueue,
    _pending: Mutex<RtspDrain>,
}
impl Phone {
    fn new(clock: MediaClockProxy) -> Self {
        let (commands, pending) = RtspQueue::new();
        Self {
            clock,
            closed: AtomicBool::new(false),
            commands,
            _pending: Mutex::new(pending),
        }
    }
}
#[async_trait]
impl AirPlayReceiverHandle for Phone {
    fn media_clock(&self) -> MediaClockBox {
        self.clock.boxed()
    }
    fn send_command(&self, command: Command) -> RtspResult<RtspFuture> {
        Ok(self
            .commands
            .request(RtspRequest::with_plist(RtspMethod::Post, "/command", command)?))
    }
    fn send_command_and_forget(&self, _: Command) {}
    fn send_hid_report(&self, _: Instant, _: &HidDevice, _: &[u8]) {}
    fn send_iap2(&self, _: CsmPacketBox) {}
    fn request_keyframe(&self) {}
    fn request_display_keyframe(&self, _: &str) {}
    fn request_ui(&self, _: &str) {}
    fn close(&self, _: RtspError) {
        self.closed.store(true, Ordering::SeqCst);
    }
    fn iface(&self) -> &str {
        "lo"
    }
    fn is_wireless(&self) -> bool {
        true
    }
    fn is_car(&self) -> bool {
        false
    }
    fn is_mirroring(&self) -> bool {
        false
    }
    async fn change_modes(&self, _: ChangeModes) -> RtspResult<AirPlayModeState> {
        Err(RtspError::Closed)
    }
    async fn change_resource_mode(&self, _: Resource) -> RtspResult<AirPlayModeState> {
        Err(RtspError::Closed)
    }
    async fn change_app_state(&self, _: AppState) -> RtspResult<AirPlayModeState> {
        Err(RtspError::Closed)
    }
}

#[test]
fn manager_handover_keeps_control_and_rejects_old_phone_events() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();
            let disconnected = Arc::new(AtomicBool::new(false));
            let requests = Arc::new(AtomicUsize::new(0));
            let seen = requests.clone();
            let eof = disconnected.clone();
            let peer = std::thread::spawn(move || {
                let (tcp, _) = listener.accept().unwrap();
                tcp.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
                let mut tcp = BufReader::new(tcp);
                loop {
                    let mut line = String::new();
                    if tcp.read_line(&mut line).unwrap() == 0 {
                        eof.store(true, Ordering::SeqCst);
                        break;
                    }
                    let mut cseq = None;
                    let mut length = 0;
                    loop {
                        line.clear();
                        assert!(tcp.read_line(&mut line).unwrap() > 0);
                        if line == "\r\n" {
                            break;
                        }
                        let (key, value) = line.trim().split_once(':').unwrap();
                        if key.eq_ignore_ascii_case("cseq") {
                            cseq = Some(value.trim().to_owned());
                        }
                        if key.eq_ignore_ascii_case("content-length") {
                            length = value.trim().parse::<usize>().unwrap();
                        }
                    }
                    assert!(length < 16384);
                    tcp.read_exact(&mut vec![0; length]).unwrap();
                    seen.fetch_add(1, Ordering::SeqCst);
                    write!(
                        tcp.get_mut(),
                        "RTSP/1.0 200 OK\r\nCSeq: {}\r\nContent-Length: 0\r\n\r\n",
                        cseq.unwrap()
                    )
                    .unwrap();
                }
            });
            let (init_tx, init_rx) = oneshot::channel();
            let mut transport =
                TcpHelper::connect_timeout(addr, Duration::from_secs(2), RtspTransmitter::new(Init(Some(init_tx)))).unwrap();
            let (queue, handle) = tokio::time::timeout(Duration::from_secs(3), init_rx)
                .await
                .unwrap()
                .unwrap();
            let clock = MediaClockProxy::new(Arc::new(Mutex::new(MediaClockSession::new())));
            let info = InfoMessageResponse {
                displays: vec![Display {
                    width_pixels: 640,
                    height_pixels: 480,
                    ..Default::default()
                }],
                ..Default::default()
            };
            let (_radio_commands, radio_drain) = CommandQueue::new();
            let car = AirPlayTransmitterProxy::new(AirPlayTransmitterImpl::new(AirPlayTransmitterBootstrapStreams {
                info: Some(info),
                media_clock: Some(clock.clone()),
                cmd_drain: Some(radio_drain),
                client: Some(AirPlayRtspClient::new(queue, handle, HomekitStorageFile::memory(), None)),
                ..Default::default()
            }));
            let command = Command::ForceKeyFrame(Default::default());
            car.send_command_noresp(&command, Duration::from_secs(2))
                .unwrap()
                .await
                .unwrap()
                .ok()
                .unwrap();
            let gate = PhoneGate::default();
            let (_tx, rx) = mpsc::unbounded();
            let mut manager = CarManager::new(TeardownGuard::noop(car.clone()), rx, None, gate.clone())
                .await
                .unwrap();
            let first = Arc::new(Phone::new(clock.clone()));
            let first_lease = gate.acquire().unwrap();
            manager.dispatch_tx_op(TxAdapterOp::Phone {
                lease: first_lease.clone(),
                op: PhoneOp::IphoneConnected { handle: first.clone() },
            });
            let (commands, drain) = CommandQueue::new();
            let mut response = Box::pin(commands.run(command.clone()));
            assert!(response.as_mut().now_or_never().is_none());
            let pending = drain.pop().unwrap();
            manager
                .iphone_peer
                .as_mut()
                .unwrap()
                .car_to_iphone_proxy
                .push_back(CarToIphoneCommandTask::new(first.clone(), pending, command.clone()));
            assert!(!first.closed.load(Ordering::SeqCst));
            let (ready_tx, ready_rx) = oneshot::channel();
            manager.dispatch_tx_op(TxAdapterOp::BeginHandover { ready: ready_tx });
            assert!(first.closed.load(Ordering::SeqCst));
            response.await.unwrap().unwrap().ok().unwrap();
            assert!(gate.acquire().is_none(), "new phone must wait for release acknowledgement");
            assert!(manager.iphone_peer.is_none());
            assert!(manager.error.is_none());
            assert!(!disconnected.load(Ordering::SeqCst));
            // Drive the actual release future queued by the production manager.
            tokio::time::timeout(Duration::from_secs(2), manager.tx_queue.pop_front().unwrap())
                .await
                .unwrap()
                .unwrap();
            ready_rx.await.unwrap().unwrap();
            let second_lease = gate
                .acquire()
                .expect("release acknowledgement opens admission");
            let second = Arc::new(Phone::new(clock));
            let second_handle: AirPlayReceiverHandleRef = second.clone();
            manager.dispatch_tx_op(TxAdapterOp::Phone {
                lease: second_lease.clone(),
                op: PhoneOp::IphoneConnected {
                    handle: second_handle.clone(),
                },
            });
            first_lease.end();
            manager.dispatch_tx_op(TxAdapterOp::Phone {
                lease: first_lease.clone(),
                op: PhoneOp::IphoneDisconnected,
            });
            let (_screen_tx, channel) = mpsc::unbounded();
            manager.dispatch_tx_op(TxAdapterOp::Phone {
                lease: first_lease.clone(),
                op: PhoneOp::AltScreenChannel { channel, uuid: None },
            });
            let (enabled, retired_features) = oneshot::channel();
            manager.dispatch_tx_op(TxAdapterOp::Phone {
                lease: first_lease,
                op: PhoneOp::NegotiateFeatures { offered: vec![ControllerFeature::AltScreen], enabled },
            });
            assert!(retired_features.await.is_err());
            assert!(manager.tx_queue.is_empty(), "retired phone must not schedule alt-screen SETUP");
            assert!(Arc::ptr_eq(&manager.iphone_peer.as_ref().unwrap().iphone, &second_handle));
            assert!(gate.owns(&second_lease));
            assert!(!second.closed.load(Ordering::SeqCst));
            assert!(manager.error.is_none());
            assert!(
                tokio::time::timeout(Duration::from_millis(50), car.closed())
                    .await
                    .is_err()
            );
            assert!(!disconnected.load(Ordering::SeqCst), "handover closed radio TCP control");
            car.send_command_noresp(&command, Duration::from_secs(2))
                .unwrap()
                .await
                .unwrap()
                .ok()
                .unwrap();
            assert_eq!(
                requests.load(Ordering::SeqCst),
                2,
                "control usable on the same TCP connection before and after handover"
            );
            drop(manager);
            car.shutdown().await;
            transport.shutdown().await;
            peer.join().unwrap();
            assert!(disconnected.load(Ordering::SeqCst));
        });
}
