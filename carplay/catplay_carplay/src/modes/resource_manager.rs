use log::warn;

use crate::modes::{
    AirPlayModeState, AirPlayModeTransaction, AppStateEnum, ChangeModes, ChangeModesResponse, EntityEnum, Resource, ResourceConstraint,
    ResourceID, ResourcePermanentEntity, ResourceState, ResourceTransaction, ResourceTransferPriority, SpeechMode, SpeechState,
};

use super::ResourceTransferType;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceController {
    pub screen: ResourceManager,
    pub main_audio: ResourceManager,
    pub phone_call: EntityEnum,
    pub speech: SpeechState,
    pub turn_by_turn: EntityEnum,
}

impl Default for ResourceController {
    fn default() -> Self {
        Self {
            screen: ResourceManager::controller_owned(),
            main_audio: ResourceManager::controller_owned(),
            phone_call: EntityEnum::None,
            speech: SpeechState::default(),
            turn_by_turn: EntityEnum::None,
        }
    }
}

impl ResourceController {
    pub fn new(screen: ResourceManager, main_audio: ResourceManager) -> Self {
        Self {
            screen,
            main_audio,
            ..Self::default()
        }
    }

    pub fn from_change_modes(modes: &ChangeModes) -> Self {
        let transaction = AirPlayModeTransaction::from(modes);
        let mut this = Self::default();

        if let Some(entity) = transaction.screen_perm {
            this.screen = ResourceManager::from_permanent_entity(entity);
        }

        if let Some(entity) = transaction.main_audio_perm {
            this.main_audio = ResourceManager::from_permanent_entity(entity);
        }

        this.screen.apply_initial_transaction(transaction.screen);
        this.main_audio
            .apply_initial_transaction(transaction.main_audio);

        if let Some(phone_call) = transaction.phone_call {
            this.phone_call = phone_call;
        }

        if let Some(turn_by_turn) = transaction.turn_by_turn {
            this.turn_by_turn = turn_by_turn;
        }

        if let Some(speech) = transaction.speech {
            this.speech = speech;
        }

        this
    }

    pub fn serialize_to_request(&self) -> ChangeModes {
        self.serialize_to_transaction().serialize_to_change_modes()
    }

    pub fn serialize_to_info_request(&self) -> ChangeModes {
        self.serialize_to_info_transaction()
            .serialize_to_info_change_modes()
    }

    pub fn serialize_to_transaction(&self) -> AirPlayModeTransaction {
        AirPlayModeTransaction {
            screen: Some(self.screen.serialize_to_transaction()),
            main_audio: Some(self.main_audio.serialize_to_transaction()),
            phone_call: Some(self.phone_call),
            turn_by_turn: Some(self.turn_by_turn),
            speech: Some(self.speech),
            ..AirPlayModeTransaction::default()
        }
    }

    pub fn serialize_to_info_transaction(&self) -> AirPlayModeTransaction {
        AirPlayModeTransaction {
            screen: self.screen.serialize_to_info_transaction(),
            screen_perm: self.screen.serialize_to_permanent_entity(),
            main_audio: self.main_audio.serialize_to_info_transaction(),
            main_audio_perm: self.main_audio.serialize_to_permanent_entity(),
            phone_call: Some(self.phone_call),
            turn_by_turn: Some(self.turn_by_turn),
            speech: Some(self.speech),
            ..AirPlayModeTransaction::default()
        }
    }

    pub fn serialize_to_state(&self) -> AirPlayModeState {
        AirPlayModeState {
            screen: self.screen.state(),
            main_audio: self.main_audio.state(),
            phone_call: self.phone_call,
            speech: self.speech,
            turn_by_turn: self.turn_by_turn,
        }
    }

    pub fn process_request(&mut self, request: &ChangeModes) -> bool {
        let mut next = *self;

        for app_state in &request.app_states {
            if !next.process_app_state(app_state) {
                return false;
            }
        }

        for resource in &request.resources {
            let id = resource.resource_id;
            let res = match id {
                ResourceID::MainScreen => &mut next.screen,
                ResourceID::MainAudio => &mut next.main_audio,
            };

            if !res.process_accessory_request(resource) {
                warn!("Rejected {id:?} resource request: {resource:?} at {res:?}");
                return false;
            }
        }

        *self = next;
        true
    }

    pub fn process_request_to_response(&mut self, request: &ChangeModes) -> ChangeModesResponse {
        if self.process_request(request) {
            ChangeModesResponse::new(self.serialize_to_state().serialize())
        } else {
            ChangeModesResponse::error(1)
        }
    }

