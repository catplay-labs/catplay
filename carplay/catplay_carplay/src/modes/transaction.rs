use crate::modes::{
    AppState, AppStateEnum, ChangeModes, EntityEnum, InitialPermanentEntity, Resource, ResourceConstraint, ResourceID,
    ResourceTransferPriority, ResourceTransferType, SpeechMode, SpeechState,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AirPlayModeTransaction {
    pub screen: Option<ResourceTransaction>,
    pub screen_perm: Option<ResourcePermanentEntity>,
    pub main_audio: Option<ResourceTransaction>,
    pub main_audio_perm: Option<ResourcePermanentEntity>,
    pub phone_call: Option<EntityEnum>,
    pub speech: Option<SpeechState>,
    pub turn_by_turn: Option<EntityEnum>,
    pub reason: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceTransaction {
    Take {
        priority: ResourceTransferPriority,
        take_constraint: ResourceConstraint,
        borrow_constraint: ResourceConstraint,
    },
    Untake,
    Borrow {
        priority: ResourceTransferPriority,
        unborrow_constraint: ResourceConstraint,
    },
    Unborrow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourcePermanentEntity {
    Controller,
    Accessory {
        take_constraint: ResourceConstraint,
        borrow_constraint: ResourceConstraint,
    },
}

impl AirPlayModeTransaction {
    pub fn serialize_to_change_modes(&self) -> ChangeModes {
        let mut resources = Vec::with_capacity(2);

        if let Some(transaction) = self.screen {
            resources.push(transaction.serialize_to_resource(ResourceID::MainScreen));
        }

        if let Some(transaction) = self.main_audio {
            resources.push(transaction.serialize_to_resource(ResourceID::MainAudio));
        }

        ChangeModes {
            app_states: self.serialize_app_states(),
            resources,
            reason_str: self.reason.clone(),
            initial_permanent_entity: vec![],
        }
    }

    pub fn serialize_to_info_change_modes(&self) -> ChangeModes {
        let mut modes = self.serialize_to_change_modes();

        if let Some(entity) = self.screen_perm {
            modes
                .initial_permanent_entity
                .push(serialize_initial_permanent_entity(ResourceID::MainScreen, entity));
        }

        if let Some(entity) = self.main_audio_perm {
            modes
                .initial_permanent_entity
                .push(serialize_initial_permanent_entity(ResourceID::MainAudio, entity));
        }

        modes
    }

    fn serialize_app_states(&self) -> Vec<AppState> {
        let mut app_states = Vec::with_capacity(3);

        if let Some(entity) = self.phone_call {
            app_states.push(AppState::new(AppStateEnum::PhoneCall, entity == EntityEnum::Accessory));
        }

        if let Some(entity) = self.turn_by_turn {
            app_states.push(AppState::new(AppStateEnum::TurnByTurn, entity == EntityEnum::Accessory));
        }

        if let Some(speech) = self.speech {
            app_states.push(AppState::speech(if speech.entity == EntityEnum::Accessory {
                speech.mode
            } else {
                SpeechMode::None
            }));
        }

        app_states
    }

    fn set_resource(&mut self, resource_id: ResourceID, transaction: ResourceTransaction) {
        match resource_id {
            ResourceID::MainScreen => self.screen = Some(transaction),
            ResourceID::MainAudio => self.main_audio = Some(transaction),
        }
    }

    fn set_initial_permanent_entity(&mut self, initial: &InitialPermanentEntity) {
        let permanent_entity = ResourcePermanentEntity::from_initial_permanent_entity(initial);

        match initial.resource_id {
            ResourceID::MainScreen => self.screen_perm = permanent_entity,
            ResourceID::MainAudio => self.main_audio_perm = permanent_entity,
        }
    }
}

fn serialize_initial_permanent_entity(resource_id: ResourceID, entity: ResourcePermanentEntity) -> InitialPermanentEntity {
    match entity {
        ResourcePermanentEntity::Accessory {
            take_constraint,
            borrow_constraint,
        } => InitialPermanentEntity::accessory(resource_id, take_constraint, borrow_constraint),
        ResourcePermanentEntity::Controller => InitialPermanentEntity::controller(resource_id),
    }
}

impl ResourcePermanentEntity {
    fn from_initial_permanent_entity(initial: &InitialPermanentEntity) -> Option<Self> {
        match initial.permanent_entity {
            EntityEnum::Accessory => Some(ResourcePermanentEntity::Accessory {
                take_constraint: initial
                    .take_constraint
                    .unwrap_or(ResourceConstraint::Anytime),
                borrow_constraint: initial
                    .borrow_constraint
                    .unwrap_or(ResourceConstraint::Anytime),
            }),
            EntityEnum::Controller => Some(ResourcePermanentEntity::Controller),
            EntityEnum::None => None,
        }
    }
}

impl ResourceTransaction {
    fn serialize_to_resource(self, resource_id: ResourceID) -> Resource {
        match self {
            ResourceTransaction::Take {
                priority,
                take_constraint,
                borrow_constraint,
            } => Resource::take(resource_id, priority, take_constraint, borrow_constraint),
            ResourceTransaction::Untake => Resource::untake(resource_id),
            ResourceTransaction::Borrow {
                priority,
                unborrow_constraint,
            } => Resource::borrow(resource_id, priority, unborrow_constraint),
            ResourceTransaction::Unborrow => Resource::unborrow(resource_id),
        }
    }

    fn from_resource(resource: &Resource) -> Self {
        match resource.transfer_type {
            ResourceTransferType::Take => ResourceTransaction::Take {
                priority: resource
                    .transfer_priority
                    .unwrap_or(ResourceTransferPriority::NiceToHave),
                take_constraint: resource
                    .take_constraint
                    .unwrap_or(ResourceConstraint::Anytime),
                borrow_constraint: resource
                    .borrow_constraint
                    .unwrap_or(ResourceConstraint::Anytime),
            },
            ResourceTransferType::Untake => ResourceTransaction::Untake,
            ResourceTransferType::Borrow => ResourceTransaction::Borrow {
                priority: resource
                    .transfer_priority
                    .unwrap_or(ResourceTransferPriority::NiceToHave),
                unborrow_constraint: resource
                    .unborrow_constraint
                    .unwrap_or(ResourceConstraint::Anytime),
            },
            ResourceTransferType::Unborrow => ResourceTransaction::Unborrow,
        }
    }
}

impl From<&ChangeModes> for AirPlayModeTransaction {
    fn from(modes: &ChangeModes) -> Self {
        let mut transaction = AirPlayModeTransaction {
            reason: modes.reason_str.clone(),
            ..Self::default()
        };

        for initial in &modes.initial_permanent_entity {
            transaction.set_initial_permanent_entity(initial);
        }

        for resource in &modes.resources {
            transaction.set_resource(resource.resource_id, ResourceTransaction::from_resource(resource));
        }

        for app_state in &modes.app_states {
            match app_state.app_state_id {
                AppStateEnum::Speech => {
                    let mode = app_state.speech_mode.unwrap_or(SpeechMode::None);
                    transaction.speech = Some(SpeechState {
                        entity: if mode == SpeechMode::None {
                            EntityEnum::None
                        } else {
                            EntityEnum::Accessory
                        },
                        mode,
                    });
                }
                AppStateEnum::PhoneCall => {
                    transaction.phone_call = Some(if app_state.state.unwrap_or(false) {
                        EntityEnum::Accessory
                    } else {
                        EntityEnum::None
                    });
                }
                AppStateEnum::TurnByTurn => {
                    transaction.turn_by_turn = Some(if app_state.state.unwrap_or(false) {
                        EntityEnum::Accessory
                    } else {
                        EntityEnum::None
                    });
                }
                AppStateEnum::Invalid => {}
            }
        }

        transaction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn info_serialization_keeps_resource_transactions_standard_and_adds_explicit_initial() {
        let transaction = AirPlayModeTransaction {
            screen: Some(ResourceTransaction::Untake),
            screen_perm: Some(ResourcePermanentEntity::Controller),
            main_audio: Some(ResourceTransaction::Take {
                priority: ResourceTransferPriority::UserInitiated,
                take_constraint: ResourceConstraint::Anytime,
                borrow_constraint: ResourceConstraint::Never,
            }),
            main_audio_perm: Some(ResourcePermanentEntity::Accessory {
                take_constraint: ResourceConstraint::UserInitiated,
                borrow_constraint: ResourceConstraint::Never,
            }),
            ..AirPlayModeTransaction::default()
        };

        assert_eq!(
            transaction.serialize_to_info_change_modes(),
            ChangeModes {
                app_states: vec![],
                resources: vec![
                    Resource::untake(ResourceID::MainScreen),
                    Resource::take(
                        ResourceID::MainAudio,
                        ResourceTransferPriority::UserInitiated,
                        ResourceConstraint::Anytime,
                        ResourceConstraint::Never,
                    ),
                ],
                reason_str: String::new(),
                initial_permanent_entity: vec![
                    InitialPermanentEntity::controller(ResourceID::MainScreen),
                    InitialPermanentEntity::accessory(ResourceID::MainAudio, ResourceConstraint::UserInitiated, ResourceConstraint::Never,),
                ],
            }
        );
    }

    #[test]
    fn runtime_serialization_omits_initial_permanent_entity() {
        let transaction = AirPlayModeTransaction {
            screen: Some(ResourceTransaction::Untake),
            screen_perm: Some(ResourcePermanentEntity::Controller),
            ..AirPlayModeTransaction::default()
        };

        assert_eq!(
            transaction.serialize_to_change_modes(),
            ChangeModes {
                app_states: vec![],
                resources: vec![Resource::untake(ResourceID::MainScreen)],
                reason_str: String::new(),
                initial_permanent_entity: vec![],
            }
        );
    }

    #[test]
    fn parses_initial_and_runtime_resources_from_change_modes() {
        let modes = ChangeModes {
            app_states: vec![AppState::new(AppStateEnum::PhoneCall, true)],
            resources: vec![Resource::borrow(
                ResourceID::MainScreen,
                ResourceTransferPriority::UserInitiated,
                ResourceConstraint::Never,
            )],
            reason_str: "test".into(),
            initial_permanent_entity: vec![InitialPermanentEntity::accessory(
                ResourceID::MainAudio,
                ResourceConstraint::UserInitiated,
                ResourceConstraint::Never,
            )],
        };

        assert_eq!(
            AirPlayModeTransaction::from(&modes),
            AirPlayModeTransaction {
                screen: Some(ResourceTransaction::Borrow {
                    priority: ResourceTransferPriority::UserInitiated,
                    unborrow_constraint: ResourceConstraint::Never,
                }),
                screen_perm: None,
                main_audio: None,
                main_audio_perm: Some(ResourcePermanentEntity::Accessory {
                    take_constraint: ResourceConstraint::UserInitiated,
                    borrow_constraint: ResourceConstraint::Never,
                }),
                phone_call: Some(EntityEnum::Accessory),
                speech: None,
                turn_by_turn: None,
                reason: "test".into(),
            }
        );
    }
}
