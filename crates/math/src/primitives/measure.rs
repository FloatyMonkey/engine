use super::shapes::*;
use std::f32::consts::PI;

pub trait Measure {
	/// Get the surface area of the shape.
	fn area(&self) -> f32;
	/// Get the volume of the shape.
	fn volume(&self) -> f32;
}

impl Measure for Sphere {
	fn area(&self) -> f32 {
		4.0 * PI * self.radius * self.radius
	}

	fn volume(&self) -> f32 {
		(4.0 / 3.0) * PI * self.radius * self.radius * self.radius
	}
}

impl Measure for Cylinder {
	fn area(&self) -> f32 {
		2.0 * PI * self.radius * (self.radius + 2.0 * self.half_height)
	}

	fn volume(&self) -> f32 {
		2.0 * PI * self.radius * self.radius * self.half_height
	}
}

impl Measure for Capsule {
	fn area(&self) -> f32 {
		4.0 * PI * self.radius * (self.half_length + self.radius)
	}

	fn volume(&self) -> f32 {
		PI * self.radius * self.radius * (2.0 * self.half_length + (4.0 / 3.0) * self.radius)
	}
}

impl Measure for Cuboid {
	fn area(&self) -> f32 {
		8.0 * (self.half_size.x * self.half_size.y
			+ self.half_size.y * self.half_size.z
			+ self.half_size.x * self.half_size.z)
	}

	fn volume(&self) -> f32 {
		8.0 * self.half_size.x * self.half_size.y * self.half_size.z
	}
}

impl Measure for TriMesh<'_> {
	fn area(&self) -> f32 {
		self.indices
			.chunks_exact(3)
			.map(|indices| {
				let a = self.vertices[indices[0]];
				let b = self.vertices[indices[1]];
				let c = self.vertices[indices[2]];

				(b - a).cross(c - a).length() / 2.0
			})
			.sum()
	}

	fn volume(&self) -> f32 {
		self.indices
			.chunks_exact(3)
			.map(|indices| {
				let a = self.vertices[indices[0]];
				let b = self.vertices[indices[1]];
				let c = self.vertices[indices[2]];

				a.dot(b.cross(c)) / 6.0
			})
			.sum()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::Vec3;

	#[test]
	fn sphere() {
		let sphere = Sphere { radius: 2.0 };

		assert_eq!(sphere.area(), 50.265484, "Incorrect area");
		assert_eq!(sphere.volume(), 33.510323, "Incorrect volume");
	}

	#[test]
	fn cylinder() {
		let cylinder = Cylinder {
			radius: 2.0,
			half_height: 1.5,
		};

		assert_eq!(cylinder.area(), 62.831856, "Incorrect area");
		assert_eq!(cylinder.volume(), 37.699112, "Incorrect volume");
	}

	#[test]
	fn capsule() {
		let capsule = Capsule {
			radius: 2.0,
			half_length: 1.5,
		};

		assert_eq!(capsule.area(), 87.9646, "Incorrect area");
		assert_eq!(capsule.volume(), 71.20944, "Incorrect volume");
	}

	#[test]
	fn cuboid() {
		let cuboid = Cuboid {
			half_size: Vec3::new(1.0, 1.5, 2.0),
		};

		assert_eq!(cuboid.area(), 52.0, "Incorrect area");
		assert_eq!(cuboid.volume(), 24.0, "Incorrect volume");
	}
}
