use devasign_task_escrow::DisputeResolution;
use soroban_sdk::{
    contracttype,
    testutils::{Address as _, Events, Ledger},
    Address, String, Symbol, TryIntoVal, Val,
};

mod test_config;
mod test_setup;

use test_config::{TestConfig, TestValidation};
use test_setup::create_test_env;

// Define test-only structs that mirror the contract event structs
// We add #[contracttype] to enable IntoVal conversion

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct EscrowCreatedEvent {
    pub task_id: String,
    pub creator: Address,
    pub bounty_amount: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ContributorAssignedEvent {
    pub task_id: String,
    pub contributor: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct FundsReleasedEvent {
    pub task_id: String,
    pub contributor: Address,
    pub amount: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct DisputeInitiatedEvent {
    pub task_id: String,
    pub disputing_party: Address,
    pub reason: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct DisputeResolvedEvent {
    pub task_id: String,
    pub resolution: DisputeResolution,
    pub resolved_by: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct RefundProcessedEvent {
    pub task_id: String,
    pub creator: Address,
    pub amount: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct BountyIncreasedEvent {
    pub task_id: String,
    pub creator: Address,
    pub added_amount: i128,
    pub new_total_amount: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct BountyDecreasedEvent {
    pub task_id: String,
    pub creator: Address,
    pub subtracted_amount: i128,
    pub new_total_amount: i128,
    pub timestamp: u64,
}

#[test]
fn test_escrow_created_event() {
    let (env, admin, usdc_address, usdc_token, _, contract_id, client) = create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    // Set ledger timestamp
    let timestamp = 12345;
    env.ledger().set_timestamp(timestamp);

    // Create test data
    let creator = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "event_test", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;

    // Fund creator with USDC
    usdc_token.mint(&creator, &bounty_amount);

    // Create escrow
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );

    // Get all events and filter for contract events
    let events = env.events().all();
    let contract_events: std::vec::Vec<_> = events.iter().filter(|e| e.0 == contract_id).collect();

    assert!(!contract_events.is_empty());

    let event = contract_events.last().unwrap();

    // Verify event topic (event name)
    let topics = &event.1;
    let topic_val: Val = topics.iter().next().unwrap();
    let topic_sym: Symbol = topic_val.try_into_val(&env).unwrap();
    assert_eq!(topic_sym, Symbol::new(&env, "escrow_created_event"));

    // Verify event data
    let expected_event = EscrowCreatedEvent {
        task_id,
        creator,
        bounty_amount,
        timestamp,
    };

    let actual_event: EscrowCreatedEvent = event.2.try_into_val(&env).unwrap();
    assert_eq!(actual_event, expected_event);
}

#[test]
fn test_contributor_assigned_event() {
    let (env, admin, usdc_address, usdc_token, _, contract_id, client) = create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    // Set ledger timestamp
    let timestamp = 20000;
    env.ledger().set_timestamp(timestamp);

    // Create test data
    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "assign_evt", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;

    // Fund creator with USDC
    usdc_token.mint(&creator, &bounty_amount);

    // Create escrow
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );

    // Assign contributor to task
    client.assign_contributor(&task_id, &contributor);

    // Get all events and filter for contract events
    let events = env.events().all();
    let contract_events: std::vec::Vec<_> = events.iter().filter(|e| e.0 == contract_id).collect();

    let event = contract_events.last().unwrap();

    // Verify event topic (event name)
    let topics = &event.1;
    let topic_val = topics.iter().next().unwrap();
    let topic_sym: Symbol = topic_val.try_into_val(&env).unwrap();
    assert_eq!(topic_sym, Symbol::new(&env, "contributor_assigned_event"));

    // Verify event data
    let expected_event = ContributorAssignedEvent {
        task_id,
        contributor,
        timestamp,
    };

    let actual_event: ContributorAssignedEvent = event.2.try_into_val(&env).unwrap();
    assert_eq!(actual_event, expected_event);
}

#[test]
fn test_bounty_increased_event() {
    let (env, admin, usdc_address, usdc_token, _, contract_id, client) = create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    // Set ledger timestamp
    let timestamp = 30000;
    env.ledger().set_timestamp(timestamp);

    // Create test data
    let creator = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "inc_evt", 1);
    let initial_bounty = TestConfig::MEDIUM_AMOUNT;
    let increase_amount = 1_000_000;

    // Fund creator with USDC for initial bounty + increase
    usdc_token.mint(&creator, &(initial_bounty + increase_amount));

    // Create escrow
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &initial_bounty,
    );

    // Increase bounty amount
    client.increase_bounty(&creator, &task_id, &increase_amount);

    // Get all events and filter for contract events
    let events = env.events().all();
    let contract_events: std::vec::Vec<_> = events.iter().filter(|e| e.0 == contract_id).collect();

    let event = contract_events.last().unwrap();

    // Verify event topic (event name)
    let topics = &event.1;
    let topic_val = topics.iter().next().unwrap();
    let topic_sym: Symbol = topic_val.try_into_val(&env).unwrap();
    assert_eq!(topic_sym, Symbol::new(&env, "bounty_increased_event"));

    // Verify event data
    let expected_event = BountyIncreasedEvent {
        task_id,
        creator,
        added_amount: increase_amount,
        new_total_amount: initial_bounty + increase_amount,
        timestamp,
    };

    let actual_event: BountyIncreasedEvent = event.2.try_into_val(&env).unwrap();
    assert_eq!(actual_event, expected_event);
}

#[test]
fn test_bounty_decreased_event() {
    let (env, admin, usdc_address, usdc_token, _, contract_id, client) = create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    // Set ledger timestamp
    let timestamp = 40000;
    env.ledger().set_timestamp(timestamp);

    // Create test data
    let creator = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "dec_evt", 1);
    let initial_bounty = TestConfig::MEDIUM_AMOUNT;
    let decrease_amount = 1_000_000;

    // Fund creator with USDC
    usdc_token.mint(&creator, &initial_bounty);

    // Create escrow
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &initial_bounty,
    );

    // Decrease bounty amount
    client.decrease_bounty(&creator, &task_id, &decrease_amount);

    // Get all events and filter for contract events
    let events = env.events().all();
    let contract_events: std::vec::Vec<_> = events.iter().filter(|e| e.0 == contract_id).collect();

    let event = contract_events.last().unwrap();

    // Verify event topic (event name)
    let topics = &event.1;
    let topic_val = topics.iter().next().unwrap();
    let topic_sym: Symbol = topic_val.try_into_val(&env).unwrap();
    assert_eq!(topic_sym, Symbol::new(&env, "bounty_decreased_event"));

    // Verify event data
    let expected_event = BountyDecreasedEvent {
        task_id,
        creator,
        subtracted_amount: decrease_amount,
        new_total_amount: initial_bounty - decrease_amount,
        timestamp,
    };

    let actual_event: BountyDecreasedEvent = event.2.try_into_val(&env).unwrap();
    assert_eq!(actual_event, expected_event);
}

#[test]
fn test_funds_released_event() {
    let (env, admin, usdc_address, usdc_token, _, contract_id, client) = create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    // Set ledger timestamp
    let timestamp = 50000;
    env.ledger().set_timestamp(timestamp);

    // Create test data
    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "release_evt", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;

    // Fund creator with USDC
    usdc_token.mint(&creator, &bounty_amount);

    // Create escrow
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );

    // Assign contributor and approve completion
    client.assign_contributor(&task_id, &contributor);
    client.approve_completion(&task_id);

    // Get all events and filter for contract events
    let events = env.events().all();
    let contract_events: std::vec::Vec<_> = events.iter().filter(|e| e.0 == contract_id).collect();

    let event = contract_events.last().unwrap();

    // Verify event topic (event name)
    let topics = &event.1;
    let topic_val = topics.iter().next().unwrap();
    let topic_sym: Symbol = topic_val.try_into_val(&env).unwrap();
    assert_eq!(topic_sym, Symbol::new(&env, "funds_released_event"));

    // Verify event data
    let expected_event = FundsReleasedEvent {
        task_id,
        contributor,
        amount: bounty_amount,
        timestamp,
    };

    let actual_event: FundsReleasedEvent = event.2.try_into_val(&env).unwrap();
    assert_eq!(actual_event, expected_event);
}

