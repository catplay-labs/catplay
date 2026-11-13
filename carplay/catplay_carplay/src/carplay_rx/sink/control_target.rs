pub(super) fn matches_target(id: &str, target: Option<&str>) -> bool {
    target.is_none_or(|target| !id.is_empty() && id.eq_ignore_ascii_case(target))
}

use std::{
    net::IpAddr,
    sync::{Arc, Mutex},
};
#[derive(Clone)]
pub(crate) struct PeerScope(Arc<()>);
#[derive(Clone, Default)]
pub(crate) struct TargetPeers(Arc<Mutex<Option<(PeerScope, Vec<IpAddr>)>>>);
impl TargetPeers {
    pub fn restrict(&self) -> PeerScope {
        let scope = PeerScope(Arc::new(()));
        *self.0.lock().unwrap() = Some((scope.clone(), Vec::new()));
        scope
    }
    pub fn update(&self, scope: &PeerScope, peers: Vec<IpAddr>) {
        if let Some((active, allowed)) = self.0.lock().unwrap().as_mut()
            && Arc::ptr_eq(&scope.0, &active.0)
        {
            *allowed = peers;
        }
    }
    pub fn allows(&self, peer: IpAddr) -> bool {
        self.0
            .lock()
            .unwrap()
            .as_ref()
            .is_none_or(|(_, peers)| peers.contains(&peer))
    }
    pub fn clear(&self) {
        *self.0.lock().unwrap() = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selected_phone_wins_even_when_old_phone_is_discovered_first() {
        let phones = ["AA:00:00:00:00:01", "bb:00:00:00:00:02"];
        assert_eq!(
            phones
                .iter()
                .find(|id| matches_target(id, Some("BB:00:00:00:00:02"))),
            Some(&phones[1])
        );
        assert!(!matches_target("", Some(phones[1])));
        assert!(!matches_target(phones[0], Some(phones[1])));
        assert!(matches_target(phones[0], None));
    }

    #[test]
    fn incoming_old_phone_and_stale_discovery_cannot_claim_the_selected_phone_slot() {
        let peers = TargetPeers::default();
        let old: IpAddr = "192.0.2.1".parse().unwrap();
        let selected: IpAddr = "192.0.2.2".parse().unwrap();
        let previous_scope = peers.restrict();
        let current_scope = peers.restrict();
        peers.update(&previous_scope, vec![old]);
        assert!(!peers.allows(old) && !peers.allows(selected));
        peers.update(&current_scope, vec![selected]);
        assert!(!peers.allows(old) && peers.allows(selected));
        peers.clear();
        peers.update(&current_scope, vec![selected]);
        assert!(peers.allows(old));
    }
}
