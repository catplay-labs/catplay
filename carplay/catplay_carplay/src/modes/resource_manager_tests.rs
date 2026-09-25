use crate::modes::{AppState, InitialPermanentEntity};

use super::*;

#[test]
fn user_initiated_satisfies_anytime_and_user_initiated_constraints() {
    let resource = ResourceManager::accessory_owned(ResourceConstraint::UserInitiated, ResourceConstraint::Anytime);

    assert!(resource.can_take(ResourceTransferPriority::UserInitiated));
    assert!(resource.can_borrow(ResourceTransferPriority::NiceToHave));
}

#[test]
fn nice_to_have_does_not_satisfy_user_initiated_or_never_constraints() {
    let resource = ResourceManager::accessory_owned(ResourceConstraint::UserInitiated, ResourceConstraint::Never);

    assert!(!resource.can_take(ResourceTransferPriority::NiceToHave));
    assert!(!resource.can_borrow(ResourceTransferPriority::UserInitiated));
}

#[test]
fn can_take_uses_unborrow_constraint_while_resource_is_borrowed() {
    let resource = ResourceManager::new(
        ResourceState::AccessoryBorrowed,
        ResourceConstraint::Anytime,
        ResourceConstraint::Anytime,
    )
    .with_unborrow_constraint(ResourceConstraint::UserInitiated);

    assert!(resource.can_take(ResourceTransferPriority::UserInitiated));
    assert!(!resource.can_take(ResourceTransferPriority::NiceToHave));
}

#[test]
fn owner_is_permanent_entity_while_entity_is_current_user() {
    let resource = ResourceManager::new(
        ResourceState::AccessoryBorrowed,
        ResourceConstraint::Anytime,
        ResourceConstraint::Anytime,
    );

    assert!(resource.is_borrowed());
    assert_eq!(resource.owner(), EntityEnum::Controller);
    assert_eq!(resource.entity(), EntityEnum::Accessory);
}

#[test]
fn controller_defaults_to_controller_owned_resources_and_no_app_states() {
    let controller = ResourceController::default();

    assert_eq!(controller.screen.owner(), EntityEnum::Controller);
    assert_eq!(controller.main_audio.owner(), EntityEnum::Controller);
    assert_eq!(controller.phone_call, EntityEnum::None);
    assert_eq!(controller.speech, SpeechState::default());
    assert_eq!(controller.turn_by_turn, EntityEnum::None);
}

#[test]
fn from_resource_maps_take_to_owned_state_and_constraints() {
    let resource = Resource::take(
        crate::modes::ResourceID::MainAudio,
        ResourceTransferPriority::UserInitiated,
        ResourceConstraint::UserInitiated,
        ResourceConstraint::Never,
    );

    let manager = ResourceManager::from_resource(&resource);

    assert_eq!(manager.state(), ResourceState::AccessoryHas);
    assert!(manager.can_take(ResourceTransferPriority::UserInitiated));
    assert!(!manager.can_take(ResourceTransferPriority::NiceToHave));
    assert!(!manager.can_borrow(ResourceTransferPriority::UserInitiated));
}

#[test]
fn controller_serializes_to_modes_changed() {
    let controller = ResourceController {
        screen: ResourceManager::new(
            ResourceState::AccessoryBorrowed,
            ResourceConstraint::Anytime,
            ResourceConstraint::Anytime,
        ),
        main_audio: ResourceManager::accessory_owned(ResourceConstraint::UserInitiated, ResourceConstraint::Never),
        phone_call: EntityEnum::Accessory,
        speech: SpeechState {
            entity: EntityEnum::Controller,
            mode: SpeechMode::Speaking,
        },
        turn_by_turn: EntityEnum::None,
    };

    let modes = controller.serialize_to_state().serialize();

    assert_eq!(modes.resources.len(), 2);
    assert_eq!(modes.resources[0].resource_id, ResourceID::MainScreen);
    assert_eq!(modes.resources[0].entity, EntityEnum::Accessory);
    assert_eq!(modes.resources[0].permanent_entity, EntityEnum::Controller);
    assert_eq!(modes.resources[1].resource_id, ResourceID::MainAudio);
    assert_eq!(modes.resources[1].entity, EntityEnum::Accessory);
    assert_eq!(modes.resources[1].permanent_entity, EntityEnum::Accessory);
    assert_eq!(modes.app_states.len(), 3);
}