#[test]
fn test_dispute_initiated_event() {
    let (env, admin, usdc_address, usdc_token, _, contract_id, client) = create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    // Set ledger timestamp
    let timestamp = 60000;
    env.ledger().set_timestamp(timestamp);

    // Create test data
    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "dispute_evt", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;

    // Fund creator with USDC
    usdc_token.mint(&creator, &bounty_amount);

    // Create escrow
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );

    // Assign contributor
    client.assign_contributor(&task_id, &contributor);

    // Initiate dispute
    let reason = TestValidation::generate_dispute_reason(&env, "quality");
    client.dispute_task(&creator, &task_id, &reason);

    // Get all events and filter for contract events
    let events = env.events().all();
    let contract_events: std::vec::Vec<_> = events.iter().filter(|e| e.0 == contract_id).collect();

    let event = contract_events.last().unwrap();

    // Verify event topic (event name)
    let topics = &event.1;
    let topic_val = topics.iter().next().unwrap();
    let topic_sym: Symbol = topic_val.try_into_val(&env).unwrap();
    assert_eq!(topic_sym, Symbol::new(&env, "dispute_initiated_event"));

    // Verify event data
    let expected_event = DisputeInitiatedEvent {
        task_id,
        disputing_party: creator,
        reason,
        timestamp,
    };

    let actual_event: DisputeInitiatedEvent = event.2.try_into_val(&env).unwrap();
    assert_eq!(actual_event, expected_event);
}

