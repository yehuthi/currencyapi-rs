//! Currency rates container.

use std::{mem::{MaybeUninit, self}, fmt, ops::{Div, Mul}};

use crate::currency::CurrencyCode;

/// Currency rates.
pub struct Rates<const N: usize, RATE> {
	currency: [MaybeUninit<CurrencyCode>; N],
	rate: [MaybeUninit<RATE>; N],
	len: u8,
}

impl<const N: usize, RATE> Rates<N, RATE> {
	/// Creates a new [`Rates`] value.
	pub const fn new() -> Self { Self {
		currency: [MaybeUninit::uninit(); N],
		rate: unsafe {
			// SAFETY: mirrors MaybeUninit::unit_array implementation.
			MaybeUninit::<[MaybeUninit<RATE>; N]>::uninit().assume_init()
		},
		len: 0,
	} }

	/// Gets the count of rates.
	#[inline] pub const fn len(&self) -> usize { self.len as usize }
	/// Gets whether there are no rates.
	#[inline] pub const fn is_empty(&self) -> bool { self.len == 0 }
	/// Removes all rates.
	#[inline] pub fn clear(&mut self) { self.len = 0; }

	/// Gets a slice of the currencies.
	pub fn currencies(&self) -> &[CurrencyCode] {
		unsafe {
			// SAFETY: self.len keeps us safe.
			let currencies = self.currency.get_unchecked(..self.len as usize);
			// SAFETY: valid per MaybeUninit docs (array example).
			mem::transmute::<
				&[MaybeUninit<CurrencyCode>],
				&[CurrencyCode],
			>(currencies)
		}
	}

	/// Gets a slice of the rates.
	pub fn rates(&self) -> &[RATE] {
		unsafe {
			// SAFETY: self.len keeps us safe.
			let rates = self.rate.get_unchecked(..self.len as usize);
			// SAFETY: valid per MaybeUninit docs (array example).
			mem::transmute::<
				&[MaybeUninit<RATE>],
				&[RATE],
			>(rates)
		}
	}

	/// Iterates over currency rates.
	pub fn iter(&self) -> impl Iterator<Item = (CurrencyCode, &RATE)> {
		self.currencies().iter().copied().zip(self.rates().iter())
	}

	/// Pushes a new currency rate.
	///
	/// # Safety
	/// Ensure there is space for the new rate, i.e. that [`Rates::len`] < `N`.
	pub unsafe fn push_unchecked(&mut self, currency: CurrencyCode, rate: RATE) {
		let i = self.len as usize;
		*self.currency.get_unchecked_mut(i) = MaybeUninit::new(currency);
		*self.rate.get_unchecked_mut(i) = MaybeUninit::new(rate);
		self.len += 1;
	}

	/// Pushes a new currency rate, if the [`Rates`] is not full.
	///
	/// Returns whether the rate was inserted.
	pub fn push(&mut self, currency: CurrencyCode, rate: RATE) -> bool {
		if (self.len as usize) < N {
			unsafe {
				// SAFETY: there's space in this branch
				self.push_unchecked(currency, rate);
			}
			true
		} else { false }
	}

	/// Appends the given iterator rates, until full.
	///
	/// Returns whether all values were appended.
	pub fn extend_capped(&mut self, iter: impl IntoIterator<Item = (CurrencyCode, RATE)>) -> bool {
		for (currency, rate) in iter {
			if !self.push(currency, rate) { return false }
		}
		true
	}

	/// Gets the rate for the given currency, if exists.
	pub fn get(&self, currency: CurrencyCode) -> Option<&RATE> {
		self.iter()
			.find(|&(c,_)| c == currency)
			.map(|(_,r)| r)
	}

	/// Covnerts an amount between currencies.
	///
	/// Returns [`None`] if either the `from` or `to` currencies are missing.
	pub fn convert(&self, amount: &RATE, from: CurrencyCode, to: CurrencyCode) -> Option<RATE>
	where for<'x> &'x RATE: Div<&'x RATE, Output = RATE>, for<'x> &'x RATE: Mul<RATE, Output = RATE> {
		let from_value = self.get(from)?;
		let to_value = self.get(to)?;
		Some(amount * (to_value / from_value))
	}
}
impl<const N: usize, RATE> Default for Rates<N, RATE> { #[inline] fn default() -> Self { Self::new() } }

impl<const N: usize, RATE: fmt::Debug> fmt::Debug for Rates<N, RATE> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut m = f.debug_map();
		for (currency, rate) in self.iter() {
			m.entry(&currency, rate);
		}
		m.finish()
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_convert() {
		use crate::currency::list::*;
		let mut rates = Rates::<3, f64>::new();
		rates.push(USD, 1.0);
		rates.push(EUR, 0.9);
		rates.push(ILS, 3.1);
		assert_eq!(rates.convert(&1234.0, USD, USD), Some(1234.));
		assert_eq!(rates.convert(&1234.0, EUR, EUR), Some(1234.));
		assert_eq!(rates.convert(&1234.0, ILS, ILS), Some(1234.));
		assert_eq!(rates.convert(&1.0, ILS, EUR), Some(1. / 3.1 * 0.9));
		assert_eq!(rates.convert(&1.0, EUR, ILS), Some(1. / 0.9 * 3.1));
	}
}
