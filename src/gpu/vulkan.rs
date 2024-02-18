#![allow(dead_code)]
#![allow(unused_variables)]

use crate::os::NativeHandle;

use std::{ffi::CString, ops::Range};

use ash::vk;

fn map_offset(offset: &[u32; 3]) -> vk::Offset3D {
	vk::Offset3D {
		x: offset[0] as _,
		y: offset[1] as _,
		z: offset[2] as _,
	}
}

fn map_extent(extent: &[u32; 3]) -> vk::Extent3D {
	vk::Extent3D {
		width: extent[0],
		height: extent[1],
		depth: extent[2],
	}
}

fn map_format(format: super::Format) -> vk::Format {
	match format {
		super::Format::Unknown => vk::Format::UNDEFINED,

		super::Format::R8UNorm => vk::Format::R8_UNORM,
		super::Format::R8SNorm => vk::Format::R8_SNORM,
		super::Format::R8UInt  => vk::Format::R8_UINT,
		super::Format::R8SInt  => vk::Format::R8_SINT,

		super::Format::R16UNorm => vk::Format::R16_UNORM,
		super::Format::R16SNorm => vk::Format::R16_SNORM,
		super::Format::R16UInt  => vk::Format::R16_UINT,
		super::Format::R16SInt  => vk::Format::R16_SINT,
		super::Format::R16Float => vk::Format::R16_SFLOAT,

		super::Format::R32UInt  => vk::Format::R32_UINT,
		super::Format::R32SInt  => vk::Format::R32_SINT,
		super::Format::R32Float => vk::Format::R32_SFLOAT,

		super::Format::RG8UNorm => vk::Format::R8G8_UNORM,
		super::Format::RG8SNorm => vk::Format::R8G8_SNORM,
		super::Format::RG8UInt  => vk::Format::R8G8_UINT,
		super::Format::RG8SInt  => vk::Format::R8G8_SINT,

		super::Format::RG16UNorm => vk::Format::R16G16_UNORM,
		super::Format::RG16SNorm => vk::Format::R16G16_SNORM,
		super::Format::RG16UInt  => vk::Format::R16G16_UINT,
		super::Format::RG16SInt  => vk::Format::R16G16_SINT,
		super::Format::RG16Float => vk::Format::R16G16_SFLOAT,

		super::Format::RG32UInt  => vk::Format::R32G32_UINT,
		super::Format::RG32SInt  => vk::Format::R32G32_SINT,
		super::Format::RG32Float => vk::Format::R32G32_SFLOAT,

		super::Format::RGB32UInt  => vk::Format::R32G32B32_UINT,
		super::Format::RGB32SInt  => vk::Format::R32G32B32_SINT,
		super::Format::RGB32Float => vk::Format::R32G32B32_SFLOAT,

		super::Format::RGBA8UNorm => vk::Format::R8G8B8A8_UNORM,
		super::Format::RGBA8SNorm => vk::Format::R8G8B8A8_SNORM,
		super::Format::RGBA8UInt  => vk::Format::R8G8B8A8_UINT,
		super::Format::RGBA8SInt  => vk::Format::R8G8B8A8_SINT,

		super::Format::RGBA16UNorm => vk::Format::R16G16B16A16_UNORM,
		super::Format::RGBA16SNorm => vk::Format::R16G16B16A16_SNORM,
		super::Format::RGBA16UInt  => vk::Format::R16G16B16A16_UINT,
		super::Format::RGBA16SInt  => vk::Format::R16G16B16A16_SINT,
		super::Format::RGBA16Float => vk::Format::R16G16B16A16_SFLOAT,

		super::Format::RGBA32UInt  => vk::Format::R32G32B32A32_UINT,
		super::Format::RGBA32SInt  => vk::Format::R32G32B32A32_SINT,
		super::Format::RGBA32Float => vk::Format::R32G32B32A32_SFLOAT,

		super::Format::BGRA8UNorm => vk::Format::B8G8R8A8_UNORM,

		super::Format::D16UNorm          => vk::Format::D16_UNORM,
		super::Format::D24UNormS8UInt    => vk::Format::D24_UNORM_S8_UINT,
		super::Format::D32Float          => vk::Format::D32_SFLOAT,
		super::Format::D32FloatS8UIntX24 => vk::Format::D32_SFLOAT_S8_UINT,
	}
}

fn map_index_format(format: super::Format) -> vk::IndexType {
	match format {
		super::Format::Unknown => vk::IndexType::NONE_KHR,
		super::Format::R8UInt  => vk::IndexType::UINT8_EXT,
		super::Format::R16UInt => vk::IndexType::UINT16,
		super::Format::R32UInt => vk::IndexType::UINT32,
		_ => panic!(),
	}
}

fn map_filter_mode(filter_mode: super::FilterMode) -> vk::Filter {
	match filter_mode {
		super::FilterMode::Nearest => vk::Filter::NEAREST,
		super::FilterMode::Linear  => vk::Filter::LINEAR,
	}
}

fn map_mip_filter_mode(filter_mode: super::FilterMode) -> vk::SamplerMipmapMode {
	match filter_mode {
		super::FilterMode::Nearest => vk::SamplerMipmapMode::NEAREST,
		super::FilterMode::Linear  => vk::SamplerMipmapMode::LINEAR,
	}
}

