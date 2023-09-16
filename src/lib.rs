//! [currencyapi](https://currencyapi.com/) API library.

#![deny(missing_docs)]

pub mod currency; pub use currency::CurrencyCode;
mod rates;        pub use rates::Rates;
mod scientific;   pub use scientific::FromScientific;
mod rate_limit;   pub use rate_limit::{RateLimit, RateLimitIgnore};
mod error;        pub use error::Error;

pub mod latest;
