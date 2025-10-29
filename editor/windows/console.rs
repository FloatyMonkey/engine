use crate::editor::MyContext;
use crate::icons;
use crate::tabs;

type GlobalLog = Vec<String>;
static LOG: std::sync::Mutex<GlobalLog> = std::sync::Mutex::new(Vec::new());

pub struct Log {}

impl log::Log for Log {
	fn enabled(&self, metadata: &log::Metadata) -> bool {
		metadata.level() <= log::Level::Info
	}

	fn log(&self, record: &log::Record) {
		if self.enabled(record.metadata()) {
			let msg = format!("{}: {}", record.level(), record.args());
			let mut log = LOG.lock().unwrap();
			log.push(msg);
		}
	}

	fn flush(&self) {}
}

pub struct ConsoleTab {
	name: String,
	auto_scroll: bool,
}

impl ConsoleTab {
	pub fn new() -> Self {
		ConsoleTab {
			name: format!("{} Console", icons::CONSOLE),
			auto_scroll: true,
		}
	}
}

impl tabs::Tab<MyContext> for ConsoleTab {
	fn title(&self) -> &str {
		&self.name
	}

	fn ui(&mut self, ui: &mut egui::Ui, _ctx: &mut MyContext) {
		let log = &mut LOG.lock().unwrap();

		let dropped_entries = log.len().saturating_sub(10000);
		drop(log.drain(..dropped_entries));

		ui.push_id("console_tab", |ui| {
			egui::Frame::none().inner_margin(4.0).show(ui, |ui| {
				ui.horizontal(|ui| {
					if ui.button("Clear").clicked() {
						log.clear();
					}
					ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
				});

				ui.add_space(4.0);

				let mut table = egui_extras::TableBuilder::new(ui)
					.striped(true)
					.column(egui_extras::Column::remainder().clip(true));

				if self.auto_scroll {
					table = table.scroll_to_row(log.len().saturating_sub(1), None);
				}

				table.body(|body| {
					body.rows(18.0, log.len(), |mut row| {
						let row_idx = row.index();
						row.col(|ui| {
							ui.label(log[row_idx].as_str());
						});
					});
				});
			});
		});
	}
}
