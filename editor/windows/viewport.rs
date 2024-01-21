use crate::math::{matrix, Mat4, Vec3, transform::Transform3, UnitQuaternion};
use crate::time::Time;

use crate::icons;
use crate::tabs;
use crate::editor::MyContext;
use crate::camera::EditorCamera;

pub struct ViewportTab {
	name: String,
}

impl ViewportTab {
	pub fn new() -> Self {
		ViewportTab {
			name: format!("{} Viewport", icons::VIEW3D),
		}
	}
}

fn navigation_gizmo(ui: &mut egui::Ui, p: egui::Pos2, rotation: UnitQuaternion<f32>) {
	let outer_radius = 40.0;
	let inner_radius = 30.0;

	let painter = ui.painter();

	let center = p + egui::Vec2::splat(outer_radius);

	if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
		if (pointer_pos - center).length_sq() < outer_radius * outer_radius {
			painter.circle_filled(center, outer_radius, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10));
		}
	}

	let scale = Vec3::new(inner_radius, -inner_radius, 1.0);

	let mut axes = [
		( "X", (rotation *  Vec3::X).cmul(scale), egui::Color32::from_rgb(246,  54,  82)),
		( "Y", (rotation *  Vec3::Y).cmul(scale), egui::Color32::from_rgb(112, 164,  28)),
		( "Z", (rotation *  Vec3::Z).cmul(scale), egui::Color32::from_rgb( 47, 132, 227)),
		("-X", (rotation * -Vec3::X).cmul(scale), egui::Color32::from_rgba_unmultiplied(246,  54,  82, 128)),
		("-Y", (rotation * -Vec3::Y).cmul(scale), egui::Color32::from_rgba_unmultiplied(112, 164,  28, 128)),
		("-Z", (rotation * -Vec3::Z).cmul(scale), egui::Color32::from_rgba_unmultiplied( 47, 132, 227, 128)),
	];

	axes.sort_by(|a, b| a.1.z.partial_cmp(&b.1.z).unwrap());

	for (name, p, color) in axes.iter() {
		let ax = egui::vec2(p.x, p.y);

		if name.starts_with('-') {
			painter.circle(center + ax, 6.0, *color, (1.0, color.to_opaque()));
		} else {
			painter.line_segment([center, center + ax], (2.0, *color));
			painter.circle_filled(center + ax, 8.0, *color);
			painter.text(center + ax, egui::Align2::CENTER_CENTER, name, egui::FontId::monospace(12.0), egui::Color32::BLACK);
		}
	}
}

impl tabs::Tab<MyContext> for ViewportTab {
	fn title(&self) -> &str {
		&self.name
	}

	fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut MyContext) {
		let cursor = ui.cursor();

		let response = ui.add(
			egui::Image::from_texture((egui::TextureId::User(ctx.viewport_texture_srv as u64), ui.available_size_before_wrap()))
				.rounding(egui::Rounding { nw: 0.0, ne: 0.0, se: 3.0, sw: 3.0 })
		);

		let time = ctx.world.get_singleton::<Time>().unwrap();
		ui.painter().text(cursor.left_top() + egui::vec2(10.0, 10.0), egui::Align2::LEFT_TOP, format!("dt: {:.2} ms", time.delta_seconds() * 1000.0), egui::FontId::monospace(12.0), egui::Color32::WHITE);

		navigation_gizmo(ui, cursor.right_top() + egui::vec2(-10.0 - 80.0, 10.0), ctx.camera_transform.rotation.inv());

		let aspect_ratio = 16.0 / 9.0; // TODO: hardcoded
		let projection_matrix = matrix::perspective(24.0_f32.to_radians(), aspect_ratio, 0.1, 1000.0);
		let view_matrix = Mat4::from(ctx.camera_transform.inv());

		if let Some(selection) = ctx.selection.iter().next() {
			if let Some(transform) = ctx.world.entity_mut(*selection).get_mut::<Transform3>() {

				let model_matrix = Mat4::from(*transform);

				let visuals = egui_gizmo::GizmoVisuals {
					x_color: egui::Color32::from_rgb(246,  54,  82),
					y_color: egui::Color32::from_rgb(112, 164,  28),
					z_color: egui::Color32::from_rgb( 47, 132, 227),
					inactive_alpha: 0.75,
					highlight_alpha: 1.0,
					..Default::default()
				};

				let gizmo = egui_gizmo::Gizmo::new("My gizmo")
					.view_matrix(view_matrix.transpose().data.into())
					.projection_matrix(projection_matrix.transpose().data.into())
					.model_matrix(model_matrix.transpose().data.into())
					.mode(egui_gizmo::GizmoMode::Translate)
					.visuals(visuals);

				if let Some(response) = gizmo.interact(ui) {
					transform.translation.x = response.translation.x;
					transform.translation.y = response.translation.y;
					transform.translation.z = response.translation.z;
				}
			}
		}
		
		EditorCamera::update(&mut ctx.camera_transform, ui, &response);
	}
}
