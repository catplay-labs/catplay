use catplay_util::{AbortOnDropHandle, EventReconciler, event_select};
use catplay_util::{mpsc, oneshot};
use futures::future::BoxFuture;
use log::{debug, error};
use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::watch;

use crate::{
    audio::{AudioPlayerBox, AudioRecorderBox},
    carplay_tx::{
        AirPlayRtspClient, AirPlayTransmitter, AirPlayTransmitterBootstrap, AirPlayTransmitterBootstrapError, AirPlayTransmitterImpl,
        AirPlayTransmitterSessionError, TeardownGuard,
    },
    clock::{MediaClockBox, MediaClockProxy},
    events::CommandPending,
    modes::AirPlayModeState,
    msg::{AudioFormat, AudioType, Command, InfoMessageResponse, StreamType},
    rtsp_frame::{RtspError, RtspFuture, RtspResult},
    screen::tx::ScreenTransmitProxy,
};

type CallbackType =
    Box<dyn Send + 'static + for<'a> FnOnce(&'a mut AirPlayTransmitterImpl) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>>;

/// Wraps [AirPlayTransmitterImpl] to create an easy to use, reference counted version of the transmitter that can be shared by multiple components.
///
/// All maintainence tasks are handled inside a background task that's alive as long as the last reference is alive.
#[derive(Clone)]
pub struct AirPlayTransmitterProxy {
    task_queue: mpsc::UnboundedSender<CallbackType>,
    cmd_queue: mpsc::SharableReceiverUnbounded<CommandPending>,
    raw_client: AirPlayRtspClient,
    media_clock: MediaClockProxy,
    info: InfoMessageResponse,
    closed: watch::Sender<Option<AirPlayTransmitterSessionError>>,
    task: Arc<Mutex<Option<AbortOnDropHandle<()>>>>,
}

pub type AirPlayTransmitterProxyRef = Arc<AirPlayTransmitterProxy>;

impl AirPlayTransmitterProxy {
    pub fn new(mut transmitter: AirPlayTransmitterImpl) -> AirPlayTransmitterProxyRef {
        let streams = transmitter.raw_streams();
        let raw_client = streams.client.clone().unwrap();
        let media_clock = streams.media_clock.clone().unwrap();
        let info = streams.info.clone().unwrap();

        let (rx, tx) = mpsc::unbounded();
        let (cmd_queue_tx, cmd_queue_rx) = mpsc::unbounded();
        let (closed, _) = watch::channel(None);
        let task = Self::task(transmitter, tx, cmd_queue_tx, closed.clone());
        let task = Arc::new(Mutex::new(Some(catplay_util::spawn(task))));

        Arc::new(Self {
            task_queue: rx,
            cmd_queue: cmd_queue_rx.sharable(),
            raw_client,
            media_clock,
            info,
            closed,
            task,
        })
    }

    pub async fn connect(bootstrap: AirPlayTransmitterBootstrap) -> Result<AirPlayTransmitterProxyRef, AirPlayTransmitterBootstrapError> {
        let b = AirPlayTransmitterImpl::connect(bootstrap).await?;
        Ok(Self::new(b))
    }

    pub async fn task(
        mut imp: AirPlayTransmitterImpl,
        mut task_queue: mpsc::UnboundedReceiver<CallbackType>,
        cmd_queue: mpsc::UnboundedSender<CommandPending>,
        closed: watch::Sender<Option<AirPlayTransmitterSessionError>>,
    ) {
        loop {
            if let Err(err) = imp.reconcile().await {
                error!("Transmitter error: {err:?}");
                closed.send_replace(Some(err));
                return;
            }

            if let Some(cb) = task_queue.take() {
                debug!("Received queued task");
                let fut = (cb)(&mut imp);
                let _ = fut.await;
                debug!("Finished queued task");
            }

            if let Some(cmd) = imp.pop_command() {
                debug!("Command observed");
                if cmd_queue.unbounded_send(cmd).is_err() {
                    debug!("cmd_queue failed send");
                }
            }

            event_select!(task_queue, imp);
            debug!("... wake up");
        }
    }

    fn schedule(&self, cb: impl 'static + Send + for<'a> FnOnce(&'a mut AirPlayTransmitterImpl) -> BoxFuture<'a, ()>) -> Result<(), ()> {
        self.task_queue.unbounded_send(Box::new(cb)).map_err(|_| ())
    }
}

impl AirPlayTransmitter for AirPlayTransmitterProxy {
    fn info_cached(&self) -> &InfoMessageResponse {
        &self.info
    }

    async fn closed(&self) -> AirPlayTransmitterSessionError {
        let mut closed = self.closed.subscribe();

        while closed.changed().await.is_ok() {
            if let Some(err) = closed.borrow().clone() {
                return err.clone();
            }
        }

        AirPlayTransmitterSessionError::Disconnected(RtspError::Closed)
    }

    async fn shutdown(&self) {
        if self.closed.borrow().is_none() {
            self.closed
                .send_replace(Some(AirPlayTransmitterSessionError::Disconnected(RtspError::Closed)));
        }

        let handle = self.task.lock().unwrap().take();
        if let Some(handle) = handle {
            handle.abort();
            handle.await.unwrap();
        }
    }

    async fn pop_command(&self) -> Option<CommandPending> {
        let mut cmd_queue = self.cmd_queue.clone();
        event_select!(cmd_queue);
        cmd_queue.take()
    }

    fn media_clock(&self) -> MediaClockBox {
        self.media_clock.boxed()
    }

    fn send_command_noresp(&self, command: &Command, timeout: Duration) -> RtspResult<RtspFuture> {
        self.raw_client
            .with_timeout(timeout)
            .command_unchecked(command)
    }

    async fn assert_modes(&self, modes: AirPlayModeState) -> RtspResult<()> {
        self.raw_client.assert_modes(modes).await
    }

    async fn setup_screen(&self, latency: Duration) -> RtspResult<TeardownGuard<ScreenTransmitProxy>> {
        let (tx, rx) = oneshot::channel();

        let _ = self.schedule(move |t| {
            Box::pin(async move {
                let _ = tx.send(t.do_setup_video(StreamType::Screen, latency).await);
            })
        });

        match rx.await {
            Err(_) => Err(RtspError::Closed),
            Ok(v) => v,
        }
    }

    async fn setup_audio(
        &self,
        latency: Duration,
        stream_type: StreamType,
        audio_format: AudioFormat,
        audio_type: AudioType,
        // Player (optional)
        player: Option<AudioPlayerBox<i16>>,
        // Recorder (required)
        recorder: AudioRecorderBox<i16>,
    ) -> RtspResult<TeardownGuard<StreamType>> {
        let (tx, rx) = oneshot::channel();

        let _ = self.schedule(move |t| {
            Box::pin(async move {
                let _ = tx.send(
                    t.do_setup_audio(latency, stream_type, audio_format, audio_type, player, recorder)
                        .await,
                );
            })
        });

        match rx.await {
            Err(_) => Err(RtspError::Closed),
            Ok(v) => v,
        }
    }

    async fn record(&self) -> RtspResult<()> {
        let (tx, rx) = oneshot::channel();

        let _ = self.schedule(move |t| {
            Box::pin(async move {
                let _ = tx.send(t.do_record().await);
            })
        });

        match rx.await {
            Err(_) => Err(RtspError::Closed),
            Ok(v) => v,
        }
    }

    async fn drain_teardown_queue(&self) -> RtspResult<()> {
        let (tx, rx) = oneshot::channel();

        let _ = self.schedule(move |t| {
            Box::pin(async move {
                let _ = tx.send(t.drain_teardown_queue().await);
            })
        });

        match rx.await {
            Err(_) => Err(RtspError::Closed),
            Ok(v) => v,
        }
    }
}
