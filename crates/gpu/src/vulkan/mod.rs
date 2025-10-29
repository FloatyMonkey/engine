#![allow(dead_code)]
#![allow(unused_variables)]

mod cmd;

use std::{
	collections::HashSet,
	ffi::{CStr, c_char, c_void},
};
use std::{ffi::CString, ops::Range};

use ash::{prelude::VkResult, vk};
use gpu_allocator::vulkan as allocator;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;

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
		super::Format::R8UInt => vk::Format::R8_UINT,
		super::Format::R8SInt => vk::Format::R8_SINT,

		super::Format::R16UNorm => vk::Format::R16_UNORM,
		super::Format::R16SNorm => vk::Format::R16_SNORM,
		super::Format::R16UInt => vk::Format::R16_UINT,
		super::Format::R16SInt => vk::Format::R16_SINT,
		super::Format::R16Float => vk::Format::R16_SFLOAT,

		super::Format::R32UInt => vk::Format::R32_UINT,
		super::Format::R32SInt => vk::Format::R32_SINT,
		super::Format::R32Float => vk::Format::R32_SFLOAT,

		super::Format::RG8UNorm => vk::Format::R8G8_UNORM,
		super::Format::RG8SNorm => vk::Format::R8G8_SNORM,
		super::Format::RG8UInt => vk::Format::R8G8_UINT,
		super::Format::RG8SInt => vk::Format::R8G8_SINT,

		super::Format::RG16UNorm => vk::Format::R16G16_UNORM,
		super::Format::RG16SNorm => vk::Format::R16G16_SNORM,
		super::Format::RG16UInt => vk::Format::R16G16_UINT,
		super::Format::RG16SInt => vk::Format::R16G16_SINT,
		super::Format::RG16Float => vk::Format::R16G16_SFLOAT,

		super::Format::RG32UInt => vk::Format::R32G32_UINT,
		super::Format::RG32SInt => vk::Format::R32G32_SINT,
		super::Format::RG32Float => vk::Format::R32G32_SFLOAT,

		super::Format::RGB32UInt => vk::Format::R32G32B32_UINT,
		super::Format::RGB32SInt => vk::Format::R32G32B32_SINT,
		super::Format::RGB32Float => vk::Format::R32G32B32_SFLOAT,

		super::Format::RGBA8UNorm => vk::Format::R8G8B8A8_UNORM,
		super::Format::RGBA8SNorm => vk::Format::R8G8B8A8_SNORM,
		super::Format::RGBA8UInt => vk::Format::R8G8B8A8_UINT,
		super::Format::RGBA8SInt => vk::Format::R8G8B8A8_SINT,

		super::Format::RGBA16UNorm => vk::Format::R16G16B16A16_UNORM,
		super::Format::RGBA16SNorm => vk::Format::R16G16B16A16_SNORM,
		super::Format::RGBA16UInt => vk::Format::R16G16B16A16_UINT,
		super::Format::RGBA16SInt => vk::Format::R16G16B16A16_SINT,
		super::Format::RGBA16Float => vk::Format::R16G16B16A16_SFLOAT,

		super::Format::RGBA32UInt => vk::Format::R32G32B32A32_UINT,
		super::Format::RGBA32SInt => vk::Format::R32G32B32A32_SINT,
		super::Format::RGBA32Float => vk::Format::R32G32B32A32_SFLOAT,

		super::Format::BGRA8UNorm => vk::Format::B8G8R8A8_UNORM,

		super::Format::D16UNorm => vk::Format::D16_UNORM,
		super::Format::D24UNormS8UInt => vk::Format::D24_UNORM_S8_UINT,
		super::Format::D32Float => vk::Format::D32_SFLOAT,
		super::Format::D32FloatS8UIntX24 => vk::Format::D32_SFLOAT_S8_UINT,
	}
}

fn map_index_format(format: super::Format) -> vk::IndexType {
	match format {
		super::Format::Unknown => vk::IndexType::NONE_KHR,
		super::Format::R8UInt => vk::IndexType::UINT8_KHR,
		super::Format::R16UInt => vk::IndexType::UINT16,
		super::Format::R32UInt => vk::IndexType::UINT32,
		_ => panic!(),
	}
}

fn map_filter_mode(filter_mode: super::FilterMode) -> vk::Filter {
	match filter_mode {
		super::FilterMode::Nearest => vk::Filter::NEAREST,
		super::FilterMode::Linear => vk::Filter::LINEAR,
	}
}

fn map_mip_filter_mode(filter_mode: super::FilterMode) -> vk::SamplerMipmapMode {
	match filter_mode {
		super::FilterMode::Nearest => vk::SamplerMipmapMode::NEAREST,
		super::FilterMode::Linear => vk::SamplerMipmapMode::LINEAR,
	}
}

fn map_address_mode(address_mode: super::AddressMode) -> vk::SamplerAddressMode {
	match address_mode {
		super::AddressMode::Clamp => vk::SamplerAddressMode::CLAMP_TO_EDGE,
		super::AddressMode::Repeat => vk::SamplerAddressMode::REPEAT,
		super::AddressMode::Mirror => vk::SamplerAddressMode::MIRRORED_REPEAT,
		super::AddressMode::MirrorOnce => vk::SamplerAddressMode::MIRROR_CLAMP_TO_EDGE,
		super::AddressMode::Border => vk::SamplerAddressMode::CLAMP_TO_BORDER,
	}
}

fn map_border_color(border_color: super::BorderColor) -> vk::BorderColor {
	match border_color {
		super::BorderColor::TransparentBlack => vk::BorderColor::FLOAT_TRANSPARENT_BLACK,
		super::BorderColor::OpaqueBlack => vk::BorderColor::FLOAT_OPAQUE_BLACK,
		super::BorderColor::White => vk::BorderColor::FLOAT_OPAQUE_WHITE,
	}
}

fn map_image_type(desc: &super::TextureDesc) -> vk::ImageType {
	if desc.depth > 1 {
		return vk::ImageType::TYPE_3D;
	}
	if desc.height > 1 {
		return vk::ImageType::TYPE_2D;
	}
	vk::ImageType::TYPE_1D
}

fn map_image_layout(layout: super::TextureLayout) -> vk::ImageLayout {
	match layout {
		super::TextureLayout::Common => vk::ImageLayout::GENERAL,
		super::TextureLayout::Present => vk::ImageLayout::PRESENT_SRC_KHR,
		super::TextureLayout::CopySrc => vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
		super::TextureLayout::CopyDst => vk::ImageLayout::TRANSFER_DST_OPTIMAL,
		super::TextureLayout::ShaderResource => vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
		super::TextureLayout::UnorderedAccess => vk::ImageLayout::GENERAL,
		super::TextureLayout::RenderTarget => vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
		super::TextureLayout::DepthStencilWrite => {
			vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
		}
		super::TextureLayout::DepthStencilRead => vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL,
	}
}

