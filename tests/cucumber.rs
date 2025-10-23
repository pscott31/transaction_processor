use cucumber::{World, given, then, when};
use transaction_processor::{Database, MyError, Transaction};

#[derive(Debug, Default, World)]
pub struct DatabaseWorld {
    database: Database,
    last_error: Option<MyError>,
}

impl DatabaseWorld {
    fn assert_balance(&self, client_id: u16, expected: f64, balance_type: &str) {
        let summary = self
            .database
            .get_account(client_id)
            .expect(&format!("Client {} should have an account", client_id));

        let actual = match balance_type {
            "available" => summary.available.to_f64(),
            "held" => summary.held.to_f64(),
            "total" => summary.total().to_f64(),
            _ => panic!("Unknown balance type: {}", balance_type),
        };

        assert!(
            (actual - expected).abs() < 0.0001,
            "Expected {} balance of {} but got {} for client {}",
            balance_type,
            expected,
            actual,
            client_id
        );
    }
}

#[given("a new database")]
fn given_new_database(world: &mut DatabaseWorld) {
    world.database = Database::new();
    world.last_error = None;
}

#[when(
    regex = r"^I process a deposit of ([0-9.]+) for client ([0-9]+) with transaction id ([0-9]+)$"
)]
#[given(
    regex = r"^I process a deposit of ([0-9.]+) for client ([0-9]+) with transaction id ([0-9]+)$"
)]
fn when_process_deposit(world: &mut DatabaseWorld, amount: String, client_id: u16, txn_id: u32) {
    let transaction = match Transaction::deposit(&amount) {
        Ok(txn) => txn,
        Err(e) => {
            world.last_error = Some(e);
            return;
        }
    };

    let result = world
        .database
        .process_transaction(client_id, txn_id, transaction);

    if let Err(err) = result {
        world.last_error = Some(err);
    } else {
        world.last_error = None;
    }
}

#[when(
    regex = r"^I process a withdrawal of ([0-9.]+) for client ([0-9]+) with transaction id ([0-9]+)$"
)]
#[given(
    regex = r"^I process a withdrawal of ([0-9.]+) for client ([0-9]+) with transaction id ([0-9]+)$"
)]
fn when_process_withdrawal(world: &mut DatabaseWorld, amount: String, client_id: u16, txn_id: u32) {
    let transaction = match Transaction::withdrawal(&amount) {
        Ok(txn) => txn,
        Err(e) => {
            world.last_error = Some(e);
            return;
        }
    };

    let result = world
        .database
        .process_transaction(client_id, txn_id, transaction);

    if let Err(err) = result {
        world.last_error = Some(err);
    } else {
        world.last_error = None;
    }
}

#[when(
    regex = r"^I attempt to process a deposit of ([0-9.]+) for client ([0-9]+) with transaction id ([0-9]+)$"
)]
fn when_attempt_deposit(world: &mut DatabaseWorld, amount: String, client_id: u16, txn_id: u32) {
    let transaction = match Transaction::deposit(&amount) {
        Ok(txn) => txn,
        Err(e) => {
            world.last_error = Some(e);
            return;
        }
    };

    let result = world
        .database
        .process_transaction(client_id, txn_id, transaction);

    world.last_error = result.err();
}

#[when(
    regex = r"^I attempt to process a withdrawal of ([0-9.]+) for client ([0-9]+) with transaction id ([0-9]+)$"
)]
fn when_attempt_withdrawal(world: &mut DatabaseWorld, amount: String, client_id: u16, txn_id: u32) {
    let transaction = match Transaction::withdrawal(&amount) {
        Ok(txn) => txn,
        Err(e) => {
            world.last_error = Some(e);
            return;
        }
    };

    let result = world
        .database
        .process_transaction(client_id, txn_id, transaction);

    world.last_error = result.err();
}

#[when(regex = r"^I dispute transaction ([0-9]+) for client ([0-9]+)$")]
#[given(regex = r"^I dispute transaction ([0-9]+) for client ([0-9]+)$")]
fn when_dispute_transaction(world: &mut DatabaseWorld, txn_id: u32, client_id: u16) {
    let result = world.database.process_transaction(
        client_id,
        txn_id, // Use the original transaction ID to dispute
        Transaction::dispute(),
    );

    if let Err(err) = result {
        world.last_error = Some(err);
    } else {
        world.last_error = None;
    }
}

#[when(regex = r"^I attempt to dispute transaction ([0-9]+) for client ([0-9]+)$")]
fn when_attempt_dispute(world: &mut DatabaseWorld, txn_id: u32, client_id: u16) {
    let result = world
        .database
        .process_transaction(client_id, txn_id, Transaction::dispute());

    world.last_error = result.err();
}

#[when(regex = r"^I resolve transaction ([0-9]+) for client ([0-9]+)$")]
#[given(regex = r"^I resolve transaction ([0-9]+) for client ([0-9]+)$")]
fn when_resolve_transaction(world: &mut DatabaseWorld, txn_id: u32, client_id: u16) {
    let result = world
        .database
        .process_transaction(client_id, txn_id, Transaction::resolve());

    if let Err(err) = result {
        world.last_error = Some(err);
    } else {
        world.last_error = None;
    }
}

