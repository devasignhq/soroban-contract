use devasign_task_escrow::{DisputeResolution, TaskStatus};
use soroban_sdk::{testutils::Address as _, Address};

mod test_config;
mod test_setup;

use test_config::{TestConfig, TestScenarios, TestValidation};
use test_setup::create_test_env;

#[test]
fn test_complete_happy_path_workflow() {
    let (env, admin, usdc_address, usdc_token, usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "happy_path", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;

    // Fund creator
    usdc_token.mint(&creator, &bounty_amount);

    // Complete workflow: Create → Assign → Approve → Payment

    // 1. Create escrow
    let result = client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );
    assert!(result == ());

    let escrow = client.get_escrow(&task_id);
    assert_eq!(escrow.status, TaskStatus::Open);
    assert_eq!(escrow.has_contributor, false);

    // 2. Assign contributor
    let result = client.assign_contributor(&task_id, &contributor);
    assert!(result == ());

    let escrow = client.get_escrow(&task_id);
    assert_eq!(escrow.status, TaskStatus::InProgress);
    assert_eq!(escrow.has_contributor, true);
    assert_eq!(escrow.contributor, contributor);

    // 3. Approve completion and release funds
    let initial_contributor_balance = usdc_token_client.balance(&contributor);

    let result = client.approve_completion(&task_id);
    assert!(result == ());

    // 4. Verify final state
    let escrow = client.get_escrow(&task_id);
    assert_eq!(escrow.status, TaskStatus::Completed);

    let final_contributor_balance = usdc_token_client.balance(&contributor);
    assert_eq!(
        final_contributor_balance,
        initial_contributor_balance + bounty_amount
    );

    // Verify contract balance is empty
    let contract_balance = client.get_contract_usdc_balance();
    assert_eq!(contract_balance, 0);
}

#[test]
fn test_complete_dispute_workflow_pay_contributor() {
    let (env, admin, usdc_address, usdc_token, usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "dispute_pay", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;

    // Setup task
    usdc_token.mint(&creator, &bounty_amount);
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );
    client.assign_contributor(&task_id, &contributor);

    // Dispute workflow: Dispute → Resolve (Pay Contributor)

    // 1. Create dispute
    let reason = TestValidation::generate_dispute_reason(&env, "quality");
    let result = client.dispute_task(&creator, &task_id, &reason);
    assert!(result == ());

    let escrow = client.get_escrow(&task_id);
    assert_eq!(escrow.status, TaskStatus::Disputed);

    let dispute_info = client.get_dispute_info(&task_id);
    assert_eq!(dispute_info.disputing_party, creator);
    assert_eq!(dispute_info.reason, reason);

    // 2. Resolve dispute - pay contributor
    let initial_contributor_balance = usdc_token_client.balance(&contributor);

    let result = client.resolve_dispute(&task_id, &DisputeResolution::PayContributor);
    assert!(result == ());

    // 3. Verify final state
    let escrow = client.get_escrow(&task_id);
    assert_eq!(escrow.status, TaskStatus::Completed);

    let final_contributor_balance = usdc_token_client.balance(&contributor);
    assert_eq!(
        final_contributor_balance,
        initial_contributor_balance + bounty_amount
    );
}

#[test]
fn test_complete_dispute_workflow_refund_creator() {
    let (env, admin, usdc_address, usdc_token, usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "dispute_refund", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;

    // Setup task
    usdc_token.mint(&creator, &bounty_amount);
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );
    client.assign_contributor(&task_id, &contributor);

    // Dispute workflow: Dispute → Resolve (Refund Creator)

    // 1. Create dispute
    let reason = TestValidation::generate_dispute_reason(&env, "scope");
    let result = client.dispute_task(&contributor, &task_id, &reason);
    assert!(result == ());

    // 2. Resolve dispute - refund creator
    let initial_creator_balance = usdc_token_client.balance(&creator);

    let result = client.resolve_dispute(&task_id, &DisputeResolution::RefundCreator);
    assert!(result == ());

    // 3. Verify final state
    let escrow = client.get_escrow(&task_id);
    assert_eq!(escrow.status, TaskStatus::Completed);

    let final_creator_balance = usdc_token_client.balance(&creator);
    assert_eq!(
        final_creator_balance,
        initial_creator_balance + bounty_amount
    );
}

