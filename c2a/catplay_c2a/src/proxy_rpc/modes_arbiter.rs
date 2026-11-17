use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use catplay_carplay::modes::{
    AppStateEnum, ChangeModes, ChangeModesResponse, EntityEnum, ModesChanged, Resource, ResourceConstraint, ResourceController, ResourceID,
    ResourceManager, ResourceState, ResourceTransferPriority, ResourceTransferType, SpeechMode, SpeechState,
};
use log::warn;

pub struct ModesArbiter {
    modes: ResourceController,
    modes_dirty: bool,
    /// when the car sent a requestUI that could not be served yet
    ui_requested_at: Option<Instant>,
    inflight_peer_change_modes: VecDeque<ChangeModes>,
    accepted_peer_change_modes: VecDeque<ChangeModes>,
    pending_peer_modes_changed: VecDeque<ModesChanged>,
}

impl ModesArbiter {
    const UI_REQUEST_TTL: Duration = Duration::from_secs(3);

    pub fn new(info_modes: &ChangeModes) -> Self {
        Self {
            modes: ResourceController::from_change_modes(info_modes),
            modes_dirty: true,
            ui_requested_at: None,
            inflight_peer_change_modes: VecDeque::new(),
            accepted_peer_change_modes: VecDeque::new(),
            pending_peer_modes_changed: VecDeque::new(),
        }
    }

    pub fn process_car_request(&mut self, request: &ChangeModes) -> ChangeModesResponse {
        self.mark_dirty();
        let resp = self.modes_mut().process_request_to_response(request);
        // some cars forbid taking the screen until they answer requestUI with a changeModes that allows it
        self.steal_screen_for_ui();
        resp
    }

    /// takes the screen for the car's requestUI, now if its constraints allow it,
    /// otherwise after a car request that arrives within [Self::UI_REQUEST_TTL]
    pub fn request_ui(&mut self) {
        self.ui_requested_at = Some(Instant::now());
        self.steal_screen_for_ui();
    }

    fn steal_screen_for_ui(&mut self) {
        let Some(requested_at) = self.ui_requested_at else {
            return;
        };

        // a later car request is not the user's tap anymore
        if requested_at.elapsed() > Self::UI_REQUEST_TTL {
            self.ui_requested_at = None;
            return;
        }

        self.try_steal_screen(ResourceTransferPriority::UserInitiated);
        if self.has_screen() {
            self.ui_requested_at = None;
        }
    }

    pub fn modes(&self) -> ResourceController {
        self.modes
    }

    pub fn modes_mut(&mut self) -> &mut ResourceController {
        &mut self.modes
    }

    pub fn modes_dirty(&self) -> bool {
        self.modes_dirty
    }

    pub fn clear_dirty(&mut self) {
        self.modes_dirty = false;
    }

    pub fn mark_dirty(&mut self) {
        self.modes_dirty = true;
    }

    pub fn reset_modes(&mut self) {
        // When iPhone peer disconnects, reset modes to be optimal for displaying the UI overlay again
        if self.modes.speech.entity == EntityEnum::Controller {
            self.modes.speech = SpeechState::default();
        }
        self.modes.phone_call = EntityEnum::None;
        self.modes.turn_by_turn = EntityEnum::None;

        self.modes.main_audio = ResourceManager::new(
            ResourceState::AccessoryHas,
            ResourceConstraint::Anytime,
            ResourceConstraint::Anytime,
        );
        self.modes.screen = ResourceManager::new(
            ResourceState::ControllerHas,
            ResourceConstraint::Anytime,
            ResourceConstraint::Anytime,
        );
        self.ui_requested_at = None;

        self.modes_dirty = true;
    }

    pub fn on_peer_disconnect(&mut self) {
        let inflights = self.inflight_peer_change_modes.len();
        let accepted = self.accepted_peer_change_modes.len();
        let pending = self.pending_peer_modes_changed.len();
        if inflights > 0 || accepted > 0 || pending > 0 {
            warn!("Removing some inflights during peer disconnect: {inflights}-{pending}-{accepted}");
            self.inflight_peer_change_modes.clear();
            self.accepted_peer_change_modes.clear();
            self.pending_peer_modes_changed.clear();
        }

        self.reset_modes();
    }

