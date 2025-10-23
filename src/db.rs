//! Transaction processing and account management
//!
//! This module contains all core types for financial transaction processing:
//! - Transaction types and error handling
//! - Account management with transaction history  
//! - Database for multi-client account management

use crate::fixed4::Fixed4;
use std::collections::HashMap;
use thiserror::Error;

// =============================================================================
// ERROR TYPES
// =============================================================================

#[derive(Debug, Error)]
pub enum MyError {
    /// Attempted withdrawal or operation when insufficient funds are available
    #[error("Insufficient funds")]
    InsufficientFunds,
    /// Attempted operation on a locked account (after chargeback)
    #[error("Account is locked")]
    AccountLocked,
    /// Referenced transaction ID does not exist in account ledger
    #[error("Transaction not found")]
    TransactionNotFound,
    /// Attempted to dispute a transaction that is already disputed
    #[error("Transaction already disputed")]
    TransactionAlreadyDisputed,
    /// Attempted operation on a transaction that has been charged back
    #[error("Transaction already charged back")]
    TransactionAlreadyChargedBack,
    /// Attempted to dispute a withdrawal transaction (only deposits can be disputed)
    #[error("Withdrawal transaction cannot be disputed")]
    TransactionIsWithdrawal,
    /// Attempted to resolve or chargeback a transaction that is not disputed
    #[error("Transaction is not disputed")]
    TransactionNotDisputed,
    /// Failed to parse amount string into valid Fixed4 decimal
    #[error("Invalid amount format: {0}")]
    InvalidAmountFormat(String),
    /// Attempted deposit or withdrawal with non-positive amount
    #[error("Amount must be positive")]
    AmountMustBePositive,
}

// =============================================================================
// TRANSACTION TYPES
// =============================================================================

/// Financial transaction operations
///
/// Represents the different types of financial transactions that can be processed:
/// - Basic operations: deposits and withdrawals
/// - Dispute resolution: dispute, resolve, and chargeback flows
pub enum Transaction {
    /// Add funds to an account
    Deposit {
        /// Amount to deposit (must be positive)
        amount: Fixed4,
    },
    /// Remove funds from an account (requires sufficient available balance)
    Withdrawal {
        /// Amount to withdraw (must be positive and ≤ available balance)
        amount: Fixed4,
    },
    /// Dispute a previous deposit transaction (moves funds from available to held)
    Dispute,
    /// Resolve a disputed transaction (moves funds back from held to available)
    Resolve,
    /// Chargeback a disputed transaction (removes funds and locks account)
    Chargeback,
}

impl Transaction {
    /// Create a deposit transaction from a string amount
    ///
    /// # Arguments
    /// * `amount` - String representation of amount with up to 4 decimal places
    ///
    /// # Examples
    /// ```
    /// # use transaction_processor::Transaction;
    /// let deposit = Transaction::deposit("123.45").unwrap();
    /// let small_deposit = Transaction::deposit("0.0001").unwrap();
    /// 
    /// // Zero and negative amounts are rejected
    /// assert!(Transaction::deposit("0").is_err());
    /// assert!(Transaction::deposit("-10.50").is_err());
    /// ```
    ///
    /// # Errors
    /// Returns [`MyError::InvalidAmountFormat`] if the string cannot be parsed
    /// Returns [`MyError::AmountMustBePositive`] if the amount is zero or negative
    pub fn deposit(amount: &str) -> Result<Self, MyError> {
        let amount: Fixed4 = amount.parse().map_err(MyError::InvalidAmountFormat)?;
        if amount <= Fixed4::zero() {
            return Err(MyError::AmountMustBePositive);
        }
        Ok(Self::Deposit { amount })
    }

    /// Create a withdrawal transaction from a string amount
    ///
    /// # Arguments
    /// * `amount` - String representation of amount with up to 4 decimal places
    ///
    /// # Examples
    /// ```
    /// # use transaction_processor::Transaction;
    /// let withdrawal = Transaction::withdrawal("50.00").unwrap();
    /// 
    /// // Zero and negative amounts are rejected
    /// assert!(Transaction::withdrawal("0").is_err());
    /// assert!(Transaction::withdrawal("-5.00").is_err());
    /// ```
    ///
    /// # Errors
    /// Returns [`MyError::InvalidAmountFormat`] if the string cannot be parsed
    /// Returns [`MyError::AmountMustBePositive`] if the amount is zero or negative
    pub fn withdrawal(amount: &str) -> Result<Self, MyError> {
        let amount: Fixed4 = amount.parse().map_err(MyError::InvalidAmountFormat)?;
        if amount <= Fixed4::zero() {
            return Err(MyError::AmountMustBePositive);
        }
        Ok(Self::Withdrawal { amount })
    }

