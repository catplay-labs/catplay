use std::collections::HashMap;

use log::{info, warn};
use tokio::sync::watch;

use crate::{BonjourCached, BonjourEntry, BonjourEntryType, BonjourHandler, BonjourMeta};

pub(crate) struct BonjourHandlerWatch<E: BonjourEntryType> {
    sender: watch::Sender<HashMap<String, BonjourCached<E::Entry>>>,
}

impl<E: BonjourEntryType> BonjourHandlerWatch<E> {
    #[allow(clippy::type_complexity)]
    pub fn new() -> (Self, watch::Receiver<HashMap<String, BonjourCached<E::Entry>>>) {
        let (tx, rx) = watch::channel(HashMap::<String, BonjourCached<E::Entry>>::new());
        (BonjourHandlerWatch::<E> { sender: tx }, rx)
    }
}

impl<E: BonjourEntryType> BonjourHandler for BonjourHandlerWatch<E> {
    fn service_type(&self) -> &'static str {
        E::SERVICE_TYPE
    }

    fn on_resolved(&self, meta: BonjourMeta) -> bool {
        if meta.service_type != E::SERVICE_TYPE {
            return false;
        }

        let entry = E::from_props(&meta.txt);
        self.sender
            .send_if_modified(|s| match s.get_mut(&meta.fullname) {
                None => {
                    info!(
                        "Found new service {} on iface {} at {:?} with {:?}",
                        meta.fullname, meta.iface, meta.addrs, meta.txt
                    );
                    s.insert(meta.fullname.to_string(), BonjourCached::new(BonjourEntry::new(meta, entry)));
                    true
                }
                Some(old) => {
                    let changed = old.entry.data != entry || old.entry.meta != meta;
                    old.entry.data = entry;
                    old.entry.meta = meta.clone();

                    if changed && !old.masked {
                        info!(
                            "Found updated service {} on iface {} at {:?} with {:?}",
                            meta.fullname, meta.iface, meta.addrs, meta.txt
                        );
                    }

                    changed && !old.masked
                }
            })
    }

    fn on_removed(&self, service_type: &str, fullname: &str) -> bool {
        if service_type != E::SERVICE_TYPE {
            return false;
        }

        self.sender.send_if_modified(|s| {
            let removed = s.remove(fullname);
            if let Some(removed) = removed {
                let meta = removed.entry.meta;
                warn!(
                    "Removed cached service {} on iface {} at {:?} with {:?}",
                    meta.fullname, meta.iface, meta.addrs, meta.txt
                );
                true
            } else {
                false
            }
        })
    }

    fn clear_cache(&self) {
        warn!("Clearing Bonjour observer cache");
        let _ = self.sender.send_replace(HashMap::new());
    }
}