fn map_address_mode(address_mode: super::AddressMode) -> vk::SamplerAddressMode {
	match address_mode {
		super::AddressMode::Clamp      => vk::SamplerAddressMode::CLAMP_TO_EDGE,
		super::AddressMode::Repeat     => vk::SamplerAddressMode::REPEAT,
		super::AddressMode::Mirror     => vk::SamplerAddressMode::MIRRORED_REPEAT,
		super::AddressMode::MirrorOnce => vk::SamplerAddressMode::MIRROR_CLAMP_TO_EDGE,
		super::AddressMode::Border     => vk::SamplerAddressMode::CLAMP_TO_BORDER,
	}
}

fn map_border_color(border_color: super::BorderColor) -> vk::BorderColor {
	match border_color {
		super::BorderColor::TransparentBlack => vk::BorderColor::FLOAT_TRANSPARENT_BLACK,
		super::BorderColor::OpaqueBlack      => vk::BorderColor::FLOAT_OPAQUE_BLACK,
		super::BorderColor::White            => vk::BorderColor::FLOAT_OPAQUE_WHITE,
	}
}

fn map_image_type(desc: &super::TextureDesc) -> vk::ImageType {
	if desc.depth > 1 { return vk::ImageType::TYPE_3D; }
	if desc.height > 1 { return vk::ImageType::TYPE_2D; }
	return vk::ImageType::TYPE_1D;
}

fn map_image_layout(layout: super::TextureLayout) -> vk::ImageLayout {
	match layout {
		super::TextureLayout::Common            => vk::ImageLayout::GENERAL,
		super::TextureLayout::Present           => vk::ImageLayout::PRESENT_SRC_KHR,
		super::TextureLayout::CopySrc           => vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
		super::TextureLayout::CopyDst           => vk::ImageLayout::TRANSFER_DST_OPTIMAL,
		super::TextureLayout::ShaderResource    => vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
		super::TextureLayout::UnorderedAccess   => vk::ImageLayout::GENERAL,
		super::TextureLayout::RenderTarget      => vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
		super::TextureLayout::DepthStencilWrite => vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
		super::TextureLayout::DepthStencilRead  => vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL,
	}
}

fn map_buffer_usage_flags(usage: super::BufferUsage) -> vk::BufferUsageFlags {
	let mut vk_flags = vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST;

	if usage.contains(super::BufferUsage::INDEX)                  { vk_flags |= vk::BufferUsageFlags::INDEX_BUFFER; }
	if usage.contains(super::BufferUsage::SHADER_RESOURCE)        { vk_flags |= vk::BufferUsageFlags::UNIFORM_TEXEL_BUFFER; }
	if usage.contains(super::BufferUsage::UNORDERED_ACCESS)       { vk_flags |= vk::BufferUsageFlags::STORAGE_TEXEL_BUFFER; }
	if usage.contains(super::BufferUsage::ACCELERATION_STRUCTURE) {
		vk_flags |= vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR;
		vk_flags |= vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR;
		vk_flags |= vk::BufferUsageFlags::SHADER_BINDING_TABLE_KHR;
	}

	vk_flags
}

fn map_image_usage_flags(usage: super::TextureUsage) -> vk::ImageUsageFlags {
	let mut vk_flags = vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST;

	if usage.contains(super::TextureUsage::RENDER_TARGET)    { vk_flags |= vk::ImageUsageFlags::COLOR_ATTACHMENT; }
	if usage.contains(super::TextureUsage::DEPTH_STENCIL)    { vk_flags |= vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT; }
	if usage.contains(super::TextureUsage::SHADER_RESOURCE)  { vk_flags |= vk::ImageUsageFlags::SAMPLED; }
	if usage.contains(super::TextureUsage::UNORDERED_ACCESS) { vk_flags |= vk::ImageUsageFlags::STORAGE; }

	vk_flags
}

fn map_compare_op(compare_op: super::CompareOp) -> vk::CompareOp {
	match compare_op {
		super::CompareOp::Never        => vk::CompareOp::NEVER,
		super::CompareOp::Always       => vk::CompareOp::ALWAYS,
		super::CompareOp::Equal        => vk::CompareOp::EQUAL,
		super::CompareOp::NotEqual     => vk::CompareOp::NOT_EQUAL,
		super::CompareOp::Less         => vk::CompareOp::LESS,
		super::CompareOp::LessEqual    => vk::CompareOp::LESS_OR_EQUAL,
		super::CompareOp::Greater      => vk::CompareOp::GREATER,
		super::CompareOp::GreaterEqual => vk::CompareOp::GREATER_OR_EQUAL,
	}
}

fn map_topology(topology: super::Topology) -> vk::PrimitiveTopology {
	match topology {
		super::Topology::PointList        => vk::PrimitiveTopology::POINT_LIST,
		super::Topology::LineList         => vk::PrimitiveTopology::LINE_LIST,
		super::Topology::LineStrip        => vk::PrimitiveTopology::LINE_STRIP,
		super::Topology::TriangleList     => vk::PrimitiveTopology::TRIANGLE_LIST,
		super::Topology::TriangleStrip    => vk::PrimitiveTopology::TRIANGLE_STRIP,
	}
}

fn map_stencil_op(stencil_op: &super::StencilOp) -> vk::StencilOp {
	match stencil_op {
		super::StencilOp::Keep           => vk::StencilOp::KEEP,
		super::StencilOp::Zero           => vk::StencilOp::ZERO,
		super::StencilOp::Replace        => vk::StencilOp::REPLACE,
		super::StencilOp::Invert         => vk::StencilOp::INVERT,
		super::StencilOp::IncrementWrap  => vk::StencilOp::INCREMENT_AND_WRAP,
		super::StencilOp::IncrementClamp => vk::StencilOp::INCREMENT_AND_CLAMP,
		super::StencilOp::DecrementWrap  => vk::StencilOp::DECREMENT_AND_WRAP,
		super::StencilOp::DecrementClamp => vk::StencilOp::DECREMENT_AND_CLAMP,
	}
}

