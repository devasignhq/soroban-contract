use devasign_task_escrow::{Error, TaskEscrowContractClient, TaskStatus};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

mod test_config;
mod test_setup;

use test_config::{TestConfig, TestValidation};
use test_setup::create_test_env;

fn setup_completed_task(
    env: &Env,
    client: &TaskEscrowContractClient,
    usdc_token: &soroban_sdk::token::StellarAssetClient,
    creator: &Address,
    contributor: &Address,
    task_prefix: &str,
) -> (String, i128) {
    let task_id = TestValidation::generate_task_id(&env, task_prefix, 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;

    // Fund creator and create escrow
    usdc_token.mint(creator, &bounty_amount);
    client.create_escrow(
        creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );

    // Assign contributor and complete task
    // Assign contributor
    client.assign_contributor(&task_id, contributor);

    (task_id, bounty_amount)
}

#[test]
fn test_approve_completion_success() {
    let (env, admin, usdc_address, usdc_token, usdc_token_client, contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let (task_id, bounty_amount) = setup_completed_task(
        &env,
        &client,
        &usdc_token,
        &creator,
        &contributor,
        "approve",
    );

    // Get initial balances
    let initial_contributor_balance = usdc_token_client.balance(&contributor);
    let initial_contract_balance = usdc_token_client.balance(&contract_id);

    // Approve completion
    let result = client.approve_completion(&task_id);
    assert!(result == ());

    // Verify payment transfer
    let final_contributor_balance = usdc_token_client.balance(&contributor);
    let final_contract_balance = usdc_token_client.balance(&contract_id);

    assert_eq!(
        final_contributor_balance,
        initial_contributor_balance + bounty_amount
    );
    assert_eq!(
        final_contract_balance,
        initial_contract_balance - bounty_amount
    );

    // Verify escrow status
    let escrow = client.get_escrow(&task_id);
    assert!(escrow.task_id == task_id);

    assert_eq!(escrow.status, TaskStatus::Completed);
}

#[test]
fn test_approve_completion_unauthorized() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let (task_id, _) =
        setup_completed_task(&env, &client, &usdc_token, &creator, &contributor, "unauth");

    // Try to approve from unauthorized address
    env.mock_all_auths_allowing_non_root_auth();
    let result = client.mock_auths(&[]).try_approve_completion(&task_id);
    assert!(result.is_err());
}

#[test]
fn test_approve_completion_task_not_found() {
    let (env, admin, usdc_address, _usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let task_id = TestValidation::generate_task_id(&env, "notfound", 1);

    // Try to approve non-existent task
    let result = client.try_approve_completion(&task_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::TaskNotFound);
}

#[test]
fn test_approve_completion_works_directly() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "notcomplete", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;

    // Create escrow and assign contributor
    usdc_token.mint(&creator, &bounty_amount);
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );
    client.assign_contributor(&task_id, &contributor);

    // Try to approve. Works now because InProgress is the correct state
    let result = client.approve_completion(&task_id);
    assert!(result == ());
}

#[test]
fn test_approve_completion_no_contributor() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "nocontrib", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;

    // Create escrow without contributor
    usdc_token.mint(&creator, &bounty_amount);
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );

    // Try to approve without contributor
    let result = client.try_approve_completion(&task_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidTaskStatus);
}

#[test]
fn test_approve_completion_already_resolved() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let (task_id, _) = setup_completed_task(
        &env,
        &client,
        &usdc_token,
        &creator,
        &contributor,
        "resolved",
    );

    // Approve completion first time
    let result = client.approve_completion(&task_id);
    assert!(result == ());

    // Try to approve again
    let result = client.try_approve_completion(&task_id);
    // Should fail because status is now Completed, and approve requires InProgress
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidTaskStatus);
}

#[test]
fn test_approve_completion_multiple_amounts() {
    let (env, admin, usdc_address, usdc_token, usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let amounts = vec![
        TestConfig::MIN_VALID_AMOUNT,
        TestConfig::SMALL_AMOUNT,
        TestConfig::MEDIUM_AMOUNT,
        TestConfig::LARGE_AMOUNT,
    ];

    for (i, &amount) in amounts.iter().enumerate() {
        let task_id = TestValidation::generate_task_id(&env, &format!("amount{}", i), 1);

        // Create, assign, complete, and approve task
        usdc_token.mint(&creator, &amount);
        client.create_escrow(
            &creator,
            &task_id,
            &TestValidation::dummy_issue_url(&env),
            &amount,
        );
        client.assign_contributor(&task_id, &contributor);

        let initial_balance = usdc_token_client.balance(&contributor);

        let result = client.approve_completion(&task_id);
        assert!(result == ());

        let final_balance = usdc_token_client.balance(&contributor);
        assert_eq!(final_balance, initial_balance + amount);

        // Verify escrow status
        let escrow = client.get_escrow(&task_id);
        assert!(escrow.task_id == task_id);
        assert_eq!(escrow.status, TaskStatus::Completed);
    }
}

#[test]
fn test_approve_completion_invalid_task_id() {
    let (env, admin, usdc_address, _usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    // Test empty task ID
    let result = client.try_approve_completion(&String::from_str(&env, &"".to_string()));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::EmptyTaskId);

    // Test too short task ID
    let result = client.try_approve_completion(&String::from_str(&env, &"ab".to_string()));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidTaskId);

    // Test too long task ID
    let long_id = "a".repeat(101);
    let result = client.try_approve_completion(&String::from_str(&env, &long_id));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidTaskId);
}

#[test]
fn test_approve_completion_not_initialized() {
    let (env, _admin, _usdc_address, _usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    let task_id = TestValidation::generate_task_id(&env, "uninit", 1);

    // Try to approve without initializing contract
    let result = client.try_approve_completion(&task_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::ContractNotInitialized);
}

#[test]
fn test_get_contract_usdc_balance() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);

    // Initial balance should be 0
    let initial_balance = client.get_contract_usdc_balance();
    assert!(initial_balance == 0);

    // Create escrow (funds should be in contract)
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;
    usdc_token.mint(&creator, &bounty_amount);
    client.create_escrow(
        &creator,
        &TestValidation::generate_task_id(&env, "balance", 1),
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );

    let balance_after_escrow = client.get_contract_usdc_balance();
    assert!(balance_after_escrow > 0);
    assert_eq!(balance_after_escrow, bounty_amount);
}

#[test]
fn test_has_sufficient_usdc_balance() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let user = Address::generate(&env);
    let amount = TestConfig::MEDIUM_AMOUNT;

    // Check insufficient balance
    let result = client.has_sufficient_usdc_balance(&user, &amount);
    assert!(result == false);

    // Fund user
    usdc_token.mint(&user, &amount);

    // Check sufficient balance
    let result = client.has_sufficient_usdc_balance(&user, &amount);
    assert!(result == true);

    // Check more than balance
    let result = client.has_sufficient_usdc_balance(&user, &(amount + 1));
    assert!(result == false);
}

#[test]
fn test_get_usdc_balance() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let user = Address::generate(&env);
    let amount = TestConfig::MEDIUM_AMOUNT;

    // Check initial balance (should be 0)
    let balance = client.get_usdc_balance(&user);
    assert!(balance == 0);

    // Fund user
    usdc_token.mint(&user, &amount);

    // Check balance after funding
    let balance = client.get_usdc_balance(&user);
    assert!(balance == amount);
}
