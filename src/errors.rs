use soroban_sdk::contracterror;

/// Comprehensive error types for the task escrow contract
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // General errors (1-9)
    TaskNotFound = 1,
    TaskAlreadyExists = 2,
    InvalidTaskStatus = 3,
    ContractNotInitialized = 4,
    
    // Permission errors (10-19)
    Unauthorized = 10,
    NotTaskCreator = 11,
    NotTaskContributor = 12,
    NotAdmin = 13,
    OnlyCreatorOrContributor = 14,
    
    // Business logic errors (20-29)
    ContributorAlreadyAssigned = 20,
    NoContributorAssigned = 21,
    InsufficientBalance = 22,
    TaskNotCompleted = 23,
    TaskNotDisputed = 24,
    TaskAlreadyResolved = 25,
    CannotRefundWithContributor = 26,
    
    // Token errors (30-39)
    TokenTransferFailed = 30,
    InvalidTokenAmount = 31,
    TokenContractNotSet = 32,
    
    // Validation errors (40-49)
    InvalidTaskId = 40,
    InvalidAddress = 41,
    InvalidAmount = 42,
    InvalidDisputeReason = 43,
    EmptyTaskId = 44,
    TaskIdTooShort = 45,
    TaskIdTooLong = 46,
    InvalidTaskIdCharacters = 47,
    AmountTooSmall = 48,
    DisputeReasonTooShort = 49,
    InvalidIssueUrl = 50,
}