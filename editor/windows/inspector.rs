use crate::icons;
use crate::tabs;
use crate::editor::MyContext;
use ecs::Name;

use egui::DragValue;

pub struct InspectorTab {
	value: (f32, f32, f32),
	name: String,
}

impl InspectorTab {
	pub fn new() -> Self {
		InspectorTab { value: Default::default(), name: format!("{} Inspector", icons::PROPERTIES) }
	}
}

impl tabs::Tab<MyContext> for InspectorTab {
	fn title(&self) -> &str {
		&self.name
	}

	fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut MyContext) {
		const WRAP_WIDTH: f32 = 235.0;
		const PERCENT: f32 = 0.35;
		let wrapping = ui.available_width() > WRAP_WIDTH;

		egui::Frame::none().inner_margin(4.0).show(ui, |ui| {
			if ctx.selection.is_empty() {
				ui.vertical_centered(|ui| {
					ui.label("Nothing selected");
				});

				return;
			}

			let width = ui.available_width();
			let height = 18.0;
			let pad = 16.0;
			let level = 0;

			let label = |ui: &mut egui::Ui, label: &str| {
				let indent = pad * level as f32;
				let _ = ui.allocate_space(egui::vec2(indent, height));
				let (id, space) = ui.allocate_space(egui::vec2(width * PERCENT - indent, height));
				let layout = egui::Layout::left_to_right(egui::emath::Align::LEFT);
				let mut ui = ui.child_ui_with_id_source(space, layout, id);
				ui.label(label);
			};

			if let Some(selection) = ctx.selection.iter().next() {
				if let Some(name) = ctx.world.entity_mut(*selection).get_mut::<Name>() {
					ui.add(egui::TextEdit::singleline(&mut name.name).desired_width(f32::INFINITY));
				}
			}

			ui.collapsing("Transform", |ui| {
				ui.horizontal(|ui| {
					label(ui, "Translation");
					let num = if wrapping { 3 } else { 1 };
					ui.columns(num, |ui| {
						ui[0.min(num - 1)].add(DragValue::new(&mut self.value.0).speed(0.1).suffix(" m"));
						ui[1.min(num - 1)].add(DragValue::new(&mut self.value.1).speed(0.1).suffix(" m"));
						ui[2.min(num - 1)].add(DragValue::new(&mut self.value.2).speed(0.1).suffix(" m"));
					});
				});

				ui.horizontal(|ui| {
					label(ui, "Rotation");
					let num = if wrapping { 3 } else { 1 };
					ui.columns(num, |ui| {
						ui[0.min(num - 1)].add(DragValue::new(&mut self.value.0).speed(0.1).suffix(" °"));
						ui[1.min(num - 1)].add(DragValue::new(&mut self.value.1).speed(0.1).suffix(" °"));
						ui[2.min(num - 1)].add(DragValue::new(&mut self.value.2).speed(0.1).suffix(" °"));
					});
				});

				ui.horizontal(|ui| {
					label(ui, "Scale");
					let num = if wrapping { 3 } else { 1 };
					ui.columns(num, |ui| {
						ui[0.min(num - 1)].add(DragValue::new(&mut self.value.0).speed(0.1));
						ui[1.min(num - 1)].add(DragValue::new(&mut self.value.1).speed(0.1));
						ui[2.min(num - 1)].add(DragValue::new(&mut self.value.2).speed(0.1));
					});
				});
			});
		});
	}
}
