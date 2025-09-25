<br/>
<div align="center">
  <a href="https://www.devasign.com" style="display: block; margin: 0 auto;">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="./public/devasign-white.png">
      <source media="(prefers-color-scheme: light)" srcset="./public/devasign-black.png">
      <img alt="DevAsign Logo" src="./public/devasign-white.png" height="80" style="display: block; margin: 0 auto;">
    </picture>
  </a>
<br/>

<br/>
</div>
<br/>

<div align="center">
    <a href="https://github.com/devasignhq/soroban-contract?tab=Apache-2.0-1-ov-file">
  <img src="https://img.shields.io/github/license/devasignhq/soroban-contract" alt="License">
<a href="https://GitHub.com/devasignhq/soroban-contract/graphs/contributors">
  <img src="https://img.shields.io/github/contributors/devasignhq/soroban-contract" alt="GitHub Contributors">
</a>
<a href="https://devasign.com">
  <img src="https://img.shields.io/badge/Visit-devasign.com-orange" alt="Visit devasign.com">
</a>
</div>
<div>
  <p align="center">
    <a href="https://x.com/devasign">
      <img src="https://img.shields.io/badge/Follow%20on%20X-000000?style=for-the-badge&logo=x&logoColor=white" alt="Follow on X" />
    </a>
    <a href="https://www.linkedin.com/company/devasign">
      <img src="https://img.shields.io/badge/Follow%20on%20LinkedIn-0077B5?style=for-the-badge&logo=linkedin&logoColor=white" alt="Follow on LinkedIn" />
    </a>
  </p>
</div>


<div align="center">
  
  **Join our stargazers :)** 

  <a href="https://github.com/devasignhq/soroban-contract">
    <img src="https://img.shields.io/github/stars/devasignhq?style=social&label=Star&maxAge=2592000" alt="GitHub stars">
  </a>

  <br/>
  </div>
  <br/>
  </div>

## Soroban Escrow Contract

Smart Contract ensuring transparent and trustless payment processing between project maintainers (bounty sponsor) and contributors (beneficiary).

- **Secure Escrow Management**: Holds USDC bounty payments in escrow until task completion.
- **Automated Payments**: Instant payment release upon task approval.
- **Dispute Resolution**: Built-in dispute handling with partial payment options.
- **Transparent Operations**: All transactions recorded on Stellar blockchain.
- **Gas Optimized**: Efficient contract design for minimal transaction costs.

## Contract Architecture

#### Core Components

- **Task Management**: Create, assign, and track task completion status.
- **Escrow System**: Secure USDC token holding and release mechanisms.
- **Contributor Assignment**: Flexible contributor assignment and management.
- **Dispute Resolution**: Multi-option dispute handling (refund, partial payment, full payment).
- **Event Emission**: Comprehensive event logging for transparency.
- **Admin Controls**: Contract initialization and administrative functions.

#### Supported Workflows

1. **Happy Path**: Create → Assign → Complete → Approve → Payment
2. **Refund Path**: Create → Refund (before assignment)
3. **Dispute Path**: Create → Assign → Complete → Dispute → Resolution

## Prerequisites

#### Development Environment
- [**Rust**](https://doc.rust-lang.org/book/ch01-01-installation.html) (v1.85.0 or higher)
- [**Stellar CLI**](https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup)
- **Git** (latest version)

<!-- ### Stellar Network Setup
- **Stellar Account** with XLM for transaction fees
- **USDC Token Contract** address (testnet or mainnet)
- **Soroban RPC Endpoint** access -->


## Installation & Setup

#### 1. Clone the Repository
```bash
git clone https://github.com/devasignhq/soroban-contracts.git
cd soroban-contracts
```

#### 2. Install the target
```bash
rustup target add wasm32v1-none
```

#### 3. Build the Contract
```bash
# Build the contract
stellar contract build
```

#### 4. Configure an Identity
```bash
# Create an identity (change 'lenny' to any name you want)
stellar keys generate --global lenny --network testnet --fund

# Get the public key of lenny 
stellar keys address lenny
```
Fund the account using Stellar's [Friendbot](https://lab.stellar.org/account/fund?$=network$id=testnet&label=Testnet&horizonUrl=https:////horizon-testnet.stellar.org&rpcUrl=https:////soroban-testnet.stellar.org&passphrase=Test%20SDF%20Network%20/;%20September%202015;&transaction$build$classic$operations@$operation_type=path_payment_strict_send&params$;&source_account=;;;&soroban$operation$params@;;)

#### 5. Deploy to Testnet
```bash
# Deploy contract
stellar contract deploy \
  --wasm target/wasm32v1-none/release/devasign_task_escrow.wasm `
  --source-account lenny \
  --network testnet \
  --alias devasign_task_escrow

# Initialize contract
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source lenny \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --usdc_token <USDC_TOKEN_ADDRESS>
```

## Testing

#### Run Tests
```bash
# Run all tests
cargo test

# Run specific test module
cargo test --test test_contributor_assignment
```

#### Test Structure
```
tests/
├── test_initialization.rs      # Contract setup tests
├── test_escrow_creation.rs     # Escrow creation tests
├── test_contributor_assignment.rs # Assignment workflow tests
├── test_task_completion.rs     # Completion workflow tests
├── test_approval_and_payment.rs # Payment release tests
├── test_disputes.rs            # Dispute handling tests
├── test_refunds.rs            # Refund mechanism tests
├── test_events.rs             # Event emission tests
├── test_performance_security.rs # Performance and security tests
└── test_integration.rs        # End-to-end integration tests
```

<!-- ## 🚀 Deployment -->

<!-- ## 📈 Integration with DevAsign API -->

## Monitoring & Events

#### Emitted Events
- `EscrowCreated`: New task escrow created
- `ContributorAssigned`: Contributor assigned to task
- `TaskCompleted`: Task marked as completed
- `FundsReleased`: Payment released to contributor
- `DisputeInitiated`: Dispute raised for task
- `DisputeResolved`: Dispute resolved with outcome
- `RefundProcessed`: Bounty refunded to creator

## License

This project is licensed under the Apache 2.0 License. See [LICENSE](https://github.com/devasignhq/soroban-contracts/blob/main/LICENSE) for more details.

## Repo Activity

<img width="100%" src="https://repobeats.axiom.co/api/embed/0c69234f1d8c60c1c18e3a822093838310a7a30b.svg" />

<!-- ## 🤝 Contributing -->

## Related Projects

- [DevAsign API Server](https://github.com/devasignhq/devasign-api) - Backend API and AI engine
- [DevAsign Project Maintainer App](https://github.com/devasignhq/app.devasign.com) - Frontend for project maintainer
- [DevAsign Contributor App](https://github.com/devasignhq/contributor.devasign.com) - Frontend for contributors


---

