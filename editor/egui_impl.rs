use std::collections::HashMap;

use std::ops::Range;

use crate::os::{self, App, Window};
use crate::gpu::{self, DeviceImpl, CmdListImpl, BufferImpl, TextureImpl};

// TODO: Dynamically reallocate
const VERTEX_BUFFER_SIZE: usize = std::mem::size_of::<egui::epaint::Vertex>() * 100000;
const INDEX_BUFFER_SIZE: usize = std::mem::size_of::<u32>() * 3 * 100000;

pub struct ScreenDesc {
	pub size_in_pixels: [u32; 2],
	pub pixels_per_point: f32,
}

impl ScreenDesc {
	fn size_in_points(&self) -> [f32; 2] {
		[
			self.size_in_pixels[0] as f32 / self.pixels_per_point,
			self.size_in_pixels[1] as f32 / self.pixels_per_point,
		]
	}
}

pub struct EguiRenderer {
	pipeline: gpu::GraphicsPipeline,
	vb: gpu::Buffer,
	ib: gpu::Buffer,
	ib_ranges: Vec<Range<usize>>,
	textures: HashMap<egui::TextureId, gpu::Texture>,
}

impl EguiRenderer {
	pub fn new(device: &mut gpu::Device, shader_compiler: &gpu::ShaderCompiler) -> Self {
		let vb = device.create_buffer(&gpu::BufferDesc {
			size: std::mem::size_of::<egui::epaint::Vertex>() * VERTEX_BUFFER_SIZE,
			usage: gpu::BufferUsage::SHADER_RESOURCE,
			cpu_access: gpu::CpuAccessFlags::WRITE,
		}, None).unwrap();

		let ib = device.create_buffer(&gpu::BufferDesc {
			size: std::mem::size_of::<u32>() * INDEX_BUFFER_SIZE,
			usage: gpu::BufferUsage::SHADER_RESOURCE,
			cpu_access: gpu::CpuAccessFlags::WRITE,
		}, None).unwrap();
		
		let slang_vs = shader_compiler.compile("shaders/egui.slang", "main_vs");
		let slang_ps = shader_compiler.compile("shaders/egui.slang", "main_ps");

		let vs = device.create_shader(&gpu::ShaderDesc {
			ty: gpu::ShaderType::Vertex,
		}, &slang_vs).unwrap();

		let fs = device.create_shader(&gpu::ShaderDesc {
			ty: gpu::ShaderType::Pixel,
		}, &slang_ps).unwrap();

		let pipeline_desc = gpu::GraphicsPipelineDesc {
			vs: Some(&vs),
			ps: Some(&fs),
			descriptor_layout: gpu::DescriptorLayout {
				push_constants: Some(gpu::PushConstantBinding {
					size: 5 * 4,
				}),
				bindings: Some(vec![
					gpu::DescriptorBinding::bindless_srv(1), // buffers
					gpu::DescriptorBinding::bindless_srv(2), // textures
				]),
				static_samplers: Some(vec![gpu::SamplerBinding {
					shader_register: 0,
					register_space: 0,
					sampler_desc: gpu::SamplerDesc {
						filter_min: gpu::FilterMode::Linear,
						filter_mag: gpu::FilterMode::Linear,
						filter_mip: gpu::FilterMode::Linear,
						..Default::default()
					}
				}]),
			},
			rasterizer: gpu::RasterizerDesc::default(),
			depth_stencil: gpu::DepthStencilDesc::default(),
			color_attachments: &[gpu::ColorAttachment {
				format: gpu::Format::RGBA8UNorm, // TODO: Hardcoded
				blend: Some(gpu::BlendDesc {
					src_color: gpu::BlendFactor::One,
					dst_color: gpu::BlendFactor::InvSrcAlpha,
					color_op: gpu::BlendOp::Add,
					src_alpha: gpu::BlendFactor::InvDstAlpha,
					dst_alpha: gpu::BlendFactor::One,
					alpha_op: gpu::BlendOp::Add,
				}),
				write_mask: gpu::ColorWriteMask::ALL,
			}],
			ds_format: gpu::Format::Unknown,
			topology: gpu::Topology::TriangleList,
		};

		let pipeline = device.create_graphics_pipeline(&pipeline_desc).unwrap();

		Self {
			pipeline,
			vb,
			ib,
			ib_ranges: Vec::new(),
			textures: Default::default(),
		}
	}

