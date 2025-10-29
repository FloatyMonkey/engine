use asset::Asset;
use math::Vec3;

#[derive(Default)]
pub struct Mesh {
	pub vertices: Vec<Vertex>,
	pub indices: Vec<usize>,
	pub vertex_groups: VertexGroups,
}

pub struct Vertex {
	pub p: Vec3,
	pub n: Vec3,
}

#[derive(Default)]
pub struct AttributeGroup<T> {
	/// Attribute names.
	pub names: Vec<String>,
	/// Offset to index into `values` for each primitive + 1.
	pub lookup: Vec<usize>,
	/// Tuples of (attribute_id, value) for all attribute values of all primitives.
	pub values: Vec<(usize, T)>,
}

pub type VertexGroups = AttributeGroup<f32>;

impl Mesh {
	pub fn new() -> Self {
		Default::default()
	}
}

#[derive(Default)]
pub struct MeshBuilder {
	pub mesh: Mesh,
}

impl MeshBuilder {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn reserve(&mut self, vertex_count: usize, _edge_count: usize, _face_count: usize) {
		self.mesh.vertices.reserve(vertex_count);
	}

	pub fn add_vertex(&mut self, coords: [f32; 3]) -> usize {
		let index = self.mesh.vertices.len();
		self.mesh.vertices.push(Vertex { p: Vec3::new(coords[0], coords[1], coords[2]), n: Vec3::ZERO });
		index
	}

	pub fn add_triangle(&mut self, v0: usize, v1: usize, v2: usize) {
		self.mesh.indices.extend_from_slice(&[
			v0, v1, v2
		]);
	}

	pub fn add_quad(&mut self, v0: usize, v1: usize, v2: usize, v3: usize) {
		self.mesh.indices.extend_from_slice(&[
			v0, v1, v2,
			v0, v2, v3,
		]);
	}

	pub fn build(self) -> Mesh {
		let mut mesh = self.mesh;
		calculate_vert_normals(&mut mesh);
		mesh
	}
}

pub fn calculate_vert_normals(mesh: &mut Mesh) {
	for vertex in &mut mesh.vertices {
		vertex.n = Vec3::ZERO;
	}

	for face in mesh.indices.chunks_exact(3) {
		let v0 = &mesh.vertices[face[0]];
		let v1 = &mesh.vertices[face[1]];
		let v2 = &mesh.vertices[face[2]];

		let e0 = v1.p - v0.p;
		let e1 = v2.p - v0.p;

		let normal = e0.cross(e1);

		for &i in face {
			mesh.vertices[i].n += normal;
		}
	}

	for vertex in &mut mesh.vertices {
		vertex.n = *vertex.n.normalize();
	}
}

impl Asset for Mesh {}
