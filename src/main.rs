use rust_processing_engine::process_csv_file_mmap;
use std::env;
use std::process;
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

    match process_csv_file_mmap(file_path, num_threads) {
        Ok(total_rows) => {
            let elapsed = start_time.elapsed();

            println!("Total Rows Processed: {}", total_rows);
            println!("Total Time Taken:     {:.2?}", elapsed);
        }
        Err(e) => {
            eprintln!("Error processing CSV: {}", e);
            process::exit(1);
        }
    }
}