#[test]
fn test_complete_dispute_workflow_partial_payment() {
    let (env, admin, usdc_address, usdc_token, usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "dispute_partial", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;

    // Setup task
    usdc_token.mint(&creator, &bounty_amount);
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );
    client.assign_contributor(&task_id, &contributor);

    // Dispute workflow: Dispute → Resolve (Partial Payment - 60% to contributor)

    // 1. Create dispute
    let reason = TestValidation::generate_dispute_reason(&env, "deadline");
    let result = client.dispute_task(&creator, &task_id, &reason);
    assert!(result == ());

    // 2. Resolve dispute - partial payment (60% to contributor, 40% to creator)
    let contributor_amount = (bounty_amount * 60) / 100;
    let creator_refund = bounty_amount - contributor_amount;

    let initial_contributor_balance = usdc_token_client.balance(&contributor);
    let initial_creator_balance = usdc_token_client.balance(&creator);

    let result = client.resolve_dispute(
        &task_id,
        &DisputeResolution::PartialPayment(contributor_amount),
    );
    assert!(result == ());

    // 3. Verify final state
    let escrow = client.get_escrow(&task_id);
    assert_eq!(escrow.status, TaskStatus::Completed);

    let final_contributor_balance = usdc_token_client.balance(&contributor);
    let final_creator_balance = usdc_token_client.balance(&creator);

    assert_eq!(
        final_contributor_balance,
        initial_contributor_balance + contributor_amount
    );
    assert_eq!(
        final_creator_balance,
        initial_creator_balance + creator_refund
    );
}

#[test]
fn test_refund_workflow() {
    let (env, admin, usdc_address, usdc_token, usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "refund_flow", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;

    // Refund workflow: Create → (Optional: Assign) → Refund

    // 1. Create escrow
    usdc_token.mint(&creator, &bounty_amount);
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );

    let escrow = client.get_escrow(&task_id);
    assert_eq!(escrow.status, TaskStatus::Open);

    // 2. Process refund
    let initial_creator_balance = usdc_token_client.balance(&creator);

    let result = client.refund(&task_id);
    assert!(result == ());

    // 3. Verify final state
    let escrow = client.get_escrow(&task_id);
    assert_eq!(escrow.status, TaskStatus::Cancelled);

    let final_creator_balance = usdc_token_client.balance(&creator);
    assert_eq!(
        final_creator_balance,
        initial_creator_balance + bounty_amount
    );
}

#[test]
fn test_multi_user_concurrent_tasks() {
    let (env, admin, usdc_address, usdc_token, usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    // Create multiple creators and contributors
    let creators: Vec<Address> = (0..3).map(|_| Address::generate(&env)).collect();
    let contributors: Vec<Address> = (0..3).map(|_| Address::generate(&env)).collect();
    let bounty_amount = TestConfig::SMALL_AMOUNT;

    // Create multiple concurrent tasks
    let mut task_ids = Vec::new();
    for (i, creator) in creators.iter().enumerate() {
        let task_id = TestValidation::generate_task_id(&env, "concurrent", i as u32);

        // Fund creator and create escrow
        usdc_token.mint(creator, &bounty_amount);
        client.create_escrow(
            creator,
            &task_id,
            &TestValidation::dummy_issue_url(&env),
            &bounty_amount,
        );

        // Assign different contributors
        client.assign_contributor(&task_id, &contributors[i]);

        task_ids.push(task_id);
    }

    // Approve all tasks and verify payments
    for (i, task_id) in task_ids.iter().enumerate() {
        let initial_balance = usdc_token_client.balance(&contributors[i]);

        client.approve_completion(&task_id);

        let final_balance = usdc_token_client.balance(&contributors[i]);
        assert_eq!(final_balance, initial_balance + bounty_amount);

        let escrow = client.get_escrow(task_id);
        assert_eq!(escrow.status, TaskStatus::Completed);
    }
}

#[test]
fn test_mixed_workflow_scenarios() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let bounty_amount = TestConfig::SMALL_AMOUNT;

    // Scenario 1: Happy path
    let task_id_1 = TestValidation::generate_task_id(&env, "mixed_happy", 1);
    usdc_token.mint(&creator, &bounty_amount);
    client.create_escrow(
        &creator,
        &task_id_1,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );
    client.assign_contributor(&task_id_1, &contributor);
    client.approve_completion(&task_id_1);

    // Scenario 2: Refund
    let task_id_2 = TestValidation::generate_task_id(&env, "mixed_refund", 2);
    usdc_token.mint(&creator, &bounty_amount);
    client.create_escrow(
        &creator,
        &task_id_2,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );
    client.refund(&task_id_2);

    // Scenario 3: Dispute resolution
    let task_id_3 = TestValidation::generate_task_id(&env, "mixed_dispute", 3);
    usdc_token.mint(&creator, &bounty_amount);
    client.create_escrow(
        &creator,
        &task_id_3,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );
    client.assign_contributor(&task_id_3, &contributor);

    let reason = TestValidation::generate_dispute_reason(&env, "quality");
    client.dispute_task(&creator, &task_id_3, &reason);
    client.resolve_dispute(&task_id_3, &DisputeResolution::PayContributor);

    // Verify all scenarios completed successfully
    let escrow_1 = client.get_escrow(&task_id_1);
    let escrow_2 = client.get_escrow(&task_id_2);
    let escrow_3 = client.get_escrow(&task_id_3);

    assert_eq!(escrow_1.status, TaskStatus::Completed);
    assert_eq!(escrow_2.status, TaskStatus::Cancelled);
    assert_eq!(escrow_3.status, TaskStatus::Completed);
}

