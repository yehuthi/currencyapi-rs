//! [currencyapi](https://currencyapi.com/) API library.

#![deny(missing_docs)]

use std::fmt::{self, Display, Formatter};

use arrayvec::{ArrayString, CapacityError};

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