fn map_polygon_mode(polygon_mode: &super::PolygonMode) -> vk::PolygonMode {
	match polygon_mode {
		super::PolygonMode::Line => vk::PolygonMode::LINE,
		super::PolygonMode::Fill => vk::PolygonMode::FILL,
	}
}

fn map_cull_mode(cull_mode: &super::CullMode) -> vk::CullModeFlags {
	match cull_mode {
		super::CullMode::None  => vk::CullModeFlags::NONE,
		super::CullMode::Front => vk::CullModeFlags::FRONT,
		super::CullMode::Back  => vk::CullModeFlags::BACK,
	}
}

fn map_blend_factor(blend_factor: &super::BlendFactor) -> vk::BlendFactor {
	match blend_factor {
		super::BlendFactor::Zero             => vk::BlendFactor::ZERO,
		super::BlendFactor::One              => vk::BlendFactor::ONE,
		super::BlendFactor::SrcColor         => vk::BlendFactor::SRC_COLOR,
		super::BlendFactor::InvSrcColor      => vk::BlendFactor::ONE_MINUS_SRC_COLOR,
		super::BlendFactor::SrcAlpha         => vk::BlendFactor::SRC_ALPHA,
		super::BlendFactor::InvSrcAlpha      => vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
		super::BlendFactor::DstColor         => vk::BlendFactor::DST_COLOR,
		super::BlendFactor::InvDstColor      => vk::BlendFactor::ONE_MINUS_DST_COLOR,
		super::BlendFactor::DstAlpha         => vk::BlendFactor::DST_ALPHA,
		super::BlendFactor::InvDstAlpha      => vk::BlendFactor::ONE_MINUS_DST_ALPHA,
		super::BlendFactor::Src1Color        => vk::BlendFactor::SRC1_COLOR,
		super::BlendFactor::InvSrc1Color     => vk::BlendFactor::ONE_MINUS_SRC1_COLOR,
		super::BlendFactor::Src1Alpha        => vk::BlendFactor::SRC1_ALPHA,
		super::BlendFactor::InvSrc1Alpha     => vk::BlendFactor::ONE_MINUS_SRC1_ALPHA,
		super::BlendFactor::SrcAlphaSat      => vk::BlendFactor::SRC_ALPHA_SATURATE,
		super::BlendFactor::ConstantColor    => vk::BlendFactor::CONSTANT_COLOR,
		super::BlendFactor::InvConstantColor => vk::BlendFactor::ONE_MINUS_CONSTANT_COLOR,
	}
}

fn map_blend_op(blend_op: &super::BlendOp) -> vk::BlendOp {
	match blend_op {
		super::BlendOp::Add         => vk::BlendOp::ADD,
		super::BlendOp::Subtract    => vk::BlendOp::SUBTRACT,
		super::BlendOp::RevSubtract => vk::BlendOp::REVERSE_SUBTRACT,
		super::BlendOp::Min         => vk::BlendOp::MIN,
		super::BlendOp::Max         => vk::BlendOp::MAX,
	}
}

fn map_acceleration_structure_flags(flags: super::AccelerationStructureBuildFlags) -> vk::BuildAccelerationStructureFlagsKHR {
	let mut vk_flags = vk::BuildAccelerationStructureFlagsKHR::empty();
	
	if flags.contains(super::AccelerationStructureBuildFlags::ALLOW_UPDATE)      { vk_flags |= vk::BuildAccelerationStructureFlagsKHR::ALLOW_UPDATE; }
	if flags.contains(super::AccelerationStructureBuildFlags::ALLOW_COMPACTION)  { vk_flags |= vk::BuildAccelerationStructureFlagsKHR::ALLOW_COMPACTION; }
	if flags.contains(super::AccelerationStructureBuildFlags::PREFER_FAST_TRACE) { vk_flags |= vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE; }
	if flags.contains(super::AccelerationStructureBuildFlags::PREFER_FAST_BUILD) { vk_flags |= vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_BUILD; }
	if flags.contains(super::AccelerationStructureBuildFlags::MINIMIZE_MEMORY)   { vk_flags |= vk::BuildAccelerationStructureFlagsKHR::LOW_MEMORY; }

	vk_flags
}

fn map_acceleration_structure_bottom_level_flags(flags: super::AccelerationStructureBottomLevelFlags) -> vk::GeometryFlagsKHR {
	let mut vk_flags = vk::GeometryFlagsKHR::empty();

	if flags.contains(super::AccelerationStructureBottomLevelFlags::OPAQUE)                          { vk_flags |= vk::GeometryFlagsKHR::OPAQUE; }
	if flags.contains(super::AccelerationStructureBottomLevelFlags::NO_DUPLICATE_ANY_HIT_INVOCATION) { vk_flags |= vk::GeometryFlagsKHR::NO_DUPLICATE_ANY_HIT_INVOCATION; }

	vk_flags
}

