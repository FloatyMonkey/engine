mod camera;
mod editor;
mod egui_impl;
mod gizmo;
mod icons;
mod tabs;
mod windows;

use engine::*;

use crate::asset::AssetServer;
use crate::egui_impl::{EguiRenderer, ScreenDesc, get_raw_input, set_full_output};
use crate::gpu::{self, CmdListImpl, DeviceImpl, SwapChainImpl, TextureImpl};
use crate::graphics::{camera::Camera, scene::Scene, pathtracer::{Compositor, PathTracer}};
use crate::math::{Mat4, transform::Transform3};
use crate::os::{self, App, Window};
use crate::scene::setup_scene;

fn main() {
	let mut assets = AssetServer::new();

	let mut app = os::platform::App::new();

	let mut device = gpu::Device::new(&gpu::DeviceDesc {
		validation: gpu::Validation::CPU | gpu::Validation::DEBUGGER,
		power_preference: gpu::PowerPreference::HighPerformance,
	});

	let shader_compiler = gpu::ShaderCompiler::new();

	let monitor = &os::platform::App::enumerate_monitors()[0];
	let mut window = app.create_window(&os::WindowDesc {
		title: "Engine".into(),
		rect: os::Rect {
			x: monitor.rect.width / 2 - 1280 / 2,
			y: monitor.rect.height / 2 - 720 / 2,
			width: 1280,
			height: 720,
		},
	});

	window.maximize();

	let swap_chain_info = gpu::SwapChainDesc {
		size: window.size().into(),
		num_buffers: 2,
		format: gpu::Format::RGBA8UNorm,
	};

	let mut swap_chain = device.create_swap_chain(&swap_chain_info, &window.native_handle()).unwrap();
	let mut cmd = device.create_cmd_list(2);

	let mut egui_renderer = EguiRenderer::new(&mut device, &shader_compiler);

	let mut renderer = device.capabilities().raytracing.then(|| {
		let scene = Scene::new(&mut device, &shader_compiler);
		let path_tracer = PathTracer::new(&mut device, &shader_compiler);

		(scene, path_tracer)
	});

	let mut compositor = Compositor::new([1920, 1080], &mut device, &shader_compiler);
	let mut gizmo_renderer = gizmo::GizmoRenderer::new([1920, 1080], &mut device, &shader_compiler);

	let mut editor = editor::Editor::new();

	setup_scene(&mut editor.context.world, &mut assets);

	while app.run() {
		swap_chain.update(&mut device, window.size().into());
		cmd.reset(&device, &swap_chain);

		let raw_input = get_raw_input(&app, &window);

		let full_output = editor.run(raw_input);

		set_full_output(&mut app, &mut window, &full_output);

		let clipped_primitives = editor.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

		for (id, image_delta) in &full_output.textures_delta.set {
			egui_renderer.create_texture(&mut device, *id, &image_delta);
		}

		// Gizmo
		let gizmo = gizmo::Gizmo::new();

		//gizmo.line(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 3.0), 0xFF0000FF);
		//gizmo.circle(Vec3::new(0.0, 0.0, 0.0), Vec3::Z, 1.0, 0x00FF00FF);
		//gizmo.sphere(Vec3::new(0.0, 0.0, 1.0), 1.0, 0x0000FFFF);

		if let Some((camera_transform, camera)) = editor.context.world.query::<(&Transform3, &Camera)>().iter().next() {
			let view_matrix = Mat4::from(camera_transform.inv());
			let projection_matrix = camera.projection_matrix();
			let view_projection = projection_matrix * view_matrix;
			gizmo_renderer.render(&mut cmd, &gizmo, &view_projection.data);
		}

		// Path Tracer
		cmd.debug_event_push("Path Tracer", gpu::Color { r: 0, g: 0, b: 255, a: 255 });

		if let Some((scene, path_tracer)) = &mut renderer {
			scene.update(&mut editor.context.world, &assets, &mut device, &mut cmd);
			path_tracer.run(&mut cmd, &scene, 20);
			compositor.process(&mut cmd, &path_tracer.output_texture, &gizmo_renderer.texture);
		}

		editor.context.viewport_texture_srv = compositor.texture().srv_index().unwrap();

		cmd.debug_event_pop();

		// Editor
		cmd.debug_event_push("Editor", gpu::Color { r: 0, g: 0, b: 255, a: 255 });

		cmd.barriers(&gpu::Barriers::texture(&[gpu::TextureBarrier {
			texture: swap_chain.backbuffer_texture(),
			old_layout: gpu::TextureLayout::Present,
			new_layout: gpu::TextureLayout::RenderTarget,
		}]));

		cmd.render_pass_begin(&gpu::RenderPassDesc {
			depth_stencil: None,
			color_attachments: &[swap_chain.backbuffer_texture()],
			color_load: gpu::LoadOp::Clear(gpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }),
			depth_load: gpu::LoadOp::Discard,
			stencil_load: gpu::LoadOp::Discard,
		});

		egui_renderer.paint(&cmd, &clipped_primitives, &ScreenDesc {
			size_in_pixels: window.size().into(),
			pixels_per_point: full_output.pixels_per_point,
		});

		cmd.render_pass_end();

		cmd.barriers(&gpu::Barriers::texture(&[gpu::TextureBarrier {
			texture: swap_chain.backbuffer_texture(),
			old_layout: gpu::TextureLayout::RenderTarget,
			new_layout: gpu::TextureLayout::Present,
		}]));

		cmd.debug_event_pop();

		for id in &full_output.textures_delta.free {
			egui_renderer.destroy_texture(&mut device, *id);
		}

		device.submit(&cmd);

		swap_chain.swap(&device);
	}

	swap_chain.wait_for_last_frame();
}