fn map_buffer_usage_flags(usage: super::BufferUsage) -> vk::BufferUsageFlags {
	let mut vk_flags = vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST;

	if usage.contains(super::BufferUsage::INDEX) {
		vk_flags |= vk::BufferUsageFlags::INDEX_BUFFER;
	}
	if usage.contains(super::BufferUsage::SHADER_RESOURCE) {
		vk_flags |= vk::BufferUsageFlags::UNIFORM_TEXEL_BUFFER;
	}
	if usage.contains(super::BufferUsage::UNORDERED_ACCESS) {
		vk_flags |= vk::BufferUsageFlags::STORAGE_TEXEL_BUFFER;
	}
	if usage.contains(super::BufferUsage::ACCELERATION_STRUCTURE) {
		vk_flags |= vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR;
		vk_flags |= vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR;
		vk_flags |= vk::BufferUsageFlags::SHADER_BINDING_TABLE_KHR; // TODO: Why is this here?
	}

	vk_flags
}

fn map_image_usage_flags(usage: super::TextureUsage) -> vk::ImageUsageFlags {
	let mut vk_flags = vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST;

	if usage.contains(super::TextureUsage::RENDER_TARGET) {
		vk_flags |= vk::ImageUsageFlags::COLOR_ATTACHMENT;
	}
	if usage.contains(super::TextureUsage::DEPTH_STENCIL) {
		vk_flags |= vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
	}
	if usage.contains(super::TextureUsage::SHADER_RESOURCE) {
		vk_flags |= vk::ImageUsageFlags::SAMPLED;
	}
	if usage.contains(super::TextureUsage::UNORDERED_ACCESS) {
		vk_flags |= vk::ImageUsageFlags::STORAGE;
	}

	vk_flags
}

fn map_compare_op(compare_op: super::CompareOp) -> vk::CompareOp {
	match compare_op {
		super::CompareOp::Never => vk::CompareOp::NEVER,
		super::CompareOp::Always => vk::CompareOp::ALWAYS,
		super::CompareOp::Equal => vk::CompareOp::EQUAL,
		super::CompareOp::NotEqual => vk::CompareOp::NOT_EQUAL,
		super::CompareOp::Less => vk::CompareOp::LESS,
		super::CompareOp::LessEqual => vk::CompareOp::LESS_OR_EQUAL,
		super::CompareOp::Greater => vk::CompareOp::GREATER,
		super::CompareOp::GreaterEqual => vk::CompareOp::GREATER_OR_EQUAL,
	}
}

fn map_topology(topology: super::Topology) -> vk::PrimitiveTopology {
	match topology {
		super::Topology::PointList => vk::PrimitiveTopology::POINT_LIST,
		super::Topology::LineList => vk::PrimitiveTopology::LINE_LIST,
		super::Topology::LineStrip => vk::PrimitiveTopology::LINE_STRIP,
		super::Topology::TriangleList => vk::PrimitiveTopology::TRIANGLE_LIST,
		super::Topology::TriangleStrip => vk::PrimitiveTopology::TRIANGLE_STRIP,
	}
}

fn map_stencil_op(stencil_op: &super::StencilOp) -> vk::StencilOp {
	match stencil_op {
		super::StencilOp::Keep => vk::StencilOp::KEEP,
		super::StencilOp::Zero => vk::StencilOp::ZERO,
		super::StencilOp::Replace => vk::StencilOp::REPLACE,
		super::StencilOp::Invert => vk::StencilOp::INVERT,
		super::StencilOp::IncrementWrap => vk::StencilOp::INCREMENT_AND_WRAP,
		super::StencilOp::IncrementClamp => vk::StencilOp::INCREMENT_AND_CLAMP,
		super::StencilOp::DecrementWrap => vk::StencilOp::DECREMENT_AND_WRAP,
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
		super::CullMode::None => vk::CullModeFlags::NONE,
		super::CullMode::Front => vk::CullModeFlags::FRONT,
		super::CullMode::Back => vk::CullModeFlags::BACK,
	}
}

fn map_blend_factor(blend_factor: &super::BlendFactor) -> vk::BlendFactor {
	match blend_factor {
		super::BlendFactor::Zero => vk::BlendFactor::ZERO,
		super::BlendFactor::One => vk::BlendFactor::ONE,
		super::BlendFactor::SrcColor => vk::BlendFactor::SRC_COLOR,
		super::BlendFactor::InvSrcColor => vk::BlendFactor::ONE_MINUS_SRC_COLOR,
		super::BlendFactor::SrcAlpha => vk::BlendFactor::SRC_ALPHA,
		super::BlendFactor::InvSrcAlpha => vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
		super::BlendFactor::DstColor => vk::BlendFactor::DST_COLOR,
		super::BlendFactor::InvDstColor => vk::BlendFactor::ONE_MINUS_DST_COLOR,
		super::BlendFactor::DstAlpha => vk::BlendFactor::DST_ALPHA,
		super::BlendFactor::InvDstAlpha => vk::BlendFactor::ONE_MINUS_DST_ALPHA,
		super::BlendFactor::Src1Color => vk::BlendFactor::SRC1_COLOR,
		super::BlendFactor::InvSrc1Color => vk::BlendFactor::ONE_MINUS_SRC1_COLOR,
		super::BlendFactor::Src1Alpha => vk::BlendFactor::SRC1_ALPHA,
		super::BlendFactor::InvSrc1Alpha => vk::BlendFactor::ONE_MINUS_SRC1_ALPHA,
		super::BlendFactor::SrcAlphaSat => vk::BlendFactor::SRC_ALPHA_SATURATE,
		super::BlendFactor::ConstantColor => vk::BlendFactor::CONSTANT_COLOR,
		super::BlendFactor::InvConstantColor => vk::BlendFactor::ONE_MINUS_CONSTANT_COLOR,
	}
}

fn map_blend_op(blend_op: &super::BlendOp) -> vk::BlendOp {
	match blend_op {
		super::BlendOp::Add => vk::BlendOp::ADD,
		super::BlendOp::Subtract => vk::BlendOp::SUBTRACT,
		super::BlendOp::RevSubtract => vk::BlendOp::REVERSE_SUBTRACT,
		super::BlendOp::Min => vk::BlendOp::MIN,
		super::BlendOp::Max => vk::BlendOp::MAX,
	}
}

fn map_acceleration_structure_flags(
	flags: super::AccelerationStructureBuildFlags,
) -> vk::BuildAccelerationStructureFlagsKHR {
	let mut vk_flags = vk::BuildAccelerationStructureFlagsKHR::empty();

	if flags.contains(super::AccelerationStructureBuildFlags::ALLOW_UPDATE) {
		vk_flags |= vk::BuildAccelerationStructureFlagsKHR::ALLOW_UPDATE;
	}
	if flags.contains(super::AccelerationStructureBuildFlags::ALLOW_COMPACTION) {
		vk_flags |= vk::BuildAccelerationStructureFlagsKHR::ALLOW_COMPACTION;
	}
	if flags.contains(super::AccelerationStructureBuildFlags::PREFER_FAST_TRACE) {
		vk_flags |= vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE;
	}
	if flags.contains(super::AccelerationStructureBuildFlags::PREFER_FAST_BUILD) {
		vk_flags |= vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_BUILD;
	}
	if flags.contains(super::AccelerationStructureBuildFlags::MINIMIZE_MEMORY) {
		vk_flags |= vk::BuildAccelerationStructureFlagsKHR::LOW_MEMORY;
	}

	vk_flags
}