    pub fn try_steal_screen(&mut self, p: ResourceTransferPriority) {
        let modes = &mut self.modes;
        if modes.screen.entity() == EntityEnum::Controller {
            return;
        }

        // If Screen is taken by Accessory(or unclaimed) but we can take it back with NiceToHave(Anytime), always do it first.
        if modes.screen.can_take(p)
            && modes.screen.take_by_controller(&Resource::take(
                ResourceID::MainScreen,
                p,
                ResourceConstraint::Anytime,
                ResourceConstraint::Anytime,
            ))
        {
            warn!("Taking opportunity to steal back screen using Take");
            self.modes_dirty = true;
        } /*else if modes.screen.can_borrow(p)
        && modes.screen.borrow_by_controller(&Resource::borrow(
        ResourceID::MainScreen,
        ResourceTransferPriority::NiceToHave,
        ResourceConstraint::Anytime,
        ))
        {
        warn!("Taking opportunity to steal back screen using Borrow");
        self.modes_dirty = true;
        }*/
    }

    pub fn has_screen(&self) -> bool {
        self.modes.screen.entity() == EntityEnum::Controller
    }

    pub fn on_peer_modes_changed(&mut self, modes: &ModesChanged) {
        self.mark_dirty();

        if !self.inflight_peer_change_modes.is_empty() {
            self.pending_peer_modes_changed.push_back(modes.clone());
            return;
        }

        let accepted = self.accepted_peer_change_modes.pop_front();
        self.apply_peer_modes(modes, accepted.as_ref());
    }

    pub fn on_peer_change_modes_start(&mut self, modes: &ChangeModes) {
        self.inflight_peer_change_modes.push_back(modes.clone());
    }

    pub fn on_peer_change_modes_end(&mut self, resp: &ChangeModesResponse) {
        let Some(request) = self.inflight_peer_change_modes.pop_front() else {
            warn!("Received peer ChangeModes response without matching inflight request: {resp:?}");
            return;
        };

        if resp.status != 0 {
            warn!("Peer rejected ChangeModes request: status={}, request={request:?}", resp.status);
            self.drain_pending_peer_modes_changed_if_idle();
            return;
        }

        self.accepted_peer_change_modes.push_back(request);
        self.drain_pending_peer_modes_changed_if_idle();
    }

    fn apply_peer_modes(&mut self, modes: &ModesChanged, accepted_request: Option<&ChangeModes>) {
        let mut next = self.modes;

        for app_state in &modes.app_states {
            match app_state.app_state_id {
                AppStateEnum::Speech => {
                    next.speech = SpeechState {
                        entity: app_state.entity,
                        mode: app_state.speech_mode.unwrap_or(SpeechMode::None),
                    };
                }
                AppStateEnum::PhoneCall => next.phone_call = app_state.entity,
                AppStateEnum::TurnByTurn => next.turn_by_turn = app_state.entity,
                AppStateEnum::Invalid => {}
            }
        }

        for resource in &modes.resources {
            let new_state = ResourceState::from_pair(resource.entity, resource.permanent_entity);
            match resource.resource_id {
                ResourceID::MainScreen => {
                    next.screen = Self::merge_peer_resource(self.modes.screen, new_state, ResourceID::MainScreen, accepted_request);
                }
                ResourceID::MainAudio => {
                    next.main_audio = Self::merge_peer_resource(self.modes.main_audio, new_state, ResourceID::MainAudio, accepted_request);
                }
            }
        }

        self.modes = next;
        self.modes_dirty = true;
    }

