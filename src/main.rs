use clap::Parser;
use std::error::Error;
use std::process;
use transaction_processor::{Database, process_csv_file};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(
    about = "A transaction processing engine that processes CSV files containing financial transactions"
)]
struct Args {
    /// Input CSV file containing transactions
    csv_file: String,

    /// Print detailed error messages to stderr
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let (database, errors) = process_csv_file(&args.csv_file)?;

    if args.verbose {
        for error in errors {
            eprintln!("{}", error);
        }
    }

    print_account_summaries(&database);

    Ok(())
}

fn print_account_summaries(database: &Database) {
    println!("client,available,held,total,locked");

    let mut client_ids = database.get_all_client_ids();
    client_ids.sort(); // Sort for consistent output

    for client_id in client_ids {
        if let Some(account) = database.get_account(client_id) {
            println!(
                "{},{},{},{},{}",
                client_id,
                account.available,
                account.held,
                account.total(),
                account.locked
            );
        }
    }
}