    fn process_app_state(&mut self, app_state: &crate::modes::AppState) -> bool {
        match app_state.app_state_id {
            AppStateEnum::Speech => {
                let Some(mode) = app_state.speech_mode else {
                    return false;
                };
                self.speech = SpeechState {
                    entity: if mode == SpeechMode::None {
                        EntityEnum::None
                    } else {
                        EntityEnum::Accessory
                    },
                    mode,
                };
            }
            AppStateEnum::PhoneCall => {
                let Some(state) = app_state.state else {
                    return false;
                };
                if state {
                    if self.phone_call == EntityEnum::Controller {
                        return false;
                    }
                    self.phone_call = EntityEnum::Accessory;
                } else if self.phone_call == EntityEnum::Accessory {
                    self.phone_call = EntityEnum::None;
                }
            }
            AppStateEnum::TurnByTurn => {
                let Some(state) = app_state.state else {
                    return false;
                };
                if state {
                    self.turn_by_turn = EntityEnum::Accessory;
                } else if self.turn_by_turn == EntityEnum::Accessory {
                    self.turn_by_turn = EntityEnum::None;
                }
            }
            AppStateEnum::Invalid => return false,
        }

        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceManager {
    state: ResourceState,
    take_constraint: ResourceConstraint,
    borrow_constraint: ResourceConstraint,
    unborrow_constraint: ResourceConstraint,
    borrows_count: usize,
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::unclaimed()
    }
}

impl ResourceManager {
    pub const fn new(state: ResourceState, take_constraint: ResourceConstraint, borrow_constraint: ResourceConstraint) -> Self {
        Self {
            state,
            take_constraint,
            borrow_constraint,
            unborrow_constraint: ResourceConstraint::Anytime,
            borrows_count: 0,
        }
    }

    pub const fn with_unborrow_constraint(mut self, unborrow_constraint: ResourceConstraint) -> Self {
        self.unborrow_constraint = unborrow_constraint;
        self
    }

    pub fn from_resource(resource: &Resource) -> Self {
        let state = ResourceState::from_request(resource.transfer_type);
        let take_constraint = resource
            .take_constraint
            .unwrap_or(ResourceConstraint::Anytime);
        let borrow_constraint = resource
            .borrow_constraint
            .unwrap_or(ResourceConstraint::Anytime);
        let unborrow_constraint = resource
            .unborrow_constraint
            .unwrap_or(ResourceConstraint::Anytime);

        Self::new(state, take_constraint, borrow_constraint).with_unborrow_constraint(unborrow_constraint)
    }

    pub const fn controller_owned() -> Self {
        Self::new(
            ResourceState::ControllerHas,
            ResourceConstraint::Anytime,
            ResourceConstraint::Anytime,
        )
    }

    pub const fn accessory_owned(take_constraint: ResourceConstraint, borrow_constraint: ResourceConstraint) -> Self {
        Self::new(ResourceState::AccessoryHas, take_constraint, borrow_constraint)
    }

    pub const fn unclaimed() -> Self {
        Self::new(ResourceState::Invalid, ResourceConstraint::Anytime, ResourceConstraint::Anytime)
    }

    pub const fn from_permanent_entity(entity: ResourcePermanentEntity) -> Self {
        match entity {
            ResourcePermanentEntity::Controller => Self::controller_owned(),
            ResourcePermanentEntity::Accessory {
                take_constraint,
                borrow_constraint,
            } => Self::accessory_owned(take_constraint, borrow_constraint),
        }
    }

    fn apply_initial_transaction(&mut self, transaction: Option<ResourceTransaction>) {
        match transaction {
            Some(ResourceTransaction::Take {
                take_constraint,
                borrow_constraint,
                ..
            }) => *self = Self::accessory_owned(take_constraint, borrow_constraint),
            Some(ResourceTransaction::Borrow { unborrow_constraint, .. }) => {
                if self.owner() == EntityEnum::Controller || self.is_unclaimed() {
                    self.state = ResourceState::AccessoryBorrowed;
                    self.unborrow_constraint = unborrow_constraint;
                    self.borrows_count = 1;
                }
            }
            Some(ResourceTransaction::Untake | ResourceTransaction::Unborrow) | None => {}
        }
    }

    /// Can the (other party) take the resource from [Self::entity] (current user).
    pub fn can_take(&self, priority: ResourceTransferPriority) -> bool {
        let constraint = if self.state.is_borrowed() {
            self.unborrow_constraint
        } else {
            self.take_constraint
        };

        Self::priority_satisfies(priority, constraint)
    }