    /// Create a dispute transaction
    ///
    /// Disputes move funds from available to held status for the referenced transaction.
    /// Only deposit transactions can be disputed.
    pub fn dispute() -> Self {
        Self::Dispute
    }

    /// Create a resolve transaction
    ///
    /// Resolves move funds from held back to available status for the referenced transaction.
    /// Can only be applied to currently disputed transactions.
    pub fn resolve() -> Self {
        Self::Resolve
    }

    /// Create a chargeback transaction
    ///
    /// Chargebacks remove held funds permanently and lock the account.
    /// Can only be applied to currently disputed transactions.
    pub fn chargeback() -> Self {
        Self::Chargeback
    }
}

/// Internal state tracking for deposit transactions
///
/// Deposits can be in different states during the dispute resolution process:
/// - Normal: Standard deposit, funds are available
/// - Disputed: Under dispute, funds moved to held status  
/// - ChargedBack: Permanently removed, account locked
#[derive(Debug)]
enum DepositState {
    /// Normal deposit state - funds are available for use
    Normal,
    /// Disputed state - funds are held pending resolution
    Disputed,
    /// Charged back state - funds permanently removed
    ChargedBack,
}

/// Internal ledger entries for transaction history
///
/// Each transaction is recorded in the account's ledger for audit trail and
/// dispute resolution. The ledger maintains the original transaction amounts
/// and states for regulatory compliance.
#[derive(Debug)]
enum LedgerEntry {
    /// Deposit transaction with amount and current dispute state
    Deposit {
        /// Original deposit amount
        amount: Fixed4,
        /// Current state in dispute resolution process
        state: DepositState,
    },
    /// Withdrawal transaction with amount (for audit trail)
    Withdrawal {
        /// Original withdrawal amount (stored for compliance)
        #[allow(dead_code)]
        amount: Fixed4,
    },
}

// =============================================================================
// ACCOUNT MANAGEMENT
// =============================================================================

/// Represents a client's account with financial transaction history
///
/// Uses HashMap for O(1) transaction lookups during disputes/resolves/chargebacks.
/// Maintains both current balance state and complete transaction history for
/// audit compliance and dispute resolution.
///
/// # Balance Types
/// - `available`: Funds available for withdrawal
/// - `held`: Funds held due to disputes (not available for withdrawal)
/// 
/// If a chargeback occurs, the account is locked and no further deposits or withdrawals
/// are allowed.
///
/// # Examples
/// ```
/// # use transaction_processor::{Database, Transaction};
/// let mut db = Database::new();
///
/// // Process a deposit
/// let deposit = Transaction::deposit("100.50").unwrap();
/// db.process_transaction(1, 1, deposit).unwrap();
///
/// // Check account state
/// let account = db.get_account(1).unwrap();
/// assert_eq!(account.available.to_f64(), 100.50);
/// assert_eq!(account.total().to_f64(), 100.50);
/// ```
#[derive(Debug)]
pub struct Account {
    /// Transaction ledger for audit trail and dispute resolution
    ledger: HashMap<u32, LedgerEntry>,
    /// Funds available for withdrawal
    pub available: Fixed4,
    /// Funds held due to disputes (not available for withdrawal)
    pub held: Fixed4,
    /// Account locked status (true after chargeback)
    pub locked: bool,
}

impl Account {
    /// Create a new empty account with zero balances
    fn new() -> Self {
        Self {
            ledger: HashMap::new(),
            available: Fixed4::zero(),
            held: Fixed4::zero(),
            locked: false,
        }
    }

    /// Calculate the total balance (available + held)
    ///
    /// Total balance represents all funds associated with the account,
    /// regardless of whether they are available for withdrawal or held.
    ///
    /// # Examples
    /// ```
    /// # use transaction_processor::{Database, Transaction};
    /// let mut db = Database::new();
    /// let deposit = Transaction::deposit("100.00").unwrap();
    /// db.process_transaction(1, 1, deposit).unwrap();
    ///
    /// let account = db.get_account(1).unwrap();
    /// assert_eq!(account.total().to_f64(), 100.00);
    /// ```
    pub fn total(&self) -> Fixed4 {
        self.available + self.held
    }

