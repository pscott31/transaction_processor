# Transaction Processor

A financial transaction processing system that handles deposits, withdrawals, and dispute resolution. 

## What it does

Processes CSV files containing financial transactions and outputs account summaries. Handles the complete dispute workflow including disputes, resolutions, and chargebacks.

## Quick Start

Create a transactions file (or use the examples in `tests/csv/`):

```csv
type,client,tx,amount
deposit,1,1,100.50
withdrawal,1,2,25.25
deposit,2,3,200.00
dispute,1,1,
resolve,1,1,
dispute,2,3,
```

Run it:

```bash
cargo run -- transactions.csv
```

You'll get:
```
client,available,held,total,locked
1,75.2500,0.0000,75.2500,false
2,0.0000,200.0000,200.0000,false
```

By default, errors in the input are silently ignored. If you pass `-v` or `--verbose`.

## Features

- **Precise decimal arithmetic** - Uses fixed precision of up to four decimal places to avoid floating point rounding errors
- **Dispute handling** - Complete workflow from dispute through resolution or chargeback
- **Negative balances** - Handles edge cases like disputing a deposit after withdrawals have occurred
- **Account locking** - Accounts are locked after chargebacks to prevent further transactions
- **Error handling** - Continues processing on invalid data with optional verbose error reporting

## Running Tests

The project includes comprehensive test coverage:

**Run all tests:**
```bash
cargo test
```

**Run BDD tests:**
```bash
cargo test --test cucumber
```

**Run integration tests:**
```bash
cargo test --test integration_tests
```

The test suite includes:
- Business scenarios written in natural language (BDD style)
- End-to-end CSV processing tests
- Documentation tests to ensure examples work correctly

Tests cover normal operations, edge cases, error conditions, and negative balance scenarios.

## Documentation

Generate API documentation:

```bash
cargo doc --open
```

This creates comprehensive documentation with working examples. All code samples in the documentation are tested.

## CLI Usage

Basic usage:
```bash
cargo run -- input.csv
```

With verbose error reporting:
```bash
cargo run -- input.csv --verbose
```

The `--verbose` flag provides detailed error messages for any problematic transactions.

## Input Format

CSV files should have this format:
```csv
type,client,tx,amount
deposit,1,1,100.0
withdrawal,1,2,50.0
dispute,1,1,
resolve,1,1,
chargeback,1,1,
```

- **type**: deposit, withdrawal, dispute, resolve, chargeback
- **client**: u16 client ID  
- **tx**: u32 transaction ID
- **amount**: decimal string (required for deposit/withdrawal, ignored for others)

## Technical Notes

**Why Fixed4?** Because `0.1 + 0.2 != 0.3` in floating point math, and that's unacceptable when dealing with money. Fixed4 stores amounts as integers (scaled by 10,000) for exact precision.

**Negative balances?** Consider this scenario: deposit $100, withdraw $75 (balance: $25), then someone disputes the original deposit. Now you have available: -$75, held: $100, total: $25. I considered hiding this from the user output and just displaying 0 when the balance is negative, but I think that is more confusing.

**Performance?** Uses HashMap for O(1) transaction lookups during dispute resolution. Should handle large transaction volumes just fine.

**Locking** After a chargeback transaction, the account is locked and additional withdrawals or deposits are rejected. I chose to continue to allow dispute/resolution/chargeback transactions as it seems feasable that a user may challenge more than one transaction.

## Design Decisions

Some choices I made and why:

**In-memory vs database:** Went with in memory storage for simplicity and performance. For production with millions of transactions, I'd swap in PostgreSQL or SQLite - the architecture makes this pretty straightforward.

**Custom fixed point type instead of using a crate like `rust_decimal`** To save a bit of space; rust decimals are 96 bits, mine are 64, and the spec hinted that efficiency was something that would be scored.

**Negative balances vs dispute tracking:** Debated whether to allow negative account balances or track dispute amounts separately. Chose negative balances because:
- It's simpler (account state is obvious at a glance)
- Real banks work this way (overdrafts are a thing)
- The math stays clean (total = available + held)

Alternative would be storing dispute deltas in the ledger and calculating balances on-demand.

## Building

```bash
# Development build
cargo build

# Optimized release build  
cargo build --release
```

## Library Usage

You can also use this as a library in other Rust projects:

```rust
use transaction_processor::{Database, Transaction};

let mut db = Database::new();

// Process some transactions
let deposit = Transaction::deposit("100.50")?;
db.process_transaction(1, 1, deposit)?;

let withdrawal = Transaction::withdrawal("25.25")?;  
db.process_transaction(1, 2, withdrawal)?;

// Check the balance
let account = db.get_account(1).unwrap();
println!("Available: {}", account.available);
```