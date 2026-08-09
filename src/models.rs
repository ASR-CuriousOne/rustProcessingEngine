use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Person {
    pub name: String,
    pub age: u8,
    pub city: String,
}

#[derive(Debug, Deserialize)]
pub struct Customer {
    #[serde(rename = "Index")]
    pub index: usize,

    #[serde(rename = "Customer Id")]
    pub customer_id: String,

    #[serde(rename = "First Name")]
    pub first_name: String,

    #[serde(rename = "Last Name")]
    pub last_name: String,

    #[serde(rename = "Company")]
    pub company: String,

    #[serde(rename = "City")]
    pub city: String,

    #[serde(rename = "Country")]
    pub country: String,

    #[serde(rename = "Phone 1")]
    pub phone_1: String,

    #[serde(rename = "Phone 2")]
    pub phone_2: Option<String>,

    #[serde(rename = "Email")]
    pub email: String,

    #[serde(rename = "Subscription Date")]
    pub subscription_date: String,

    #[serde(rename = "Website")]
    pub website: String,
}
