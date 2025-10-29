use math::{UnitQuaternion, Vec3, transform::Transform3};

pub struct EditorCamera;

impl EditorCamera {
	pub fn update(transform: &mut Transform3, ui: &mut egui::Ui, response: &egui::Response) {
		const PAN_SPEED: f32 = 0.005;
		const LOOK_SPEED: f32 = 0.003;
		const ZOOM_SPEED: f32 = 0.004;
		const MOVE_SPEED: f32 = 3.0;

		let response = response.interact(egui::Sense::click_and_drag());

		let delta = response.drag_delta();

		if response.dragged_by(egui::PointerButton::Middle) {
			let right = transform.rotation * Vec3::X * -delta.x;
			let up = transform.rotation * Vec3::Y * delta.y;
			transform.translation += (right + up) * PAN_SPEED;
		}

		if response.dragged_by(egui::PointerButton::Secondary) {
			let yaw = UnitQuaternion::from_axis_angle(Vec3::Z, -delta.x * LOOK_SPEED);
			let pitch = UnitQuaternion::from_axis_angle(Vec3::X, -delta.y * LOOK_SPEED);

			transform.rotation = yaw * transform.rotation * pitch;
		}

		if response.hovered() {
			let dt = ui.input(|i| i.predicted_dt);
			let scroll = ui.input(|i| i.smooth_scroll_delta.y);

			transform.translation -= transform.rotation * Vec3::Z * scroll * ZOOM_SPEED;

			let mut movement = Vec3::ZERO;

			if ui.input(|i| i.key_down(egui::Key::Z)) {
				movement -= *Vec3::Z;
			} // TODO: Prevent dereferencing these
			if ui.input(|i| i.key_down(egui::Key::S)) {
				movement += *Vec3::Z;
			}
			if ui.input(|i| i.key_down(egui::Key::Q)) {
				movement -= *Vec3::X;
			}
			if ui.input(|i| i.key_down(egui::Key::D)) {
				movement += *Vec3::X;
			}

			if movement != Vec3::ZERO {
				transform.translation +=
					transform.rotation * *movement.normalize() * dt * MOVE_SPEED;
			}
		}
	}
}