fn map_acceleration_structure_geometry_flags(
	flags: super::AccelerationStructureGeometryFlags,
) -> vk::GeometryFlagsKHR {
	let mut vk_flags = vk::GeometryFlagsKHR::empty();

	if flags.contains(super::AccelerationStructureGeometryFlags::OPAQUE) {
		vk_flags |= vk::GeometryFlagsKHR::OPAQUE;
	}
	if flags.contains(super::AccelerationStructureGeometryFlags::NO_DUPLICATE_ANY_HIT_INVOCATION) {
		vk_flags |= vk::GeometryFlagsKHR::NO_DUPLICATE_ANY_HIT_INVOCATION;
	}

	vk_flags
}

fn map_acceleration_structure_instance_flags(
	flags: super::AccelerationStructureInstanceFlags,
) -> vk::GeometryInstanceFlagsKHR {
	let mut vk_flags = vk::GeometryInstanceFlagsKHR::empty();

	if flags.contains(super::AccelerationStructureInstanceFlags::TRIANGLE_CULL_DISABLE) {
		vk_flags |= vk::GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE;
	}
	if flags.contains(super::AccelerationStructureInstanceFlags::TRIANGLE_FRONT_CCW) {
		vk_flags |= vk::GeometryInstanceFlagsKHR::TRIANGLE_FRONT_COUNTERCLOCKWISE;
	}
	if flags.contains(super::AccelerationStructureInstanceFlags::FORCE_OPAQUE) {
		vk_flags |= vk::GeometryInstanceFlagsKHR::FORCE_OPAQUE;
	}
	if flags.contains(super::AccelerationStructureInstanceFlags::FORCE_NON_OPAQUE) {
		vk_flags |= vk::GeometryInstanceFlagsKHR::FORCE_NO_OPAQUE;
	}

	vk_flags
}

fn map_load_op<T: Default>(load_op: super::LoadOp<T>) -> (vk::AttachmentLoadOp, T) {
	match load_op {
		super::LoadOp::Load => (vk::AttachmentLoadOp::LOAD, Default::default()),
		super::LoadOp::Clear(value) => (vk::AttachmentLoadOp::CLEAR, value),
		super::LoadOp::Discard => (vk::AttachmentLoadOp::DONT_CARE, Default::default()),
	}
}

fn map_store_op(store_op: super::StoreOp) -> vk::AttachmentStoreOp {
	match store_op {
		super::StoreOp::Store => vk::AttachmentStoreOp::STORE,
		super::StoreOp::Discard => vk::AttachmentStoreOp::DONT_CARE,
	}
}

pub struct Surface {
	swapchain_ext: ash::khr::swapchain::Device,

	swapchain: vk::SwapchainKHR,
	textures: Vec<Texture>,
	acquire_semaphores: Vec<vk::Semaphore>,
	next_semaphore: vk::Semaphore,
	bb_index: usize,
}

impl super::SurfaceImpl<Device> for Surface {
	fn update(&mut self, device: &mut Device, size: [u32; 2]) {
		todo!()
	}

	fn wait_for_last_frame(&mut self) {
		todo!()
	}

	fn acquire(&mut self) -> &Texture {
		let (index, _suboptimal) = unsafe {
			self.swapchain_ext
				.acquire_next_image(
					self.swapchain,
					u64::MAX,
					self.next_semaphore,
					vk::Fence::null(),
				)
				.unwrap()
		};

		std::mem::swap(
			&mut self.acquire_semaphores[index as usize],
			&mut self.next_semaphore,
		);

		self.bb_index = index as usize;
		&self.textures[index as usize]
	}

	fn present(&mut self, device: &Device) {
		let swapchains = [self.swapchain];
		let image_indices = [self.bb_index as u32];
		let wait_semaphores = [todo!()];

		let present_info = vk::PresentInfoKHR::default()
			.swapchains(&swapchains)
			.image_indices(&image_indices)
			.wait_semaphores(&wait_semaphores);

		unsafe {
			self.swapchain_ext
				.queue_present(device.graphics_queue, &present_info)
		}
		.unwrap();
	}
}

pub struct Buffer {
	buffer: vk::Buffer,
	allocation: allocator::Allocation,
	srv_index: Option<usize>,
	uav_index: Option<usize>,
	cpu_ptr: *mut u8,
	gpu_ptr: u64,
}

impl super::BufferImpl<Device> for Buffer {
	fn srv_index(&self) -> Option<u32> {
		self.srv_index.map(|i| i as u32)
	}

	fn uav_index(&self) -> Option<u32> {
		self.uav_index.map(|i| i as u32)
	}

	fn cpu_ptr(&self) -> *mut u8 {
		self.cpu_ptr
	}

	fn gpu_ptr(&self) -> super::GpuPtr {
		super::GpuPtr(self.gpu_ptr)
	}
}

pub struct Texture {
	image: vk::Image,
	allocation: allocator::Allocation,
	srv_index: Option<usize>,
	uav_index: Option<usize>,
	rtv: Option<vk::ImageView>,
	dsv: Option<vk::ImageView>,
}

impl super::TextureImpl<Device> for Texture {
	fn srv_index(&self) -> Option<u32> {
		self.srv_index.map(|i| i as u32)
	}

	fn uav_index(&self) -> Option<u32> {
		self.uav_index.map(|i| i as u32)
	}
}

pub struct Sampler {
	sampler: vk::Sampler,
}

impl super::SamplerImpl<Device> for Sampler {}

pub struct AccelerationStructure {
	acceleration_structure: vk::AccelerationStructureKHR,
	srv_index: Option<usize>,
	gpu_ptr: u64,
}

impl super::AccelerationStructureImpl<Device> for AccelerationStructure {
	fn srv_index(&self) -> Option<u32> {
		self.srv_index.map(|i| i as u32)
	}

	fn gpu_ptr(&self) -> super::GpuPtr {
		super::GpuPtr(self.gpu_ptr)
	}

	fn instance_descriptor_size() -> usize {
		size_of::<vk::AccelerationStructureInstanceKHR>()
	}

	fn write_instance_descriptor(
		instance: &super::AccelerationStructureInstance,
		slice: &mut [u8],
	) {
		let t = &instance.transform;

		let vk_instance = vk::AccelerationStructureInstanceKHR {
			transform: vk::TransformMatrixKHR {
				matrix: [
					t[0][0], t[0][1], t[0][2], t[0][3], t[1][0], t[1][1], t[1][2], t[1][3],
					t[2][0], t[2][1], t[2][2], t[2][3],
				],
			},
			instance_custom_index_and_mask: vk::Packed24_8::new(instance.user_id, instance.mask),
			instance_shader_binding_table_record_offset_and_flags: vk::Packed24_8::new(
				instance.contribution_to_hit_group_index,
				map_acceleration_structure_instance_flags(instance.flags).as_raw() as _,
			),
			acceleration_structure_reference: vk::AccelerationStructureReferenceKHR {
				device_handle: instance.bottom_level.0,
			},
		};

		unsafe {
			std::ptr::copy_nonoverlapping(
				&vk_instance as *const _ as _,
				slice.as_mut_ptr(),
				size_of::<vk::AccelerationStructureInstanceKHR>(),
			);
		}
	}
}

pub struct GraphicsPipeline {
	pipeline: vk::Pipeline,
}

