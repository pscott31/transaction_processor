use std::io::Write;
use tempfile::NamedTempFile;

// Import the CSV processing function from main.rs
use transaction_processor::process_csv_file;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_temp_csv(content: &str) -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(content.as_bytes())
            .expect("Failed to write to temp file");
        temp_file
    }

    #[test]
    fn test_basic_transactions() {
        let csv_content = r#"type,client,tx,amount
deposit,1,1,1.0
deposit,2,2,2.0
deposit,1,3,2.0
withdrawal,1,4,1.5
withdrawal,2,5,3.0"#;

        let temp_file = create_temp_csv(csv_content);
        let (database, errors) = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();

        // Should have one error (insufficient funds for client 2)
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Insufficient funds"));
        assert!(errors[0].contains("6")); // Line 6

        // Resolve should move funds back to available
        let account1 = database.get_account(1).unwrap();
        assert_eq!(account1.available.to_f64(), 1.5);
        assert_eq!(account1.held.to_f64(), 0.0);
        assert_eq!(account1.total().to_f64(), 1.5);
        assert!(!account1.locked);

        // Check client 2: deposited 2.0, withdrawal failed, should still have 2.0
        let account2 = database.get_account(2).unwrap();
        assert_eq!(account2.available.to_f64(), 2.0);
        assert_eq!(account2.held.to_f64(), 0.0);
        assert_eq!(account2.total().to_f64(), 2.0);
        assert!(!account2.locked);
    }

    #[test]
    fn test_advanced_transactions_with_disputes() {
        let csv_content = r#"type,client,tx,amount
deposit,1,1,1.0
deposit,2,2,2.0
deposit,1,3,2.0
withdrawal,1,4,1.5
withdrawal,2,5,3.0
dispute,1,1,
resolve,1,1,
deposit,1,6,1.0
dispute,1,3,
chargeback,1,3,"#;

        let temp_file = create_temp_csv(csv_content);
        let (database, errors) = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();

        // Should have one error (insufficient funds for client 2)
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Insufficient funds"));

        // Verify final state after chargeback
        let account1 = database.get_account(1).unwrap();
        assert_eq!(account1.available.to_f64(), 0.5);
        assert_eq!(account1.held.to_f64(), 0.0);
        assert_eq!(account1.total().to_f64(), 0.5);
        assert!(account1.locked);

        // Check client 2: unchanged
        let account2 = database.get_account(2).unwrap();
        assert_eq!(account2.available.to_f64(), 2.0);
        assert!(!account2.locked);
    }

    #[test]
    fn test_transaction_type_errors() {
        let csv_content = r#"type,client,tx,amount
deposit,1,1,1.0
invalid_transaction,2,2,2.0
deposit,1,3,abc
deposit,3,4,5.0"#;

        let temp_file = create_temp_csv(csv_content);
        let (database, errors) = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();

        // Should have 2 errors
        assert_eq!(errors.len(), 2);

        // Check error messages contain line numbers and expected errors
        assert!(
            errors[0].contains("3")
                && errors[0].contains("Unknown transaction type: invalid_transaction")
        );
        assert!(errors[1].contains("4") && errors[1].contains("Invalid amount format"));

        // Check that valid transactions still processed
        let account1 = database.get_account(1).unwrap();
        assert_eq!(account1.available.to_f64(), 1.0);

        let account3 = database.get_account(3).unwrap();
        assert_eq!(account3.available.to_f64(), 5.0);
    }

    #[test]
    fn test_csv_parsing_errors() {
        let csv_content = r#"type,client,tx,amount
deposit,1,1,1.0
deposit,not_a_number,2,2.0
deposit,3,not_a_number,1.5
deposit,4,3,0.5"#;

        let temp_file = create_temp_csv(csv_content);
        let (database, errors) = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();

        // Should have 2 CSV parsing errors
        assert_eq!(errors.len(), 2);

        // Check error messages contain line numbers and parsing errors
        assert!(errors[0].contains("3") && errors[0].contains("invalid digit found in string"));
        assert!(errors[1].contains("4") && errors[1].contains("invalid digit found in string"));

        // Check that valid transactions still processed
        let account1 = database.get_account(1).unwrap();
        assert_eq!(account1.available.to_f64(), 1.0);

        let account4 = database.get_account(4).unwrap();
        assert_eq!(account4.available.to_f64(), 0.5);
    }

    #[test]
    fn test_precision_handling() {
        let csv_content = r#"type,client,tx,amount
deposit,1,1,0.0001
deposit,1,2,0.9999
deposit,2,3,123.45678
withdrawal,1,4,1.0"#;

        let temp_file = create_temp_csv(csv_content);
        let (database, errors) = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();

        // Should have one error for too many decimal places
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Too many decimal places"));

        // Check precision handling
        let account1 = database.get_account(1).unwrap();
        assert_eq!(account1.available.to_f64(), 0.0); // 0.0001 + 0.9999 - 1.0 = 0.0
        assert_eq!(account1.total().to_f64(), 0.0);

        // Client 2 transaction should have failed due to precision error
        assert!(database.get_account(2).is_none());
    }

    #[test]
    fn test_empty_file() {
        let csv_content = "type,client,tx,amount\n";

        let temp_file = create_temp_csv(csv_content);
        let (database, errors) = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();

        // No errors, no accounts
        assert_eq!(errors.len(), 0);
        assert_eq!(database.get_all_client_ids().len(), 0);
    }

    #[test]
    fn test_dispute_nonexistent_transaction() {
        let csv_content = r#"type,client,tx,amount
deposit,1,1,100.0
dispute,1,999,"#;

        let temp_file = create_temp_csv(csv_content);
        let (database, errors) = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();

        // Should have one error for transaction not found
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Transaction not found"));
        assert!(errors[0].contains("3")); // Line 3

        // Original deposit should still be there
        let account1 = database.get_account(1).unwrap();
        assert_eq!(account1.available.to_f64(), 100.0);
        assert!(!account1.locked);
    }

    #[test]
    fn test_transaction_audit_trail() {
        let csv_content = r#"type,client,tx,amount
deposit,1,1,100.0
withdrawal,1,2,25.5
deposit,1,3,50.0"#;

        let temp_file = create_temp_csv(csv_content);
        let (database, errors) = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(errors.len(), 0);

        let account1 = database.get_account(1).unwrap();
        // Verify final balance
        assert_eq!(account1.available.to_f64(), 124.5); // 100 - 25.5 + 50

        // Verify all transactions are stored
        assert_eq!(account1.transaction_count(), 3);
        assert!(account1.has_transaction(1)); // deposit
        assert!(account1.has_transaction(2)); // withdrawal  
        assert!(account1.has_transaction(3)); // deposit
    }

    #[test]
    fn test_whitespace_in_headers_and_fields() {
        // Test CSV with spaces after commas in header (like the demo test.csv)
        let csv_content = r#"type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
withdrawal, 1, 4, 0.5"#;

        let temp_file = create_temp_csv(csv_content);
        let (database, errors) = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(errors.len(), 0);

        let account1 = database.get_account(1).unwrap();
        assert_eq!(account1.available.to_f64(), 0.5); // 1.0 - 0.5

        let account2 = database.get_account(2).unwrap();
        assert_eq!(account2.available.to_f64(), 2.0);
    }

    #[test]
    fn test_mixed_whitespace_scenarios() {
        // Test various whitespace combinations: tabs, spaces before and after values
        let csv_content = "type,client,tx,amount\n\
                          deposit,\t1\t,1, 100.50 \n\
                          withdrawal, 1 ,2,25.25\n\
                          dispute,1, 1 ,\n\
                          resolve, 1,1,";

        let temp_file = create_temp_csv(csv_content);
        let (database, errors) = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(errors.len(), 0);

        let account1 = database.get_account(1).unwrap();
        // After dispute and resolve, should have original balance
        assert_eq!(account1.available.to_f64(), 75.25); // 100.50 - 25.25
        assert_eq!(account1.held.to_f64(), 0.0);
    }

    #[test]
    fn test_leading_and_trailing_whitespace_in_amounts() {
        // Test amounts with leading/trailing spaces and tabs
        let csv_content = r#"type,client,tx,amount
deposit,1,1,  123.45  
deposit,2,2,	99.99	
withdrawal,1,3,  50.00"#;

        let temp_file = create_temp_csv(csv_content);
        let (database, errors) = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(errors.len(), 0);

        let account1 = database.get_account(1).unwrap();
        assert_eq!(account1.available.to_f64(), 73.45); // 123.45 - 50.00

        let account2 = database.get_account(2).unwrap();
        assert_eq!(account2.available.to_f64(), 99.99);
    }

    #[test]
    fn test_whitespace_with_transaction_types() {
        // Test transaction types with various whitespace
        let csv_content = r#"type,client,tx,amount
 deposit ,1,1,100.0
	withdrawal	,1,2,25.0
 dispute ,1,1,
	resolve	,1,1,
 chargeback ,1,1,"#;

        let temp_file = create_temp_csv(csv_content);
        let (database, errors) = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();

        // Should have one error - chargeback after resolve puts transaction in normal state
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Transaction is not disputed"));

        let account1 = database.get_account(1).unwrap();
        assert_eq!(account1.available.to_f64(), 75.0); // 100.0 - 25.0 (after resolve)
        assert_eq!(account1.held.to_f64(), 0.0);
        assert!(!account1.locked); // Chargeback failed, so not locked
    }

    #[test]
    fn test_demo_test_csv_format() {
        // Exact format from the specification's demo test.csv
        let csv_content = r#"type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
withdrawal, 2, 5, 3.0"#;

        let temp_file = create_temp_csv(csv_content);
        let (database, errors) = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();

        // Should have one error (insufficient funds for client 2's withdrawal)
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Insufficient funds"));

        // Client 1: 1.0 + 2.0 - 1.5 = 1.5
        let account1 = database.get_account(1).unwrap();
        assert_eq!(account1.available.to_f64(), 1.5);
        assert_eq!(account1.total().to_f64(), 1.5);

        // Client 2: 2.0 (withdrawal of 3.0 failed)
        let account2 = database.get_account(2).unwrap();
        assert_eq!(account2.available.to_f64(), 2.0);
        assert_eq!(account2.total().to_f64(), 2.0);
    }
}
