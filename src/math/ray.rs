use super::{Unit, Vec3};

/// An infinite half-line starting at `origin` and going in `direction`.
pub struct Ray {
	pub origin: Vec3,
	pub direction: Unit<Vec3>,
}

impl Ray {
	/// Create a new `Ray` from a given origin and direction.
	pub fn new(origin: Vec3, direction: Unit<Vec3>) -> Self {
		Self { origin, direction }
	}

	/// Get a point at a given distance along the ray.
	pub fn at(&self, distance: f32) -> Vec3 {
		self.origin + self.direction * distance
	}
}