#[test]
fn controller_serializes_to_change_modes_request() {
    let controller = ResourceController {
        screen: ResourceManager::accessory_owned(ResourceConstraint::UserInitiated, ResourceConstraint::Never),
        main_audio: ResourceManager::new(
            ResourceState::AccessoryBorrowed,
            ResourceConstraint::Anytime,
            ResourceConstraint::Anytime,
        ),
        phone_call: EntityEnum::Accessory,
        speech: SpeechState {
            entity: EntityEnum::Controller,
            mode: SpeechMode::Speaking,
        },
        turn_by_turn: EntityEnum::None,
    };

    assert_eq!(
        controller.serialize_to_request(),
        ChangeModes {
            app_states: vec![
                AppState::new(AppStateEnum::PhoneCall, true),
                AppState::new(AppStateEnum::TurnByTurn, false),
                AppState::speech(SpeechMode::None),
            ],
            resources: vec![
                Resource::take(
                    ResourceID::MainScreen,
                    ResourceTransferPriority::UserInitiated,
                    ResourceConstraint::UserInitiated,
                    ResourceConstraint::Never,
                ),
                Resource::borrow(
                    ResourceID::MainAudio,
                    ResourceTransferPriority::UserInitiated,
                    ResourceConstraint::Anytime,
                ),
            ],
            reason_str: String::new(),
            initial_permanent_entity: vec![],
        }
    );
}

#[test]
fn controller_serializes_info_request_with_initial_permanent_entities() {
    let controller = ResourceController {
        screen: ResourceManager::controller_owned(),
        main_audio: ResourceManager::accessory_owned(ResourceConstraint::Anytime, ResourceConstraint::Never),
        ..ResourceController::default()
    };

    assert_eq!(
        controller.serialize_to_info_request(),
        ChangeModes {
            app_states: vec![
                AppState::new(AppStateEnum::PhoneCall, false),
                AppState::new(AppStateEnum::TurnByTurn, false),
                AppState::speech(SpeechMode::None),
            ],
            resources: vec![],
            reason_str: String::new(),
            initial_permanent_entity: vec![
                InitialPermanentEntity::controller(ResourceID::MainScreen),
                InitialPermanentEntity::accessory(ResourceID::MainAudio, ResourceConstraint::Anytime, ResourceConstraint::Never,),
            ],
        }
    );
}

#[test]
fn controller_serializes_to_airplay_mode_state() {
    let controller = ResourceController {
        screen: ResourceManager::accessory_owned(ResourceConstraint::UserInitiated, ResourceConstraint::Never),
        main_audio: ResourceManager::new(
            ResourceState::AccessoryBorrowed,
            ResourceConstraint::Anytime,
            ResourceConstraint::Anytime,
        ),
        phone_call: EntityEnum::Accessory,
        speech: SpeechState {
            entity: EntityEnum::Accessory,
            mode: SpeechMode::RecognizingSpeech,
        },
        turn_by_turn: EntityEnum::Controller,
    };

    let state = controller.serialize_to_state();

    assert_eq!(state.screen, ResourceState::AccessoryHas);
    assert_eq!(state.main_audio, ResourceState::AccessoryBorrowed);
    assert_eq!(state.phone_call, EntityEnum::Accessory);
    assert_eq!(state.speech.entity, EntityEnum::Accessory);
    assert_eq!(state.speech.mode, SpeechMode::RecognizingSpeech);
    assert_eq!(state.turn_by_turn, EntityEnum::Controller);
}

#[test]
fn controller_from_change_modes_preserves_initial_resources_and_app_states() {
    let modes = ChangeModes::initial();

    let controller = ResourceController::from_change_modes(&modes);

    assert_eq!(controller.screen.state(), ResourceState::AccessoryHas);
    assert_eq!(controller.main_audio.state(), ResourceState::AccessoryHas);
    assert_eq!(controller.phone_call, EntityEnum::None);
    assert_eq!(controller.turn_by_turn, EntityEnum::None);
    assert_eq!(controller.speech.entity, EntityEnum::None);
    assert_eq!(controller.speech.mode, SpeechMode::None);
}

#[test]
fn controller_from_change_modes_uses_initial_permanent_entity_as_base_state() {
    let modes = ChangeModes {
        app_states: vec![],
        resources: vec![],
        reason_str: "test".into(),
        initial_permanent_entity: vec![
            InitialPermanentEntity::accessory(ResourceID::MainScreen, ResourceConstraint::UserInitiated, ResourceConstraint::Never),
            InitialPermanentEntity::controller(ResourceID::MainAudio),
        ],
    };

    let controller = ResourceController::from_change_modes(&modes);

    assert_eq!(controller.screen.state(), ResourceState::AccessoryHas);
    assert!(
        controller
            .screen
            .can_take(ResourceTransferPriority::UserInitiated)
    );
    assert!(
        !controller
            .screen
            .can_take(ResourceTransferPriority::NiceToHave)
    );
    assert!(
        !controller
            .screen
            .can_borrow(ResourceTransferPriority::UserInitiated)
    );
    assert_eq!(controller.main_audio.state(), ResourceState::ControllerHas);
}