	pub fn paint(&mut self, device: &gpu::Device, cmd: &gpu::CmdList, clipped_primitives: &[egui::ClippedPrimitive], screen_desc: &ScreenDesc) {
		self.update_buffers(clipped_primitives);

		let size_in_pixels = screen_desc.size_in_pixels;
		let size_in_points = screen_desc.size_in_points();
		let vb_id = self.vb.srv_index().unwrap();

		cmd.set_viewport(&gpu::Viewport {
			x: 0.0,
			y: 0.0,
			width: size_in_pixels[0] as f32,
			height: size_in_pixels[1] as f32,
			min_depth: 0.0,
			max_depth: 1.0,
		});

		cmd.set_index_buffer(&self.ib, 0, gpu::Format::R32UInt);
		cmd.set_graphics_pipeline(&self.pipeline);
		cmd.set_graphics_root_table(&device, 1, 0);
		cmd.graphics_push_constants(0, gpu::slice_as_u8_slice(&size_in_points));
		cmd.graphics_push_constants(2 * 4, gpu::as_u8_slice(&vb_id));

		self.paint_primitives(cmd, clipped_primitives, screen_desc);
	}
	
	fn paint_primitives(&self, cmd: &gpu::CmdList, clipped_primitives: &[egui::ClippedPrimitive], screen_desc: &ScreenDesc) {
		for (i, egui::ClippedPrimitive { clip_rect, primitive }) in clipped_primitives.iter().enumerate() {
			match primitive {
				egui::epaint::Primitive::Mesh(mesh) => {
					self.paint_mesh(cmd, clip_rect, mesh, i, screen_desc);
				}
				egui::epaint::Primitive::Callback(_) => {
					unimplemented!()
				}
			}
		}
	}

	fn update_buffers(&mut self, clipped_primitives: &[egui::ClippedPrimitive]) {
		self.ib_ranges.clear();

		let map_vb = self.vb.cpu_ptr() as *mut egui::epaint::Vertex;
		let map_ib = self.ib.cpu_ptr() as *mut u32;

		let mut vertex_offset = 0;

		for egui::ClippedPrimitive { clip_rect: _, primitive } in clipped_primitives {
			match primitive {
				egui::epaint::Primitive::Mesh(mesh) => {
					let index_offset = self.ib_ranges.last().unwrap_or(&(0..0)).end;

					unsafe {
						std::ptr::copy_nonoverlapping(mesh.vertices.as_ptr(), map_vb.add(vertex_offset), mesh.vertices.len());
					}

					for i in 0..mesh.indices.len() {
						unsafe {
							*map_ib.add(index_offset + i) = mesh.indices[i] + vertex_offset as u32;
						}
					}

					self.ib_ranges.push(index_offset..(mesh.indices.len() + index_offset));

					vertex_offset += mesh.vertices.len();
				}
				egui::epaint::Primitive::Callback(_) => {
					unimplemented!()
				}
			}
		}
	}

	fn paint_mesh(&self, cmd: &gpu::CmdList, clip_rect: &egui::Rect, mesh: &egui::epaint::Mesh, i: usize, screen_desc: &ScreenDesc) {
		let pixels_per_point = screen_desc.pixels_per_point;
		let size_in_pixels = screen_desc.size_in_pixels;

		let clip_min_x = clip_rect.min.x * pixels_per_point;
		let clip_min_y = clip_rect.min.y * pixels_per_point;
		let clip_max_x = clip_rect.max.x * pixels_per_point;
		let clip_max_y = clip_rect.max.y * pixels_per_point;

		let clip_min_x = clip_min_x.round() as u32;
		let clip_min_y = clip_min_y.round() as u32;
		let clip_max_x = clip_max_x.round() as u32;
		let clip_max_y = clip_max_y.round() as u32;

		let clip_min_x = clip_min_x.clamp(0, size_in_pixels[0] as u32);
		let clip_min_y = clip_min_y.clamp(0, size_in_pixels[1] as u32);
		let clip_max_x = clip_max_x.clamp(clip_min_x, size_in_pixels[0] as u32);
		let clip_max_y = clip_max_y.clamp(clip_min_y, size_in_pixels[1] as u32);

		cmd.set_scissor(&gpu::Scissor {
			left: clip_min_x,
			top: clip_min_y,
			right: clip_max_x,
			bottom: clip_max_y,
		});

		let texture_id = match &mesh.texture_id {
			egui::TextureId::Managed(_) => {
				self.textures.get(&mesh.texture_id).unwrap().srv_index().unwrap()
			}
			egui::TextureId::User(id) => *id as u32,
		};

		let ib_range = &self.ib_ranges[i];

		cmd.graphics_push_constants(3 * 4, gpu::as_u8_slice(&texture_id));
		cmd.draw_indexed(ib_range.len() as u32, 1, ib_range.start as u32, 0, 0);
	}