impl super::GraphicsPipelineImpl<Device> for GraphicsPipeline {}

pub struct ComputePipeline {
	pipeline: vk::Pipeline,
}

impl super::ComputePipelineImpl<Device> for ComputePipeline {}

pub struct RaytracingPipeline {
	pipeline: vk::Pipeline,
}

impl super::RaytracingPipelineImpl<Device> for RaytracingPipeline {
	fn shader_identifier_size(&self) -> usize {
		todo!()
	}

	fn write_shader_identifier(&self, group_index: usize, slice: &mut [u8]) {
		todo!()
	}
}

unsafe extern "system" fn debug_callback(
	message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
	_message_type: vk::DebugUtilsMessageTypeFlagsEXT,
	callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
	_user_data: *mut c_void,
) -> vk::Bool32 {
	let log_level = match message_severity {
		vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => log::Level::Debug,
		vk::DebugUtilsMessageSeverityFlagsEXT::INFO => log::Level::Info,
		vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => log::Level::Warn,
		vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => log::Level::Error,
		_ => unreachable!(),
	};

	let message = unsafe { CStr::from_ptr((*callback_data).p_message) };

	log::log!(target: "gpu::vulkan", log_level, "{}", message.to_str().unwrap());
	println!("{}", message.to_str().unwrap()); // TODO: Remove

	vk::FALSE
}

fn pick_physical_device_and_queue_family_indices(
	instance: &ash::Instance,
	extensions: &[&CStr],
) -> VkResult<Option<(vk::PhysicalDevice, u32)>> {
	Ok(unsafe { instance.enumerate_physical_devices() }?
		.into_iter()
		.find_map(|physical_device| {
			let has_extensions =
				unsafe { instance.enumerate_device_extension_properties(physical_device) }.map(
					|exts| {
						let set: HashSet<&CStr> = exts
							.iter()
							.map(|ext| unsafe {
								CStr::from_ptr(&ext.extension_name as *const c_char)
							})
							.collect();

						extensions.iter().all(|ext| set.contains(ext))
					},
				);

			if has_extensions != Ok(true) {
				return None;
			}

			let graphics_family =
				unsafe { instance.get_physical_device_queue_family_properties(physical_device) }
					.into_iter()
					.enumerate()
					.find(|(_, queue_family)| {
						queue_family.queue_count > 0
							&& queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
					});

			graphics_family.map(|(i, _)| (physical_device, i as u32))
		}))
}

pub struct Device {
	adapter_info: super::AdapterInfo,
	capabilities: super::Capabilities,

	entry: ash::Entry,
	device: ash::Device,
	instance: ash::Instance,
	physical_device: vk::PhysicalDevice,
	allocator: allocator::Allocator,
	graphics_queue: vk::Queue,
	command_pool: vk::CommandPool,
	//rt_pipeline_properties: vk::PhysicalDeviceRayTracingPipelinePropertiesKHR,
	acceleration_structure_ext: ash::khr::acceleration_structure::Device,
	ray_tracing_pipeline_ext: ash::khr::ray_tracing_pipeline::Device,
	debug_utils_ext: Option<ash::ext::debug_utils::Instance>,
}

impl super::DeviceImpl for Device {
	type Surface = Surface;
	type CmdList = CmdList;
	type Buffer = Buffer;
	type Texture = Texture;
	type Sampler = Sampler;
	type AccelerationStructure = AccelerationStructure;
	type GraphicsPipeline = GraphicsPipeline;
	type ComputePipeline = ComputePipeline;
	type RaytracingPipeline = RaytracingPipeline;

