pub mod config;
pub mod models;

pub use config::BenchConfig;
pub use models::Customer;

use csv::ByteRecord;
use memmap2::MmapOptions;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

pub fn process_csv<R, F>(reader: R, mut on_record: F) -> Result<(), Box<dyn Error>>
where
    R: Read,
    F: FnMut(Customer),
{
    let mut rdr = csv::Reader::from_reader(reader);

    for result in rdr.deserialize() {
        let record: Customer = result?;

        on_record(record);
    }

    Ok(())
}

pub fn process_csv_parallel_callback<R, F>(
    reader: R,
    batch_size: usize,
    num_threads: usize,
    on_record: F,
) -> Result<(), Box<dyn Error>>
where
    R: Read,
    F: Fn(Customer) + Sync,
{
    let (sender, receiver) = mpsc::sync_channel::<Vec<Customer>>(num_threads * 2);

    let shared_receiver = Arc::new(Mutex::new(receiver));

    let mut parse_error = None;

    thread::scope(|scope| {
        for _ in 0..num_threads {
            let rx = Arc::clone(&shared_receiver);
            let callback = &on_record;

            scope.spawn(move || {
                loop {
                    let batch_opt = {
                        let lock = rx.lock().unwrap();
                        lock.recv().ok()
                    };

                    if let Some(batch) = batch_opt {
                        for record in batch {
                            callback(record);
                        }
                    } else {
                        break;
                    }
                }
            });
        }

        let mut rdr = csv::Reader::from_reader(reader);
        let mut iter = rdr.deserialize::<Customer>();

        let mut should_stop = false;

        loop {
            let mut batch = Vec::with_capacity(batch_size);

            for _ in 0..batch_size {
                match iter.next() {
                    Some(Ok(record)) => batch.push(record),
                    Some(Err(error)) => {
                        parse_error = Some(error);
                        should_stop = true;
                        break;
                    }
                    None => {
                        should_stop = true;
                        break;
                    }
                }
            }

            if !batch.is_empty() {
                if sender.send(batch).is_err() {
                    break;
                }
            }

            if should_stop {
                break;
            }
        }

        drop(sender);
    });

    if let Some(e) = parse_error {
        return Err(Box::new(e));
    }

    Ok(())
}

pub fn process_csv_parallel_parsing<R, F>(
    reader: R,
    num_threads: usize,
    batch_size: usize,
    on_record: F,
) -> Result<(), Box<dyn Error>>
where
    R: Read,
    F: Fn(Customer) + Sync,
{
    let mut rdr = csv::Reader::from_reader(reader);
    let headers = rdr.byte_headers()?.clone();

    let (sender, receiver) = mpsc::sync_channel::<Vec<ByteRecord>>(num_threads * 2);
    let shared_receiver = Arc::new(Mutex::new(receiver));
    let mut io_error = None;

    thread::scope(|scope| {
        for _ in 0..num_threads {
            let rx = Arc::clone(&shared_receiver);
            let callback = &on_record;
            let worker_headers = headers.clone();

            scope.spawn(move || {
                loop {
                    let batch_opt = {
                        let lock = rx.lock().unwrap();
                        lock.recv().ok()
                    };

                    if let Some(batch) = batch_opt {
                        for record in batch {
                            match record.deserialize::<Customer>(Some(&worker_headers)) {
                                Ok(customer) => callback(customer),
                                Err(_) => {
                                    continue;
                                }
                            }
                        }
                    } else {
                        break;
                    }
                }
            });
        }

        let mut should_stop = false;

        loop {
            let mut batch = Vec::with_capacity(batch_size);
            for _ in 0..batch_size {
                let mut record = ByteRecord::new();

                match rdr.read_byte_record(&mut record) {
                    Ok(true) => batch.push(record),
                    Ok(false) => {
                        should_stop = true;
                        break;
                    }
                    Err(e) => {
                        io_error = Some(e);
                        should_stop = true;
                        break;
                    }
                }
            }

            if !batch.is_empty() {
                if sender.send(batch).is_err() {
                    break;
                }
            }

            if should_stop {
                break;
            }
        }

        drop(sender);
    });

    if let Some(e) = io_error {
        return Err(Box::new(e));
    }

    Ok(())
}

fn find_next_newline(data: &[u8], start: usize) -> usize {
    let mut pos = start;
    while pos < data.len() && data[pos] != b'\n' {
        pos += 1;
    }
    if pos < data.len() {
        pos + 1
    } else {
        data.len()
    }
}

pub fn process_csv_file_mmap<F>(
    file_path: &str,
    num_threads: usize,
    on_record: F,
) -> Result<(), Box<dyn Error>>
where
    F: Fn(Customer) + Sync,
{
    let file = File::open(file_path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let data: &[u8] = &mmap;

    let mut main_reader = csv::ReaderBuilder::new().from_reader(data);
    let headers = main_reader.byte_headers()?.clone();

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
            let thread_headers = headers.clone();
            let callback = &on_record;

            scope.spawn(move || {
                let mut chunk_reader = csv::ReaderBuilder::new()
                    .has_headers(false)
                    .from_reader(chunk);

                let mut record = csv::ByteRecord::new();

                while chunk_reader.read_byte_record(&mut record).unwrap_or(false) {
                    if let Ok(customer) = record.deserialize::<Customer>(Some(&thread_headers)) {
                        callback(customer);
                    }
                }
            });
        }
    });

    Ok(())
}
