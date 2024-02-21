mod tab;
mod tree;

pub use self::tab::Tab;
pub use self::tree::{Node, NodeIndex, Split, Tree};

use egui::style::Margin;
use egui::*;

struct HoverData {
	rect: Rect,
	tabs: Option<Rect>,
	dst: NodeIndex,
	pointer: egui::Pos2,
}

impl HoverData {
	fn resolve(&self) -> (Option<Split>, Rect) {
		if let Some(tabs) = self.tabs {
			return (None, tabs);
		}

		let (rect, pointer) = (self.rect, self.pointer);

		let center = rect.center();
		let pts = [
			center.distance(pointer),
			rect.left_center().distance(pointer),
			rect.right_center().distance(pointer),
			rect.center_top().distance(pointer),
			rect.center_bottom().distance(pointer),
		];

		let position = pts
			.into_iter()
			.enumerate()
			.min_by(|(_, lhs), (_, rhs)| lhs.total_cmp(rhs))
			.map(|(idx, _)| idx)
			.unwrap();

		let (target, other) = match position {
			0 => (None, Rect::EVERYTHING),
			1 => (Some(Split::Left), Rect::everything_left_of(center.x)),
			2 => (Some(Split::Right), Rect::everything_right_of(center.x)),
			3 => (Some(Split::Above), Rect::everything_above(center.y)),
			4 => (Some(Split::Below), Rect::everything_below(center.y)),
			_ => unreachable!(),
		};

		(target, rect.intersect(other))
	}
}

#[derive(Clone, Debug, Default)]
struct State {
	drag_start: Option<Pos2>,
}

impl State {
	pub fn load(ctx: &Context, id: Id) -> Self {
		ctx.data_mut(|d| d.get_temp(id).unwrap_or(Self { drag_start: None }))
	}

	fn store(self, ctx: &Context, id: Id) {
		ctx.data_mut(|d| d.insert_temp(id, self));
	}
}

fn top_rounding(rounding: Rounding) -> Rounding {
	Rounding { nw: rounding.nw, ne: rounding.ne, sw: 0.0, se: 0.0 }
}

fn bottom_rounding(rounding: Rounding) -> Rounding {
	Rounding { nw: 0.0, ne: 0.0, sw: rounding.sw, se: rounding.se }
}