fn map_acceleration_structure_instance_flags(flags: super::AccelerationStructureInstanceFlags) -> vk::GeometryInstanceFlagsKHR {
	let mut vk_flags = vk::GeometryInstanceFlagsKHR::empty();

	if flags.contains(super::AccelerationStructureInstanceFlags::TRIANGLE_CULL_DISABLE) { vk_flags |= vk::GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE; }
	if flags.contains(super::AccelerationStructureInstanceFlags::TRIANGLE_FRONT_CCW)    { vk_flags |= vk::GeometryInstanceFlagsKHR::TRIANGLE_FRONT_COUNTERCLOCKWISE; }
	if flags.contains(super::AccelerationStructureInstanceFlags::FORCE_OPAQUE)          { vk_flags |= vk::GeometryInstanceFlagsKHR::FORCE_OPAQUE; }
	if flags.contains(super::AccelerationStructureInstanceFlags::FORCE_NON_OPAQUE)      { vk_flags |= vk::GeometryInstanceFlagsKHR::FORCE_NO_OPAQUE; }

	vk_flags
}

struct SwapChain {}

impl super::SwapChainImpl<Device> for SwapChain {
	fn update(&mut self, device: &mut Device, size: [u32; 2]) {
		todo!()
	}

	fn wait_for_last_frame(&mut self) {
		todo!()
	}

	fn num_buffers(&self) -> u32 {
		todo!()
	}

	fn backbuffer_index(&self) -> u32 {
		todo!()
	}

	fn backbuffer_texture(&self) -> &Texture {
		todo!()
	}

	fn swap(&mut self, device: &Device) {
		todo!()
	}
}

struct Buffer {
	buffer: vk::Buffer,
}

impl super::BufferImpl<Device> for Buffer {
	fn srv_index(&self) -> Option<u32> {
		todo!()
	}

	fn uav_index(&self) -> Option<u32> {
		todo!()
	}
	
	fn cpu_ptr(&self) -> *mut u8 {
		todo!()
	}

	fn gpu_ptr(&self) -> super::GpuPtr {
		todo!()
	}
}

struct Texture {
	image: vk::Image,
}

impl super::TextureImpl<Device> for Texture {
	fn srv_index(&self) -> Option<u32> {
		todo!()
	}

	fn uav_index(&self) -> Option<u32> {
		todo!()
	}
}

struct Sampler {
	sampler: vk::Sampler,
}

impl super::SamplerImpl<Device> for Sampler {}

struct AccelerationStructure {}

impl super::AccelerationStructureImpl<Device> for AccelerationStructure {
	fn srv_index(&self) -> Option<u32> {
		todo!()
	}

	fn instance_descriptor_size() -> usize {
		std::mem::size_of::<vk::AccelerationStructureInstanceKHR>()
	}

	fn write_instance_descriptor(instance: &super::AccelerationStructureInstance, slice: &mut [u8]) {
		let t = &instance.transform;

		let vk_instance = vk::AccelerationStructureInstanceKHR {
			transform: vk::TransformMatrixKHR {
				matrix: [
					t[0][0], t[0][1], t[0][2], t[0][3],
					t[1][0], t[1][1], t[1][2], t[1][3],
					t[2][0], t[2][1], t[2][2], t[2][3],
				],
			},
			instance_custom_index_and_mask: vk::Packed24_8::new(
				instance.user_id,
				instance.mask,
			),
			instance_shader_binding_table_record_offset_and_flags: vk::Packed24_8::new(
				instance.contribution_to_hit_group_index,
				map_acceleration_structure_instance_flags(instance.flags).as_raw() as _,
			),
			acceleration_structure_reference: vk::AccelerationStructureReferenceKHR {
				device_handle: instance.bottom_level.0,
			},
		};

		unsafe {
			std::ptr::copy_nonoverlapping(&vk_instance as *const _ as _, slice.as_mut_ptr(), std::mem::size_of::<vk::AccelerationStructureInstanceKHR>());
		}
	}
}

struct GraphicsPipeline {
	pipeline: vk::Pipeline,
}

impl super::GraphicsPipelineImpl<Device> for GraphicsPipeline {
	
}

struct ComputePipeline {
	pipeline: vk::Pipeline,
}

impl super::ComputePipelineImpl<Device> for ComputePipeline {
	
}

struct RaytracingPipeline {
	pipeline: vk::Pipeline,
}

impl super::RaytracingPipelineImpl<Device> for RaytracingPipeline {
	fn shader_identifier_size(&self) -> usize {
		todo!()
	}

	fn write_shader_identifier(&self, name: &str, slice: &mut [u8]) {
		todo!()
	}
}

struct Device {
	device: ash::Device,
}

impl super::DeviceImpl for Device {
	type SwapChain = SwapChain;
	type CmdList = CmdList;
	type Buffer = Buffer;
	type Texture = Texture;
	type Sampler = Sampler;
	type AccelerationStructure = AccelerationStructure;
	type GraphicsPipeline = GraphicsPipeline;
	type ComputePipeline = ComputePipeline;
	type RaytracingPipeline = RaytracingPipeline;

	fn new(desc: &super::DeviceDesc) -> Self {
		todo!()
	}

	fn create_swap_chain(&mut self, desc: &super::SwapChainDesc, window_handle: &NativeHandle) -> Result<Self::SwapChain, super::Error> {
		todo!()
	}

	fn create_cmd_list(&self, num_buffers: u32) -> Self::CmdList {
		todo!()
	}

	fn create_buffer(&mut self, desc: &super::BufferDesc) -> Result<Self::Buffer, super::Error> {
		todo!()
	}

