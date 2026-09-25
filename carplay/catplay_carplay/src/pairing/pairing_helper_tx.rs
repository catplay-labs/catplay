use std::time::{Duration, Instant};

use catplay_hap::{
    HomekitStorageError, HomekitStorageRef,
    auth_setup::{MfiSapSession, MfiSapSuccess},
    pair_setup::{CARPLAY_MAGIC_PIN, PairSetup},
    pair_verify::PairVerify,
    tlv::{self, ErrorContainer},
};
use catplay_util::spawn_blocking;
use log::{debug, warn};
use uuid::Uuid;

use crate::rtsp_frame::{HttpHeader, RtspError, RtspQueue, RtspRequest, RtspResponse};

pub struct PairingHelperTx {
    rtsp: RtspQueue,
    homekit: HomekitStorageRef,

    advertised_id: Option<Uuid>,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum PairingErrorTx {
    #[error("PairVerify failed to complete within timeout: {0:?}")]
    PairVerifyTimeout(Duration),
    #[error("PairVerify failed with received TLV error: {0}")]
    PairVerifyRemote(tlv::Error),
    #[error("PairVerify failed with local TLV error: {0}")]
    PairVerifyLocal(tlv::Error),
    #[error("PairVerify did not follow proper sequence while error was not signalized")]
    PairVerifyUnexpectedState,
    #[error("PairVerify failed with received HTTP status: {0}")]
    PairVerifyRemoteHTTPStatus(RtspError),
    #[error("PairVerify received mismatched UUID ({uuid} vs expected {expected})")]
    PairVerifyMismatchUuid { uuid: Uuid, expected: Uuid },

    #[error("PairSetup failed to complete within timeout: {0:?}")]
    PairSetupTimeout(Duration),
    #[error("PairSetup failed with received TLV error: {0}")]
    PairSetupRemote(tlv::Error),
    #[error("PairSetup failed with local TLV error: {0}")]
    PairSetupLocal(tlv::Error),
    #[error("PairSetup did not follow proper sequence while error was not signalized")]
    PairSetupUnexpectedState,
    #[error("PairSetup failed with received HTTP status: {0}")]
    PairSetupRemoteHTTPStatus(RtspError),
    #[error("PairSetup received mismatched UUID ({uuid} vs expected {expected})")]
    PairSetupMismatchUuid { uuid: Uuid, expected: Uuid },

    #[error("AuthSetup failed to complete within timeout: {0:?}")]
    AuthSetupTimeout(Duration),
    #[error("AuthSetup failed with local error: {0}")]
    AuthSetupLocal(String),
    #[error("AuthSetup did not follow proper sequence while error was not signalized")]
    AuthSetupUnexpectedState,
    #[error("AuthSetup failed with received HTTP status: {0}")]
    AuthSetupRemoteHTTPStatus(RtspError),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("Unexpected I/O error during HomeKit storage flush: {0}")]
    Homekit(#[from] HomekitStorageError),

    #[error("Unknown Ring crypto error")]
    Ring,
}

impl From<String> for PairingErrorTx {
    fn from(value: String) -> Self {
        PairingErrorTx::SessionError(value)
    }
}

impl PairingHelperTx {
    const PAIR_VERIFY_TIMEOUT: Duration = Duration::from_millis(10000);
    const PAIR_SETUP_TIMEOUT: Duration = Duration::from_millis(30000);
    const AUTH_SETUP_TIMEOUT: Duration = Duration::from_millis(10000);
    const ALLOW_UNPAIR: bool = true;

    pub fn new(homekit: HomekitStorageRef, rtsp: RtspQueue, advertised_id: Option<Uuid>) -> Self {
        Self {
            homekit,
            rtsp,
            advertised_id,
        }
    }

