use devasign_task_escrow::{DisputeResolution, Error, TaskStatus};
use soroban_sdk::testutils::Ledger;
use soroban_sdk::{testutils::Address as _, Address, String};

mod test_config;
mod test_setup;

use test_config::{TestAssertions, TestConfig, TestValidation};
use test_setup::create_test_env;

// ============================================================================
// PERFORMANCE TESTS
// ============================================================================

#[test]
fn test_gas_usage_escrow_creation() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;

    // Measure gas usage for escrow creation
    let mut gas_measurements = Vec::new();

    for i in 0..TestConfig::SMALL_BENCHMARK_SIZE {
        let task_id = TestValidation::generate_task_id(&env, "gas_test", i);

        usdc_token.mint(&creator, &bounty_amount);

        // Reset ledger for consistent measurement
        env.ledger().set_sequence_number(100 + i as u32);

        let initial_gas = env.cost_estimate().budget().cpu_instruction_cost();
        client.create_escrow(
            &creator,
            &task_id,
            &TestValidation::dummy_issue_url(&env),
            &bounty_amount,
        );
        let final_gas = env.cost_estimate().budget().cpu_instruction_cost();

        gas_measurements.push(final_gas - initial_gas);
    }

    // Verify gas usage is reasonable and consistent
    let max_expected_gas = 1_000_000u64; // Adjust based on actual measurements
    for &gas_used in &gas_measurements {
        TestAssertions::assert_reasonable_gas_usage(gas_used, max_expected_gas);
    }

    // Check performance consistency (coefficient of variation < 20%)
    TestAssertions::assert_performance_consistency(&gas_measurements, 20.0);
}

#[test]
fn test_gas_usage_complete_workflow() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let bounty_amount = TestConfig::SMALL_AMOUNT;

    let mut workflow_gas_measurements = Vec::new();

    for i in 0..TestConfig::SMALL_BENCHMARK_SIZE {
        let task_id = TestValidation::generate_task_id(&env, "workflow_gas", i);

        usdc_token.mint(&creator, &bounty_amount);

        let initial_gas = env.cost_estimate().budget().cpu_instruction_cost();

        // Complete workflow
        client.create_escrow(
            &creator,
            &task_id,
            &TestValidation::dummy_issue_url(&env),
            &bounty_amount,
        );
        client.assign_contributor(&task_id, &contributor);
        client.approve_completion(&task_id);

        let final_gas = env.cost_estimate().budget().cpu_instruction_cost();
        workflow_gas_measurements.push(final_gas - initial_gas);
    }

    // Verify workflow gas usage
    let max_expected_workflow_gas = 5_000_000u64; // Adjust based on actual measurements
    for &gas_used in &workflow_gas_measurements {
        TestAssertions::assert_reasonable_gas_usage(gas_used, max_expected_workflow_gas);
    }

    TestAssertions::assert_performance_consistency(&workflow_gas_measurements, 25.0);
}

#[test]
fn test_performance_large_amounts() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let large_amounts = vec![
        TestConfig::LARGE_AMOUNT,
        TestConfig::HUGE_AMOUNT,
        TestConfig::MAX_VALID_AMOUNT,
    ];

    let mut large_amount_gas = Vec::new();

    for (i, &amount) in large_amounts.iter().enumerate() {
        let task_id = TestValidation::generate_task_id(&env, "large_amount", i as u32);

        usdc_token.mint(&creator, &amount);

        let initial_gas = env.cost_estimate().budget().cpu_instruction_cost();

        client.create_escrow(
            &creator,
            &task_id,
            &TestValidation::dummy_issue_url(&env),
            &amount,
        );
        client.assign_contributor(&task_id, &contributor);
        client.approve_completion(&task_id);

        let final_gas = env.cost_estimate().budget().cpu_instruction_cost();
        large_amount_gas.push(final_gas - initial_gas);
    }

    // Verify that gas usage doesn't scale significantly with amount size
    let max_variance =
        large_amount_gas.iter().max().unwrap() - large_amount_gas.iter().min().unwrap();
    let avg_gas = large_amount_gas.iter().sum::<u64>() / large_amount_gas.len() as u64;

    // Gas usage should not vary by more than 50% regardless of amount size
    assert!(
        max_variance < avg_gas / 2,
        "Gas usage should not vary significantly with amount size"
    );
}

