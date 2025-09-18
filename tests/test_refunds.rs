use soroban_sdk::{testutils::Address as _, Address, Env, String};
use devasign_task_escrow::{TaskEscrowContractClient, TaskStatus, Error};

mod test_setup;
mod test_config;

use test_setup::create_test_env;
use test_config::{TestConfig, TestValidation, TestScenarios};

/// Helper function to setup a task ready for refund testing
fn setup_task_for_refund(
    env: &Env,
    client: &TaskEscrowContractClient,
    usdc_token: &soroban_sdk::token::StellarAssetClient,
    creator: &Address,
    task_prefix: &str,
    assign_contributor: bool,
) -> (String, i128) {
    let task_id = TestValidation::generate_task_id(&env, task_prefix, 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;
    
    // Fund creator and create escrow
    usdc_token.mint(creator, &bounty_amount);
    client.create_escrow(creator, &task_id, &bounty_amount);
    
    // Optionally assign contributor
    if assign_contributor {
        let contributor = Address::generate(env);
        client.assign_contributor(&task_id, &contributor);
    }
    
    (task_id, bounty_amount)
}

#[test]
fn test_refund_success_no_contributor() {
    let (
        env, 
        admin, 
        usdc_address,
        usdc_token, 
        usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);
    
    let creator = Address::generate(&env);
    let (task_id, bounty_amount) = setup_task_for_refund(&env, &client, &usdc_token, &creator, "refund_no_contrib", false);
    
    // Get initial balance
    let initial_balance = usdc_token_client.balance(&creator);
    
    // Process refund
    let result = client.refund(&task_id);
    assert!(result == ());
    
    // Verify refund
    let final_balance = usdc_token_client.balance(&creator);
    assert_eq!(final_balance, initial_balance + bounty_amount);
    
    // Verify escrow status
    let escrow = client.get_escrow(&task_id);
    assert_eq!(escrow.status, TaskStatus::Cancelled);
}

#[test]
fn test_refund_success_with_contributor() {
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
    let (task_id, _) = setup_task_for_refund(&env, &client, &usdc_token, &creator, "refund_with_contrib", true);
    
    // Process refund
    let result = client.try_refund(&task_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::ContributorAlreadyAssigned);
}

#[test]
fn test_refund_multiple_amounts() {
    let (
        env, 
        admin, 
        usdc_address,
        usdc_token, 
        usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);
    
    let creator = Address::generate(&env);
    let amounts = TestScenarios::standard_amounts();
    
    for (i, &amount) in amounts.iter().enumerate() {
        let task_id = TestValidation::generate_task_id(&env, "refund_multi", i as u32);
        
        // Fund creator and create escrow
        usdc_token.mint(&creator, &amount);
        client.create_escrow(&creator, &task_id, &amount);
        
        // Get initial balance
        let initial_balance = usdc_token_client.balance(&creator);
        
        // Process refund
        let result = client.refund(&task_id);
        assert!(result == ());
        
        // Verify refund
        let final_balance = usdc_token_client.balance(&creator);
        assert_eq!(final_balance, initial_balance + amount);
    }
}

#[test]
fn test_refund_boundary_amounts() {
    let (
        env, 
        admin, 
        usdc_address,
        usdc_token, 
        usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);
    
    let creator = Address::generate(&env);
    let amounts = TestScenarios::boundary_amounts();
    
    for (i, &amount) in amounts.iter().enumerate() {
        let task_id = TestValidation::generate_task_id(&env, "refund_boundary", i as u32);
        
        // Fund creator and create escrow
        usdc_token.mint(&creator, &amount);
        client.create_escrow(&creator, &task_id, &amount);
        
        // Get initial balance
        let initial_balance = usdc_token_client.balance(&creator);
        
        // Process refund
        let result = client.refund(&task_id);
        assert!(result == ());
        
        // Verify refund
        let final_balance = usdc_token_client.balance(&creator);
        assert_eq!(final_balance, initial_balance + amount);
    }
}

#[test]
fn test_refund_unauthorized() {
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
    let (task_id, _) = setup_task_for_refund(&env, &client, &usdc_token, &creator, "refund_unauth", false);
    
    // Try to refund from unauthorized address
    env.mock_all_auths_allowing_non_root_auth();
    let result = client.mock_auths(&[]).try_refund(&task_id);
    assert!(result.is_err());
}

#[test]
fn test_refund_task_not_found() {
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
    
    // Try to refund non-existent task
    let result = client.try_refund(&task_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::TaskNotFound);
}

#[test]
fn test_refund_invalid_task_id() {
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
    let result = client.try_refund(
        &String::from_str(&env, &"".to_string())
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::EmptyTaskId);
    
    // Test too short task ID
    let result = client.try_refund(
        &String::from_str(&env, &"ab".to_string())
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidTaskId);
    
    // Test too long task ID
    let long_id = "a".repeat(101);
    let result = client.try_refund(
        &String::from_str(&env, &long_id)
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidTaskId);
}

#[test]
fn test_refund_task_already_refunded() {
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
    let (task_id, _) = setup_task_for_refund(&env, &client, &usdc_token, &creator, "refund_already", false);
    
    // Process first refund
    let result = client.refund(&task_id);
    assert!(result == ());
    
    // Try to refund again
    let result = client.try_refund(&task_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidTaskStatus);
}

#[test]
fn test_refund_not_initialized() {
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
    
    // Try to refund without initializing contract
    let result = client.try_refund(&task_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::ContractNotInitialized);
}

#[test]
fn test_refund_contract_balance_verification() {
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
    let (task_id, bounty_amount) = setup_task_for_refund(&env, &client, &usdc_token, &creator, "refund_balance", false);
    
    // Verify contract has the funds before refund
    let contract_balance_before = client.get_contract_usdc_balance();
    assert_eq!(contract_balance_before, bounty_amount);
    
    // Process refund
    let result = client.refund(&task_id);
    assert!(result == ());
    
    // Verify contract balance after refund
    let contract_balance_after = client.get_contract_usdc_balance();
    assert_eq!(contract_balance_after, 0);
}

#[test]
fn test_refund_multiple_tasks_same_creator() {
    let (
        env, 
        admin, 
        usdc_address,
        usdc_token, 
        usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);
    
    let creator = Address::generate(&env);
    let task_count = 3;
    let bounty_amount = TestConfig::SMALL_AMOUNT;
    
    // Create multiple tasks
    let mut task_ids = Vec::new();
    for i in 0..task_count {
        let task_id = TestValidation::generate_task_id(&env, "multi_refund", i);
        usdc_token.mint(&creator, &bounty_amount);
        client.create_escrow(&creator, &task_id, &bounty_amount);
        task_ids.push(task_id);
    }
    
    // Get initial balance
    let initial_balance = usdc_token_client.balance(&creator);
    
    // Refund all tasks
    for task_id in &task_ids {
        let result = client.refund(task_id);
        assert!(result == ());
    }
    
    // Verify total refund
    let final_balance = usdc_token_client.balance(&creator);
    let expected_total_refund = bounty_amount * task_count as i128;
    assert_eq!(final_balance, initial_balance + expected_total_refund);
}

#[test]
fn test_refund_timestamp_verification() {
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
    let (task_id, _) = setup_task_for_refund(&env, &client, &usdc_token, &creator, "refund_timestamp", false);
    
    // Get escrow before refund
    let escrow_before = client.get_escrow(&task_id);
    
    // Process refund
    let result = client.refund(&task_id);
    assert!(result == ());
    
    // Verify escrow after refund
    let escrow_after = client.get_escrow(&task_id);
    
    // Verify status changed and timestamps are reasonable
    assert_eq!(escrow_after.status, TaskStatus::Cancelled);
    assert_eq!(escrow_after.created_at, escrow_before.created_at);
}