use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use async_trait::async_trait;
use catplay_csm::decoder::CsmPacketBox;
use catplay_plist::CachingSerializer;
use catplay_util::mpsc;
use futures::{FutureExt, future::BoxFuture};
use log::{debug, error};

use crate::{
    carplay_rx::AirPlayReceiverHandle,
    clock::{MediaClock, MediaClockBox, MediaClockProxy},
    events::CommandsClient,
    modes::{AirPlayModeState, AppState, ChangeModes, ChangeModesResponse, Resource},
    msg::{Command, CommandForceKeyFrame, CommandHidSendReport, CommandRequestUI, HidDevice},
    rtsp_frame::{HttpStatus, RtspError, RtspFuture, RtspQueue, RtspResult},
};

#[derive(Clone)]
pub struct AirPlayReceiverHandleImpl {
    events_rtsp: Option<RtspQueue>,
    media_clock: MediaClockProxy,
    task_queue: mpsc::UnboundedSender<BoxFuture<'static, ()>>,
    iface: String,
    wireless: bool,
    close_pending_tx: mpsc::Sender<RtspError>,

    serializer: Arc<Mutex<CachingSerializer>>,
}

impl AirPlayReceiverHandleImpl {
    const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);

    pub fn new(
        events_rtsp: Option<RtspQueue>,
        media_clock: MediaClockProxy,
        task_queue: mpsc::UnboundedSender<BoxFuture<'static, ()>>,
        iface: &str,
        wireless: bool,
        close_pending_tx: mpsc::Sender<RtspError>,
    ) -> Self {
        Self {
            events_rtsp,
            media_clock,
            task_queue,
            iface: iface.into(),
            wireless,
            close_pending_tx,
            serializer: Default::default(),
        }
    }
}

#[async_trait]
impl AirPlayReceiverHandle for AirPlayReceiverHandleImpl {
    fn media_clock(&self) -> MediaClockBox {
        self.media_clock.boxed()
    }

    fn request_ui(&self, ui: &str) {
        self.send_command_and_forget(Command::RequestUI(CommandRequestUI {
            url: match ui {
                "" => None,
                v => Some(v.into()),
            },
        }));
    }

    fn send_command(&self, cmd: Command) -> RtspResult<RtspFuture> {
        if let Some(events_rtsp) = self.events_rtsp.as_ref() {
            events_rtsp.send_command_caching(&cmd, Self::COMMAND_TIMEOUT, &mut self.serializer.lock().unwrap())
        } else {
            Err(RtspError::EventsUnsupported)
        }
    }

    fn send_command_and_forget(&self, cmd: Command) {
        let start = Instant::now();
        match self.send_command(cmd.clone()) {
            Ok(fut) => {
                let fut = fut
                    .then(async move |res| match res {
                        Err(err) => {
                            error!("Command {cmd:?} was rejected by remote: {err:?}");
                        }
                        Ok(ret) => {
                            if ret.status != HttpStatus::Ok {
                                error!(
                                    "Command {cmd:?} was rejected by remote with status {}({})",
                                    ret.status,
                                    ret.status.as_code()
                                );
                            } else {
                                debug!("Command {cmd:?} was accepted by remote in {:?}", Instant::now() - start);
                            }
                        }
                    })
                    .boxed();

                if self.task_queue.unbounded_send(fut).is_err() {
                    error!("Failed to schedule command, task queue already closed");
                }
            }
            Err(err) => {
                error!("Failed to schedule command {cmd:?}: {err:?}");
            }
        }
    }

    fn send_hid_report(&self, ts: Instant, device: &HidDevice, report: &[u8]) {
        let cmd = CommandHidSendReport {
            hid_report: report.to_vec().into(),
            // Provide a timestamp but only if the clock is synchronized at the time of call.
            timestamp: self.media_clock.encode_remote(ts).map(|t| t.0),
            uuid: device.uuid.clone(),
        };

        self.send_command_and_forget(Command::HidSendReport(cmd));
    }

    fn request_keyframe(&self) {
        self.send_command_and_forget(Command::ForceKeyFrame(CommandForceKeyFrame { uuid: None }));
    }

    fn request_display_keyframe(&self, uuid: &str) {
        self.send_command_and_forget(Command::ForceKeyFrame(CommandForceKeyFrame {
            uuid: Some(uuid.to_owned()),
        }));
    }

    fn close(&self, err: RtspError) {
        if let Err(err) = self.close_pending_tx.clone().try_send(err) {
            debug!("Already closed while attempting to close: {err}");
        }
    }

    fn send_iap2(&self, _csm: CsmPacketBox) {
        // self.send_command_and_forget(Command::IApSendMessage(csm));
    }

    fn iface(&self) -> &str {
        &self.iface
    }

    fn is_wireless(&self) -> bool {
        self.wireless
    }

    async fn change_resource_mode(&self, resource: Resource) -> RtspResult<AirPlayModeState> {
        self.change_modes(ChangeModes {
            app_states: vec![],
            resources: vec![resource],
            reason_str: "change_resource_mode".into(),
            initial_permanent_entity: vec![],
        })
        .await
    }

    async fn change_app_state(&self, app_state: AppState) -> RtspResult<AirPlayModeState> {
        self.change_modes(ChangeModes {
            app_states: vec![app_state],
            resources: vec![],
            reason_str: "change_app_state".into(),
            initial_permanent_entity: vec![],
        })
        .await
    }

    async fn change_modes(&self, modes: ChangeModes) -> RtspResult<AirPlayModeState> {
        let fut = self.send_command(Command::ChangeModes(modes))?;
        let resp: ChangeModesResponse = fut.await?.ok_payload()?;
        if resp.status != 0 {
            return Err(RtspError::UnexpectedState(format!(
                "changeModes rejected with status {}",
                resp.status
            )));
        }
        let modes = match resp.params {
            None => return Err(RtspError::UnexpectedState("changeModes has no params at status = 0".into())),
            Some(ref v) => v.into(),
        };

        Ok(modes)
    }

    fn is_car(&self) -> bool {
        true
    }

    fn is_mirroring(&self) -> bool {
        false
    }
}
