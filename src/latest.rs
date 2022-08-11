//! API for the [`latest`](https://currencyapi.com/docs/latest) endpoint.

use std::ops::{Div, Mul};

use atoi::atoi;
use chrono::{DateTime, Utc};
use serde_json as json;
use smallstr::SmallString;
use smallvec::SmallVec;

use crate::currency::{self, CurrencyCode};

/// A [`Builder`] buffer for all currencies.
pub type AllCurrencies = std::iter::Empty<CurrencyCode>;

/// [`Request`] builder.
///
/// # Examples
/// ```
/// # use currencyapi::latest::Builder;
/// Builder::from("…").build();
/// ```
#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Builder<'a, const N: usize, T = AllCurrencies> {
	/// The [API token](https://currencyapi.com/docs/#authentication-api-key-information).
	pub token: &'a str,
	/// The [`base_currency`](https://currencyapi.com/docs/latest#:~:text=Your%20API%20Key-,base_currency,-string).
	pub base_currency: Option<CurrencyCode>,
	/// The [`currencies`](https://currencyapi.com/docs/latest#:~:text=based%20on%20USD-,currencies,-string).
	pub currencies: T,
}

impl<'a, const N: usize, T> Builder<'a, N, T> {
	/// Sets the [`currencies`](Builder::currencies).
	pub fn currencies<const N2: usize, T2>(self, currencies: T2) -> Builder<'a, N2, T2> {
		Builder {
			token: self.token,
			base_currency: self.base_currency,
			currencies,
		}
	}

	/// Sets the [`base_currency`](Builder::base_currency).
	pub fn base_currency(&mut self, base_currency: Option<CurrencyCode>) -> &mut Self {
		self.base_currency = base_currency;
		self
	}
}

impl<'a> Builder<'a, { currency::list::ARRAY.len() }, AllCurrencies> {
	/// Creates a new [`Builder`] with the given [API token](Builder::token).
	pub const fn new(token: &'a str) -> Self {
		Builder {
			token,
			base_currency: None,
			currencies: std::iter::empty(),
		}
	}
}

impl<'a> From<&'a str> for Builder<'a, { currency::list::ARRAY.len() }, AllCurrencies> {
	fn from(token: &'a str) -> Self {
		Self::new(token)
	}
}

impl<'a, const N: usize, T: IntoIterator<Item = CurrencyCode>> Builder<'a, N, T> {
	/// Builds the [`Request`].
	pub fn build(self) -> Request<N> {
		self.into()
	}
}

/// Calculates the [`Builder`] buffer size.
///
/// # Examples
/// ```
/// # use currencyapi::latest::{Builder, buffer_size};
/// # use currencyapi::currency;
/// Builder::from("…").currencies::<{ buffer_size(2) },_>([currency::list::USD, currency::list::EUR]);
/// ```
pub const fn buffer_size(currencies_len: usize) -> usize {
	"https://api.currencyapi.com/v3/latest?apikey=".len()
		+ /* API key length */ 36 + "&base_currency=XXX&currencies=".len()
		+ currencies_len * 3
		+ /* Comma-separators */ currencies_len.saturating_sub(1)
}

/// The [`latest`](https://currencyapi.com/docs/latest) endpoint.
#[derive(Debug, Hash, Clone)]
pub struct Request<const N: usize>(SmallString<[u8; N]>);

impl<const N: usize> Request<N> {
	/// Sends the request.
	pub async fn send<const M: usize, T: TryFrom<f64>>(
		&self,
		client: &reqwest::Client,
	) -> Result<Response<M, T>, Error> {
		let response = client.get(self.0.as_str()).send().await?;

		if response.status() == 429 {
			return Err(Error::RateLimitError);
		}

		let response = response.error_for_status()?;
		let rate_limit = (&response)
			.try_into()
			.map_err(|_| Error::RateLimitParseError)?;
		let payload = response.json::<json::Value>().await?;
		let last_updated_at = payload
			.get("meta")
			.and_then(|meta| meta.get("last_updated_at"))
			.and_then(|last_updated_at| last_updated_at.as_str())
			.ok_or(Error::ResponseParseError)
			.and_then(|last_updated_at| {
				last_updated_at
					.parse()
					.map_err(|_| Error::ResponseParseError)
			})?;
		let mut currencies = SmallVec::new();
		let mut values = SmallVec::new();

		let data = payload
			.get("data")
			.and_then(|data| data.as_object())
			.ok_or(Error::ResponseParseError)?;
		for (currency, value_object) in data {
			currencies.push(
				CurrencyCode::try_from(currency.as_str()).map_err(|_| Error::ResponseParseError)?,
			);
			values.push(
				value_object
					.get("value")
					.and_then(|value| value.as_f64())
					.and_then(|value| T::try_from(value).ok())
					.ok_or(Error::ResponseParseError)?,
			);
		}

		Ok(Response {
			last_updated_at,
			currencies,
			values,
			rate_limit,
		})
	}
}

