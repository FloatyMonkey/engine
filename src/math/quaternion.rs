use std::ops::{Add, Sub, Mul, Div, Neg};

use super::{Dual, Unit};
use super::num::{Float, FloatOps, Number};

use super::matrix::{Vector3, Matrix3};
use super::unwind_radians;

/// A quaternion. See [`UnitQuaternion`] for a quaternion that may be used to represent a rotation.
#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub struct Quaternion<T> {
	pub i: T,
	pub j: T,
	pub k: T,
	pub w: T,
}

impl<T: Number> Quaternion<T> {
	pub const ZERO: Self = Self {
		i: T::ZERO, j: T::ZERO, k: T::ZERO, w: T::ZERO
	};

	/// A quaternions multiplicative identity.
	pub const IDENTITY: Self = Self {
		i: T::ZERO, j: T::ZERO, k: T::ZERO, w: T::ONE
	};

	/// Constructs a real quaternion.
	pub const fn from_real(real: T) -> Self {
		Self { i: T::ZERO, j: T::ZERO, k: T::ZERO, w: real }
	}

	/// Constructs a pure quaternion.
	pub fn from_imag(imag: Vector3<T>) -> Self {
		Self { i: imag.x, j: imag.y, k: imag.z, w: T::ZERO }
	}

	/// Constructs a quaternion from its real/scalar and imaginary/vector parts.
	pub fn from_parts(real: T, imag: Vector3<T>) -> Self {
		Self { i: imag.x, j: imag.y, k: imag.z, w: real }
	}
}

impl<T: Number> Quaternion<T> {
	/// The real/scalar part `w` of this quaternion.
	pub fn real(&self) -> T {
		self.w
	}

	/// The imaginary/vector part `(i, j, k)` of this quaternion.
	pub fn imag(&self) -> Vector3<T> {
		Vector3::new(self.i, self.j, self.k)
	}
}

impl<T: Float + FloatOps<T>> Quaternion<T> {
	/// Returns the dot product of `self` and `rhs`.
	/// This is equal to the cosine of the angle between the two quaternions, multiplied by their lengths.
	pub fn dot(&self, rhs: Self) -> T {
		self.i * rhs.i + self.j * rhs.j + self.k * rhs.k + self.w * rhs.w
	}

	/// Returns the squared length (L2 norm) of this quaternion.
	pub fn length_sq(&self) -> T {
		self.dot(*self)
	}

	/// Returns the length (L2 norm) of this quaternion.
	pub fn length(&self) -> T {
		self.length_sq().sqrt()
	}

	/// Returns the conjugate of this quaternion.
	pub fn conj(&self) -> Self {
		Self::from_parts(self.w, -self.imag())
	}

	/// Returns the inverse of this quaternion.
	pub fn inv(&self) -> Option<Self> {
		let length_sq = self.length_sq();
		(length_sq > T::ZERO).then(|| self.conj() / length_sq)
	}

	/// Normalizes this quaternion.
	pub fn normalize(&self) -> Unit<Self> {
		Unit::new_unchecked(*self / self.length())
	}

	pub fn log(&self) -> Self {
		let length = self.imag().length();

		if length < T::SMALL_EPSILON {
			Self::from_imag(self.imag())
		} else {
			let half_angle = length.atan2(self.real());
			let axis = self.imag() / length;

			Self::from_imag(axis * half_angle)
		}
	}

	pub fn exp(&self) -> Self {
		let half_angle = self.imag().length();

		if half_angle < T::SMALL_EPSILON {
			Self::from_imag(self.imag())
		} else {
			let (sin, cos) = half_angle.sin_cos();
			let axis = self.imag() / half_angle;

			Self::from_parts(cos, axis * sin)
		}
	}
}

/// A unit quaternion. May be used to represent a 3D rotation.
pub type UnitQuaternion<T> = Unit<Quaternion<T>>;

impl<T: Number> UnitQuaternion<T> {
	pub const IDENTITY: Self = Self::new_unchecked(Quaternion::IDENTITY);
}

impl<T: Float + FloatOps<T>> UnitQuaternion<T> {
	pub fn conj(&self) -> Self {
		Self::new_unchecked(self.as_ref().conj())
	}

	pub fn inv(&self) -> Self {
		self.conj()
	}

	/// Returns the shortest equivalent of the rotation.
	/// Ensures the quaternion is on the hemisphere closest to the identity quaternion.
	pub fn abs(&self) -> Self {
		if self.w < T::ZERO {
			-*self
		} else {
			*self
		}
	}

