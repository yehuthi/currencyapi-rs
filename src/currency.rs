//! [Currency codes](CurrencyCode).

use std::{
	error::Error,
	fmt::{self, Display, Formatter},
	num::NonZeroU8,
	str::FromStr,
};

/// [Currency code](https://en.wikipedia.org/wiki/ISO_4217).
#[derive(Debug, Hash, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct CurrencyCode {
	/// The code in uppercase alpha ASCII bytes.
	code: [NonZeroU8; 3],
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
		s.try_into()
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
		code.fmt(f)
	}
}

/// Invalid currency code error.
///
/// Valid currency codes are three uppercase alpha ASCII characters.
#[derive(Debug, Hash, Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct InvalidCurrencyCodeError;

impl Display for InvalidCurrencyCodeError {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		"invalid currency code".fmt(f)
	}
}

impl Error for InvalidCurrencyCodeError {}