	fn new(desc: &super::DeviceDesc) -> Self {
		let entry = unsafe { ash::Entry::load() }.unwrap();

		let mut instance_extension_names = Vec::new();

		let instance = {
			const VK_LAYER_KHRONOS_VALIDATION: &CStr = c"VK_LAYER_KHRONOS_validation";

			let mut layers: Vec<&CStr> = Vec::new();

			if !desc.validation.is_empty() {
				layers.push(VK_LAYER_KHRONOS_VALIDATION);
			}

			let supported_layer_names = unsafe {
				entry
					.enumerate_instance_layer_properties()
					.unwrap()
					.into_iter()
					.map(|layer| CStr::from_ptr(layer.layer_name.as_ptr()))
					.collect::<Vec<_>>()
			};

			for layer in &layers {
				if !supported_layer_names.contains(layer) {
					log::warn!(target: "gpu::vulkan", "Requested layer not found: {:?}", layer);
				}
			}

			instance_extension_names.push(vk::KHR_SURFACE_NAME);

			if !desc.validation.is_empty() {
				instance_extension_names.push(vk::EXT_DEBUG_UTILS_NAME);
			}

			if cfg!(target_os = "windows") {
				instance_extension_names.push(vk::KHR_WIN32_SURFACE_NAME);
			}

			let layer_ptrs = layers.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();

			let extension_ptrs = instance_extension_names
				.iter()
				.map(|s| s.as_ptr())
				.collect::<Vec<_>>();

			let application_name = CString::new("App").unwrap(); // TODO: Hardcoded
			let engine_name = CString::new("Engine").unwrap(); // TODO: Hardcoded

			let application_info = vk::ApplicationInfo::default()
				.application_name(application_name.as_c_str())
				.application_version(vk::make_api_version(0, 0, 1, 0)) // TODO: Hardcoded
				.engine_name(engine_name.as_c_str())
				.engine_version(vk::make_api_version(0, 0, 1, 0)) // TODO: Hardcoded
				.api_version(vk::API_VERSION_1_3);

			let instance_create_info = vk::InstanceCreateInfo::default()
				.application_info(&application_info)
				.enabled_layer_names(&layer_ptrs)
				.enabled_extension_names(&extension_ptrs);

			unsafe { entry.create_instance(&instance_create_info, None) }.unwrap()
		};

		let debug_utils_ext = instance_extension_names
			.contains(&vk::EXT_DEBUG_UTILS_NAME)
			.then(|| ash::ext::debug_utils::Instance::new(&entry, &instance));

		if let Some(ext) = &debug_utils_ext {
			let debug_utils_messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
				.message_severity(
					//vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
					vk::DebugUtilsMessageSeverityFlagsEXT::INFO
						| vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
						| vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
				)
				.message_type(
					vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
						| vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
						| vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
				)
				.pfn_user_callback(Some(debug_callback));

			let _messenger = unsafe {
				ext.create_debug_utils_messenger(&debug_utils_messenger_create_info, None)
			}
			.unwrap();
		}

		let device_extension_names = [
			vk::KHR_DEFERRED_HOST_OPERATIONS_NAME, // Required by AccelerationStructure
			vk::KHR_ACCELERATION_STRUCTURE_NAME,
			vk::KHR_RAY_TRACING_PIPELINE_NAME,
			//vk::KHR_RAY_QUERY_NAME,
			vk::KHR_SWAPCHAIN_NAME, // TODO: Conditionally enable
		];

		let (physical_device, queue_family_index) =
			pick_physical_device_and_queue_family_indices(&instance, &device_extension_names)
				.unwrap()
				.unwrap();

		let device = {
			let queue_create_info = vk::DeviceQueueCreateInfo::default()
				.queue_family_index(queue_family_index)
				.queue_priorities(&[1.0]);

			let mut features2 = vk::PhysicalDeviceFeatures2::default();
			unsafe { instance.get_physical_device_features2(physical_device, &mut features2) };

			let mut vulkan11_features = vk::PhysicalDeviceVulkan11Features::default()
				.variable_pointers(true)
				.variable_pointers_storage_buffer(true);

			let mut vulkan12_features = vk::PhysicalDeviceVulkan12Features::default()
				.shader_int8(true)
				.buffer_device_address(true)
				.vulkan_memory_model(true)
				.runtime_descriptor_array(true)
				.shader_sampled_image_array_non_uniform_indexing(true)
				.shader_storage_buffer_array_non_uniform_indexing(true)
				.shader_storage_image_array_non_uniform_indexing(true)
				.descriptor_indexing(true);

			let mut vulkan13_features = vk::PhysicalDeviceVulkan13Features::default()
				.dynamic_rendering(true)
				.synchronization2(true); // TODO: Actually use this

			let mut as_feature = vk::PhysicalDeviceAccelerationStructureFeaturesKHR::default()
				.acceleration_structure(true);

			let mut raytracing_pipeline =
				vk::PhysicalDeviceRayTracingPipelineFeaturesKHR::default()
					.ray_tracing_pipeline(true);

			let queue_create_infos = [queue_create_info];
			let device_extension_names_ptrs = device_extension_names.map(|n| n.as_ptr());

			let device_create_info = vk::DeviceCreateInfo::default()
				.push_next(&mut features2)
				.push_next(&mut vulkan11_features)
				.push_next(&mut vulkan12_features)
				.push_next(&mut vulkan13_features)
				.push_next(&mut as_feature)
				.push_next(&mut raytracing_pipeline)
				.queue_create_infos(&queue_create_infos)
				.enabled_extension_names(&device_extension_names_ptrs);

			unsafe { instance.create_device(physical_device, &device_create_info, None) }.unwrap()
		};

		let allocator = allocator::Allocator::new(&allocator::AllocatorCreateDesc {
			instance: instance.clone(),
			device: device.clone(),
			physical_device,
			debug_settings: Default::default(),
			buffer_device_address: true,
			allocation_sizes: Default::default(),
		})
		.unwrap();

		let graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

		let command_pool = {
			let command_pool_create_info = vk::CommandPoolCreateInfo::default()
				.queue_family_index(queue_family_index)
				.flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

			unsafe { device.create_command_pool(&command_pool_create_info, None) }.unwrap()
		};

		let acceleration_structure_ext =
			ash::khr::acceleration_structure::Device::new(&instance, &device);
		let ray_tracing_pipeline_ext =
			ash::khr::ray_tracing_pipeline::Device::new(&instance, &device);

		let mut rt_pipeline_properties =
			vk::PhysicalDeviceRayTracingPipelinePropertiesKHR::default();

		let mut physical_device_properties2 =
			vk::PhysicalDeviceProperties2::default().push_next(&mut rt_pipeline_properties);

		unsafe {
			instance
				.get_physical_device_properties2(physical_device, &mut physical_device_properties2);
		}

		let adapter_info = super::AdapterInfo {
			name: physical_device_properties2
				.properties
				.device_name_as_c_str()
				.unwrap_or_default()
				.to_string_lossy()
				.into_owned(),
			vendor: physical_device_properties2.properties.vendor_id,
			device: physical_device_properties2.properties.device_id,
			backend: super::Backend::Vulkan,
		};

		let capabilities = super::Capabilities {
			raytracing: true, // TODO: Hardcoded
		};

		Self {
			adapter_info,
			capabilities,
			entry,
			instance,
			device,
			physical_device,
			allocator,
			graphics_queue,
			command_pool,
			acceleration_structure_ext,
			ray_tracing_pipeline_ext,
			debug_utils_ext,
			//rt_pipeline_properties,
		}
	}

	fn create_surface(
		&mut self,
		desc: &super::SurfaceDesc,
		window_handle: super::WindowHandle,
	) -> Result<Self::Surface, super::Error> {
		// TODO: Remove dependency on win32, move to platform layer
		let hinstance = unsafe { GetModuleHandleW(None) }.unwrap();
		let surface_create_info = vk::Win32SurfaceCreateInfoKHR::default()
			.hinstance(hinstance.0 as vk::HINSTANCE)
			.hwnd(window_handle.0 as vk::HWND);

		let win32_surface_ext = ash::khr::win32_surface::Instance::new(&self.entry, &self.instance);
		let surface =
			unsafe { win32_surface_ext.create_win32_surface(&surface_create_info, None) }.unwrap();

		let swapchain_ext = ash::khr::swapchain::Device::new(&self.instance, &self.device);

		let present_mode = match desc.present_mode {
			super::PresentMode::Immediate => vk::PresentModeKHR::IMMEDIATE,
			super::PresentMode::Mailbox => vk::PresentModeKHR::MAILBOX,
			super::PresentMode::Fifo => vk::PresentModeKHR::FIFO,
		};

		let info = vk::SwapchainCreateInfoKHR::default()
			.surface(surface)
			.min_image_count(desc.num_buffers)
			.image_format(map_format(desc.format))
			.image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR) // TODO: Hardcoded
			.image_extent(vk::Extent2D {
				width: desc.size[0],
				height: desc.size[1],
			})
			.image_array_layers(1)
			.image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
			.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
			.pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
			.composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
			.present_mode(present_mode)
			.clipped(true);

		let swapchain = unsafe { swapchain_ext.create_swapchain(&info, None) }.unwrap();

		let swapchain_images = unsafe { swapchain_ext.get_swapchain_images(swapchain) }.unwrap();

		let textures = swapchain_images
			.iter()
			.map(|image| {
				// TODO: todo!("Image views");

				Texture {
					image: *image,
					allocation: unsafe { std::mem::zeroed() }, // TODO: Terrible, fix this
					srv_index: None,
					uav_index: None,
					rtv: None,
					dsv: None,
				}
			})
			.collect::<Vec<Texture>>();

		let acquire_semaphores = swapchain_images
			.iter()
			.map(|image| {
				let semaphore_create_info = vk::SemaphoreCreateInfo::default();
				unsafe { self.device.create_semaphore(&semaphore_create_info, None) }.unwrap()
			})
			.collect::<Vec<vk::Semaphore>>();

		let next_semaphore = {
			let semaphore_create_info = vk::SemaphoreCreateInfo::default();
			unsafe { self.device.create_semaphore(&semaphore_create_info, None) }.unwrap()
		};

