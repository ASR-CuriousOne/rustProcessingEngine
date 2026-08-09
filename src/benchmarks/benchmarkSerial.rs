use rust_processing_engine::{BenchConfig, Customer, process_csv};
use std::env;
use std::fs;
use std::process;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!(
            "Usage: cargo run --release --bin {} -- <directory_path>",
            args[0]
        );
        process::exit(1);
    }

    let config = match BenchConfig::new(&args[1]) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Configuration Error: {}", e);
            process::exit(1);
        }
    };

    println!("Starting benchmark on directory: {:?}", config.dir_path);

    let mut total_files = 0usize;
    let mut total_bytes = 0u64;
    let mut total_rows = 0u64;

    let global_start_time = Instant::now();

    let entries = match fs::read_dir(config.dir_path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Failed to read directory: {}", e);
            process::exit(1);
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("csv") {
            let metadata = match fs::metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let file_size = metadata.len();
            let file = match fs::File::open(&path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Failed to open {:?}: {}", path, e);
                    continue;
                }
            };

            let mut file_rows = 0u64;
            let file_start = Instant::now();

            let result = process_csv(file, |_customer: Customer| {
                file_rows += 1;
            });

            let file_elapsed = file_start.elapsed();

            match result {
                Ok(_) => {
                    total_files += 1;
                    total_bytes += file_size;
                    total_rows += file_rows;

                    let file_mb = file_size as f64 / (1024.0 * 1024.0);
                    println!(
                        "  [OK] {:<25} | {:>8} rows | {:>6.2} MB | {:>8.2?} ",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        file_rows,
                        file_mb,
                        file_elapsed
                    );
                }
                Err(e) => {
                    eprintln!(
                        "  [ERR] Failed parsing {:?}: {}",
                        path.file_name().unwrap(),
                        e
                    );
                }
            }
        }
    }

    let global_elapsed = global_start_time.elapsed();
    let duration_secs = global_elapsed.as_secs_f64();
    let total_mb = total_bytes as f64 / (1024.0 * 1024.0);

    let throughput_mb_s = if duration_secs > 0.0 {
        total_mb / duration_secs
    } else {
        0.0
    };
    let rows_per_sec = if duration_secs > 0.0 {
        total_rows as f64 / duration_secs
    } else {
        0.0
    };
    let avg_time_per_file = if total_files > 0 {
        (duration_secs * 1000.0) / total_files as f64
    } else {
        0.0
    };

    println!("\n=================== BENCHMARK RESULTS ===================");
    println!("Files Processed:   {}", total_files);
    println!(
        "Total Data Read:   {:.2} MB ({} bytes)",
        total_mb, total_bytes
    );
    println!("Total Rows Read:   {}", total_rows);
    println!("Total Time Taken:  {:.4} seconds", duration_secs);
    println!("---------------------------------------------------------");
    println!("Throughput:        {:.2} MB/s", throughput_mb_s);
    println!("Processing Speed:  {:.0} rows/sec", rows_per_sec);
    println!("Avg Time per File: {:.2} ms", avg_time_per_file);
    println!("=========================================================");
}
