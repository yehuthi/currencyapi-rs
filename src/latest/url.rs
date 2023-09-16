use std::io;

use super::{NoBaseCurrency, BaseCurrency, Builder};
use crate::currency::CurrencyCode;

impl<'a, Currency: AsRef<[u8]>, Currencies: IntoIterator<Item = Currency>, BaseCurrency: BaseCurrencyUrlPart> Builder<'a, Currencies, BaseCurrency> {
	pub(crate) fn write_url(self, mut writer: impl io::Write) -> io::Result<()> {
		writer.write_all(b"https://api.currencyapi.com/v3/latest")?;
		let sep = if self.base_currency.write_url_part(&mut writer, b"?")? { b"&" } else { b"?" };
		let mut currencies = self.currencies.into_iter();
		if let Some(currencies_head) = currencies.next() {
			writer.write_all(sep).unwrap();
			writer.write_all(b"currencies=").unwrap();
			writer.write_all(currencies_head.as_ref()).unwrap();
			for currency in currencies {
				writer.write_all(b",").unwrap();
				writer.write_all(currency.as_ref()).unwrap();
			}
		}
		Ok(())
	}
}

mod private {
	use std::io;

	use super::*;

	pub trait BaseCurrencyUrlPart {
		/// Writes the base currency URL parameter into the string.
		///
		/// Returns whether there was a base currency to write.
		/// If there is something to write, writes `prefix` first.
		fn write_url_part(&self, writer: impl io::Write, prefix: &[u8]) -> io::Result<bool>;
	}

	impl BaseCurrencyUrlPart for NoBaseCurrency {
		#[inline] fn write_url_part(&self, _: impl io::Write, _: &[u8]) -> io::Result<bool> { Ok(false) }
	}

	impl BaseCurrencyUrlPart for BaseCurrency<CurrencyCode> {
		fn write_url_part(&self, mut writer: impl io::Write, prefix: &[u8]) -> io::Result<bool> {
			writer.write_all(prefix)?;
			writer.write_all(b"base_currency=")?;
			writer.write_all(self.0.as_ref())?;
			Ok(true)
		}
	}

	impl BaseCurrencyUrlPart for BaseCurrency<Option<CurrencyCode>> {
		fn write_url_part(&self, writer: impl io::Write, prefix: &[u8]) -> io::Result<bool> {
			match self.0 {
				Some(inner) => BaseCurrency(inner).write_url_part(writer, prefix),
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
