use super::super::mesh::{Mesh, MeshBuilder};

pub fn grid(size_x: f32, size_y: f32, vertices_x: usize, vertices_y: usize) -> Mesh {
	assert!(vertices_x >= 2);
	assert!(vertices_y >= 2);

	let mut mesh = MeshBuilder::new();

	let faces_x = vertices_x - 1;
	let faces_y = vertices_y - 1;

	let vertex_count = vertices_x * vertices_y;
	let edge_count = faces_x * vertices_y + faces_y * vertices_x;
	let face_count = faces_x * faces_y;

	mesh.reserve(vertex_count, edge_count, face_count);

	let delta_x = size_x / faces_x as f32;
	let delta_y = size_y / faces_y as f32;
	let shift_x = size_x / 2.0;
	let shift_y = size_y / 2.0;

	for y in 0..vertices_y {
		for x in 0..vertices_x {
			mesh.add_vertex([
				x as f32 * delta_x - shift_x,
				y as f32 * delta_y - shift_y,
				0.0,
			]);
		}
	}

	for y in 0..faces_y {
		for x in 0..faces_x {
			let i0 = x + y * vertices_x;
			let i1 = i0 + 1;
			let i2 = i1 + vertices_x;
			let i3 = i0 + vertices_x;
			mesh.add_quad(i0, i1, i2, i3);
		}
	}

	mesh.build()
}
