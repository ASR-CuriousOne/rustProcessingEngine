use crate::ZeroCopyParse;
use crate::models::{OhlcvData, OhlcvParser};
use crate::process_csv_in_memory;
use memmap2::MmapOptions;
use std::error::Error;
use std::fs::File;

pub struct DataLoader;

impl DataLoader {
    pub fn load_ohlcv(
        file_path: &str,
        num_threads: usize,
    ) -> Result<Vec<OhlcvData>, Box<dyn Error>> {
        let file = File::open(file_path)?;

        let mmap = unsafe { MmapOptions::new().map(&file)? };

        let data = process_csv_in_memory::<OhlcvParser>(&mmap, num_threads)?;

        Ok(data)
    }
}

pub struct StreamingLoader;

impl StreamingLoader {
    pub fn stream_ohlcv<F>(file_path: &str, mut on_bar: F) -> Result<(), Box<dyn Error>>
    where
        F: FnMut(&OhlcvData),
    {
        let file = File::open(file_path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        let chunk: &[u8] = &mmap;

        let data_start = crate::find_next_newline(chunk, 0);
        let mut line_start = data_start;

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
                let parsed = OhlcvParser::parse(&fields_arr[..count]);
                on_bar(&parsed);
            }
        }

        Ok(())
    }
}