impl<'a, const N: usize, T: IntoIterator<Item = CurrencyCode>> From<Builder<'a, N, T>>
	for Request<N>
{
	fn from(builder: Builder<'a, N, T>) -> Self {
		let mut url = SmallString::with_capacity(N);
		url.push_str("https://api.currencyapi.com/v3/latest?apikey=");
		url.push_str(builder.token);
		if let Some(base_currency) = builder.base_currency {
			url.push_str("&base_currency=");
			url.push_str(base_currency.as_ref());
		}

		let mut currencies = builder.currencies.into_iter();
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
}

/// [`latest` endpoint](Request) response data.
#[derive(Debug, Clone)]
pub struct Response<const N: usize, T> {
	/// Datetime to let you know then this dataset was last updated. ― [Latest endpoint docs](https://currencyapi.com/docs/latest#:~:text=datetime%20to%20let%20you%20know%20then%20this%20dataset%20was%20last%20updated).
	pub last_updated_at: DateTime<Utc>,
	/// The currencies column.
	pub currencies: SmallVec<[CurrencyCode; N]>,
	/// The values column.
	pub values: SmallVec<[T; N]>,
	/// Rate-limit data.
	pub rate_limit: RateLimit,
}

impl<const N: usize, T> Response<N, T> {
	/// Iterates over the currencies and their values.
	pub fn iter(&self) -> impl Iterator<Item = (CurrencyCode, &T)> + '_ {
		std::iter::zip(self.currencies.iter().copied(), self.values.iter())
	}

	/// Gets the value for the given currency.
	pub fn get(&self, currency: CurrencyCode) -> Option<&T> {
		self.currencies
			.iter()
			.copied()
			.position(|c| c == currency)
			.map(|i| &self.values[i])
	}

	/// Currency conversion.
	///
	/// Returns [`None`] if either currencies are missing.
	pub fn convert(&self, from: CurrencyCode, to: CurrencyCode, amount: &T) -> Option<T>
	where
		for<'x> &'x T: Div<&'x T, Output = T>,
		for<'x> &'x T: Mul<T, Output = T>,
	{
		let from_value = self.get(from)?;
		let to_value = self.get(to)?;
		Some(amount * (to_value / from_value))
	}
}

/// [Rate-limit data](https://currencyapi.com/docs/#rate-limit-and-quotas) from response headers.
#[derive(Debug, Hash, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct RateLimit {
	/// How many requests can be made in a minute.
	pub limit_minute: usize,
	/// How many requests can be made in a month.
	pub limit_month: usize,
	/// How many remaining requests can be made in the minute of request.
	pub remainig_minute: usize,
	/// How many remaining requests can be made in the month of request.
	pub remaining_month: usize,
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

/// An error from the API or from the HTTP client.
#[derive(Debug, thiserror::Error)]
pub enum Error {
	/// The rate-limit was hit.
	#[error("you have hit your rate limit or your monthly limit")]
	RateLimitError,
	/// HTTP error.
	#[error("http error: {0}")]
	HttpError(#[from] reqwest::Error),
	/// Failed to parse the response.
	#[error("failed to parse the response")]
	ResponseParseError,
	/// Failed to parse the rate-limit headers.
	#[error("failed to parse the rate-limits headers from the response")]
	RateLimitParseError,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_convert() {
		let usd = (*b"USD").try_into().unwrap();
		let eur = (*b"EUR").try_into().unwrap();
		let ils = (*b"ILS").try_into().unwrap();
		let response = Response {
			last_updated_at: Utc::now(),
			currencies: SmallVec::from([usd, eur, ils]),
			values: SmallVec::from([1.0, 0.9, 3.1]),
			rate_limit: Default::default(),
		};
		assert_eq!(response.convert(usd, usd, &1234.0), Some(1234.));
		assert_eq!(response.convert(eur, eur, &1234.0), Some(1234.));
		assert_eq!(response.convert(ils, ils, &1234.0), Some(1234.));
		assert_eq!(response.convert(ils, eur, &1.0), Some(1. / 3.1 * 0.9));
		assert_eq!(response.convert(eur, ils, &1.0), Some(1. / 0.9 * 3.1));
	}
}
