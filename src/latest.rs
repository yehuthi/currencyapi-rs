//! API for the [`latest`](https://currencyapi.com/docs/latest) endpoint.

use std::io;

use atoi::atoi;
use chrono::{DateTime, Utc};
use serde_json as json;

use crate::{currency::CurrencyCode, rates::Rates};

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
pub struct Builder<'a, Currencies = AllCurrencies, BaseCurrency = NoBaseCurrency> {
	/// The [API token](https://currencyapi.com/docs/#authentication-api-key-information).
	pub token: &'a str,
	/// The [`base_currency`](https://currencyapi.com/docs/latest#:~:text=Your%20API%20Key-,base_currency,-string).
	pub base_currency: BaseCurrency,
	/// The [`currencies`](https://currencyapi.com/docs/latest#:~:text=based%20on%20USD-,currencies,-string).
	pub currencies: Currencies,
}

/// A base currency parameter for [`Builder`].
pub struct BaseCurrency<T>(pub T);

/// A type for [`Builder`] indicating the request does not specify a base currency parameter.
pub struct NoBaseCurrency;

mod private {
	use super::*;
	pub trait BaseCurrencyUrlPart {
		/// Writes the base currency URL parameter into the string.
		///
		/// Returns whether there was a base currency to write.
		/// If there is something to write, writes `prefix` first.
		fn write(&self, writer: impl io::Write, prefix: &[u8]) -> io::Result<bool>;
	}

	impl BaseCurrencyUrlPart for NoBaseCurrency {
		#[inline] fn write(&self, _: impl io::Write, _: &[u8]) -> io::Result<bool> { Ok(false) }
	}

	impl BaseCurrencyUrlPart for BaseCurrency<CurrencyCode> {
		fn write(&self, mut writer: impl io::Write, prefix: &[u8]) -> io::Result<bool> {
			writer.write_all(prefix)?;
			writer.write_all(b"base_currency=")?;
			writer.write_all(self.0.as_ref())?;
			Ok(true)
		}
	}

	impl BaseCurrencyUrlPart for BaseCurrency<Option<CurrencyCode>> {
		fn write(&self, writer: impl io::Write, prefix: &[u8]) -> io::Result<bool> {
			match self.0 {
				Some(inner) => BaseCurrency(inner).write(writer, prefix),
				None => Ok(true),
			}
		}
	}
}

/// Types that can be used for [`Builder`]'s base currency.
pub trait BaseCurrencyUrlPart: private::BaseCurrencyUrlPart {}
impl BaseCurrencyUrlPart for NoBaseCurrency {}
impl BaseCurrencyUrlPart for BaseCurrency<CurrencyCode> {}
impl BaseCurrencyUrlPart for BaseCurrency<Option<CurrencyCode>> {}


impl<'a, Currencies, BaseCurrency> Builder<'a, Currencies, BaseCurrency> {
	/// Sets the [`currencies`](Builder::currencies).
	pub fn currencies<CurrenciesNew>(self, currencies: CurrenciesNew) -> Builder<'a, CurrenciesNew, BaseCurrency> {
		Builder {
			token: self.token,
			base_currency: self.base_currency,
			currencies,
		}
	}

	/// Sets the [`base_currency`](Builder::base_currency).
	pub fn base_currency<BaseCurrencyNew>(self, base_currency: BaseCurrencyNew) -> Builder<'a, Currencies, self::BaseCurrency<BaseCurrencyNew>> where self::BaseCurrency<BaseCurrencyNew>: BaseCurrencyUrlPart {
		Builder {
			token: self.token,
			base_currency: BaseCurrency(base_currency),
			currencies: self.currencies,
		}
	}

	/// Clears the [`base_currency`](Builder::base_currency) parameter. 
	pub fn base_currency_clear(self) -> Builder<'a, Currencies, NoBaseCurrency> {
		Builder {
			token: self.token,
			base_currency: NoBaseCurrency,
			currencies: self.currencies,
		}
	}
}

impl<'a> Builder<'a, AllCurrencies, NoBaseCurrency> {
	/// Creates a new [`Builder`] with the given [API token](Builder::token).
	pub const fn new(token: &'a str) -> Self {
		Builder {
			token,
			base_currency: NoBaseCurrency,
			currencies: std::iter::empty(),
		}
	}
}

impl<'a> From<&'a str> for Builder<'a, AllCurrencies> {
	#[inline] fn from(token: &'a str) -> Self { Self::new(token) }
}

impl<'a, Currencies: IntoIterator<Item = CurrencyCode>, BaseCurrency: BaseCurrencyUrlPart> Builder<'a, Currencies, BaseCurrency> {
	/// Builds the [`Request`].
	pub fn build(self) -> Request { self.into() }
}

/// The [`latest`](https://currencyapi.com/docs/latest) endpoint.
#[derive(Debug)]
pub struct Request(reqwest::Request);

impl Clone for Request {
	#[inline] fn clone(&self) -> Self {
		// try_clone should always succeed since there's no body stream.
		Self(self.0.try_clone().unwrap())
	}
}

impl Request {
	/// Sends the request.
	pub async fn send<const N: usize, T: TryFrom<f64>>(
		self,
		client: &reqwest::Client,
	) -> Result<Response<N, T>, Error> {
		let response = client.execute(self.0).await?;

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
		let mut rates = Rates::new();

		let data = payload
			.get("data")
			.and_then(|data| data.as_object())
			.ok_or(Error::ResponseParseError)?;
		for (currency, value_object) in data {
			if currency.as_str().len() != 3 { continue; } // XXX
			let currency =
				CurrencyCode::try_from(currency.as_str()).map_err(|_| Error::ResponseParseError)?;
			let rate = value_object
				.get("value")
				.and_then(|value| value.as_f64())
				.and_then(|value| T::try_from(value).ok())
				.ok_or(Error::ResponseParseError)?;
			rates.push(currency, rate);
		}

		Ok(Response {
			last_updated_at,
			rates,
			rate_limit,
		})
	}
}

impl<'a, Currencies: IntoIterator<Item = CurrencyCode>, BaseCurrency: BaseCurrencyUrlPart> From<Builder<'a, Currencies, BaseCurrency>>
	for Request
{
	fn from(builder: Builder<'a, Currencies, BaseCurrency>) -> Self {
		use std::io::Write;

		const URL_BUF_CAPACITY: usize = "https://api.currencyapi.com/v3/latest?base_currency=XXX&currencies=".len() + (crate::currency::list::ARRAY.len() + /* slack */ 30) * 4 - 1;
		let mut url_buf = [0u8; URL_BUF_CAPACITY];
		let mut writer = &mut url_buf[..];

		writer.write_all(b"https://api.currencyapi.com/v3/latest").unwrap();
		let sep = if builder.base_currency.write(&mut writer, b"?").unwrap() { b"&" } else { b"?" };

		let mut currencies = builder.currencies.into_iter();
		if let Some(currencies_head) = currencies.next() {
			writer.write_all(sep).unwrap();
			writer.write_all(b"currencies=").unwrap();
			writer.write_all(currencies_head.as_ref()).unwrap();
			for currency in currencies {
				writer.write_all(b",").unwrap();
				writer.write_all(currency.as_ref()).unwrap();
			}
		}

		let url = unsafe {
			// SAFETY: the buffer is built from valid UTF-8.
			std::str::from_utf8_unchecked(&url_buf)
		};
		let url = url.parse::<reqwest::Url>().unwrap();
		let mut request = reqwest::Request::new(reqwest::Method::GET, url);
		request.headers_mut().insert("apikey", builder.token.parse().unwrap());
		Self(request)
	}
}

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
