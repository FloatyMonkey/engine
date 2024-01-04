use crate::gpu::{self, DeviceImpl, CmdListImpl, BufferImpl};
use crate::graphics::scene::Vertex;
use crate::geometry::mesh::{AttributeGroup, Mesh};
use crate::math::matrix::Mat3x4;

fn to_gpu_data(vertex_groups: &AttributeGroup<f32>) -> (Vec<u32>, Vec<u32>) {
	let lookup = vertex_groups.lookup.iter().map(|&i| {
		i as u32
	}).collect::<Vec<_>>();

	let values = vertex_groups.values.iter().map(|&(attribute_id, value)| {
		(attribute_id as u32) | ((value * 65535.0) as u32) << 16
	}).collect::<Vec<_>>();

	(lookup, values)
}

#[repr(C)]
struct PushConstants {
	num_vertices: u32,

	in_vertices: u32,
	out_vertices: u32,

	lookup_stream: u32,
	weight_stream: u32,
	bone_matrices: u32,
}

pub struct BoneDeform {
	num_vertices: usize,
	lookup_buffer: gpu::Buffer,
	weights_buffer: gpu::Buffer,
	bone_transforms_buffer: gpu::Buffer,
	transformed_vertex_buffer: gpu::Buffer,
	compute_pipeline: gpu::ComputePipeline,
}

impl BoneDeform {
	pub fn new(device: &mut gpu::Device, shader_compiler: &gpu::ShaderCompiler, mesh: &Mesh, bone_count: usize) -> Self {
		let shader = shader_compiler.compile("shaders/geometry/bone-deform.slang", "main");

		let descriptor_layout = gpu::DescriptorLayout {
			push_constants: Some(gpu::PushConstantBinding {
				size: std::mem::size_of::<PushConstants>() as u32,
			}),
			bindings: Some(vec![
				gpu::DescriptorBinding::bindless_srv(1),
				gpu::DescriptorBinding::bindless_uav(2),
			]),
			static_samplers: None,
		};

		let cs = device.create_shader(&gpu::ShaderDesc {
			ty: gpu::ShaderType::Compute,
			src: &shader,
		}).unwrap();

		let compute_pipeline = device.create_compute_pipeline(&gpu::ComputePipelineDesc {
			cs, descriptor_layout: &descriptor_layout,
		}).unwrap();

		let (lookup, values) = to_gpu_data(&mesh.vertex_groups);

		let lookup_buffer = device.create_buffer(&gpu::BufferDesc {
			size: std::mem::size_of::<u32>() * lookup.len(),
			usage: gpu::BufferUsage::SHADER_RESOURCE,
			memory: gpu::Memory::GpuOnly,
		}).unwrap();

		let weights_buffer = device.create_buffer(&gpu::BufferDesc {
			size: std::mem::size_of::<u32>() * values.len(),
			usage: gpu::BufferUsage::SHADER_RESOURCE,
			memory: gpu::Memory::GpuOnly,
		}).unwrap();

		device.upload_buffer(&lookup_buffer, gpu::slice_as_u8_slice(&lookup));
		device.upload_buffer(&weights_buffer, gpu::slice_as_u8_slice(&values));

		let bone_transforms_buffer = device.create_buffer(&gpu::BufferDesc {
			size: std::mem::size_of::<Mat3x4>() * bone_count,
			usage: gpu::BufferUsage::SHADER_RESOURCE,
			memory: gpu::Memory::CpuToGpu,
		}).unwrap();

		let transformed_vertex_buffer = device.create_buffer(&gpu::BufferDesc {
			size: std::mem::size_of::<Vertex>() * mesh.vertices.len(),
			usage: gpu::BufferUsage::SHADER_RESOURCE | gpu::BufferUsage::UNORDERED_ACCESS,
			memory: gpu::Memory::GpuOnly,
		}).unwrap();

		Self {
			num_vertices: mesh.vertices.len(),
			lookup_buffer,
			weights_buffer,
			bone_transforms_buffer,
			transformed_vertex_buffer,
			compute_pipeline,
		}
	}

	pub fn update_bone_transforms(&mut self, transforms: &[Mat3x4]) {
		let ptr = self.bone_transforms_buffer.cpu_ptr() as *mut Mat3x4;
		unsafe { std::ptr::copy_nonoverlapping(transforms.as_ptr(), ptr, transforms.len()); }
	}

	pub fn execute(&self, device: &gpu::Device, cmd: &gpu::CmdList, vertex_srv: u32) {
		let push_constants = PushConstants {
			num_vertices: self.num_vertices as u32,
			in_vertices: vertex_srv,
			out_vertices: self.transformed_vertex_buffer.uav_index().unwrap(),
			lookup_stream: self.lookup_buffer.srv_index().unwrap(),
			weight_stream: self.weights_buffer.srv_index().unwrap(),
			bone_matrices: self.bone_transforms_buffer.srv_index().unwrap(),
		};

		cmd.set_compute_pipeline(&self.compute_pipeline);
		cmd.set_compute_root_table(&device, 1, 0);
		cmd.compute_push_constants(0, gpu::as_u8_slice(&push_constants));

		cmd.dispatch(push_constants.num_vertices.div_ceil(32), 1, 1);
		
		cmd.barriers(&[gpu::Barrier::global()]);
	}

	pub fn get_transformed_vertex_buffer(&self) -> &gpu::Buffer {
		&self.transformed_vertex_buffer
	}
}