		Ok(Surface {
			swapchain_ext,
			swapchain,
			textures,
			acquire_semaphores,
			next_semaphore,
			bb_index: 0,
		})
	}

	fn create_cmd_list(&mut self, num_buffers: u32) -> Self::CmdList {
		let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
			.command_pool(self.command_pool)
			.level(vk::CommandBufferLevel::PRIMARY)
			.command_buffer_count(num_buffers);

		let command_buffers = unsafe {
			self.device
				.allocate_command_buffers(&command_buffer_allocate_info)
		}
		.unwrap();

		CmdList {
			bb_index: 0,
			command_buffers,
			device: self.device.clone(),
			acceleration_structure_ext: self.acceleration_structure_ext.clone(),
			ray_tracing_pipeline_ext: self.ray_tracing_pipeline_ext.clone(),
			debug_utils_ext: None, // TODO: pass this in
		}
	}

	fn create_buffer(&mut self, desc: &super::BufferDesc) -> Result<Self::Buffer, super::Error> {
		let default_flags = vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS; // TODO: Hardcoded default flags

		let buffer_info = vk::BufferCreateInfo::default()
			.size(desc.size as _)
			.usage(map_buffer_usage_flags(desc.usage) | default_flags)
			.sharing_mode(vk::SharingMode::EXCLUSIVE);

		let buffer = unsafe { self.device.create_buffer(&buffer_info, None) }.unwrap();

		let allocation = self
			.allocator
			.allocate(&allocator::AllocationCreateDesc {
				requirements: unsafe { self.device.get_buffer_memory_requirements(buffer) },
				location: match desc.memory {
					super::Memory::GpuOnly => gpu_allocator::MemoryLocation::GpuOnly,
					super::Memory::CpuToGpu => gpu_allocator::MemoryLocation::CpuToGpu,
					super::Memory::GpuToCpu => gpu_allocator::MemoryLocation::GpuToCpu,
				},
				linear: true,
				name: "Allocation",
				allocation_scheme: allocator::AllocationScheme::GpuAllocatorManaged,
			})
			.unwrap();

		unsafe {
			self.device
				.bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
		}
		.unwrap();

		let mapped_ptr = allocation
			.mapped_ptr()
			.map_or(std::ptr::null_mut(), |ptr| ptr.as_ptr() as _);

		let device_address = unsafe {
			self.device
				.get_buffer_device_address(&vk::BufferDeviceAddressInfo::default().buffer(buffer))
		};

		Ok(Buffer {
			buffer,
			allocation,
			srv_index: None, // TODO
			uav_index: None,
			cpu_ptr: mapped_ptr,
			gpu_ptr: device_address,
		})
	}

	fn create_texture(&mut self, desc: &super::TextureDesc) -> Result<Self::Texture, super::Error> {
		let create_info = vk::ImageCreateInfo::default()
			.image_type(map_image_type(desc))
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

		let allocation = self
			.allocator
			.allocate(&allocator::AllocationCreateDesc {
				requirements: unsafe { self.device.get_image_memory_requirements(image) },
				location: gpu_allocator::MemoryLocation::GpuOnly,
				linear: true,
				name: "Allocation",
				allocation_scheme: allocator::AllocationScheme::GpuAllocatorManaged,
			})
			.unwrap();

		unsafe {
			self.device
				.bind_image_memory(image, allocation.memory(), allocation.offset())
		}
		.unwrap();

		let rtv = desc
			.usage
			.contains(super::TextureUsage::RENDER_TARGET)
			.then(|| {
				let create_info = vk::ImageViewCreateInfo::default();
				todo!();
				unsafe { self.device.create_image_view(&create_info, None) }.unwrap()
			});

		let dsv = desc
			.usage
			.contains(super::TextureUsage::DEPTH_STENCIL)
			.then(|| {
				let create_info = vk::ImageViewCreateInfo::default();
				todo!();
				unsafe { self.device.create_image_view(&create_info, None) }.unwrap()
			});

		Ok(Texture {
			image,
			allocation,
			srv_index: todo!(),
			uav_index: todo!(),
			rtv,
			dsv,
		})
	}

	fn create_sampler(&mut self, desc: &super::SamplerDesc) -> Result<Self::Sampler, super::Error> {
		let mut create_info = vk::SamplerCreateInfo::default()
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
			create_info = create_info.border_color(map_border_color(border_color));
		}

		let sampler = unsafe { self.device.create_sampler(&create_info, None) }.unwrap();

		Ok(Sampler { sampler })
	}

	fn create_acceleration_structure(
		&mut self,
		desc: &super::AccelerationStructureDesc<Self>,
	) -> Result<Self::AccelerationStructure, super::Error> {
		let create_info = vk::AccelerationStructureCreateInfoKHR::default()
			.buffer(desc.buffer.buffer)
			.offset(desc.offset as _)
			.size(desc.size as _)
			.ty(match desc.ty {
				super::AccelerationStructureType::TopLevel => {
					vk::AccelerationStructureTypeKHR::TOP_LEVEL
				}
				super::AccelerationStructureType::BottomLevel => {
					vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL
				}
			});

		let acceleration_structure = unsafe {
			self.acceleration_structure_ext
				.create_acceleration_structure(&create_info, None)
		}
		.unwrap();

		let srv_index = todo!();

		let gpu_ptr = unsafe {
			self.acceleration_structure_ext
				.get_acceleration_structure_device_address(
					&vk::AccelerationStructureDeviceAddressInfoKHR::default()
						.acceleration_structure(acceleration_structure),
				)
		};

		Ok(AccelerationStructure {
			acceleration_structure,
			srv_index,
			gpu_ptr,
		})
	}

	fn create_graphics_pipeline(
		&self,
		desc: &super::GraphicsPipelineDesc,
	) -> Result<Self::GraphicsPipeline, super::Error> {
		let mut shader_modules = Vec::new();
		let mut shader_stages = Vec::new();

		if let Some(shader) = desc.vs {
			let create_info = vk::ShaderModuleCreateInfo {
				code_size: shader.len(),
				p_code: shader.as_ptr() as *const u32,
				..Default::default()
			};

			let module = unsafe { self.device.create_shader_module(&create_info, None) }.unwrap();

			shader_modules.push(module);

			let stage = vk::PipelineShaderStageCreateInfo::default()
				.stage(vk::ShaderStageFlags::VERTEX)
				.module(module)
				.name(c"main");

			shader_stages.push(stage);
		}

		if let Some(shader) = desc.ps {
			let create_info = vk::ShaderModuleCreateInfo {
				code_size: shader.len(),
				p_code: shader.as_ptr() as *const u32,
				..Default::default()
			};

			let module = unsafe { self.device.create_shader_module(&create_info, None) }.unwrap();

			shader_modules.push(module);

			let stage = vk::PipelineShaderStageCreateInfo::default()
				.stage(vk::ShaderStageFlags::FRAGMENT)
				.module(module)
				.name(c"main");

			shader_stages.push(stage);
		}

		let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default();

		let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default()
			.topology(map_topology(desc.topology));

		let viewport_state = vk::PipelineViewportStateCreateInfo::default()
			.viewport_count(1)
			.scissor_count(1);

		let mut rasterization_state = vk::PipelineRasterizationStateCreateInfo::default()
			.depth_clamp_enable(true)
			.polygon_mode(map_polygon_mode(&desc.rasterizer.polygon_mode))
			.cull_mode(map_cull_mode(&desc.rasterizer.cull_mode))
			.front_face(if desc.rasterizer.front_ccw {
				vk::FrontFace::COUNTER_CLOCKWISE
			} else {
				vk::FrontFace::CLOCKWISE
			})
			.line_width(1.0);

		let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
			.rasterization_samples(vk::SampleCountFlags::TYPE_1)
			.sample_mask(&[1]);

		let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::default()
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
				.depth_bias_constant_factor(desc.rasterizer.depth_bias.constant)
				.depth_bias_clamp(desc.rasterizer.depth_bias.clamp)
				.depth_bias_slope_factor(desc.rasterizer.depth_bias.slope);
		}

		let color_attachments = desc
			.color_attachments
			.iter()
			.map(|attachment| {
				let mut vk_attachment = vk::PipelineColorBlendAttachmentState::default()
					.color_write_mask(vk::ColorComponentFlags::from_raw(
						attachment.write_mask.bits() as u32,
					));

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

				vk_attachment
			})
			.collect::<Vec<_>>();

		let color_blend_state =
			vk::PipelineColorBlendStateCreateInfo::default().attachments(&color_attachments);

		let dynamic_state = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&[
			vk::DynamicState::VIEWPORT,
			vk::DynamicState::SCISSOR,
			vk::DynamicState::BLEND_CONSTANTS,
			vk::DynamicState::STENCIL_REFERENCE,
		]);

		let color_target_formats = desc
			.color_attachments
			.iter()
			.map(|attachment| map_format(attachment.format))
			.collect::<Vec<_>>();

		let depth_stencil_format = map_format(desc.depth_stencil.format);

		let mut rendering = vk::PipelineRenderingCreateInfo::default()
			.color_attachment_formats(&color_target_formats)
			.depth_attachment_format(depth_stencil_format)
			.stencil_attachment_format(depth_stencil_format);

		let create_info = vk::GraphicsPipelineCreateInfo::default()
			.stages(&shader_stages)
			// TODO: layout
			.vertex_input_state(&vertex_input_state)
			.input_assembly_state(&input_assembly_state)
			.viewport_state(&viewport_state)
			.rasterization_state(&rasterization_state)
			.multisample_state(&multisample_state)
			.depth_stencil_state(&depth_stencil_state)
			.color_blend_state(&color_blend_state)
			.dynamic_state(&dynamic_state)
			.push_next(&mut rendering);

		let pipeline = unsafe {
			self.device
				.create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None)
		}
		.unwrap()[0];

		for module in shader_modules {
			unsafe { self.device.destroy_shader_module(module, None) };
		}

		Ok(GraphicsPipeline { pipeline })
	}

	fn create_compute_pipeline(
		&self,
		desc: &super::ComputePipelineDesc,
	) -> Result<Self::ComputePipeline, super::Error> {
		let shader_module_create_info = vk::ShaderModuleCreateInfo {
			p_code: desc.cs.as_ptr() as _,
			code_size: desc.cs.len(),
			..Default::default()
		};

		let shader_module = unsafe {
			self.device
				.create_shader_module(&shader_module_create_info, None)
		}
		.unwrap();

		let shader_stage = vk::PipelineShaderStageCreateInfo::default()
			.stage(vk::ShaderStageFlags::COMPUTE)
			.module(shader_module)
			.name(c"main");

		let create_info = vk::ComputePipelineCreateInfo::default()
			.stage(shader_stage)
			.layout(todo!());

		let pipeline = unsafe {
			self.device
				.create_compute_pipelines(vk::PipelineCache::null(), &[create_info], None)
		}
		.unwrap()[0];

		unsafe { self.device.destroy_shader_module(shader_module, None) };

		Ok(ComputePipeline { pipeline })
	}

	fn create_raytracing_pipeline(
		&self,
		desc: &super::RaytracingPipelineDesc,
	) -> Result<Self::RaytracingPipeline, super::Error> {
		let modules = desc
			.libraries
			.iter()
			.map(|library| {
				let create_info = vk::ShaderModuleCreateInfo {
					code_size: library.shader.len(),
					p_code: library.shader.as_ptr() as *const u32,
					..Default::default()
				};

				unsafe { self.device.create_shader_module(&create_info, None) }.unwrap()
			})
			.collect::<Vec<_>>();

		let names = desc
			.libraries
			.iter()
			.map(|library| CString::new(library.entry.clone()).unwrap())
			.collect::<Vec<_>>();

		let stages = desc
			.libraries
			.iter()
			.enumerate()
			.map(|(i, library)| {
				let stage = match library.ty {
					super::ShaderType::Raygen => vk::ShaderStageFlags::RAYGEN_KHR,
					super::ShaderType::Miss => vk::ShaderStageFlags::MISS_KHR,
					super::ShaderType::Intersection => vk::ShaderStageFlags::INTERSECTION_KHR,
					super::ShaderType::ClosestHit => vk::ShaderStageFlags::CLOSEST_HIT_KHR,
					super::ShaderType::AnyHit => vk::ShaderStageFlags::ANY_HIT_KHR,
					super::ShaderType::Callable => vk::ShaderStageFlags::CALLABLE_KHR,
					_ => panic!(),
				};

				vk::PipelineShaderStageCreateInfo::default()
					.stage(stage)
					.module(modules[i])
					.name(names[i].as_c_str())
			})
			.collect::<Vec<_>>();

		let groups = desc
			.groups
			.iter()
			.map(|group| {
				let ty = match group.ty {
					super::ShaderGroupType::General => vk::RayTracingShaderGroupTypeKHR::GENERAL,
					super::ShaderGroupType::Triangles => {
						vk::RayTracingShaderGroupTypeKHR::TRIANGLES_HIT_GROUP
					}
					super::ShaderGroupType::Procedural => {
						vk::RayTracingShaderGroupTypeKHR::PROCEDURAL_HIT_GROUP
					}
				};

				vk::RayTracingShaderGroupCreateInfoKHR::default()
					.ty(ty)
					.general_shader(group.general.unwrap_or(vk::SHADER_UNUSED_KHR))
					.closest_hit_shader(group.closest_hit.unwrap_or(vk::SHADER_UNUSED_KHR))
					.any_hit_shader(group.any_hit.unwrap_or(vk::SHADER_UNUSED_KHR))
					.intersection_shader(group.intersection.unwrap_or(vk::SHADER_UNUSED_KHR))
			})
			.collect::<Vec<_>>();

		let create_info = vk::RayTracingPipelineCreateInfoKHR::default()
			.stages(&stages)
			.groups(&groups)
			.max_pipeline_ray_recursion_depth(desc.max_trace_recursion_depth)
			.layout(todo!());

		let pipeline = unsafe {
			self.ray_tracing_pipeline_ext.create_ray_tracing_pipelines(
				vk::DeferredOperationKHR::null(),
				vk::PipelineCache::null(),
				&[create_info],
				None,
			)
		}
		.unwrap()[0];

		for module in modules {
			unsafe { self.device.destroy_shader_module(module, None) };
		}

		Ok(RaytracingPipeline { pipeline })
	}

	fn create_texture_view(
		&mut self,
		desc: &super::TextureViewDesc,
		texture: &Self::Texture,
	) -> super::TextureView {
		todo!()
	}

	fn submit(&self, cmd: &Self::CmdList) {
		let command_buffer = cmd.command_buffers[cmd.bb_index];

		unsafe { self.device.end_command_buffer(command_buffer) }.unwrap();

		let command_buffers = [command_buffer];

		let submit_infos = vk::SubmitInfo::default().command_buffers(&command_buffers);

		unsafe {
			self.device
				.queue_submit(self.graphics_queue, &[submit_infos], vk::Fence::null())
		}
		.unwrap();
	}

	fn queue_wait(&self) {
		unsafe { self.device.queue_wait_idle(self.graphics_queue) }.unwrap();
	}

	fn adapter_info(&self) -> &super::AdapterInfo {
		&self.adapter_info
	}

	fn capabilities(&self) -> &super::Capabilities {
		&self.capabilities
	}

	fn acceleration_structure_sizes(
		&self,
		desc: &super::AccelerationStructureBuildInputs,
	) -> super::AccelerationStructureSizes {
		let info = AccelerationStructureInfo::build(desc);

		let mut size_info = vk::AccelerationStructureBuildSizesInfoKHR::default();
		unsafe {
			self.acceleration_structure_ext
				.get_acceleration_structure_build_sizes(
					vk::AccelerationStructureBuildTypeKHR::DEVICE,
					&info.build_info,
					&info.max_primitve_counts,
					&mut size_info,
				)
		}

		super::AccelerationStructureSizes {
			acceleration_structure_size: size_info.acceleration_structure_size as _,
			update_scratch_size: size_info.update_scratch_size as _,
			build_scratch_size: size_info.build_scratch_size as _,
		}
	}
}

