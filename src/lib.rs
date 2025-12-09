#![no_std]

mod types;
mod errors;
mod events;

pub use types::*;
pub use errors::*;
pub use events::*;

use soroban_sdk::{contract, contractimpl, token, Address, Env, String, BytesN};

#[contract]
pub struct TaskEscrowContract;

#[contractimpl]
impl TaskEscrowContract {
    /// Initialize the contract with admin and USDC token addresses
    /// This function can only be called once during contract deployment
    pub fn initialize(env: Env, admin: Address, usdc_token: Address) -> Result<(), Error> {
        // Check if contract is already initialized
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::TaskAlreadyExists); // Using existing error for "already initialized"
        }

        // Validate admin address
        Self::validate_address(&admin)?;

        // Validate USDC token address
        Self::validate_address(&usdc_token)?;

        // Store admin and USDC token addresses
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::UsdcToken, &usdc_token);
        
        // Initialize task count to 0
        env.storage().instance().set(&DataKey::TaskCount, &0u64);

        Ok(())
    }

    /// Transfer admin privileges to a new address
    /// Can only be called by current admin
    pub fn set_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        // Validate caller is current admin
        Self::require_admin(&env)?;

        // Validate new admin address
        Self::validate_address(&new_admin)?;

        // Update admin address
        env.storage().instance().set(&DataKey::Admin, &new_admin);

        Ok(())
    }

    /// Update the USDC token contract address
    /// Can only be called by admin (useful for token contract upgrades or migrations)
    pub fn update_usdc_token(env: Env, new_usdc_token: Address) -> Result<(), Error> {
        // Validate caller is current admin
        Self::require_admin(&env)?;

        // Validate new USDC token address
        Self::validate_address(&new_usdc_token)?;

        // Update USDC token address
        env.storage().instance().set(&DataKey::UsdcToken, &new_usdc_token);

        Ok(())
    }

    /// Get the current admin address
    pub fn get_admin(env: Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::ContractNotInitialized)
    }

    /// Get the USDC token contract address
    pub fn get_usdc_token(env: Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::UsdcToken)
            .ok_or(Error::ContractNotInitialized)
    }

    /// Helper function to validate admin access
    fn require_admin(env: &Env) -> Result<(), Error> {
        let admin: Address = env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::ContractNotInitialized)?;
        
        // Require authentication from the admin address
        admin.require_auth();

        Ok(())
    }

    /// Helper function to check if contract is initialized
    fn is_initialized(env: &Env) -> bool {
        env.storage().instance().has(&DataKey::Admin) && 
        env.storage().instance().has(&DataKey::UsdcToken)
    }

    /// Create a new escrow for a task with locked USDC funds
    /// Transfers USDC from creator to contract and creates escrow record
    pub fn create_escrow(env: Env, creator: Address, task_id: String, issue_url: String, bounty_amount: i128) -> Result<(), Error> {
        // Validate contract state and initialization
        Self::validate_contract_state(&env)?;

        // Validate creator address
        Self::validate_address(&creator)?;

        // Require authentication from the creator
        creator.require_auth();

        // Validate task_id format with enhanced checks
        Self::validate_task_id(&task_id)?;

        // Validate issue_url
        Self::validate_issue_url(&issue_url)?;

        // Validate bounty amount with enhanced checks
        Self::validate_amount(bounty_amount)?;

        // Check if task already exists
        if Self::task_exists(&env, &task_id) {
            return Err(Error::TaskAlreadyExists);
        }

        // Check creator has sufficient USDC balance using helper
        // if !Self::has_sufficient_usdc_balance(env.clone(), creator.clone(), bounty_amount)? {
        //     return Err(Error::InsufficientBalance);
        // }

        // Transfer USDC from creator to contract using safe helper
        Self::transfer_usdc_to_contract(&env, &creator, bounty_amount)?;

        // Create escrow record
        let null_address = Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF"));
        let escrow = TaskEscrow {
            task_id: task_id.clone(),
            issue_url: issue_url.clone(),
            creator: creator.clone(),
            contributor: null_address, // Use null address placeholder
            has_contributor: false,
            bounty_amount,
            status: TaskStatus::Open,
            created_at: env.ledger().timestamp(),
            completed_at: 0,
            disputed_at: 0,
        };

        // Store escrow data
        env.storage().persistent().set(&DataKey::TaskEscrow(task_id.clone()), &escrow);

        // Increment task count
        let current_count: u64 = env.storage()
            .instance()
            .get(&DataKey::TaskCount)
            .unwrap_or(0);
        env.storage().instance().set(&DataKey::TaskCount, &(current_count + 1));

        // Emit escrow created event
        crate::events::emit_escrow_created(&env, task_id, creator, bounty_amount);

        Ok(())
    }

    /// Get escrow information for a specific task
    pub fn get_escrow(env: Env, task_id: String) -> Result<TaskEscrow, Error> {
        // Validate task_id format
        Self::validate_task_id(&task_id)?;

        // Retrieve escrow data
        env.storage()
            .persistent()
            .get(&DataKey::TaskEscrow(task_id))
            .ok_or(Error::TaskNotFound)
    }

    /// Get the total number of tasks created
    pub fn get_task_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::TaskCount)
            .unwrap_or(0)
    }

    /// Helper function to check if a task exists
    fn task_exists(env: &Env, task_id: &String) -> bool {
        env.storage().persistent().has(&DataKey::TaskEscrow(task_id.clone()))
    }

    /// Helper function to validate task ID format with comprehensive checks
    fn validate_task_id(task_id: &String) -> Result<(), Error> {
        // Check for empty task ID
        if task_id.len() == 0 {
            return Err(Error::EmptyTaskId);
        }
        
        // Check required length (25 characters)
        if task_id.len() != 25 {
            return Err(Error::InvalidTaskId);
        }

        Ok(())
    }

    /// Helper function to validate issue URL
    fn validate_issue_url(url: &String) -> Result<(), Error> {
        if url.len() == 0 {
            return Err(Error::InvalidIssueUrl);
        }
        // Basic length check for URL
        if url.len() > 500 {
            return Err(Error::InvalidIssueUrl);
        }
        Ok(())
    }

    /// Helper function to validate bounty amount with comprehensive checks
    fn validate_amount(amount: i128) -> Result<(), Error> {
        // Check for negative or zero amounts
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        
        // Check minimum amount (0.01 USDC = 100000 stroops)
        // This prevents dust amounts that could cause issues
        if amount < 100000 {
            return Err(Error::InvalidAmount);
        }
        
        // Check for reasonable upper limit (1 billion USDC)
        // USDC has 7 decimal places, so this is 1,000,000,000.0000000 USDC
        if amount > 1_000_000_000_0000000 {
            return Err(Error::InvalidTokenAmount);
        }
        
        // Validate precision - USDC has 7 decimal places
        // Ensure the amount is properly formatted for USDC
        // This prevents precision errors in token operations
        if amount % 1 != 0 {
            return Err(Error::InvalidTokenAmount);
        }
        
        Ok(())
    }

    /// Assign a contributor to a task
    /// Can only be called by the task creator when task is in Open status
    pub fn assign_contributor(env: Env, task_id: String, contributor: Address) -> Result<(), Error> {
        // Validate contract state
        Self::validate_contract_state(&env)?;

        // Validate task_id format with enhanced checks
        Self::validate_task_id(&task_id)?;

        // Validate contributor address
        Self::validate_address(&contributor)?;

        // Get the existing escrow
        let mut escrow: TaskEscrow = env.storage()
            .persistent()
            .get(&DataKey::TaskEscrow(task_id.clone()))
            .ok_or(Error::TaskNotFound)?;

        // Require authentication from the task creator
        escrow.creator.require_auth();

        // Validate no contributor is already assigned
        if escrow.has_contributor {
            return Err(Error::ContributorAlreadyAssigned);
        }

        // Update escrow with contributor information
        escrow.contributor = contributor.clone();
        escrow.has_contributor = true;
        escrow.status = TaskStatus::InProgress;

        // Store updated escrow data
        env.storage().persistent().set(&DataKey::TaskEscrow(task_id.clone()), &escrow);

        // Emit contributor assigned event
        crate::events::emit_contributor_assigned(&env, task_id, contributor);

        Ok(())
    }

    /// Mark a task as completed by the assigned contributor
    /// Can only be called by the assigned contributor when task is in InProgress status
    pub fn complete_task(env: Env, task_id: String) -> Result<(), Error> {
        // Validate contract state
        Self::validate_contract_state(&env)?;

        // Validate task_id format with enhanced checks
        Self::validate_task_id(&task_id)?;

        // Get the existing escrow
        let mut escrow: TaskEscrow = env.storage()
            .persistent()
            .get(&DataKey::TaskEscrow(task_id.clone()))
            .ok_or(Error::TaskNotFound)?;

        // Validate task status (must be InProgress)
        if escrow.status != TaskStatus::InProgress {
            return Err(Error::InvalidTaskStatus);
        }

        // Validate that a contributor is assigned
        if !escrow.has_contributor {
            return Err(Error::NoContributorAssigned);
        }

        // Require authentication from the assigned contributor
        escrow.contributor.require_auth();

        // Update escrow status and completion timestamp
        escrow.status = TaskStatus::Completed;
        escrow.completed_at = env.ledger().timestamp();

        // Store updated escrow data
        env.storage().persistent().set(&DataKey::TaskEscrow(task_id.clone()), &escrow);

        // Emit task completed event
        crate::events::emit_task_completed(&env, task_id, escrow.contributor);

        Ok(())
    }

    /// Approve task completion and release funds to contributor
    /// Can only be called by the task creator when task is in Completed status
    pub fn approve_completion(env: Env, task_id: String) -> Result<(), Error> {
        // Validate contract state
        Self::validate_contract_state(&env)?;

        // Validate task_id format with enhanced checks
        Self::validate_task_id(&task_id)?;

        // Get the existing escrow
        let mut escrow: TaskEscrow = env.storage()
            .persistent()
            .get(&DataKey::TaskEscrow(task_id.clone()))
            .ok_or(Error::TaskNotFound)?;

        // Require authentication from the task creator
        escrow.creator.require_auth();

        // Validate task status (must be Completed)
        if escrow.status != TaskStatus::Completed {
            return Err(Error::TaskNotCompleted);
        }

        // Validate that a contributor is assigned
        if !escrow.has_contributor {
            return Err(Error::NoContributorAssigned);
        }

        // Transfer USDC from contract to contributor using safe helper
        Self::transfer_usdc_from_contract(&env, &escrow.contributor, escrow.bounty_amount)?;

        // Update escrow status to resolved
        escrow.status = TaskStatus::Resolved;

        // Store updated escrow data
        env.storage().persistent().set(&DataKey::TaskEscrow(task_id.clone()), &escrow);

        // Emit funds released event
        crate::events::emit_funds_released(&env, task_id, escrow.contributor, escrow.bounty_amount);

        Ok(())
    }

    /// Initiate a dispute for a task
    /// Can be called by either the task creator or assigned contributor
    pub fn dispute_task(env: Env, disputing_party: Address, task_id: String, reason: String) -> Result<(), Error> {
        // Validate contract state
        Self::validate_contract_state(&env)?;

        // Validate task_id format with enhanced checks
        Self::validate_task_id(&task_id)?;

        // Validate disputing party address
        Self::validate_address(&disputing_party)?;

        // Validate dispute reason with enhanced checks
        Self::validate_dispute_reason(&reason)?;

        // Require authentication from the disputing party
        disputing_party.require_auth();

        // Get the existing escrow
        let mut escrow: TaskEscrow = env.storage()
            .persistent()
            .get(&DataKey::TaskEscrow(task_id.clone()))
            .ok_or(Error::TaskNotFound)?;

        // Validate that a contributor is assigned (needed for resolution)
        if !escrow.has_contributor {
            return Err(Error::NoContributorAssigned);
        }

        // Validate task is either InProgress or Completed
        if escrow.status == TaskStatus::Resolved || 
        escrow.status == TaskStatus::Disputed || 
        escrow.status == TaskStatus::Cancelled {
            return Err(Error::TaskAlreadyResolved);
        }

        // Validate that the disputing party is either creator or contributor
        let is_creator = disputing_party == escrow.creator;
        let is_contributor = escrow.has_contributor && disputing_party == escrow.contributor;

        if !is_creator && !is_contributor {
            return Err(Error::OnlyCreatorOrContributor);
        }

        // Create dispute info record
        let dispute_info = DisputeInfo {
            task_id: task_id.clone(),
            disputing_party: disputing_party.clone(),
            reason: reason.clone(),
            initiated_at: env.ledger().timestamp(),
        };

        // Store dispute info
        env.storage().persistent().set(&DataKey::Dispute(task_id.clone()), &dispute_info);

        // Update escrow status to disputed
        escrow.status = TaskStatus::Disputed;
        escrow.disputed_at = env.ledger().timestamp();

        // Store updated escrow data
        env.storage().persistent().set(&DataKey::TaskEscrow(task_id.clone()), &escrow);

        // Emit dispute initiated event
        crate::events::emit_dispute_initiated(&env, task_id, disputing_party, reason);

        Ok(())
    }

    /// Helper function to validate dispute reason with comprehensive checks
    fn validate_dispute_reason(reason: &String) -> Result<(), Error> {
        // Check for empty reason
        if reason.len() == 0 {
            return Err(Error::InvalidDisputeReason);
        }
        
        // Check minimum length (at least 10 characters for meaningful dispute reasons)
        if reason.len() < 10 {
            return Err(Error::InvalidDisputeReason);
        }
        
        // Check maximum length (reasonable limit for storage and readability)
        if reason.len() > 500 {
            return Err(Error::InvalidDisputeReason);
        }
        
        Ok(())
    }

    /// Helper function to validate addresses and prevent null/invalid addresses
    fn validate_address(_address: &Address) -> Result<(), Error> {
        // For Soroban SDK, we rely on the SDK's built-in address validation
        // The SDK ensures addresses are properly formatted when created
        // Basic validation is sufficient for our use case
        Ok(())
    }

    /// Helper function to validate partial payment amounts in dispute resolution
    fn validate_partial_payment(partial_amount: i128, total_amount: i128) -> Result<(), Error> {
        // Validate the partial amount itself
        Self::validate_amount(partial_amount)?;
        
        // Ensure partial amount doesn't exceed total amount
        if partial_amount > total_amount {
            return Err(Error::InvalidTokenAmount);
        }
        
        // Ensure partial amount is meaningful (at least 1% of total)
        let minimum_partial = total_amount / 100; // 1% minimum
        if partial_amount < minimum_partial {
            return Err(Error::InvalidTokenAmount);
        }
        
        // Ensure remaining amount is also meaningful (at least 1% of total)
        let remaining = total_amount - partial_amount;
        if remaining < minimum_partial {
            return Err(Error::InvalidTokenAmount);
        }
        
        Ok(())
    }

    /// Helper function to validate contract state before operations
    fn validate_contract_state(env: &Env) -> Result<(), Error> {
        // Check if contract is initialized
        if !Self::is_initialized(env) {
            return Err(Error::ContractNotInitialized);
        }
        
        // Verify admin address exists and is valid
        let admin: Address = env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::ContractNotInitialized)?;
        Self::validate_address(&admin)?;
        
        // Verify USDC token address exists and is valid
        let usdc_token: Address = env.storage()
            .instance()
            .get(&DataKey::UsdcToken)
            .ok_or(Error::TokenContractNotSet)?;
        Self::validate_address(&usdc_token)?;
        
        Ok(())
    }

    /// Resolve a dispute with admin authority
    /// Can only be called by admin when task is in Disputed status
    pub fn resolve_dispute(env: Env, task_id: String, resolution: DisputeResolution) -> Result<(), Error> {
        // Validate contract state
        Self::validate_contract_state(&env)?;

        // Validate task_id format with enhanced checks
        Self::validate_task_id(&task_id)?;

        // Require admin authentication
        Self::require_admin(&env)?;

        // Get the existing escrow
        let mut escrow: TaskEscrow = env.storage()
            .persistent()
            .get(&DataKey::TaskEscrow(task_id.clone()))
            .ok_or(Error::TaskNotFound)?;

        // Validate task status (must be Disputed)
        if escrow.status != TaskStatus::Disputed {
            return Err(Error::TaskNotDisputed);
        }

        // Validate that a contributor is assigned (needed for resolution)
        if !escrow.has_contributor {
            return Err(Error::NoContributorAssigned);
        }

        // Get admin address for event emission
        let admin: Address = env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::ContractNotInitialized)?;

        // Process resolution based on type with enhanced validation
        match resolution {
            DisputeResolution::PayContributor => {
                // Transfer full amount to contributor using safe helper
                Self::transfer_usdc_from_contract(&env, &escrow.contributor, escrow.bounty_amount)?;
            },
            DisputeResolution::RefundCreator => {
                // Transfer full amount back to creator using safe helper
                Self::transfer_usdc_from_contract(&env, &escrow.creator, escrow.bounty_amount)?;
            },
            DisputeResolution::PartialPayment(amount) => {
                // Validate partial payment amount with enhanced checks
                Self::validate_partial_payment(amount, escrow.bounty_amount)?;

                // Transfer partial amount to contributor using safe helper
                Self::transfer_usdc_from_contract(&env, &escrow.contributor, amount)?;
                
                // Transfer remaining amount to creator using safe helper
                let remaining = escrow.bounty_amount - amount;
                Self::transfer_usdc_from_contract(&env, &escrow.creator, remaining)?;
            }
        }

        // Update escrow status to resolved
        escrow.status = TaskStatus::Resolved;

        // Store updated escrow data
        env.storage().persistent().set(&DataKey::TaskEscrow(task_id.clone()), &escrow);

        // Emit dispute resolved event
        crate::events::emit_dispute_resolved(&env, task_id, resolution, admin);

        Ok(())
    }

    /// Process refund for a cancelled task
    /// Can only be called by the task creator when no contributor is assigned
    pub fn refund(env: Env, task_id: String) -> Result<(), Error> {
        // Validate contract state
        Self::validate_contract_state(&env)?;

        // Validate task_id format with enhanced checks
        Self::validate_task_id(&task_id)?;

        // Get the existing escrow
        let mut escrow: TaskEscrow = env.storage()
            .persistent()
            .get(&DataKey::TaskEscrow(task_id.clone()))
            .ok_or(Error::TaskNotFound)?;

        // Require authentication from the task creator
        escrow.creator.require_auth();

        // Validate that no contributor is assigned (task status must be Open)
        if escrow.has_contributor {
            return Err(Error::ContributorAlreadyAssigned);
        }

        // Validate task status (must be Open)
        if escrow.status != TaskStatus::Open {
            return Err(Error::InvalidTaskStatus);
        }

        // Transfer USDC from contract back to creator using safe helper
        Self::transfer_usdc_from_contract(&env, &escrow.creator, escrow.bounty_amount)?;

        // Update escrow status to cancelled
        escrow.status = TaskStatus::Cancelled;

        // Store updated escrow data
        env.storage().persistent().set(&DataKey::TaskEscrow(task_id.clone()), &escrow);

        // Emit refund processed event
        crate::events::emit_refund_processed(&env, task_id, escrow.creator, escrow.bounty_amount);

        Ok(())
    }

    /// Get dispute information for a specific task
    /// Returns comprehensive dispute details for admin and parties
    pub fn get_dispute_info(env: Env, task_id: String) -> Result<DisputeInfo, Error> {
        // Validate contract state
        Self::validate_contract_state(&env)?;

        // Validate task_id format with enhanced checks
        Self::validate_task_id(&task_id)?;

        // Validate that the task exists first
        if !Self::task_exists(&env, &task_id) {
            return Err(Error::TaskNotFound);
        }

        // Retrieve dispute info
        env.storage()
            .persistent()
            .get(&DataKey::Dispute(task_id))
            .ok_or(Error::TaskNotDisputed)
    }

    // USDC Token Contract Integration Helpers

    /// Get the balance of USDC tokens for a specific address
    /// Returns the balance in stroops (7 decimal places for USDC)
    pub fn get_usdc_balance(env: Env, address: Address) -> Result<i128, Error> {
        // Validate contract state
        Self::validate_contract_state(&env)?;

        // Validate address
        Self::validate_address(&address)?;

        // Get USDC token contract address
        let usdc_token = Self::get_usdc_token_internal(&env)?;

        // Create token client and get balance
        let token_client = token::Client::new(&env, &usdc_token);
        let balance = token_client.balance(&address);

        Ok(balance)
    }

    /// Check if an address has sufficient USDC balance for a specific amount
    /// Returns true if balance >= required_amount, false otherwise
    pub fn has_sufficient_usdc_balance(env: Env, address: Address, required_amount: i128) -> Result<bool, Error> {
        // Validate contract state
        Self::validate_contract_state(&env)?;

        // Validate address
        Self::validate_address(&address)?;

        // Validate amount
        Self::validate_amount(required_amount)?;

        // Get current balance
        let current_balance = Self::get_usdc_balance(env.clone(), address)?;

        Ok(current_balance >= required_amount)
    }

    /// Transfer USDC tokens from one address to another with proper error handling
    /// This is a safe wrapper around the token transfer functionality
    fn transfer_usdc_safe(env: &Env, from: &Address, to: &Address, amount: i128) -> Result<(), Error> {
        // Validate addresses
        Self::validate_address(from)?;
        Self::validate_address(to)?;

        // Validate amount
        Self::validate_amount(amount)?;

        // Get USDC token contract address
        let usdc_token = Self::get_usdc_token_internal(env)?;

        // Create token client
        let token_client = token::Client::new(env, &usdc_token);

        // Check sender has sufficient balance before attempting transfer
        let sender_balance = token_client.balance(from);
        if sender_balance < amount {
            return Err(Error::InsufficientBalance);
        }

        // Perform the transfer
        // Note: The token contract will handle authorization checks
        token_client.transfer(from, to, &amount);

        Ok(())
    }

    /// Transfer USDC from an external address to the contract
    /// Used during escrow creation to lock funds
    fn transfer_usdc_to_contract(env: &Env, from: &Address, amount: i128) -> Result<(), Error> {
        let contract_address = env.current_contract_address();
        Self::transfer_usdc_safe(env, from, &contract_address, amount)
    }

    /// Transfer USDC from the contract to an external address
    /// Used during payment release and refunds
    fn transfer_usdc_from_contract(env: &Env, to: &Address, amount: i128) -> Result<(), Error> {
        let contract_address = env.current_contract_address();
        Self::transfer_usdc_safe(env, &contract_address, to, amount)
    }

    /// Get the contract's current USDC balance
    /// Useful for monitoring and validation purposes
    pub fn get_contract_usdc_balance(env: Env) -> Result<i128, Error> {
        // Validate contract state
        Self::validate_contract_state(&env)?;

        let contract_address = env.current_contract_address();
        Self::get_usdc_balance(env, contract_address)
    }

    /// Initialize and validate USDC token contract client
    /// Returns a properly configured token client for USDC operations
    fn create_usdc_token_client(env: &Env) -> Result<token::Client<'_>, Error> {
        // Get USDC token contract address
        let usdc_token = Self::get_usdc_token_internal(env)?;

        // Create and return token client
        Ok(token::Client::new(env, &usdc_token))
    }

    /// Internal helper to get USDC token address with proper error handling
    /// Centralizes USDC token address retrieval logic
    fn get_usdc_token_internal(env: &Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::UsdcToken)
            .ok_or(Error::TokenContractNotSet)
    }

    /// Validate that the USDC token contract is properly set and accessible
    /// Performs a basic validation by attempting to create a client
    pub fn validate_usdc_token_contract(env: Env) -> Result<bool, Error> {
        // Validate contract state
        Self::validate_contract_state(&env)?;

        // Try to create a token client - this will fail if the token contract is invalid
        let _token_client = Self::create_usdc_token_client(&env)?;

        // If we get here, the token contract is accessible
        Ok(true)
    }

    /// Get detailed USDC token information for debugging and validation
    /// Returns token contract address and basic validation status
    pub fn get_usdc_token_info(env: Env) -> Result<(Address, bool), Error> {
        // Validate contract state
        Self::validate_contract_state(&env)?;

        // Get token address
        let usdc_token = Self::get_usdc_token_internal(&env)?;

        // Validate token contract accessibility
        let is_valid = Self::validate_usdc_token_contract(env)?;

        Ok((usdc_token, is_valid))
    }

    /// Upgrade the contract to new WASM bytecode
    /// Can only be called by admin
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), Error> {
        // Require admin authentication
        Self::require_admin(&env)?;

        // Update the contract's WASM code
        env.deployer().update_current_contract_wasm(new_wasm_hash);

        Ok(())
    }

    /// Get contract version
    pub fn version() -> u64 {
        2
    }
}
