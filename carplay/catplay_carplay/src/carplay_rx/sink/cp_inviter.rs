use std::{io, time::Duration};

use catplay_bonjour::{Bonjour, BonjourEntryHandle, BonjourType};
use catplay_util::{AbortOnDropHandle, sleep, spawn};
use log::debug;
use macaddr::MacAddr6;
use tokio::select;

use crate::ctrl::{CarPlayCtrlBonjourEntry, CarPlayCtrlClient};

/// Keeps sending CarPlay invites to iPhones, until the struct is dropped.
pub struct CarPlayInviter {
    _bonjour: Bonjour,
    _entry: BonjourEntryHandle,
    _mac_addr: MacAddr6,
    _task: AbortOnDropHandle<()>,
}

impl CarPlayInviter {
    pub fn new(iface: &str, mac_addr: MacAddr6, entry: BonjourEntryHandle) -> io::Result<Self> {
        let bonjour = Bonjour::with_iface(iface, BonjourType::Ipv6AndIpv4)?;
        Ok(Self::with_bonjour(&bonjour, mac_addr, entry))
    }

    pub fn with_bonjour(bonjour: &Bonjour, mac_addr: MacAddr6, entry: BonjourEntryHandle) -> Self {
        let task = spawn(Self::work(bonjour.clone(), mac_addr, entry.clone()));

        Self {
            _bonjour: bonjour.clone(),
            _task: task,
            _mac_addr: mac_addr,
            _entry: entry,
        }
    }

    async fn work(bonjour: Bonjour, mac_addr: MacAddr6, entry: BonjourEntryHandle) {
        const TIMEOUT: Duration = Duration::from_secs(3);
        const MAX_ATTEMPTS: usize = 5;
        const RETRY_DELAY: Duration = Duration::from_millis(1000);

        // Note that some OEMs in Wi-Fi mode perform their own "stack ranking" if there are multiple phones in the network before inviting ?

        debug!("Watching for clients");
        let mut recv = bonjour.watch::<CarPlayCtrlBonjourEntry>();

        // Possibly sleep here after watch is started to differentiate between 1 vs >1 phones on first iteration?

        loop {
            let b = recv.borrow().clone();
            // Forcefully select first phone for now
            let phone = b.iter().next();

            if let Some(phone) = phone {
                // For some reason, iPhone often loses or discards our initial Bonjour advertisement
                // and then invites get accepted but never result in any connections.
                // This is easily reproducible if Wi-Fi connects before Bluetooth does,
                // and restarting Wi-Fi on iPhone's side results in immediate connection...

                // To fix it, we force advertisement before inviting.
                entry.reannounce();

                let client = CarPlayCtrlClient::new(&phone.1.entry);
                let hostname = &phone.1.entry.meta.hostname;

                // Note that id is guaranteed to be a Bluetooth ID of the phone
                let _id = &phone.1.entry.data.id;
                for i in 0..MAX_ATTEMPTS {
                    debug!("Inviting client(attempt {}/{}): {:?}", i + 1, MAX_ATTEMPTS, hostname);
                    select! {
                        result = client.connect(&mac_addr, TIMEOUT) => {
                            match result {
                                Ok(_) => { debug!("Invite accepted by {}", hostname); break },
                                Err(err) => {
                                    debug!("Invite rejected by {}: {err:?}", hostname)
                                }
                            }
                        }

                        Ok(_) = recv.changed() => {
                            debug!("Interrupting, because a change was detected");
                        }
                    }
                }
            }

            select! {
                _ = sleep(RETRY_DELAY) => {}
                Ok(_) = recv.changed() => {
                    debug!("Interrupting, because a change was detected");
                }
            }
        }
    }
}

impl Drop for CarPlayInviter {
    fn drop(&mut self) {
        debug!("Dropped CarPlayInviter!");
    }
}