	/// Creates a new unit quaternion from a vector.
	/// The vectors direction represents the rotation axis and its length the rotation angle.
	pub fn from_scaled_axis(scaled_axis: Vector3<T>) -> Self {
		let q = Quaternion::from_imag(scaled_axis / T::TWO);
		Self::new_unchecked(q.exp())
	}

	/// Returns a vector where the direction represents the rotation axis and its length the rotation angle.
	pub fn scaled_axis(&self) -> Vector3<T> {
		let log = self.as_ref().log();
		log.imag() * T::TWO
	}

	/// Creates a new unit quaternion from a unit vector (rotation axis) and an angle (rotation angle).
	pub fn from_axis_angle(axis: Unit<Vector3<T>>, angle: T) -> Self {
		let (sin, cos) = (angle / T::TWO).sin_cos();
		Self::new_unchecked(Quaternion::from_parts(cos, *axis * sin))
	}

	/// Creates a new unit quaternion from Euler angles.
	pub fn from_euler(pitch: T, roll: T, yaw: T) -> Self {
		let (sx, cx) = (pitch / T::TWO).sin_cos();
		let (sy, cy) = (roll / T::TWO).sin_cos();
		let (sz, cz) = (yaw / T::TWO).sin_cos();

		Self::new_unchecked(Quaternion {
			i: sx * cy * cz - cx * sy * sz,
			j: cx * sy * cz + sx * cy * sz,
			k: cx * cy * sz - sx * sy * cz,
			w: cx * cy * cz + sx * sy * sz,
		})
	}

	/// Converts this unit quaternion to a 3x3 rotation matrix.
	pub fn to_matrix3(&self) -> Matrix3<T> {
		let x2 = self.i + self.i;
		let y2 = self.j + self.j;
		let z2 = self.k + self.k;
		let x2w = x2 * self.w;
		let y2w = y2 * self.w;
		let z2w = z2 * self.w;
		let x2x = x2 * self.i;
		let y2x = y2 * self.i;
		let z2x = z2 * self.i;
		let y2y = y2 * self.j;
		let z2y = z2 * self.j;
		let z2z = z2 * self.k;

		Matrix3::from_array([
			T::ONE - (y2y + z2z), y2x - z2w, z2x + y2w,
			y2x + z2w, T::ONE - (x2x + z2z), z2y - x2w,
			z2x - y2w, z2y + x2w, T::ONE - (x2x + y2y),
		])
	}

	/// The euler angles, returned in the form (roll, pitch, yaw).
	pub fn euler(&self) -> (T, T, T) {
		(
			T::atan2(T::TWO * (self.j * self.k + self.w * self.i), self.w * self.w - self.i * self.i - self.j * self.j + self.k * self.k),
			T::asin(-T::TWO * (self.i * self.k - self.w * self.j)),
			T::atan2(T::TWO * (self.i * self.j + self.w * self.k), self.w * self.w + self.i * self.i - self.j * self.j - self.k * self.k),
		)
	}

	/// The rotation angle in [0, pi].
	pub fn angle(&self) -> T {
		T::TWO * self.w.acos()
	}

	/// The rotation axis or `None` if the rotation is zero.
	pub fn axis(&self) -> Option<Unit<Vector3<T>>> {
		let length_sq = T::ONE - self.w * self.w;

		if length_sq <= T::ZERO {
			return None;
		}

		Some(Unit::new_unchecked(self.imag() / length_sq.sqrt()))
	}

	/// The rotation axis and angle in (0, pi] or `None` if the rotation is zero.
	pub fn axis_angle(&self) -> Option<(Unit<Vector3<T>>, T)> {
		self.axis().map(|axis| (axis, self.angle()))
	}

	/// The rotation angle around `twist_axis`.
	pub fn twist_angle(&self, twist_axis: Unit<Vector3<T>>) -> T {
		unwind_radians(T::TWO * self.imag().dot(*twist_axis).atan2(self.real()))
	}

	/// Decomposes this quaternion such that `q = swing * twist` where:
	/// * swing = rotation around axis perpendicular to `twist_axis`
	/// * twist = rotation around `twist_axis`
	pub fn swing_twist(&self, twist_axis: Unit<Vector3<T>>) -> (Self, Self) {
		let projection = self.imag().project_onto(twist_axis);

		let twist = Quaternion::from_parts(self.real(), projection);
		
		let twist = if twist.length_sq() < T::SMALL_EPSILON {
			Self::IDENTITY
		} else {
			twist.normalize()
		};

		let swing = *self * twist.inv();

		(swing, twist)
	}