	fn create_texture(&mut self, desc: &super::TextureDesc) -> Result<Self::Texture, super::Error> {
		let create_info = vk::ImageCreateInfo::builder()
			.image_type(map_image_type(&desc))
			.format(map_format(desc.format))
			.extent(vk::Extent3D {
				width: desc.width as _,
				height: desc.height as _,
				depth: desc.depth,
			})
			.mip_levels(desc.mip_levels)
			.array_layers(desc.array_size)
			.samples(vk::SampleCountFlags::TYPE_1)
			.tiling(vk::ImageTiling::OPTIMAL)
			.usage(map_image_usage_flags(desc.usage))
			.sharing_mode(vk::SharingMode::EXCLUSIVE)
			.initial_layout(map_image_layout(desc.layout));

		let image = unsafe { self.device.create_image(&create_info, None) }.unwrap();

		// TODO: Attach memory

		Ok(Texture { image })
	}

	fn create_sampler(&mut self, desc: &super::SamplerDesc) -> Result<Self::Sampler, super::Error> {
		let mut create_info = vk::SamplerCreateInfo::builder()
			.address_mode_u(map_address_mode(desc.address_u))
			.address_mode_v(map_address_mode(desc.address_v))
			.address_mode_w(map_address_mode(desc.address_w))
			.min_filter(map_filter_mode(desc.filter_min))
			.mag_filter(map_filter_mode(desc.filter_mag))
			.mipmap_mode(map_mip_filter_mode(desc.filter_mip))
			.min_lod(desc.min_lod)
			.max_lod(desc.max_lod)
			.mip_lod_bias(desc.lod_bias)
			.anisotropy_enable(desc.max_anisotropy > 1)
			.max_anisotropy(desc.max_anisotropy as f32);

		if let Some(compare) = desc.compare {
			create_info = create_info
				.compare_enable(true)
				.compare_op(map_compare_op(compare));
		}

		if let Some(border_color) = desc.border_color {
			create_info = create_info
				.border_color(map_border_color(border_color));
		}

		let sampler = unsafe { self.device.create_sampler(&create_info, None) }.unwrap();

		Ok(Sampler { sampler })
	}

	fn create_acceleration_structure(&mut self, desc: &super::AccelerationStructureDesc<Self>) -> Result<Self::AccelerationStructure, super::Error> {
		todo!()
	}

	fn create_graphics_pipeline(&self, desc: &super::GraphicsPipelineDesc) -> Result<Self::GraphicsPipeline, super::Error> {
		let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
			.topology(map_topology(desc.topology));

		let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
			.viewport_count(1)
			.scissor_count(1);

		let mut rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
			.depth_clamp_enable(true)
			.polygon_mode(map_polygon_mode(&desc.rasterizer.polygon_mode))
			.cull_mode(map_cull_mode(&desc.rasterizer.cull_mode))
			.front_face(if desc.rasterizer.front_ccw { vk::FrontFace::COUNTER_CLOCKWISE } else { vk::FrontFace::CLOCKWISE })
			.line_width(1.0);

		let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
			.rasterization_samples(vk::SampleCountFlags::TYPE_1)
			.sample_mask(&[1]);

		let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
			.depth_test_enable(desc.depth_stencil.depth_test_enable)
			.depth_write_enable(desc.depth_stencil.depth_write_enable)
			.depth_compare_op(map_compare_op(desc.depth_stencil.depth_op))
			.stencil_test_enable(desc.depth_stencil.stencil_enable)
			.front(vk::StencilOpState {
				fail_op: map_stencil_op(&desc.depth_stencil.front_face.fail),
				pass_op: map_stencil_op(&desc.depth_stencil.front_face.pass),
				depth_fail_op: map_stencil_op(&desc.depth_stencil.front_face.depth_fail),
				compare_op: map_compare_op(desc.depth_stencil.front_face.func),
				compare_mask: desc.depth_stencil.stencil_read_mask as _,
				write_mask: desc.depth_stencil.stencil_write_mask as _,
				reference: 0,
			})
			.back(vk::StencilOpState {
				fail_op: map_stencil_op(&desc.depth_stencil.back_face.fail),
				pass_op: map_stencil_op(&desc.depth_stencil.back_face.pass),
				depth_fail_op: map_stencil_op(&desc.depth_stencil.back_face.depth_fail),
				compare_op: map_compare_op(desc.depth_stencil.back_face.func),
				compare_mask: desc.depth_stencil.stencil_read_mask as _,
				write_mask: desc.depth_stencil.stencil_write_mask as _,
				reference: 0,
			});

		if desc.rasterizer.depth_bias.constant != 0.0 || desc.rasterizer.depth_bias.slope != 0.0 {
			rasterization_state = rasterization_state
				.depth_bias_enable(true)
				.depth_bias_constant_factor(desc.rasterizer.depth_bias.constant as f32)
				.depth_bias_clamp(desc.rasterizer.depth_bias.clamp)
				.depth_bias_slope_factor(desc.rasterizer.depth_bias.slope);
		}

		let color_attachments = desc.color_attachments.iter().map(|attachment| {
			let mut vk_attachment = vk::PipelineColorBlendAttachmentState::builder()
				.color_write_mask(vk::ColorComponentFlags::from_raw(attachment.write_mask.bits() as u32));

			if let Some(ref blend) = attachment.blend {
				vk_attachment = vk_attachment
					.blend_enable(true)
					.src_color_blend_factor(map_blend_factor(&blend.src_color))
					.dst_color_blend_factor(map_blend_factor(&blend.dst_color))
					.color_blend_op(map_blend_op(&blend.color_op))
					.src_alpha_blend_factor(map_blend_factor(&blend.src_alpha))
					.dst_alpha_blend_factor(map_blend_factor(&blend.dst_alpha))
					.alpha_blend_op(map_blend_op(&blend.alpha_op));
			}

			vk_attachment.build()
		}).collect::<Vec<_>>();

		let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
			.attachments(&color_attachments);

		let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
			.dynamic_states(&[
				vk::DynamicState::VIEWPORT,
				vk::DynamicState::SCISSOR,
				vk::DynamicState::BLEND_CONSTANTS,
				vk::DynamicState::STENCIL_REFERENCE,
			]);

		let color_target_formats = desc.color_attachments
			.iter()
			.map(|attachment| map_format(attachment.format))
			.collect::<Vec<_>>();

		let depth_stencil_format = map_format(desc.depth_stencil.format);

		let mut rendering = vk::PipelineRenderingCreateInfo::builder()
			.color_attachment_formats(&color_target_formats)
			.depth_attachment_format(depth_stencil_format)
			.stencil_attachment_format(depth_stencil_format);

		let graphics_pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
			// layout
			// stages
			.input_assembly_state(&input_assembly_state)
			.viewport_state(&viewport_state)
			.rasterization_state(&rasterization_state)
			.multisample_state(&multisample_state)
			.depth_stencil_state(&depth_stencil_state)
			.color_blend_state(&color_blend_state)
			.dynamic_state(&dynamic_state)
			.push_next(&mut rendering)
			.build();

		let pipeline = unsafe { self.device.create_graphics_pipelines(vk::PipelineCache::null(), &[graphics_pipeline_create_info], None) }.unwrap()[0];

		Ok(GraphicsPipeline { pipeline })
	}

