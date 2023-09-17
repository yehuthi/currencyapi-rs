//! [`Error`] type.

/// An error from the API or from the HTTP client.
#[derive(Debug, thiserror::Error)]
pub enum Error {
	/// The rate-limit was hit.
	#[error("exceeded rate limit or month limit")]
	RateLimitError,
	/// HTTP error.
	#[error("HTTP error: {0}")]
	HttpError(#[from] reqwest::Error),
	/// Failed to parse the response.
	#[error("failed to parse the response")]
	ResponseParseError,
	/// Failed to parse the rate-limit headers.
	#[error("failed to parse the rate-limits headers from the response")]
	RateLimitParseError,
}
