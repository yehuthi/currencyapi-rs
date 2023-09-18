//! [Currency codes](CurrencyCode).

use std::{
	fmt::{self, Debug, Display, Formatter},
	num::NonZeroU8, str::FromStr, mem, ptr, hash::Hash,
};

use serde::{Serialize, Serializer, Deserialize, Deserializer};

const CURRENCY_LEN_MIN: usize = 2;
const CURRENCY_LEN_MAX: usize = 5;

/// [Currency code](https://en.wikipedia.org/wiki/ISO_4217).
///
/// It's recommended to use the constants in the [`currencies`](list) module.
#[derive(Debug, Clone, Copy)]
#[repr(C, align(8))]
pub struct CurrencyCode {
	// Notes about the representation of the code:
	// - Variable-length (CURRENCY_LEN_MIN to CURRENCY_LEN_MAX).
	// - Stored in 8 bytes.
	// - Its value is the code in uppercase ASCII, followed by zeroes.
	// - The first CURRENCY_LEN_MIN is split as NonZeroU8 to enable niche optimization.

	/// The first `CURRENCY_LEN_MIN` letters of the code.
	code_head: [NonZeroU8; CURRENCY_LEN_MIN],
	/// The tail of the code.
	code_tail: [u8; CURRENCY_LEN_MAX - CURRENCY_LEN_MIN],
	/// Padding, must be zeroed out.
	padding: [u8; 8 - CURRENCY_LEN_MAX],
}

impl CurrencyCode {
	const fn as_u64(self) -> u64 {
		unsafe {
			*(&self as *const Self as *const u8 as *const u64)
		}
	}
}

/// The default currency code is [`USD`](list::USD).
///
/// It is chosen for being the most traded currency.
impl Default for CurrencyCode { #[inline] fn default() -> Self { list::USD } }

impl PartialEq for CurrencyCode {
	#[inline] fn eq(&self, other: &Self) -> bool { self.as_u64() == other.as_u64() }
} impl Eq for CurrencyCode {}

impl Hash for CurrencyCode {
	#[inline] fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.as_u64().hash(state) }
}

impl PartialOrd for CurrencyCode {
	#[inline] fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		self.as_u64().partial_cmp(&other.as_u64())
	}
}

impl Ord for CurrencyCode {
	#[inline] fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.as_u64().cmp(&other.as_u64())
	}
}

impl CurrencyCode {
	/// Creates a new [`CurrencyCode`] value.
	///
	/// # Safety
	/// - See comments for [`CurrencyCode`] struct.
	/// - The const parameter `N` must be in the range [CURRENCY_LEN_MIN..CURRENCY_LEN_MAX].
	const unsafe fn from_array_unchecked<const N: usize>(code: [u8; N]) -> Self {
		let mut buf = [0u8; mem::size_of::<CurrencyCode>()];
		let mut n = 0;
		while n < N {
			buf[n] = code[n];
			n += 1;
		}
		std::mem::transmute(buf)
	}

	/// Creates a new [`CurrencyCode`] value.
	///
	/// # Safety
	/// Ensure that the code's length is within range [2..5].
	/// The code must consist only of uppercase ASCII characters, and be terminated by zeroes until
	/// the end of the slice.
	pub unsafe fn new_unchecked(code: &[u8]) -> Self {
		let mut buf = [0u8; CURRENCY_LEN_MAX];
		ptr::copy_nonoverlapping::<u8>(
			code.as_ptr(),
			buf.as_mut_ptr(),
			code.len()
		);
		Self::from_array_unchecked(buf)
	}
}

impl TryFrom<&[u8]> for CurrencyCode {
	type Error = Error;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let len = value.len();
		if len < CURRENCY_LEN_MIN { return Err(Error::TooShort); }
		if len > CURRENCY_LEN_MAX { return Err(Error::TooLong); }
		let bad_char = value[..CURRENCY_LEN_MIN].iter().find(|&&c| !c.is_ascii_uppercase())
			.and(value[CURRENCY_LEN_MIN..].iter().find(|&&c| !c.is_ascii_uppercase() && c != 0))
			.copied();
		if let Some(bad_char) = bad_char { return Err(Error::InvalidCharacter(bad_char)); }
		unsafe { Ok(Self::new_unchecked(value)) }
	}
}

