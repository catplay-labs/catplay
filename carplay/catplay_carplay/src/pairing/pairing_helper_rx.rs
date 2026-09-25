use std::sync::Arc;

use bytes::BytesMut;
use catplay_fairplay::{FairPlayError, FairPlaySession};
use catplay_hap::{HomekitStorage, HomekitStorageRef, auth_setup::MfiSapError};
use catplay_hap::{
    auth_setup::MfiSapSession,
    pair_setup::{CARPLAY_MAGIC_PIN, PairSetup},
    pair_verify::PairVerify,
};
use catplay_mfi::MfiDeficeRef;

use catplay_util::spawn_blocking;
use log::{debug, error, trace};

use crate::rtsp_frame::{HttpHeader, HttpStatus, RtspError, RtspMethod, RtspRequest, RtspResponse, RtspResult};

pub enum PairingStateRx {
    PairSetup,
    PairVerify,

    PendingEncrypt { key: [u8; 32] },
    Encrypted { key: [u8; 32] },
}

pub struct PairingHelperRx {
    pub pair_setup_session: Option<PairSetup>,
    pub pair_verify_session: Option<PairVerify>,
    pub auth_setup_session: MfiSapSession,
    pub state: PairingStateRx,

    pub homekit: HomekitStorageRef,
    pub mfi: Option<MfiDeficeRef>,
    pub fairplay: FairPlaySession,

    pub mirror_audio_key: Option<[u8; 16]>,
    pub mirror_audio_iv: Option<[u8; 16]>,
}

impl PairingHelperRx {
    pub fn new(homekit: Arc<dyn HomekitStorage>, mfi: Option<MfiDeficeRef>) -> Self {
        Self {
            pair_setup_session: None,
            pair_verify_session: None,
            auth_setup_session: MfiSapSession::server(),
            state: PairingStateRx::PairSetup,
            homekit,
            mfi,

            mirror_audio_iv: None,
            mirror_audio_key: None,
            fairplay: FairPlaySession::default(),
        }
    }

    pub fn transition_pending_to_encrypted(&mut self) -> Option<[u8; 32]> {
        if let PairingStateRx::PendingEncrypt { key } = self.state {
            self.state = PairingStateRx::Encrypted { key };
            Some(key)
        } else {
            None
        }
    }

    /// Shared secret - result of /pair-verify
    pub fn shared_secret(&self) -> Option<[u8; 32]> {
        if let PairingStateRx::Encrypted { key } = self.state {
            Some(key)
        } else {
            None
        }
    }

    /// AES session keys - result of /auth-setup or /fp-setup combined with initial setup
    pub fn aes_keys(&self) -> Option<([u8; 16], [u8; 16])> {
        if let Some(key) = self.mirror_audio_key
            && let Some(iv) = self.mirror_audio_iv
        {
            return Some((key, iv));
        }

        None
    }

