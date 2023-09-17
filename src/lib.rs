//! [currencyapi](https://currencyapi.com/) API library.
//!
//! > **Note:** experimental
//!
//! The starting point of this library is the [`Rates`] type for currency rates,  which provides:
//! - [x] [Latest Exchange Rates](https://currencyapi.com/docs/latest) - [`Rates::fetch_latest`]
//! - [ ] [Historical Exchange Rates](https://currencyapi.com/docs/historical)
//!
//! The [Convert Exchange Rates](https://currencyapi.com/docs/convert) endpoint is not provided but
//! conversion is implemented via [`Rates::convert`].
//!
//! ## Example
//! ```ignore
//! async fn main() {
//!   let mut rates = Rates::<rust_decimal::Decimal>::new(); // requires `rust_decimal` feature and crate
//!   let request = request.base_currency(EUR).currencies([EUR,USD,GBP]).build();
//!   let metadata = rates
//!   	.fetch_latest::<DateTime<Utc>, RateLimitIgnore>(&client, request) // DateTime<Utc> from the `chrono` crate
//!   	.await
//!   	.unwrap();
//!   println!("Fetched {} rates as of {}", rates.len(), metadata.last_updated_at);
//!   for (currency, value) in rates.iter() { println!("{currency} {value}"); }
//! }
//! ```

#![deny(missing_docs)]

mod currency_impl;
pub use currency_impl::{CurrencyCode, list as currency, Error as CurrencyError};
mod url;
pub mod latest;

mod rates;      pub use rates::Rates;
mod scientific; pub use scientific::FromScientific;
mod rate_limit; pub use rate_limit::{RateLimit, RateLimitIgnore};
mod error;      pub use error::Error;


use std::str::FromStr;

use rate_limit::RateLimitData;

impl<const N: usize, RATE> Rates<RATE, N> {
    /// Fetches a [`latest`] [`Request`](latest::Request).
    pub async fn fetch_latest<DateTime: FromStr, RateLimit: for<'x> RateLimitData<'x>>(&mut self, client: &reqwest::Client, request: latest::Request) -> Result<latest::Metadata<DateTime, RateLimit>, Error> where RATE: FromScientific {
        request.send::<N, DateTime, RATE, RateLimit>(self, client).await
    }
}
