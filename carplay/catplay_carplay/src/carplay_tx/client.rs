use std::{sync::Arc, time::Duration};

use crate::{
    common::AIRPLAY_TX_SDK_USER_AGENT,
    modes::AirPlayModeState,
    msg::{
        Command, CommandModesChanged, FeedbackPayload, InfoMessage, InfoMessageResponse, InitialSetup, InitialSetupResponse, Setup,
        SetupResponse, TeardownPayload,
    },
    pairing::{PairingErrorTx, PairingHelperTx},
    rtsp_frame::{HttpHeader, RtspError, RtspFuture, RtspMethod, RtspQueue, RtspRequest, RtspResult},
    rtsp_session::RtspTransmitterHandle,
};
use catplay_hap::{HomekitStorageRef, auth_setup::MfiSapSuccess};
use log::debug;
use uuid::Uuid;

#[derive(Clone)]
pub struct AirPlayRtspClient {
    rtsp: RtspQueue,
    homekit: HomekitStorageRef,
    remote_uuid: Option<Uuid>,
    handle: Arc<RtspTransmitterHandle>,
    timeout: Duration,
}

impl AirPlayRtspClient {
    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(11);
    const INFO_TIMEOUT: Duration = Duration::from_secs(10);

    pub fn new(rtsp: RtspQueue, handle: RtspTransmitterHandle, homekit: HomekitStorageRef, remote_uuid: Option<Uuid>) -> Self {
        Self {
            rtsp,
            homekit,
            remote_uuid,
            handle: Arc::new(handle),
            timeout: Self::DEFAULT_TIMEOUT,
        }
    }

    pub fn with_timeout(&self, timeout: Duration) -> Self {
        Self { timeout, ..self.clone() }
    }

    pub async fn initial_setup(&self, req: InitialSetup) -> RtspResult<InitialSetupResponse> {
        let mut req = RtspRequest::with_plist(RtspMethod::Setup, "/setup", req)?;
        req.set_header(HttpHeader::UserAgent, AIRPLAY_TX_SDK_USER_AGENT);
        let resp = self
            .rtsp
            .request_timeout(req, self.timeout)
            .await?
            .ok_payload()?;

        Ok(resp)
    }

    pub async fn setup(&self, req: Setup) -> RtspResult<SetupResponse> {
        let mut req = RtspRequest::with_plist(RtspMethod::Setup, "/", req)?;
        req.set_header(HttpHeader::UserAgent, AIRPLAY_TX_SDK_USER_AGENT);
        let resp = self
            .rtsp
            .request_timeout(req, self.timeout)
            .await?
            .ok_payload()?;

        Ok(resp)
    }

    pub async fn info(&self) -> RtspResult<InfoMessageResponse> {
        let mut req = RtspRequest::with_plist(RtspMethod::Get, "/info", InfoMessage::default())?;
        req.set_header(HttpHeader::UserAgent, AIRPLAY_TX_SDK_USER_AGENT);
        let resp = self
            .rtsp
            .request_timeout(req, Self::INFO_TIMEOUT)
            .await?
            .ok_payload()?;

        Ok(resp)
    }

    pub async fn teardown(&self, req: TeardownPayload) -> RtspResult<()> {
        let mut req = RtspRequest::with_plist(RtspMethod::Teardown, "/", req)?;
        req.set_header(HttpHeader::UserAgent, AIRPLAY_TX_SDK_USER_AGENT);
        let _resp = self.rtsp.request_timeout(req, self.timeout).await?.ok()?;

        Ok(())
    }

    pub async fn record(&self) -> RtspResult<()> {
        let mut req = RtspRequest::new(RtspMethod::Record, "/");
        req.set_header(HttpHeader::UserAgent, AIRPLAY_TX_SDK_USER_AGENT);
        let _resp = self.rtsp.request_timeout(req, self.timeout).await?.ok()?;

        Ok(())
    }

    pub fn command_unchecked(&self, cmd: &Command) -> RtspResult<RtspFuture> {
        let ev = cmd.serialize().map_err(RtspError::SerializationFailed)?;
        let mut req = RtspRequest::with_payload(RtspMethod::Post, "/command", &ev[..]);
        req.set_header(HttpHeader::ContentType, "application/x-apple-binary-plist");
        req.set_header(HttpHeader::UserAgent, AIRPLAY_TX_SDK_USER_AGENT);
        let resp = self.rtsp.request_timeout(req, self.timeout);

        Ok(resp)
    }

    pub async fn feedback(&self) -> RtspResult<()> {
        let mut req = RtspRequest::new(RtspMethod::Post, "/feedback");
        req.set_header(HttpHeader::UserAgent, AIRPLAY_TX_SDK_USER_AGENT);
        let _resp = self.rtsp.request_timeout(req, self.timeout).await?.ok()?;

        Ok(())
    }

    pub async fn feedback_payload(&self) -> RtspResult<Option<FeedbackPayload>> {
        let mut req = RtspRequest::new(RtspMethod::Post, "/feedback");
        req.set_header(HttpHeader::UserAgent, AIRPLAY_TX_SDK_USER_AGENT);
        let resp = self.rtsp.request_timeout(req, self.timeout).await?.ok()?;

        if resp.payload.is_empty() {
            return Ok(None);
        }

        Ok(Some(resp.get_plist()?))
    }

    pub fn feedback_throwaway(&self) {
        let mut req = RtspRequest::new(RtspMethod::Post, "/feedback");
        req.set_header(HttpHeader::UserAgent, AIRPLAY_TX_SDK_USER_AGENT);
        drop(self.rtsp.request(req));
    }

    pub async fn pair(&self) -> Result<[u8; 32], PairingErrorTx> {
        let mut helper = PairingHelperTx::new(self.homekit.clone(), self.rtsp.clone(), self.remote_uuid);
        let key = helper.pair().await?;
        debug!("Passing key to encrypt callback");
        self.handle.encrypt(key);
        Ok(key)
    }

    pub async fn auth_setup(&self) -> Result<MfiSapSuccess, PairingErrorTx> {
        let mut helper = PairingHelperTx::new(self.homekit.clone(), self.rtsp.clone(), self.remote_uuid);
        helper.auth_setup().await
    }

    pub fn close_connection(&self) {
        self.handle.close();
    }

    pub async fn assert_modes(&self, modes: AirPlayModeState) -> RtspResult<()> {
        let cmd = Command::ModesChanged(CommandModesChanged(modes.serialize()));
        let fut = self.command_unchecked(&cmd)?;
        let _resp = fut.await?.ok()?;
        Ok(())
    }
}
