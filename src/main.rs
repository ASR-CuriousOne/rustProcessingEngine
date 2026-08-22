use rust_processing_engine::BacktestEngine;
use rust_processing_engine::{EMAStrategy};
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --release -- <file_path>");
        process::exit(1);
    }
    let file_path = &args[1];

    let mut strategy = EMAStrategy::new(50);

    let mut engine = BacktestEngine::new(10000.0, &mut strategy);

    if let Err(e) = engine.run(file_path) {
        eprintln!("Backtest failed: {}", e);
        process::exit(1);
    }
}
