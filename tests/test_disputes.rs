use devasign_task_escrow::{DisputeResolution, Error, TaskEscrowContractClient, TaskStatus};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

mod test_config;
mod test_setup;

use test_config::{TestConfig, TestScenarios, TestValidation};
use test_setup::create_test_env;

fn setup_task_for_dispute(
    env: &Env,
    client: &TaskEscrowContractClient,
    usdc_token: &soroban_sdk::token::StellarAssetClient,
    creator: &Address,
    contributor: &Address,
    task_prefix: &str,
    _complete_task: bool, // parameter kept for signature compatibility but ignored
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

    // Assign contributor
    client.assign_contributor(&task_id, contributor);

    (task_id, bounty_amount)
}

#[test]
fn test_dispute_task_by_creator() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let (task_id, _) = setup_task_for_dispute(
        &env,
        &client,
        &usdc_token,
        &creator,
        &contributor,
        "creator_dispute",
        true,
    );

    let reason = TestValidation::generate_dispute_reason(&env, "quality");

    // Dispute task
    let result = client.dispute_task(&creator, &task_id, &reason);
    assert!(result == ());

    // Verify dispute info
    let dispute = client.get_dispute_info(&task_id);
    assert!(dispute.task_id == task_id);

    assert_eq!(dispute.disputing_party, creator);
    assert_eq!(dispute.reason, reason);
}

#[test]
fn test_dispute_task_by_contributor() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let (task_id, _) = setup_task_for_dispute(
        &env,
        &client,
        &usdc_token,
        &creator,
        &contributor,
        "contrib_dispute",
        true,
    );

    let reason = TestValidation::generate_dispute_reason(&env, "payment");

    // Dispute task
    let result = client.dispute_task(&contributor, &task_id, &reason);
    assert!(result == ());

    // Verify dispute info
    let dispute = client.get_dispute_info(&task_id);
    assert!(dispute.task_id == task_id);

    assert_eq!(dispute.disputing_party, contributor);
    assert_eq!(dispute.reason, reason);
}

#[test]
fn test_dispute_task_unauthorized() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let (task_id, _) = setup_task_for_dispute(
        &env,
        &client,
        &usdc_token,
        &creator,
        &contributor,
        "unauth_dispute",
        true,
    );

    let reason = TestValidation::generate_dispute_reason(&env, "quality");

    // Try to dispute from unauthorized address
    let result = client.try_dispute_task(&unauthorized, &task_id, &reason);
    assert!(result.is_err());
}

#[test]
fn test_dispute_task_invalid_reason() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let (task_id, _) = setup_task_for_dispute(
        &env,
        &client,
        &usdc_token,
        &creator,
        &contributor,
        "invalid_reason",
        true,
    );

    // Test empty reason
    let result =
        client.try_dispute_task(&creator, &task_id, &String::from_str(&env, &"".to_string()));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidDisputeReason);

    // Test too short reason
    let result = client.try_dispute_task(
        &creator,
        &task_id,
        &String::from_str(&env, &"short".to_string()),
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidDisputeReason);

    // Test too long reason
    let long_reason = "a".repeat(501);
    let result = client.try_dispute_task(&creator, &task_id, &String::from_str(&env, &long_reason));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidDisputeReason);
}

