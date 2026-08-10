use rust_processing_engine::{Customer, process_csv_parallel};
use std::env;
use std::fs::File;
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
    }

    let file_path = &args[1];

    let file = match File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open file '{}': {}", file_path, e);
            process::exit(1);
        }
    };

    let active_customers = AtomicUsize::new(0);

    println!("Processing csv file {}", file_path);

    let result = process_csv_parallel(file, 10_000, 4, |customer: Customer| {
        if customer.country == "United States" {
            active_customers.fetch_add(1, Ordering::Relaxed);
        }
    });

    if let Err(e) = result {
        eprintln!("Failed to parse CSV: {}", e);
        process::exit(1);
    }
}