impl FromStr for CurrencyCode {
	type Err = Error;
	#[inline] fn from_str(s: &str) -> Result<Self, Self::Err> { <Self as TryFrom<&[u8]>>::try_from(s.as_ref()) }
}

impl AsRef<[u8]> for CurrencyCode {
	#[inline] fn as_ref(&self) -> &[u8] {
		let tail_len = self.code_tail.into_iter().take_while(|&c| c != 0).count();
		unsafe {
			// SAFETY:
			// (1) `tail` adjacently tails `head` (per repr(C), tested).
			// (2) NonZeroU8 is repr(transparent) on u8: https://doc.rust-lang.org/std/num/struct.NonZeroU8.html#:~:text=%23%5Brepr(transparent)%5D.
			std::slice::from_raw_parts(
				self as *const Self as *const u8,
				CURRENCY_LEN_MIN + tail_len
			)
		}
	}
}

impl AsRef<str> for CurrencyCode {
	#[inline] fn as_ref(&self) -> &str {
		unsafe {
			// safety: the code is always ASCII per the invariant documented in CurrencyCode::code therefore
			// valid UTF-8 .
			std::str::from_utf8_unchecked(self.as_ref())
		}
	}
}

impl Display for CurrencyCode {
	#[inline] fn fmt(&self, f: &mut Formatter) -> fmt::Result { Display::fmt(AsRef::<str>::as_ref(&self), f) }
}

impl Serialize for CurrencyCode {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		<Self as AsRef<str>>::as_ref(&self).serialize(serializer)
	}
}

impl<'de> Deserialize<'de> for CurrencyCode {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		struct Visitor;

		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = CurrencyCode;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("a currency code")
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: serde::de::Error {
				v.parse().map_err(serde::de::Error::custom)
			}
		}

		deserializer.deserialize_str(Visitor)
	}
}

/// Invalid currency code error.
///
/// Valid currency codes are three uppercase alpha ASCII characters.
#[derive(Debug, thiserror::Error)]
pub enum Error {
	/// The currency code is too short.
	#[error("the currency code is too short")]
	TooShort,
	/// The currency code is too long.
	#[error("the currency code is too long")]
	TooLong,
	/// The currency code has an invalid character.
	#[error("invalid currency code character ({0:?})")]
	InvalidCharacter(u8),
}

pub mod list {
	//! [Currencies](super::CurrencyCode) constants.
	//!
	//! This module defines all known currencies as constants, as well as [`ARRAY`]
	//! which contains all of them in a constant array.

	/// Defines const [`super::CurrencyCode`]s.
	///
	/// # Safety
	/// Ensure all arguments consist of only uppercase alpha characters.
	macro_rules! unsafe_define_currencies {
		($from_fn:expr, $($currency:ident),*) => {
			$(
				#[doc=concat!("The [", stringify!($currency), "](https://www.google.com/search?q=USD+to+", stringify!($currency), ") currency code.")]
				pub const $currency: crate::CurrencyCode = unsafe { crate::CurrencyCode::from_array_unchecked(*bstringify::bstringify!($currency)) };
			)*
			/// The length of all currencies defined in this module.
			const LEN: usize = 0 $(+ { stringify!($currency); 1} )*;
			/// An array of all the currencies defined in this module.
			pub const ARRAY: [crate::CurrencyCode; LEN] = [ $( $currency ),* ];
		};
	}

