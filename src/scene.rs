use asset::AssetServer;
use ecs::{Name, World};
use math::{transform::Transform3, Vec3, UnitQuaternion, PI};
use graphics::camera::Camera;

pub fn setup_scene(world: &mut World, assets: &mut AssetServer) {
	usd::populate_world_from_usd("../assets/usd/ybot-scene.usdc", world, assets);

	world.spawn((
		Name::new("Camera"),
		Camera::default(),
		Transform3 {
			translation: Vec3::new(0.0, -5.0, 0.9),
			rotation: UnitQuaternion::from_axis_angle(Vec3::X, PI / 2.0),
			scale: Vec3::ONE,
		},
	));
}
