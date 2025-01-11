use core::ops::{Mul, Add, Sub, Div, Rem, Neg, Shl, Shr, BitOr, BitAnd, BitXor};
use core::ops::{MulAssign, AddAssign, SubAssign, DivAssign, RemAssign, ShlAssign, ShrAssign, BitOrAssign, BitAndAssign, BitXorAssign};
use core::cmp::{PartialEq, PartialOrd};

/// Forward a method to an inherent method or a base trait method.
macro_rules! forward {
	($( Self :: $method:ident ( self $( , $arg:ident : $ty:ty )* ) -> $ret:ty ; )*) => {$(
		#[inline]
		fn $method(self $( , $arg : $ty )* ) -> $ret {
			Self::$method(self $( , $arg )* )
		}
	)*};
}

pub trait Cast<T: Number> where Self: Sized {
	fn from_f64(v: f64) -> Self;
	fn as_f64(&self) -> f64;
}

pub fn cast<T: Number, U: Number>(v: T) -> U {
	U::from_f64(v.as_f64()) // TODO: cast immediately to correct type
}

pub trait NumOps<Rhs = Self, Output = Self>:
	Add<Rhs, Output = Output> +
	Sub<Rhs, Output = Output> +
	Mul<Rhs, Output = Output> +
	Div<Rhs, Output = Output> +
	Rem<Rhs, Output = Output>
{}

impl<T, Rhs, Output> NumOps<Rhs, Output> for T where T:
	Add<Rhs, Output = Output> +
	Sub<Rhs, Output = Output> +
	Mul<Rhs, Output = Output> +
	Div<Rhs, Output = Output> +
	Rem<Rhs, Output = Output>
{}

pub trait NumAssignOps<Rhs = Self>:
	AddAssign<Rhs> +
	SubAssign<Rhs> +
	MulAssign<Rhs> +
	DivAssign<Rhs> +
	RemAssign<Rhs>
{}

impl<T, Rhs> NumAssignOps<Rhs> for T where T:
	AddAssign<Rhs> +
	SubAssign<Rhs> +
	MulAssign<Rhs> +
	DivAssign<Rhs> +
	RemAssign<Rhs>
{}

pub trait Base<T: Number>: Copy + NumOps<T, T> + NumAssignOps<T> + where Self: Sized {
	const ZERO: Self;
	const ONE: Self;
	const TWO: Self;

	const MIN: Self;
	const MAX: Self;
}

pub trait Number: Base<Self> + Cast<Self> + Default + PartialEq + PartialOrd  {}

pub trait NumberOps<T: Number> {
	fn min(a: Self, b: Self) -> Self;
	fn max(a: Self, b: Self) -> Self;
}

macro_rules! number_impl {
	($t:ident) => {
		impl Base<$t> for $t {
			const ZERO: Self = 0 as Self;
			const ONE: Self = 1 as Self;
			const TWO: Self = 2 as Self;

			const MIN: Self = $t::MIN;
			const MAX: Self = $t::MAX;
		}

		impl Number for $t {}

		impl NumberOps<$t> for $t {
			fn min(a: Self, b: Self) -> Self {
				a.min(b)
			}

			fn max(a: Self, b: Self) -> Self {
				a.max(b)
			}
		}

		impl Cast<$t> for $t {
			fn from_f64(v: f64) -> Self {
				v as Self
			}

			fn as_f64(&self) -> f64 {
				*self as f64
			}
		}
	}
}

number_impl!(u8);
number_impl!(i8);
number_impl!(u16);
number_impl!(i16);
number_impl!(u32);
number_impl!(i32);
number_impl!(u64);
number_impl!(i64);
number_impl!(u128);
number_impl!(i128);
number_impl!(usize);
number_impl!(isize);
number_impl!(f32);
number_impl!(f64);

pub trait SignedNumber: Number + Neg<Output=Self> {
	const MINUS_ONE: Self;
}

pub trait SignedNumberOps<T: SignedNumber>: Neg<Output=Self> {
	fn signum(self) -> Self;
	fn abs(self) -> Self;
}

macro_rules! signed_number_impl {
	($t:ident, $minus_one:literal) => {
		impl SignedNumber for $t {
			const MINUS_ONE: Self = $minus_one;
		}

		impl SignedNumberOps<$t> for $t {
			forward! {
				Self::signum(self) -> Self;
				Self::abs(self) -> Self;
			}
		}
	}
}

