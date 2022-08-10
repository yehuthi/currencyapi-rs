//! [Currency codes](CurrencyCode).

use std::{
	error::Error,
	fmt::{self, Debug, Display, Formatter},
	num::NonZeroU8,
	str::FromStr,
};

/// [Currency code](https://en.wikipedia.org/wiki/ISO_4217).
///
/// It's recommended to use the constants in the [`currency::list`](list) module.
#[derive(Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct CurrencyCode {
	/// The code in uppercase alpha ASCII bytes.
	code: [NonZeroU8; 3],
}

impl Debug for CurrencyCode {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		let mut s = f.debug_struct("CurrencyCode");
		let code: &[NonZeroU8; 3] = &self.code;
		let code: &[u8; 3] = unsafe { std::mem::transmute(code) };
		if let Ok(code) = std::str::from_utf8(code) {
			s.field("code", &code);
		} else {
			s.field("code", &self.code);
		}
		s.finish()
	}
}

/// The default currency code is `USD`.
///
/// It is chosen for being the most traded currency.
impl Default for CurrencyCode {
	fn default() -> Self {
		unsafe { Self::from_bytes_unchecked(*b"USD") }
	}
}

impl CurrencyCode {
	/// Creates a new [`CurrencyCode`].
	///
	/// # Safety
	/// Ensure the code is uppercase alpha ASCII.
	pub const unsafe fn new_unchecked(code: [NonZeroU8; 3]) -> Self {
		Self { code }
	}

	/// Creates a new [`CurrencyCode`].
	///
	/// # Safety
	/// See [`Self::new_unchecked`].
	pub const unsafe fn from_bytes_unchecked(bytes: [u8; 3]) -> Self {
		Self {
			code: [
				NonZeroU8::new_unchecked(bytes[0]),
				NonZeroU8::new_unchecked(bytes[1]),
				NonZeroU8::new_unchecked(bytes[2]),
			],
		}
	}
}

impl TryFrom<[NonZeroU8; 3]> for CurrencyCode {
	type Error = InvalidCurrencyCodeError;

	fn try_from(value: [NonZeroU8; 3]) -> Result<Self, Self::Error> {
		Self::try_from([value[0].get(), value[1].get(), value[2].get()])
	}
}

impl TryFrom<[u8; 3]> for CurrencyCode {
	type Error = InvalidCurrencyCodeError;

	fn try_from(value: [u8; 3]) -> Result<Self, Self::Error> {
		if value.into_iter().all(|byte| byte.is_ascii_uppercase()) {
			Ok(unsafe {
				Self::new_unchecked([
					NonZeroU8::new_unchecked(value[0]),
					NonZeroU8::new_unchecked(value[1]),
					NonZeroU8::new_unchecked(value[2]),
				])
			})
		} else {
			Err(InvalidCurrencyCodeError)
		}
	}
}

impl<'a> TryFrom<&'a [u8]> for CurrencyCode {
	type Error = InvalidCurrencyCodeError;

	fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
		let values: [u8; 3] = value.try_into().map_err(|_| InvalidCurrencyCodeError)?;
		Self::try_from(values)
	}
}

impl<'a> TryFrom<&'a str> for CurrencyCode {
	type Error = InvalidCurrencyCodeError;

	fn try_from(value: &'a str) -> Result<Self, Self::Error> {
		Self::try_from(value.as_bytes())
	}
}

impl FromStr for CurrencyCode {
	type Err = InvalidCurrencyCodeError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let b = s.as_bytes();
		let mut b: [u8; 3] = b.try_into().map_err(|_| InvalidCurrencyCodeError)?;
		for char in &mut b {
			char.make_ascii_uppercase();
		}
		b.try_into()
	}
}

impl AsRef<[NonZeroU8]> for CurrencyCode {
	fn as_ref(&self) -> &[NonZeroU8] {
		&self.code
	}
}

impl AsRef<[u8]> for CurrencyCode {
	fn as_ref(&self) -> &[u8] {
		let code: &[NonZeroU8; 3] = &self.code;
		let code: &[u8; 3] = unsafe {
			// safety: NonZeroU8 is repr(transparent) on u8: https://doc.rust-lang.org/std/num/struct.NonZeroU8.html#:~:text=%23%5Brepr(transparent)%5D.
			std::mem::transmute(code)
		};
		code
	}
}

