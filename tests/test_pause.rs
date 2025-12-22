use devasign_task_escrow::Error;
use soroban_sdk::{testutils::Address as _, Address};

mod test_config;
mod test_setup;

use test_config::{TestConfig, TestValidation};
use test_setup::create_test_env;

#[test]
fn test_pause_unpause_by_admin() {
    let (_env, admin, usdc_address, _usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    // Pause contract
    let result = client.set_paused(&true);
    assert!(result == ());

    // Unpause contract
    let result = client.set_paused(&false);
    assert!(result == ());
}

#[test]
fn test_pause_unauthorized() {
    let (env, admin, usdc_address, _usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let _unauthorized = Address::generate(&env);
    env.mock_all_auths_allowing_non_root_auth();

    // Try to set pause from unauthorized address
    let result = client.mock_auths(&[]).try_set_paused(&mut true);
    assert!(result.is_err());
}

#[test]
fn test_operations_fail_when_paused() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "pause_test", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;

    // Fund creator
    usdc_token.mint(&creator, &bounty_amount);

    // Pause contract
    client.set_paused(&true);

    // 1. Try create_escrow
    let result = client.try_create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::ContractPaused);

    // Unpause to create setup
    client.set_paused(&false);
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );

    // Pause again
    client.set_paused(&true);

    // 2. Try increase_bounty
    let result = client.try_increase_bounty(&creator, &task_id, &TestConfig::SMALL_AMOUNT);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::ContractPaused);

    // 3. Try assign_contributor
    let result = client.try_assign_contributor(&task_id, &contributor);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::ContractPaused);

    // Unpause to assign
    client.set_paused(&false);
    client.assign_contributor(&task_id, &contributor);

    // Pause again
    client.set_paused(&true);

    // 4. Try approve_completion
    let result = client.try_approve_completion(&task_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::ContractPaused);

    // 5. Try dispute_task
    let reason = TestValidation::generate_dispute_reason(&env, "pause");
    let result = client.try_dispute_task(&creator, &task_id, &reason);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::ContractPaused);

    // 6. Try refund (need a new task for this as regular flow has contributor)
    client.set_paused(&false);
    let task_id_2 = TestValidation::generate_task_id(&env, "pause_refund", 2);
    usdc_token.mint(&creator, &bounty_amount);
    client.create_escrow(
        &creator,
        &task_id_2,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );
    client.set_paused(&true);

    let result = client.try_refund(&task_id_2);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::ContractPaused);
}
