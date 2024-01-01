use crate::math::{matrix::Vec3, transform::Transform3};

pub struct Camera {
	/// Focal length in millimeters.
	pub focal_length: f32,
	/// Focus distance in meters.
	pub focus_distance: f32,

	// full-frame is 36x24mm
	pub sensor_width: f32,
	pub sensor_height: f32,

	pub depth_of_field: bool,
	pub f_stop: f32,
	pub shutter_speed: f32,
	pub iso: f32,
}

impl Default for Camera {
	fn default() -> Self {
		Self {
			focal_length: 50.0,
			focus_distance: 3.0,

			sensor_width: 36.0,
			sensor_height: 36.0 / (16.0 / 9.0), //24.0,

			depth_of_field: false,
			f_stop: 22.0, // 1.4
			shutter_speed: 1.0 / 125.0,
			iso: 100.0,
		}
	}
}

#[repr(C)]
pub struct GpuCamera {
	pub u: Vec3,
	pub scale_u: f32,
	pub v: Vec3,
	pub scale_v: f32,
	pub w: Vec3,
	pub scale_w: f32,
	pub position: Vec3,
	pub aperture_radius: f32,
}

impl GpuCamera {
	pub fn from_camera(camera: &Camera, transform: &Transform3) -> Self {
		let u = transform.rotation * Vec3::X;
		let v = transform.rotation * Vec3::Y;
		let w = transform.rotation * -Vec3::Z;

		let scale_u = 0.5 * camera.sensor_width / camera.focal_length * camera.focus_distance;
		let scale_v = 0.5 * camera.sensor_height / camera.focal_length * camera.focus_distance;
		let scale_w = camera.focus_distance;

		let position = transform.translation;

		let aperture_radius = if camera.depth_of_field {
			0.5 * (camera.focal_length / camera.f_stop) * 0.001
		} else {
			0.0
		};

		Self { u, scale_u, v, scale_v, w, scale_w, position, aperture_radius }
	}
}

enum LensUnit {
	/// Field of view in radians.
	FieldOfView(f32),
	/// Focal length in millimeters.
	FocalLength(f32),
}

// f-stop = focal_length (mm) / aperture diameter (mm)

fn focal_length_to_fov(focal_length: f32, sensor_size: f32) -> f32 {
	2.0 * (0.5 * sensor_size / focal_length).atan()
}

fn fov_to_focal_length(fov: f32, sensor_size: f32) -> f32 {
	0.5 * sensor_size / (0.5 * fov).tan()
}

fn compute_ev_100(aperture: f32, shutter_time: f32, iso: f32) -> f32 {
	(aperture * aperture / shutter_time * 100.0 / iso).log2()
}

fn exposure_from_ev_100(ev_100: f32) -> f32 {
	1.0 / 2.0f32.powf(ev_100)
}