#[test]
fn controller_from_change_modes_applies_resources_over_initial_permanent_entity() {
    let modes = ChangeModes {
        app_states: vec![],
        resources: vec![
            Resource::borrow(
                ResourceID::MainAudio,
                ResourceTransferPriority::NiceToHave,
                ResourceConstraint::Anytime,
            ),
            Resource::take(
                ResourceID::MainScreen,
                ResourceTransferPriority::UserInitiated,
                ResourceConstraint::Never,
                ResourceConstraint::UserInitiated,
            ),
        ],
        reason_str: "test".into(),
        initial_permanent_entity: vec![
            InitialPermanentEntity::controller(ResourceID::MainAudio),
            InitialPermanentEntity::controller(ResourceID::MainScreen),
        ],
    };

    let controller = ResourceController::from_change_modes(&modes);

    assert_eq!(controller.main_audio.state(), ResourceState::AccessoryBorrowed);
    assert_eq!(controller.main_audio.borrows_count, 1);
    assert_eq!(controller.screen.state(), ResourceState::AccessoryHas);
    assert!(
        !controller
            .screen
            .can_take(ResourceTransferPriority::UserInitiated)
    );
    assert!(
        controller
            .screen
            .can_borrow(ResourceTransferPriority::UserInitiated)
    );
    assert!(
        !controller
            .screen
            .can_borrow(ResourceTransferPriority::NiceToHave)
    );
}

#[test]
fn resources_only_cannot_express_controller_borrowed_state() {
    let resource_variants = [
        Resource::take(
            ResourceID::MainScreen,
            ResourceTransferPriority::UserInitiated,
            ResourceConstraint::Anytime,
            ResourceConstraint::Anytime,
        ),
        Resource::borrow(
            ResourceID::MainScreen,
            ResourceTransferPriority::UserInitiated,
            ResourceConstraint::Anytime,
        ),
        Resource::untake(ResourceID::MainScreen),
        Resource::unborrow(ResourceID::MainScreen),
    ];

    for resource in resource_variants {
        let transfer_type = resource.transfer_type;
        let modes = ChangeModes {
            app_states: vec![],
            resources: vec![resource],
            reason_str: "resources-only".into(),
            initial_permanent_entity: vec![],
        };

        let controller = ResourceController::from_change_modes(&modes);

        println!("{:?}", controller.screen);
        assert_ne!(
            controller.screen.state(),
            ResourceState::ControllerBorrowed,
            "resources-only initial mode unexpectedly expressed ControllerBorrowed via {:?}",
            transfer_type
        );
    }
}

#[test]
fn resources_only_borrow_auto_claims_unclaimed_resource() {
    let modes = ChangeModes {
        app_states: vec![],
        resources: vec![Resource::borrow(
            ResourceID::MainScreen,
            ResourceTransferPriority::UserInitiated,
            ResourceConstraint::Never,
        )],
        reason_str: "resources-only borrow".into(),
        initial_permanent_entity: vec![],
    };

    let controller = ResourceController::from_change_modes(&modes);

    assert_eq!(controller.screen.state(), ResourceState::AccessoryBorrowed);
    assert_eq!(controller.screen.owner(), EntityEnum::Controller);
    assert_eq!(controller.screen.unborrow_constraint(), ResourceConstraint::Never);
    assert_eq!(controller.screen.borrows_count, 1);
}

#[test]
fn initial_import_treats_release_transactions_as_noop() {
    for resource in [Resource::untake(ResourceID::MainScreen), Resource::unborrow(ResourceID::MainScreen)] {
        let modes = ChangeModes {
            app_states: vec![],
            resources: vec![resource],
            reason_str: "release transaction".into(),
            initial_permanent_entity: vec![InitialPermanentEntity::accessory(
                ResourceID::MainScreen,
                ResourceConstraint::UserInitiated,
                ResourceConstraint::Never,
            )],
        };

        let controller = ResourceController::from_change_modes(&modes);

        assert_eq!(controller.screen.state(), ResourceState::AccessoryHas);
        assert!(
            controller
                .screen
                .can_take(ResourceTransferPriority::UserInitiated)
        );
        assert!(
            !controller
                .screen
                .can_borrow(ResourceTransferPriority::UserInitiated)
        );
    }
}

