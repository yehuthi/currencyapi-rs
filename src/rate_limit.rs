//! [`RateLimit`]

use std::convert::Infallible;

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

/// Ignore rate limit data.
pub struct RateLimitIgnore;

impl TryFrom<&reqwest::Response> for RateLimit {
	type Error = ();

	fn try_from(value: &reqwest::Response) -> Result<Self, Self::Error> {
		let headers = value.headers();
		let h = |name| {
			headers
				.get(name)
				.ok_or(())
				.and_then(|value| atoi::atoi(value.as_bytes()).ok_or(()))
		};
		Ok(Self {
			limit_minute: h("X-RateLimit-Limit-Quota-Minute")?,
			limit_month: h("X-RateLimit-Limit-Quota-Month")?,
			remainig_minute: h("X-RateLimit-Remaining-Quota-Minute")?,
			remaining_month: h("X-RateLimit-Remaining-Quota-Month")?,
		})
	}
}

impl TryFrom<&reqwest::Response> for RateLimitIgnore {
	type Error = Infallible;
	#[inline] fn try_from(_: &reqwest::Response) -> Result<Self, Self::Error> { Ok(RateLimitIgnore) }
}


mod private {
	use super::*;
	pub trait Sealed<'a>: TryFrom<&'a reqwest::Response> {}
	impl<'a> Sealed<'a> for RateLimit {}
	impl<'a> Sealed<'a> for RateLimitIgnore {}
}

pub trait RateLimitData<'a>: private::Sealed<'a> {}
impl<'a> RateLimitData<'a> for RateLimit {}
impl<'a> RateLimitData<'a> for RateLimitIgnore {}
