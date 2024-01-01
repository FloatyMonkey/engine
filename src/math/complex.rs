use super::Unit;

#[repr(C)]
pub struct Complex<T> {
	pub r: T,
	pub i: T,
}

/// A unit complex number. May be used to represent a 2D rotation.
pub type UnitComplex<T> = Unit<Complex<T>>;