#[test]
fn test_large_scale_operations() {
    let (env, admin, usdc_address, usdc_token, usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let task_count = 10;
    let bounty_amount = TestConfig::MIN_VALID_AMOUNT;

    // Create many tasks
    let mut task_ids = Vec::new();
    for i in 0..task_count {
        let task_id = TestValidation::generate_task_id(&env, "large_scale", i);

        usdc_token.mint(&creator, &bounty_amount);
        client.create_escrow(
            &creator,
            &task_id,
            &TestValidation::dummy_issue_url(&env),
            &bounty_amount,
        );
        client.assign_contributor(&task_id, &contributor);

        task_ids.push(task_id);
    }

    // Approve all tasks
    let initial_balance = usdc_token_client.balance(&contributor);

    for task_id in &task_ids {
        client.approve_completion(task_id);
    }

    // Verify total payment
    let final_balance = usdc_token_client.balance(&contributor);
    let expected_total = bounty_amount * task_count as i128;
    assert_eq!(final_balance, initial_balance + expected_total);
}

#[test]
fn test_workflow_with_different_amounts() {
    let (env, admin, usdc_address, usdc_token, usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let amounts = TestScenarios::standard_amounts();

    let mut total_expected = 0i128;

    // Create tasks with different amounts
    for (i, &amount) in amounts.iter().enumerate() {
        let task_id = TestValidation::generate_task_id(&env, "diff_amounts", i as u32);

        usdc_token.mint(&creator, &amount);
        client.create_escrow(
            &creator,
            &task_id,
            &TestValidation::dummy_issue_url(&env),
            &amount,
        );
        client.assign_contributor(&task_id, &contributor);
        client.approve_completion(&task_id);

        total_expected += amount;
    }

    // Verify total payments
    let final_balance = usdc_token_client.balance(&contributor);
    assert_eq!(final_balance, total_expected);

    // Verify all escrows are resolved
    for (i, _) in amounts.iter().enumerate() {
        let task_id = TestValidation::generate_task_id(&env, "diff_amounts", i as u32);
        let escrow = client.get_escrow(&task_id);
        assert_eq!(escrow.status, TaskStatus::Completed);
    }
}

#[test]
fn test_admin_operations_workflow() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    // Verify initial admin
    let current_admin = client.get_admin();
    assert_eq!(current_admin, admin);

    // Create a task and dispute it
    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "admin_ops", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;

    usdc_token.mint(&creator, &bounty_amount);
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );
    client.assign_contributor(&task_id, &contributor);

    let reason = TestValidation::generate_dispute_reason(&env, "quality");
    client.dispute_task(&creator, &task_id, &reason);

    // Admin resolves dispute
    client.resolve_dispute(
        &task_id,
        &DisputeResolution::PartialPayment(bounty_amount / 2),
    );

    // Change admin
    let new_admin = Address::generate(&env);
    client.set_admin(&new_admin);

    // Verify new admin
    let updated_admin = client.get_admin();
    assert_eq!(updated_admin, new_admin);

    // Verify dispute was resolved
    let escrow = client.get_escrow(&task_id);
    assert_eq!(escrow.status, TaskStatus::Completed);
}
