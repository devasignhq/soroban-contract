use soroban_sdk::{
    testutils::Address as _, 
    Address, 
    Env, 
    String, 
    testutils::Events,
    IntoVal
};
use devasign_task_escrow::{TaskEscrowContractClient, DisputeResolution};

mod test_setup;
mod test_config;

use test_setup::create_test_env;
use test_config::{TestConfig, TestValidation};

/// Helper function to setup a basic escrow for event testing
fn setup_basic_escrow(
    env: &Env,
    _client: &TaskEscrowContractClient,
    usdc_token: &soroban_sdk::token::StellarAssetClient,
    creator: &Address,
    task_prefix: &str,
) -> (String, i128) {
    let task_id = TestValidation::generate_task_id(&env, task_prefix, 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;
    
    // Fund creator
    usdc_token.mint(creator, &bounty_amount);
    
    (task_id, bounty_amount)
}

#[test]
fn test_escrow_created_event() {
    let (
        env, 
        admin, 
        usdc_address,
        usdc_token, 
        _usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();
    
    // Initialize contract
    client.initialize(&admin, &usdc_address);
    
    let creator = Address::generate(&env);
    let (task_id, bounty_amount) = setup_basic_escrow(&env, &client, &usdc_token, &creator, "event_escrow");
    
    // Create escrow and capture events
    client.create_escrow(&creator, &task_id, &bounty_amount);
    
    // Verify escrow created event was emitted
    let events = env.events().all();
    let escrow_events: Vec<_> = events
        .iter()
        .filter(|e| e.1 == ("EscrowCreated",).into_val(&env))
        .collect();
    
    assert!(escrow_events.len() == 1, "EscrowCreated event should be emitted");
}

#[test]
fn test_contributor_assigned_event() {
    let (
        env, 
        admin, 
        usdc_address,
        usdc_token, 
        _usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();
    
    // Initialize contract
    client.initialize(&admin, &usdc_address);
    
    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let (task_id, bounty_amount) = setup_basic_escrow(&env, &client, &usdc_token, &creator, "event_assign");
    
    // Create escrow
    client.create_escrow(&creator, &task_id, &bounty_amount);
    
    // Assign contributor and capture events
    client.assign_contributor(&task_id, &contributor);
    
    // Verify contributor assigned event was emitted
    let events = env.events().all();
    let assignment_events: Vec<_> = events
        .iter()
        .filter(|e| e.1 == ("ContributorAssigned",).into_val(&env))
        .collect();
    
    assert!(assignment_events.len() == 1, "ContributorAssigned event should be emitted");
}

#[test]
fn test_task_completed_event() {
    let (
        env, 
        admin, 
        usdc_address,
        usdc_token, 
        _usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();
    
    // Initialize contract
    client.initialize(&admin, &usdc_address);
    
    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let (task_id, bounty_amount) = setup_basic_escrow(&env, &client, &usdc_token, &creator, "event_complete");
    
    // Setup task
    client.create_escrow(&creator, &task_id, &bounty_amount);
    client.assign_contributor(&task_id, &contributor);
    
    // Complete task and capture events
    client.complete_task(&task_id);
    
    // Verify task completed event was emitted
    let events = env.events().all();
    let completion_events: Vec<_> = events
        .iter()
        .filter(|e| e.1 == ("TaskCompleted",).into_val(&env))
        .collect();
    
    assert!(completion_events.len() == 1, "TaskCompleted event should be emitted");
}

#[test]
fn test_funds_released_event() {
    let (
        env, 
        admin, 
        usdc_address,
        usdc_token, 
        _usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();
    
    // Initialize contract
    client.initialize(&admin, &usdc_address);
    
    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let (task_id, bounty_amount) = setup_basic_escrow(&env, &client, &usdc_token, &creator, "event_release");
    
    // Setup and complete task
    client.create_escrow(&creator, &task_id, &bounty_amount);
    client.assign_contributor(&task_id, &contributor);
    client.complete_task(&task_id);
    
    // Approve completion and capture events
    client.approve_completion(&task_id);
    
    // Verify funds released event was emitted
    let events = env.events().all();
    let release_events: Vec<_> = events
        .iter()
        .filter(|e| e.1 == ("FundsReleased",).into_val(&env))
        .collect();
    
    assert!(release_events.len() == 1, "FundsReleased event should be emitted");
}

#[test]
fn test_refund_processed_event() {
    let (
        env, 
        admin, 
        usdc_address,
        usdc_token, 
        _usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();
    
    // Initialize contract
    client.initialize(&admin, &usdc_address);
    
    let creator = Address::generate(&env);
    let (task_id, bounty_amount) = setup_basic_escrow(&env, &client, &usdc_token, &creator, "event_refund");
    
    // Create escrow
    client.create_escrow(&creator, &task_id, &bounty_amount);
    
    // Process refund and capture events
    client.refund(&task_id);
    
    // Verify refund processed event was emitted
    let events = env.events().all();
    let refund_events: Vec<_> = events
        .iter()
        .filter(|e| e.1 == ("RefundProcessed",).into_val(&env))
        .collect();
    
    assert!(refund_events.len() == 1, "RefundProcessed event should be emitted");
}

#[test]
fn test_dispute_initiated_event() {
    let (
        env, 
        admin, 
        usdc_address,
        usdc_token, 
        _usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();
    
    // Initialize contract
    client.initialize(&admin, &usdc_address);
    
    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let (task_id, bounty_amount) = setup_basic_escrow(&env, &client, &usdc_token, &creator, "event_dispute");
    
    // Setup and complete task
    client.create_escrow(&creator, &task_id, &bounty_amount);
    client.assign_contributor(&task_id, &contributor);
    client.complete_task(&task_id);
    
    // Initiate dispute and capture events
    let reason = TestValidation::generate_dispute_reason(&env, "quality");
    client.dispute_task(&creator, &task_id, &reason);
    
    // Verify dispute initiated event was emitted
    let events = env.events().all();
    let dispute_events: Vec<_> = events
        .iter()
        .filter(|e| e.1 == ("DisputeInitiated",).into_val(&env))
        .collect();
    
    assert!(dispute_events.len() == 1, "DisputeInitiated event should be emitted");
}

#[test]
fn test_dispute_resolved_event() {
    let (
        env, 
        admin, 
        usdc_address,
        usdc_token, 
        _usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();
    
    // Initialize contract
    client.initialize(&admin, &usdc_address);
    
    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let (task_id, bounty_amount) = setup_basic_escrow(&env, &client, &usdc_token, &creator, "event_resolve");
    
    // Setup, complete, and dispute task
    client.create_escrow(&creator, &task_id, &bounty_amount);
    client.assign_contributor(&task_id, &contributor);
    client.complete_task(&task_id);
    
    let reason = TestValidation::generate_dispute_reason(&env, "quality");
    client.dispute_task(&creator, &task_id, &reason);
    
    // Resolve dispute and capture events
    client.resolve_dispute(&task_id, &DisputeResolution::PayContributor);
    
    // Verify dispute resolved event was emitted
    let events = env.events().all();
    let resolution_events: Vec<_> = events
        .iter()
        .filter(|e| e.1 == ("DisputeResolved",).into_val(&env))
        .collect();
    
    assert!(resolution_events.len() == 1, "DisputeResolved event should be emitted");
}

#[test]
fn test_multiple_events_in_workflow() {
    let (
        env, 
        admin, 
        usdc_address,
        usdc_token, 
        _usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();
    
    // Initialize contract
    client.initialize(&admin, &usdc_address);
    
    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let (task_id, bounty_amount) = setup_basic_escrow(&env, &client, &usdc_token, &creator, "event_workflow");
    
    // Execute complete workflow
    client.create_escrow(&creator, &task_id, &bounty_amount);
    client.assign_contributor(&task_id, &contributor);
    client.complete_task(&task_id);
    client.approve_completion(&task_id);
    
    // Verify multiple events were emitted in sequence
    let events = env.events().all();
    
    // Should have at least: escrow_created, contributor_assigned, task_completed, funds_released
    // let event_topics: Vec<String> = events
    //     .iter()
    //     .filter_map(|e| e.get(0).map(|t| t.to_string()))
    //     .collect();
    
    // Basic check that multiple events were emitted
    assert!(events.len() >= 4, "Should emit multiple events during workflow");
}

#[test]
fn test_event_emission_with_different_amounts() {
    let (
        env, 
        admin, 
        usdc_address,
        usdc_token, 
        _usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();
    
    // Initialize contract
    client.initialize(&admin, &usdc_address);
    
    let creator = Address::generate(&env);
    let amounts = vec![
        TestConfig::SMALL_AMOUNT,
        TestConfig::MEDIUM_AMOUNT,
        TestConfig::LARGE_AMOUNT,
    ];
    
    for (i, &amount) in amounts.iter().enumerate() {
        let task_id = TestValidation::generate_task_id(&env, "event_amounts", i as u32);
        
        // Fund and create escrow
        usdc_token.mint(&creator, &amount);
        client.create_escrow(&creator, &task_id, &amount);
        
        // Verify event was emitted for each amount
        let events = env.events().all();
        let escrow_events: Vec<_> = events
            .iter()
        .filter(|e| e.1 == ("EscrowCreated",).into_val(&env))
            .collect();
        
        assert!(escrow_events.len() >= (i + 1), "Should emit event for each escrow creation");
    }
}

#[test]
fn test_event_emission_order() {
    let (
        env, 
        admin, 
        usdc_address,
        usdc_token, 
        _usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();
    
    // Initialize contract
    client.initialize(&admin, &usdc_address);
    
    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let (task_id, bounty_amount) = setup_basic_escrow(&env, &client, &usdc_token, &creator, "event_order");
    
    // Clear any existing events
    let initial_event_count = env.events().all().len();
    
    // Execute operations in sequence
    client.create_escrow(&creator, &task_id, &bounty_amount);
    let events_after_create = env.events().all().len();
    
    client.assign_contributor(&task_id, &contributor);
    let events_after_assign = env.events().all().len();
    
    client.complete_task(&task_id);
    let events_after_complete = env.events().all().len();
    
    // Verify events were emitted in order (each operation should add events)
    assert!(events_after_create > initial_event_count, "Create should emit events");
    assert!(events_after_assign > events_after_create, "Assign should emit events");
    assert!(events_after_complete > events_after_assign, "Complete should emit events");
}

#[test]
fn test_no_events_on_failed_operations() {
    let (
        env, 
        admin, 
        usdc_address,
        _usdc_token, 
        _usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();
    
    // Initialize contract
    client.initialize(&admin, &usdc_address);
    
    let creator = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "event_fail", 1);
    
    // Get initial event count
    let initial_event_count = env.events().all().len();
    
    // Try to create escrow without funding (should fail)
    let result = client.try_create_escrow(&creator, &task_id, &TestConfig::MEDIUM_AMOUNT);
    assert!(result.is_err());
    
    // Verify no additional events were emitted on failure
    let final_event_count = env.events().all().len();
    assert_eq!(final_event_count, initial_event_count, "Failed operations should not emit events");
}