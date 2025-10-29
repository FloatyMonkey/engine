#![allow(dead_code)]

pub mod isometry;
pub mod matrix;
pub mod num;
pub mod primitives;
pub mod transform;

mod complex;
mod dual;
mod quaternion;
mod unit;

use num::{Number, NumberOps, cast};

pub use complex::{Complex, UnitComplex};
pub use dual::Dual;
pub use matrix::{Matrix, Matrix2, Matrix3, Matrix4, Vector, Vector2, Vector3, Vector4};
pub use quaternion::{Quaternion, UnitQuaternion};
pub use unit::Unit;

pub type Vec2 = Vector2<f32>;
pub type Vec3 = Vector3<f32>;

pub type Mat3 = Matrix3<f32>;
pub type Mat4 = Matrix4<f32>;
pub type Mat3x4 = Matrix<f32, 3, 4>;

pub const E: f32 = std::f32::consts::E;
pub const PI: f32 = std::f32::consts::PI;

/// Clamps x to be in the range [min, max].
pub fn clamp<T: Number + NumberOps<T>>(x: T, min: T, max: T) -> T {
	T::max(min, T::min(max, x))
}

/// Wraps x to be in the range [min, max].
pub fn wrap<T: Number>(mut x: T, min: T, max: T) -> T {
	let range = max - min;

	while x < min {
		x += range;
	}
	while x > max {
		x -= range;
	}

	x
}

/// Unwinds an angle in radians to the range [-pi, pi].
pub fn unwind_radians<T: Number>(radians: T) -> T {
	wrap(radians, cast(-PI), cast(PI))
}

/// Unwinds an angle in degrees to the range [-180, 180].
pub fn unwind_degrees<T: Number>(degrees: T) -> T {
	wrap(degrees, cast(-180.0), cast(180.0))
}

/// Remaps a value from one range to another.
/// The minimum of either range may be larger or smaller than the maximum.
pub fn map_range<T: Number>(x: T, min: T, max: T, new_min: T, new_max: T) -> T {
	(x - min) * (new_max - new_min) / (max - min) + new_min
}
