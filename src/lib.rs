//! # Transaction Processing Engine
//!
//! A robust financial transaction processing system with support for:
//! - Precise decimal arithmetic using fixed-point numbers
//! - Multi-client account management  
//! - Dispute resolution workflows (dispute → resolve/chargeback)
//! - CSV transaction file processing
//! - Comprehensive error handling and audit trails
//!
//! ## Quick Start
//!
//! ```rust
//! use transaction_processor::{Database, Transaction};
//!
//! let mut db = Database::new();
//!
//! // Process a deposit
//! let deposit = Transaction::deposit("100.50")?;
//! db.process_transaction(1, 1, deposit)?;
//!
//! // Process a withdrawal  
//! let withdrawal = Transaction::withdrawal("25.25")?;
//! db.process_transaction(1, 2, withdrawal)?;
//!
//! // Check account balance
//! let account = db.get_account(1).unwrap();
//! assert_eq!(account.available.to_f64(), 75.25);
//! # Ok::<(), transaction_processor::MyError>(())
//! ```
//!
//! ## Modules
//!
//! - [`db`] - Core transaction processing and account management
//! - [`fixed4`] - Fixed-point decimal arithmetic with 4 decimal places
//! - [`csv_processor`] - CSV file processing utilities

pub mod csv_processor;
pub mod db;
pub mod fixed4;
pub use csv_processor::*;
pub use db::*;
pub use fixed4::*;
