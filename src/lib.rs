//! [currencyapi](https://currencyapi.com/) API library.

#![deny(missing_docs)]

pub mod currency;
mod url;

pub use currency::CurrencyCode;
mod rates;        pub use rates::Rates;
mod scientific;   pub use scientific::FromScientific;
mod rate_limit;   pub use rate_limit::{RateLimit, RateLimitIgnore};
mod error;        pub use error::Error;

pub mod latest;

use std::str::FromStr;

use rate_limit::RateLimitData;

impl<const N: usize, RATE> Rates<RATE, N> {
    /// Fetches a [`latest`] request.
    pub async fn fetch_latest<DateTime: FromStr, RateLimit: for<'x> RateLimitData<'x>>(&mut self, client: &reqwest::Client, request: latest::Request) -> Result<latest::Metadata<DateTime, RateLimit>, Error> where RATE: FromScientific {
        request.send::<N, DateTime, RATE, RateLimit>(self, client).await
    }
}
