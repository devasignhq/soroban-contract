use soroban_sdk::{testutils::Address as _, Address};
use devasign_task_escrow::{Error};

mod test_setup;
mod test_config;

use test_setup::create_test_env;

#[test]
fn test_initialize_success() {
    let (
        _env, 
        admin, 
        usdc_address,
        _usdc_token, 
        _usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();
    
    let result = client.initialize(&admin, &usdc_address);
    assert!(result == ());
    
    // Verify admin is set correctly
    let stored_admin = client.get_admin();
    assert!(stored_admin == admin);
    
    // Verify USDC token is set correctly
    let stored_token = client.get_usdc_token();
    assert!(stored_token == usdc_address);
    
    // Verify task count is initialized to 0
    let task_count = client.get_task_count();
    assert_eq!(task_count, 0);
}

#[test]
fn test_initialize_already_initialized() {
    let (
        env, 
        admin, 
        usdc_address,
        _usdc_token, 
        _usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();
    
    // Initialize once
    let result = client.initialize(&admin, &usdc_address);
    assert!(result == ());
    
    // Try to initialize again
    let new_admin = Address::generate(&env);
    let result = client.try_initialize(&new_admin, &usdc_address);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::TaskAlreadyExists);
}

#[test]
fn test_set_admin_success() {
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
    
    // Set new admin
    let new_admin = Address::generate(&env);
    let result = client.set_admin(&new_admin);
    assert!(result == ());
    
    // Verify new admin is set
    let stored_admin = client.get_admin();
    assert!(stored_admin == new_admin);
}

#[test]
fn test_set_admin_unauthorized() {
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
    
    // Try to set admin from non-admin address
    let unauthorized = Address::generate(&env);
    env.mock_all_auths_allowing_non_root_auth();
    
    let result = client.mock_auths(&[]).try_set_admin(&unauthorized);
    assert!(result.is_err());
}

#[test]
fn test_get_admin_not_initialized() {
    let (
        _env, 
        _admin, 
        _usdc_address,
        _usdc_token, 
        _usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();
    
    let result = client.try_get_admin();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::ContractNotInitialized);
}

#[test]
fn test_get_usdc_token_not_initialized() {
    let (
        _env, 
        _admin, 
        _usdc_address,
        _usdc_token, 
        _usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();
    
    let result = client.try_get_usdc_token();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::ContractNotInitialized);
}

#[test]
fn test_validate_usdc_token_contract() {
    let (
        _env, 
        admin, 
        usdc_address,
        _usdc_token, 
        _usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();
    
    // Initialize contract
    client.initialize(&admin, &usdc_address);
    
    // Validate USDC token contract
    let result = client.validate_usdc_token_contract();
    assert!(result == true);
}

#[test]
fn test_get_usdc_token_info() {
    let (
        _env, 
        admin, 
        usdc_address,
        _usdc_token, 
        _usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();
    
    // Initialize contract
    client.initialize(&admin, &usdc_address);
    
    // Get token info
    let result = client.get_usdc_token_info();
    
    let (token_address, is_valid) = result;
    assert_eq!(token_address, usdc_address);
    assert_eq!(is_valid, true);
}