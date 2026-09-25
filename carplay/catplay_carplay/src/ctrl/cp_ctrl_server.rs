use std::io;

use catplay_bonjour::{
    Bonjour, BonjourCached, BonjourEntryHandle, BonjourLiveCache, BonjourType,
    raw::{CarplayCtrlAnnounceParams, MdnsAnnouncer},
};
use catplay_tokio::TcpServer;
use catplay_util::{AsyncShutdown, EventSleeper, mpsc, notify::Notify};
use log::{debug, warn};

use crate::{
    common::{AIRPLAY_TX_IOS_BOARD, AIRPLAY_TX_SDK_VERSION, AirPlayBonjourEntry, AirPlayFeature},
    ctrl::{CarPlayCtrlBonjourEntry, CarPlayCtrlInvite, CarPlayCtrlSession, DeviceInfoBonjourEntry},
};

#[derive(EventSleeper)]
#[sleep_fut(self.car_cache.changed())]
pub struct CarPlayCtrlServer {
    _bonjour: Bonjour,
    bonjour_id: String,
    bonjour_handle: Option<BonjourEntryHandle>,
    bonjour_handle_info: Option<BonjourEntryHandle>,

    #[sleep]
    server: TcpServer<CarPlayCtrlSession>,
    #[sleep]
    invite_rx: mpsc::UnboundedReceiver<CarPlayCtrlInvite>,
    invite_status: InviteStatus,
    port: u16,
    iface: String,

    car_cache: BonjourLiveCache<AirPlayBonjourEntry>,
    #[sleep]
    notify: Notify,
    had_first_invite: bool,
}

#[derive(Default)]
struct InviteStatus {
    pending: Option<BonjourCached<AirPlayBonjourEntry>>,
    cache_miss: bool,
}

impl CarPlayCtrlServer {
    const HOSTNAME: &str = "iPhone.local.";
    const INSTANCE_NAME: &str = "iPhone";

    const USE_BONJOUR: bool = true;

    pub fn new(bonjour_id: &str, port: u16, iface: &str) -> io::Result<Self> {
        let invite_status = InviteStatus::default();
        let bonjour = Bonjour::with_iface(iface, BonjourType::Ipv6AndIpv4)?;

        let car_cache = bonjour.watch::<AirPlayBonjourEntry>();

        let server = {
            let (invite_tx, invite_rx) = mpsc::unbounded();
            let server = TcpServer::bind_iface(iface, port, move |_peer| CarPlayCtrlSession::with_invite_sender(invite_tx.clone()))?;
            (server, invite_rx)
        };
        let (server, invite_rx) = server;

        let info = DeviceInfoBonjourEntry {
            model: AIRPLAY_TX_IOS_BOARD.into(),
        };

        let entry = CarPlayCtrlBonjourEntry {
            srcvers: AIRPLAY_TX_SDK_VERSION.into(),
            id: bonjour_id.into(),
        };

        let bonjour_handle_info = if Self::USE_BONJOUR {
            Some(bonjour.register::<DeviceInfoBonjourEntry>(info, Self::HOSTNAME, Self::INSTANCE_NAME, 0, &[])?)
        } else {
            None
        };

        let bonjour_handle = if Self::USE_BONJOUR {
            Some(bonjour.register::<CarPlayCtrlBonjourEntry>(entry, Self::HOSTNAME, Self::INSTANCE_NAME, port, &[])?)
        } else {
            None
        };

        Ok(Self {
            _bonjour: bonjour,
            bonjour_id: bonjour_id.into(),
            bonjour_handle,
            bonjour_handle_info,
            port,
            server,
            invite_rx,
            invite_status,
            car_cache,
            iface: iface.into(),
            notify: Notify::new(),
            had_first_invite: false,
        })
    }

    pub fn force_announce(&mut self) -> io::Result<()> {
        if let Some(handle) = self.bonjour_handle_info.as_mut() {
            handle.reannounce();
        }
        if let Some(handle) = self.bonjour_handle.as_mut() {
            handle.reannounce();
        }

        if !Self::USE_BONJOUR {
            let announcer = MdnsAnnouncer::new(&self.iface)?;
            let Some(addr_v6_ll) = announcer.ll_addr else {
                return Ok(());
            };
            announcer.send_mdns(
                &CarplayCtrlAnnounceParams {
                    instance: "iPhone SIM",
                    host: "iPhone-SIM",
                    port: self.port,
                    addr_v6_ll,
                    srcvers: AIRPLAY_TX_SDK_VERSION,
                    model: AIRPLAY_TX_IOS_BOARD,
                    device_id: &self.bonjour_id,
                    ttl: 4500,
                }
                .build_announce(),
            )?;
        }

        Ok(())
    }

