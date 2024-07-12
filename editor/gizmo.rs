use engine::math::{Vec2, Vec3, UnitQuaternion, Unit};
use engine::gpu::{self, BufferImpl, CmdListImpl, DeviceImpl, TextureImpl};

#[repr(C)]
struct Vertex {
	position: Vec3,
	size: f32,
	color: u32,
	// TODO: Depth/visibility based alpha
}

fn circle_iter(radius: f32, segments: usize) -> impl Iterator<Item = Vec2> {
	(0..segments).map(move |i| {
		let angle = i as f32 * std::f32::consts::TAU / segments as f32;
		let sin_cos = angle.sin_cos();
		Vec2::new(sin_cos.0, sin_cos.1) * radius
	})
}

fn arc_iter(radius: f32, segments: usize, start: f32, end: f32) -> impl Iterator<Item = Vec2> {
	(0..segments + 1).map(move |i| {
		let angle = start + (end - start) * i as f32 / segments as f32;
		let sin_cos = angle.sin_cos();
		Vec2::new(sin_cos.0, sin_cos.1) * radius
	})
}

#[derive(Clone, Copy)]
pub struct Stroke {
	pub color: u32,
	pub width: f32,
}

impl From<u32> for Stroke {
	fn from(color: u32) -> Self {
		Self { color, width: 3.0 }
	}
}

impl From<(u32, f32)> for Stroke {
	fn from((color, width): (u32, f32)) -> Self {
		Self { color, width }
	}
}

pub struct Gizmo {
	vertices: Vec<Vertex>,
}

impl Gizmo {
	pub fn new() -> Self {
		Self {
			vertices: Vec::new(),
		}
	}

	pub fn line(&mut self, start: Vec3, end: Vec3, stroke: impl Into<Stroke>) {
		let stroke = stroke.into();

		self.vertices.push(Vertex {
			position: start,
			size: stroke.width,
			color: stroke.color,
		});

		self.vertices.push(Vertex {
			position: end,
			size: stroke.width,
			color: stroke.color,
		});
	}

	pub fn line_loop(&mut self, points: impl IntoIterator<Item = Vec3>, color: u32) {
		let mut points = points.into_iter();
		let mut prev = points.next().unwrap();
		let first = prev;
		for point in points {
			self.line(prev, point, color);
			prev = point;
		}
		// Close the loop
		self.line(prev, first, color);
	}

	pub fn circle(&mut self, center: Vec3, normal: Unit<Vec3>, radius: f32, color: u32) {
		let rotation = UnitQuaternion::between(Vec3::Z, normal);
		let positions = circle_iter(radius, 64).map(|p| center + rotation * p.extend(0.0));
		self.line_loop(positions, color);
	}

	pub fn sphere(&mut self, center: Vec3, radius: f32, color: u32) {
		let positions = circle_iter(radius, 64).map(|p| center + Vec3::new(0.0, p.x, p.y));
		self.line_loop(positions, color);

		let positions = circle_iter(radius, 64).map(|p| center + Vec3::new(p.x, 0.0, p.y));
		self.line_loop(positions, color);

		let positions = circle_iter(radius, 64).map(|p| center + Vec3::new(p.x, p.y, 0.0));
		self.line_loop(positions, color);
	}

	pub fn capsule(&mut self, center: Vec3, radius: f32, height: f32, color: u32) {
		let cylinder_half_height = height / 2.0 - radius;

		let pi = std::f32::consts::PI;
		let res = 64;

		let top_arc_x = arc_iter(radius, res, - pi / 2.0, pi / 2.0).map(|p| center + Vec3::new(0.0, p.x, p.y + cylinder_half_height));
		let bottom_arc_x = arc_iter(radius, res, pi / 2.0, 3.0 * pi / 2.0).map(|p| center + Vec3::new(0.0, p.x, p.y - cylinder_half_height));

		let positions = top_arc_x.chain(bottom_arc_x);
		self.line_loop(positions, color);

		let top_arc_y = arc_iter(radius, res, - pi / 2.0, pi / 2.0).map(|p| center + Vec3::new(p.x, 0.0, p.y + cylinder_half_height));
		let bottom_arc_y = arc_iter(radius, res, pi / 2.0, 3.0 * pi / 2.0).map(|p| center + Vec3::new(p.x, 0.0, p.y - cylinder_half_height));

		let positions = top_arc_y.chain(bottom_arc_y);
		self.line_loop(positions, color);

		let positions = circle_iter(radius, res).map(|p| center + Vec3::new(p.x, p.y, cylinder_half_height));
		self.line_loop(positions, color);

		let positions = circle_iter(radius, res).map(|p| center + Vec3::new(p.x, p.y, -cylinder_half_height));
		self.line_loop(positions, color);
	}

	pub fn cylinder(&mut self, center: Vec3, radius: f32, height: f32, color: u32) {
		let half_height = height / 2.0;

		let res = 64;

		let positions = circle_iter(radius, res).map(|p| center + Vec3::new(p.x, p.y, half_height));
		self.line_loop(positions, color);

		let positions = circle_iter(radius, res).map(|p| center + Vec3::new(p.x, p.y, -half_height));
		self.line_loop(positions, color);

		for (a, b) in circle_iter(radius, 4).map(|p| ((center + Vec3::new(p.x, p.y, -half_height)), (center + Vec3::new(p.x, p.y, half_height)))) {
			self.line(a, b, color);
		}
	}
}