	pub fn create_texture(&mut self, device: &mut gpu::Device, id: egui::TextureId, delta: &egui::epaint::ImageDelta) {
		let pixels: Vec<u8> = match &delta.image {
			egui::ImageData::Color(image) => {
				assert_eq!(image.width() * image.height(), image.pixels.len());
				image.pixels.iter().flat_map(|color| color.to_array()).collect()
			}
			egui::ImageData::Font(image) => {
				assert_eq!(image.width() * image.height(), image.pixels.len());
				image.srgba_pixels(None).flat_map(|color| color.to_array()).collect()
			}
		};

		if let Some(_pos) = delta.pos {
			println!("Can't update partial texture!");
		} else {
			let texture = device.create_texture(&gpu::TextureDesc {
				width: delta.image.width() as u64,
				height: delta.image.height() as u64,
				depth: 1,
				array_size: 1,
				mip_levels: 1,
				samples: 1,
				format: gpu::Format::RGBA8UNorm,
				usage: gpu::TextureUsage::SHADER_RESOURCE,
				state: gpu::ResourceState::ShaderResource,
			}, Some(&pixels));

			self.textures.insert(id, texture.unwrap());
		}
	}

	pub fn destroy_texture(&mut self, _device: &mut gpu::Device, _tex_id: egui::TextureId) {
		println!("Can't destroy texture!");
	}
}

pub fn get_raw_input(app: &os::platform::App, window: &os::platform::Window) -> egui::RawInput {
	let mut events: Vec<egui::Event> = vec![];

	let scale_factor = window.scale_factor();
	let mouse_client_pos = window.mouse_pos_client(app.mouse_pos());
	let pos_in_points = egui::pos2(
		mouse_client_pos.x as f32 / scale_factor,
		mouse_client_pos.y as f32 / scale_factor,
	);

	events.push(egui::Event::PointerMoved(pos_in_points));

	for event in app.events() {
		match event {
			os::Event::Key { key, pressed } => {
				events.push(egui::Event::Key { key: map_key(key), pressed, repeat: false, modifiers: Default::default() });
			},
			os::Event::Text { character } => {
				if is_printable_char(character) {
					events.push(egui::Event::Text(character.to_string()));
				}
			},
			os::Event::MouseButton { button, pressed } => {
				let button = match button {
					os::MouseButton::Left => egui::PointerButton::Primary,
					os::MouseButton::Middle => egui::PointerButton::Middle,
					os::MouseButton::Right => egui::PointerButton::Secondary,
				};
				events.push(egui::Event::PointerButton { pos: pos_in_points, button, pressed, modifiers: egui::Modifiers::NONE });
			},
			os::Event::MouseWheel { delta } => {
				const POINTS_PER_SCROLL_LINE: f32 = 50.0;
				events.push(egui::Event::Scroll(egui::vec2(
					delta[0] * POINTS_PER_SCROLL_LINE,
					delta[1] * POINTS_PER_SCROLL_LINE,
				)));
			},
		}
	}

	egui::RawInput {
		screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(window.size().x as f32, window.size().y as f32))),
		focused: window.is_focused(),
		events,
		..Default::default()
	}
}

pub fn set_full_output(app: &mut os::platform::App, _window: &mut os::platform::Window, output: &egui::FullOutput) {
	let os_cursor = map_cursor(output.platform_output.cursor_icon);

	app.set_cursor(&os_cursor);
}

/// Ignores special keys (backspace, delete, F1, …) and '\r', '\n', '\t'.
fn is_printable_char(chr: char) -> bool {
	let is_in_private_use_area =
		'\u{e000}' <= chr && chr <= '\u{f8ff}' ||
		'\u{f0000}' <= chr && chr <= '\u{ffffd}' ||
		'\u{100000}' <= chr && chr <= '\u{10fffd}';

	!is_in_private_use_area && !chr.is_ascii_control()
}

