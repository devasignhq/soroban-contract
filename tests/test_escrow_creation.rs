use soroban_sdk::{testutils::Address as _, Address, String};
use devasign_task_escrow::{Error, TaskStatus};

mod test_setup;
mod test_config;

use test_setup::create_test_env;
use test_config::{TestConfig, TestScenarios, TestValidation};

#[test]
fn test_create_escrow_success() {
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
    
    // Create test data
    let creator: Address = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "test", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;
    
    // Fund creator with USDC
    usdc_token.mint(&creator, &bounty_amount);
    
    // Create escrow
    let result = client.create_escrow(&creator, &task_id, &TestValidation::dummy_issue_url(&env), &bounty_amount);
    assert!(result == ());
    
    // Verify escrow was created
    let escrow = client.get_escrow(&task_id);
    assert!(escrow.task_id == task_id);
    
    let escrow_data = escrow;
    assert_eq!(escrow_data.creator, creator);
    assert_eq!(escrow_data.task_id, task_id);
    assert_eq!(escrow_data.issue_url, TestValidation::dummy_issue_url(&env));
    assert_eq!(escrow_data.bounty_amount, bounty_amount);
    assert_eq!(escrow_data.status, TaskStatus::Open);
    assert_eq!(escrow_data.has_contributor, false);
    
    // Verify task count increased
    let task_count = client.get_task_count();
    assert_eq!(task_count, 1);
}

#[test]
fn test_create_escrow_multiple_amounts() {
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
    let amounts = TestScenarios::standard_amounts();
    
    for (i, &amount) in amounts.iter().enumerate() {
        let task_id = TestValidation::generate_task_id(&env, "multi", i as u32);
        
        // Fund creator with USDC
        usdc_token.mint(&creator, &amount);
        
        // Create escrow
        let result = client.create_escrow(&creator, &task_id, &TestValidation::dummy_issue_url(&env), &amount);
        assert!(result == ());
        
        // Verify escrow
        let escrow = client.get_escrow(&task_id);
        assert!(escrow.task_id == task_id);
        assert_eq!(escrow.bounty_amount, amount);
    }
    
    // Verify task count
    let task_count = client.get_task_count();
    assert_eq!(task_count, amounts.len() as u64);
}

#[test]
fn test_create_escrow_boundary_amounts() {
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
    let amounts = TestScenarios::boundary_amounts();
    
    for (i, &amount) in amounts.iter().enumerate() {
        let task_id = TestValidation::generate_task_id(&env, "boundary", i as u32);
        
        // Fund creator with USDC
        usdc_token.mint(&creator, &amount);
        
        // Create escrow
        let result = client.create_escrow(&creator, &task_id, &TestValidation::dummy_issue_url(&env), &amount);
        assert!(result == ());
    }
}

#[test]
fn test_create_escrow_invalid_amounts() {
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
    let invalid_amounts = TestScenarios::invalid_amounts();
    
    for (i, &amount) in invalid_amounts.iter().enumerate() {
        let task_id = TestValidation::generate_task_id(&env, "invalid", i as u32);
        
        // Try to create escrow with invalid amount
        let result = client.try_create_escrow(&creator, &task_id, &TestValidation::dummy_issue_url(&env), &amount);
        assert!(result.is_err(), "Should fail with invalid amount: {}", amount);
        
        let error = result.unwrap_err().unwrap();
        assert!(matches!(error, Error::InvalidAmount | Error::InvalidTokenAmount));
    }
}

#[test]
fn test_create_escrow_duplicate_task_id() {
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
    let task_id = TestValidation::generate_task_id(&env, "duplicate", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;
    
    // Fund creator with USDC
    usdc_token.mint(&creator, &(bounty_amount * 2));
    
    // Create first escrow
    let result = client.create_escrow(&creator, &task_id, &TestValidation::dummy_issue_url(&env), &bounty_amount);
    assert!(result == ());
    
    // Try to create duplicate
    let result = client.try_create_escrow(&creator, &task_id, &TestValidation::dummy_issue_url(&env), &bounty_amount);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::TaskAlreadyExists);
}

#[test]
fn test_create_escrow_invalid_task_ids() {
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
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;
    
    // Fund creator
    usdc_token.mint(&creator, &(bounty_amount * 10));
    
    // Test empty task ID
    let result = client.try_create_escrow(
        &creator, 
        &String::from_str(&env, &"".to_string()), 
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::EmptyTaskId);
    
    // Test too short task ID
    let result = client.try_create_escrow(
        &creator, 
        &String::from_str(&env, &"ab".to_string()), 
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidTaskId);
    
    // Test too long task ID
    let long_id = "a".repeat(101);
    let result = client.try_create_escrow(
        &creator, 
        &String::from_str(&env, &long_id), 
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidTaskId);
}

#[test]
fn test_create_escrow_insufficient_balance() {
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
    let task_id = TestValidation::generate_task_id(&env, "insufficient", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;
    
    // Don't fund creator (insufficient balance)
    let result = client.try_create_escrow(&creator, &task_id, &TestValidation::dummy_issue_url(&env), &bounty_amount);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InsufficientBalance);
}

#[test]
fn test_create_escrow_not_initialized() {
    let (
        env, 
        _admin, 
        _usdc_address,
        _usdc_token, 
        _usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();
    
    let creator = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "uninit", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;
    
    // Try to create escrow without initializing contract
    let result = client.try_create_escrow(&creator, &task_id, &TestValidation::dummy_issue_url(&env), &bounty_amount);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::ContractNotInitialized);
}

#[test]
fn test_get_escrow_not_found() {
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
    
    // Try to get non-existent escrow
    let result = client.try_get_escrow(&task_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::TaskNotFound);
}

#[test]
fn test_get_escrow_invalid_task_id() {
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
    
    // Try to get escrow with invalid task ID
    let result = client.try_get_escrow(&String::from_str(&env, &"ab".to_string()));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidTaskId);
}