use gpu::{self, AccelerationStructureImpl, BufferImpl, DeviceImpl, CmdListImpl};

pub struct Blas {
	pub accel: gpu::AccelerationStructure,
	pub build_inputs: gpu::AccelerationStructureBuildInputs,
	pub buffer: gpu::Buffer,
	pub scratch_buffer: gpu::Buffer,
}

impl Blas {
	pub fn create(device: &mut gpu::Device, vertex_buffer: &gpu::Buffer, index_buffer: &gpu::Buffer, vertex_count: usize, index_count: usize, vertex_stride: usize) -> Self {
		let geo = gpu::AccelerationStructureTriangles {
			vertex_buffer: vertex_buffer.gpu_ptr(),
			vertex_format: gpu::Format::RGB32Float,
			vertex_count,
			vertex_stride,
			index_buffer: index_buffer.gpu_ptr(),
			index_format: gpu::Format::R32UInt,
			index_count,
			transform: gpu::GpuPtr::NULL,
			flags: gpu::AccelerationStructureGeometryFlags::OPAQUE,
		};

		let build_inputs = gpu::AccelerationStructureBuildInputs {
			flags: gpu::AccelerationStructureBuildFlags::PREFER_FAST_TRACE,
			entries: gpu::AccelerationStructureEntries::Triangles(vec![geo]),
		};

		let sizes = device.acceleration_structure_sizes(&build_inputs);

		let buffer = device.create_buffer(&gpu::BufferDesc {
			size: sizes.acceleration_structure_size,
			usage: gpu::BufferUsage::ACCELERATION_STRUCTURE,
			memory: gpu::Memory::GpuOnly,
		}).unwrap();

		let scratch_buffer = device.create_buffer(&gpu::BufferDesc {
			size: sizes.build_scratch_size,
			usage: gpu::BufferUsage::UNORDERED_ACCESS,
			memory: gpu::Memory::GpuOnly,
		}).unwrap();

		let accel = device.create_acceleration_structure(&gpu::AccelerationStructureDesc {
			ty: gpu::AccelerationStructureType::BottomLevel,
			buffer: &buffer,
			offset: 0,
			size: sizes.acceleration_structure_size,
		}).unwrap();

		Self {
			accel,
			build_inputs,
			buffer,
			scratch_buffer,
		}
	}

	pub fn set_vertex_buffer(&mut self, vertex_buffer: &gpu::Buffer) {
		if let gpu::AccelerationStructureEntries::Triangles(geo) = &mut self.build_inputs.entries {
			geo[0].vertex_buffer = vertex_buffer.gpu_ptr();
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
			memory: gpu::Memory::CpuToGpu,
		}).unwrap();

		let build_inputs = gpu::AccelerationStructureBuildInputs {
			flags: gpu::AccelerationStructureBuildFlags::PREFER_FAST_TRACE,
			entries: gpu::AccelerationStructureEntries::Instances(
				gpu::AccelerationStructureInstances { data: instance_buffer.gpu_ptr(), count }
			),
		};

		let sizes = device.acceleration_structure_sizes(&build_inputs);

		let buffer = device.create_buffer(&gpu::BufferDesc {
			size: sizes.acceleration_structure_size,
			usage: gpu::BufferUsage::ACCELERATION_STRUCTURE,
			memory: gpu::Memory::GpuOnly,
		}).unwrap();

		let scratch_buffer = device.create_buffer(&gpu::BufferDesc {
			size: sizes.build_scratch_size,
			usage: gpu::BufferUsage::UNORDERED_ACCESS,
			memory: gpu::Memory::GpuOnly,
		}).unwrap();

		let accel = device.create_acceleration_structure(&gpu::AccelerationStructureDesc {
			ty: gpu::AccelerationStructureType::TopLevel,
			buffer: &buffer,
			offset: 0,
			size: sizes.acceleration_structure_size,
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
