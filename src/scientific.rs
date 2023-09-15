//! [`FromScientific`]

/// Scientific notation parsing.
pub trait FromScientific: Sized {
	/// The parse error type.
	type Error;

	/// Parses a decimal number from a string.
	///
	/// The number representation may or may not be in scientific notation.
	fn parse_scientific(s: &str) -> Result<Self, Self::Error>;
}

impl FromScientific for f64 {
	type Error = serde_json::Error;
	fn parse_scientific(s: &str) -> Result<Self, Self::Error> { serde_json::from_str::<f64>(s) }
}

impl FromScientific for f32 {
	type Error = serde_json::Error;
	fn parse_scientific(s: &str) -> Result<Self, Self::Error> { serde_json::from_str::<f32>(s) }
}

#[cfg(feature = "rust_decimal")]
impl FromScientific for rust_decimal::Decimal {
	type Error = rust_decimal::Error;
	fn parse_scientific(s: &str) -> Result<Self, Self::Error> {
		// from_scientific rejects non-scientific so trying both
		s.parse::<Self>().or_else(|_| Self::from_scientific(s))
	}
}
