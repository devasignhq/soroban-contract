use crate::types::*;
use soroban_sdk::{Address, Env, String};

/// Emit an EscrowCreated event
pub fn emit_escrow_created(env: &Env, task_id: String, creator: Address, bounty_amount: i128) {
    EscrowCreatedEvent {
        task_id,
        creator,
        bounty_amount,
        timestamp: env.ledger().timestamp(),
    }
    .publish(env);
}

/// Emit a ContributorAssigned event
pub fn emit_contributor_assigned(env: &Env, task_id: String, contributor: Address) {
    ContributorAssignedEvent {
        task_id,
        contributor,
        timestamp: env.ledger().timestamp(),
    }
    .publish(env);
}

/// Emit a TaskCompleted event
pub fn emit_task_completed(env: &Env, task_id: String, contributor: Address) {
    TaskCompletedEvent {
        task_id,
        contributor,
        timestamp: env.ledger().timestamp(),
    }
    .publish(env);
}

/// Emit a FundsReleased event
pub fn emit_funds_released(env: &Env, task_id: String, contributor: Address, amount: i128) {
    FundsReleasedEvent {
        task_id,
        contributor,
        amount,
        timestamp: env.ledger().timestamp(),
    }
    .publish(env);
}

/// Emit a DisputeInitiated event
pub fn emit_dispute_initiated(
    env: &Env,
    task_id: String,
    disputing_party: Address,
    reason: String,
) {
    DisputeInitiatedEvent {
        task_id,
        disputing_party,
        reason,
        timestamp: env.ledger().timestamp(),
    }
    .publish(env);
}

/// Emit a DisputeResolved event
pub fn emit_dispute_resolved(
    env: &Env,
    task_id: String,
    resolution: DisputeResolution,
    resolved_by: Address,
) {
    DisputeResolvedEvent {
        task_id,
        resolution,
        resolved_by,
        timestamp: env.ledger().timestamp(),
    }
    .publish(env);
}

/// Emit a RefundProcessed event
pub fn emit_refund_processed(env: &Env, task_id: String, creator: Address, amount: i128) {
    RefundProcessedEvent {
        task_id,
        creator,
        amount,
        timestamp: env.ledger().timestamp(),
    }
    .publish(env);
}

/// Emit a BountyIncreased event
pub fn emit_bounty_increased(
    env: &Env,
    task_id: String,
    creator: Address,
    added_amount: i128,
    new_total_amount: i128,
) {
    BountyIncreasedEvent {
        task_id,
        creator,
        added_amount,
        new_total_amount,
        timestamp: env.ledger().timestamp(),
    }
    .publish(env);
}

/// Emit a BountyDecreased event
pub fn emit_bounty_decreased(
    env: &Env,
    task_id: String,
    creator: Address,
    subtracted_amount: i128,
    new_total_amount: i128,
) {
    BountyDecreasedEvent {
        task_id,
        creator,
        subtracted_amount,
        new_total_amount,
        timestamp: env.ledger().timestamp(),
    }
    .publish(env);
}

/// Emit a ContractUpgraded event
pub fn emit_contract_upgraded(env: &Env, new_wasm_hash: soroban_sdk::BytesN<32>, admin: Address) {
    ContractUpgradedEvent {
        new_wasm_hash,
        admin,
        timestamp: env.ledger().timestamp(),
    }
    .publish(env);
}
