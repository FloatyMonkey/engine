use gpu::{self, CmdListImpl, DeviceImpl, SamplerImpl, TextureImpl};

#[repr(C)]
struct PushConstants {
	input: gpu::DescriptorHandle,
	output: gpu::DescriptorHandle,
	linear_sampler: gpu::DescriptorHandle,
	output_res: [u32; 2],
	input_mip: u32,
}

pub struct MipGen {
	mipgen_pipeline: gpu::ComputePipeline,
	sampler: gpu::Sampler,
}

impl MipGen {
	pub fn setup(device: &mut gpu::Device, shader_compiler: &gpu::ShaderCompiler) -> Self {
		let shader = shader_compiler.compile("shaders/mipgen.slang", "main");

		let compute_pipeline = device
			.create_compute_pipeline(&gpu::ComputePipelineDesc {
				cs: ("main", &shader),
			})
			.unwrap();

		let sampler = device
			.create_sampler(&gpu::SamplerDesc {
				filter_min: gpu::FilterMode::Linear,
				filter_mag: gpu::FilterMode::Linear,
				filter_mip: gpu::FilterMode::Linear,
				..Default::default()
			})
			.unwrap();

		Self {
			mipgen_pipeline: compute_pipeline,
			sampler,
		}
	}

	pub fn generate_mips(
		&self,
		cmd: &gpu::CmdList,
		texture: &gpu::Texture,
		base_resolution: u32,
		uavs: &[gpu::DescriptorHandle], // Largest to smallest mip resolution, excluding the base mip.
	) {
		let mip_levels = uavs.len() + 1; // TODO: Get from texture.

		cmd.set_compute_pipeline(&self.mipgen_pipeline);

		for i in 1..mip_levels {
			let output_res = gpu::at_mip_level(base_resolution, i as u32);

			let push_constants = PushConstants {
				input: texture.srv().unwrap(),
				output: uavs[i - 1],
				linear_sampler: self.sampler.handle(),
				output_res: [output_res; 2],
				input_mip: (i - 1) as u32,
			};

			cmd.compute_push_constants(0, gpu::as_u8_slice(&push_constants));
			cmd.dispatch([output_res.div_ceil(16), output_res.div_ceil(16), 1]);

			cmd.barriers(&gpu::Barriers::global());
		}
	}
}
