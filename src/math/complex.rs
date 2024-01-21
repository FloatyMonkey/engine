use super::Unit;

#[repr(C)]
pub struct Complex<T> {
	pub r: T,
	pub i: T,
}

impl<T> Complex<T> {
	pub const fn new(r: T, i: T) -> Self {
		Self { r, i }
	}
}

/// A unit complex number. May be used to represent a 2D rotation.
pub type UnitComplex<T> = Unit<Complex<T>>;
