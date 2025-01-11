use crate::Vec3;

pub struct Sphere {
	pub radius: f32,
}

pub struct Cylinder {
	pub radius: f32,
	pub half_height: f32,
}

pub struct Capsule {
	/// Radius of the capsule.
	pub radius: f32,
	/// Height of the the cylinder part, exluding the hemispheres.
	pub half_length: f32,
}

pub struct Cuboid {
	pub half_size: Vec3,
}

pub struct TriMesh<'a> {
	pub vertices: &'a [Vec3],
	pub indices: &'a [usize],
}
