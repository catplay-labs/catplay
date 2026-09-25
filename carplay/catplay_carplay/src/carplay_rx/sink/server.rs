use std::{io, sync::Arc};

use catplay_bonjour::{Bonjour, BonjourEntryHandle, BonjourType};
use catplay_hap::HomekitStorageRef;
use catplay_mfi::MfiDeficeRef;
use catplay_tokio::TcpServer;
use catplay_util::AsyncShutdown;
use log::debug;
use macaddr::MacAddr6;

use crate::{
    carplay_rx::{
        AirPlayReceiverProfile, AirPlayReceiverSink, AirPlayReceiverSinkCallback,
        sink::{AirPlayReceiver, AirPlayServerShared, CarPlayInviter},
    },
    common::AirPlayBonjourEntry,
    rtsp_session::RtspReceiver,
};

pub struct AirPlayServer {
    port: u16,
    mac_addr: MacAddr6,
    iface: String,
    homekit: HomekitStorageRef,
    mfi: Option<MfiDeficeRef>,

    bonjour_entry: Option<BonjourEntryHandle>,
    bonjour: Option<Bonjour>,
    inviter: Option<CarPlayInviter>,

    listener_handle: Option<TcpServer<RtspReceiver<AirPlayReceiver>>>,
    shared: AirPlayServerShared,
    sink: AirPlayReceiverSinkCallback,
    profile: AirPlayReceiverProfile,
}

impl AirPlayServer {
    pub fn new<T: AirPlayReceiverSink>(
        port: u16,
        mac_addr: MacAddr6,
        iface: &str,
        homekit: HomekitStorageRef,
        mfi: Option<MfiDeficeRef>,
        shared: AirPlayServerShared,
        profile: AirPlayReceiverProfile,
        sink: impl Fn() -> T + Send + Sync + 'static,
    ) -> Self {
        Self {
            port,
            mac_addr,
            iface: iface.into(),
            homekit,
            mfi,
            bonjour_entry: None,
            bonjour: None,
            inviter: None,
            listener_handle: None,
            shared,
            profile,
            sink: Arc::new(move || Box::new((sink)())),
        }
    }

    pub async fn bind(&mut self) -> io::Result<()> {
        debug!("Bind called with addr {}:{}", self.iface, self.port);
        let homekit = self.homekit.clone();
        let mfi = self.mfi.clone();
        let shared = self.shared.clone();

        let callback = self.sink.clone();
        let iface = self.iface.clone();
        let profile = self.profile;
        let mac_addr = self.mac_addr;

        let listener = TcpServer::bind_iface(&self.iface, self.port, move |peer| {
            debug!("Accepted connection from peer {peer:?}");
            let sink = (callback)();

            let session = AirPlayReceiver::new(&iface, mac_addr, homekit.clone(), mfi.clone(), shared.clone(), sink, profile);

            RtspReceiver::new(session)
        })?;
        self.listener_handle.replace(listener);

        Ok(())
    }

    pub fn start_advertise(&mut self) -> io::Result<()> {
        let bonjour = Bonjour::with_iface(&self.iface, BonjourType::Ipv6AndIpv4)?;
        let entry = AirPlayReceiver::bonjour(self.profile, self.homekit.clone(), self.mac_addr);
        let handle = bonjour.register::<AirPlayBonjourEntry>(entry, "carplay.local.", "carplay", self.port, &[])?;

        self.bonjour_entry.replace(handle);
        self.bonjour.replace(bonjour);

        debug!("AirPlayServer started advertising");
        Ok(())
    }

    pub fn start_inviting(&mut self) -> bool {
        let Some(bonjour) = self.bonjour.as_ref() else {
            return false;
        };
        let Some(bonjour_entry) = self.bonjour_entry.as_ref() else {
            return false;
        };

        if self.inviter.is_some() {
            return true;
        }

        self.inviter
            .replace(CarPlayInviter::with_bonjour(bonjour, self.mac_addr, bonjour_entry.clone()));
        true
    }

    pub fn stop_inviting(&mut self) {
        self.inviter = None;
    }
}

impl AsyncShutdown for AirPlayServer {
    async fn shutdown(&mut self) {
        // TODO
    }
}

impl Drop for AirPlayServer {
    fn drop(&mut self) {
        debug!("AirPlayServer was dropped!");
    }
}