	fn create_compute_pipeline(&self, desc: &super::ComputePipelineDesc) -> Result<Self::ComputePipeline, super::Error> {
		todo!()
	}

	fn create_raytracing_pipeline(&self, desc: &super::RaytracingPipelineDesc) -> Result<Self::RaytracingPipeline, super::Error> {
		todo!()
	}

	fn create_texture_view(&mut self, desc: &super::TextureViewDesc, texture: &Self::Texture) -> super::TextureView {
		todo!()
	}

	fn upload_buffer(&mut self, buffer: &Self::Buffer, data: &[u8]) {
		todo!()
	}

	fn upload_texture(&mut self, texture: &Self::Texture, data: &[u8]) {
		todo!()
	}

	fn submit(&self, cmd: &Self::CmdList) {
		todo!()
	}

	fn adapter_info(&self) -> &super::AdapterInfo {
		todo!()
	}

	fn capabilities(&self) -> &super::Capabilities {
		todo!()
	}

	fn acceleration_structure_sizes(&self, desc: &super::AccelerationStructureBuildInputs) -> super::AccelerationStructureSizes {
		todo!()
	}
}

struct CmdList {
	command_buffer: vk::CommandBuffer,
	device: ash::Device,

	// TODO: Initialize variables, move to Device
	acceleration_structure_ext: ash::extensions::khr::AccelerationStructure,
	ray_tracing_pipeline_ext: ash::extensions::khr::RayTracingPipeline,
	debug_utils_ext: Option<ash::extensions::ext::DebugUtils>,
}

impl CmdList {
	fn cmd(&self) -> vk::CommandBuffer {
		self.command_buffer
	}
}

impl super::CmdListImpl<Device> for CmdList {
	fn reset(&mut self, device: &Device, swap_chain: &SwapChain) {
		todo!()
	}

	fn copy_buffer(
		&self,
		src: &Buffer,
		src_offset: u64,
		dst: &Buffer,
		dst_offset: u64,
		size: u64,
	) {
		unsafe {
			self.device.cmd_copy_buffer(self.command_buffer, src.buffer, dst.buffer, &[vk::BufferCopy {
				src_offset,
				dst_offset,
				size,
			}]);
		}
	}

	fn copy_texture(
		&self,
		src: &Texture,
		src_mip_level: u32,
		src_array_slice: u32,
		src_offset: [u32; 3],
		dst: &Texture,
		dst_mip_level: u32,
		dst_array_slice: u32,
		dst_offset: [u32; 3],
		size: [u32; 3],
	) {
		unsafe {
			self.device.cmd_copy_image(self.command_buffer, src.image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, dst.image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[vk::ImageCopy {
				src_subresource: vk::ImageSubresourceLayers {
					aspect_mask: vk::ImageAspectFlags::COLOR, // TODO: Correct aspects
					mip_level: src_mip_level,
					base_array_layer: src_array_slice,
					layer_count: 1,
				},
				src_offset: map_offset(&src_offset),
				dst_subresource: vk::ImageSubresourceLayers {
					aspect_mask: vk::ImageAspectFlags::COLOR, // TODO: Correct aspects
					mip_level: dst_mip_level,
					base_array_layer: dst_array_slice,
					layer_count: 1,
				},
				dst_offset: map_offset(&dst_offset),
				extent: map_extent(&size),
			}]);
		}
	}

	fn copy_buffer_to_texture(
		&self,
		src: &Buffer,
		src_offset: u64,
		src_bytes_per_row: u32,
		dst: &Texture,
		dst_mip_level: u32,
		dst_array_slice: u32,
		dst_offset: [u32; 3],
		size: [u32; 3],
	) {
		unsafe {
			self.device.cmd_copy_buffer_to_image(self.command_buffer, src.buffer, dst.image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[vk::BufferImageCopy {
				buffer_offset: src_offset,
				buffer_row_length: src_bytes_per_row,
				buffer_image_height: 0,
				image_subresource: vk::ImageSubresourceLayers {
					aspect_mask: vk::ImageAspectFlags::COLOR, // TODO: Correct aspects
					mip_level: dst_mip_level,
					base_array_layer: dst_array_slice,
					layer_count: 1,
				},
				image_offset: map_offset(&dst_offset),
				image_extent: map_extent(&size),
			}]);
		}
	}

