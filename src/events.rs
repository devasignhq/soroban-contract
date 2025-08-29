use soroban_sdk::{Env, Address, String};
use crate::types::*;

/// Event emission helper functions for consistent event handling

/// Emit an EscrowCreated event
pub fn emit_escrow_created(
    env: &Env,
    task_id: String,
    creator: Address,
    bounty_amount: i128,
) {
    let event = EscrowCreatedEvent {
        task_id,
        creator,
        bounty_amount,
        timestamp: env.ledger().timestamp(),
    };
    env.events().publish(("EscrowCreated",), event);
}

/// Emit a ContributorAssigned event
pub fn emit_contributor_assigned(
    env: &Env,
    task_id: String,
    contributor: Address,
) {
    let event = ContributorAssignedEvent {
        task_id,
        contributor,
        timestamp: env.ledger().timestamp(),
    };
    env.events().publish(("ContributorAssigned",), event);
}

/// Emit a TaskCompleted event
pub fn emit_task_completed(
    env: &Env,
    task_id: String,
    contributor: Address,
) {
    let event = TaskCompletedEvent {
        task_id,
        contributor,
        timestamp: env.ledger().timestamp(),
    };
    env.events().publish(("TaskCompleted",), event);
}

/// Emit a FundsReleased event
pub fn emit_funds_released(
    env: &Env,
    task_id: String,
    contributor: Address,
    amount: i128,
) {
    let event = FundsReleasedEvent {
        task_id,
        contributor,
        amount,
        timestamp: env.ledger().timestamp(),
    };
    env.events().publish(("FundsReleased",), event);
}

/// Emit a DisputeInitiated event
pub fn emit_dispute_initiated(
    env: &Env,
    task_id: String,
    disputing_party: Address,
    reason: String,
) {
    let event = DisputeInitiatedEvent {
        task_id,
        disputing_party,
        reason,
        timestamp: env.ledger().timestamp(),
    };
    env.events().publish(("DisputeInitiated",), event);
}

/// Emit a DisputeResolved event
pub fn emit_dispute_resolved(
    env: &Env,
    task_id: String,
    resolution: DisputeResolution,
    resolved_by: Address,
) {
    let event = DisputeResolvedEvent {
        task_id,
        resolution,
        resolved_by,
        timestamp: env.ledger().timestamp(),
    };
    env.events().publish(("DisputeResolved",), event);
}

/// Emit a RefundProcessed event
pub fn emit_refund_processed(
    env: &Env,
    task_id: String,
    creator: Address,
    amount: i128,
) {
    let event = RefundProcessedEvent {
        task_id,
        creator,
        amount,
        timestamp: env.ledger().timestamp(),
    };
    env.events().publish(("RefundProcessed",), event);
}