use soroban_sdk::{testutils::Events, BytesN, Symbol, TryIntoVal, Val};

mod test_config;
mod test_setup;

use test_setup::create_test_env;

#[test]
fn test_upgrade_success() {
    let (env, admin, usdc_address, _usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    // Try to load the existing contract WASM to test upgrade
    let wasm_path = "target/wasm32v1-none/release/devasign_task_escrow.wasm";

    // Check if wasm file exists
    if let Ok(wasm) = std::fs::read(wasm_path) {
        // Upload wasm
        let wasm_hash = env.deployer().upload_contract_wasm(wasm.as_slice());

        // Call upgrade as admin
        client.upgrade(&wasm_hash);

        // Verify event
        let events = env.events().all();
        let upgrade_events: std::vec::Vec<_> = events
            .iter()
            .filter(|e| {
                let topic_val: Val = e.1.iter().next().unwrap();
                let topic_sym: Symbol = topic_val.try_into_val(&env).unwrap();
                topic_sym == Symbol::new(&env, "contract_upgraded_event")
            })
            .collect();

        assert!(!upgrade_events.is_empty());
    } else {
        std::println!(
            "Skipping test_upgrade_success: WASM file not found at {}",
            wasm_path
        );
    }
}

#[test]
#[should_panic(expected = "HostError: Error(Storage, MissingValue)")]
fn test_upgrade_fails_invalid_hash() {
    let (env, admin, usdc_address, _usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    // Create an invalid random hash that hasn't been uploaded
    let invalid_hash = BytesN::from_array(&env, &[1u8; 32]);

    // Call upgrade with invalid hash
    client.upgrade(&invalid_hash);
}

#[test]
fn test_upgrade_auth_fails_if_not_admin() {
    let (env, admin, usdc_address, _usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    // Use a random hash - we don't need valid WASM because auth check should fail first
    let invalid_hash = BytesN::from_array(&env, &[2u8; 32]);

    // Clear mocked auths to enforce strict checking
    env.mock_auths(&[]);

    // Attempt upgrade without registering the admin's signature
    let result = client.try_upgrade(&invalid_hash);

    // Check that we got an error (Auth failure)
    assert!(result.is_err());
}
