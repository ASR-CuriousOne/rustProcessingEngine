use indicatif::{ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub struct Monitor {
    pub total_rows: Arc<AtomicUsize>,
    pub matching_rows: Arc<AtomicUsize>,
    start_time: Instant,
    file_size_bytes: u64,
    pb: ProgressBar,
}

impl Monitor {
    pub fn new(file_size_bytes: u64) -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                .template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap(),
        );
        pb.set_message("Parsing CSV...");
        pb.enable_steady_tick(Duration::from_millis(100));

        Self {
            total_rows: Arc::new(AtomicUsize::new(0)),
            matching_rows: Arc::new(AtomicUsize::new(0)),
            start_time: Instant::now(),
            file_size_bytes,
            pb,
        }
    }

    pub fn start_ui_thread(&self) {
        let monitor_total = Arc::clone(&self.total_rows);
        let monitor_pb = self.pb.clone();
        let monitor_start = self.start_time;

        thread::spawn(move || {
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
    }

    pub fn finish_and_report(self) {
        self.pb.finish_with_message("Parsing complete!");

        let elapsed = self.start_time.elapsed();
        let throughput_mb_s =
            (self.file_size_bytes as f64 / (1024.0 * 1024.0)) / elapsed.as_secs_f64();

        println!(
            "Total Rows Processed: {}",
            self.total_rows.load(Ordering::Relaxed)
        );
        println!(
            "Total Matching Rows:  {}",
            self.matching_rows.load(Ordering::Relaxed)
        );
        println!("Total Time Taken:     {:.2?}", elapsed);
        println!("Throughput:           {:.2} MB/s", throughput_mb_s);
    }
}