#[test]
fn test_performance_concurrent_operations() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let bounty_amount = TestConfig::SMALL_AMOUNT;
    let concurrent_count = TestConfig::MEDIUM_CONCURRENT_COUNT;

    // Create multiple users
    let creators: Vec<Address> = (0..concurrent_count)
        .map(|_| Address::generate(&env))
        .collect();
    let contributors: Vec<Address> = (0..concurrent_count)
        .map(|_| Address::generate(&env))
        .collect();

    let initial_gas = env.cost_estimate().budget().cpu_instruction_cost();

    // Simulate concurrent operations
    for i in 0..concurrent_count {
        let task_id = TestValidation::generate_task_id(&env, "concurrent_perf", i);

        usdc_token.mint(&creators[i as usize], &bounty_amount);
        client.create_escrow(
            &creators[i as usize],
            &task_id,
            &TestValidation::dummy_issue_url(&env),
            &bounty_amount,
        );
        client.assign_contributor(&task_id, &contributors[i as usize]);
    }

    let final_gas = env.cost_estimate().budget().cpu_instruction_cost();
    let total_gas = final_gas - initial_gas;

    // Verify reasonable gas usage for concurrent operations
    let max_expected_concurrent_gas = 10_000_000u64;
    TestAssertions::assert_reasonable_gas_usage(total_gas, max_expected_concurrent_gas);
}

#[test]
fn test_memory_efficiency_large_scale() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let bounty_amount = TestConfig::MIN_VALID_AMOUNT;
    let large_scale_count = TestConfig::LARGE_BENCHMARK_SIZE;

    // Create many tasks to test memory efficiency
    for i in 0..large_scale_count {
        let task_id = TestValidation::generate_task_id(&env, "memory_test", i);

        usdc_token.mint(&creator, &bounty_amount);
        client.create_escrow(
            &creator,
            &task_id,
            &TestValidation::dummy_issue_url(&env),
            &bounty_amount,
        );

        // Verify each escrow can be retrieved (tests storage efficiency)
        let escrow = client.get_escrow(&task_id);
        assert_eq!(escrow.bounty_amount, bounty_amount);
    }

    // Test that we can still perform operations efficiently
    let test_task_id = TestValidation::generate_task_id(&env, "memory_test", 0);
    client.assign_contributor(&test_task_id, &contributor);
    client.approve_completion(&test_task_id);

    let escrow = client.get_escrow(&test_task_id);
    assert_eq!(escrow.status, TaskStatus::Completed);
}

// ============================================================================
// SECURITY TESTS
// ============================================================================

#[test]
fn test_authorization_boundaries() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let malicious_user = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "auth_test", 1);
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

    // Test unauthorized operations

    // 1. Malicious user tries to approve completion
    env.mock_all_auths_allowing_non_root_auth();
    let result = client.mock_auths(&[]).try_approve_completion(&task_id);
    assert!(result.is_err());

    // 2. Malicious user tries to refund
    env.mock_all_auths_allowing_non_root_auth();
    let result = client.mock_auths(&[]).try_refund(&task_id);
    assert!(result.is_err());

    // 3. Malicious user tries to dispute
    let reason = TestValidation::generate_dispute_reason(&env, "malicious");
    env.mock_all_auths_allowing_non_root_auth();
    let result = client
        .mock_auths(&[])
        .try_dispute_task(&malicious_user, &task_id, &reason);
    assert!(result.is_err());

    // 4. Non-admin tries to resolve dispute (first create a legitimate dispute)
    client.dispute_task(&creator, &task_id, &reason);

    env.mock_all_auths_allowing_non_root_auth();
    let result = client
        .mock_auths(&[])
        .try_resolve_dispute(&task_id, &DisputeResolution::PayContributor);
    assert!(result.is_err());

    // 5. Non-admin tries to set admin
    env.mock_all_auths_allowing_non_root_auth();
    let result = client.mock_auths(&[]).try_set_admin(&malicious_user);
    assert!(result.is_err());
}

