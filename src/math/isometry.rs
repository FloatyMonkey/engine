use std::ops::Mul;
use super::matrix::{Vec3, Vector2, Vector3};
use super::{UnitComplex, UnitQuaternion};

pub struct Isometry<T, R> {
	pub translation: T,
	pub rotation: R,
}

/// A 2-dimensional direct isometry using a [`UnitComplex`] number for its rotational part.
pub type Isometry2<T = f32> = Isometry<Vector2<T>, UnitComplex<T>>;

/// A 3-dimensional direct isometry using a [`UnitQuaternion`] for its rotational part.
pub type Isometry3<T = f32> = Isometry<Vector3<T>, UnitQuaternion<T>>;

impl Isometry3 {
	pub fn identity() -> Self {
		Self {
			translation: Vec3::ZERO,
			rotation: UnitQuaternion::identity(),
		}
	}

	pub fn inv(&self) -> Self {
		let rotation = self.rotation.inv();
		let translation = rotation * -self.translation;

		Self { translation, rotation }
	}

	pub fn transform(&self, point: Vec3) -> Vec3 {
		self.rotation * point + self.translation
	}
}

impl Mul for Isometry3 {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		let rotation = self.rotation * rhs.rotation;
		let translation = self.rotation * rhs.translation + self.translation;

		Self { translation, rotation }
	}
}
