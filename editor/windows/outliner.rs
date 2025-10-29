use ecs::{self, Entity, Name, World};
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

	if e.contains::<graphics::camera::Camera> () {
		icons::OUTLINER_DATA_CAMERA
	} else if e.contains::<graphics::scene::DomeLight>() {
		icons::LIGHT_HEMI
	} else if e.contains::<graphics::scene::RectLight>() {
		icons::LIGHT_AREA
	} else if e.contains::<graphics::scene::SphereLight>() {
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
				.sense(egui::Sense::click_and_drag())
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
					for (entity, name) in &ctx.world.query::<(Entity, &Name)>() {
						body.row(16.0, |mut row| {
							row.set_selected(ctx.selection.contains(&entity));

							row.col(|ui| {
								ui.label(
									egui::RichText::new(format!("{}", icon_for_entity(&ctx.world, entity)))
										.color(egui::Color32::from_rgb(193, 133, 84))
										.size(18.0),
								);
							});
							row.col(|ui| {
								ui.label(name.name.to_string());
							});
							row.col(|ui| {
								ui.label(format!("({},{})", entity.index(), entity.generation()));
							});

							let res = row.response();

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
						})
					}
					cmds.execute(&mut ctx.world);
				});
		});
	}
}
