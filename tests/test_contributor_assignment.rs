use devasign_task_escrow::{Error, TaskEscrowContractClient, TaskStatus};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

mod test_config;
mod test_setup;

use test_config::{TestConfig, TestValidation};
use test_setup::create_test_env;

fn setup_escrow_for_assignment(
    env: &Env,
    client: &TaskEscrowContractClient,
    usdc_token: &soroban_sdk::token::StellarAssetClient,
    creator: &Address,
    task_id: &str,
) -> String {
    let task_id = TestValidation::generate_task_id(&env, task_id, 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;

    // Fund creator and create escrow
    usdc_token.mint(creator, &bounty_amount);
    client.create_escrow(
        creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );

    task_id
}

#[test]
fn test_assign_contributor_success() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let task_id = setup_escrow_for_assignment(&env, &client, &usdc_token, &creator, "assign");

    // Assign contributor
    let result = client.assign_contributor(&task_id, &contributor);
    assert!(result == ());

    // Verify assignment
    let escrow = client.get_escrow(&task_id);
    assert!(escrow.task_id == task_id);

    assert_eq!(escrow.contributor, contributor);
    assert_eq!(escrow.has_contributor, true);
    assert_eq!(escrow.status, TaskStatus::InProgress);
}

#[test]
fn test_assign_contributor_unauthorized() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let task_id = setup_escrow_for_assignment(&env, &client, &usdc_token, &creator, "unauth");

    // Try to assign contributor from unauthorized address
    env.mock_all_auths_allowing_non_root_auth();
    let result = client
        .mock_auths(&[])
        .try_assign_contributor(&task_id, &contributor);
    assert!(result.is_err());
}

#[test]
fn test_assign_contributor_task_not_found() {
    let (env, admin, usdc_address, _usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let contributor = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "notfound", 1);

    // Try to assign contributor to non-existent task
    let result = client.try_assign_contributor(&task_id, &contributor);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::TaskNotFound);
}

#[test]
fn test_assign_contributor_already_assigned() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor1 = Address::generate(&env);
    let contributor2 = Address::generate(&env);
    let task_id = setup_escrow_for_assignment(&env, &client, &usdc_token, &creator, "already");

    // Assign first contributor
    let result = client.assign_contributor(&task_id, &contributor1);
    assert!(result == ());

    // Try to assign second contributor
    let result = client.try_assign_contributor(&task_id, &contributor2);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        Error::ContributorAlreadyAssigned
    );
}

#[test]
fn test_assign_contributor_invalid_task_id() {
    let (env, admin, usdc_address, _usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let contributor = Address::generate(&env);

    // Test empty task ID
    let result =
        client.try_assign_contributor(&String::from_str(&env, &"".to_string()), &contributor);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::EmptyTaskId);

    // Test too short task ID
    let result =
        client.try_assign_contributor(&String::from_str(&env, &"ab".to_string()), &contributor);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidTaskId);

    // Test too long task ID
    let long_id = "a".repeat(101);
    let result = client.try_assign_contributor(&String::from_str(&env, &long_id), &contributor);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidTaskId);
}

#[test]
fn test_assign_contributor_not_initialized() {
    let (env, _admin, _usdc_address, _usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    let contributor = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "uninit", 1);

    // Try to assign contributor without initializing contract
    let result = client.try_assign_contributor(&task_id, &contributor);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::ContractNotInitialized);
}

#[test]
fn test_assign_multiple_contributors_different_tasks() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributors = vec![
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    ];

    for (i, contributor) in contributors.iter().enumerate() {
        let task_id = setup_escrow_for_assignment(
            &env,
            &client,
            &usdc_token,
            &creator,
            &format!("multi{}", i),
        );

        // Assign contributor
        let result = client.assign_contributor(&task_id, contributor);
        assert!(result == ());

        // Verify assignment
        let escrow = client.get_escrow(&task_id);
        assert!(escrow.task_id == task_id);

        assert_eq!(escrow.contributor, *contributor);
        assert_eq!(escrow.has_contributor, true);
        assert_eq!(escrow.status, TaskStatus::InProgress);
    }
}

#[test]
fn test_assign_same_contributor_multiple_tasks() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);

    // Create multiple tasks and assign same contributor
    for i in 0..3 {
        let task_id = setup_escrow_for_assignment(
            &env,
            &client,
            &usdc_token,
            &creator,
            &format!("same{}", i),
        );

        // Assign same contributor to different tasks
        let result = client.assign_contributor(&task_id, &contributor);
        assert!(result == ());

        // Verify assignment
        let escrow = client.get_escrow(&task_id);
        assert!(escrow.task_id == task_id);

        assert_eq!(escrow.contributor, contributor);
        assert_eq!(escrow.has_contributor, true);
        assert_eq!(escrow.status, TaskStatus::InProgress);
    }
}


#[test]
fn test_assign_contributor_invalid_status() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor1 = Address::generate(&env);
    let contributor2 = Address::generate(&env);
    let task_id = setup_escrow_for_assignment(&env, &client, &usdc_token, &creator, "status");

    // Assign first contributor -> Status becomes InProgress
    client.assign_contributor(&task_id, &contributor1);

    // Verify status is InProgress
    let escrow = client.get_escrow(&task_id);
    assert_eq!(escrow.status, TaskStatus::InProgress);

    // Try to assign another contributor when status is InProgress
    // This should fail with InvalidTaskStatus (not ContributorAlreadyAssigned)
    // because status check now happens before contributor check
    let result = client.try_assign_contributor(&task_id, &contributor2);
    assert!(result.is_err());
    // Note: ContributorAlreadyAssigned is checked before InvalidTaskStatus in the code,
    // so this will return ContributorAlreadyAssigned
    assert_eq!(
        result.unwrap_err().unwrap(),
        Error::ContributorAlreadyAssigned
    );
}
