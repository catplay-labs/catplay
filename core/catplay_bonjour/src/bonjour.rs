use std::{
    collections::HashMap,
    io,
    net::{IpAddr, SocketAddr},
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Duration,
};

use log::{debug, info, warn};
use tokio::{sync::watch, time::interval};
use tokio_util::task::AbortOnDropHandle;

use crate::{
    BonjourCached, BonjourEntryType, BonjourError, BonjourHandler, BonjourHandlerWatch, BonjourMeta,
    entry_handle::BonjourEntryHandle,
    raw::{MdnsAnnouncer, RawMdnsBackend, RawMdnsEvent},
};

enum BonjourBackend {
    Raw(Arc<RawMdnsBackend>),
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum BonjourType {
    Ipv6,
    Ipv4,
    Ipv6AndIpv4,
}
#[derive(Clone)]
#[allow(unused)]
pub struct Bonjour {
    backend: Arc<BonjourBackend>,
    iface: Option<String>,
}

impl Bonjour {
    pub fn with_iface(iface: &str, btype: BonjourType) -> Result<Self, BonjourError> {
        if let Err((family, source)) = MdnsAnnouncer::probe_multicast_stability(iface, btype) {
            return Err(BonjourError::MulticastUnstable {
                iface: iface.to_string(),
                family,
                source,
            });
        }

        let backend = RawMdnsBackend::new(iface, btype)?;
        Ok(Self {
            backend: Arc::new(BonjourBackend::Raw(Arc::new(backend))),
            iface: Some(iface.to_string()),
        })
    }

    pub fn register<E: BonjourEntryType>(
        &self,
        entry: E::Entry,
        hostname: &str,
        instance_name: &str,
        port: u16,
        ips: &[IpAddr],
    ) -> io::Result<BonjourEntryHandle> {
        let mut txt = HashMap::new();
        E::to_props(entry, &mut txt);
        let addrs = ips.iter().map(|ip| SocketAddr::new(*ip, port)).collect();

        let meta = BonjourMeta {
            iface: self.iface.clone().unwrap_or_default(),
            hostname: hostname.into(),
            port,
            ips: ips.into(),
            addrs,
            txt,
            service_type: E::SERVICE_TYPE.into(),
            instance_name: instance_name.into(),
            fullname: format!("{instance_name}.{}", E::SERVICE_TYPE),
        };

        debug!("Registering Bonjour [{}] meta: {:?}", meta.service_type, meta);

        match self.backend.as_ref() {
            BonjourBackend::Raw(raw) => {
                let reg_token = raw.register(&meta)?;
                info!("Starting advertisement of Bonjour '{}' at {}", meta.instance_name, E::SERVICE_TYPE);

                let fullname = meta.fullname.clone();
                let raw_reannounce = raw.clone();
                let reg_token_reannounce = reg_token.clone();
                let reannounce_fn = Arc::new(move || raw_reannounce.reannounce(&fullname, &reg_token_reannounce));

                let raw_close = raw.clone();
                let meta_close = meta.clone();
                let reg_token_close = reg_token.clone();
                let close_fn = Arc::new(move || raw_close.unregister(&meta_close, &reg_token_close));

                Ok(BonjourEntryHandle::new(meta, reannounce_fn, close_fn))
            }
        }
    }

    pub fn watch<E: BonjourEntryType>(&self) -> BonjourLiveCache<E> {
        let (handler, rx) = BonjourHandlerWatch::<E>::new();
        let handler = Arc::new(handler);

        let task = match self.backend.as_ref() {
            BonjourBackend::Raw(raw) => {
                let handler2 = handler.clone();
                let mut events = raw.subscribe();
                let raw = raw.clone();
                tokio::spawn(async move {
                    info!("Starting Bonjour observer for {}", handler2.service_type());
                    handler2.clear_cache();
                    for meta in raw.snapshot_service(handler2.service_type()) {
                        let _ = handler2.on_resolved(meta);
                    }
                    raw.query_service(handler2.service_type());
                    let mut query_tick = interval(Duration::from_secs(5));
                    loop {
                        tokio::select! {
                            _ = query_tick.tick() => {
                                raw.query_service(handler2.service_type());
                            }
                            event = events.recv() => {
                                let ev = match event {
                                    Ok(ev) => ev,
                                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                                        warn!(
                                            "Bonjour observer lagged for {} (skipped {} events), rehydrating cache",
                                            handler2.service_type(),
                                            skipped
                                        );
                                        handler2.clear_cache();
                                        for meta in raw.snapshot_service(handler2.service_type()) {
                                            let _ = handler2.on_resolved(meta);
                                        }
                                        raw.query_service(handler2.service_type());
                                        continue;
                                    }
                                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                                };
                                let mut dirty = false;
                                debug!("Bonjour event: {ev:?}");

                                match ev {
                                    RawMdnsEvent::Resolved(meta) => dirty |= handler2.on_resolved(meta),
                                    RawMdnsEvent::Removed { service_type, fullname } => dirty |= handler2.on_removed(&service_type, &fullname),
                                }
                                if dirty {
                                    debug!("Bonjour cache updated ({})", handler2.service_type());
                                }
                            }
                        }
                    }
                })
            }
        };

