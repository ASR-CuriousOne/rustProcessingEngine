pub mod config;
pub mod data;
pub mod engine;
pub mod models;
pub mod monitor;
pub mod strategies;

pub use config::BenchConfig;
pub use data::StreamingLoader;
pub use engine::{BacktestEngine, Broker, Strategy};
#[cfg(not(feature = "fast-csv"))]
pub use models::{Customer, CustomerParser};
pub use models::{OhlcvData, OhlcvParser};
pub use monitor::Monitor;
pub use strategies::{BuyOnGreenStrategy, EMAStrategy};

use memchr::memchr;
use memmap2::MmapOptions;
use mimalloc::MiMalloc;
use std::error::Error;
use std::fs::File;
use std::thread;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub trait ZeroCopyParse {
    type Output<'a>;
    unsafe fn parse<'a>(fields: &[&'a [u8]]) -> Self::Output<'a>;
}

fn find_next_newline(data: &[u8], start: usize) -> usize {
    memchr(b'\n', &data[start..])
        .map(|p| start + p + 1)
        .unwrap_or(data.len())
}

pub fn process_csv<P, F>(
    file_path: &str,
    num_threads: usize,
    on_record: F,
) -> Result<(), Box<dyn Error>>
where
    P: ZeroCopyParse,
    F: for<'a> Fn(P::Output<'a>) + Sync,
{
    let file = File::open(file_path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let data: &[u8] = &mmap;

    let data_start = find_next_newline(data, 0);

    let mut boundaries = vec![data_start];
    let file_size = data.len();
    let chunk_size = (file_size - data_start) / num_threads;

    for i in 1..num_threads {
        let guess = data_start + (i * chunk_size);
        let aligned = find_next_newline(data, guess);
        boundaries.push(aligned);
    }

    boundaries.push(file_size);

    thread::scope(|scope| {
        for i in 0..num_threads {
            let start = boundaries[i];
            let end = boundaries[i + 1];
            let chunk = &data[start..end];
            let callback = &on_record;

            scope.spawn(move || {
                #[cfg(feature = "fast-csv")]
                {
                    let mut line_start = 0;
                    while line_start < chunk.len() {
                        let line_end = memchr::memchr(b'\n', &chunk[line_start..])
                            .map(|p| line_start + p)
                            .unwrap_or(chunk.len());

                        let mut line = &chunk[line_start..line_end];
                        line_start = line_end + 1;

                        if line.last() == Some(&b'\r') {
                            line = &line[..line.len() - 1];
                        }
                        if line.is_empty() {
                            continue;
                        }

                        let mut fields_arr = [&b""[..]; 16];
                        let mut field_start = 0;
                        let mut count = 0;

                        for comma_pos in memchr::memchr_iter(b',', line) {
                            if count < 16 {
                                fields_arr[count] = &line[field_start..comma_pos];
                                count += 1;
                            }
                            field_start = comma_pos + 1;
                        }

                        if count < 16 {
                            fields_arr[count] = &line[field_start..];
                            count += 1;
                        }

                        unsafe {
                            let parsed = P::parse(&fields_arr[..count]);
                            callback(parsed);
                        }
                    }
                }

                #[cfg(not(feature = "fast-csv"))]
                {
                    let mut chunk_reader = csv::ReaderBuilder::new()
                        .has_headers(false)
                        .from_reader(chunk);

                    let mut record = csv::ByteRecord::new();

                    while chunk_reader.read_byte_record(&mut record).unwrap_or(false) {
                        let mut fields_arr = [&b""[..]; 16];
                        let mut count = 0;

                        for (i, field) in record.iter().enumerate().take(16) {
                            fields_arr[i] = field;
                            count += 1;
                        }

                        unsafe {
                            let parsed = P::parse(&fields_arr[..count]);
                            callback(parsed);
                        }
                    }
                }
            });
        }
    });

    Ok(())
}

pub fn process_csv_in_memory<'a, P>(
    data: &'a [u8],
    num_threads: usize,
) -> Result<Vec<P::Output<'a>>, Box<dyn Error>>
where
    P: ZeroCopyParse,
    P::Output<'a>: Send,
{
    let data_start = find_next_newline(data, 0);
    let mut boundaries = vec![data_start];
    let file_size = data.len();

    if file_size > data_start {
        let chunk_size = (file_size - data_start) / num_threads;
        for i in 1..num_threads {
            let guess = data_start + (i * chunk_size);
            let aligned = find_next_newline(data, guess);
            boundaries.push(aligned);
        }
    }
    boundaries.push(file_size);

    let thread_results = thread::scope(|scope| {
        let mut handles = Vec::with_capacity(num_threads);

        for i in 0..num_threads {
            let start = boundaries[i];
            let end = boundaries[i + 1];
            let chunk = &data[start..end];

            let handle = scope.spawn(move || {
                let mut local_data = Vec::new();
                #[cfg(feature = "fast-csv")]
                {
                    let mut line_start = 0;
                    while line_start < chunk.len() {
                        let line_end = memchr::memchr(b'\n', &chunk[line_start..])
                            .map(|p| line_start + p)
                            .unwrap_or(chunk.len());

                        let mut line = &chunk[line_start..line_end];
                        line_start = line_end + 1;

                        if line.last() == Some(&b'\r') {
                            line = &line[..line.len() - 1];
                        }
                        if line.is_empty() {
                            continue;
                        }

                        let mut fields_arr = [&b""[..]; 16];
                        let mut field_start = 0;
                        let mut count = 0;

                        for comma_pos in memchr::memchr_iter(b',', line) {
                            if count < 16 {
                                fields_arr[count] = &line[field_start..comma_pos];
                                count += 1;
                            }
                            field_start = comma_pos + 1;
                        }

                        if count < 16 {
                            fields_arr[count] = &line[field_start..];
                            count += 1;
                        }

                        unsafe {
                            let parsed = P::parse(&fields_arr[..count]);
                            local_data.push(parsed);
                        }
                    }
                }

                #[cfg(not(feature = "fast-csv"))]
                {
                    let mut chunk_reader = csv::ReaderBuilder::new()
                        .has_headers(false)
                        .from_reader(chunk);

                    let mut record = csv::ByteRecord::new();

                    while chunk_reader.read_byte_record(&mut record).unwrap_or(false) {
                        let mut fields_arr = [&b""[..]; 16];
                        let mut count = 0;

                        for (i, field) in record.iter().enumerate().take(16) {
                            fields_arr[i] = field;
                            count += 1;
                        }

                        unsafe {
                            let parsed = P::parse(&fields_arr[..count]);
                            local_data.push(parsed);
                        }
                    }
                }

                local_data
            });
            handles.push(handle);
        }

        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    });

    let total_capacity: usize = thread_results.iter().map(|v| v.len()).sum();
    let mut ordered_data = Vec::with_capacity(total_capacity);

    for mut batch in thread_results {
        ordered_data.append(&mut batch);
    }

    Ok(ordered_data)
}
