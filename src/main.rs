use rust_processing_engine::{Monitor, OhlcvData, OhlcvParser, process_csv};
use std::env;
use std::fs;
use std::process;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --release -- <file_path>");
        process::exit(1);
    }
    let file_path = &args[1];

    let file_size_bytes = fs::metadata(&file_path)
        .unwrap_or_else(|e| {
            eprintln!("Failed to get file metadata: {}", e);
            process::exit(1);
        })
        .len();

    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    println!("Starting CSV parser...");
    println!("File: {}", file_path);
    println!("Threads: {}", num_threads);
    println!("---------------------------------------------------------");

    let monitor = Monitor::new(file_size_bytes);
    monitor.start_ui_thread();

    let parser_total = Arc::clone(&monitor.total_rows);
    let parser_matching = Arc::clone(&monitor.matching_rows);

    let result = process_csv::<OhlcvParser, _>(file_path, num_threads, |candle: OhlcvData| {
        parser_total.fetch_add(1, Ordering::Relaxed);
        if candle.volume > 1903851.0 {
            parser_matching.fetch_add(1, Ordering::Relaxed);
        }
    });

    if let Err(e) = result {
        eprintln!("Failed to parse CSV: {}", e);
        process::exit(1);
    }

    monitor.finish_and_report();
}
