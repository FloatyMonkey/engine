use std::collections::HashSet;

use crate::math::{UnitQuaternion, Vec3, transform::Transform3, PI};
use crate::ecs::{Entity, World};
use crate::time::Time;

use super::tabs;
use super::windows;

pub struct MyContext {
	pub world: World,
	pub selection: HashSet::<Entity>,
	pub viewport_texture_srv: u32,
	pub camera_transform: Transform3,
}

pub struct Editor {
	pub egui_ctx: egui::Context,
	pub context: MyContext,
	tree: tabs::Tree<MyContext>,
}

impl Editor {
	pub fn new() -> Self {
		let egui_ctx = egui::Context::default();

		let default_font = egui::FontData::from_static(include_bytes!("../resources/Inter-Regular.ttf"));
		let icon_font = egui::FontData::from_static(include_bytes!("../resources/icon.ttf"));

		let mut fonts = egui::FontDefinitions::empty();

		fonts.font_data.insert("Inter-Regular".to_owned(), default_font);
		fonts.font_data.insert("icons".to_owned(), icon_font);

		if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
			family.push("Inter-Regular".to_owned());
			family.push("icons".to_owned());
		}

		if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
			family.push("Inter-Regular".to_owned()); // TODO: this is not monospace
			family.push("icons".to_owned());
		}

		egui_ctx.set_fonts(fonts);

		let mut world = World::new();
		world.add_singleton(Time::new());

		Self {
			egui_ctx,
			context: MyContext {
				world,
				selection: HashSet::new(),
				viewport_texture_srv: 0,
				camera_transform: Transform3 {
					translation: Vec3::new(0.0, -5.0, 0.9),
					rotation: UnitQuaternion::from_axis_angle(Vec3::X, PI / 2.0),
					scale: Vec3::ONE,
				},
			},
			tree: Self::setup_tree(),
		}
	}

	pub fn run(&mut self, raw_input: egui::RawInput) -> egui::FullOutput {
		if let Some(time) = self.context.world.get_singleton_mut::<Time>() {
			time.update();
		}

		self.egui_ctx.clone().run(raw_input, |ctx| {
			egui::CentralPanel::default().show(&ctx, |ui| {
				let style = tabs::Style::from_egui(ui.style().as_ref());

				let id = egui::Id::new("Tabs");
				let layer_id = egui::LayerId::background();
				let max_rect = self.egui_ctx.available_rect();
				let clip_rect = self.egui_ctx.available_rect();

				let mut ui = egui::Ui::new(self.egui_ctx.clone(), layer_id, id, max_rect, clip_rect);
				tabs::show(&mut ui, id, &style, &mut self.tree, &mut self.context);
			});
		})
	}

	fn setup_tree() -> tabs::Tree<MyContext> {
		let viewport = Box::new(windows::ViewportTab::new());

		let outliner = Box::new(windows::OutlinerTab::new());
		let inspector = Box::new(windows::InspectorTab::new());

		let content = Box::new(windows::ContentTab::new());

		let mut tree = tabs::Tree::new(vec![viewport]);

		let [a, b] = tree.split_right(tabs::NodeIndex::root(), 0.85, vec![outliner]);
		let [_, _] = tree.split_below(a, 0.9, vec![content]);
		let [_, _] = tree.split_below(b, 0.5, vec![inspector]);

		tree
	}
}