	fn copy_texture_to_buffer(
		&self,
		src: &Texture,
		src_mip_level: u32,
		src_array_slice: u32,
		src_offset: [u32; 3],
		dst: &Buffer,
		dst_offset: u64,
		dst_bytes_per_row: u32,
		size: [u32; 3],
	) {
		unsafe {
			self.device.cmd_copy_image_to_buffer(self.command_buffer, src.image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, dst.buffer, &[vk::BufferImageCopy {
				buffer_offset: dst_offset,
				buffer_row_length: dst_bytes_per_row,
				buffer_image_height: 0,
				image_subresource: vk::ImageSubresourceLayers {
					aspect_mask: vk::ImageAspectFlags::COLOR, // TODO: Correct aspects
					mip_level: src_mip_level,
					base_array_layer: src_array_slice,
					layer_count: 1,
				},
				image_offset: map_offset(&src_offset),
				image_extent: map_extent(&size),
			}]);
		}
	}

	fn render_pass_begin(&self, desc: &super::RenderPassDesc<Device>) {
		let vk_info = vk::RenderingInfo::builder();

		// TODO: Implement

		unsafe {
			self.device.cmd_begin_rendering(self.command_buffer, &vk_info);
		}
	}

	fn render_pass_end(&self) {
		unsafe {
			self.device.cmd_end_rendering(self.command_buffer);
		}
	}

	fn barriers(&self, barriers: &super::Barriers<Device>) {
		let memory_barriers = barriers.global.iter().map(|barrier| {
			vk::MemoryBarrier2::builder()
				.build()
		}).collect::<Vec<_>>();

		let buffer_memory_barriers = barriers.buffer.iter().map(|barrier| {
			vk::BufferMemoryBarrier2::builder()
				.buffer(barrier.buffer.buffer)
				.build()
		}).collect::<Vec<_>>();

		let image_memory_barriers = barriers.texture.iter().map(|barrier| {
			vk::ImageMemoryBarrier2::builder()
				.old_layout(map_image_layout(barrier.old_layout))
				.new_layout(map_image_layout(barrier.new_layout))
				.image(barrier.texture.image)
				.build()
		}).collect::<Vec<_>>();

		let dependency_info = vk::DependencyInfo::builder()
			.memory_barriers(&memory_barriers)
			.buffer_memory_barriers(&buffer_memory_barriers)
			.image_memory_barriers(&image_memory_barriers);

		unsafe {
			self.device.cmd_pipeline_barrier2(self.command_buffer, &dependency_info);
		}

		panic!("Not fully implemented");
	}

	fn set_viewport(&self, rect: &super::Rect<f32>, depth: Range<f32>) {
		let vk_viewport = vk::Viewport {
			x: rect.left,
			y: rect.bottom,
			width: rect.right - rect.left,
			height: rect.top - rect.bottom,
			min_depth: depth.start,
			max_depth: depth.end,
		};

		unsafe {
			self.device.cmd_set_viewport(self.command_buffer, 0, &[vk_viewport]);
		}
	}

	fn set_scissor(&self, rect: &super::Rect<u32>) {
		let vk_rect = vk::Rect2D {
			offset: vk::Offset2D {
				x: rect.left as _,
				y: rect.top as _,
			},
			extent: vk::Extent2D {
				width: (rect.right - rect.left),
				height: (rect.top - rect.bottom),
			},
		};

		unsafe {
			self.device.cmd_set_scissor(self.command_buffer, 0, &[vk_rect]);
		}
	}

	fn set_blend_constant(&self, color: super::Color<f32>) {
		unsafe {
			self.device.cmd_set_blend_constants(self.command_buffer, &[color.r, color.g, color.b, color.a]);
		}
	}

	fn set_stencil_reference(&self, reference: u32) {
		unsafe {
			self.device.cmd_set_stencil_reference(self.command_buffer, vk::StencilFaceFlags::FRONT_AND_BACK, reference);
		}
	}

	fn set_index_buffer(&self, buffer: &Buffer, offset: u64, format: super::Format) {
		let vk_format = map_index_format(format);

		unsafe {
			self.device.cmd_bind_index_buffer(self.command_buffer, buffer.buffer, offset, vk_format);
		}
	}

	fn set_graphics_pipeline(&self, pipeline: &GraphicsPipeline) {
		unsafe {
			self.device.cmd_bind_pipeline(self.command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline.pipeline);
		}
	}

	fn set_compute_pipeline(&self, pipeline: &ComputePipeline) {
		unsafe {
			self.device.cmd_bind_pipeline(self.command_buffer, vk::PipelineBindPoint::COMPUTE, pipeline.pipeline);
		}
	}

	fn set_raytracing_pipeline(&self, pipeline: &RaytracingPipeline) {
		unsafe {
			self.device.cmd_bind_pipeline(self.command_buffer, vk::PipelineBindPoint::RAY_TRACING_KHR, pipeline.pipeline);
		}
	}

	fn graphics_push_constants(&self, offset: u32, data: &[u8]) {
		/*unsafe {
			self.device.cmd_push_constants(
				self.command_buffer,
				self.pipeline_layout,
				vk::ShaderStageFlags::ALL_GRAPHICS,
				0,
				data,
			);
		}*/
		todo!()
	}