    fn merge_peer_resource(
        old: ResourceManager,
        new_state: ResourceState,
        id: ResourceID,
        accepted_request: Option<&ChangeModes>,
    ) -> ResourceManager {
        if new_state == ResourceState::AccessoryBorrowed {
            let unborrow_constraint = accepted_request
                .and_then(|request| Self::find_resource(request, id))
                .filter(|resource| resource.transfer_type == ResourceTransferType::Borrow)
                .and_then(|resource| resource.unborrow_constraint)
                .unwrap_or_else(|| {
                    if old.state() == ResourceState::AccessoryBorrowed {
                        old.unborrow_constraint()
                    } else {
                        ResourceConstraint::Anytime
                    }
                });

            return ResourceManager::new(new_state, ResourceConstraint::Anytime, ResourceConstraint::Anytime)
                .with_unborrow_constraint(unborrow_constraint);
        }

        if new_state.owner() == EntityEnum::Controller {
            return ResourceManager::new(new_state, ResourceConstraint::Anytime, ResourceConstraint::Anytime);
        }

        let mut take_constraint = if old.owner() == EntityEnum::Accessory {
            old.take_constraint()
        } else {
            ResourceConstraint::Anytime
        };
        let mut borrow_constraint = if old.owner() == EntityEnum::Accessory {
            old.borrow_constraint()
        } else {
            ResourceConstraint::Anytime
        };

        if let Some(resource) = accepted_request.and_then(|request| Self::find_resource(request, id))
            && resource.transfer_type == ResourceTransferType::Take
        {
            take_constraint = resource.take_constraint.unwrap_or(take_constraint);
            borrow_constraint = resource.borrow_constraint.unwrap_or(borrow_constraint);
        }

        ResourceManager::new(new_state, take_constraint, borrow_constraint)
    }

    fn find_resource(request: &ChangeModes, id: ResourceID) -> Option<&Resource> {
        request
            .resources
            .iter()
            .find(|resource| resource.resource_id == id)
    }

    fn drain_pending_peer_modes_changed_if_idle(&mut self) {
        if !self.inflight_peer_change_modes.is_empty() {
            return;
        }

        while let Some(modes) = self.pending_peer_modes_changed.pop_front() {
            let accepted = self.accepted_peer_change_modes.pop_front();
            self.apply_peer_modes(&modes, accepted.as_ref());
        }
    }
}

#[cfg(test)]
mod tests {
    use catplay_carplay::modes::{
        ChangeModesResponse, EntityEnum, InitialPermanentEntity, ModesChanged, Resource, ResourceChanged, ResourceConstraint, ResourceID,
        ResourceState, ResourceTransferPriority,
    };

    use super::*;

    /// initial modes of a car that owns the screen and never lets it be taken
    fn screen_never_takeable_modes() -> ChangeModes {
        ChangeModes {
            app_states: vec![],
            resources: vec![
                Resource::take(
                    ResourceID::MainScreen,
                    ResourceTransferPriority::UserInitiated,
                    ResourceConstraint::Never,
                    ResourceConstraint::Anytime,
                ),
                Resource::take(
                    ResourceID::MainAudio,
                    ResourceTransferPriority::UserInitiated,
                    ResourceConstraint::UserInitiated,
                    ResourceConstraint::Anytime,
                ),
            ],
            reason_str: "initial".into(),
            initial_permanent_entity: vec![],
        }
    }

    fn car_takes_screen_allowing_user_initiated_take() -> ChangeModes {
        ChangeModes {
            app_states: vec![],
            resources: vec![Resource::take(
                ResourceID::MainScreen,
                ResourceTransferPriority::UserInitiated,
                ResourceConstraint::UserInitiated,
                ResourceConstraint::Anytime,
            )],
            reason_str: "no reason".into(),
            initial_permanent_entity: vec![],
        }
    }

