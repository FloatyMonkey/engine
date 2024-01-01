use crate::gpu::{self, AccelerationStructureImpl, BufferImpl, CmdListImpl, DeviceImpl, RaytracingPipelineImpl, TextureImpl};
use super::camera::GpuCamera;
use super::scene;
use rand::{RngCore, SeedableRng, rngs::StdRng};

#[repr(C)]
struct PushConstants {
	camera: GpuCamera,
	tlas_index: u32,
	combined_index: u32,
	instance_data_index: u32,
	light_data_index: u32,
	light_count: u32,
	infinite_light_count: u32,
	seed: u32,
	accumulation_factor: f32,
}

pub struct PathTracer {
	pipeline: gpu::RaytracingPipeline,
	shader_table: gpu::Buffer,
	sample_index: usize,
	rng: StdRng,

	resolution: [u32; 2],
	pub output_texture: gpu::Texture,
}

impl PathTracer {
	pub fn new(device: &mut gpu::Device, shader_compiler: &gpu::ShaderCompiler) -> Self {
		// Create the pipeline

		let shader_raygen = shader_compiler.compile("shaders/pathtracer/kernel.slang", "raygen");
		let shader_miss = shader_compiler.compile("shaders/pathtracer/kernel.slang", "miss");
		let shader_closesthit = shader_compiler.compile("shaders/pathtracer/kernel.slang", "closesthit");

		let libraries = vec![
			gpu::ShaderLibrary { ty: gpu::ShaderType::Raygen, entry: "raygen".to_string(), shader: shader_raygen },
			gpu::ShaderLibrary { ty: gpu::ShaderType::Miss, entry: "miss".to_string(), shader: shader_miss },
			gpu::ShaderLibrary { ty: gpu::ShaderType::ClosestHit, entry: "closesthit".to_string(), shader: shader_closesthit },
		];

		let hit_groups = vec![
			gpu::ShaderHitGroup {
				ty: gpu::ShaderHitGroupType::Triangles,
				name: "hit_group".to_string(),
				general: None,
				closest_hit: Some(2),
				any_hit: None,
				intersection: None, 
			},
			// TODO: Other shaders as hit groups
		];

		let descriptor_layout = gpu::DescriptorLayout {
			push_constants: Some(gpu::PushConstantBinding {
				size: std::mem::size_of::<PushConstants>() as u32,
			}),
			bindings: Some(vec![
				gpu::DescriptorBinding::bindless_srv(1), // Buffers
				gpu::DescriptorBinding::bindless_srv(2), // Acceleration structures
				gpu::DescriptorBinding::bindless_srv(3), // Textures 2D Float4
				gpu::DescriptorBinding::bindless_srv(4), // Textures 2D Float
				gpu::DescriptorBinding::bindless_uav(5), // RWTextures
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

		let pipeline_desc = gpu::RaytracingPipelineDesc {
			max_trace_recursion_depth: 3, // TODO: Too high?
			max_attribute_size: 8,
			max_payload_size: 128, // TODO: Arbitrary
			libraries,
			hit_groups,
			descriptor_layout,
		};

		let pipeline = device.create_raytracing_pipeline(&pipeline_desc).unwrap();

		// Create the shader table

		// TODO: Ensure alignment
		// Address must be aligned to 64 bytes (D3D12_RAYTRACING_SHADER_TABLE_BYTE_ALIGNMENT)
		// The stride must be aligned to 32 bytes (D3D12_RAYTRACING_SHADER_RECORD_BYTE_ALIGNMENT)
		// let shader_identifier_size = pipeline.get_shader_identifier_size();
		
		let table_alignment = 64;
		let mut shader_table_data = vec![0u8; table_alignment * 3]; // Works for now with single entry per table

		pipeline.write_shader_identifier("raygen", &mut shader_table_data[table_alignment * 0..]);
		pipeline.write_shader_identifier("miss", &mut shader_table_data[table_alignment * 1..]);
		pipeline.write_shader_identifier("hit_group", &mut shader_table_data[table_alignment * 2..]);

		let shader_table = device.create_buffer(&gpu::BufferDesc {
			size: shader_table_data.len(),
			usage: gpu::BufferUsage::empty(),
			cpu_access: gpu::CpuAccessFlags::empty(),
		}, Some(&shader_table_data)).unwrap();

		// Create the output texture

		let resolution = [1920_u32, 1080_u32]; // TODO: Hardcoded

		let output_texture_desc = gpu::TextureDesc {
			width: resolution[0] as _,
			height: resolution[1] as _,
			depth: 1,
			array_size: 1,
			mip_levels: 1,
			samples: 1,
			format: gpu::Format::RGBA32Float,
			usage: gpu::TextureUsage::SHADER_RESOURCE | gpu::TextureUsage::UNORDERED_ACCESS,
			state: gpu::ResourceState::UnorderedAccess,
		};

		let output_texture = device.create_texture(&output_texture_desc, None).unwrap();

		Self {
			pipeline,
			shader_table,
			resolution,
			output_texture,
			sample_index: 0,
			rng: StdRng::seed_from_u64(0),
			
		}
	}

	pub fn render(&mut self, device: &gpu::Device, cmd: &mut gpu::CmdList, scene: &scene::Scene) {
		let push_constants = PushConstants {
			camera: GpuCamera::from_camera(&scene.camera, &scene.camera_transform),
			tlas_index: scene.tlas.accel.srv_index(),
			combined_index: self.output_texture.uav_index().unwrap(),
			instance_data_index: scene.instance_data_buffer.srv_index().unwrap(),
			light_data_index: scene.light_data_buffer.srv_index().unwrap(),
			light_count: scene.light_count as _,
			infinite_light_count: scene.infinite_light_count as _,
			seed: self.rng.next_u32(),
			accumulation_factor: 1.0 / (self.sample_index + 1) as f32,
		};

		cmd.set_raytracing_pipeline(&self.pipeline);
		cmd.set_compute_root_table(&device, 1, 0);
		cmd.compute_push_constants(0, gpu::as_u8_slice(&push_constants));

		cmd.dispatch_rays(&gpu::DispatchRaysDesc {
			raygen: Some(gpu::ShaderTable {
				buffer: &self.shader_table,
				offset: 64 * 0,
				size: 32,
				stride: 32,
			}),
			miss: Some(gpu::ShaderTable {
				buffer: &self.shader_table,
				offset: 64 * 1,
				size: 32,
				stride: 32,
			}),
			hit_group: Some(gpu::ShaderTable {
				buffer: &self.shader_table,
				offset: 64 * 2,
				size: 32,
				stride: 32,
			}),
			callable: None,
			width: self.resolution[0],
			height: self.resolution[1],
			depth: 1,
		});

		cmd.barriers(&[gpu::Barrier::global()]);

		self.sample_index += 1;
	}

	pub fn reset(&mut self) {
		self.rng = StdRng::seed_from_u64(0);
		self.sample_index = 0;
	}
}

#[repr(C)]
struct PostProcessPushConstants {
	input_id: u32,
	output_id: u32,
	output_res: [u32; 2],
}

pub struct PostProcessor {
	res: [u32; 2],
	texture: gpu::Texture,
	pipeline: gpu::ComputePipeline,
}

impl PostProcessor {
	pub fn new(res: [u32; 2], device: &mut gpu::Device, shader_compiler: &gpu::ShaderCompiler) -> Self {
		let shader = shader_compiler.compile("shaders/postprocess.slang", "main");

		let descriptor_layout = gpu::DescriptorLayout {
			push_constants: Some(gpu::PushConstantBinding {
				size: std::mem::size_of::<PostProcessPushConstants>() as u32,
			}),
			bindings: Some(vec![
				gpu::DescriptorBinding::bindless_srv(1),
				gpu::DescriptorBinding::bindless_uav(2),
			]),
			static_samplers: None,
		};

		let cs = device.create_shader(&gpu::ShaderDesc {
			ty: gpu::ShaderType::Compute,
		}, &shader).unwrap();

		let pipeline = device.create_compute_pipeline(&gpu::ComputePipelineDesc {
			cs, descriptor_layout: &descriptor_layout,
		}).unwrap();

		let texture = device.create_texture(&gpu::TextureDesc {
			width: res[0] as _,
			height: res[1] as _,
			depth: 1,
			array_size: 1,
			mip_levels: 1,
			samples: 1,
			format: gpu::Format::RGBA32Float,
			usage: gpu::TextureUsage::SHADER_RESOURCE | gpu::TextureUsage::UNORDERED_ACCESS,
			state: gpu::ResourceState::UnorderedAccess,
		}, None).unwrap();

		Self {
			res,
			texture,
			pipeline,
		}
	}

	pub fn process(&mut self, device: &gpu::Device, cmd: &mut gpu::CmdList, input: &gpu::Texture) {
		cmd.set_compute_pipeline(&self.pipeline);
		cmd.set_compute_root_table(&device, 1, 0);

		let push_constants = PostProcessPushConstants {
			input_id: input.srv_index().unwrap(),
			output_id: self.texture.uav_index().unwrap(),
			output_res: self.res,
		};

		cmd.compute_push_constants(0, gpu::as_u8_slice(&push_constants));
		cmd.dispatch(self.res[0].div_ceil(16), self.res[1].div_ceil(16), 1);

		cmd.barriers(&[gpu::Barrier::global()]);
	}

	pub fn texture(&self) -> &gpu::Texture {
		&self.texture
	}
}
