use crate::ZeroCopyParse;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Customer<'a> {
    #[serde(rename = "Index")]
    pub index: usize,
    #[serde(borrow, rename = "Customer Id")]
    pub customer_id: &'a str,
    #[serde(borrow, rename = "First Name")]
    pub first_name: &'a str,
    #[serde(borrow, rename = "Last Name")]
    pub last_name: &'a str,
    #[serde(borrow, rename = "Company")]
    pub company: &'a str,
    #[serde(borrow, rename = "City")]
    pub city: &'a str,
    #[serde(borrow, rename = "Country")]
    pub country: &'a str,
    #[serde(borrow, rename = "Phone 1")]
    pub phone_1: &'a str,
    #[serde(borrow, rename = "Phone 2")]
    pub phone_2: Option<&'a str>,
    #[serde(borrow, rename = "Email")]
    pub email: &'a str,
    #[serde(borrow, rename = "Subscription Date")]
    pub subscription_date: &'a str,
    #[serde(borrow, rename = "Website")]
    pub website: &'a str,
}

pub struct CustomerParser;

impl ZeroCopyParse for CustomerParser {
    type Output<'a> = Customer<'a>;

    unsafe fn parse<'a>(record: &'a csv::ByteRecord) -> Self::Output<'a> {
        unsafe {
            Customer {
                index: std::str::from_utf8_unchecked(&record[0])
                    .parse()
                    .unwrap_or(0),
                customer_id: std::str::from_utf8_unchecked(&record[1]),
                first_name: std::str::from_utf8_unchecked(&record[2]),
                last_name: std::str::from_utf8_unchecked(&record[3]),
                company: std::str::from_utf8_unchecked(&record[4]),
                city: std::str::from_utf8_unchecked(&record[5]),
                country: std::str::from_utf8_unchecked(&record[6]),
                phone_1: std::str::from_utf8_unchecked(&record[7]),
                phone_2: if record[8].is_empty() {
                    None
                } else {
                    Some(std::str::from_utf8_unchecked(&record[8]))
                },
                email: std::str::from_utf8_unchecked(&record[9]),
                subscription_date: std::str::from_utf8_unchecked(&record[10]),
                website: std::str::from_utf8_unchecked(&record[11]),
            }
        }
    }
}