    #[test]
    fn requested_ui_takes_screen_once_car_allows_it() {
        let mut arbiter = ModesArbiter::new(&screen_never_takeable_modes());

        arbiter.request_ui();
        assert!(!arbiter.has_screen());

        let resp = arbiter.process_car_request(&car_takes_screen_allowing_user_initiated_take());
        assert_eq!(resp.status, 0);
        assert!(arbiter.has_screen());
        assert!(arbiter.modes_dirty());

        // the request is spent, the car can take the screen back and keep it
        let resp = arbiter.process_car_request(&car_takes_screen_allowing_user_initiated_take());
        assert_eq!(resp.status, 0);
        assert!(!arbiter.has_screen());
    }

    #[test]
    fn requested_ui_served_at_once_does_not_steal_later() {
        let mut arbiter = ModesArbiter::new(&car_takes_screen_allowing_user_initiated_take());

        arbiter.request_ui();
        assert!(arbiter.has_screen());

        let resp = arbiter.process_car_request(&car_takes_screen_allowing_user_initiated_take());
        assert_eq!(resp.status, 0);
        assert!(!arbiter.has_screen());
    }

    #[test]
    fn requested_ui_stays_pending_while_car_keeps_screen_untakeable() {
        let mut arbiter = ModesArbiter::new(&screen_never_takeable_modes());

        arbiter.request_ui();
        let resp = arbiter.process_car_request(&screen_never_takeable_modes());
        assert_eq!(resp.status, 0);
        assert!(!arbiter.has_screen());

        let resp = arbiter.process_car_request(&car_takes_screen_allowing_user_initiated_take());
        assert_eq!(resp.status, 0);
        assert!(arbiter.has_screen());
    }

    #[test]
    fn requested_ui_expires() {
        let mut arbiter = ModesArbiter::new(&screen_never_takeable_modes());

        arbiter.request_ui();
        arbiter.ui_requested_at = Instant::now().checked_sub(ModesArbiter::UI_REQUEST_TTL + Duration::from_secs(1));
        assert!(arbiter.ui_requested_at.is_some());

        let resp = arbiter.process_car_request(&car_takes_screen_allowing_user_initiated_take());
        assert_eq!(resp.status, 0);
        assert!(!arbiter.has_screen());
        assert!(arbiter.ui_requested_at.is_none());
    }

    #[test]
    fn car_change_modes_without_requested_ui_keeps_screen_with_car() {
        let mut arbiter = ModesArbiter::new(&screen_never_takeable_modes());

        let resp = arbiter.process_car_request(&car_takes_screen_allowing_user_initiated_take());
        assert_eq!(resp.status, 0);
        assert!(!arbiter.has_screen());
    }

    #[test]
    fn peer_change_modes_cycle_preserves_accessory_constraints() {
        let info_modes = ChangeModes {
            app_states: vec![],
            resources: vec![Resource::untake(ResourceID::MainScreen), Resource::untake(ResourceID::MainAudio)],
            reason_str: "initial".into(),
            initial_permanent_entity: vec![
                InitialPermanentEntity::controller(ResourceID::MainScreen),
                InitialPermanentEntity::controller(ResourceID::MainAudio),
            ],
        };
        let mut arbiter = ModesArbiter::new(&info_modes);

        let request = ChangeModes {
            app_states: vec![],
            resources: vec![Resource::take(
                ResourceID::MainAudio,
                ResourceTransferPriority::UserInitiated,
                ResourceConstraint::Never,
                ResourceConstraint::UserInitiated,
            )],
            reason_str: "car takes main audio".into(),
            initial_permanent_entity: vec![],
        };
        let changed = ModesChanged {
            app_states: vec![],
            resources: vec![ResourceChanged {
                resource_id: ResourceID::MainAudio,
                entity: EntityEnum::Accessory,
                permanent_entity: EntityEnum::Accessory,
            }],
            reason_str: "accepted".into(),
        };

        arbiter.on_peer_change_modes_start(&request);
        arbiter.on_peer_change_modes_end(&ChangeModesResponse::new(changed.clone()));
        arbiter.on_peer_modes_changed(&changed);

        assert_eq!(arbiter.modes.main_audio.state(), ResourceState::AccessoryHas);
        assert_eq!(arbiter.modes.main_audio.take_constraint(), ResourceConstraint::Never);
        assert_eq!(arbiter.modes.main_audio.borrow_constraint(), ResourceConstraint::UserInitiated);
    }

