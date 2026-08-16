use indicatif::{ProgressBar, ProgressStyle};
use rust_processing_engine::{OhlcvData, OhlcvParser, process_csv};
use std::env;
use std::fs;
use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

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

    println!("Starting CSV parser...");
    println!("File: {}", file_path);
    println!("Threads: {}", num_threads);
    println!("---------------------------------------------------------");

    let start_time = Instant::now();

    let total_rows = Arc::new(AtomicUsize::new(0));
    let matching_rows = Arc::new(AtomicUsize::new(0));

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .unwrap(),
    );
    pb.set_message("Parsing CSV...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let monitor_total = Arc::clone(&total_rows);
    let monitor_pb = pb.clone();

    thread::spawn(move || {
        let monitor_start = Instant::now();
        while !monitor_pb.is_finished() {
            let current_rows = monitor_total.load(Ordering::Relaxed);
            let elapsed = monitor_start.elapsed().as_secs_f64();
            let rps = if elapsed > 0.0 {
                (current_rows as f64 / elapsed) as u64
            } else {
                0
            };
            monitor_pb.set_message(format!("Parsed {} rows ({} rows/sec)", current_rows, rps));
            thread::sleep(Duration::from_millis(100));
        }
    });

    let parser_total = Arc::clone(&total_rows);
    let parser_matching = Arc::clone(&matching_rows);

    let result = process_csv::<OhlcvParser, _>(file_path, num_threads, |candle: OhlcvData| {
        parser_total.fetch_add(1, Ordering::Relaxed);
        if candle.volume > 1903851.0 {
            parser_matching.fetch_add(1, Ordering::Relaxed);
        }
    });

    pb.finish_with_message("Parsing complete.");

    if let Err(e) = result {
        eprintln!("Failed to parse CSV: {}", e);
        process::exit(1);
    }

    let elapsed = start_time.elapsed();
    let throughput_mb_s = (file_size_bytes as f64 / (1024.0 * 1024.0)) / elapsed.as_secs_f64();

    println!(
        "Total Rows Processed: {}",
        parser_total.load(Ordering::Relaxed)
    );
    println!(
        "Total Matching Rows:  {}",
        parser_matching.load(Ordering::Relaxed)
    );
    println!("Total Time Taken:     {:.2?}", elapsed);
    println!("Throughput:           {:.2} MB/s", throughput_mb_s);
}