    pub async fn pair(&mut self) -> Result<[u8; 32], PairingErrorTx> {
        let mut slowpath = false;
        let start = Instant::now();

        // If HomekitID from Bonjour entry is in our db, go to /pair-verify (fast-path)
        if let Some(advertised_id) = self.advertised_id
            && self.homekit.find_paired_by_id(&advertised_id).is_some()
        {
            debug!("Fast-path: remote has a known Bonjour ID");
            let result = self.perform_pair_verify(advertised_id).await;
            let Err(err) = result else { return Ok(result.unwrap()) };

            let unpair = matches!(
                err,
                PairingErrorTx::PairVerifyRemote(_) | PairingErrorTx::PairVerifyMismatchUuid { .. } | PairingErrorTx::PairVerifyLocal(_)
            ) && Self::ALLOW_UNPAIR;

            if !unpair {
                return Err(err);
            }

            // Unpair the device if /pair-verify fails with TLV error.
            // This should help if other party has regenerated it's key while keeping the old uuid.
            debug!("Unpairing because of /pair-verify error: {err:?}");
            self.homekit.unpair(&advertised_id)?;
        } else {
            warn!("Slow-path: remote has an unknown Bonjour ID");
            slowpath = true;
        }

        // If id is unknown or /pair-verify has failed, goto /pair-setup
        // If /pair-setup fails at this point, maybe local database has corrupted, it's unlikely to be fixable without clicking "unpair" in the car
        let uuid = self.perform_pair_setup(self.advertised_id).await?;
        if slowpath {
            warn!("... finished PairSetup slow-path after {:?}", Instant::now() - start);
        }
        // Follow successful /path-setup with /pair-verify to encrypt the session
        // /pair-verify is expected to never fail at this stage
        let key = self.perform_pair_verify(uuid).await?;
        Ok(key)
    }

    pub async fn auth_setup(&mut self) -> Result<MfiSapSuccess, PairingErrorTx> {
        let start = Instant::now();
        let deadline = start + Self::AUTH_SETUP_TIMEOUT;

        let (client, payload_m1) = MfiSapSession::client().map_err(PairingErrorTx::AuthSetupLocal)?;
        let (client, _) = self.auth_setup_once(client, &payload_m1, deadline).await?;

        let MfiSapSession::Finish { success } = client else {
            return Err(PairingErrorTx::AuthSetupUnexpectedState);
        };

        if success.response.is_none() {
            return Err(PairingErrorTx::AuthSetupUnexpectedState);
        }

        debug!("AuthSetup final state in {:?}", Instant::now() - start);
        Ok(success)
    }

    async fn pair_setup_once(
        &mut self,
        state: PairSetup,
        homekit: HomekitStorageRef,
        body: &[u8],
        deadline: Instant,
    ) -> Result<(PairSetup, Vec<u8>), PairingErrorTx> {
        // Get remote payload

        let mut req = RtspRequest::post("/pair-setup", body);
        req.set_header(HttpHeader::ContentType, "application/pairing+tlv8");
        req.set_header(HttpHeader::HomeKitPairing, "2");

        let resp = self
            .rtsp
            .request_until(req, deadline)
            .await
            .and_then(RtspResponse::ok)
            .map_err(PairingErrorTx::PairSetupRemoteHTTPStatus)?;

        debug!("pair_setup: {:?}", resp);
        let payload = resp.payload;

        if let Some(error) = ErrorContainer::decode(&payload) {
            return Err(PairingErrorTx::PairSetupRemote(error.error));
        };

        // Feed into local state. Don't block the event loop with CPU-heavy math
        let handle = spawn_blocking(move || {
            state
                .handle(homekit.as_ref(), &payload)
                .map_err(|(e, _)| PairingErrorTx::PairSetupLocal(e))
        });
        handle.await.unwrap()
    }

    async fn pair_verify_once(
        &mut self,
        state: PairVerify,
        homekit: HomekitStorageRef,
        body: &[u8],
        deadline: Instant,
    ) -> Result<(PairVerify, Vec<u8>), PairingErrorTx> {
        // Get remote payload

        let mut req = RtspRequest::post("/pair-verify", body);
        req.set_header(HttpHeader::ContentType, "application/pairing+tlv8");
        req.set_header(HttpHeader::HomeKitPairing, "2");
        req.set_header(HttpHeader::PairDerive, "1");

        let resp = self
            .rtsp
            .request_until(req, deadline)
            .await
            .and_then(RtspResponse::ok)
            .map_err(PairingErrorTx::PairVerifyRemoteHTTPStatus)?;

        debug!("pair_verify: {:?}", resp);
        let payload = resp.payload;

        if let Some(error) = ErrorContainer::decode(&payload) {
            return Err(PairingErrorTx::PairVerifyRemote(error.error));
        };

        // Feed into local state. Don't block the event loop with CPU-heavy math
        let handle = spawn_blocking(move || {
            state
                .handle(homekit.as_ref(), &payload)
                .map_err(|(e, _)| PairingErrorTx::PairVerifyLocal(e))
        });
        handle.await.unwrap()
    }

