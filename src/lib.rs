pub mod models;
pub mod config;

pub use models::Customer;
pub use config::BenchConfig;

use std::error::Error;
use std::io::Read;

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
