use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};
use devasign_task_escrow::{Error, TaskEscrowContract};

mod test_setup;
mod test_config;

use test_setup::create_test_env;

/// Helper function to create a mock WASM hash for testing
fn create_mock_wasm_hash(env: &Env) -> BytesN<32> {
    // Create a mock 32-byte hash for testing
    // In a real scenario, this would be the hash of compiled WASM bytecode
    BytesN::from_array(
        env,
        &[
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
            0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
        ],
    )
}

/// Helper function to create a different mock WASM hash for testing
fn create_different_mock_wasm_hash(env: &Env) -> BytesN<32> {
    BytesN::from_array(
        env,
        &[
            0xff, 0xfe, 0xfd, 0xfc, 0xfb, 0xfa, 0xf9, 0xf8,
            0xf7, 0xf6, 0xf5, 0xf4, 0xf3, 0xf2, 0xf1, 0xf0,
            0xef, 0xee, 0xed, 0xec, 0xeb, 0xea, 0xe9, 0xe8,
            0xe7, 0xe6, 0xe5, 0xe4, 0xe3, 0xe2, 0xe1, 0xe0,
        ],
    )
}

#[test]
fn test_version() {
    let (
        _env, 
        _admin, 
        _usdc_address,
        _usdc_token, 
        _usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();
    
    // Test that version returns the expected value
    let version = client.version();
    assert_eq!(version, 1);
}

#[test]
fn test_upgrade_success() {
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
    
    // Create a mock WASM hash
    let new_wasm_hash = create_mock_wasm_hash(&env);
    
    // Upgrade the contract (as admin)
    let result = client.upgrade(&new_wasm_hash);
    assert!(result == ());
}

#[test]
fn test_upgrade_unauthorized() {
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
    
    // Create a mock WASM hash
    let new_wasm_hash = create_mock_wasm_hash(&env);
    
    // Try to upgrade from non-admin address
    env.mock_all_auths_allowing_non_root_auth();
    
    let result = client.mock_auths(&[]).try_upgrade(&new_wasm_hash);
    assert!(result.is_err());
}

#[test]
fn test_upgrade_not_initialized() {
    let (
        env, 
        _admin, 
        _usdc_address,
        _usdc_token, 
        _usdc_token_client, 
        _contract_id, 
        client
    ) = create_test_env();
    
    // Create a mock WASM hash
    let new_wasm_hash = create_mock_wasm_hash(&env);
    
    // Try to upgrade before initialization
    let result = client.try_upgrade(&new_wasm_hash);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::ContractNotInitialized);
}

#[test]
fn test_upgrade_multiple_times() {
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
    
    // First upgrade
    let first_wasm_hash = create_mock_wasm_hash(&env);
    let result1 = client.upgrade(&first_wasm_hash);
    assert!(result1 == ());
    
    // Second upgrade with different hash
    let second_wasm_hash = create_different_mock_wasm_hash(&env);
    let result2 = client.upgrade(&second_wasm_hash);
    assert!(result2 == ());
    
    // Verify version remains consistent
    let version = client.version();
    assert_eq!(version, 1);
}

#[test]
fn test_upgrade_after_admin_change() {
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
    
    // Change admin
    let new_admin = Address::generate(&env);
    client.set_admin(&new_admin);
    
    // Create a mock WASM hash
    let new_wasm_hash = create_mock_wasm_hash(&env);
    
    // Old admin should not be able to upgrade
    env.mock_all_auths_allowing_non_root_auth();
    let result = client.mock_auths(&[]).try_upgrade(&new_wasm_hash);
    assert!(result.is_err());
    
    // New admin should be able to upgrade
    env.mock_all_auths();
    let result = client.upgrade(&new_wasm_hash);
    assert!(result == ());
}

#[test]
fn test_upgrade_preserves_contract_state() {
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
    
    // Create some state by creating an escrow
    let creator = Address::generate(&env);
    let task_id = soroban_sdk::String::from_str(&env, "test-task-id-0000000001");
    let bounty_amount = 1000_0000000i128; // 1,000 USDC
    
    // Mint USDC to creator
    usdc_token.mint(&creator, &bounty_amount);
    
    // Create escrow
    client.create_escrow(&creator, &task_id, &bounty_amount);
    
    // Verify task count before upgrade
    let task_count_before = client.get_task_count();
    assert_eq!(task_count_before, 1);
    
    // Verify escrow exists before upgrade
    let escrow_before = client.get_escrow(&task_id);
    assert!(escrow_before.task_id == task_id);
    
    // Perform upgrade
    let new_wasm_hash = create_mock_wasm_hash(&env);
    let result = client.upgrade(&new_wasm_hash);
    assert!(result == ());
    
    // Verify state is preserved after upgrade
    let task_count_after = client.get_task_count();
    assert_eq!(task_count_after, 1);
    
    // Verify escrow still exists after upgrade
    let escrow_after = client.get_escrow(&task_id);
    assert!(escrow_after.task_id == task_id);
    
    // Verify admin is still the same
    let stored_admin = client.get_admin();
    assert_eq!(stored_admin, admin);
    
    // Verify USDC token is still the same
    let stored_token = client.get_usdc_token();
    assert_eq!(stored_token, usdc_address);
}

#[test]
fn test_version_is_static() {
    // Create multiple test environments
    let (
        _env1, 
        _admin1, 
        _usdc_address1,
        _usdc_token1, 
        _usdc_token_client1, 
        _contract_id1, 
        client1
    ) = create_test_env();
    
    let (
        _env2, 
        _admin2, 
        _usdc_address2,
        _usdc_token2, 
        _usdc_token_client2, 
        _contract_id2, 
        client2
    ) = create_test_env();
    
    // Version should be the same across different contract instances
    let version1 = client1.version();
    let version2 = client2.version();
    
    assert_eq!(version1, version2);
    assert_eq!(version1, 1);
}

#[test]
fn test_upgrade_with_same_hash() {
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
    
    // Create a mock WASM hash
    let wasm_hash = create_mock_wasm_hash(&env);
    
    // First upgrade
    let result1 = client.upgrade(&wasm_hash);
    assert!(result1 == ());
    
    // Upgrade again with the same hash (should still succeed)
    let result2 = client.upgrade(&wasm_hash);
    assert!(result2 == ());
}
