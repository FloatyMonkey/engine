use crate::ecs::{self, Entity, World};
use crate::icons;
use crate::tabs;
use crate::editor::MyContext;

pub struct OutlinerTab {
	name: String,
}

impl OutlinerTab {
	pub fn new() -> Self {
		OutlinerTab { name: format!("{} Outliner", icons::OUTLINER) }
	}
}

fn icon_for_entity(world: &World, entity: Entity) -> char {
	let e = world.entity(entity);

	if e.contains::<crate::scene::DomeLight>() {
		icons::LIGHT_HEMI
	} else if e.contains::<crate::scene::RectLight>() {
		icons::LIGHT_AREA
	} else if e.contains::<crate::scene::SphereLight>() {
		icons::LIGHT_POINT
	} else {
		icons::DOT
	}
}

impl tabs::Tab<MyContext> for OutlinerTab {
	fn title(&self) -> &str {
		&self.name
	}

	fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut MyContext) {
		egui::Frame::none().inner_margin(4.0).show(ui, |ui| {
			egui_extras::TableBuilder::new(ui)
				.striped(true)
				.column(egui_extras::Column::exact(20.0))
				.column(egui_extras::Column::exact(150.0))
				.column(egui_extras::Column::remainder())
				.header(20.0, |mut header| {
					header.col(|_| {});
					header.col(|ui| {
						ui.label("name");
					});
					header.col(|ui| {
						ui.label("id");
					});
				})
				.body(|mut body| {
					let mut cmds = ecs::Commands::new();
					for (entity, name) in &ctx.world.query::<(Entity, &crate::scene::Name)>() {
						body.row(18.0, |mut row| {
							row.col(|ui| {
								ui.label(
									egui::RichText::new(format!("{}", icon_for_entity(&ctx.world, entity)))
										.color(egui::Color32::from_rgb(193, 133, 84))
										.size(18.0),
								);
							});
							row.col(|ui| {
								let text = egui::RichText::new(format!("{}", name.name))
									.color(if ctx.selection.contains(&entity) {
										egui::Color32::from_rgb(224, 162, 59)
									} else {
										ui.style().visuals.text_color()
									});
									
								let res = ui.add(egui::Label::new(text).sense(egui::Sense::click()));

								if res.clicked() {
									ctx.selection.clear();
									ctx.selection.insert(entity);
								}

								res.context_menu(|ui| {
									if ui.button("Delete").clicked() {
										ctx.selection.remove(&entity);
										cmds.despawn(entity);
										ui.close_menu();
									}
								});
							});
							row.col(|ui| {
								ui.label(format!("({},{})", entity.index(), entity.generation()));
							});
						})
					}
					cmds.execute(&mut ctx.world);
				});
		});
	}
}
