use super::super::mesh::{Mesh, MeshBuilder};

use std::f32::consts::PI;

/// Creates a UV-Sphere Mesh.
/// # Arguments
/// * `meridians` - Number of 'vertical' lines.
/// * `parallels` - Number of 'horizontal' lines.
pub fn sphere(radius: f32, meridians: usize, parallels: usize) -> Mesh {
	assert!(meridians >= 3);
	assert!(parallels >= 2);

	let mut mesh = MeshBuilder::new();

	let vertex_count = meridians * (parallels - 1) + 2;
	let edge_count = meridians * (parallels * 2 - 1);
	let face_count = (meridians * (parallels - 2)) + (meridians * 2);

	mesh.reserve(vertex_count, edge_count, face_count);

	let delta_theta = PI / parallels as f32;
	let delta_phi = 2.0 * PI / meridians as f32;

	let v_top = mesh.add_vertex([0.0, 0.0, radius]);

	for p in 0..parallels - 1 {
		let theta = delta_theta * (p + 1) as f32;
		let sin_theta = theta.sin();
		let cos_theta = theta.cos();

		for m in 0..meridians {
			let phi = delta_phi * m as f32;
			let x = radius * sin_theta * phi.cos();
			let y = radius * sin_theta * phi.sin();
			let z = radius * cos_theta;

			mesh.add_vertex([x, y, z]);
		}
	}

	let v_bottom = mesh.add_vertex([0.0, 0.0, -radius]);

	for m in 0..meridians {
		let i0 = m + 1;
		let i1 = (m + 1) % meridians + 1;
		mesh.add_triangle(v_top, i0, i1);
	}

	for p in 0..parallels - 2 {
		let idx0 = p * meridians + 1;
		let idx1 = (p + 1) * meridians + 1;

		for m in 0..meridians {
			let i0 = idx0 + m;
			let i1 = idx1 + m;
			let i2 = idx1 + (m + 1) % meridians;
			let i3 = idx0 + (m + 1) % meridians;
			mesh.add_quad(i0, i1, i2, i3);
		}
	}

	for m in 0..meridians {
		let i0 = m + meridians * (parallels - 2) + 1;
		let i1 = (m + 1) % meridians + meridians * (parallels - 2) + 1;
		mesh.add_triangle(v_bottom, i1, i0);
	}

	mesh.build()
}