#[test]
fn test_reentrancy_protection() {
    let (env, admin, usdc_address, usdc_token, usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "reentrancy", 1);
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

    // Test that operations cannot be called multiple times in same transaction
    // Basic test - more sophisticated reentrancy tests will require custom malicious contracts

    // Approve completion
    client.approve_completion(&task_id);

    // Try to approve again (should fail due to status change)
    let result = client.try_approve_completion(&task_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidTaskStatus);

    // Verify final state is correct
    let escrow = client.get_escrow(&task_id);
    assert_eq!(escrow.status, TaskStatus::Completed);

    // Verify contributor received payment only once
    let contributor_balance = usdc_token_client.balance(&contributor);
    assert_eq!(contributor_balance, bounty_amount);
}

#[test]
fn test_input_validation_security() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;

    // Test malicious inputs

    // 1. Extremely long task ID
    let malicious_task_id = &String::from_str(&env, &"a".repeat(1000));
    usdc_token.mint(&creator, &bounty_amount);
    let result = client.try_create_escrow(
        &creator,
        &malicious_task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidTaskId);

    // 2. Extremely long dispute reason
    let valid_task_id = TestValidation::generate_task_id(&env, "security", 1);
    client.create_escrow(
        &creator,
        &valid_task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );

    let contributor = Address::generate(&env);
    client.assign_contributor(&valid_task_id, &contributor);

    let malicious_reason = &String::from_str(&env, &"a".repeat(1000));
    let result = client.try_dispute_task(&creator, &valid_task_id, &malicious_reason);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InvalidDisputeReason);

    // 3. Invalid amounts (overflow attempts)
    let task_id_2 = TestValidation::generate_task_id(&env, "security", 2);
    let result = client.try_create_escrow(
        &creator,
        &task_id_2,
        &TestValidation::dummy_issue_url(&env),
        &i128::MAX,
    );
    assert!(result.is_err());
    // Should fail due to amount validation or insufficient balance
}