#[test]
fn test_resolve_dispute_pay_contributor() {
    let (env, admin, usdc_address, usdc_token, usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let (task_id, bounty_amount) = setup_task_for_dispute(
        &env,
        &client,
        &usdc_token,
        &creator,
        &contributor,
        "resolve_pay",
        true,
    );

    // Create dispute
    let reason = TestValidation::generate_dispute_reason(&env, "quality");
    client.dispute_task(&creator, &task_id, &reason);

    // Get initial balances
    let initial_contributor_balance = usdc_token_client.balance(&contributor);

    // Resolve dispute - pay contributor
    let result = client.resolve_dispute(&task_id, &DisputeResolution::PayContributor);
    assert!(result == ());

    // Verify payment
    let final_contributor_balance = usdc_token_client.balance(&contributor);
    assert_eq!(
        final_contributor_balance,
        initial_contributor_balance + bounty_amount
    );

    // Verify escrow status
    let escrow = client.get_escrow(&task_id);
    assert_eq!(escrow.status, TaskStatus::Completed);
}

#[test]
fn test_resolve_dispute_refund_creator() {
    let (env, admin, usdc_address, usdc_token, usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let (task_id, bounty_amount) = setup_task_for_dispute(
        &env,
        &client,
        &usdc_token,
        &creator,
        &contributor,
        "resolve_refund",
        true,
    );

    // Create dispute
    let reason = TestValidation::generate_dispute_reason(&env, "scope");
    client.dispute_task(&contributor, &task_id, &reason);

    // Get initial balances
    let initial_creator_balance = usdc_token_client.balance(&creator);

    // Resolve dispute - refund creator
    let result = client.resolve_dispute(&task_id, &DisputeResolution::RefundCreator);
    assert!(result == ());

    // Verify refund
    let final_creator_balance = usdc_token_client.balance(&creator);
    assert_eq!(
        final_creator_balance,
        initial_creator_balance + bounty_amount
    );

    // Verify escrow status
    let escrow = client.get_escrow(&task_id);
    assert_eq!(escrow.status, TaskStatus::Completed);
}

#[test]
fn test_resolve_dispute_partial_payment() {
    let (env, admin, usdc_address, usdc_token, usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let (task_id, bounty_amount) = setup_task_for_dispute(
        &env,
        &client,
        &usdc_token,
        &creator,
        &contributor,
        "resolve_partial",
        true,
    );

    // Create dispute
    let reason = TestValidation::generate_dispute_reason(&env, "deadline");
    client.dispute_task(&creator, &task_id, &reason);

    // Get initial balances
    let initial_creator_balance = usdc_token_client.balance(&creator);
    let initial_contributor_balance = usdc_token_client.balance(&contributor);

    // Resolve dispute - 60% to contributor, 40% to creator
    let partial_amount = (bounty_amount * 60) / 100;
    let result =
        client.resolve_dispute(&task_id, &DisputeResolution::PartialPayment(partial_amount));
    assert!(result == ());

    // Verify payments
    let final_creator_balance = usdc_token_client.balance(&creator);
    let final_contributor_balance = usdc_token_client.balance(&contributor);

    let refund_amount = bounty_amount - partial_amount;
    assert_eq!(
        final_contributor_balance,
        initial_contributor_balance + partial_amount
    );
    assert_eq!(
        final_creator_balance,
        initial_creator_balance + refund_amount
    );

    // Verify escrow status
    let escrow = client.get_escrow(&task_id);
    assert_eq!(escrow.status, TaskStatus::Completed);
}

#[test]
fn test_resolve_dispute_unauthorized() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let (task_id, _) = setup_task_for_dispute(
        &env,
        &client,
        &usdc_token,
        &creator,
        &contributor,
        "unauth_resolve",
        true,
    );

    // Create dispute
    let reason = TestValidation::generate_dispute_reason(&env, "quality");
    client.dispute_task(&creator, &task_id, &reason);

    // Try to resolve from unauthorized address
    env.mock_all_auths_allowing_non_root_auth();
    let result = client
        .mock_auths(&[])
        .try_resolve_dispute(&task_id, &DisputeResolution::PayContributor);
    assert!(result.is_err());
}

#[test]
fn test_resolve_dispute_not_disputed() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let (task_id, _) = setup_task_for_dispute(
        &env,
        &client,
        &usdc_token,
        &creator,
        &contributor,
        "not_disputed",
        true,
    );

    // Try to resolve without dispute
    let result = client.try_resolve_dispute(&task_id, &DisputeResolution::PayContributor);
    assert!(result.is_err());
}

#[test]
fn test_get_dispute_info_not_found() {
    let (env, admin, usdc_address, _usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let task_id = TestValidation::generate_task_id(&env, "notfound", 1);

    // Try to get dispute info for non-existent task
    let result = client.try_get_dispute_info(&task_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::TaskNotFound);
}

#[test]
fn test_get_dispute_info_not_disputed() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let (task_id, _) = setup_task_for_dispute(
        &env,
        &client,
        &usdc_token,
        &creator,
        &contributor,
        "no_dispute",
        true,
    );

    // Try to get dispute info for non-disputed task
    let result = client.try_get_dispute_info(&task_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::TaskNotDisputed);
}

#[test]
fn test_dispute_resolution_scenarios() {
    let (env, admin, usdc_address, usdc_token, usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let resolutions = TestScenarios::dispute_resolutions();

    for (i, (_, resolution)) in resolutions.iter().enumerate() {
        let creator = Address::generate(&env);
        let contributor = Address::generate(&env);
        let (task_id, bounty_amount) = setup_task_for_dispute(
            &env,
            &client,
            &usdc_token,
            &creator,
            &contributor,
            &format!("scenario_{}", i),
            true,
        );

        // Create dispute
        let reason = TestValidation::generate_dispute_reason(&env, "quality");
        client.dispute_task(&creator, &task_id, &reason);

        // Get initial balances
        let initial_creator_balance = usdc_token_client.balance(&creator);
        let initial_contributor_balance = usdc_token_client.balance(&contributor);

        // Resolve dispute
        let result = client.resolve_dispute(&task_id, resolution);
        assert!(result == ());

        // Verify final balances based on resolution type
        let final_creator_balance = usdc_token_client.balance(&creator);
        let final_contributor_balance = usdc_token_client.balance(&contributor);

        match resolution {
            DisputeResolution::PayContributor => {
                assert_eq!(
                    final_contributor_balance,
                    initial_contributor_balance + bounty_amount
                );
                assert_eq!(final_creator_balance, initial_creator_balance);
            }
            DisputeResolution::RefundCreator => {
                assert_eq!(
                    final_creator_balance,
                    initial_creator_balance + bounty_amount
                );
                assert_eq!(final_contributor_balance, initial_contributor_balance);
            }
            DisputeResolution::PartialPayment(partial_amount) => {
                let refund_amount = bounty_amount - partial_amount;
                assert_eq!(
                    final_contributor_balance,
                    initial_contributor_balance + partial_amount
                );
                assert_eq!(
                    final_creator_balance,
                    initial_creator_balance + refund_amount
                );
            }
        }

        // Verify escrow status
        let escrow = client.get_escrow(&task_id);
        assert_eq!(escrow.status, TaskStatus::Completed);
    }
}
