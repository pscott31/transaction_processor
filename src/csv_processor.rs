use crate::{Database, Transaction};
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct TransactionRecord {
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<String>, // Optional because dispute, resolve, chargeback don't have amounts
}

pub fn process_csv_file(file_path: &str) -> Result<(Database, Vec<String>), Box<dyn Error>> {
    let mut database = Database::new();
    let mut errors = Vec::new();

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All) // Trim whitespace from both headers and fields
        .from_path(file_path)?;

    for (line_num, result) in reader.deserialize().enumerate() {
        let line_number = line_num + 2; // +1 for 0-based index, +1 for header row

        let record: TransactionRecord = match result {
            Ok(record) => record,
            Err(e) => {
                errors.push(format!(
                    "Error parsing CSV at {}:{}: {}",
                    file_path, line_number, e
                ));
                continue;
            }
        };

        // Process the transaction
        if let Err(e) = process_transaction_record(&mut database, record) {
            errors.push(format!(
                "Error processing transaction at {}:{}: {}",
                file_path, line_number, e
            ));
            continue;
        }
    }

    Ok((database, errors))
}

fn process_transaction_record(
    database: &mut Database,
    record: TransactionRecord,
) -> Result<(), Box<dyn Error>> {
    let transaction = match record.transaction_type.to_lowercase().as_str() {
        "deposit" => {
            let amount = record.amount.ok_or("Deposit requires an amount")?;
            Transaction::deposit(&amount)?
        }
        "withdrawal" => {
            let amount = record.amount.ok_or("Withdrawal requires an amount")?;
            Transaction::withdrawal(&amount)?
        }
        "dispute" => Transaction::dispute(),
        "resolve" => Transaction::resolve(),
        "chargeback" => Transaction::chargeback(),
        _ => return Err(format!("Unknown transaction type: {}", record.transaction_type).into()),
    };

    database.process_transaction(record.client, record.tx, transaction)?;
    Ok(())
}
