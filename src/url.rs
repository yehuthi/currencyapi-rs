//! URL building

use std::io;

pub mod capacity {
	// const ISO8601_LEN_MAX: usize = 30;
	const CURRENCIES_MAX_CAPACITY: usize = (crate::currency::list::ARRAY.len() + /* slack */ 10) * 4 - 1;

	// pub const URL_CAPACITY_STATUS: usize = "https://api.currencyapi.com/v3/status".len();
	// pub const URL_CAPACITY_CURRENCIES: usize = "https://api.currencyapi.com/v3/currencies?currencies=".len() + CURRENCIES_MAX_CAPACITY;
	pub const URL_CAPACITY_LATEST: usize = "https://api.currencyapi.com/v3/latest?base_currency=XXX&currencies=".len() + CURRENCIES_MAX_CAPACITY;
	// pub const URL_CAPACITY_HISTORICAL: usize = "https://api.currencyapi.com/v3/historical?base_currency=XXX&date=0000-00-00&currencies=".len() + CURRENCIES_MAX_CAPACITY;
	// pub const URL_CAPACITY_RANGE: usize = "https://api.currencyapi.com/v3/range?datetime_start=".len() + ISO8601_LEN_MAX + "&datetime_end=".len() + ISO8601_LEN_MAX + "&accuracy=quarter_hour&base_currency=XXX&currencies=".len() + CURRENCIES_MAX_CAPACITY;
}

pub trait UrlPart: Sized {
	#[inline] fn write_url_part(self, _write: impl io::Write, _prefix: &[u8]) -> io::Result<bool> { Ok(false) }
}

impl<Inner: UrlPart> UrlPart for Option<Inner> {
	#[inline] fn write_url_part(self, write: impl io::Write, prefix: &[u8]) -> io::Result<bool> {
		if let Some(inner) = self {
			inner.write_url_part(write, prefix)
		} else { Ok(false) }
	}
}

pub mod base {
	use super::UrlPart;

	pub struct BaseUrl(&'static str);

	macro_rules! defbase {
		($base:literal, $($id:ident <- $endpoint:literal),* $(,)?) => {
			$(
				#[doc = concat!("[`BaseUrl`] to the `", $endpoint, "` endpoint.")]
				pub const $id: BaseUrl = BaseUrl(concat!($base, $endpoint));
			)*
		};
	}

	defbase!("https://api.currencyapi.com/v3/",
		// STATUS <- "status",
		// CURRENCIES <- "currencies",
		LATEST <- "latest",
		// HISTORICAL <- "historical",
		// RANGE   <- "range",
		// CONVERT <- "convert",
	);

	impl UrlPart for BaseUrl {
		#[inline] fn write_url_part(self, mut write: impl std::io::Write, prefix: &[u8]) -> std::io::Result<bool> {
			write.write_all(prefix)?;
			write.write_all(self.0.as_ref())?;
			Ok(true)
		}
	}
}
pub use base::*;

mod base_currency {
	use crate::CurrencyCode;

	use super::UrlPart;

	/// A base currency parameter for [`Builder`].
	pub struct BaseCurrency<T>(pub T);

	/// A type for [`Builder`] indicating the request does not specify a base currency parameter.
	pub struct NoBaseCurrency;

	impl UrlPart for NoBaseCurrency {}

	impl<'a> UrlPart for BaseCurrency<&'a str> {
		#[inline] fn write_url_part(self, mut write: impl std::io::Write, prefix: &[u8]) -> std::io::Result<bool> {
			write.write_all(prefix)?;
			write.write_all(b"base_currency=")?;
			write.write_all(self.0.as_ref())?;
			Ok(true)
		}
	}

	impl<'a> UrlPart for BaseCurrency<CurrencyCode> {
		#[inline] fn write_url_part(self, write: impl std::io::Write, prefix: &[u8]) -> std::io::Result<bool> {
			BaseCurrency(self.0.as_ref()).write_url_part(write, prefix)
		}
	}

	impl UrlPart for BaseCurrency<Option<CurrencyCode>> {
		#[inline] fn write_url_part(self, write: impl std::io::Write, prefix: &[u8]) -> std::io::Result<bool> {
			self.0.map(BaseCurrency).write_url_part(write, prefix)
		}
	}
}
pub use base_currency::{BaseCurrency, NoBaseCurrency};

mod currencies {
	use crate::CurrencyCode;

	use super::UrlPart;

	pub struct Currencies<T>(pub T);

	impl<T: IntoIterator<Item = CurrencyCode>> UrlPart for Currencies<T> {
		fn write_url_part(self, mut write: impl std::io::Write, prefix: &[u8]) -> std::io::Result<bool> {
			let mut iter = self.0.into_iter();
			if let Some(head) = iter.next() {
				write.write_all(prefix)?;
				write.write_all(b"currencies=")?;
				write.write_all(head.as_ref())?;
				for currency in iter {
					write.write_all(b",")?;
					write.write_all(currency.as_ref())?;
				}
				Ok(true)
			} else { Ok(false) }
		}
	}
}
pub use currencies::Currencies;
