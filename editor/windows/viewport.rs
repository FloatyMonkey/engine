use graphics::camera::Camera;
use math::{Mat4, Vec3, transform::Transform3, Unit, UnitQuaternion, Quaternion};

use crate::time::Time;
use crate::icons;
use crate::tabs;
use crate::editor::MyContext;
use crate::camera::EditorCamera;

const FRAME_TIME_SMOOTHING: f32 = 30.0;

pub struct ViewportTab {
	name: String,
	frame_time: f32,
	gizmo_mode: Option<egui_gizmo::GizmoMode>,
}

impl ViewportTab {
	pub fn new() -> Self {
		ViewportTab {
			name: format!("{} Viewport", icons::VIEW3D),
			frame_time: 0.0,
			gizmo_mode: None,
		}
	}
}

fn navigation_gizmo(ui: &mut egui::Ui, p: egui::Pos2, rotation: UnitQuaternion<f32>) {
	let outer_radius = 40.0;
	let inner_radius = 30.0;

	let painter = ui.painter();

	let center = p + egui::Vec2::splat(outer_radius);

	if let Some(pointer_pos) = ui.ctx().pointer_latest_pos()
		&& (pointer_pos - center).length_sq() < outer_radius * outer_radius {
			painter.circle_filled(center, outer_radius, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10));
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
		let max_rect = ui.available_rect_before_wrap();

		ui.set_clip_rect(max_rect);

		// TODO: Also scale the render target texture
		let camera = ctx.world.query::<&mut Camera>().iter().next().unwrap();
		camera.sensor_height = camera.sensor_width / (max_rect.width() / max_rect.height());

		let response = ui.add(
			egui::Image::from_texture((egui::TextureId::User(ctx.viewport_texture_srv as u64), max_rect.size()))
				.rounding(egui::Rounding { nw: 0.0, ne: 0.0, se: 3.0, sw: 3.0 })
				.sense(egui::Sense::click_and_drag())
		);

		let mut toolbar_ui = ui.child_ui_with_id_source(
			egui::Align2::LEFT_TOP.align_size_within_rect(egui::vec2(32.0, 400.0), max_rect.shrink(10.0)),
			egui::Layout::top_down_justified(egui::Align::Center),
			egui::Id::new("toolbar"),
		);

		egui::Frame::none().show(&mut toolbar_ui, |ui| {
			ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

			let visuals = &mut ui.style_mut().visuals;

			visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
			visuals.widgets.hovered.expansion = 0.0;
			visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
			visuals.widgets.active.expansion = 0.0;

			visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgba_premultiplied(10, 11, 12, 210);

			visuals.selection.stroke = egui::Stroke::NONE;
			visuals.selection.bg_fill = egui::Color32::from_rgba_premultiplied(4, 119, 213, 240);

			if ui.add(egui::ImageButton::new(egui::include_image!("../../resources/mouse-pointer.svg")).selected(self.gizmo_mode.is_none()).rounding(egui::Rounding{nw: 5.0, ne: 5.0, se: 0.0, sw: 0.0})).clicked() {
				self.gizmo_mode = None;
			}

			if ui.add(egui::ImageButton::new(egui::include_image!("../../resources/move-3d.svg")).selected(self.gizmo_mode == Some(egui_gizmo::GizmoMode::Translate))).clicked() {
				self.gizmo_mode = Some(egui_gizmo::GizmoMode::Translate);
			}

			if ui.add(egui::ImageButton::new(egui::include_image!("../../resources/rotate-3d.svg")).selected(self.gizmo_mode == Some(egui_gizmo::GizmoMode::Rotate))).clicked() {
				self.gizmo_mode = Some(egui_gizmo::GizmoMode::Rotate);
			}

			if ui.add(egui::ImageButton::new(egui::include_image!("../../resources/scale-3d.svg")).selected(self.gizmo_mode == Some(egui_gizmo::GizmoMode::Scale)).rounding(egui::Rounding{nw: 0.0, ne: 0.0, se: 5.0, sw: 5.0})).clicked() {
				self.gizmo_mode = Some(egui_gizmo::GizmoMode::Scale);
			}
		});

		let time = ctx.world.get_singleton::<Time>().unwrap();
		self.frame_time += (time.delta_seconds() - self.frame_time) / FRAME_TIME_SMOOTHING;
		ui.painter().text(cursor.left_top() + egui::vec2(55.0, 10.0), egui::Align2::LEFT_TOP, format!("dt: {:.2} ms", self.frame_time * 1000.0), egui::FontId::monospace(12.0), egui::Color32::WHITE);

		let (camera_transform, camera) = ctx.world.query::<(&Transform3, &Camera)>().iter().next().unwrap();

		navigation_gizmo(ui, cursor.right_top() + egui::vec2(-10.0 - 80.0, 10.0), camera_transform.rotation.inv());

		let view_matrix = Mat4::from(camera_transform.inv());
		let projection_matrix = camera.projection_matrix();

		if let Some(selection) = ctx.selection.iter().next()
			&& let (Some(mode), Some(transform)) = (self.gizmo_mode, ctx.world.entity_mut(*selection).get_mut::<Transform3>()) {

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
					.mode(mode)
					.visuals(visuals);

				if let Some(response) = gizmo.interact(ui) {
					transform.translation.x = response.translation.x;
					transform.translation.y = response.translation.y;
					transform.translation.z = response.translation.z;

					transform.rotation = Unit::new_unchecked(Quaternion {
						i: response.rotation.v.x,
						j: response.rotation.v.y,
						k: response.rotation.v.z,
						w: response.rotation.s,
					});

					transform.scale.x = response.scale.x;
					transform.scale.y = response.scale.y;
					transform.scale.z = response.scale.z;
				}
			}

		// TODO: Ideally, we shouldn't query the camera twice in this function
		if let Some((camera_transform, _)) = ctx.world.query::<(&mut Transform3, &Camera)>().iter().next() {
			EditorCamera::update(camera_transform, ui, &response);
		}
	}
}