	fn compute_push_constants(&self, offset: u32, data: &[u8]) {
		/*unsafe {
			self.device.cmd_push_constants(
				self.command_buffer,
				self.pipeline_layout,
				vk::ShaderStageFlags::COMPUTE,
				0,
				data,
			);
		}*/
		todo!()
	}

	fn draw(&self, vertices: Range<u32>, instances: Range<u32>) {
		unsafe {
			self.device.cmd_draw(self.command_buffer, vertices.len() as u32, instances.len() as u32, vertices.start, vertices.start);
		}
	}

	fn draw_indexed(&self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
		unsafe {
			self.device.cmd_draw_indexed(self.command_buffer, indices.len() as u32, instances.len() as u32, indices.start, base_vertex, instances.start);
		}
	}

	fn dispatch(&self, x: u32, y: u32, z: u32) {
		unsafe {
			self.device.cmd_dispatch(self.command_buffer, x, y, z);
		}
	}

	fn dispatch_rays(&self, desc: &super::DispatchRaysDesc) {
		unsafe {
			self.ray_tracing_pipeline_ext.cmd_trace_rays(
				self.command_buffer,
				&desc.raygen.as_ref().map_or(Default::default(), |t| vk::StridedDeviceAddressRegionKHR {
					device_address: t.ptr.0,
					stride: t.stride as _,
					size: t.size as _,
				}),
				&desc.miss.as_ref().map_or(Default::default(), |t| vk::StridedDeviceAddressRegionKHR {
					device_address: t.ptr.0,
					stride: t.stride as _,
					size: t.size as _,
				}),
				&desc.hit_group.as_ref().map_or(Default::default(), |t| vk::StridedDeviceAddressRegionKHR {
					device_address: t.ptr.0,
					stride: t.stride as _,
					size: t.size as _,
				}),
				&desc.callable.as_ref().map_or(Default::default(), |t| vk::StridedDeviceAddressRegionKHR {
					device_address: t.ptr.0,
					stride: t.stride as _,
					size: t.size as _,
				}),
				desc.width,
				desc.height,
				desc.depth,
			);
		}
	}

	fn build_acceleration_structure(&self, desc: &super::AccelerationStructureBuildDesc<Device>) {
		todo!()
	}

	fn debug_marker(&self, name: &str, color: super::Color<u8>) {
		if let Some(debug_utils) = &self.debug_utils_ext {
			let label = CString::new(name).unwrap();
			let label = vk::DebugUtilsLabelEXT::builder()
				.label_name(&label)
				.color(color.to_f32().into());

			unsafe {
				debug_utils.cmd_insert_debug_utils_label(self.command_buffer, &label);
			}
		}
	}

	fn debug_event_push(&self, name: &str, color: super::Color<u8>) {
		if let Some(debug_utils) = &self.debug_utils_ext {
			let label = CString::new(name).unwrap();
			let label = vk::DebugUtilsLabelEXT::builder()
				.label_name(&label)
				.color(color.to_f32().into());

			unsafe {
				debug_utils.cmd_begin_debug_utils_label(self.command_buffer, &label);
			}
		}
	}

	fn debug_event_pop(&self) {
		if let Some(debug_utils) = &self.debug_utils_ext {
			unsafe {
				debug_utils.cmd_end_debug_utils_label(self.command_buffer);
			}
		}
	}
}

/*
fn build_acceleration_structure_input(inputs: super::AccelerationStructureBuildInputs) {

	let geometries: Vec<vk::AccelerationStructureGeometryKHR> = Vec::new();
	let primitive_counts: Vec<usize> = Vec::new();
	//let range_infos: Vec<vk::AccelerationStructureBuildRangeInfoKHR> = Vec::new();

	for geo in inputs.geometry {
		let vk_geo = match geo.part {

			// TODO: pass count to next struct
			super::GeometryPart::AABBs(aabbs) => vk::AccelerationStructureGeometryKHR {
				geometry_type: vk::GeometryTypeKHR::AABBS,
				flags: map_acceleration_structure_bottom_level_flags(geo.flags),
				geometry: vk::AccelerationStructureGeometryDataKHR {
					aabbs: vk::AccelerationStructureGeometryAabbsDataKHR {
						data: vk::DeviceOrHostAddressConstKHR {
							device_address: aabbs.data.0,
						},
						stride: aabbs.stride as _,
						..Default::default()
					},
				},
				..Default::default()
			},

			// TODO: pass index_count / 3 to next struct
			super::GeometryPart::Triangles(triangles) => vk::AccelerationStructureGeometryKHR {
				geometry_type: vk::GeometryTypeKHR::TRIANGLES,
				flags: map_acceleration_structure_bottom_level_flags(geo.flags),
				geometry: vk::AccelerationStructureGeometryDataKHR {
					triangles: vk::AccelerationStructureGeometryTrianglesDataKHR {
						vertex_format: map_format(triangles.vertex_format),
						vertex_data: vk::DeviceOrHostAddressConstKHR {
							device_address: triangles.vertex_data.0,
						},
						vertex_stride: triangles.vertex_stride as _,
						max_vertex: triangles.vertex_count as _,
						index_type: map_index_format(triangles.index_format),
						index_data: vk::DeviceOrHostAddressConstKHR {
							device_address: triangles.index_data.0,
						},
						transform_data: vk::DeviceOrHostAddressConstKHR {
							device_address: triangles.transform.0,
						},
						..Default::default()
					},
				},
				..Default::default()
			},
		};
	}
}
*/