#[when(regex = r"^I attempt to resolve transaction ([0-9]+) for client ([0-9]+)$")]
fn when_attempt_resolve(world: &mut DatabaseWorld, txn_id: u32, client_id: u16) {
    let result = world
        .database
        .process_transaction(client_id, txn_id, Transaction::resolve());

    world.last_error = result.err();
}

#[when(regex = r"^I chargeback transaction ([0-9]+) for client ([0-9]+)$")]
#[given(regex = r"^I chargeback transaction ([0-9]+) for client ([0-9]+)$")]
fn when_chargeback_transaction(world: &mut DatabaseWorld, txn_id: u32, client_id: u16) {
    let result = world
        .database
        .process_transaction(client_id, txn_id, Transaction::chargeback());

    if let Err(err) = result {
        world.last_error = Some(err);
    } else {
        world.last_error = None;
    }
}

#[when(regex = r"^I attempt to chargeback transaction ([0-9]+) for client ([0-9]+)$")]
fn when_attempt_chargeback(world: &mut DatabaseWorld, txn_id: u32, client_id: u16) {
    let result = world
        .database
        .process_transaction(client_id, txn_id, Transaction::chargeback());

    world.last_error = result.err();
}

#[then(regex = r"^the available balance for client ([0-9]+) should be ([-]?[0-9.]+)$")]
fn then_available_balance(world: &mut DatabaseWorld, client_id: u16, expected: String) {
    let expected_f64 = expected.parse::<f64>().expect("Invalid expected balance");
    world.assert_balance(client_id, expected_f64, "available");
}

#[then(regex = r"^the held balance for client ([0-9]+) should be ([-]?[0-9.]+)$")]
fn then_held_balance(world: &mut DatabaseWorld, client_id: u16, expected: String) {
    let expected_f64 = expected.parse::<f64>().expect("Invalid expected balance");
    world.assert_balance(client_id, expected_f64, "held");
}

#[then(regex = r"^the total balance for client ([0-9]+) should be ([-]?[0-9.]+)$")]
fn then_total_balance(world: &mut DatabaseWorld, client_id: u16, expected: String) {
    let expected_f64 = expected.parse::<f64>().expect("Invalid expected balance");
    world.assert_balance(client_id, expected_f64, "total");
}

#[then(regex = r"^the account for client ([0-9]+) should not be locked$")]
fn then_account_not_locked(world: &mut DatabaseWorld, client_id: u16) {
    let account = world
        .database
        .get_account(client_id)
        .expect(&format!("Client {} should have an account", client_id));
    assert!(
        !account.locked,
        "Account for client {} should not be locked",
        client_id
    );
}

#[then(regex = r"^the account for client ([0-9]+) should be locked$")]
fn then_account_locked(world: &mut DatabaseWorld, client_id: u16) {
    let account = world
        .database
        .get_account(client_id)
        .expect(&format!("Client {} should have an account", client_id));
    assert!(
        account.locked,
        "Account for client {} should be locked",
        client_id
    );
}

#[then(regex = r#"^the transaction should fail with "([^"]*)"$"#)]
fn then_transaction_should_fail(world: &mut DatabaseWorld, expected_error: String) {
    let error = world
        .last_error
        .as_ref()
        .expect("Expected an error but transaction succeeded");

    let error_message = error.to_string();
    assert!(
        error_message.contains(&expected_error),
        "Expected error containing '{}' but got '{}'",
        expected_error,
        error_message
    );
}

#[when(
    regex = r#"^I attempt to process a deposit of "([^"]*)" for client ([0-9]+) with transaction id ([0-9]+)$"#
)]
fn when_attempt_deposit_quoted(
    world: &mut DatabaseWorld,
    amount: String,
    client_id: u16,
    txn_id: u32,
) {
    let transaction = match Transaction::deposit(&amount) {
        Ok(txn) => txn,
        Err(e) => {
            world.last_error = Some(e);
            return;
        }
    };

    let result = world
        .database
        .process_transaction(client_id, txn_id, transaction);
    world.last_error = result.err();
}

#[when(
    regex = r#"^I attempt to process a withdrawal of "([^"]*)" for client ([0-9]+) with transaction id ([0-9]+)$"#
)]
fn when_attempt_withdrawal_quoted(
    world: &mut DatabaseWorld,
    amount: String,
    client_id: u16,
    txn_id: u32,
) {
    let transaction = match Transaction::withdrawal(&amount) {
        Ok(txn) => txn,
        Err(e) => {
            world.last_error = Some(e);
            return;
        }
    };

    let result = world
        .database
        .process_transaction(client_id, txn_id, transaction);
    world.last_error = result.err();
}

#[tokio::test]
async fn run_cucumber_tests() {
    DatabaseWorld::run("features").await;
}