#[test]
fn test_dispute_resolved_event() {
    let (env, admin, usdc_address, usdc_token, _, contract_id, client) = create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    // Set ledger timestamp
    let timestamp = 70000;
    env.ledger().set_timestamp(timestamp);

    // Create test data
    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "resolve_evt", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;

    // Fund creator with USDC
    usdc_token.mint(&creator, &bounty_amount);

    // Create escrow
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );

    // Assign contributor and initiate dispute
    client.assign_contributor(&task_id, &contributor);
    let reason = TestValidation::generate_dispute_reason(&env, "quality");
    client.dispute_task(&creator, &task_id, &reason);

    // Resolve dispute
    client.resolve_dispute(&task_id, &DisputeResolution::PayContributor);

    // Get all events and filter for contract events
    let events = env.events().all();
    let contract_events: std::vec::Vec<_> = events.iter().filter(|e| e.0 == contract_id).collect();

    let event = contract_events.last().unwrap();

    // Verify event topic (event name)
    let topics = &event.1;
    let topic_val = topics.iter().next().unwrap();
    let topic_sym: Symbol = topic_val.try_into_val(&env).unwrap();
    assert_eq!(topic_sym, Symbol::new(&env, "dispute_resolved_event"));

    // Verify event data
    let expected_event = DisputeResolvedEvent {
        task_id,
        resolved_by: admin,
        resolution: DisputeResolution::PayContributor,
        timestamp,
    };

    let actual_event: DisputeResolvedEvent = event.2.try_into_val(&env).unwrap();
    assert_eq!(actual_event, expected_event);
}

#[test]
fn test_refund_processed_event() {
    let (env, admin, usdc_address, usdc_token, _, contract_id, client) = create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    // Set ledger timestamp
    let timestamp = 80000;
    env.ledger().set_timestamp(timestamp);

    // Create test data
    let creator = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "refund_evt", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;

    // Fund creator with USDC
    usdc_token.mint(&creator, &bounty_amount);

    // Create escrow
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );

    // Process refund
    client.refund(&task_id);

    // Get all events and filter for contract events
    let events = env.events().all();
    let contract_events: std::vec::Vec<_> = events.iter().filter(|e| e.0 == contract_id).collect();

    let event = contract_events.last().unwrap();

    // Verify event topic (event name)
    let topics = &event.1;
    let topic_val = topics.iter().next().unwrap();
    let topic_sym: Symbol = topic_val.try_into_val(&env).unwrap();
    assert_eq!(topic_sym, Symbol::new(&env, "refund_processed_event"));

    // Verify event data
    let expected_event = RefundProcessedEvent {
        task_id,
        creator,
        amount: bounty_amount,
        timestamp,
    };

    let actual_event: RefundProcessedEvent = event.2.try_into_val(&env).unwrap();
    assert_eq!(actual_event, expected_event);
}