	/// Returns the shortest rotation for transforming `from` to `to`.
	pub fn between(from: Unit<Vector3<T>>, to: Unit<Vector3<T>>) -> Self {
		let cross = from.cross(*to);
		let dot = from.dot(*to);

		Quaternion::from_parts(T::ONE + dot, cross).normalize()
	}

	/// Performs a linear interpolation between `self` and `rhs` along the shortest path.
	pub fn nlerp(&self, rhs: Self, t: T) -> Self {
		let dot = self.dot(*rhs);
		let sign = if dot < T::ZERO { -T::ONE } else { T::ONE };
		(**self + (rhs * sign - **self) * t).normalize()
	}
}

/// A dual quaternion. May be used to represent a 3D isometry.
pub type DualQuaternion<T> = Dual<Quaternion<T>>;

impl<T: Number> DualQuaternion<T> {
	pub const IDENTITY: Self = Self::new(Quaternion::IDENTITY, Quaternion::ZERO);
}

impl<T: Neg<Output=T>> Neg for Quaternion<T> {
	type Output = Quaternion<T>;

	fn neg(self) -> Self::Output {
		Self {
			i: -self.i,
			j: -self.j,
			k: -self.k,
			w: -self.w,
		}
	}
}

impl<T: Add<Output=T>> Add for Quaternion<T> {
	type Output = Quaternion<T>;

	fn add(self, rhs: Self) -> Self::Output {
		Self {
			i: self.i + rhs.i,
			j: self.j + rhs.j,
			k: self.k + rhs.k,
			w: self.w + rhs.w,
		}
	}
}

impl<T: Sub<Output=T>> Sub for Quaternion<T> {
	type Output = Quaternion<T>;

	fn sub(self, rhs: Self) -> Self::Output {
		Self {
			i: self.i - rhs.i,
			j: self.j - rhs.j,
			k: self.k - rhs.k,
			w: self.w - rhs.w,
		}
	}
}

impl<T: Mul<Output=T> + Copy> Mul<T> for Quaternion<T> {
	type Output = Quaternion<T>;

	fn mul(self, rhs: T) -> Self::Output {
		Self {
			i: self.i * rhs,
			j: self.j * rhs,
			k: self.k * rhs,
			w: self.w * rhs,
		}
	}
}

impl<T: Div<Output=T> + Copy> Div<T> for Quaternion<T> {
	type Output = Quaternion<T>;

	fn div(self, rhs: T) -> Self::Output {
		Self {
			i: self.i / rhs,
			j: self.j / rhs,
			k: self.k / rhs,
			w: self.w / rhs,
		}
	}
}

impl<T: Float> Mul<Quaternion<T>> for Quaternion<T> {
	type Output = Quaternion<T>;

	fn mul(self, rhs: Self) -> Self::Output {
		Self {
			i: self.w * rhs.i + self.i * rhs.w + self.j * rhs.k - self.k * rhs.j,
			j: self.w * rhs.j + self.j * rhs.w + self.k * rhs.i - self.i * rhs.k,
			k: self.w * rhs.k + self.k * rhs.w + self.i * rhs.j - self.j * rhs.i,
			w: self.w * rhs.w - self.i * rhs.i - self.j * rhs.j - self.k * rhs.k,
		}
	}
}

impl<T: Float> Mul<UnitQuaternion<T>> for UnitQuaternion<T> {
	type Output = UnitQuaternion<T>;

	fn mul(self, rhs: Self) -> Self::Output {
		UnitQuaternion::new_unchecked(self * *rhs)
	}
}

impl<T: Float + FloatOps<T>> Mul<Vector3<T>> for UnitQuaternion<T> {
	type Output = Vector3<T>;

	fn mul(self, rhs: Vector3<T>) -> Self::Output {
		let t = self.imag().cross(rhs) * T::TWO;
		rhs + t * self.real() + self.imag().cross(t)
	}
}

impl<T: Float + FloatOps<T>> Mul<Unit<Vector3<T>>> for UnitQuaternion<T> {
	type Output = Unit<Vector3<T>>;

	fn mul(self, rhs: Unit<Vector3<T>>) -> Self::Output {
		Unit::new_unchecked(self * *rhs)
	}
}
