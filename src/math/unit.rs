use std::ops::{Deref, Mul, Neg};

/// A wrapper that ensures the underlying value has a unit norm.
#[derive(Clone, Copy, PartialEq)]
pub struct Unit<T> {
	unit: T,
}

impl<T> Unit<T> {
	/// Wraps the given value, assuming it is already normalized.
	pub const fn new_unchecked(unit: T) -> Self {
		Self { unit }
	}
}

impl<T> AsRef<T> for Unit<T> {
	fn as_ref(&self) -> &T {
		&self.unit
	}
}

impl<T> Deref for Unit<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.unit
	}
}

impl<T: Neg<Output=T>> Neg for Unit<T> {
	type Output = Unit<T>;

	fn neg(self) -> Self::Output {
		Unit::new_unchecked(-self.unit)
	}
}

impl<T: Mul<U>, U> Mul<U> for Unit<T> {
	type Output = T::Output;

	fn mul(self, rhs: U) -> Self::Output {
		self.unit * rhs
	}
}
