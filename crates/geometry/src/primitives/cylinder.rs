use super::super::mesh::{Mesh, MeshBuilder};

use std::f32::consts::PI;

/// Creates a cylinder Mesh.
/// # Arguments
/// * `resolution` - Number of vertices on the top and bottom circles.
/// * `segments` - Number of segments along the height of the cylinder.
pub fn cylinder(radius: f32, height: f32, resolution: usize, segments: usize, caps: bool) -> Mesh {
	assert!(resolution >= 3);
	assert!(segments >= 1);

	let mut mesh = MeshBuilder::new();

	let vertex_count = resolution * (segments + 1) + if caps { 2 } else { 0 };
	let edge_count =
		resolution * (segments + 1) + resolution * segments + if caps { 2 * resolution } else { 0 };
	let face_count = resolution * segments + if caps { 2 * resolution } else { 0 };

	mesh.reserve(vertex_count, edge_count, face_count);

	let delta_phi = 2.0 * PI / resolution as f32;
	let delta_z = height / segments as f32;
	let shift_z = height / 2.0;

	for s in 0..=segments {
		let z = shift_z - s as f32 * delta_z;

		for r in 0..resolution {
			let phi = r as f32 * delta_phi;
			let x = radius * phi.cos();
			let y = radius * phi.sin();

			mesh.add_vertex([x, y, z]);
		}
	}

	for s in 0..segments {
		let idx0 = s * resolution;
		let idx1 = (s + 1) * resolution;

		for r in 0..resolution {
			let i0 = idx0 + r;
			let i1 = idx1 + r;
			let i2 = idx1 + (r + 1) % resolution;
			let i3 = idx0 + (r + 1) % resolution;
			mesh.add_quad(i0, i1, i2, i3);
		}
	}

	if caps {
		// TODO: Ensure that the normals are flat for the caps.
		let v_top = mesh.add_vertex([0.0, 0.0, shift_z]);
		let v_bottom = mesh.add_vertex([0.0, 0.0, -shift_z]);

		for r in 0..resolution {
			let i0 = r;
			let i1 = (r + 1) % resolution;
			mesh.add_triangle(v_top, i0, i1);
		}

		for r in 0..resolution {
			let i0 = r + resolution * segments;
			let i1 = (r + 1) % resolution + resolution * segments;
			mesh.add_triangle(v_bottom, i1, i0);
		}
	}

	mesh.build()
}
