# DevAsign Task Escrow Smart Contract

<br/>
<div align="center">
  <a href="https://www.devasign.com" style="display: block; margin: 0 auto;">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="./public/devasign-white.png">
      <source media="(prefers-color-scheme: light)" srcset="./public/devasign-black.png">
      <img alt="DevAsign Logo" src="./public/devasign-white.png" height="120" style="display: block; margin: 0 auto;">
    </picture>
  </a>
</div>

<br/>

A Soroban smart contract that manages secure escrow payments for open-source projects bounties on the Stellar blockchain network.

## 🚀 Overview

The DevAsign Task Escrow Contract is a core component of the DevAsign ecosystem, providing secure, automated bounty management for open-source contributions. Built on Stellar's Soroban platform, it ensures transparent and trustless payment processing between project creators and contributors.

### Key Features

- **Secure Escrow Management**: Holds USDC bounty payments in escrow until task completion
- **Automated Payments**: Instant payment release upon task approval
- **Dispute Resolution**: Built-in dispute handling with partial payment options
- **Transparent Operations**: All transactions recorded on Stellar blockchain
- **Gas Optimized**: Efficient contract design for minimal transaction costs

## 🏗️ Contract Architecture

### Core Components

- **Task Management**: Create, assign, and track task completion status
- **Escrow System**: Secure USDC token holding and release mechanisms
- **Contributor Assignment**: Flexible contributor assignment and management
- **Dispute Resolution**: Multi-option dispute handling (refund, partial payment, full payment)
- **Event Emission**: Comprehensive event logging for transparency
- **Admin Controls**: Contract initialization and administrative functions

### Supported Workflows

1. **Happy Path**: Create → Assign → Complete → Approve → Payment
2. **Refund Path**: Create → Refund (before assignment)
3. **Dispute Path**: Create → Assign → Complete → Dispute → Resolution

## 📋 Prerequisites

### Development Environment
- [**Rust**](https://doc.rust-lang.org/book/ch01-01-installation.html) (v1.85.0 or higher)
- [**Stellar CLI**](https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup)
- **Git** (latest version)

<!-- ### Stellar Network Setup
- **Stellar Account** with XLM for transaction fees
- **USDC Token Contract** address (testnet or mainnet)
- **Soroban RPC Endpoint** access -->


## 🛠️ Installation & Setup

### 1. Clone the Repository
```bash
git clone https://github.com/devasignhq/soroban-contracts.git
cd soroban-contracts
```

### 2. Install the target
```bash
rustup target add wasm32v1-none
```

### 3. Build the Contract
```bash
# Build the contract
stellar contract build
```

### 4. Configure an Identity
```bash
# Create an identity (change 'lenny' to any name you want)
stellar keys generate --global lenny --network testnet --fund

# Get the public key of lenny 
stellar keys address lenny
```
Fund the account using Stellar's [Friendbot](https://lab.stellar.org/account/fund?$=network$id=testnet&label=Testnet&horizonUrl=https:////horizon-testnet.stellar.org&rpcUrl=https:////soroban-testnet.stellar.org&passphrase=Test%20SDF%20Network%20/;%20September%202015;&transaction$build$classic$operations@$operation_type=path_payment_strict_send&params$;&source_account=;;;&soroban$operation$params@;;)

### 5. Deploy to Testnet
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

## 🧪 Testing

### Run Tests
```bash
# Run all tests
cargo test

# Run specific test module
cargo test --test test_contributor_assignment
```

### Test Structure
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

## 🔍 Monitoring & Events

### Emitted Events
- `EscrowCreated`: New task escrow created
- `ContributorAssigned`: Contributor assigned to task
- `TaskCompleted`: Task marked as completed
- `FundsReleased`: Payment released to contributor
- `DisputeInitiated`: Dispute raised for task
- `DisputeResolved`: Dispute resolved with outcome
- `RefundProcessed`: Bounty refunded to creator

## 📄 License

This project is licensed under the Apache 2.0 License. See [LICENSE](https://github.com/devasignhq/soroban-contracts/blob/main/LICENSE) for more details.

<!-- ## 🤝 Contributing -->

## 🔗 Related Projects

- [DevAsign API Server](https://github.com/devasignhq/devasign-api) - Backend API and AI engine
- [DevAsign Project Maintainer App](https://github.com/devasignhq/app.devasign.com) - Frontend for project maintainer
- [DevAsign Contributor App](https://github.com/devasignhq/contributor.devasign.com) - Frontend for contributors


---