    /// Can the (other party) borrow the resource from [Self::entity] (current user).
    pub fn can_borrow(&self, priority: ResourceTransferPriority) -> bool {
        Self::priority_satisfies(priority, self.borrow_constraint)
    }

    pub fn is_borrowed(&self) -> bool {
        self.state.is_borrowed()
    }

    pub fn is_unclaimed(&self) -> bool {
        self.state.is_unclaimed()
    }

    pub fn owner(&self) -> EntityEnum {
        self.state.owner()
    }

    pub fn entity(&self) -> EntityEnum {
        self.state.entity()
    }

    pub fn state(&self) -> ResourceState {
        self.state
    }

    pub fn take_constraint(&self) -> ResourceConstraint {
        self.take_constraint
    }

    pub fn borrow_constraint(&self) -> ResourceConstraint {
        self.borrow_constraint
    }

    pub fn unborrow_constraint(&self) -> ResourceConstraint {
        self.unborrow_constraint
    }

    pub fn serialize_to_transaction(&self) -> ResourceTransaction {
        match self.state {
            ResourceState::AccessoryHas => ResourceTransaction::Take {
                priority: ResourceTransferPriority::UserInitiated,
                take_constraint: self.take_constraint,
                borrow_constraint: self.borrow_constraint,
            },
            ResourceState::AccessoryBorrowed => ResourceTransaction::Borrow {
                priority: ResourceTransferPriority::UserInitiated,
                unborrow_constraint: self.unborrow_constraint,
            },
            ResourceState::ControllerHas | ResourceState::ControllerBorrowed | ResourceState::Invalid => ResourceTransaction::Untake,
        }
    }

    pub fn serialize_to_info_transaction(&self) -> Option<ResourceTransaction> {
        match self.state {
            ResourceState::AccessoryBorrowed => Some(self.serialize_to_transaction()),
            ResourceState::AccessoryHas => None,
            ResourceState::ControllerHas | ResourceState::ControllerBorrowed | ResourceState::Invalid => None,
        }
    }

    pub fn serialize_to_permanent_entity(&self) -> Option<ResourcePermanentEntity> {
        match self.owner() {
            EntityEnum::Controller => Some(ResourcePermanentEntity::Controller),
            EntityEnum::Accessory => Some(ResourcePermanentEntity::Accessory {
                take_constraint: self.take_constraint,
                borrow_constraint: self.borrow_constraint,
            }),
            EntityEnum::None => None,
        }
    }

    pub fn process_accessory_request(&mut self, resource: &Resource) -> bool {
        match resource.transfer_type {
            ResourceTransferType::Take => self.take_by_accessory(resource),
            ResourceTransferType::Untake => self.untake_by_accessory(resource),
            ResourceTransferType::Borrow => self.borrow_by_accessory(resource),
            ResourceTransferType::Unborrow => self.unborrow_by_accessory(resource),
        }
    }

    pub fn process_controller_request(&mut self, resource: &Resource) -> bool {
        match resource.transfer_type {
            ResourceTransferType::Take => self.take_by_controller(resource),
            ResourceTransferType::Untake => self.untake_by_controller(resource),
            ResourceTransferType::Borrow => self.borrow_by_controller(resource),
            ResourceTransferType::Unborrow => self.unborrow_by_controller(resource),
        }
    }

    pub fn take_by_accessory(&mut self, resource: &Resource) -> bool {
        self.take_by(resource, false)
    }

    pub fn take_by_controller(&mut self, resource: &Resource) -> bool {
        self.take_by(resource, true)
    }

    fn take_by(&mut self, resource: &Resource, controller: bool) -> bool {
        let Some((priority, take_constraint, borrow_constraint)) = Self::take_request_fields(resource) else {
            return false;
        };

        let actor = Self::entity_for_controller_flag(controller);
        let other = Self::other_entity(actor);
        let actor_has = ResourceState::from_pair(actor, actor);
        let actor_borrowed = ResourceState::from_pair(actor, other);
        let other_borrowed = ResourceState::from_pair(other, actor);

        if self.state == other_borrowed && !Self::priority_satisfies(priority, self.unborrow_constraint) {
            return false;
        }

        if matches!(self.state, state if state != actor_has && state != other_borrowed)
            && (self.state == actor_borrowed || self.owner() == other || self.state.is_unclaimed())
            && !Self::priority_satisfies(priority, self.take_constraint)
        {
            return false;
        }

        *self = Self::new(actor_has, take_constraint, borrow_constraint);
        true
    }

    pub fn untake_by_accessory(&mut self, resource: &Resource) -> bool {
        self.untake_by(resource, false)
    }