pub struct CmdList {
	bb_index: usize,
	command_buffers: Vec<vk::CommandBuffer>,
	device: ash::Device,

	// TODO: Initialize variables, move to Device
	acceleration_structure_ext: ash::khr::acceleration_structure::Device,
	ray_tracing_pipeline_ext: ash::khr::ray_tracing_pipeline::Device,
	debug_utils_ext: Option<ash::ext::debug_utils::Device>,
}

struct AccelerationStructureInfo<'a> {
	build_info: vk::AccelerationStructureBuildGeometryInfoKHR<'a>,
	_geometry: Box<[vk::AccelerationStructureGeometryKHR<'a>]>,
	build_range_infos: Box<[vk::AccelerationStructureBuildRangeInfoKHR]>,
	max_primitve_counts: Box<[u32]>,
}

impl<'a> AccelerationStructureInfo<'a> {
	fn map_aabbs(
		aabbs: &'_ super::AccelerationStructureAABBs,
	) -> vk::AccelerationStructureGeometryKHR<'_> {
		let aabbs_data = vk::AccelerationStructureGeometryAabbsDataKHR::default()
			.data(vk::DeviceOrHostAddressConstKHR {
				device_address: aabbs.data.0,
			})
			.stride(aabbs.stride as _);

		vk::AccelerationStructureGeometryKHR::default()
			.geometry_type(vk::GeometryTypeKHR::AABBS)
			.geometry(vk::AccelerationStructureGeometryDataKHR { aabbs: aabbs_data })
			.flags(map_acceleration_structure_geometry_flags(aabbs.flags))
	}

	fn map_triangles(
		triangles: &'_ super::AccelerationStructureTriangles,
	) -> vk::AccelerationStructureGeometryKHR<'_> {
		let triangles_data = vk::AccelerationStructureGeometryTrianglesDataKHR::default()
			.vertex_format(map_format(triangles.vertex_format))
			.vertex_data(vk::DeviceOrHostAddressConstKHR {
				device_address: triangles.vertex_buffer.0,
			})
			.vertex_stride(triangles.vertex_stride as _)
			.max_vertex(triangles.vertex_count as _)
			.index_type(map_index_format(triangles.index_format))
			.index_data(vk::DeviceOrHostAddressConstKHR {
				device_address: triangles.index_buffer.0,
			})
			.transform_data(vk::DeviceOrHostAddressConstKHR {
				device_address: triangles.transform.0,
			});

		vk::AccelerationStructureGeometryKHR::default()
			.geometry_type(vk::GeometryTypeKHR::TRIANGLES)
			.geometry(vk::AccelerationStructureGeometryDataKHR {
				triangles: triangles_data,
			})
			.flags(map_acceleration_structure_geometry_flags(triangles.flags))
	}

	fn map_instances(
		instances: &'_ super::AccelerationStructureInstances,
	) -> vk::AccelerationStructureGeometryKHR<'_> {
		let instances_data = vk::AccelerationStructureGeometryInstancesDataKHR::default().data(
			vk::DeviceOrHostAddressConstKHR {
				device_address: instances.data.0,
			},
		);

		vk::AccelerationStructureGeometryKHR::default()
			.geometry_type(vk::GeometryTypeKHR::INSTANCES)
			.geometry(vk::AccelerationStructureGeometryDataKHR {
				instances: instances_data,
			})
	}

	fn build(desc: &'a super::AccelerationStructureBuildInputs) -> Self {
		let (geometries, max_primitve_counts, ranges) = match &desc.entries {
			super::AccelerationStructureEntries::Instances(instances) => {
				let ranges = vk::AccelerationStructureBuildRangeInfoKHR::default()
					.primitive_count(instances.count as u32);

				(
					vec![Self::map_instances(instances)],
					vec![instances.count as u32],
					vec![ranges],
				)
			}
			super::AccelerationStructureEntries::Triangles(in_geometries) => {
				let geometries: Vec<vk::AccelerationStructureGeometryKHR> = in_geometries
					.iter()
					.map(|triangles| Self::map_triangles(triangles))
					.collect();

				let primitive_counts: Vec<u32> = in_geometries
					.iter()
					.map(|triangles| {
						if triangles.index_buffer.is_null() {
							triangles.vertex_count as u32
						} else {
							triangles.index_count as u32 / 3
						}
					})
					.collect();

				let ranges: Vec<vk::AccelerationStructureBuildRangeInfoKHR> = in_geometries
					.iter()
					.map(|triangles| {
						vk::AccelerationStructureBuildRangeInfoKHR::default().primitive_count(
							if triangles.index_buffer.is_null() {
								triangles.vertex_count as u32
							} else {
								triangles.index_count as u32 / 3
							},
						)
					})
					.collect();

				(geometries, primitive_counts, ranges)
			}
			super::AccelerationStructureEntries::AABBs(aabbs) => {
				let geometries: Vec<vk::AccelerationStructureGeometryKHR> =
					aabbs.iter().map(|aabb| Self::map_aabbs(aabb)).collect();

				let primitive_counts: Vec<u32> =
					aabbs.iter().map(|aabb| aabb.count as u32).collect();

				let ranges: Vec<vk::AccelerationStructureBuildRangeInfoKHR> = aabbs
					.iter()
					.map(|aabb| {
						vk::AccelerationStructureBuildRangeInfoKHR::default()
							.primitive_count(aabb.count as u32)
					})
					.collect();

				(geometries, primitive_counts, ranges)
			}
		};

		let ty = match &desc.entries {
			super::AccelerationStructureEntries::Instances(_) => {
				vk::AccelerationStructureTypeKHR::TOP_LEVEL
			}
			_ => vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL,
		};

		let mut build_info = vk::AccelerationStructureBuildGeometryInfoKHR::default()
			.ty(ty)
			.flags(map_acceleration_structure_flags(desc.flags));

		build_info.geometry_count = geometries.len() as _;
		build_info.p_geometries = geometries.as_ptr();

		Self {
			build_info,
			max_primitve_counts: max_primitve_counts.into_boxed_slice(),
			build_range_infos: ranges.into_boxed_slice(),
			_geometry: geometries.into_boxed_slice(),
		}
	}
}
