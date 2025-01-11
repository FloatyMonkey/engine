use gpu::{self, DeviceImpl, TextureImpl, CmdListImpl};

#[repr(C)]
struct PushConstants {
	output_id: u32,
	output_res: [u32; 2],
	input_id: u32,
	input_mip: u32,
}

pub struct MipGen {
	mipgen_pipeline: gpu::ComputePipeline,
}

impl MipGen {
	pub fn setup(device: &mut gpu::Device, shader_compiler: &gpu::ShaderCompiler) -> Self {
		let shader = shader_compiler.compile("shaders/mipgen.slang", "main");

		let descriptor_layout = gpu::DescriptorLayout {
			push_constants: Some(gpu::PushConstantBinding {
				size: size_of::<PushConstants>() as u32,
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

		let compute_pipeline = device.create_compute_pipeline(&gpu::ComputePipelineDesc {
			cs: &shader,
			descriptor_layout: &descriptor_layout,
		}).unwrap();

		Self {
			mipgen_pipeline: compute_pipeline,
		}
	}

	pub fn generate_mips(
		&self,
		cmd: &gpu::CmdList,
		texture: &gpu::Texture,
		base_resolution: u32,
		uavs: &[gpu::TextureView], // Largest to smallest mip resolution, excluding the base mip.
	) {
		let mip_levels = uavs.len() + 1; // TODO: Get from texture.

		cmd.set_compute_pipeline(&self.mipgen_pipeline);

		for i in 1..mip_levels {
			let output_res = gpu::at_mip_level(base_resolution, i as u32);

			let push_constants = PushConstants {
				output_id: uavs[i - 1].index,
				output_res: [output_res; 2],
				input_id: texture.srv_index().unwrap(),
				input_mip: (i - 1) as u32,
			};

			cmd.compute_push_constants(0, gpu::as_u8_slice(&push_constants));
			cmd.dispatch([output_res.div_ceil(16), output_res.div_ceil(16), 1]);

			cmd.barriers(&gpu::Barriers::global());
		}
	}
}
