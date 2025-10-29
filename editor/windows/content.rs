use crate::editor::MyContext;
use crate::icons;
use crate::tabs;

pub struct ContentTab {
	name: String,
}

impl ContentTab {
	pub fn new() -> Self {
		ContentTab {
			name: format!("{} Content", icons::ASSET_MANAGER),
		}
	}
}

impl tabs::Tab<MyContext> for ContentTab {
	fn title(&self) -> &str {
		&self.name
	}

	fn ui(&mut self, ui: &mut egui::Ui, _ctx: &mut MyContext) {
		ui.horizontal(|ui| {
			for i in 0..100 {
				egui::Frame::window(ui.style())
					.shadow(egui::epaint::Shadow {
						extrusion: 8.0,
						color: egui::Color32::from_black_alpha(25),
					})
					.inner_margin(egui::Margin::symmetric(10.0, 30.0))
					.outer_margin(egui::Margin::same(5.0))
					.fill(egui::Color32::from_gray(48))
					.stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(255)))
					.show(ui, |ui| {
						ui.label(format!("Asset {}", i));
					});
			}
		});
	}
}
