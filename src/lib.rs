pub mod config;
pub mod models;
pub mod monitor;

pub use config::BenchConfig;
#[cfg(not(feature = "fast-csv"))]
pub use models::{Customer, CustomerParser};
pub use models::{OhlcvData, OhlcvParser};
pub use monitor::Monitor;

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