    async fn auth_setup_once(
        &mut self,
        state: MfiSapSession,
        body: &[u8],
        deadline: Instant,
    ) -> Result<(MfiSapSession, Vec<u8>), PairingErrorTx> {
        let mut req = RtspRequest::post("/auth-setup", body);
        req.set_header(HttpHeader::ContentType, "application/octet-stream");

        let resp = self
            .rtsp
            .request_until(req, deadline)
            .await
            .and_then(RtspResponse::ok)
            .map_err(PairingErrorTx::AuthSetupRemoteHTTPStatus)?;

        debug!("auth_setup: {:?}", resp);
        let payload = resp.payload;

        let handle = spawn_blocking(move || {
            state
                .handle_client_response(&payload)
                .map_err(PairingErrorTx::AuthSetupLocal)
        });
        handle.await.unwrap()
    }

    async fn perform_pair_verify(&mut self, uuid: Uuid) -> Result<[u8; 32], PairingErrorTx> {
        let start = Instant::now();
        let deadline = start + Self::PAIR_VERIFY_TIMEOUT;

        let task = async move {
            let (controller, payload_m1) = PairVerify::client().map_err(|_| PairingErrorTx::Ring)?;
            let (controller, payload_m3) = self
                .pair_verify_once(controller, self.homekit.clone(), &payload_m1, deadline)
                .await?;
            let (controller, _) = self
                .pair_verify_once(controller, self.homekit.clone(), &payload_m3, deadline)
                .await?;
            Result::<PairVerify, PairingErrorTx>::Ok(controller)
        };

        let controller = task.await?;

        let PairVerify::Finish {
            device_pairing_id,
            shared_secret,
            ..
        } = controller
        else {
            return Err(PairingErrorTx::PairVerifyUnexpectedState);
        };

        debug!("PairVerify final state: {:?} in {:?}", controller, Instant::now() - start);

        if uuid != device_pairing_id {
            return Err(PairingErrorTx::PairVerifyMismatchUuid {
                uuid: device_pairing_id,
                expected: uuid,
            });
        }

        debug!("Controller has finished pairing with shared_secret {shared_secret:?}");
        Ok(shared_secret)
    }

    async fn perform_pair_setup(&mut self, uuid: Option<Uuid>) -> Result<Uuid, PairingErrorTx> {
        let start = Instant::now();
        let deadline = start + Self::PAIR_SETUP_TIMEOUT;

        let task = async move {
            let (controller, payload_m1) = PairSetup::client(CARPLAY_MAGIC_PIN);
            let (controller, payload_m3) = self
                .pair_setup_once(controller, self.homekit.clone(), &payload_m1, deadline)
                .await?;
            let (controller, payload_m5) = self
                .pair_setup_once(controller, self.homekit.clone(), &payload_m3, deadline)
                .await?;
            let (controller, _) = self
                .pair_setup_once(controller, self.homekit.clone(), &payload_m5, deadline)
                .await?;
            Result::<PairSetup, PairingErrorTx>::Ok(controller)
        };

        let controller = task.await?;

        let PairSetup::Finished { device_pairing_id } = controller else {
            return Err(PairingErrorTx::PairSetupUnexpectedState);
        };

        debug!("PairSetup final state: {:?} in {:?}", controller, Instant::now() - start);

        if let Some(uuid) = uuid
            && uuid != device_pairing_id
        {
            return Err(PairingErrorTx::PairSetupMismatchUuid {
                uuid: device_pairing_id,
                expected: uuid,
            });
        }

        Ok(device_pairing_id)
    }
}