signed_number_impl!(i8, -1);
signed_number_impl!(i16, -1);
signed_number_impl!(i32, -1);
signed_number_impl!(i64, -1);
signed_number_impl!(i128, -1);
signed_number_impl!(f32, -1.0);
signed_number_impl!(f64, -1.0);

pub trait Integer: Number +
	Shl<Output=Self> + ShlAssign +
	Shr<Output=Self> + ShrAssign +
	BitOr<Output=Self> + BitOrAssign +
	BitAnd<Output=Self> + BitAndAssign +
	BitXor<Output=Self> + BitXorAssign {
}

pub trait IntegerOps<T: Integer> {
	fn pow(self, exp: u32) -> Self;
}

macro_rules! integer_impl {
	($t:ident) => {
		impl Integer for $t {
		}

		impl IntegerOps<$t> for $t {
			forward! {
				Self::pow(self, exp: u32) -> Self;
			}
		}
	}
}

integer_impl!(u8);
integer_impl!(i8);
integer_impl!(u16);
integer_impl!(i16);
integer_impl!(u32);
integer_impl!(i32);
integer_impl!(u64);
integer_impl!(i64);
integer_impl!(u128);
integer_impl!(i128);

pub trait Float: SignedNumber {
	const SMALL_EPSILON: Self;
}

pub trait FloatOps<T: Float>: where Self: Sized {
	fn acos(self) -> Self;
	fn approx(self, b: Self, eps: T) -> bool;
	fn asin(self) -> Self;
	fn atan(self) -> Self;
	fn atan2(self, x: Self) -> Self;
	fn ceil(self) -> Self;
	fn copysign(self, sign: T) -> Self;
	fn cos(self) -> Self;
	fn cosh(self) -> Self;
	fn exp(self) -> Self;
	fn exp2(self) -> Self;
	fn floor(self) -> Self;
	fn fract(self) -> Self;
	fn is_finite(self) -> bool;
	fn is_infinite(self) -> bool;
	fn is_nan(self) -> bool;
	fn log(self, base: T) -> Self;
	fn log10(self) -> Self;
	fn log2(self) -> Self;
	fn mul_add(self, a: Self, b: Self) -> Self;
	fn powf(self, exp: T) -> Self;
	fn powi(self, exp: i32) -> Self;
	fn recip(self) -> Self;
	fn round(self) -> Self;
	fn sin_cos(self) -> (Self, Self);
	fn sin(self) -> Self;
	fn sinh(self) -> Self;
	fn sqrt(self) -> Self;
	fn tan(self) -> Self;
	fn tanh(self) -> Self;
	fn to_degrees(self) -> Self;
	fn to_radians(self) -> Self;
	fn trunc(self) -> Self;
}

macro_rules! float_impl {
	($t:ident) => {
		impl Float for $t {
			const SMALL_EPSILON: Self = 1e-30;
		}

		impl FloatOps<$t> for $t {
			fn approx(self, b: Self, eps: Self) -> bool {
				Self::abs(self - b) < eps
			}

			forward! {
				Self::acos(self) -> Self;
				Self::asin(self) -> Self;
				Self::atan(self) -> Self;
				Self::atan2(self, y: Self) -> Self;
				Self::ceil(self) -> Self;
				Self::copysign(self, sign: $t) -> Self;
				Self::cos(self) -> Self;
				Self::cosh(self) -> Self;
				Self::exp(self) -> Self;
				Self::exp2(self) -> Self;
				Self::floor(self) -> Self;
				Self::fract(self) -> Self;
				Self::is_finite(self) -> bool;
				Self::is_infinite(self) -> bool;
				Self::is_nan(self) -> bool;
				Self::log(self, base: Self) -> Self;
				Self::log10(self) -> Self;
				Self::log2(self) -> Self;
				Self::mul_add(self, a: Self, b: Self) -> Self;
				Self::powf(self, exp: $t) -> Self;
				Self::powi(self, exp: i32) -> Self;
				Self::recip(self) -> Self;
				Self::round(self) -> Self;
				Self::sin_cos(self) -> (Self, Self);
				Self::sin(self) -> Self;
				Self::sinh(self) -> Self;
				Self::sqrt(self) -> Self;
				Self::tan(self) -> Self;
				Self::tanh(self) -> Self;
				Self::to_degrees(self) -> Self;
				Self::to_radians(self) -> Self;
				Self::trunc(self) -> Self;
			}
		}
	}
}

float_impl!(f32);
float_impl!(f64);
