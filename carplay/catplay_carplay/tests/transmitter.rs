use std::{
    net::{Ipv4Addr, SocketAddrV4},
    time::Duration,
};

use async_trait::async_trait;
use catplay_carplay::{
    audio::{AudioPlayer, AudioPlayerBox, AudioRecorder, AudioRecorderBox, AudioSinkBox, AudioSourceBox, AudioStreamBasicDescription},
    carplay_rx::{
        AirPlayReceiverProfile, AirPlayReceiverSink,
        sink::{AirPlayReceiver, AirPlayServerShared},
    },
    carplay_tx::{AirPlayTransmitterBootstrap, AirPlayTransmitterImpl},
    msg::{AudioFormat, AudioType, Display, InfoMessageResponse, StreamType},
    rtsp_frame::{RtspError, RtspResult},
    rtsp_session::RtspReceiver,
    screen::{
        rx::{ScreenReceiverSink, ScreenReceiverSinkBox},
        tx::ScreenTransmitSink,
    },
    video::{AvccConfig, AvccConfigExtended, EncodedVideoFrame},
};
use catplay_hap::HomekitStorageFile;
use catplay_tokio::TcpHelper;
use catplay_tracing::logger::setup_test_logger;
use catplay_util::{AsyncShutdown, EventReconciler, EventSleeper};
use log::debug;
use macaddr::MacAddr6;
use tokio::{runtime::Handle, time::sleep};

struct MockSink {}
struct MockScreenSink {}
struct MockRecorder {}
struct MockPlayer {}

impl AudioRecorder for MockRecorder {
    type Sample = i16;

    fn init(&mut self, source: AudioSinkBox<Self::Sample>) -> RtspResult<()> {
        Ok(())
    }

    fn start(&mut self) {}

    fn stop(&mut self, drain: bool) {}
}

impl AudioPlayer for MockPlayer {
    type Sample = i16;

    fn init(&mut self, source: AudioSourceBox<Self::Sample>) -> RtspResult<()> {
        Ok(())
    }

    fn start(&mut self) {}

    fn stop(&mut self, drain: bool) {}
}

#[async_trait]
impl ScreenReceiverSink for MockScreenSink {
    async fn init(&mut self) -> RtspResult<()> {
        Ok(())
    }

    async fn process_frame(&mut self, frame: EncodedVideoFrame) -> RtspResult<()> {
        Ok(())
    }

    async fn set_avcc_config(&mut self, _config: AvccConfigExtended) -> RtspResult<()> {
        debug!("AVCC: {_config:?}");
        Ok(())
    }
}

impl AsyncShutdown for MockScreenSink {}

#[async_trait]
impl AirPlayReceiverSink for MockSink {
    async fn on_info(&mut self, info: &mut InfoMessageResponse) {
        info.displays.push(Display::default());
    }

    async fn open_screen(&mut self, latency: Duration) -> RtspResult<ScreenReceiverSinkBox> {
        Ok(Box::new(MockScreenSink {}))
    }

    async fn open_audio(
        &mut self,
        latency: Duration,
        stream_type: StreamType,
        audio_type: AudioType,
        audio_format: AudioFormat,
        pcm_format: AudioStreamBasicDescription,
        duplex: bool,
    ) -> RtspResult<AudioPlayerBox<i16>> {
        Ok(Box::new(MockPlayer {}))
    }

    async fn open_microphone(
        &mut self,
        stream_type: StreamType,
        audio_type: AudioType,
        pcm_format: AudioStreamBasicDescription,
    ) -> RtspResult<AudioRecorderBox<i16>> {
        Ok(Box::new(MockRecorder {}))
    }
}

impl AsyncShutdown for MockSink {}
impl EventSleeper for MockSink {}
impl EventReconciler for MockSink {
    type Error = RtspError;
}

#[tokio::test]
async fn sends_pair_request() {
    // setup_test_logger(true);
    let hk_rx = HomekitStorageFile::memory();
    let hk_tx = HomekitStorageFile::memory();

    let shared = AirPlayServerShared::new();

    let receiver = AirPlayReceiver::new(
        "fake_iface",
        MacAddr6::default(),
        hk_rx,
        None,
        shared,
        Box::new(MockSink {}),
        AirPlayReceiverProfile::CarPlay,
    );
    let session = RtspReceiver::new(receiver);
    let (_rx, bind) = TcpHelper::accept_timeout(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0), Duration::ZERO, session).unwrap();

    let bootstrap = AirPlayTransmitterBootstrap {
        homekit: hk_tx,
        peer_ip: bind,
        controller_features: vec![],
        remote_homekit_id: None,
    };
    let _transmitter = AirPlayTransmitterImpl::connect(bootstrap).await.unwrap();
    debug!("Paired!")
}

#[tokio::test]
async fn setups_video_audio_streams() {
    setup_test_logger(true);
    let hk_rx = HomekitStorageFile::memory();
    let hk_tx = HomekitStorageFile::memory();

    let shared = AirPlayServerShared::new();

    let receiver = AirPlayReceiver::new(
        "fake_iface",
        MacAddr6::default(),
        hk_rx,
        None,
        shared,
        Box::new(MockSink {}),
        AirPlayReceiverProfile::CarPlay,
    );
    let session = RtspReceiver::new(receiver);
    let (_rx, bind) = TcpHelper::accept_timeout(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0), Duration::ZERO, session).unwrap();

    let bootstrap = AirPlayTransmitterBootstrap {
        homekit: hk_tx,
        peer_ip: bind,
        controller_features: vec![],
        remote_homekit_id: None,
    };

    let sps_pps: Vec<u8> = vec![
        0x00, 0x00, 0x00, 0x01, // start code
        0x67, 0x64, 0x00, 0x28, 0xAC, 0x2B, 0x40, 0x78, 0x02, 0x27, 0xE5, 0xC0, 0x44, 0x00, 0x00, 0x03, 0x00, 0x04, 0x00, 0x00, 0x03, 0x00,
        0xC8, 0x3C, 0x48, 0x96, 0x11, 0x80, 0x00, 0x00, 0x00, 0x01, // start code
        0x68, 0xEE, 0x3C, 0xB0,
    ];

    let config = AvccConfigExtended {
        hevc: false,
        avcc: AvccConfig { nal_size_len: 4, sps_pps },
        video_latency: Duration::from_millis(70),
        width: 1920,
        height: 1080,
        respect_timestamps: true,
    };

    let mut _transmitter = AirPlayTransmitterImpl::connect(bootstrap).await.unwrap();
    let mut sink = _transmitter
        .do_setup_video(StreamType::Screen, Duration::from_millis(75))
        .await
        .unwrap();
    sink.push_avcc_config(config).unwrap();

    let _ = _transmitter
        .do_setup_audio(
            Duration::from_millis(32),
            StreamType::MainAudio,
            AudioFormat::PCM_16000_MONO,
            AudioType::Telephony,
            Some(Box::new(MockPlayer {})),
            Box::new(MockRecorder {}),
        )
        .await
        .unwrap();
    sleep(Duration::from_millis(500)).await;

    // TODO assert result
    debug!("Paired!")
}
