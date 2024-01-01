use std::ops::Mul;
use super::matrix::{Vec3, Mat4, Vector2, Vector3};
use super::num::Number;
use super::{UnitComplex, UnitQuaternion};

#[derive(Clone, Copy, PartialEq)]
pub struct Transform<T, R, S> {
	pub translation: T,
	pub rotation: R,
	pub scale: S,
}

pub type Transform2<T = f32> = Transform<Vector2<T>, UnitComplex<T>, T>;
pub type Transform3<T = f32> = Transform<Vector3<T>, UnitQuaternion<T>, Vector3<T>>;

impl<T: Number> Transform3<T> {
	pub const fn identity() -> Self {
		Self {
			translation: Vector3::ZERO,
			rotation: UnitQuaternion::identity(),
			scale: Vector3::ONE,
		}
	}

	pub const fn from_translation(translation: Vector3<T>) -> Self {
		Self { translation, ..Self::identity() }
	}

	pub const fn from_rotation(rotation: UnitQuaternion<T>) -> Self {
		Self { rotation, ..Self::identity() }
	}

	pub const fn from_scale(scale: Vector3<T>) -> Self {
		Self { scale, ..Self::identity() }
	}

	pub const fn with_translation(self, translation: Vector3<T>) -> Self {
		Self { translation, ..self }
	}

	pub const fn with_rotation(self, rotation: UnitQuaternion<T>) -> Self {
		Self { rotation, ..self }
	}

	pub const fn with_scale(self, scale: Vector3<T>) -> Self {
		Self { scale, ..self }
	}
}

impl Transform3<f32> {
	pub fn inv(&self) -> Self {
		let scale = Vec3::new(1.0 / self.scale.x, 1.0 / self.scale.y, 1.0 / self.scale.z);
		let rotation = self.rotation.inv();
		let translation = rotation * -self.translation.cmul(scale);

		Self { translation, rotation, scale }
	}
}

impl Mul for Transform3 {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		let scale = self.scale.cmul(rhs.scale);
		let rotation = self.rotation * rhs.rotation;
		let translation = self.rotation * self.scale.cmul(rhs.translation) + self.translation;

		Self { translation, rotation, scale }
	}
}

impl From<Transform3> for Mat4 {
	fn from(transform: Transform3) -> Self {
		let translation = transform.translation;
		let rotation = transform.rotation.to_matrix3();
		let scale = transform.scale;

		Mat4::from_array([
			rotation[0] * scale.x, rotation[1] * scale.y, rotation[2] * scale.z, translation.x,
			rotation[3] * scale.x, rotation[4] * scale.y, rotation[5] * scale.z, translation.y,
			rotation[6] * scale.x, rotation[7] * scale.y, rotation[8] * scale.z, translation.z,
			0.0, 0.0, 0.0, 1.0,
		])
	}
}