	// Currencies are documented here: https://currencyapi.com/docs/currency-list
	// DEPRECATED NOTE:
	//	   To update this list, open dev-tools on the page, evaluate
	//	   ```js
	//	   [...document.querySelectorAll("td:first-child")].map(td => td.textContent).join()
	//	   ```
	//	   right click on the result, select "Copy string contents", and paste below between the parentheses.
	// The docs aren't synced tightly enough, it's better to update by making a request and pulling
	// the currencies from there. This can be easily done in the [currencyapi
	// dashboard](https://app.currencyapi.com/dashboard).
	// Paste into browser developer console and:
	// ```js
	// Object.keys(payload.data).join(", ")
	// ```
	unsafe_define_currencies!(
		ADA, AED, AFN, ALL, AMD, ANG, AOA, ARB, ARS, AUD, AVAX, AWG, AZN, BAM, BBD, BDT, BGN, BHD, BIF, BMD, BNB, BND, BOB, BRL, BSD, BTC, BTN, BUSD, BWP, BYN, BYR, BZD, CAD, CDF, CHF, CLF, CLP, CNY, COP, CRC, CUC, CUP, CVE, CZK, DAI, DJF, DKK, DOP, DOT, DZD, EGP, ERN, ETB, ETH, EUR, FJD, FKP, GBP, GEL, GGP, GHS, GIP, GMD, GNF, GTQ, GYD, HKD, HNL, HRK, HTG, HUF, IDR, ILS, IMP, INR, IQD, IRR, ISK, JEP, JMD, JOD, JPY, KES, KGS, KHR, KMF, KPW, KRW, KWD, KYD, KZT, LAK, LBP, LKR, LRD, LSL, LTC, LTL, LVL, LYD, MAD, MATIC, MDL, MGA, MKD, MMK, MNT, MOP, MRO, MUR, MVR, MWK, MXN, MYR, MZN, NAD, NGN, NIO, NOK, NPR, NZD, OMR, OP, PAB, PEN, PGK, PHP, PKR, PLN, PYG, QAR, RON, RSD, RUB, RWF, SAR, SBD, SCR, SDG, SEK, SGD, SHP, SLL, SOL, SOS, SRD, STD, SVC, SYP, SZL, THB, TJS, TMT, TND, TOP, TRY, TTD, TWD, TZS, UAH, UGX, USD, USDC, USDT, UYU, UZS, VEF, VND, VUV, WST, XAF, XAG, XAU, XCD, XDR, XOF, XPD, XPF, XPT, XRP, YER, ZAR, ZMK, ZMW, ZWL
	);
}

#[cfg(test)]
mod tests {
	use super::*;

	const AVAX_MANUAL: CurrencyCode = CurrencyCode {
		code_head: unsafe { [
			NonZeroU8::new_unchecked(b'A'),
			NonZeroU8::new_unchecked(b'V'),
		] },
		code_tail: [b'A', b'X', 0],
		padding: [0; 8 - CURRENCY_LEN_MAX],
		};

	#[test]
	fn test_repr() {
		assert_eq!(
			mem::size_of::<CurrencyCode>(),
			mem::size_of::<u64>(),
			"sizeof(CurrencyCode) = sizeof(u64)"
		);

		let avax = AVAX_MANUAL;
		assert_eq!(
			&avax as *const _ as usize,
			avax.code_head.as_ptr() as usize,
			"&currency = &currency.code_head"
		);
		assert_eq!(
			avax.code_head.as_ptr() as usize + CURRENCY_LEN_MIN,
			avax.code_tail.as_ptr() as usize
		);
	}

	#[test]
	fn test_as_ref_bytes_4() {
		assert_eq!(
			<CurrencyCode as AsRef<[u8]>>::as_ref(&AVAX_MANUAL),
			b"AVAX",
		);
	}

	#[test]
	fn test_parse_1() {
		match "A".parse::<CurrencyCode>() {
			Err(Error::TooShort) => {},
			_ => panic!(),
		}
	}

	#[test]
	fn test_parse_2() {
		assert_eq!(
			"OP".parse::<CurrencyCode>().unwrap(),
			unsafe { CurrencyCode::from_array_unchecked(*b"OP") },
		);
	}

	#[test]
	fn test_parse_3() {
		assert_eq!(
			"USD".parse::<CurrencyCode>().unwrap(),
			unsafe { CurrencyCode::from_array_unchecked(*b"USD") },
		);
	}

	#[test]
	fn test_parse_4() {
		assert_eq!(
			"AVAX".parse::<CurrencyCode>().unwrap(),
			unsafe { CurrencyCode::from_array_unchecked(*b"AVAX") },
		);
	}

	#[test]
	fn test_parse_5() {
		assert_eq!(
			"MATIC".parse::<CurrencyCode>().unwrap(),
			unsafe { CurrencyCode::from_array_unchecked(*b"MATIC") },
		);
	}

	#[test]
	fn test_parse_6() {
		match "ABCDEF".parse::<CurrencyCode>() {
			Err(Error::TooLong) => {},
			_ => panic!(),
		}
	}

	#[test]
	fn test_serde() {
		let value = crate::currency::USD;
		let json = "\"USD\"";
		let serialized = serde_json::to_string::<CurrencyCode>(&value).unwrap();
		let deserialized = serde_json::from_str::<CurrencyCode>(json).unwrap();
		assert_eq!(serialized, json);
		assert_eq!(deserialized, value);
	}
}