pub fn show<Context>(
	ui: &mut Ui,
	id: egui::Id,
	style: &Style,
	tree: &mut Tree<Context>,
	context: &mut Context,
) {
	let mut state = State::load(ui.ctx(), id);

	let mut rect = ui.max_rect();

	ui.painter().rect_filled(rect, 0.0, Color32::from_gray(0));

	if tree.is_empty() || tree[NodeIndex::root()].is_none() {
		return;
	}

	let rect = {
		rect.min += style.padding.left_top();
		rect.max -= style.padding.right_bottom();
		rect
	};

	tree[NodeIndex::root()].set_rect(rect);

	let mut drag_data = None;
	let mut hover_data = None;

	let pixels_per_point = ui.ctx().pixels_per_point();

	for tree_index in 0..tree.len() {
		let tree_index = NodeIndex(tree_index);
		match &mut tree[tree_index] {
			Node::None => (),

			Node::Horizontal { fraction, rect } => {
				let rect = expand_to_pixel(*rect, pixels_per_point);

				let (left, _separator, right) = style.hsplit(ui, fraction, rect);

				tree[tree_index.left()].set_rect(left);
				tree[tree_index.right()].set_rect(right);
			}

			Node::Vertical { fraction, rect } => {
				let rect = expand_to_pixel(*rect, pixels_per_point);

				let (top, _separator, bottom) = style.vsplit(ui, fraction, rect);

				tree[tree_index.left()].set_rect(top);
				tree[tree_index.right()].set_rect(bottom);
			}

			Node::Leaf { rect, tabs, active, viewport } => {
				let rect = *rect;
				ui.set_clip_rect(rect);

				let height_topbar = style.tabbar_height;

				let bottom_y = rect.min.y + height_topbar;
				let tabbar = rect.intersect(Rect::everything_above(bottom_y));

				let full_response = ui.allocate_rect(rect, egui::Sense::hover());
				let tabs_response = ui.allocate_rect(tabbar, egui::Sense::hover());

				// tabs
				ui.scope(|ui| {
					ui.painter().rect_filled(tabbar, top_rounding(style.tab_rounding), style.tabbar_background);

					let mut ui = ui.child_ui(tabbar, Default::default());
					ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

					ui.horizontal(|ui| {
						for (tab_index, tab) in tabs.iter().enumerate() {
							let id = egui::Id::new((tree_index, tab_index, "tab"));
							let is_being_dragged = ui.memory(|m| m.is_being_dragged(id));

							let is_active = *active == tab_index || is_being_dragged;
							let label = tab.title().to_string();

							if is_being_dragged {
								let layer_id = egui::LayerId::new(egui::Order::Tooltip, id);
								let response = ui
									.with_layer_id(layer_id, |ui| {
										style.tab_title(ui, label, is_active)
									})
									.response;

								let sense = egui::Sense::click_and_drag();
								let response = ui
									.interact(response.rect, id, sense)
									.on_hover_cursor(egui::CursorIcon::Grabbing);

								if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
									let center = response.rect.center();
									let start = state.drag_start.unwrap_or(center);

									let delta = pointer_pos - start;
									if delta.x.abs() > 30.0 || delta.y.abs() > 6.0 {
										ui.ctx().translate_layer(layer_id, delta);

										drag_data = Some((tree_index, tab_index));
									}
								}

								if response.clicked() {
									*active = tab_index;
								}
							} else {
								let response = style.tab_title(ui, label, is_active);
								let sense = egui::Sense::click_and_drag();
								let response = ui.interact(response.rect, id, sense);
								if response.drag_started() {
									state.drag_start = response.hover_pos();
								}
							}
						}
					});
				});

				// tab body
				if let Some(tab) = tabs.get_mut(*active) {
					let top_y = rect.min.y + height_topbar;
					let rect = rect.intersect(Rect::everything_below(top_y));
					let rect = expand_to_pixel(rect, pixels_per_point);

					*viewport = rect;

					ui.painter().rect_filled(rect, bottom_rounding(style.tab_rounding), style.background);

					let mut ui = ui.child_ui(rect, Default::default());
					tab.ui(&mut ui, context);
				}

				let is_being_dragged = ui.memory(|m| m.is_anything_being_dragged());
				if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
					// TODO: These rect.contains() don't feel like the best solution, but it works for now.
					if is_being_dragged && full_response.rect.contains(pointer_pos) {
						hover_data = Some(HoverData {
							rect,
							dst: tree_index,
							tabs: tabs_response.rect.contains(pointer_pos).then(|| tabs_response.rect),
							pointer: pointer_pos,
						});
					}
				}
			}
		}
	}

	if let (Some((src, tab_index)), Some(hover)) = (drag_data, hover_data) {
		let dst = hover.dst;

		if tree[src].is_leaf() && tree[dst].is_leaf() {
			let (target, helper) = hover.resolve();

			let id = egui::Id::new("helper");
			let layer_id = egui::LayerId::new(egui::Order::Foreground, id);
			let painter = ui.ctx().layer_painter(layer_id);
			painter.rect_filled(helper, 0.0, style.selection);

			if ui.input(|i| i.pointer.any_released()) {
				if let Node::Leaf { active, .. } = &mut tree[src] {
					if *active >= tab_index {
						*active = active.saturating_sub(1);
					}
				}

				let tab = tree[src].remove_tab(tab_index).unwrap();

				if let Some(target) = target {
					tree.split(dst, target, 0.5, Node::leaf(tab));
				} else {
					tree[dst].append_tab(tab);
				}

				tree.remove_empty_leaf();
				for node in tree.iter_mut() {
					if let Node::Leaf { tabs, active, .. } = node {
						if *active >= tabs.len() {
							*active = 0;
						}
					}
				}
			}
		}
	}

	state.store(ui.ctx(), id);
}

fn expand_to_pixel(mut rect: egui::Rect, ppi: f32) -> egui::Rect {
	rect.min = map_to_pixel_pos(rect.min, ppi, f32::floor);
	rect.max = map_to_pixel_pos(rect.max, ppi, f32::ceil);
	rect
}

fn map_to_pixel_pos(mut pos: egui::Pos2, ppi: f32, map: fn(f32) -> f32) -> egui::Pos2 {
	pos.x = map_to_pixel(pos.x, ppi, map);
	pos.y = map_to_pixel(pos.y, ppi, map);
	pos
}

#[inline(always)]
fn map_to_pixel(point: f32, ppi: f32, map: fn(f32) -> f32) -> f32 {
	map(point * ppi) / ppi
}

pub struct Style {
	pub padding: Margin,

	pub background: Color32,
	pub selection: Color32,
	pub separator_size: f32,
	pub separator_extra: f32,

	pub tabbar_height: f32,
	pub tabbar_background: Color32,

	pub tab_text: Color32,
	pub tab_rounding: Rounding,
}