#[test]
fn initial_permanent_entity_supplies_base_owner_needed_for_controller_borrowed_state() {
    let modes = ChangeModes {
        app_states: vec![],
        resources: vec![],
        reason_str: "initial accessory owner".into(),
        initial_permanent_entity: vec![crate::modes::InitialPermanentEntity::accessory(
            ResourceID::MainScreen,
            ResourceConstraint::UserInitiated,
            ResourceConstraint::Anytime,
        )],
    };
    let mut controller = ResourceController::from_change_modes(&modes);

    assert_eq!(controller.screen.state(), ResourceState::AccessoryHas);
    assert!(controller.screen.borrow_by_controller(&Resource::borrow(
        ResourceID::MainScreen,
        ResourceTransferPriority::UserInitiated,
        ResourceConstraint::Anytime,
    )));
    assert_eq!(controller.screen.state(), ResourceState::ControllerBorrowed);
}

#[derive(Clone, Copy, Debug)]
enum Actor {
    Accessory,
    Controller,
}

#[derive(Clone, Copy, Debug)]
enum Operation {
    TakeNiceAny,
    TakeUserAny,
    TakeUserStrict,
    BorrowNice,
    Untake,
    Unborrow,
}

#[derive(Clone, Copy, Debug)]
struct Transition {
    name: &'static str,
    actor: Actor,
    input_state: ResourceState,
    input_take: ResourceConstraint,
    input_borrow: ResourceConstraint,
    input_unborrow: ResourceConstraint,
    input_count: usize,
    operation: Operation,
    accepted: bool,
    output_state: ResourceState,
    output_count: usize,
    output_take: ResourceConstraint,
    output_borrow: ResourceConstraint,
    output_unborrow: ResourceConstraint,
}

impl Transition {
    fn new(
        name: &'static str,
        actor: Actor,
        input_state: ResourceState,
        operation: Operation,
        accepted: bool,
        output_state: ResourceState,
    ) -> Self {
        Self {
            name,
            actor,
            input_state,
            input_take: ResourceConstraint::Anytime,
            input_borrow: ResourceConstraint::Anytime,
            input_unborrow: ResourceConstraint::Anytime,
            input_count: 0,
            operation,
            accepted,
            output_state,
            output_count: 0,
            output_take: ResourceConstraint::Anytime,
            output_borrow: ResourceConstraint::Anytime,
            output_unborrow: ResourceConstraint::Anytime,
        }
    }

    fn with_input_constraints(mut self, take: ResourceConstraint, borrow: ResourceConstraint) -> Self {
        self.input_take = take;
        self.input_borrow = borrow;
        self
    }

    fn with_input_count(mut self, count: usize) -> Self {
        self.input_count = count;
        self
    }

    fn with_input_unborrow_constraint(mut self, unborrow: ResourceConstraint) -> Self {
        self.input_unborrow = unborrow;
        self
    }

    fn with_output_constraints(mut self, take: ResourceConstraint, borrow: ResourceConstraint) -> Self {
        self.output_take = take;
        self.output_borrow = borrow;
        self
    }

    fn with_output_unborrow_constraint(mut self, unborrow: ResourceConstraint) -> Self {
        self.output_unborrow = unborrow;
        self
    }

    fn with_output_count(mut self, count: usize) -> Self {
        self.output_count = count;
        self
    }
}

fn transition_resource(operation: Operation) -> Resource {
    match operation {
        Operation::TakeNiceAny => Resource::take(
            ResourceID::MainAudio,
            ResourceTransferPriority::NiceToHave,
            ResourceConstraint::Anytime,
            ResourceConstraint::Anytime,
        ),
        Operation::TakeUserAny => Resource::take(
            ResourceID::MainAudio,
            ResourceTransferPriority::UserInitiated,
            ResourceConstraint::Anytime,
            ResourceConstraint::Anytime,
        ),
        Operation::TakeUserStrict => Resource::take(
            ResourceID::MainAudio,
            ResourceTransferPriority::UserInitiated,
            ResourceConstraint::Never,
            ResourceConstraint::UserInitiated,
        ),
        Operation::BorrowNice => Resource::borrow(
            ResourceID::MainAudio,
            ResourceTransferPriority::NiceToHave,
            ResourceConstraint::Anytime,
        ),
        Operation::Untake => Resource::untake(ResourceID::MainAudio),
        Operation::Unborrow => Resource::unborrow(ResourceID::MainAudio),
    }
}

