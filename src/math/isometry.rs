use std::ops::Mul;
use super::matrix::{Vector2, Vector3};
use super::num::Number;
use super::{UnitComplex, UnitQuaternion};

pub struct Isometry<T, R> {
	pub translation: T,
	pub rotation: R,
}

/// A 2-dimensional direct isometry using a [`UnitComplex`] number for its rotational part.
pub type Isometry2<T = f32> = Isometry<Vector2<T>, UnitComplex<T>>;

/// A 3-dimensional direct isometry using a [`UnitQuaternion`] for its rotational part.
pub type Isometry3<T = f32> = Isometry<Vector3<T>, UnitQuaternion<T>>;

impl<T: Number> Isometry3<T> {
	pub const IDENTITY: Self = Self {
		translation: Vector3::ZERO,
		rotation: UnitQuaternion::IDENTITY,
	};

	pub const fn from_translation(translation: Vector3<T>) -> Self {
		Self { translation, ..Self::IDENTITY }
	}

	pub const fn from_rotation(rotation: UnitQuaternion<T>) -> Self {
		Self { rotation, ..Self::IDENTITY }
	}

	pub const fn with_translation(self, translation: Vector3<T>) -> Self {
		Self { translation, ..self }
	}

	pub const fn with_rotation(self, rotation: UnitQuaternion<T>) -> Self {
		Self { rotation, ..self }
	}
}

impl Isometry3 {
	pub fn inv(&self) -> Self {
		let rotation = self.rotation.inv();
		let translation = rotation * -self.translation;

		Self { translation, rotation }
	}

	/// Translates and rotates a point by this isometry.
	pub fn transform_point(&self, point: Vector3<f32>) -> Vector3<f32> {
		self.rotation * point + self.translation
	}

	/// Translates and rotates a point by the inverse of this isometry.
	/// Shorthand for `inv()` followed by `transform_point()`.
	pub fn inv_transform_point(&self, point: Vector3<f32>) -> Vector3<f32> {
		self.rotation.inv() * (point - self.translation)
	}

	/// Rotates a vector by this isometry.
	pub fn transform_vector(&self, vector: Vector3<f32>) -> Vector3<f32> {
		self.rotation * vector
	}

	/// Rotates a vector by the inverse of this isometry.
	/// Shorthand for `inv()` followed by `transform_vector()`.
	pub fn inv_transform_vector(&self, vector: Vector3<f32>) -> Vector3<f32> {
		self.rotation.inv() * vector
	}
}

impl Mul for Isometry3 {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		let translation = self.rotation * rhs.translation + self.translation;
		let rotation = self.rotation * rhs.rotation;

		Self { translation, rotation }
	}
}
