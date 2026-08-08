use rust_processing_engine::{Customer, process_csv};
use std::env;
use std::fs::File;
use std::process;

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

    println!("Processing csv file {}", file_path);

    let result = process_csv(file, |customer: Customer| {
        if customer.first_name == "Roy" {
            println!(
                "A {} lives in {}, {} and works at {}",
                customer.first_name, customer.city, customer.country, customer.company
            );
        }
    });

    if let Err(e) = result {
        eprintln!("Failed to parse CSV: {}", e);
        process::exit(1);
    }
}
