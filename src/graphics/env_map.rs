use super::mipgen::MipGen;
use crate::gpu::{self, DeviceImpl, TextureImpl, CmdListImpl};

const RESOLUTION: usize = 512;
const SAMPLES_PER_PIXEL: usize = 64;

pub struct ImportanceMap {
	pub importance_map: gpu::Texture,
	prepare_pipeline: gpu::ComputePipeline,
	uavs: Vec<gpu::TextureView>,
	mipgen: MipGen,
	dirty: bool,
}

#[repr(C)]
struct PushConstants {
	env_map_id: u32,
	importance_map_id: u32,

	output_res: [u32; 2],
	output_res_in_samples: [u32; 2],
	num_samples: [u32; 2],
	inv_samples: f32,
}

impl ImportanceMap {
	pub fn setup(device: &mut gpu::Device, shader_compiler: &gpu::ShaderCompiler) -> Self {
		// Setup env map prepare shader.

		let shader = shader_compiler.compile("shaders/pathtracer/kernels/env-map-prepare.slang", "main");

		let descriptor_layout = gpu::DescriptorLayout {
			push_constants: Some(gpu::PushConstantBinding {
				size: std::mem::size_of::<PushConstants>() as u32,
			}),
			bindings: Some(vec![
				gpu::DescriptorBinding::bindless_srv(1),
				gpu::DescriptorBinding::bindless_uav(2),
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
		};

		let cs = device.create_shader(&gpu::ShaderDesc {
			ty: gpu::ShaderType::Compute,
			src: &shader,
		}).unwrap();

		let compute_pipeline = device.create_compute_pipeline(&gpu::ComputePipelineDesc {
			cs, descriptor_layout: &descriptor_layout,
		}).unwrap();

		// Setup importance map.

		let mip_levels = gpu::max_mip_level(RESOLUTION as u32) + 1;

		let importance_map = device.create_texture(&gpu::TextureDesc {
			width: RESOLUTION as u64,
			height: RESOLUTION as u64,
			depth: 1,
			array_size: 1,
			mip_levels,
			samples: 1,
			format: gpu::Format::R32Float,
			usage: gpu::TextureUsage::SHADER_RESOURCE | gpu::TextureUsage::UNORDERED_ACCESS,
			layout: gpu::TextureLayout::ShaderResource,
		}).unwrap();

		let uavs = (1..mip_levels).map(|i| {
			device.create_texture_view(&gpu::TextureViewDesc {
				first_mip_level: i,
				mip_level_count: 1,
			}, &importance_map)
		}).collect::<Vec<_>>();

		Self {
			importance_map,
			prepare_pipeline: compute_pipeline,
			uavs,
			mipgen: MipGen::setup(device, shader_compiler),
			dirty: true,
		}
	}

	pub fn update(&mut self, device: &gpu::Device, cmd: &mut gpu::CmdList, env_map_srv_index: u32) {
		if !self.dirty {
			return;
		}

		assert!(RESOLUTION.is_power_of_two());
		assert!(SAMPLES_PER_PIXEL.is_power_of_two());

		let dimension = RESOLUTION as u32;
		let samples = SAMPLES_PER_PIXEL as u32;

		let samples_x = (samples as f32).sqrt().max(1.0) as u32;
		let samples_y = samples / samples_x;
		assert_eq!(samples, samples_x * samples_y);

		// Transform the env map to the importance map.
		let push_constants = PushConstants {
			env_map_id: env_map_srv_index,
			importance_map_id: self.importance_map.uav_index().unwrap(),

			output_res: [dimension; 2],
			output_res_in_samples: [dimension * samples_x, dimension * samples_y],
			num_samples: [samples_x, samples_y],
			inv_samples: 1.0 / (samples_x * samples_y) as f32,
		};

		cmd.set_compute_pipeline(&self.prepare_pipeline);
		cmd.set_compute_root_table(&device, 1, 0);
		cmd.compute_push_constants(0, gpu::as_u8_slice(&push_constants));

		cmd.dispatch(dimension.div_ceil(16), dimension.div_ceil(16), 1);
		
		cmd.barriers(&[gpu::Barrier::global()]);

		// Generate mips.
		self.mipgen.generate_mips(device, cmd, &self.importance_map, dimension, &self.uavs);

		self.dirty = false;
	}

	pub fn base_mip(&self) -> u32 {
		gpu::max_mip_level(RESOLUTION as u32)
	}
}
