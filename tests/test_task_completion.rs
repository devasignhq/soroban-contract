use soroban_sdk::{testutils::Address as _, Address, Env, String};
use soroban_sdk::testutils::Ledger;
use devasign_task_escrow::{TaskEscrowContractClient, Error, TaskStatus};

mod test_setup;
mod test_config;

use test_setup::create_test_env;
use test_config::{TestConfig, TestValidation};

fn setup_task_in_progress(
    env: &Env,
    client: &TaskEscrowContractClient,
    usdc_token: &soroban_sdk::token::StellarAssetClient,
    creator: &Address,
    contributor: &Address,
    task_prefix: &str,
) -> String {
    let task_id = TestValidation::generate_task_id(&env, task_prefix, 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;
    
    // Fund creator and create escrow
    usdc_token.mint(creator, &bounty_amount);
    client.create_escrow(creator, &task_id, &bounty_amount);
    
    // Assign contributor
    client.assign_contributor(&task_id, contributor);
    
    task_id
}

#[test]
fn test_complete_task_success() {
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
    let task_id = setup_task_in_progress(&env, &client, &usdc_token, &creator, &contributor, "complete");
    
    // Complete task
    let result = client.complete_task(&task_id);
    assert!(result == ());
    
    // Verify completion
    let escrow = client.get_escrow(&task_id);
    assert!(escrow.task_id == task_id);
    
    assert_eq!(escrow.status, TaskStatus::Completed);
}

#[test]
fn test_complete_task_unauthorized() {
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
    let task_id = setup_task_in_progress(&env, &client, &usdc_token, &creator, &contributor, "unauth");
    
    // Try to complete task from unauthorized address
    env.mock_all_auths_allowing_non_root_auth();
    let result = client.mock_auths(&[]).try_complete_task(&task_id);
    assert!(result.is_err());
}

#[test]
fn test_complete_task_not_found() {
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
    
    let task_id = TestValidation::generate_task_id(&env, "notfound", 1);
    
    // Try to complete non-existent task
    let result = client.try_complete_task(&task_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::TaskNotFound);
}

#[test]
fn test_complete_task_invalid_status_open() {
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
    let task_id = TestValidation::generate_task_id(&env, "open", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;
    
    // Create escrow but don't assign contributor
    usdc_token.mint(&creator, &bounty_amount);
    client.create_escrow(&creator, &task_id, &bounty_amount);
    
    // Try to complete task that's still open
    let result = client.try_complete_task(&task_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidTaskStatus);
}

#[test]
fn test_complete_task_no_contributor() {
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
    let task_id = TestValidation::generate_task_id(&env, "nocontrib", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;
    
    // Create escrow but don't assign contributor
    usdc_token.mint(&creator, &bounty_amount);
    client.create_escrow(&creator, &task_id, &bounty_amount);
    
    // Try to complete task without contributor
    let result = client.try_complete_task(&task_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidTaskStatus);
}

#[test]
fn test_complete_task_already_completed() {
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
    let task_id = setup_task_in_progress(&env, &client, &usdc_token, &creator, &contributor, "already");
    
    // Complete task first time
    let result = client.complete_task(&task_id);
    assert!(result == ());
    
    // Try to complete again
    let result = client.try_complete_task(&task_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidTaskStatus);
}

#[test]
fn test_complete_task_invalid_task_id() {
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
    
    // Test empty task ID
    let result = client.try_complete_task(
        &String::from_str(&env, &"".to_string())
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::EmptyTaskId);
    
    // Test too short task ID
    let result = client.try_complete_task(
        &String::from_str(&env, &"ab".to_string())
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidTaskId);
    
    // Test too long task ID
    let long_id = "a".repeat(101);
    let result = client.try_complete_task(
        &String::from_str(&env, &long_id)
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidTaskId);
}

#[test]
fn test_complete_task_not_initialized() {
    let (
        env, 
        _admin, 
        _usdc_address,
        _usdc_token, 
        _usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();
    
    let task_id = TestValidation::generate_task_id(&env, "uninit", 1);
    
    // Try to complete task without initializing contract
    let result = client.try_complete_task(&task_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::ContractNotInitialized);
}

#[test]
fn test_complete_multiple_tasks() {
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
    
    // Create and complete multiple tasks
    for i in 0..3 {
        let task_id = setup_task_in_progress(&env, &client, &usdc_token, &creator, &contributor, &format!("multi{}", i));
        
        // Complete task
        let result = client.complete_task(&task_id);
        assert!(result == ());
        
        // Verify completion
        let escrow = client.get_escrow(&task_id);
        assert!(escrow.task_id == task_id);
        

        assert_eq!(escrow.status, TaskStatus::Completed);
    }
}

#[test]
fn test_complete_task_timestamp_progression() {
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
    
    let mut previous_timestamp = 0u64;
    
    // Create and complete multiple tasks, checking timestamp progression
    for i in 0..3 {
        let task_id = setup_task_in_progress(&env, &client, &usdc_token, &creator, &contributor, &format!("time{}", i));
        
        // Advance ledger time
        env.ledger().with_mut(|li| li.timestamp += 100);
        
        // Complete task
        let result = client.complete_task(&task_id);
        assert!(result == ());
        
        // Verify timestamp progression
        let escrow = client.get_escrow(&task_id);
        assert!(escrow.task_id == task_id);
        

        assert!(escrow.completed_at > previous_timestamp);
        previous_timestamp = escrow.completed_at;
    }
}