impl AsRef<str> for CurrencyCode {
	fn as_ref(&self) -> &str {
		unsafe {
			// safety: the code is always ASCII per the invariant documented in CurrencyCode::code therefore
			// valid UTF-8 .
			std::str::from_utf8_unchecked(self.as_ref())
		}
	}
}

impl Display for CurrencyCode {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		let code: &str = self.as_ref();
		Display::fmt(&code, f)
	}
}

/// Invalid currency code error.
///
/// Valid currency codes are three uppercase alpha ASCII characters.
#[derive(Debug, Hash, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct InvalidCurrencyCodeError;

impl Display for InvalidCurrencyCodeError {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt("invalid currency code", f)
	}
}

impl Error for InvalidCurrencyCodeError {}

pub mod list {
	//! A list of currencies.

	/// Defines const [`super::CurrencyCode`]s.
	///
	/// # Safety
	/// Ensure all arguments consist of only uppercase alpha characters.
	macro_rules! unsafe_define_currencies {
		($($currency:ident),*) => (
			$(
				#[doc=concat!("The [", stringify!($currency), "](https://www.google.com/search?q=USD+to+", stringify!($currency), ") currency code.")]
				pub const $currency: crate::currency::CurrencyCode = unsafe { crate::currency::CurrencyCode::from_bytes_unchecked(*bstringify::bstringify!($currency)) };
			)*
		);
	}

	// Currencies are documented here: https://currencyapi.com/docs/currency-list
	// To update this list, open dev-tools on the page, evaluate
	// ```js
	// [...document.querySelectorAll("td:first-child")].map(td => td.textContent).join()
	// ```
	// right click on the result, select "Copy string contents", and paste below between the parentheses.
	unsafe_define_currencies!(
		AED, AFN, ALL, AMD, ANG, AOA, ARS, AUD, AWG, AZN, BAM, BBD, BDT, BGN, BHD, BIF, BIH, BMD,
		BND, BOB, BRL, BSD, BTC, BTN, BWP, BYN, BYR, BZD, CAD, CDF, CHF, CLF, CLP, CNY, COP, CRC,
		CUC, CUP, CVE, CZK, DJF, DKK, DOP, DZD, EGP, ERN, ETB, ETH, EUR, FJD, FKP, GBP, GEL, GGP,
		GHS, GIP, GMD, GNF, GTQ, GYD, HKD, HNL, HRK, HRV, HTG, HUF, IDR, ILS, IMP, INR, IQD, IRR,
		ISK, JEP, JMD, JOD, JPY, KES, KGS, KHR, KMF, KPW, KRW, KWD, KYD, KZT, LAK, LBP, LKR, LRD,
		LSL, LTC, LTL, LVL, LYD, MAD, MDL, MGA, MKD, MMK, MNT, MOP, MRO, MUR, MVR, MWK, MXN, MYR,
		MZN, NAD, NGN, NIO, NOK, NPR, NZD, OMR, PAB, PEN, PGK, PHP, PKR, PLN, PYG, QAR, RON, RSD,
		RUB, RWF, SAR, SBD, SCR, SDG, SEK, SGD, SHP, SLL, SOS, SRD, SSP, STD, SVC, SYP, SZL, THB,
		TJS, TMT, TND, TOP, TRY, TTD, TWD, TZS, UAH, UGX, URY, USD, UYU, UZS, VEF, VND, VUV, WST,
		XAF, XAG, XAU, XCD, XDR, XOF, XPF, XRP, YER, ZAR, ZMK, ZWL
	);
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse() {
		assert_eq!(
			"USD".parse::<CurrencyCode>(),
			Ok(unsafe { CurrencyCode::from_bytes_unchecked(*b"USD") })
		);
	}

	#[test]
	fn test_parse_case_insensitive() {
		assert_eq!("usd".parse::<CurrencyCode>(), "USD".parse::<CurrencyCode>());
	}
}
