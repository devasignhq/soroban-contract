use devasign_task_escrow::Error;
use soroban_sdk::{testutils::Address as _, Address};

mod test_config;
mod test_setup;

use test_config::{TestConfig, TestValidation};
use test_setup::create_test_env;

#[test]
fn test_increase_bounty_success() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    // Create test data
    let creator = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "test", 1);
    let initial_bounty = TestConfig::MEDIUM_AMOUNT;
    let increase_amount = 1_000_000; // 0.1 USDC

    // Fund creator with USDC (enough for initial + increase)
    usdc_token.mint(&creator, &(initial_bounty + increase_amount));

    // Create escrow
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &initial_bounty,
    );

    // Increase bounty
    let result = client.increase_bounty(&creator, &task_id, &increase_amount);
    assert!(result == ());

    // Verify escrow updated
    let escrow = client.get_escrow(&task_id);
    assert_eq!(escrow.bounty_amount, initial_bounty + increase_amount);

    // Verify balances
    // Contract should have initial + increase
    assert_eq!(
        usdc_token.balance(&contract_id),
        initial_bounty + increase_amount
    );
    // Creator should have 0 (since we minted exactly what was needed)
    assert_eq!(usdc_token.balance(&creator), 0);
}

#[test]
fn test_decrease_bounty_success() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    // Create test data
    let creator = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "test", 1);
    let initial_bounty = TestConfig::MEDIUM_AMOUNT;
    let decrease_amount = 1_000_000; // 0.1 USDC

    // Fund creator with USDC
    usdc_token.mint(&creator, &initial_bounty);

    // Create escrow
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &initial_bounty,
    );

    // Decrease bounty
    let result = client.decrease_bounty(&creator, &task_id, &decrease_amount);
    assert!(result == ());

    // Verify escrow updated
    let escrow = client.get_escrow(&task_id);
    assert_eq!(escrow.bounty_amount, initial_bounty - decrease_amount);

    // Verify balances
    // Contract should have initial - decrease
    assert_eq!(
        usdc_token.balance(&contract_id),
        initial_bounty - decrease_amount
    );
    // Creator should have decrease_amount back
    assert_eq!(usdc_token.balance(&creator), decrease_amount);
}

#[test]
fn test_increase_bounty_invalid_amount() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "test", 1);
    let initial_bounty = TestConfig::MEDIUM_AMOUNT;

    usdc_token.mint(&creator, &initial_bounty);
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &initial_bounty,
    );

    // Try 0 amount
    let result = client.try_increase_bounty(&creator, &task_id, &0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidAmount);

    // Try negative amount
    let result = client.try_increase_bounty(&creator, &task_id, &-100);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidAmount);
}

#[test]
fn test_decrease_bounty_invalid_amount() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "test", 1);
    let initial_bounty = TestConfig::MEDIUM_AMOUNT;

    usdc_token.mint(&creator, &initial_bounty);
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &initial_bounty,
    );

    // Try 0 amount
    let result = client.try_decrease_bounty(&creator, &task_id, &0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidAmount);

    // Try negative amount
    let result = client.try_decrease_bounty(&creator, &task_id, &-100);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidAmount);
}

#[test]
fn test_decrease_bounty_insufficient() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "test", 1);
    let initial_bounty = 1_000_000; // 0.1 USDC

    usdc_token.mint(&creator, &initial_bounty);
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &initial_bounty,
    );

    // Try to decrease by more than available
    let result = client.try_decrease_bounty(&creator, &task_id, &(initial_bounty + 1));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidAmount);

    // Try to decrease by exact amount (should fail as it would leave 0, use refund instead)
    let result = client.try_decrease_bounty(&creator, &task_id, &initial_bounty);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidAmount);
}

#[test]
fn test_increase_bounty_not_creator() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let other = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "test", 1);
    let initial_bounty = TestConfig::MEDIUM_AMOUNT;

    usdc_token.mint(&creator, &initial_bounty);
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &initial_bounty,
    );

    // Fund other user
    usdc_token.mint(&other, &TestConfig::MEDIUM_AMOUNT);

    let result = client.try_increase_bounty(&other, &task_id, &1_000_000);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::NotTaskCreator);
}

#[test]
fn test_decrease_bounty_not_creator() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let other = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "test", 1);
    let initial_bounty = TestConfig::MEDIUM_AMOUNT;

    usdc_token.mint(&creator, &initial_bounty);
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &initial_bounty,
    );

    let result = client.try_decrease_bounty(&other, &task_id, &1_000_000);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::NotTaskCreator);
}

#[test]
fn test_increase_bounty_task_not_found() {
    let (env, admin, usdc_address, _usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "notfound", 1);

    let result = client.try_increase_bounty(&creator, &task_id, &1_000_000);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::TaskNotFound);
}

#[test]
fn test_decrease_bounty_task_not_found() {
    let (env, admin, usdc_address, _usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "notfound", 1);

    let result = client.try_decrease_bounty(&creator, &task_id, &1_000_000);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::TaskNotFound);
}

#[test]
fn test_decrease_bounty_invalid_status() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "test", 1);
    let initial_bounty = TestConfig::MEDIUM_AMOUNT;

    usdc_token.mint(&creator, &initial_bounty);
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &initial_bounty,
    );

    // Assign contributor -> Status becomes InProgress
    client.assign_contributor(&task_id, &contributor);

    // Attempt to decrease bounty - should fail for InProgress
    let result = client.try_decrease_bounty(&creator, &task_id, &1_000_000);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidTaskStatus);
}


#[test]
fn test_increase_bounty_invalid_status() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "test", 1);
    let initial_bounty = TestConfig::MEDIUM_AMOUNT;
    let increase_amount = 1_000_000; // 0.1 USDC

    // Fund creator with enough for initial + potential increase
    usdc_token.mint(&creator, &(initial_bounty + increase_amount));
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &initial_bounty,
    );

    // Assign contributor -> Status becomes InProgress
    client.assign_contributor(&task_id, &contributor);

    // Attempt to increase bounty - should fail for InProgress
    let result = client.try_increase_bounty(&creator, &task_id, &increase_amount);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidTaskStatus);
}
