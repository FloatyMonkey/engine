use crate::{
	asset::{AssetId, AssetServer},
	ecs::World,
	geometry::{self, mesh::Mesh},
	graphics::{camera::Camera, scene::Image},
	math::{transform::Transform3, Vec3, UnitQuaternion, PI},
};

pub struct Name {
	pub name: String,
}

impl Name {
	pub fn new(name: impl Into<String>) -> Self {
		Self {
			name: name.into(),
		}
	}
}

pub struct Renderable {
	pub mesh: AssetId<Mesh>,
}

pub struct DomeLight {
	pub image: AssetId<Image>,
}

pub struct SphereLight {
	pub emission: [f32; 3],
	pub radius: f32,
}

pub struct RectLight {
	pub emission: [f32; 3],
	pub width: f32,
	pub height: f32,
}

pub fn setup_scene(world: &mut World, assets: &mut AssetServer) {
	let proc_mesh = geometry::primitives::grid::grid(10.0, 10.0, 2, 2);
	//let proc_mesh = geometry::primitives::sphere::sphere(1.0, 32, 16);
	//let proc_mesh = geometry::primitives::torus::torus(32, 8, 3.0, 0.5);

	let grid_mesh = assets.insert(proc_mesh);
	let ybot_mesh = assets.load("../assets/YBot-mesh.fme");

	world.spawn((
		Name::new("Camera"),
		Camera::default(),
		Transform3 {
			translation: Vec3::new(0.0, -5.0, 0.9),
			rotation: UnitQuaternion::from_axis_angle(Vec3::X, PI / 2.0),
			scale: Vec3::ONE,
		},
	));

	world.spawn((
		Name::new("Grid"),
		Transform3::<f32>::IDENTITY,
		Renderable { mesh: grid_mesh },
	));

	world.spawn((
		Name::new("YBot 1"),
		Transform3::<f32>::IDENTITY,
		Renderable { mesh: ybot_mesh },
	));

	world.spawn((
		Name::new("YBot 2"),
		Transform3::from_translation(Vec3::new(2.0, 0.5, 0.0)),
		Renderable { mesh: ybot_mesh },
	));

	world.spawn((
		Name::new("Dome Light"),
		DomeLight {
			image: assets.load("../assets/meadow_2_1k.exr"),
		},
	));

	world.spawn((
		Name::new("Rect Light"),
		Transform3::from_translation(Vec3::new(0.0, 0.0, 3.0)),
		RectLight {
			emission: [0.0, 0.0, 250.0],
			width: 0.5,
			height: 0.5,
		},
	));

	world.spawn((
		Name::new("Sphere Light"),
		Transform3::from_translation(Vec3::new(0.0, 0.0, 3.0)),
		SphereLight {
			emission: [250.0, 0.0, 0.0],
			radius: 0.25,
		},
	));
}
