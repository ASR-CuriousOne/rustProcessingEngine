use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::process;

fn main() -> io::Result<()> {
    let input_path = "./testData/ohlcv/DOGEUSDT.csv";
    let output_path = "./testData/ohlcv/test.csv";
    let target_size_mb: u64 = 4096;

    let target_bytes = target_size_mb * 1024 * 1024;

    let input_file = match File::open(input_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open input file: {}", e);
            process::exit(1);
        }
    };
    let mut reader = BufReader::new(input_file);

    let mut header = String::new();
    reader.read_line(&mut header)?;

    let mut data_buffer = Vec::new();
    reader.read_to_end(&mut data_buffer)?;

    if data_buffer.is_empty() {
        eprintln!("Error: Input file contains no data below the header.");
        process::exit(1);
    }

    println!("Generating file up to {} MB...", target_size_mb);

    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);

    writer.write_all(header.as_bytes())?;

    let mut current_bytes = header.len() as u64;
    let mut last_reported_mb = 0;

    while current_bytes < target_bytes {
        writer.write_all(&data_buffer)?;
        current_bytes += data_buffer.len() as u64;

        let current_mb = current_bytes / (1024 * 1024);
        if current_mb > last_reported_mb {
            print!("Current size: {} MB\r", current_mb);
            io::stdout().flush()?;
            last_reported_mb = current_mb;
        }
    }

    writer.flush()?;

    println!("\nDone! Output saved to {}", output_path);

    Ok(())
}
