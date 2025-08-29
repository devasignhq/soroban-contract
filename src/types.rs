use soroban_sdk::{contracttype, Address, String};

/// Main escrow data structure for storing task information
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct TaskEscrow {
    pub task_id: String,
    pub creator: Address,
    pub contributor: Address, // Will use a special "null" address when no contributor
    pub has_contributor: bool, // Flag to indicate if contributor is set
    pub bounty_amount: i128,
    pub status: TaskStatus,
    pub created_at: u64,
    pub completed_at: u64, // Will use 0 when not set
    pub disputed_at: u64, // Will use 0 when not set
}

/// Task status enumeration for lifecycle management
#[derive(Clone, PartialEq, Debug)]
#[contracttype]
pub enum TaskStatus {
    Open,
    InProgress,
    Completed,
    Disputed,
    Resolved,
    Cancelled,
}

/// Dispute information structure
#[derive(Clone)]
#[contracttype]
pub struct DisputeInfo {
    pub task_id: String,
    pub disputing_party: Address,
    pub reason: String,
    pub initiated_at: u64,
}

/// Storage keys for contract data organization
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    TaskEscrow(String),      // task_id -> TaskEscrow
    Admin,                   // Admin address
    UsdcToken,              // USDC token contract address
    TaskCount,              // Total number of tasks
    Dispute(String),        // task_id -> DisputeInfo
}

/// Dispute resolution options for admin
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DisputeResolution {
    PayContributor,         // Full payment to contributor
    RefundCreator,          // Full refund to creator
    PartialPayment(i128),   // Partial payment to contributor, rest to creator
}

// Event structures for off-chain integration

/// Event emitted when a new escrow is created
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct EscrowCreatedEvent {
    pub task_id: String,
    pub creator: Address,
    pub bounty_amount: i128,
    pub timestamp: u64,
}

/// Event emitted when a contributor is assigned to a task
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ContributorAssignedEvent {
    pub task_id: String,
    pub contributor: Address,
    pub timestamp: u64,
}

/// Event emitted when a task is marked as completed
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct TaskCompletedEvent {
    pub task_id: String,
    pub contributor: Address,
    pub timestamp: u64,
}

/// Event emitted when funds are released to contributor
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct FundsReleasedEvent {
    pub task_id: String,
    pub contributor: Address,
    pub amount: i128,
    pub timestamp: u64,
}

/// Event emitted when a dispute is initiated
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct DisputeInitiatedEvent {
    pub task_id: String,
    pub disputing_party: Address,
    pub reason: String,
    pub timestamp: u64,
}

/// Event emitted when a dispute is resolved
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct DisputeResolvedEvent {
    pub task_id: String,
    pub resolution: DisputeResolution,
    pub resolved_by: Address,
    pub timestamp: u64,
}

/// Event emitted when a refund is processed
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RefundProcessedEvent {
    pub task_id: String,
    pub creator: Address,
    pub amount: i128,
    pub timestamp: u64,
}