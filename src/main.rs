use rust_processing_engine::{Customer, process_csv_file_mmap};
use std::env;
use std::process;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: cargo run --release -- <file_path>");
        process::exit(1);
    }

    let file_path = &args[1];

    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    println!("Starting memory-mapped CSV parser...");
    println!("File: {}", file_path);
    println!("Threads: {}", num_threads);
    println!("---------------------------------------------------------");

    let start_time = Instant::now();
    let total_rows = AtomicUsize::new(0);

    let result = process_csv_file_mmap(file_path, num_threads, |customer: Customer| {
        total_rows.fetch_add(1, Ordering::Relaxed);
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

    let elapsed = start_time.elapsed();
    println!(
        "Total Rows Processed: {}",
        total_rows.load(Ordering::Relaxed)
    );
    println!("Total Time Taken:     {:.2?}", elapsed);
}
