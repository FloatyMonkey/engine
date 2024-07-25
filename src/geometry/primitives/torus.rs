use super::super::mesh::{Mesh, MeshBuilder};

use std::f32::consts::PI;

pub fn torus(tubular_segments: usize, radial_segments: usize, radius: f32, thickness: f32) -> Mesh {
	assert!(radial_segments >= 3);
	assert!(tubular_segments >= 3);

	let mut mesh = MeshBuilder::new();

	let vertex_count = radial_segments * tubular_segments;
	let edge_count = vertex_count * 2;
	let face_count = vertex_count;

	mesh.reserve(vertex_count, edge_count, face_count);

	let delta_theta = 2.0 * PI / radial_segments as f32;
	let delta_phi = 2.0 * PI / tubular_segments as f32;

	for rs in 0..radial_segments {
		let theta = rs as f32 * delta_theta;
		let sin_theta = theta.sin();
		let cos_theta = theta.cos();

		for ts in 0..tubular_segments {
			let phi = ts as f32 * delta_phi;
			let x = (radius + thickness * cos_theta) * phi.cos();
			let y = (radius + thickness * cos_theta) * phi.sin();
			let z = thickness * sin_theta;

			mesh.add_vertex([x, y, z]);
		}
	}

	for rs in 0..radial_segments {
		let rs_next = (rs + 1) % radial_segments;

		for ts in 0..tubular_segments {
			let ts_next = (ts + 1) % tubular_segments;

			let i0 = rs * tubular_segments + ts;
			let i1 = rs * tubular_segments + ts_next;
			let i2 = rs_next * tubular_segments + ts_next;
			let i3 = rs_next * tubular_segments + ts;
			mesh.add_quad(i0, i1, i2, i3);
		}
	}

	mesh.build()
}