    /// Get transaction count for testing/audit purposes
    ///
    /// Returns the total number of transactions recorded in this account's ledger.
    /// Useful for audit trails and testing transaction history completeness.
    pub fn transaction_count(&self) -> usize {
        self.ledger.len()
    }

    /// Check if a transaction exists (for testing)
    ///
    /// # Arguments
    /// * `txn_id` - Transaction ID to check
    ///
    /// # Returns
    /// `true` if the transaction exists in the account's ledger, `false` otherwise
    pub fn has_transaction(&self, txn_id: u32) -> bool {
        self.ledger.contains_key(&txn_id)
    }

    /// Process a transaction for this account
    fn add_transaction(&mut self, txn_id: u32, txn: Transaction) -> Result<(), MyError> {
        match txn {
            Transaction::Deposit { amount } => {
                self.available += amount;
                self.ledger.insert(
                    txn_id,
                    LedgerEntry::Deposit {
                        amount,
                        state: DepositState::Normal,
                    },
                );
            }
            Transaction::Withdrawal { amount } => {
                if self.available >= amount {
                    self.available -= amount;
                    self.ledger
                        .insert(txn_id, LedgerEntry::Withdrawal { amount });
                } else {
                    return Err(MyError::InsufficientFunds);
                }
            }
            Transaction::Dispute => {
                let entry = self
                    .ledger
                    .get_mut(&txn_id)
                    .ok_or(MyError::TransactionNotFound)?;

                match entry {
                    LedgerEntry::Withdrawal { .. } => {
                        return Err(MyError::TransactionIsWithdrawal);
                    }
                    LedgerEntry::Deposit { amount, state } => match state {
                        DepositState::Normal => {
                            self.available -= *amount;
                            self.held += *amount;
                            *state = DepositState::Disputed;
                        }
                        DepositState::Disputed => {
                            return Err(MyError::TransactionAlreadyDisputed);
                        }
                        DepositState::ChargedBack => {
                            return Err(MyError::TransactionAlreadyChargedBack);
                        }
                    },
                }
            }
            Transaction::Resolve => {
                let entry = self
                    .ledger
                    .get_mut(&txn_id)
                    .ok_or(MyError::TransactionNotFound)?;
                match entry {
                    LedgerEntry::Withdrawal { .. } => {
                        return Err(MyError::TransactionIsWithdrawal);
                    }
                    LedgerEntry::Deposit { amount, state } => match state {
                        DepositState::Disputed => {
                            self.held -= *amount;
                            self.available += *amount;
                            *state = DepositState::Normal;
                        }
                        DepositState::Normal => {
                            return Err(MyError::TransactionNotDisputed);
                        }
                        DepositState::ChargedBack => {
                            return Err(MyError::TransactionAlreadyChargedBack);
                        }
                    },
                }
            }
            Transaction::Chargeback => {
                let entry = self
                    .ledger
                    .get_mut(&txn_id)
                    .ok_or(MyError::TransactionNotFound)?;
                match entry {
                    LedgerEntry::Withdrawal { .. } => {
                        return Err(MyError::TransactionIsWithdrawal);
                    }
                    LedgerEntry::Deposit { amount, state } => match state {
                        DepositState::ChargedBack => {
                            return Err(MyError::TransactionAlreadyChargedBack);
                        }
                        DepositState::Normal => {
                            return Err(MyError::TransactionNotDisputed);
                        }
                        DepositState::Disputed => {
                            self.held -= *amount;
                            *state = DepositState::ChargedBack;
                            self.locked = true;
                        }
                    },
                }
            }
        }
        Ok(())
    }
}

// =============================================================================
// DATABASE
// =============================================================================

/// In-memory database for managing client accounts and transactions
///
/// The Database manages multiple client accounts and processes financial transactions.
/// It ensures data consistency, handles error conditions, and maintains audit trails
/// for regulatory compliance.
///
/// # Features
/// - Multi-client account management
/// - Transaction processing with balance validation
/// - Dispute resolution workflow (dispute → resolve/chargeback)
/// - Account locking after chargebacks
/// - Complete audit trail maintenance
///
/// # Examples
/// ```
/// # use transaction_processor::{Database, Transaction};
/// let mut db = Database::new();
///
/// // Process transactions for different clients
/// let deposit1 = Transaction::deposit("100.00").unwrap();
/// let deposit2 = Transaction::deposit("200.00").unwrap();
///
/// db.process_transaction(1, 1, deposit1).unwrap();
/// db.process_transaction(2, 2, deposit2).unwrap();
///
/// // Check balances
/// assert_eq!(db.get_account(1).unwrap().available.to_f64(), 100.00);
/// assert_eq!(db.get_account(2).unwrap().available.to_f64(), 200.00);
/// ```
#[derive(Debug, Default)]
pub struct Database {
    /// Map of client IDs to their accounts
    accounts: HashMap<u16, Account>,
}

