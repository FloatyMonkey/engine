use super::super::mesh::{Mesh, MeshBuilder};

pub fn tetrahedron() -> Mesh {
	let mut mesh = MeshBuilder::new();
	mesh.reserve(4, 6, 4);

	// Coordinates on the unit sphere
	let a = 1.0 / 3.0;
	let b = (8.0_f32 / 9.0).sqrt();
	let c = (2.0_f32 / 9.0).sqrt();
	let d = (2.0_f32 / 3.0).sqrt();

	let v0 = mesh.add_vertex([0.0, 0.0, 1.0]);
	let v1 = mesh.add_vertex([-c, d, -a]);
	let v2 = mesh.add_vertex([-c, -d, -a]);
	let v3 = mesh.add_vertex([b, 0.0, -a]);

	mesh.add_triangle(v0, v1, v2);
	mesh.add_triangle(v0, v2, v3);
	mesh.add_triangle(v0, v3, v1);
	mesh.add_triangle(v3, v2, v1);

	mesh.build()
}

pub fn hexahedron() -> Mesh {
	let mut mesh = MeshBuilder::new();
	mesh.reserve(8, 12, 6);

	// Coordinates on the unit sphere
	let a = 1.0 / 3.0_f32.sqrt();

	let v0 = mesh.add_vertex([-a, -a, -a]);
	let v1 = mesh.add_vertex([a, -a, -a]);
	let v2 = mesh.add_vertex([a, a, -a]);
	let v3 = mesh.add_vertex([-a, a, -a]);
	let v4 = mesh.add_vertex([-a, -a, a]);
	let v5 = mesh.add_vertex([a, -a, a]);
	let v6 = mesh.add_vertex([a, a, a]);
	let v7 = mesh.add_vertex([-a, a, a]);

	mesh.add_quad(v3, v2, v1, v0);
	mesh.add_quad(v2, v6, v5, v1);
	mesh.add_quad(v5, v6, v7, v4);
	mesh.add_quad(v0, v4, v7, v3);
	mesh.add_quad(v3, v7, v6, v2);
	mesh.add_quad(v1, v5, v4, v0);

	mesh.build()
}

pub fn octahedron() -> Mesh {
	let mut mesh = MeshBuilder::new();
	mesh.reserve(6, 12, 8);

	let v0 = mesh.add_vertex([0.0, 1.0, 0.0]);
	let v1 = mesh.add_vertex([1.0, 0.0, 0.0]);
	let v2 = mesh.add_vertex([0.0, -1.0, 0.0]);
	let v3 = mesh.add_vertex([-1.0, 0.0, 0.0]);
	let v4 = mesh.add_vertex([0.0, 0.0, 1.0]);
	let v5 = mesh.add_vertex([0.0, 0.0, -1.0]);

	mesh.add_triangle(v1, v0, v4);
	mesh.add_triangle(v0, v3, v4);
	mesh.add_triangle(v3, v2, v4);
	mesh.add_triangle(v2, v1, v4);
	mesh.add_triangle(v1, v5, v0);
	mesh.add_triangle(v0, v5, v3);
	mesh.add_triangle(v3, v5, v2);
	mesh.add_triangle(v2, v5, v1);

	mesh.build()
}

pub fn icosahedron() -> Mesh {
	let mut mesh = MeshBuilder::new();
	mesh.reserve(12, 30, 20);

	// Coordinates on the unit sphere
	let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;
	let scale = (1.0 + phi * phi).sqrt();

	let a = 1.0 / scale;
	let b = phi / scale;

	let vertices = [
		[-a, 0.0, b],
		[a, 0.0, b],
		[-a, 0.0, -b],
		[a, 0.0, -b],
		[0.0, b, a],
		[0.0, b, -a],
		[0.0, -b, a],
		[0.0, -b, -a],
		[b, a, 0.0],
		[-b, a, 0.0],
		[b, -a, 0.0],
		[-b, -a, 0.0],
	];

	let faces = [
		[0, 1, 4],
		[0, 4, 9],
		[9, 4, 5],
		[4, 8, 5],
		[4, 1, 8],
		[8, 1, 10],
		[8, 10, 3],
		[5, 8, 3],
		[5, 3, 2],
		[2, 3, 7],
		[7, 3, 10],
		[7, 10, 6],
		[7, 6, 11],
		[11, 6, 0],
		[0, 6, 1],
		[6, 10, 1],
		[9, 11, 0],
		[9, 2, 11],
		[9, 5, 2],
		[7, 11, 2],
	];

	vertices.iter().for_each(|v| {
		mesh.add_vertex(*v);
	});

	faces.iter().for_each(|f| {
		mesh.add_triangle(f[0], f[1], f[2]);
	});

	mesh.build()
}