        BonjourLiveCache {
            rx,
            service_type: E::SERVICE_TYPE.into(),
            _task: Arc::new(AbortOnDropHandle::new(task)),
        }
    }
}

pub struct BonjourLiveCache<E: BonjourEntryType> {
    service_type: String,
    rx: watch::Receiver<HashMap<String, BonjourCached<E::Entry>>>,
    _task: Arc<AbortOnDropHandle<()>>,
}

impl<E: BonjourEntryType> Clone for BonjourLiveCache<E> {
    fn clone(&self) -> Self {
        Self {
            service_type: self.service_type.clone(),
            rx: self.rx.clone(),
            _task: self._task.clone(),
        }
    }
}

impl<E: BonjourEntryType> Deref for BonjourLiveCache<E> {
    type Target = watch::Receiver<HashMap<String, BonjourCached<E::Entry>>>;

    fn deref(&self) -> &Self::Target {
        &self.rx
    }
}

impl<E: BonjourEntryType> DerefMut for BonjourLiveCache<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rx
    }
}

impl<E: BonjourEntryType> Drop for BonjourLiveCache<E> {
    fn drop(&mut self) {
        info!("Stopping Bonjour observer for {}", self.service_type)
    }
}

impl Drop for Bonjour {
    fn drop(&mut self) {
        debug!("Bonjour was dropped!");
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        net::{IpAddr, Ipv4Addr, SocketAddr},
        time::Duration,
    };

    use catplay_tracing::logger::setup_test_logger;
    use tokio::time::sleep;

    use crate::{Bonjour, BonjourCached, BonjourEntryType, BonjourMeta, bonjour::BonjourType, get_bonjour_txt_optional};

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct CarPlayCtrlBonjourEntry {
        pub srcvers: String,
        pub id: String,
    }

    impl BonjourEntryType for CarPlayCtrlBonjourEntry {
        const SERVICE_TYPE: &'static str = "_carplay-ctrl._tcp.local.";
        type Entry = CarPlayCtrlBonjourEntry;

        fn from_props(data: &HashMap<String, String>) -> CarPlayCtrlBonjourEntry {
            Self {
                srcvers: get_bonjour_txt_optional(data, "srcvers"),
                id: get_bonjour_txt_optional(data, "id"),
            }
        }

        fn to_props(data: CarPlayCtrlBonjourEntry, output: &mut HashMap<String, String>) {
            output.insert("srcvers".into(), data.srcvers.clone());
            output.insert("id".into(), data.id.clone());
        }
    }

    #[tokio::test]
    async fn test_wifi_register() {
        setup_test_logger(false);

        let bonjour = Bonjour::with_iface("wlan0", BonjourType::Ipv4).unwrap();
        let entry = CarPlayCtrlBonjourEntry {
            srcvers: "123".into(),
            id: "321".into(),
        };
        let _handle = bonjour
            .register::<CarPlayCtrlBonjourEntry>(entry, "carplay2.local.", "carplay2", 7000, &[])
            .unwrap();
        let _watcher = bonjour.watch::<CarPlayCtrlBonjourEntry>();

        sleep(Duration::from_secs(30)).await;
    }

    #[tokio::test]
    async fn test_register_unregister_autoip() {
        setup_test_logger(false);

        let bonjour = Bonjour::with_iface("lo", BonjourType::Ipv6AndIpv4).unwrap();
        let entry = CarPlayCtrlBonjourEntry {
            srcvers: "123".into(),
            id: "321".into(),
        };
        let ips = vec![IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))];

        let handle = bonjour
            .register::<CarPlayCtrlBonjourEntry>(entry, "carplay.local.", "carplay", 7000, &[])
            .unwrap();
        let watcher = bonjour.watch::<CarPlayCtrlBonjourEntry>();

        sleep(Duration::from_secs(5)).await;

        let expect_meta = BonjourMeta {
            iface: "lo".into(),
            hostname: "carplay.local.".into(),
            port: 7000,
            ips: ips.clone(),
            txt: HashMap::from([("id".into(), "321".into()), ("srcvers".into(), "123".into())]),
            service_type: "_carplay-ctrl._tcp.local.".into(),
            instance_name: "carplay".into(),
            addrs: ips.iter().map(|i| SocketAddr::new(*i, 7000)).collect(),
            fullname: "carplay._carplay-ctrl._tcp.local.".into(),
        };
        let expect_entry = CarPlayCtrlBonjourEntry {
            srcvers: "123".into(),
            id: "321".into(),
        };

        let mut expect_map = HashMap::new();
        expect_map.insert(
            "carplay._carplay-ctrl._tcp.local.".to_string(),
            BonjourCached::<CarPlayCtrlBonjourEntry> {
                entry: crate::BonjourEntry {
                    meta: expect_meta,
                    data: expect_entry,
                },
                last_ping: None,
                masked: false,
            },
        );

        assert_eq!(watcher.borrow().clone(), expect_map, "Registration failed");

        drop(handle);
        sleep(Duration::from_secs(3)).await;
        assert_eq!(watcher.borrow().clone(), HashMap::new(), "Unregistration failed");
    }
}
