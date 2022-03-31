//! [currencyapi](https://currencyapi.com/) API library.

#![deny(missing_docs)]

use std::fmt::{self, Display, Formatter};

use arrayvec::{ArrayString, CapacityError};
use atoi::atoi;
use chrono::{DateTime, Utc};
use currency::CurrencyCode;
use serde_json as json;
use smallstr::SmallString;
use smallvec::SmallVec;

pub mod currency;

#[repr(transparent)]
#[derive(Debug, Hash, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
/// [API key](https://currencyapi.com/docs/#authentication-api-key-information) token.
///
/// Manage yours [here](https://app.currencyapi.com/api-keys).
pub struct Token {
	/// The token string.
	///
	/// The length of the token is 36 but we use 128 capacity for forward-compatibility.
	token: ArrayString<128>,
}

impl<'a> TryFrom<&'a str> for Token {
	type Error = CapacityError<&'a str>;

	fn try_from(value: &'a str) -> Result<Self, Self::Error> {
		ArrayString::try_from(value).map(|token| Self { token })
	}
}

impl AsRef<str> for Token {
	fn as_ref(&self) -> &str {
		self.token.as_ref()
	}
}

impl Display for Token {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		self.token.fmt(f)
	}
}

/// [`latest`](https://currencyapi.com/docs/latest) endpoint.
#[derive(Debug, Hash, Clone)]
pub struct Latest(SmallString<[u8; 256]>);

impl Latest {
	/// Creates a new [`latest`] endpoint request.
	///
	/// Takes the [API key](https://currencyapi.com/docs/#authentication-api-key-information) token,
	/// the [`base_currency`](https://currencyapi.com/docs/latest#:~:text=Your%20API%20Key-,base_currency,-string), and
	/// the [`currencies`](https://currencyapi.com/docs/latest#:~:text=based%20on%20USD-,currencies,-string) parameters.
	pub fn new(
		token: &str,
		base_currency: Option<CurrencyCode>,
		mut currencies: impl Iterator<Item = CurrencyCode>,
	) -> Self {
		let mut url = SmallString::from("https://api.currencyapi.com/v3/latest?apikey=");
		url.push_str(token);
		if let Some(base_currency) = base_currency {
			url.push_str("&base_currency=");
			url.push_str(base_currency.as_ref());
		}

		if let Some(currencies_head) = currencies.next() {
			url.push_str("&currencies=");
			url.push_str(currencies_head.as_ref());
			for currency in currencies {
				url.push_str(",");
				url.push_str(currency.as_ref());
			}
		}

		Self(url)
	}

	/// Sends the request.
	pub async fn send<const N: usize>(
		&self,
		client: &reqwest::Client,
	) -> reqwest::Result<LatestResponse<N>> {
		let response = client
			.get(self.0.as_str())
			.send()
			.await?
			.error_for_status()?;
		let rate_limit = (&response).try_into().unwrap();
		let payload = response.json::<json::Value>().await?;
		let last_updated_at = payload
			.get("meta")
			.unwrap()
			.get("last_updated_at")
			.unwrap()
			.as_str()
			.unwrap()
			.parse()
			.unwrap();
		let mut currencies = SmallVec::new();
		let mut values = SmallVec::new();

		for (currency, value_object) in payload.get("data").unwrap().as_object().unwrap() {
			currencies.push(CurrencyCode::try_from(currency.as_str()).unwrap());
			values.push(value_object.get("value").unwrap().as_f64().unwrap());
		}

		Ok(LatestResponse {
			last_updated_at,
			currencies,
			values,
			rate_limit,
		})
	}
}

/// [Rate-limit data](https://currencyapi.com/docs/#rate-limit-and-quotas) from response headers.
#[derive(Debug, Hash, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct RateLimit {
	limit_minute: usize,
	limit_month: usize,
	remainig_minute: usize,
	remaining_month: usize,
}

impl TryFrom<&reqwest::Response> for RateLimit {
	type Error = ();

	fn try_from(value: &reqwest::Response) -> Result<Self, Self::Error> {
		let headers = value.headers();
		let h = |name| {
			headers
				.get(name)
				.ok_or(())
				.and_then(|value| atoi(value.as_bytes()).ok_or(()))
		};
		Ok(Self {
			limit_minute: h("X-RateLimit-Limit-Quota-Minute")?,
			limit_month: h("X-RateLimit-Limit-Quota-Month")?,
			remainig_minute: h("X-RateLimit-Remaining-Quota-Minute")?,
			remaining_month: h("X-RateLimit-Remaining-Quota-Month")?,
		})
	}
}

/// [`latest` endpoint](Latest) response data.
#[derive(Debug, Clone)]
pub struct LatestResponse<const N: usize> {
	/// Datetime to let you know then this dataset was last updated. ― [Latest endpoint docs](https://currencyapi.com/docs/latest#:~:text=datetime%20to%20let%20you%20know%20then%20this%20dataset%20was%20last%20updated).
	pub last_updated_at: DateTime<Utc>,
	/// The currencies column.
	pub currencies: SmallVec<[CurrencyCode; N]>,
	/// The values column.
	pub values: SmallVec<[f64; N]>,
	/// Rate-limit data.
	pub rate_limit: RateLimit,
}

impl<const N: usize> LatestResponse<N> {
	/// Iterates over the currencies and their values.
	pub fn iter(&self) -> impl Iterator<Item = (CurrencyCode, f64)> + '_ {
		std::iter::zip(self.currencies.iter().copied(), self.values.iter().copied())
	}
}