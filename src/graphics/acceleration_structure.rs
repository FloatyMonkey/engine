use crate::gpu::{self, AccelerationStructureImpl, BufferImpl, DeviceImpl, CmdListImpl};

pub struct Blas {
	pub accel: gpu::AccelerationStructure,
	pub build_inputs: gpu::AccelerationStructureBuildInputs,
	pub buffer: gpu::Buffer,
	pub scratch_buffer: gpu::Buffer,
}

impl Blas {
	pub fn create(device: &mut gpu::Device, vertex_buffer: &gpu::Buffer, index_buffer: &gpu::Buffer, vertex_count: usize, index_count: usize, vertex_stride: usize) -> Self {

		let geo = gpu::AccelerationStructureTrianglesDesc {
			vertex_buffer: vertex_buffer.gpu_ptr(),
			vertex_format: gpu::Format::RGB32Float,
			vertex_count,
			vertex_stride,
			index_buffer: index_buffer.gpu_ptr(),
			index_format: gpu::Format::R32UInt,
			index_count,
			transform: gpu::GpuPtr::null(),
		};

		let build_inputs = gpu::AccelerationStructureBuildInputs {
			kind: gpu::AccelerationStructureKind::BottomLevel,
			flags: gpu::AccelerationStructureBuildFlags::PREFER_FAST_TRACE,
			instances: gpu::AccelerationStructureInstancesDesc { data: gpu::GpuPtr::null(), count: 0 },
			geometry: vec![gpu::GeometryDesc {
				flags: gpu::AccelerationStructureBottomLevelFlags::OPAQUE,
				part: gpu::GeometryPart::Triangles(geo),
			}],
		};

		let sizes = device.acceleration_structure_sizes(&build_inputs);

		let buffer = device.create_buffer(&gpu::BufferDesc {
			size: gpu::align_pow2(sizes.acceleration_structure_size as u64, 256) as usize, // TODO: hardcoded alignment
			usage: gpu::BufferUsage::ACCELERATION_STRUCTURE,
			cpu_access: gpu::CpuAccessFlags::empty(),
		}, None).unwrap();

		let scratch_buffer = device.create_buffer(&gpu::BufferDesc {
			size: gpu::align_pow2(sizes.build_scratch_buffer_size as u64, 256) as usize, // TODO: hardcoded alignment
			usage: gpu::BufferUsage::empty(),
			cpu_access: gpu::CpuAccessFlags::empty(),
		}, None).unwrap();

		let accel = device.create_acceleration_structure(&gpu::AccelerationStructureDesc {
			kind: gpu::AccelerationStructureKind::BottomLevel,
			buffer: &buffer,
			offset: 0,
		}).unwrap();

		Self {
			accel,
			build_inputs,
			buffer,
			scratch_buffer,
		}
	}

	pub fn set_vertex_buffer(&mut self, vertex_buffer: &gpu::Buffer) {
		if let gpu::GeometryPart::Triangles(geo) = &mut self.build_inputs.geometry[0].part {
			geo.vertex_buffer = vertex_buffer.gpu_ptr();
		}
	}

	pub fn build(&mut self, cmd: &gpu::CmdList) {
		cmd.build_acceleration_structure(&gpu::AccelerationStructureBuildDesc {
			inputs: &self.build_inputs,
			src: None,
			dst: &self.accel,
			scratch_data: self.scratch_buffer.gpu_ptr(),
		});
	}
}

pub struct Tlas {
	pub accel: gpu::AccelerationStructure,
	pub build_inputs: gpu::AccelerationStructureBuildInputs,
	pub buffer: gpu::Buffer,
	pub scratch_buffer: gpu::Buffer,

	pub instance_buffer: gpu::Buffer,
}

impl Tlas {
	pub fn create(device: &mut gpu::Device, count: usize) -> Self {
		let instance_descriptor_size = gpu::AccelerationStructure::instance_descriptor_size();

		let instance_buffer = device.create_buffer(&gpu::BufferDesc {
			size: instance_descriptor_size * count,
			usage: gpu::BufferUsage::SHADER_RESOURCE,
			cpu_access: gpu::CpuAccessFlags::WRITE,
		}, None).unwrap();

		let build_inputs = gpu::AccelerationStructureBuildInputs {
			kind: gpu::AccelerationStructureKind::TopLevel,
			flags: gpu::AccelerationStructureBuildFlags::PREFER_FAST_TRACE,
			instances: gpu::AccelerationStructureInstancesDesc { data: instance_buffer.gpu_ptr(), count },
			geometry: vec![],
		};

		let sizes = device.acceleration_structure_sizes(&build_inputs);

		let buffer = device.create_buffer(&gpu::BufferDesc {
			size: gpu::align_pow2(sizes.acceleration_structure_size as u64, 256) as usize, // TODO: hardcoded alignment
			usage: gpu::BufferUsage::ACCELERATION_STRUCTURE,
			cpu_access: gpu::CpuAccessFlags::empty(),
		}, None).unwrap();

		let scratch_buffer = device.create_buffer(&gpu::BufferDesc {
			size: gpu::align_pow2(sizes.build_scratch_buffer_size as u64, 256) as usize, // TODO: hardcoded alignment
			usage: gpu::BufferUsage::empty(),
			cpu_access: gpu::CpuAccessFlags::empty(),
		}, None).unwrap();

		let accel = device.create_acceleration_structure(&gpu::AccelerationStructureDesc {
			kind: gpu::AccelerationStructureKind::TopLevel,
			buffer: &buffer,
			offset: 0,
		}).unwrap();

		Self {
			accel,
			build_inputs,
			buffer,
			scratch_buffer,
			instance_buffer,
		}
	}

	pub fn build(&mut self, cmd: &gpu::CmdList) {
		cmd.build_acceleration_structure(&gpu::AccelerationStructureBuildDesc {
			inputs: &self.build_inputs,
			src: None,
			dst: &self.accel,
			scratch_data: self.scratch_buffer.gpu_ptr(),
		});
	}
}
