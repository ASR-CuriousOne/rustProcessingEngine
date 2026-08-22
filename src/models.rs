use crate::ZeroCopyParse;

#[cfg(not(feature = "fast-csv"))]
use serde::Deserialize;

#[cfg(not(feature = "fast-csv"))]
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

#[cfg(not(feature = "fast-csv"))]
pub struct CustomerParser;

#[cfg(not(feature = "fast-csv"))]
impl ZeroCopyParse for CustomerParser {
    type Output<'a> = Customer<'a>;

    unsafe fn parse<'a>(fields: &[&'a [u8]]) -> Self::Output<'a> {
        unsafe {
            Customer {
                index: std::str::from_utf8_unchecked(fields.get(0).copied().unwrap_or(b"0"))
                    .parse()
                    .unwrap_or(0),
                customer_id: std::str::from_utf8_unchecked(fields.get(1).copied().unwrap_or(b"")),
                first_name: std::str::from_utf8_unchecked(fields.get(2).copied().unwrap_or(b"")),
                last_name: std::str::from_utf8_unchecked(fields.get(3).copied().unwrap_or(b"")),
                company: std::str::from_utf8_unchecked(fields.get(4).copied().unwrap_or(b"")),
                city: std::str::from_utf8_unchecked(fields.get(5).copied().unwrap_or(b"")),
                country: std::str::from_utf8_unchecked(fields.get(6).copied().unwrap_or(b"")),
                phone_1: std::str::from_utf8_unchecked(fields.get(7).copied().unwrap_or(b"")),
                phone_2: {
                    let f = fields.get(8).copied().unwrap_or(b"");
                    if f.is_empty() {
                        None
                    } else {
                        Some(std::str::from_utf8_unchecked(f))
                    }
                },
                email: std::str::from_utf8_unchecked(fields.get(9).copied().unwrap_or(b"")),
                subscription_date: std::str::from_utf8_unchecked(
                    fields.get(10).copied().unwrap_or(b""),
                ),
                website: std::str::from_utf8_unchecked(fields.get(11).copied().unwrap_or(b"")),
            }
        }
    }
}

#[derive(Debug)]
pub struct OhlcvData {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl OhlcvData {
    pub fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }

    pub fn spread(&self) -> f64 {
        self.high - self.low
    }

    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }
}

pub struct OhlcvParser;

impl ZeroCopyParse for OhlcvParser {
    type Output<'a> = OhlcvData;

    unsafe fn parse<'a>(fields: &[&'a [u8]]) -> Self::Output<'a> {
        unsafe {
            OhlcvData {
                timestamp: std::str::from_utf8_unchecked(fields.get(0).copied().unwrap_or(b"0"))
                    .parse()
                    .unwrap_or(0),
                open: fast_float::parse(fields.get(1).copied().unwrap_or(b"0")).unwrap_or(0.0),
                high: fast_float::parse(fields.get(2).copied().unwrap_or(b"0")).unwrap_or(0.0),
                low: fast_float::parse(fields.get(3).copied().unwrap_or(b"0")).unwrap_or(0.0),
                close: fast_float::parse(fields.get(4).copied().unwrap_or(b"0")).unwrap_or(0.0),
                volume: fast_float::parse(fields.get(5).copied().unwrap_or(b"0")).unwrap_or(0.0),
            }
        }
    }
}
