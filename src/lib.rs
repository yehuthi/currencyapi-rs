//! [currencyapi](https://currencyapi.com/) API library.

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
    /// Fetches a [`latest`] request.
    pub async fn fetch_latest<DateTime: FromStr, RateLimit: for<'x> RateLimitData<'x>>(&mut self, client: &reqwest::Client, request: latest::Request) -> Result<latest::Metadata<DateTime, RateLimit>, Error> where RATE: FromScientific {
        request.send::<N, DateTime, RATE, RateLimit>(self, client).await
    }
}