    pub fn decrypt_fairplay_key(&mut self, key: &[u8], iv: &[u8; 16]) -> Result<(), FairPlayError> {
        match self.fairplay.decrypt_audio_key(key) {
            Ok(v) => {
                self.mirror_audio_key.replace(v);
                self.mirror_audio_iv.replace(*iv);
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub fn decrypt_mfi_sap_key(&mut self, key: &[u8], iv: &[u8; 16]) -> Result<(), MfiSapError> {
        match self.auth_setup_session.xcrypt_audio_key(key) {
            Ok(v) => {
                self.mirror_audio_key.replace(v);
                self.mirror_audio_iv.replace(*iv);
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub fn allow_unpaired(&mut self, req: &RtspRequest) -> bool {
        let url = &req.url;
        let allow_unpaired = req.method == RtspMethod::Post && (url == "/pair-setup" || url == "/pair-verify" || url == "/auth-setup");

        allow_unpaired || matches!(self.state, PairingStateRx::Encrypted { .. })
    }

    pub async fn respond_pair_setup(&mut self, req: &RtspRequest, resp: &mut RtspResponse) -> RtspResult<()> {
        if !matches!(self.state, PairingStateRx::PairSetup) {
            return Err(HttpStatus::NotAcceptable)?;
        }

        self.pair_verify_session.take(); // Reset pair-verify if there is a state leftover

        let session = self
            .pair_setup_session
            .take()
            .unwrap_or_else(|| PairSetup::server(CARPLAY_MAGIC_PIN));
        resp.set_header(HttpHeader::ContentType, "application/pairing+tlv8");

        let payload = req.payload.clone();
        let homekit = self.homekit.clone();
        let ret = spawn_blocking(move || session.handle(&*homekit, &payload)).await;
        let Ok(ret) = ret else {
            return Err(HttpStatus::InternalServerError)?;
        };

        match ret {
            Err((err, payload)) => {
                debug!("pair_setup err: {err}");
                resp.payload = BytesMut::from(&payload[..]);
                // Resets pair-setup session by removing state
                resp.status = HttpStatus::Ok
            }
            Ok((session, payload)) => {
                trace!("pair_setup ok: {session:?}");
                resp.payload = BytesMut::from(&payload[..]);

                match session {
                    PairSetup::Finished { device_pairing_id } => {
                        self.state = PairingStateRx::PairVerify;
                        debug!("Pairing complete! Remote id: {device_pairing_id}");
                    }
                    _ => {
                        // Continue session
                        self.pair_setup_session.replace(session);
                    }
                }

                resp.status = HttpStatus::Ok
            }
        }

        Ok(())
    }

    pub async fn respond_pair_verify(&mut self, req: &RtspRequest, resp: &mut RtspResponse) -> RtspResult<()> {
        if !matches!(self.state, PairingStateRx::PairSetup | PairingStateRx::PairVerify) {
            return Err(HttpStatus::NotAcceptable)?;
        }

        let session = self
            .pair_verify_session
            .take()
            .map_or_else(PairVerify::server, Ok);

        let Ok(session) = session else {
            return Err(HttpStatus::InternalServerError)?;
        };

        resp.set_header(HttpHeader::ContentType, "application/pairing+tlv8");

        let payload = req.payload.clone();
        let homekit = self.homekit.clone();
        let ret = spawn_blocking(move || session.handle(&*homekit, &payload)).await;
        let Ok(ret) = ret else {
            return Err(HttpStatus::InternalServerError)?;
        };

        match ret {
            Err((err, payload)) => {
                // Resets pair-verify session by removing state
                debug!("pair_verify err: {err}");
                resp.payload = BytesMut::from(&payload[..]);
                resp.status = HttpStatus::Ok;
            }
            Ok((session, payload)) => {
                trace!("pair_verify ok: {session:?}");
                resp.payload = BytesMut::from(&payload[..]);

                match session {
                    PairVerify::Finish { shared_secret, .. } => {
                        debug!("pending connection encrypt with key {shared_secret:?}");
                        self.state = PairingStateRx::PendingEncrypt { key: shared_secret };
                    }
                    _ => {
                        // Continue session
                        self.pair_verify_session.replace(session);
                    }
                }

                resp.status = HttpStatus::Ok
            }
        }

        Ok(())
    }

    pub async fn respond_auth_setup(&mut self, req: &RtspRequest, resp: &mut RtspResponse) -> RtspResult<()> {
        let mfi = self.mfi.clone();
        let Some(mfi) = mfi else {
            return Err(HttpStatus::NotImplemented)?;
        };

        let session = MfiSapSession::server();

        resp.set_header(HttpHeader::ContentType, "application/octet-stream");

        let payload = req.payload.clone();
        let ret = spawn_blocking(move || session.handle(mfi, &payload)).await;
        let Ok(ret) = ret else {
            return Err(HttpStatus::InternalServerError)?;
        };

        match ret {
            Err(err) => {
                error!("AuthSetup error, is MFi chip connected?: {err}");
                resp.payload = err.as_bytes().into();
                resp.status = HttpStatus::InternalServerError;
            }
            Ok((session, data)) => {
                trace!("auth_setup ok: {data:?}");
                resp.payload = BytesMut::from(&data[..]);
                resp.status = HttpStatus::Ok;
                self.auth_setup_session = session;
            }
        }

        Ok(())
    }

    pub fn respond_fp_setup(&mut self, req: &RtspRequest, resp: &mut RtspResponse) -> RtspResult<()> {
        let reply = self
            .fairplay
            .respond(&req.payload)
            .map_err(|err| RtspError::ProtocolViolationString(format!("invalid FairPlay handshake: {err}")))?;
        resp.payload = reply.as_slice().into();
        resp.set_header(HttpHeader::ContentType, "application/octet-stream");
        Ok(())
    }
}