impl Default for Style {
	fn default() -> Self {
		Self {
			padding: Margin::same(2.0),

			background: Color32::DARK_GREEN,
			selection: Color32::from_rgb_additive(0, 92, 128),
			separator_size: 3.0,
			separator_extra: 100.0,

			tabbar_height: 20.0,
			tabbar_background: Color32::RED,

			tab_text: Color32::WHITE,
			tab_rounding: Rounding::same(3.0),
		}
	}
}

impl Style {
	pub fn from_egui(style: &egui::Style) -> Self {
		Self {
			selection: style.visuals.selection.bg_fill.linear_multiply(0.5),

			background: Color32::from_gray(0x24),// style.visuals.window_fill(),
			tabbar_background: Color32::from_gray(0x15),//style.visuals.faint_bg_color,

			tab_text: style.visuals.widgets.active.fg_stroke.color,

			..Self::default()
		}
	}

	fn hsplit(&self, ui: &mut Ui, fraction: &mut f32, rect: Rect) -> (Rect, Rect, Rect) {
		let pixels_per_point = ui.ctx().pixels_per_point();

		let mut separator = rect;

		let midpoint = rect.min.x + rect.width() * *fraction;
		separator.min.x = midpoint - self.separator_size * 0.5;
		separator.max.x = midpoint + self.separator_size * 0.5;

		let response = ui
			.allocate_rect(separator, Sense::click_and_drag())
			.on_hover_cursor(CursorIcon::ResizeHorizontal);

		{
			let delta = response.drag_delta().x;
			let range = rect.max.x - rect.min.x;
			let min = (self.separator_extra / range).min(1.0);
			let max = 1.0 - min;
			let (min, max) = (min.min(max), max.max(min));
			*fraction = (*fraction + delta / range).clamp(min, max);
		}

		let midpoint = rect.min.x + rect.width() * *fraction;
		separator.min.x = map_to_pixel(midpoint - self.separator_size * 0.5, pixels_per_point, f32::round);
		separator.max.x = map_to_pixel(midpoint + self.separator_size * 0.5, pixels_per_point, f32::round);

		(
			rect.intersect(Rect::everything_right_of(separator.max.x)),
			separator,
			rect.intersect(Rect::everything_left_of(separator.min.x)),
		)
	}

	fn vsplit(&self, ui: &mut Ui, fraction: &mut f32, rect: Rect) -> (Rect, Rect, Rect) {
		let pixels_per_point = ui.ctx().pixels_per_point();

		let mut separator = rect;

		let midpoint = rect.min.y + rect.height() * *fraction;
		separator.min.y = midpoint - self.separator_size * 0.5;
		separator.max.y = midpoint + self.separator_size * 0.5;

		let response = ui
			.allocate_rect(separator, Sense::click_and_drag())
			.on_hover_cursor(CursorIcon::ResizeVertical);

		{
			let delta = response.drag_delta().y;
			let range = rect.max.y - rect.min.y;
			let min = (self.separator_extra / range).min(1.0);
			let max = 1.0 - min;
			let (min, max) = (min.min(max), max.max(min));
			*fraction = (*fraction + delta / range).clamp(min, max);
		}

		let midpoint = rect.min.y + rect.height() * *fraction;
		separator.min.y = map_to_pixel(midpoint - self.separator_size * 0.5, pixels_per_point, f32::round);
		separator.max.y = map_to_pixel(midpoint + self.separator_size * 0.5, pixels_per_point, f32::round);

		(
			rect.intersect(Rect::everything_below(separator.max.y)),
			separator,
			rect.intersect(Rect::everything_above(separator.min.y)),
		)
	}

	fn tab_title(&self, ui: &mut egui::Ui, label: String, active: bool) -> egui::Response {
		let px = ui.ctx().pixels_per_point().recip();

		let font_id = egui::FontId::proportional(14.0);
		let galley = ui.painter().layout_no_wrap(label, font_id, self.tab_text);

		let offset = egui::vec2(8.0, 0.0);
		let text_size = galley.size();

		let mut desired_size = text_size + offset * 2.0;
		desired_size.y = self.tabbar_height;

		let (rect, response) = ui.allocate_at_least(desired_size, egui::Sense::hover());
		let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

		if active {
			let mut tab = rect;

			tab.min.x += px;
			tab.max.x -= px;
			tab.min.y += px;
			ui.painter().rect_filled(tab, top_rounding(self.tab_rounding), self.background);
		}

		let pos = egui::Align2::LEFT_CENTER
			.align_size_within_rect(galley.size(), rect.shrink2(egui::vec2(8.0, 0.0)))
			.min;

		ui.painter().galley(pos, galley, Color32::PLACEHOLDER);

		response
	}
}
