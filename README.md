# 🪙 Token BDB - Fungible Token Smart Contract for Stellar

A production-ready fungible token smart contract built with **Soroban** (Stellar's smart contract platform), implementing the **CAP-46** standard for token interoperability.

## 📋 Overview

Token BDB is a fully-featured ERC-20-like token implementation on Stellar, compatible with wallets, DEXs, and the entire Stellar ecosystem. It follows best practices for security, gas efficiency, and storage optimization. Made For Codigo Futura by Buen dia Builders.

## ✨ Features

- ✅ **CAP-46 Standard Compliance** - Full compatibility with Stellar's fungible token standard
- 🔐 **Access Control** - Admin-only minting with proper authorization checks
- 💰 **Core Token Operations** - Transfer, approve, allowance, mint, and burn
- 📊 **Rich Event Emission** - Detailed events for indexers and explorers
- ⚡ **Gas Optimized** - Smart storage management with automatic cleanup
- 🛡️ **Overflow Protection** - Safe arithmetic operations with checked math
- ⏰ **TTL Management** - Automatic storage lifetime extension
- 🎯 **Zero-balance Optimization** - Automatic key removal for efficiency

## 🏗️ Project Structure

```
src/
├── lib.rs        # Main contract implementation
├── storage.rs    # Storage keys and data structures
├── errors.rs     # Custom error definitions
└── test.rs       # Unit tests
```

## 📦 Storage Architecture

### Instance Storage (Global Metadata)
- `Admin` - Contract administrator address
- `TokenName` - Full token name (max 100 chars)
- `TokenSymbol` - Token ticker symbol (max 32 chars)
- `Decimals` - Token precision (max 18, typically 7 for Stellar)
- `TotalSupply` - Total tokens in circulation
- `Initialized` - Initialization flag

### Persistent Storage (User Data)
- `Balance(Address)` - Individual user balances
- `Allowance(Address, Address)` - Spending permissions between accounts

## 🚀 Contract Functions

### Initialization

```rust
fn initialize(
    env: Env,
    admin: Address,
    name: String,
    symbol: String,
    decimals: u32
) -> Result<(), TokenError>
```

Initializes the token with metadata. Can only be called once.

### Core Operations

#### `mint(env, to, amount)`
Creates new tokens and adds them to recipient's balance. **Admin only**.

#### `burn(env, from, amount)`
Destroys tokens from the specified account. Requires owner authorization.

#### `transfer(env, from, to, amount)`
Transfers tokens between accounts. Requires sender authorization.

#### `approve(env, from, spender, amount)`
Grants spending permission to another address. Set to 0 to revoke.

#### `transfer_from(env, spender, from, to, amount)`
Transfers tokens on behalf of another user using approved allowance.

### Query Functions

- `balance(env, account)` - Returns account balance
- `allowance(env, from, spender)` - Returns approved spending amount
- `name(env)` - Returns token name
- `symbol(env)` - Returns token symbol
- `decimals(env)` - Returns decimal places
- `total_supply(env)` - Returns total token supply
- `admin(env)` - Returns administrator address

## 🔒 Security Features

### Input Validation
- ✅ Amount must be positive (> 0)
- ✅ No self-transfers allowed
- ✅ Metadata length validation
- ✅ Decimal places capped at 18

### Authorization
- 🔑 Admin signature required for minting
- 🔑 Owner signature required for transfers and burns
- 🔑 Spender signature required for transfer_from

### Overflow Protection
- ➕ All arithmetic uses `checked_add`/`checked_sub`
- 🛡️ Returns `OverflowError` on overflow detection

## ⚙️ Error Codes

| Error | Code | Description |
|-------|------|-------------|
| `AlreadyInitialized` | 1 | Contract already initialized |
| `InvalidAmount` | 2 | Amount must be > 0 |
| `InsufficientBalance` | 3 | Not enough tokens |
| `InsufficientAllowance` | 4 | Allowance too low |
| `NotInitialized` | 5 | Contract not initialized |
| `InvalidDecimals` | 6 | Decimals > 18 |
| `OverflowError` | 7 | Arithmetic overflow |
| `InvalidRecipient` | 8 | Cannot transfer to self |
| `InvalidMetadata` | 9 | Invalid name/symbol |

## 📡 Events

The contract emits rich events for all operations:

- `init` - Token initialization
- `mint` - Token creation
- `burn` - Token destruction
- `transfer` - Token transfers
- `approve` - Allowance approvals
- `trnsf_frm` - Delegated transfers

Each event includes relevant data like amounts, balances, and addresses.

## 🛠️ Building & Testing

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Soroban CLI](https://soroban.stellar.org/docs/getting-started/setup)

### Build

```bash
cargo build --target wasm32-unknown-unknown --release
```

### Test

```bash
cargo test
```

### Deploy to Testnet

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/token_bdb.wasm \
  --network testnet
```

## 🎯 Usage Example

```rust
// Initialize the token
token.initialize(
    &env,
    &admin,
    String::from_str(&env, "Buen Día Builders Token"),
    String::from_str(&env, "BDB"),
    7
);

// Mint tokens to a user
token.mint(&env, &user, &1_000_0000000); // 1000 tokens with 7 decimals

// Transfer tokens
token.transfer(&env, &user, &recipient, &100_0000000); // 100 tokens

// Approve spending
token.approve(&env, &user, &spender, &50_0000000); // 50 tokens

// Transfer on behalf
token.transfer_from(&env, &spender, &user, &recipient, &25_0000000);
```

## 📝 Best Practices Implemented

- ✅ **Check-Effects-Interactions Pattern** - State updates before external calls
- ✅ **Storage Optimization** - Remove zero-balance entries
- ✅ **TTL Management** - Extend lifetime for active accounts (100k-200k ledgers)
- ✅ **Event-Driven Architecture** - Rich events for off-chain indexing
- ✅ **Fail-Fast Validation** - Early input validation to save gas
- ✅ **Atomic Operations** - All state changes within single transaction

## 🌟 Stellar-Specific Optimizations

- Uses 7 decimals (Stellar standard) by default
- Instance storage for global data (cheaper than persistent)
- Persistent storage with TTL for user data
- Short symbol names for events (max 9 chars)
- Efficient key removal on zero balances

## 📚 Resources

- [Soroban Documentation](https://soroban.stellar.org/docs)
- [CAP-46 Token Standard](https://stellar.org/protocol/cap-46)
- [Stellar Developer Discord](https://discord.gg/stellardev)

## 📄 License

- [Buen dia Builders Project](https://www.buendiabuilders.com/)

## 🤝 Contributing

Contributions are welcome! Please open an issue or pull request.

---

Built with ❤️ for Codigo Futura by Buen dia Builders for Stellar ecosystem# token_bdb