fn apply_transition(resource: &mut ResourceManager, actor: Actor, operation: Operation) -> bool {
    let request = transition_resource(operation);
    match (actor, operation) {
        (Actor::Accessory, Operation::TakeNiceAny | Operation::TakeUserAny | Operation::TakeUserStrict) => {
            resource.take_by_accessory(&request)
        }
        (Actor::Accessory, Operation::BorrowNice) => resource.borrow_by_accessory(&request),
        (Actor::Accessory, Operation::Untake) => resource.untake_by_accessory(&request),
        (Actor::Accessory, Operation::Unborrow) => resource.unborrow_by_accessory(&request),
        (Actor::Controller, Operation::TakeNiceAny | Operation::TakeUserAny | Operation::TakeUserStrict) => {
            resource.take_by_controller(&request)
        }
        (Actor::Controller, Operation::BorrowNice) => resource.borrow_by_controller(&request),
        (Actor::Controller, Operation::Untake) => resource.untake_by_controller(&request),
        (Actor::Controller, Operation::Unborrow) => resource.unborrow_by_controller(&request),
    }
}

#[test]
fn resource_manager_transition_table() {
    let transitions = [
        Transition::new(
            "accessory takes controller-owned resource",
            Actor::Accessory,
            ResourceState::ControllerHas,
            Operation::TakeUserStrict,
            true,
            ResourceState::AccessoryHas,
        )
        .with_output_constraints(ResourceConstraint::Never, ResourceConstraint::UserInitiated),
        Transition::new(
            "accessory takes unclaimed resource",
            Actor::Accessory,
            ResourceState::Invalid,
            Operation::TakeUserStrict,
            true,
            ResourceState::AccessoryHas,
        )
        .with_output_constraints(ResourceConstraint::Never, ResourceConstraint::UserInitiated),
        Transition::new(
            "accessory take blocked by controller take constraint",
            Actor::Accessory,
            ResourceState::ControllerHas,
            Operation::TakeUserAny,
            false,
            ResourceState::ControllerHas,
        )
        .with_input_constraints(ResourceConstraint::Never, ResourceConstraint::Anytime)
        .with_output_constraints(ResourceConstraint::Never, ResourceConstraint::Anytime),
        Transition::new(
            "accessory borrow-to-take upgrade blocked by owner take constraint",
            Actor::Accessory,
            ResourceState::AccessoryBorrowed,
            Operation::TakeUserAny,
            false,
            ResourceState::AccessoryBorrowed,
        )
        .with_input_constraints(ResourceConstraint::Never, ResourceConstraint::Anytime)
        .with_input_count(1)
        .with_output_constraints(ResourceConstraint::Never, ResourceConstraint::Anytime)
        .with_output_count(1),
        Transition::new(
            "accessory cannot borrow when already owner",
            Actor::Accessory,
            ResourceState::AccessoryHas,
            Operation::BorrowNice,
            false,
            ResourceState::AccessoryHas,
        ),
        Transition::new(
            "accessory borrows controller-owned resource",
            Actor::Accessory,
            ResourceState::ControllerHas,
            Operation::BorrowNice,
            true,
            ResourceState::AccessoryBorrowed,
        )
        .with_output_count(1),
        Transition::new(
            "accessory borrows unclaimed resource",
            Actor::Accessory,
            ResourceState::Invalid,
            Operation::BorrowNice,
            true,
            ResourceState::AccessoryBorrowed,
        )
        .with_output_count(1),
        Transition::new(
            "accessory repeated borrow increments count",
            Actor::Accessory,
            ResourceState::AccessoryBorrowed,
            Operation::BorrowNice,
            true,
            ResourceState::AccessoryBorrowed,
        )
        .with_input_count(1)
        .with_output_count(2),
        Transition::new(
            "accessory unborrow decrements count without releasing",
            Actor::Accessory,
            ResourceState::AccessoryBorrowed,
            Operation::Unborrow,
            true,
            ResourceState::AccessoryBorrowed,
        )
        .with_input_count(2)
        .with_output_count(1),
        Transition::new(
            "accessory final unborrow returns to controller",
            Actor::Accessory,
            ResourceState::AccessoryBorrowed,
            Operation::Unborrow,
            true,
            ResourceState::ControllerHas,
        )
        .with_input_count(1),
        Transition::new(
            "controller take blocked by accessory unborrow constraint",
            Actor::Controller,
            ResourceState::AccessoryBorrowed,
            Operation::TakeUserAny,
            false,
            ResourceState::AccessoryBorrowed,
        )
        .with_input_count(1)
        .with_input_unborrow_constraint(ResourceConstraint::Never)
        .with_output_count(1)
        .with_output_unborrow_constraint(ResourceConstraint::Never),
        Transition::new(
            "controller take terminates accessory borrow when unborrow constraint allows",
            Actor::Controller,
            ResourceState::AccessoryBorrowed,
            Operation::TakeUserStrict,
            true,
            ResourceState::ControllerHas,
        )
        .with_input_count(1)
        .with_input_unborrow_constraint(ResourceConstraint::UserInitiated)
        .with_output_constraints(ResourceConstraint::Never, ResourceConstraint::UserInitiated),
        Transition::new(
            "accessory untake releases owned resource",
            Actor::Accessory,
            ResourceState::AccessoryHas,
            Operation::Untake,
            true,
            ResourceState::Invalid,
        ),
        Transition::new(
            "controller borrows accessory-owned resource",
            Actor::Controller,
            ResourceState::AccessoryHas,
            Operation::BorrowNice,
            true,
            ResourceState::ControllerBorrowed,
        )
        .with_output_count(1),
        Transition::new(
            "controller repeated borrow increments count",
            Actor::Controller,
            ResourceState::ControllerBorrowed,
            Operation::BorrowNice,
            true,
            ResourceState::ControllerBorrowed,
        )
        .with_input_count(1)
        .with_output_count(2),
        Transition::new(
            "controller final unborrow returns to accessory",
            Actor::Controller,
            ResourceState::ControllerBorrowed,
            Operation::Unborrow,
            true,
            ResourceState::AccessoryHas,
        )
        .with_input_count(1),
        Transition::new(
            "controller take blocked by accessory take constraint",
            Actor::Controller,
            ResourceState::AccessoryHas,
            Operation::TakeNiceAny,
            false,
            ResourceState::AccessoryHas,
        )
        .with_input_constraints(ResourceConstraint::UserInitiated, ResourceConstraint::Anytime)
        .with_output_constraints(ResourceConstraint::UserInitiated, ResourceConstraint::Anytime),
        Transition::new(
            "controller take accepts user priority and stores new constraints",
            Actor::Controller,
            ResourceState::AccessoryHas,
            Operation::TakeUserStrict,
            true,
            ResourceState::ControllerHas,
        )
        .with_input_constraints(ResourceConstraint::UserInitiated, ResourceConstraint::Anytime)
        .with_output_constraints(ResourceConstraint::Never, ResourceConstraint::UserInitiated),
        Transition::new(
            "controller takes unclaimed resource",
            Actor::Controller,
            ResourceState::Invalid,
            Operation::TakeUserStrict,
            true,
            ResourceState::ControllerHas,
        )
        .with_output_constraints(ResourceConstraint::Never, ResourceConstraint::UserInitiated),
        Transition::new(
            "controller cannot borrow when already owner",
            Actor::Controller,
            ResourceState::ControllerHas,
            Operation::BorrowNice,
            false,
            ResourceState::ControllerHas,
        ),
        Transition::new(
            "controller borrows unclaimed resource",
            Actor::Controller,
            ResourceState::Invalid,
            Operation::BorrowNice,
            true,
            ResourceState::ControllerBorrowed,
        )
        .with_output_count(1),
        Transition::new(
            "controller untake releases owned resource",
            Actor::Controller,
            ResourceState::ControllerHas,
            Operation::Untake,
            true,
            ResourceState::Invalid,
        ),
    ];

    for transition in transitions {
        let mut resource = ResourceManager::new(transition.input_state, transition.input_take, transition.input_borrow);
        resource.borrows_count = transition.input_count;
        resource.unborrow_constraint = transition.input_unborrow;

        let accepted = apply_transition(&mut resource, transition.actor, transition.operation);

        assert_eq!(accepted, transition.accepted, "{}", transition.name);
        assert_eq!(resource.state(), transition.output_state, "{}", transition.name);
        assert_eq!(resource.borrows_count, transition.output_count, "{}", transition.name);
        assert_eq!(resource.take_constraint, transition.output_take, "{}", transition.name);
        assert_eq!(resource.borrow_constraint, transition.output_borrow, "{}", transition.name);
        assert_eq!(resource.unborrow_constraint, transition.output_unborrow, "{}", transition.name);
    }
}
