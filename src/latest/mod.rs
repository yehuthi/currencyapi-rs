//! API for the [`latest`](https://currencyapi.com/docs/latest) endpoint.

mod request;
pub use request::*;

mod url;

use chrono::{DateTime, Utc};

use crate::{rates::Rates, rate_limit::RateLimit};

/// [`latest` endpoint](Request) response data.
#[derive(Debug)]
pub struct Response<const N: usize, RATE> {
	/// Datetime to let you know then this dataset was last updated. ― [Latest endpoint docs](https://currencyapi.com/docs/latest#:~:text=datetime%20to%20let%20you%20know%20then%20this%20dataset%20was%20last%20updated).
	pub last_updated_at: DateTime<Utc>,
	/// The currency rates.
	pub rates: Rates<N, RATE>,
	/// Rate-limit data.
	pub rate_limit: RateLimit,
}
