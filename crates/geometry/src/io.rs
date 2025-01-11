use super::mesh::{Mesh, Vertex, VertexGroups};
use math::Vec3;

use std::fs::File;
use std::io::{Cursor, Read};
use byteorder::{LittleEndian, ReadBytesExt};

fn parse_binary(data: &[u8]) -> Mesh {
	let mut cursor = Cursor::new(data);

	let vertex_count = cursor.read_u32::<LittleEndian>().unwrap() as usize;
	let index_count = cursor.read_u32::<LittleEndian>().unwrap() as usize;

	let mut vertices = Vec::with_capacity(vertex_count);
	for _ in 0..vertex_count {
		let mut p = Vec3::ZERO;
		for i in 0..3 {
			p[i] = cursor.read_f32::<LittleEndian>().unwrap();
		}

		let mut n = Vec3::ZERO;
		for i in 0..3 {
			n[i] = cursor.read_f32::<LittleEndian>().unwrap();
		}

		vertices.push(Vertex { p, n });
	}

	let mut indices = Vec::with_capacity(index_count);
	for _ in 0..index_count {
		indices.push(cursor.read_u32::<LittleEndian>().unwrap() as usize);
	}

	let vertex_group_count = cursor.read_u32::<LittleEndian>().unwrap() as usize;

	let vertex_groups = if vertex_group_count > 0 {
		let mut names = Vec::with_capacity(vertex_group_count);
		for _ in 0..vertex_group_count {
			let name_len = cursor.read_u32::<LittleEndian>().unwrap();
			let mut buffer = vec![0; name_len as usize];
			cursor.read_exact(&mut buffer).unwrap();
			let name = String::from_utf8(buffer).unwrap();
			names.push(name);
		}
		
		let mut lookup = Vec::with_capacity(vertex_count + 1);
		for _ in 0..vertex_count + 1 {
			lookup.push(
				cursor.read_u64::<LittleEndian>().unwrap() as usize
			)
		}

		let value_count = *lookup.last().unwrap();

		let mut values = Vec::with_capacity(value_count);
		for _ in 0..value_count {
			values.push((
				cursor.read_u64::<LittleEndian>().unwrap() as usize,
				cursor.read_f32::<LittleEndian>().unwrap(),
			))
		}

		VertexGroups { names, lookup, values }
	} else {
		VertexGroups::default()
	};

	Mesh {
		vertices,
		indices,
		vertex_groups,
	}
}

fn load_binary_from_file(path: &str) -> Result<Vec<u8>, std::io::Error> {
	let mut file = File::open(path)?;
	let mut data = Vec::new();
	file.read_to_end(&mut data)?;
	Ok(data)
}

pub fn load_mesh(path: &str) -> Mesh {
	let binary = load_binary_from_file(path).unwrap();
	parse_binary(&binary)
}
