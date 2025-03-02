use super::shapes::*;
use crate::{Vec2, Vec3};
use std::f32::consts::PI;
use rand::Rng;

pub trait ShapeSample {
	/// Uniformly sample a point on the boundary of this shape.
	fn sample_boundary<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec3;

	/// Uniformly sample a point in the interior of this shape.
	fn sample_interior<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec3;
}

impl ShapeSample for Sphere {
	fn sample_boundary<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec3 {
		sample_sphere_boundary(rng) * self.radius
	}

	fn sample_interior<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec3 {
		let r_cubed = rng.random_range(0.0..=(self.radius * self.radius * self.radius));
		let r = r_cubed.cbrt();
		sample_sphere_boundary(rng) * r
	}
}

impl ShapeSample for Cylinder {
	fn sample_boundary<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec3 {
		if rng.random_bool((self.radius / (self.radius + 2.0 * self.half_height)) as f64) {
			let circle = sample_circle_interior(rng) * self.radius;
			if rng.random() {
				Vec3::new(circle.x, circle.y, self.half_height)
			} else {
				Vec3::new(circle.x, circle.y, -self.half_height)
			}
		} else {
			let circle = sample_circle_boundary(rng) * self.radius;
			let z = rng.random_range(-self.half_height..=self.half_height);
			Vec3::new(circle.x, circle.y, z)
		}
	}

	fn sample_interior<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec3 {
		let xy = sample_circle_interior(rng) * self.radius;
		let z = rng.random_range(-self.half_height..=self.half_height);
		Vec3::new(xy.x, xy.y, z)
	}
}

impl ShapeSample for Capsule {
	fn sample_boundary<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec3 {
		let tube_area = 4.0 * PI * self.radius * self.half_length;
		let capsule_area = tube_area + 4.0 * PI * self.radius * self.radius;

		if rng.random_bool((tube_area / capsule_area) as f64) {
			let circle = sample_circle_interior(rng) * self.radius;
			let z = rng.random_range(-self.half_length..=self.half_length);
			Vec3::new(circle.x, circle.y, z)
		} else {
			let sphere = Sphere { radius: self.radius };
			let point = sphere.sample_boundary(rng);
			point + (Vec3::Z * self.half_length) * point.z.signum()
		}
	}

	fn sample_interior<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec3 {
		let tube_volume = 2.0 * PI * self.radius * self.radius * self.half_length;
		let capsule_volume = tube_volume + 4.0 / 3.0 * PI * self.radius * self.radius * self.radius;

		if rng.random_bool((tube_volume / capsule_volume) as f64) {
			let cylinder = Cylinder { radius: self.radius, half_height: self.half_length };
			cylinder.sample_interior(rng)
		} else {
			let sphere = Sphere { radius: self.radius };
			let point = sphere.sample_interior(rng);
			point + (Vec3::Z * self.half_length) * point.z.signum()
		}
	}
}

impl ShapeSample for Cuboid {
	fn sample_boundary<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec3 {
		let u = rng.random_range(-1.0..1.0);
		let v = rng.random_range(-1.0..1.0);
		let w = if rng.random() { -1.0 } else { 1.0 };

		// These are not the actual areas, because they use the half sizes.
		// This is fine, since we only need ratios for the probabilities.
		let area_xy = self.half_size.x * self.half_size.y;
		let area_yz = self.half_size.y * self.half_size.z;
		let area_xz = self.half_size.x * self.half_size.z;

		let area = area_xy + area_yz + area_xz;
		let p_xy = area_xy / area;
		let p_yz = area_yz / area;

		let r = rng.random_range(0.0..1.0);

		if r < p_xy {
			Vec3::new(u, v, w).cmul(self.half_size)
		} else if r < p_xy + p_yz {
			Vec3::new(w, u, v).cmul(self.half_size)
		} else {
			Vec3::new(u, w, v).cmul(self.half_size)
		}
	}

	fn sample_interior<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec3 {
		let x = rng.random_range(-self.half_size.x..self.half_size.x);
		let y = rng.random_range(-self.half_size.y..self.half_size.y);
		let z = rng.random_range(-self.half_size.z..self.half_size.z);

		Vec3::new(x, y, z)
	}
}

fn sample_circle_boundary<R: Rng + ?Sized>(rng: &mut R) -> Vec2 {
	let theta = rng.random_range(0.0..2.0 * PI);
	let (sin, cos) = theta.sin_cos();
	Vec2::new(cos, sin)
}

fn sample_circle_interior<R: Rng + ?Sized>(rng: &mut R) -> Vec2 {
	let theta = rng.random_range(0.0..2.0 * PI);
	let r = (rng.random_range(0.0..=1.0) as f32).sqrt();
	let (sin, cos) = theta.sin_cos();
	Vec2::new(r * cos, r * sin)
}

fn sample_sphere_boundary<R: Rng + ?Sized>(rng: &mut R) -> Vec3 {
	let z = rng.random_range(-1f32..=1f32);
	let (a_sin, a_cos) = rng.random_range(-PI..=PI).sin_cos();
	let c = (1f32 - z * z).sqrt();
	let x = a_sin * c;
	let y = a_cos * c;

	Vec3::new(x, y, z)
}
