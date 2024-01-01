use super::Number;
use std::ops::{Add, Div, Mul, Sub};

#[repr(C)]
pub struct Dual<T> {
	pub r: T,
	pub d: T,
}

impl<T: Number> Add for Dual<T> {
	type Output = Dual<T>;

	fn add(self, rhs: Dual<T>) -> Self::Output {
		Self {
			r: self.r + rhs.r,
			d: self.d + rhs.d,
		}
	}
}

impl<T: Number> Sub for Dual<T> {
	type Output = Dual<T>;

	fn sub(self, rhs: Dual<T>) -> Self::Output {
		Self {
			r: self.r - rhs.r,
			d: self.d - rhs.d,
		}
	}
}

impl<T: Number> Mul for Dual<T> {
	type Output = Dual<T>;

	fn mul(self, rhs: Dual<T>) -> Self::Output {
		Self {
			r: self.r * rhs.r,
			d: self.r * rhs.d + self.d * rhs.r,
		}
	}
}

impl<T: Number> Div for Dual<T> {
	type Output = Dual<T>;

	fn div(self, rhs: Dual<T>) -> Self::Output {
		Self {
			r: self.r / rhs.r,
			d: (self.d * rhs.r - self.r * rhs.d) / (rhs.r * rhs.r),
		}
	}
}
