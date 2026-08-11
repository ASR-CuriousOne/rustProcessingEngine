use rust_processing_engine::{Customer, process_csv_file_mmap};
use std::env;
use std::fs;
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

    let metadata = match fs::metadata(&file_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to get file metadata: {}", e);
            process::exit(1);
        }
    };

    let file_size_bytes: u64 = metadata.len();

    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    println!("Starting memory-mapped CSV parser...");
    println!("File: {}", file_path);
    println!("Threads: {}", num_threads);
    println!("---------------------------------------------------------");

    let start_time = Instant::now();
    let total_rows = AtomicUsize::new(0);
    let matching_rows = AtomicUsize::new(0);

    let result = process_csv_file_mmap(file_path, num_threads, |customer: Customer| {
        total_rows.fetch_add(1, Ordering::Relaxed);
        if customer.first_name == "Roy" {
            matching_rows.fetch_add(1, Ordering::Relaxed);
        }
    });

    if let Err(e) = result {
        eprintln!("Failed to parse CSV: {}", e);
        process::exit(1);
    }

    let elapsed = start_time.elapsed();

    let throughput_mb_s = (file_size_bytes as f64 / (1024.0 * 1024.0)) / elapsed.as_secs_f64();

    println!(
        "Total Rows Processed: {}",
        total_rows.load(Ordering::Relaxed)
    );
    println!(
        "Total Matching Rows:  {}",
        matching_rows.load(Ordering::Relaxed)
    );

    println!("Total Time Taken:     {:.2?}", elapsed);
    println!("Throughput:           {:.2} MB/s", throughput_mb_s);
}