    pub fn force_invite(&mut self) {
        if self.invite_status.pending.is_none() {
            self.invite_status = InviteStatus {
                pending: None,
                cache_miss: true,
            };
        }

        self.notify.notify();
    }

    pub fn pop_invite(&mut self) -> Option<BonjourCached<AirPlayBonjourEntry>> {
        if let Some(invite) = self.invite_rx.take() {
            self.process_invite(invite);
        }

        if let Some(car) = self.car_cache.borrow_and_update().iter().next() {
            if self.invite_status.cache_miss {
                warn!(
                    "Bonjour entry arrived post-invite, unglitching automatically with: {:?}",
                    car.1.entry
                );
                self.invite_status.pending.replace(car.1.clone());
                self.invite_status.cache_miss = false;
            } else if !car
                .1
                .entry
                .data
                .features
                .contains(AirPlayFeature::CARPLAY_CONTROL)
                && !self.had_first_invite
                && self.invite_status.pending.is_none()
            {
                warn!(
                    "Legacy headunit without CARPLAY_CONTROL flag, inviting self (only once per session): {:?}",
                    car.1.entry
                );
                self.invite_status.pending.replace(car.1.clone());
                self.had_first_invite = true;
            }
        }

        self.invite_status.pending.take()
    }

    fn process_invite(&mut self, invite: CarPlayCtrlInvite) {
        // Don't bother with matching by MAC address for now; assume (realistically) only one AirPlay server on the network
        let cache = self.car_cache.borrow();
        let airplay = cache.iter().next();
        let Some(airplay) = airplay else {
            warn!("Ignoring carplay-ctrl invite {invite:?} - missing matching AirPlay entry in Bonjour cache");
            self.invite_status.cache_miss = true;
            return;
        };

        let bonjour = airplay.1;
        debug!("Accepted invite {invite:?} with Bonjour {:?}", bonjour.entry);
        self.invite_status.pending.replace(bonjour.clone());
        self.invite_status.cache_miss = false;
    }
}

impl AsyncShutdown for CarPlayCtrlServer {
    async fn shutdown(&mut self) {
        debug!("CarPlayCtrlServer is shutting down!");
        self.server.shutdown().await;
    }
}

impl Drop for CarPlayCtrlServer {
    fn drop(&mut self) {
        debug!("CarPlayCtrlServer was dropped!");
    }
}

#[cfg(test)]
mod tests {
    use std::{io, net::Ipv4Addr, time::Duration};

    use catplay_bonjour::Bonjour;
    use catplay_util::AsyncShutdown;
    use macaddr::MacAddr6;
    use tokio::{spawn, sync::mpsc, time::sleep};

    use crate::ctrl::{CarPlayCtrlBonjourEntry, CarPlayCtrlClient, CarPlayCtrlInvite, CarPlayCtrlServer};

    // #[tokio::test]
    // async fn test_single_invite_and_shutdown() -> io::Result<()> {
    //     const _IP: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
    //     const IFACE: &str = "lo";
    //     const PORT: u16 = 5050;
    //     const ID: &str = "id";
    //     const DELAY: Duration = Duration::from_millis(1000);
    //     const MAC: MacAddr6 = MacAddr6::new(0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF);

    //     setup_test_logger(false);

    //     let mut srv = CarPlayCtrlServer::new(ID,  PORT, IFACE)?;

    //     let bonjour = Bonjour::with_iface(IFACE)?;

    //     let watcher = bonjour.watch::<CarPlayCtrlBonjourEntry>();
    //     sleep(DELAY).await;

    //     let entries = watcher.borrow().clone();
    //     let entry = entries.values().next().expect("bonjour not found");

    //     let client = CarPlayCtrlClient::new(&entry.entry);
    //     spawn(async move { client.connect(&MAC, Duration::from_millis(1000)).await.expect("failed to connect") });

    //     // Test single invite
    //     sleep(DELAY).await;

    //     let invite = srv.pop_invite().expect("expected invite");
    //     assert_eq!(invite, CarPlayCtrlInvite::new(MAC));

    //     // Test shutdown

    //     srv.shutdown().await;

    //     let client = CarPlayCtrlClient::new(&entry.entry);
    //     client.connect(&MAC, Duration::from_millis(1000)).await.expect_err("Connection refused");

    //     Ok(())
    // }
}
