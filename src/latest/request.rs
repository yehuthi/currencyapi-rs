use std::{collections::HashMap, str::FromStr};

use serde::Deserialize;
use serde_json::value::RawValue;

use crate::{currency::CurrencyCode, scientific::FromScientific, rates::Rates, Error, rate_limit::RateLimitData};

use super::{url::BaseCurrencyUrlPart, Metadata};

/// Request to the [`latest`](https://currencyapi.com/docs/latest) endpoint.
#[derive(Debug)]
pub struct Request(pub(crate) reqwest::Request);

impl Clone for Request {
	#[inline] fn clone(&self) -> Self {
		// try_clone should always succeed since there should never be a body stream.
		Self(self.0.try_clone().unwrap())
	}
}

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

/// A [`Builder`] buffer for all currencies.
pub type AllCurrencies = std::iter::Empty<CurrencyCode>;

/// A base currency parameter for [`Builder`].
pub struct BaseCurrency<T>(pub T);

/// A type for [`Builder`] indicating the request does not specify a base currency parameter.
pub struct NoBaseCurrency;

impl<'a> From<&'a str> for Builder<'a, AllCurrencies, NoBaseCurrency> {
	#[inline] fn from(token: &'a str) -> Self { Self::new(token) }
}

impl<'a, Currencies, BaseCurrency> Builder<'a, Currencies, BaseCurrency> {
	/// Sets the [`currencies`](Builder::currencies).
	#[inline] pub fn currencies<CurrenciesNew>(self, currencies: CurrenciesNew) -> Builder<'a, CurrenciesNew, BaseCurrency> {
		Builder {
			token: self.token,
			base_currency: self.base_currency,
			currencies,
		}
	}

	/// Sets the [`base_currency`](Builder::base_currency).
	#[inline] pub fn base_currency<BaseCurrencyNew>(self, base_currency: BaseCurrencyNew) -> Builder<'a, Currencies, self::BaseCurrency<BaseCurrencyNew>> where self::BaseCurrency<BaseCurrencyNew>: BaseCurrencyUrlPart {
		Builder {
			token: self.token,
			base_currency: BaseCurrency(base_currency),
			currencies: self.currencies,
		}
	}

	/// Clears the [`base_currency`](Builder::base_currency) parameter.
	#[inline] pub fn base_currency_clear(self) -> Builder<'a, Currencies, NoBaseCurrency> {
		Builder {
			token: self.token,
			base_currency: NoBaseCurrency,
			currencies: self.currencies,
		}
	}
}

impl<'a> Builder<'a, AllCurrencies, NoBaseCurrency> {
	/// Creates a new [`Builder`] with the given [API token](Builder::token).
	#[inline] pub const fn new(token: &'a str) -> Self {
		Builder {
			token,
			base_currency: NoBaseCurrency,
			currencies: std::iter::empty(),
		}
	}
}

impl<'a, Currencies: IntoIterator<Item = CurrencyCode>, BaseCurrency: BaseCurrencyUrlPart> Builder<'a, Currencies, BaseCurrency> {
	/// Builds the [`Request`].
	#[inline] pub fn build(self) -> Request { self.into() }
}

impl<'a, Currencies: IntoIterator<Item = CurrencyCode>, BaseCurrency: BaseCurrencyUrlPart> From<Builder<'a, Currencies, BaseCurrency>> for Request {
	#[inline] fn from(builder: Builder<'a, Currencies, BaseCurrency>) -> Self {
		const URL_BUF_CAPACITY: usize = "https://api.currencyapi.com/v3/latest?base_currency=XXX&currencies=".len() + (crate::currency::list::ARRAY.len() + /* slack */ 30) * 4 - 1;
		let mut url_buf = [0u8; URL_BUF_CAPACITY];
		let mut writer = &mut url_buf[..];
		let token = builder.token;
		builder.write_url(&mut writer).expect("failed to construct /latest request URL");

		let url_len = writer.as_ptr() as usize - url_buf.as_ptr() as usize;
		let url_buf = &url_buf[..url_len];
		let url = unsafe {
			// SAFETY: the buffer is built from valid UTF-8.
			std::str::from_utf8_unchecked(&url_buf)
		};
		let url = url.parse::<reqwest::Url>().unwrap();
		let mut request = reqwest::Request::new(reqwest::Method::GET, url);
		request.headers_mut().insert("apikey", token.parse().unwrap());
		Self(request)
	}
}

impl Request {
	/// Sends the request.
	#[inline] pub async fn send<const N: usize, DateTime: FromStr, RATE: FromScientific, RateLimit: for<'x> RateLimitData<'x>>(
		self,
		rates: &mut Rates<RATE, N>,
		client: &reqwest::Client,
	) -> Result<Metadata<DateTime, RateLimit>, Error> {
		let response = client.execute(self.0).await?;
		if response.status() == 429 { return Err(Error::RateLimitError); }
		let response = response.error_for_status()?;

		#[derive(Deserialize)]
		struct Payload<'a> {
			#[serde(borrow)]
			meta: PayloadMeta<'a>,
			#[serde(borrow)]
			data: PayloadData<'a>,
		}

		#[derive(Deserialize)]
		struct PayloadMeta<'a> { last_updated_at: &'a str }

		#[derive(Deserialize)]
		struct PayloadData<'a> (#[serde(borrow)] HashMap<&'a str, PayloadDataEntry<'a>>);

		#[derive(Deserialize)]
		struct PayloadDataEntry<'a> { #[serde(borrow)] value: &'a RawValue }

		let rate_limit = (&response)
			.try_into()
			.map_err(|_| Error::RateLimitParseError)?;
		let payload = response.bytes().await?;
		let payload = serde_json::from_slice::<Payload>(&payload).unwrap();
		let last_updated_at = payload.meta.last_updated_at.parse::<DateTime>().unwrap_or_else(|_| todo!());
		rates.extend_capped(
			payload.data.0.iter()
				.map(|(&currency, entry)| (currency.parse().unwrap(), RATE::parse_scientific(entry.value.get()).unwrap_or_else(|_| todo!())))
		);
		Ok(Metadata {
			last_updated_at,
			rate_limit,
		})
	}
}