fn map_cursor(cursor: egui::CursorIcon) -> os::Cursor {
	use egui::CursorIcon as CU;

	match cursor {
		CU::Default => os::Cursor::Arrow,
		CU::None => os::Cursor::None,
		CU::Help => os::Cursor::Help,
		CU::PointingHand => os::Cursor::Hand,
		CU::Wait => os::Cursor::Wait,
		CU::Crosshair => os::Cursor::Crosshair,
		CU::Text => os::Cursor::Text,
		CU::Move => os::Cursor::ResizeAll,
		CU::NoDrop => os::Cursor::NotAllowed,
		CU::NotAllowed => os::Cursor::NotAllowed,
		CU::ResizeHorizontal | CU::ResizeColumn | CU::ResizeEast | CU::ResizeWest => os::Cursor::ResizeEw,
		CU::ResizeVertical | CU::ResizeRow | CU::ResizeNorth | CU::ResizeSouth => os::Cursor::ResizeNs,
		CU::ResizeNeSw | CU::ResizeNorthEast | CU::ResizeSouthWest => os::Cursor::ResizeNeSw,
		CU::ResizeNwSe | CU::ResizeNorthWest | CU::ResizeSouthEast => os::Cursor::ResizeNwSe,
		_ => os::Cursor::Arrow,
	}
}

fn map_key(key: os::Key) -> egui::Key {
	match key {
		os::Key::ArrowDown => egui::Key::ArrowDown,
		os::Key::ArrowLeft => egui::Key::ArrowLeft,
		os::Key::ArrowRight => egui::Key::ArrowRight,
		os::Key::ArrowUp => egui::Key::ArrowUp,

		os::Key::Escape => egui::Key::Escape,
		os::Key::Tab => egui::Key::Tab,
		os::Key::Backspace => egui::Key::Backspace,
		os::Key::Enter => egui::Key::Enter,
		os::Key::Space => egui::Key::Space,

		os::Key::Insert => egui::Key::Insert,
		os::Key::Delete => egui::Key::Delete,
		os::Key::Home => egui::Key::Home,
		os::Key::End => egui::Key::End,
		os::Key::PageUp => egui::Key::PageUp,
		os::Key::PageDown => egui::Key::PageDown,

		os::Key::Minus => egui::Key::Minus,
		os::Key::PlusEquals => egui::Key::PlusEquals,

		os::Key::Num0 => egui::Key::Num0,
		os::Key::Num1 => egui::Key::Num1,
		os::Key::Num2 => egui::Key::Num2,
		os::Key::Num3 => egui::Key::Num3,
		os::Key::Num4 => egui::Key::Num4,
		os::Key::Num5 => egui::Key::Num5,
		os::Key::Num6 => egui::Key::Num6,
		os::Key::Num7 => egui::Key::Num7,
		os::Key::Num8 => egui::Key::Num8,
		os::Key::Num9 => egui::Key::Num9,

		os::Key::A => egui::Key::A,
		os::Key::B => egui::Key::B,
		os::Key::C => egui::Key::C,
		os::Key::D => egui::Key::D,
		os::Key::E => egui::Key::E,
		os::Key::F => egui::Key::F,
		os::Key::G => egui::Key::G,
		os::Key::H => egui::Key::H,
		os::Key::I => egui::Key::I,
		os::Key::J => egui::Key::J,
		os::Key::K => egui::Key::K,
		os::Key::L => egui::Key::L,
		os::Key::M => egui::Key::M,
		os::Key::N => egui::Key::N,
		os::Key::O => egui::Key::O,
		os::Key::P => egui::Key::P,
		os::Key::Q => egui::Key::Q,
		os::Key::R => egui::Key::R,
		os::Key::S => egui::Key::S,
		os::Key::T => egui::Key::T,
		os::Key::U => egui::Key::U,
		os::Key::V => egui::Key::V,
		os::Key::W => egui::Key::W,
		os::Key::X => egui::Key::X,
		os::Key::Y => egui::Key::Y,
		os::Key::Z => egui::Key::Z,

		os::Key::F1 => egui::Key::F1,
		os::Key::F2 => egui::Key::F2,
		os::Key::F3 => egui::Key::F3,
		os::Key::F4 => egui::Key::F4,
		os::Key::F5 => egui::Key::F5,
		os::Key::F6 => egui::Key::F6,
		os::Key::F7 => egui::Key::F7,
		os::Key::F8 => egui::Key::F8,
		os::Key::F9 => egui::Key::F9,
		os::Key::F10 => egui::Key::F10,
		os::Key::F11 => egui::Key::F11,
		os::Key::F12 => egui::Key::F12,
		os::Key::F13 => egui::Key::F13,
		os::Key::F14 => egui::Key::F14,
		os::Key::F15 => egui::Key::F15,
		os::Key::F16 => egui::Key::F16,
		os::Key::F17 => egui::Key::F17,
		os::Key::F18 => egui::Key::F18,
		os::Key::F19 => egui::Key::F19,
		os::Key::F20 => egui::Key::F20,
	}
}