impl Database {
    /// Create a new empty database
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    /// Process a financial transaction for a client
    ///
    /// Creates a new account if the client doesn't exist. Validates business rules
    /// such as sufficient funds for withdrawals and account lock status.
    ///
    /// # Arguments
    /// * `client_id` - Unique identifier for the client
    /// * `txn_id` - Unique identifier for this transaction
    /// * `transaction` - The transaction to process
    ///
    /// # Examples
    /// ```
    /// # use transaction_processor::{Database, Transaction};
    /// let mut db = Database::new();
    ///
    /// // Process a deposit
    /// let deposit = Transaction::deposit("100.00").unwrap();
    /// db.process_transaction(1, 1, deposit).unwrap();
    ///
    /// // Process a withdrawal  
    /// let withdrawal = Transaction::withdrawal("25.00").unwrap();
    /// db.process_transaction(1, 2, withdrawal).unwrap();
    ///
    /// let account = db.get_account(1).unwrap();
    /// assert_eq!(account.available.to_f64(), 75.00);
    /// ```
    ///
    /// # Errors
    /// - [`MyError::InsufficientFunds`] - Withdrawal amount exceeds available balance
    /// - [`MyError::AccountLocked`] - Attempted deposit/withdrawal on locked account
    /// - [`MyError::TransactionNotFound`] - Dispute/resolve/chargeback on non-existent transaction
    /// - Other transaction-specific errors (see [`MyError`] for complete list)
    pub fn process_transaction(
        &mut self,
        client_id: u16,
        txn_id: u32,
        transaction: Transaction,
    ) -> Result<(), MyError> {
        self.accounts.entry(client_id).or_insert_with( Account::new);
        let account = self.accounts.get_mut(&client_id).unwrap();

        // Only check if account is locked for deposit/withdrawal transactions
        // Dispute, resolve, and chargeback operations should be allowed on locked accounts
        match transaction {
            Transaction::Deposit { .. } | Transaction::Withdrawal { .. } => {
                if account.locked {
                    return Err(MyError::AccountLocked);
                }
            }
            Transaction::Dispute | Transaction::Resolve | Transaction::Chargeback => {
                // These operations are allowed on locked accounts
            }
        }

        account.add_transaction(txn_id, transaction)
    }

    /// Get an account by client ID
    ///
    /// # Arguments
    /// * `client_id` - Unique identifier for the client
    ///
    /// # Returns
    /// `Some(&Account)` if the client exists, `None` otherwise
    ///
    /// # Examples
    /// ```
    /// # use transaction_processor::{Database, Transaction};
    /// let mut db = Database::new();
    ///
    /// // Account doesn't exist yet
    /// assert!(db.get_account(1).is_none());
    ///
    /// // Create account by processing transaction
    /// let deposit = Transaction::deposit("100.00").unwrap();
    /// db.process_transaction(1, 1, deposit).unwrap();
    ///
    /// // Now account exists
    /// let account = db.get_account(1).unwrap();
    /// assert_eq!(account.available.to_f64(), 100.00);
    /// ```
    pub fn get_account(&self, client_id: u16) -> Option<&Account> {
        self.accounts.get(&client_id)
    }

    /// Get all client IDs that have accounts
    ///
    /// Returns a vector of all client IDs that have processed at least one transaction.
    /// Useful for generating reports or iterating over all accounts.
    ///
    /// # Examples
    /// ```
    /// # use transaction_processor::{Database, Transaction};
    /// let mut db = Database::new();
    ///
    /// // No clients initially
    /// assert!(db.get_all_client_ids().is_empty());
    ///
    /// // Add some clients
    /// let deposit = Transaction::deposit("100.00").unwrap();
    /// db.process_transaction(1, 1, deposit).unwrap();
    /// db.process_transaction(3, 2, Transaction::deposit("200.00").unwrap()).unwrap();
    ///
    /// let mut client_ids = db.get_all_client_ids();
    /// client_ids.sort();
    /// assert_eq!(client_ids, vec![1, 3]);
    /// ```
    pub fn get_all_client_ids(&self) -> Vec<u16> {
        self.accounts.keys().copied().collect()
    }
}