#[test]
fn test_state_consistency_under_stress() {
    let (env, admin, usdc_address, usdc_token, _usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let bounty_amount = TestConfig::SMALL_AMOUNT;
    let stress_count = TestConfig::MEDIUM_BENCHMARK_SIZE;

    // Perform many operations rapidly
    let mut task_ids = Vec::new();

    for i in 0..stress_count {
        let task_id = TestValidation::generate_task_id(&env, "stress", i);

        usdc_token.mint(&creator, &bounty_amount);
        client.create_escrow(
            &creator,
            &task_id,
            &TestValidation::dummy_issue_url(&env),
            &bounty_amount,
        );

        task_ids.push(task_id);
    }

    // Verify all escrows were created correctly
    for task_id in &task_ids {
        let escrow = client.get_escrow(task_id);
        assert_eq!(escrow.status, TaskStatus::Open);
        assert_eq!(escrow.bounty_amount, bounty_amount);
        assert_eq!(escrow.creator, creator);
    }

    // Perform mixed operations on different tasks
    for (i, task_id) in task_ids.iter().enumerate() {
        match i % 3 {
            0 => {
                // Complete workflow
                client.assign_contributor(task_id, &contributor);
                client.approve_completion(task_id);
            }
            1 => {
                // Refund
                client.refund(task_id);
            }
            2 => {
                // Dispute workflow
                client.assign_contributor(task_id, &contributor);
                let reason = TestValidation::generate_dispute_reason(&env, "stress");
                client.dispute_task(&creator, task_id, &reason);
                client.resolve_dispute(task_id, &DisputeResolution::PayContributor);
            }
            _ => unreachable!(),
        }
    }

    // Verify final state consistency
    let mut resolved_count = 0;
    let mut refunded_count = 0;

    for task_id in &task_ids {
        let escrow = client.get_escrow(task_id);
        match escrow.status {
            TaskStatus::Completed => resolved_count += 1,
            TaskStatus::Cancelled => refunded_count += 1,
            _ => panic!("Unexpected task status after stress test"),
        }
    }

    // Verify counts match expected distribution
    let expected_resolved = 7;
    let expected_refunded = 3;

    assert_eq!(resolved_count, expected_resolved);
    assert_eq!(refunded_count, expected_refunded);
}

#[test]
fn test_token_interaction_security() {
    let (env, admin, usdc_address, usdc_token, usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let task_id = TestValidation::generate_task_id(&env, "token_security", 1);
    let bounty_amount = TestConfig::MEDIUM_AMOUNT;

    // Test token balance checks

    // 1. Verify insufficient balance is caught
    let result = client.try_create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), Error::InsufficientBalance);

    // 2. Fund creator and create escrow
    usdc_token.mint(&creator, &bounty_amount);
    client.create_escrow(
        &creator,
        &task_id,
        &TestValidation::dummy_issue_url(&env),
        &bounty_amount,
    );

    // 3. Verify contract holds the funds
    let contract_balance = client.get_contract_usdc_balance();
    assert_eq!(contract_balance, bounty_amount);

    // 4. Verify creator's balance was reduced
    let creator_balance = usdc_token_client.balance(&creator);
    assert_eq!(creator_balance, 0);

    // 5. Complete workflow and verify proper fund transfer
    client.assign_contributor(&task_id, &contributor);
    client.approve_completion(&task_id);

    // 6. Verify funds were transferred to contributor
    let final_contributor_balance = usdc_token_client.balance(&contributor);
    assert_eq!(final_contributor_balance, bounty_amount);

    // 7. Verify contract balance is now zero
    let final_contract_balance = client.get_contract_usdc_balance();
    assert_eq!(final_contract_balance, 0);
}

#[test]
fn test_edge_case_boundary_conditions() {
    let (env, admin, usdc_address, usdc_token, usdc_token_client, _contract_id, client) =
        create_test_env();

    // Initialize contract
    client.initialize(&admin, &usdc_address);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);

    // Test minimum valid amount
    let min_task_id = TestValidation::generate_task_id(&env, "min_amount", 1);
    usdc_token.mint(&creator, &TestConfig::MIN_VALID_AMOUNT);

    let result = client.create_escrow(
        &creator,
        &min_task_id,
        &TestValidation::dummy_issue_url(&env),
        &TestConfig::MIN_VALID_AMOUNT,
    );
    assert!(result == ());

    // Complete workflow with minimum amount
    client.assign_contributor(&min_task_id, &contributor);
    client.approve_completion(&min_task_id);

    let contributor_balance = usdc_token_client.balance(&contributor);
    assert_eq!(contributor_balance, TestConfig::MIN_VALID_AMOUNT);

    // Test maximum valid amount (if system can handle it)
    let max_task_id = TestValidation::generate_task_id(&env, "max_amount", 1);
    usdc_token.mint(&creator, &TestConfig::MAX_VALID_AMOUNT);

    let result = client.create_escrow(
        &creator,
        &max_task_id,
        &TestValidation::dummy_issue_url(&env),
        &TestConfig::MAX_VALID_AMOUNT,
    );
    assert!(result == ());

    // Verify escrow was created with max amount
    let escrow = client.get_escrow(&max_task_id);
    assert_eq!(escrow.bounty_amount, TestConfig::MAX_VALID_AMOUNT);

    // Test partial payment edge cases
    client.assign_contributor(&max_task_id, &contributor);

    let reason = TestValidation::generate_dispute_reason(&env, "boundary");
    client.dispute_task(&creator, &max_task_id, &reason);

    // Test 1% partial payment (very small percentage)
    let tiny_partial = TestConfig::MAX_VALID_AMOUNT / 100;
    let result = client.resolve_dispute(
        &max_task_id,
        &DisputeResolution::PartialPayment(tiny_partial),
    );
    assert!(result == ());
}