// TODO: Dynamically reallocate
const VERTEX_BUFFER_SIZE: usize = 1024 * 1024;

#[repr(C)]
struct PushConstants {
	view_projection: [[f32; 4]; 4],
	screen_size: [f32; 2],
	vb_index: u32,
	depth_texture_id: u32,
}

pub struct GizmoRenderer {
	resolution: [u32; 2],
	vb: gpu::Buffer,
	pipeline: gpu::GraphicsPipeline,
	pub texture: gpu::Texture,
}

impl GizmoRenderer {
	pub fn new(resolution: [u32; 2], device: &mut gpu::Device, shader_compiler: &gpu::ShaderCompiler) -> Self {
		let vb = device.create_buffer(&gpu::BufferDesc {
			size: std::mem::size_of::<egui::epaint::Vertex>() * VERTEX_BUFFER_SIZE,
			usage: gpu::BufferUsage::SHADER_RESOURCE,
			memory: gpu::Memory::CpuToGpu,
		}).unwrap();

		let vertex_shader = shader_compiler.compile("shaders/editor/gizmo.slang", "main_vs");
		let pixel_shader = shader_compiler.compile("shaders/editor/gizmo.slang", "main_ps");

		let pipeline_desc = gpu::GraphicsPipelineDesc {
			vs: Some(&vertex_shader),
			ps: Some(&pixel_shader),
			descriptor_layout: gpu::DescriptorLayout {
				push_constants: Some(gpu::PushConstantBinding {
					size: std::mem::size_of::<PushConstants>() as u32,
				}),
				bindings: Some(vec![
					gpu::DescriptorBinding::bindless_srv(1), // buffers
					gpu::DescriptorBinding::bindless_srv(2), // textures
				]),
				static_samplers: Some(vec![
					gpu::SamplerBinding {
						shader_register: 0,
						register_space: 0,
						sampler_desc: gpu::SamplerDesc {
							filter_min: gpu::FilterMode::Linear,
							filter_mag: gpu::FilterMode::Linear,
							filter_mip: gpu::FilterMode::Linear,
							..Default::default()
						},
					},
				]),
			},
			rasterizer: gpu::RasterizerDesc::default(),
			depth_stencil: gpu::DepthStencilDesc::default(),
			color_attachments: &[gpu::ColorAttachment {
				format: gpu::Format::RGBA8UNorm, // TODO: Hardcoded
				blend: Some(gpu::BlendDesc {
					src_color: gpu::BlendFactor::One,
					dst_color: gpu::BlendFactor::InvSrcAlpha,
					color_op: gpu::BlendOp::Add,
					src_alpha: gpu::BlendFactor::One,
					dst_alpha: gpu::BlendFactor::InvSrcAlpha,
					alpha_op: gpu::BlendOp::Add,
				}),
				write_mask: gpu::ColorWriteMask::ALL,
			}],
			topology: gpu::Topology::TriangleStrip,
		};

		let pipeline = device.create_graphics_pipeline(&pipeline_desc).unwrap();

		let texture = device.create_texture(&gpu::TextureDesc {
			width: resolution[0] as _,
			height: resolution[1] as _,
			depth: 1,
			array_size: 1,
			mip_levels: 1,
			format: gpu::Format::RGBA8UNorm,
			usage: gpu::TextureUsage::SHADER_RESOURCE | gpu::TextureUsage::RENDER_TARGET,
			layout: gpu::TextureLayout::ShaderResource,
		}).unwrap();

		Self {
			resolution,
			vb,
			pipeline,
			texture,
		}
	}

	pub fn render(&mut self, cmd: &mut gpu::CmdList, gizmo: &Gizmo, view_projection: &[[f32; 4]; 4], depth_texture: &gpu::Texture) {
		let map_vb = self.vb.cpu_ptr() as *mut Vertex;
		unsafe {
			std::ptr::copy_nonoverlapping(gizmo.vertices.as_ptr(), map_vb, gizmo.vertices.len());
		}

		let push_constants = PushConstants {
			view_projection: *view_projection,
			screen_size: [self.resolution[0] as f32, self.resolution[1] as f32],
			vb_index: self.vb.srv_index().unwrap(),
			depth_texture_id: depth_texture.srv_index().unwrap(),
		};

		cmd.set_graphics_pipeline(&self.pipeline);
		cmd.graphics_push_constants(0, gpu::as_u8_slice(&push_constants));

		cmd.barriers(&gpu::Barriers::texture(&[gpu::TextureBarrier {
			texture: &self.texture,
			old_layout: gpu::TextureLayout::ShaderResource,
			new_layout: gpu::TextureLayout::RenderTarget,
		}]));

		cmd.render_pass_begin(&gpu::RenderPassDesc {
			colors: &[gpu::RenderTarget {
				texture: &self.texture,
				load_op: gpu::LoadOp::Clear(gpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }),
				store_op: gpu::StoreOp::Store,
			}],
			depth_stencil: None,
		});

		let rect = gpu::Rect::from_size(self.resolution);

		cmd.set_viewport(&rect.into(), 0.0..1.0);
		cmd.set_scissor(&rect);

		cmd.draw(0..4, 0..gizmo.vertices.len() as u32 / 2);

		cmd.render_pass_end();

		cmd.barriers(&gpu::Barriers::texture(&[gpu::TextureBarrier {
			texture: &self.texture,
			old_layout: gpu::TextureLayout::RenderTarget,
			new_layout: gpu::TextureLayout::ShaderResource,
		}]));
	}
}