    #[test]
    fn peer_change_modes_cycle_preserves_accessory_borrow_constraint() {
        let info_modes = ChangeModes {
            app_states: vec![],
            resources: vec![Resource::untake(ResourceID::MainScreen), Resource::untake(ResourceID::MainAudio)],
            reason_str: "initial".into(),
            initial_permanent_entity: vec![
                InitialPermanentEntity::controller(ResourceID::MainScreen),
                InitialPermanentEntity::controller(ResourceID::MainAudio),
            ],
        };
        let mut arbiter = ModesArbiter::new(&info_modes);

        let request = ChangeModes {
            app_states: vec![],
            resources: vec![Resource::borrow(
                ResourceID::MainAudio,
                ResourceTransferPriority::UserInitiated,
                ResourceConstraint::Never,
            )],
            reason_str: "car borrows main audio".into(),
            initial_permanent_entity: vec![],
        };
        let changed = ModesChanged {
            app_states: vec![],
            resources: vec![ResourceChanged {
                resource_id: ResourceID::MainAudio,
                entity: EntityEnum::Accessory,
                permanent_entity: EntityEnum::Controller,
            }],
            reason_str: "accepted".into(),
        };

        arbiter.on_peer_change_modes_start(&request);
        arbiter.on_peer_change_modes_end(&ChangeModesResponse::new(changed.clone()));
        arbiter.on_peer_modes_changed(&changed);

        assert_eq!(arbiter.modes.main_audio.state(), ResourceState::AccessoryBorrowed);
        assert_eq!(arbiter.modes.main_audio.take_constraint(), ResourceConstraint::Anytime);
        assert_eq!(arbiter.modes.main_audio.borrow_constraint(), ResourceConstraint::Anytime);
        assert_eq!(arbiter.modes.main_audio.unborrow_constraint(), ResourceConstraint::Never);
    }

    #[test]
    fn peer_modes_changed_is_debounced_while_change_modes_is_inflight() {
        let info_modes = ChangeModes {
            app_states: vec![],
            resources: vec![Resource::untake(ResourceID::MainScreen), Resource::untake(ResourceID::MainAudio)],
            reason_str: "initial".into(),
            initial_permanent_entity: vec![
                InitialPermanentEntity::controller(ResourceID::MainScreen),
                InitialPermanentEntity::controller(ResourceID::MainAudio),
            ],
        };
        let mut arbiter = ModesArbiter::new(&info_modes);

        let request = ChangeModes {
            app_states: vec![],
            resources: vec![Resource::take(
                ResourceID::MainAudio,
                ResourceTransferPriority::UserInitiated,
                ResourceConstraint::Never,
                ResourceConstraint::UserInitiated,
            )],
            reason_str: "car takes main audio".into(),
            initial_permanent_entity: vec![],
        };
        let changed = ModesChanged {
            app_states: vec![],
            resources: vec![ResourceChanged {
                resource_id: ResourceID::MainAudio,
                entity: EntityEnum::Accessory,
                permanent_entity: EntityEnum::Accessory,
            }],
            reason_str: "accepted before response".into(),
        };

        arbiter.on_peer_change_modes_start(&request);
        arbiter.on_peer_modes_changed(&changed);

        assert_eq!(arbiter.modes.main_audio.state(), ResourceState::ControllerHas);

        arbiter.on_peer_change_modes_end(&ChangeModesResponse { status: 0, params: None });

        assert_eq!(arbiter.modes.main_audio.state(), ResourceState::AccessoryHas);
        assert_eq!(arbiter.modes.main_audio.take_constraint(), ResourceConstraint::Never);
        assert_eq!(arbiter.modes.main_audio.borrow_constraint(), ResourceConstraint::UserInitiated);
    }
}