    pub fn untake_by_controller(&mut self, resource: &Resource) -> bool {
        self.untake_by(resource, true)
    }

    fn untake_by(&mut self, resource: &Resource, controller: bool) -> bool {
        if !Self::release_request_fields_are_empty(resource) {
            return false;
        }

        if self.owner() == Self::entity_for_controller_flag(controller) {
            *self = Self::unclaimed();
        }

        true
    }

    pub fn borrow_by_accessory(&mut self, resource: &Resource) -> bool {
        self.borrow_by(resource, false)
    }

    pub fn borrow_by_controller(&mut self, resource: &Resource) -> bool {
        self.borrow_by(resource, true)
    }

    fn borrow_by(&mut self, resource: &Resource, controller: bool) -> bool {
        let Some(priority) = Self::borrow_request_priority(resource) else {
            return false;
        };

        let actor = Self::entity_for_controller_flag(controller);
        let other = Self::other_entity(actor);
        let actor_has = ResourceState::from_pair(actor, actor);
        let actor_borrowed = ResourceState::from_pair(actor, other);
        let other_borrowed = ResourceState::from_pair(other, actor);

        if self.state == actor_borrowed {
            self.borrows_count = self.borrows_count.saturating_add(1);
            return true;
        }

        if self.state == actor_has || self.state == other_borrowed {
            return false;
        }

        if self.owner() == other || self.state.is_unclaimed() {
            if !self.can_borrow(priority) {
                return false;
            }
            self.unborrow_constraint = resource
                .unborrow_constraint
                .unwrap_or(ResourceConstraint::Anytime);
            self.state = actor_borrowed;
            self.borrows_count = 1;
            return true;
        }

        false
    }

    pub fn unborrow_by_accessory(&mut self, resource: &Resource) -> bool {
        self.unborrow_by(resource, false)
    }

    pub fn unborrow_by_controller(&mut self, resource: &Resource) -> bool {
        self.unborrow_by(resource, true)
    }

    fn unborrow_by(&mut self, resource: &Resource, controller: bool) -> bool {
        if !Self::release_request_fields_are_empty(resource) {
            return false;
        }

        let actor = Self::entity_for_controller_flag(controller);
        let other = Self::other_entity(actor);
        let actor_borrowed = ResourceState::from_pair(actor, other);
        let other_has = ResourceState::from_pair(other, other);
        let other_borrowed = ResourceState::from_pair(other, actor);

        if self.state == actor_borrowed {
            self.borrows_count = self.borrows_count.saturating_sub(1);
            if self.borrows_count == 0 {
                self.state = other_has;
                self.unborrow_constraint = ResourceConstraint::Anytime;
            }
            return true;
        }

        if self.state == other_borrowed {
            return false;
        }

        warn!("Requested Unborrow by {actor:?} but actor is not borrowing at: {self:?}");
        true
    }

    fn entity_for_controller_flag(controller: bool) -> EntityEnum {
        if controller {
            EntityEnum::Controller
        } else {
            EntityEnum::Accessory
        }
    }

    fn other_entity(entity: EntityEnum) -> EntityEnum {
        match entity {
            EntityEnum::Controller => EntityEnum::Accessory,
            EntityEnum::Accessory => EntityEnum::Controller,
            EntityEnum::None => EntityEnum::None,
        }
    }

    fn take_request_fields(resource: &Resource) -> Option<(ResourceTransferPriority, ResourceConstraint, ResourceConstraint)> {
        Some((resource.transfer_priority?, resource.take_constraint?, resource.borrow_constraint?))
    }

    fn borrow_request_priority(resource: &Resource) -> Option<ResourceTransferPriority> {
        if resource.take_constraint.is_some() || resource.borrow_constraint.is_some() || resource.unborrow_constraint.is_none() {
            return None;
        }

        resource.transfer_priority
    }

    fn release_request_fields_are_empty(resource: &Resource) -> bool {
        resource.transfer_priority.is_none()
            && resource.take_constraint.is_none()
            && resource.borrow_constraint.is_none()
            && resource.unborrow_constraint.is_none()
    }

    fn priority_satisfies(priority: ResourceTransferPriority, constraint: ResourceConstraint) -> bool {
        match constraint {
            ResourceConstraint::Anytime => true,
            ResourceConstraint::UserInitiated => matches!(priority, ResourceTransferPriority::UserInitiated),
            ResourceConstraint::Never => false,
        }
    }
}

#[cfg(test)]
#[path = "resource_manager_tests.rs"]
mod resource_manager_tests;
