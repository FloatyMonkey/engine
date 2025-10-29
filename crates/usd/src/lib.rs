use asset::AssetServer;
use ecs::{Name, World};
use geometry::mesh::{Mesh, Vertex, VertexGroups};
use graphics::scene::{DomeLight, Image, RectLight, Renderable, SphereLight};
use math::{Quaternion, Unit, UnitQuaternion, Vec3, transform::Transform3};

use openusd_rs::{gf, sdf, usd, usd_geom, usd_lux};

fn convert_mesh(mesh: &usd_geom::Mesh) -> Mesh {
	let points = mesh.points_attr().get::<Vec<gf::Vec3f>>();
	let normals = mesh.normals_attr().get::<Vec<gf::Vec3f>>();

	let vertices = points
		.iter()
		.zip(normals.iter())
		.map(|(p, n)| Vertex {
			p: Vec3::new(p.x, p.y, p.z),
			n: Vec3::new(n.x, n.y, n.z),
		})
		.collect();

	let triangles = usd_geom::triangulate(mesh);

	let indices = triangles.iter().map(|i| *i as usize).collect();

	let mut mesh = Mesh {
		vertices,
		indices,
		vertex_groups: VertexGroups::default(),
	};

	// TODO: Use normals from USD mesh.
	geometry::mesh::calculate_vert_normals(&mut mesh);

	mesh
}

fn traverse_recurse(
	stage_path: &str,
	stage: &usd::Stage,
	world: &mut World,
	transform_stack: &mut Vec<Transform3>,
	assets: &mut AssetServer,
	prim: &usd::Prim,
) {
	let xform = usd_geom::XformOp::get_local_transform(prim);

	if let Some(xform) = xform {
		transform_stack.push(from_usd_transform3d(xform));
	}

	let get_transform = |stack: &mut Vec<Transform3>| {
		stack
			.iter()
			.fold(Transform3::<f32>::IDENTITY, |acc, xform| acc * *xform)
	};

	match prim.type_name().as_str() {
		"Mesh" => {
			let mesh = usd_geom::Mesh::define(stage, prim.path().clone());
			let mesh = convert_mesh(&mesh);
			let mesh = assets.insert(mesh);

			world.spawn((
				Name::new(prim.path()),
				get_transform(transform_stack),
				Renderable { mesh },
			));
		}
		"SphereLight" => {
			let light = usd_lux::SphereLight::define(stage, prim.path().clone());

			let color = from_usd_vec3f(light.color_attr().get::<gf::Vec3f>());
			let intensity = light.intensity_attr().get::<f32>();

			world.spawn((
				Name::new(prim.path()),
				get_transform(transform_stack),
				SphereLight {
					emission: (color * intensity).into(),
					radius: light.radius_attr().get::<f32>(),
				},
			));
		}
		"RectLight" => {
			let light = usd_lux::RectLight::define(stage, prim.path().clone());

			let color = from_usd_vec3f(light.color_attr().get::<gf::Vec3f>());
			let intensity = light.intensity_attr().get::<f32>();

			world.spawn((
				Name::new(prim.path()),
				get_transform(transform_stack),
				RectLight {
					emission: (color * intensity).into(),
					width: light.width_attr().get::<f32>(),
					height: light.height_attr().get::<f32>(),
				},
			));
		}
		"DomeLight" => {
			let light = usd_lux::DomeLight::define(stage, prim.path().clone());

			// TODO: Handle at least intensity (and maybe color?)
			//let color = from_usd_vec3f(light.color_attr().get::<gf::Vec3f>());
			//let intensity = light.intensity_attr().get::<f32>();

			let texture_file_ref = light.texture_file_attr().get::<sdf::AssetPath>();

			let root_path = std::path::Path::new(stage_path);
			let parent_path = root_path.parent().unwrap_or(root_path);
			let texuture_path = parent_path.join(texture_file_ref.authored_path.clone()); // TODO: .asset_path()

			let texture = Image::from_file(texuture_path);
			let texture_asset = assets.insert(texture);

			world.spawn((
				Name::new(prim.path()),
				get_transform(transform_stack),
				DomeLight {
					image: texture_asset,
				},
			));
		}
		_ => {}
	}

	for child in prim.children() {
		traverse_recurse(stage_path, stage, world, transform_stack, assets, &child);
	}

	if xform.is_some() {
		transform_stack.pop();
	}
}

pub fn populate_world_from_usd(filepath: &str, world: &mut World, assets: &mut AssetServer) {
	let stage = usd::Stage::open(filepath);

	let pseudo_root = stage.pseudo_root();

	let mut transform_stack: Vec<Transform3> = Vec::new();

	traverse_recurse(
		filepath,
		&stage,
		world,
		&mut transform_stack,
		assets,
		&pseudo_root,
	);
}

fn from_usd_vec3f(v: gf::Vec3f) -> Vec3 {
	Vec3::new(v.x, v.y, v.z)
}

fn from_usd_vec3d(v: gf::Vec3d) -> Vec3 {
	Vec3::new(v.x as f32, v.y as f32, v.z as f32)
}

fn from_usd_quatd(q: gf::Quatd) -> UnitQuaternion<f32> {
	Unit::new_unchecked(Quaternion {
		i: q.i as f32,
		j: q.j as f32,
		k: q.k as f32,
		w: q.w as f32,
	})
}

fn from_usd_transform3d(t: gf::Transform3d) -> Transform3 {
	Transform3 {
		translation: from_usd_vec3d(t.translation),
		rotation: from_usd_quatd(t.rotation),
		scale: from_usd_vec3d(t.scale),
	}
}
