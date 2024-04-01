use crate::os::NativeHandle;

use std::ffi::CString;
use std::mem::ManuallyDrop;
use std::ops::Range;
use std::result;
use std::str;

use windows::{
	core::*, Win32::Foundation::*,
	Win32::Graphics::{Direct3D::*, Direct3D12::*, Dxgi::*, Dxgi::Common::*},
	Win32::System::{LibraryLoader::*, Threading::*},
};

#[no_mangle]
pub static D3D12SDKVersion: u32 = 611;
#[no_mangle]
pub static D3D12SDKPath: &[u8; 9] = b".\\D3D12\\\0";

#[derive(Clone, Copy)]
struct WinPixEventRuntime {
	begin_event: extern "stdcall" fn(*const std::ffi::c_void, u64, PSTR) -> i32,
	end_event: extern "stdcall" fn(*const std::ffi::c_void) -> i32,
	set_marker: extern "stdcall" fn(*const std::ffi::c_void, u64, PSTR) -> i32,
}

impl WinPixEventRuntime {
	pub fn load() -> Option<Self> {
		unsafe {
			let module = LoadLibraryA(s!("WinPixEventRuntime.dll")).ok()?;

			Some(Self {
				begin_event: std::mem::transmute(GetProcAddress(module, s!("PIXBeginEventOnCommandList"))?),
				end_event: std::mem::transmute(GetProcAddress(module, s!("PIXEndEventOnCommandList"))?),
				set_marker: std::mem::transmute(GetProcAddress(module, s!("PIXSetMarkerOnCommandList"))?),
			})
		}
	}

	pub fn set_marker_on_command_list(&self, command_list: &ID3D12GraphicsCommandList7, color: u64, name: &str) {
		let null_name = CString::new(name).unwrap();
		(self.set_marker)(command_list.as_raw(), color, PSTR(null_name.as_ptr() as _));
	}

	pub fn begin_event_on_command_list(&self, command_list: &ID3D12GraphicsCommandList7, color: u64, name: &str) {
		let null_name = CString::new(name).unwrap();
		(self.begin_event)(command_list.as_raw(), color, PSTR(null_name.as_ptr() as _));
	}

	pub fn end_event_on_command_list(&self, command_list: &ID3D12GraphicsCommandList7) {
		(self.end_event)(command_list.as_raw());
	}
}

pub struct Device {
	adapter_info: super::AdapterInfo,
	capabilities: super::Capabilities,
	dxgi_factory: IDXGIFactory6,
	device: ID3D12Device10,
	command_allocator: ID3D12CommandAllocator,
	command_list: ID3D12GraphicsCommandList7,
	command_queue: ID3D12CommandQueue,
	pix: Option<WinPixEventRuntime>,
	resource_heap: Heap,
	sampler_heap: Heap,
	rtv_heap: Heap,
	dsv_heap: Heap,
}

#[derive(Clone)]
pub struct SwapChain {
	size: [u32; 2],
	num_buffers: u32,
	flags: u32,
	bb_index: usize,
	swap_chain: IDXGISwapChain3,
	backbuffer_textures: Vec<Texture>,
	fence: ID3D12Fence,
	fence_last_signalled_value: u64,
	fence_event: HANDLE,
	frame_fence_value: Vec<u64>,
	require_wait: Vec<bool>,
}

pub struct AccelerationStructure {
	resource: ID3D12Resource,
	srv_index: Option<usize>,
}

pub struct GraphicsPipeline {
	pipeline_state: ID3D12PipelineState,
	root_signature: ID3D12RootSignature,
	topology: D3D_PRIMITIVE_TOPOLOGY,
}

pub struct ComputePipeline {
	pipeline_state: ID3D12PipelineState,
	root_signature: ID3D12RootSignature,
}

pub struct RaytracingPipeline {
	state_object: ID3D12StateObject,
	root_signature: ID3D12RootSignature,
}

#[derive(Clone)]
pub struct CmdList {
	bb_index: usize,
	command_allocator: Vec<ID3D12CommandAllocator>,
	command_list: Vec<ID3D12GraphicsCommandList7>,
	pix: Option<WinPixEventRuntime>,
	resource_heap_base: D3D12_GPU_DESCRIPTOR_HANDLE, // TODO: Get from device itself.
}

#[derive(Clone)]
pub struct Buffer {
	resource: ID3D12Resource,
	size: u64,
	srv_index: Option<usize>,
	uav_index: Option<usize>,
	mapped_ptr: *mut u8,
}

#[derive(Clone)]
pub struct Texture {
	desc: super::TextureDesc,
	resource: ID3D12Resource,
	srv_index: Option<usize>,
	uav_index: Option<usize>,
	rtv: Option<D3D12_CPU_DESCRIPTOR_HANDLE>,
	dsv: Option<D3D12_CPU_DESCRIPTOR_HANDLE>,
}

#[derive(Clone)]
pub struct Sampler {
	_sampler: D3D12_CPU_DESCRIPTOR_HANDLE,
}

fn map_box(offset: &[u32; 3], size: &[u32; 3]) -> D3D12_BOX {
	D3D12_BOX {
		left: offset[0],
		top: offset[1],
		front: offset[2],
		right: offset[0] + size[0],
		bottom: offset[1] + size[1],
		back: offset[2] + size[2],
	}
}

fn map_power_preference(power_preference: super::PowerPreference) -> DXGI_GPU_PREFERENCE {
	match power_preference {
		super::PowerPreference::None => DXGI_GPU_PREFERENCE_UNSPECIFIED,
		super::PowerPreference::LowPower => DXGI_GPU_PREFERENCE_MINIMUM_POWER,
		super::PowerPreference::HighPerformance => DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE,
	}
}

fn map_format(format: super::Format) -> DXGI_FORMAT {
	match format {
		super::Format::Unknown => DXGI_FORMAT_UNKNOWN,

		super::Format::R8UNorm => DXGI_FORMAT_R8_UNORM,
		super::Format::R8SNorm => DXGI_FORMAT_R8_SNORM,
		super::Format::R8UInt  => DXGI_FORMAT_R8_UINT,
		super::Format::R8SInt  => DXGI_FORMAT_R8_SINT,

		super::Format::R16UNorm => DXGI_FORMAT_R16_UNORM,
		super::Format::R16SNorm => DXGI_FORMAT_R16_SNORM,
		super::Format::R16UInt  => DXGI_FORMAT_R16_UINT,
		super::Format::R16SInt  => DXGI_FORMAT_R16_SINT,
		super::Format::R16Float => DXGI_FORMAT_R16_FLOAT,

		super::Format::R32UInt  => DXGI_FORMAT_R32_UINT,
		super::Format::R32SInt  => DXGI_FORMAT_R32_SINT,
		super::Format::R32Float => DXGI_FORMAT_R32_FLOAT,

		super::Format::RG8UNorm => DXGI_FORMAT_R8G8_UNORM,
		super::Format::RG8SNorm => DXGI_FORMAT_R8G8_SNORM,
		super::Format::RG8UInt  => DXGI_FORMAT_R8G8_UINT,
		super::Format::RG8SInt  => DXGI_FORMAT_R8G8_SINT,

		super::Format::RG16UNorm => DXGI_FORMAT_R16G16_UNORM,
		super::Format::RG16SNorm => DXGI_FORMAT_R16G16_SNORM,
		super::Format::RG16UInt  => DXGI_FORMAT_R16G16_UINT,
		super::Format::RG16SInt  => DXGI_FORMAT_R16G16_SINT,
		super::Format::RG16Float => DXGI_FORMAT_R16G16_FLOAT,

		super::Format::RG32UInt  => DXGI_FORMAT_R32G32_UINT,
		super::Format::RG32SInt  => DXGI_FORMAT_R32G32_SINT,
		super::Format::RG32Float => DXGI_FORMAT_R32G32_FLOAT,

		super::Format::RGB32UInt  => DXGI_FORMAT_R32G32B32_UINT,
		super::Format::RGB32SInt  => DXGI_FORMAT_R32G32B32_SINT,
		super::Format::RGB32Float => DXGI_FORMAT_R32G32B32_FLOAT,

		super::Format::RGBA8UNorm => DXGI_FORMAT_R8G8B8A8_UNORM,
		super::Format::RGBA8SNorm => DXGI_FORMAT_R8G8B8A8_SNORM,
		super::Format::RGBA8UInt  => DXGI_FORMAT_R8G8B8A8_UINT,
		super::Format::RGBA8SInt  => DXGI_FORMAT_R8G8B8A8_SINT,

		super::Format::RGBA16UNorm => DXGI_FORMAT_R16G16B16A16_UNORM,
		super::Format::RGBA16SNorm => DXGI_FORMAT_R16G16B16A16_SNORM,
		super::Format::RGBA16UInt  => DXGI_FORMAT_R16G16B16A16_UINT,
		super::Format::RGBA16SInt  => DXGI_FORMAT_R16G16B16A16_SINT,
		super::Format::RGBA16Float => DXGI_FORMAT_R16G16B16A16_FLOAT,

		super::Format::RGBA32UInt  => DXGI_FORMAT_R32G32B32A32_UINT,
		super::Format::RGBA32SInt  => DXGI_FORMAT_R32G32B32A32_SINT,
		super::Format::RGBA32Float => DXGI_FORMAT_R32G32B32A32_FLOAT,

		super::Format::BGRA8UNorm => DXGI_FORMAT_B8G8R8A8_UNORM,

		super::Format::D16UNorm          => DXGI_FORMAT_D16_UNORM,
		super::Format::D24UNormS8UInt    => DXGI_FORMAT_D24_UNORM_S8_UINT,
		super::Format::D32Float          => DXGI_FORMAT_D32_FLOAT,
		super::Format::D32FloatS8UIntX24 => DXGI_FORMAT_D32_FLOAT_S8X24_UINT,
	}
}

fn map_filter_mode(filter_mode: super::FilterMode) -> D3D12_FILTER_TYPE {
	match filter_mode {
		super::FilterMode::Nearest => D3D12_FILTER_TYPE_POINT,
		super::FilterMode::Linear  => D3D12_FILTER_TYPE_LINEAR,
	}
}

fn map_sampler_filter(desc: &super::SamplerDesc) -> D3D12_FILTER {
	let reduction = match desc.compare {
		Some(_) => D3D12_FILTER_REDUCTION_TYPE_COMPARISON,
		None    => D3D12_FILTER_REDUCTION_TYPE_STANDARD,
	};

	let mut filter = D3D12_FILTER(
		map_filter_mode(desc.filter_min).0 << D3D12_MIN_FILTER_SHIFT |
		map_filter_mode(desc.filter_mag).0 << D3D12_MAG_FILTER_SHIFT |
		map_filter_mode(desc.filter_mip).0 << D3D12_MIP_FILTER_SHIFT |
		reduction.0 << D3D12_FILTER_REDUCTION_TYPE_SHIFT
	);

	if desc.max_anisotropy > 1 {
		filter.0 |= D3D12_FILTER_ANISOTROPIC.0;
	}

	filter
}

fn map_address_mode(address_mode: super::AddressMode) -> D3D12_TEXTURE_ADDRESS_MODE {
	match address_mode {
		super::AddressMode::Clamp      => D3D12_TEXTURE_ADDRESS_MODE_CLAMP,
		super::AddressMode::Repeat     => D3D12_TEXTURE_ADDRESS_MODE_WRAP,
		super::AddressMode::Mirror     => D3D12_TEXTURE_ADDRESS_MODE_MIRROR,
		super::AddressMode::MirrorOnce => D3D12_TEXTURE_ADDRESS_MODE_MIRROR_ONCE,
		super::AddressMode::Border     => D3D12_TEXTURE_ADDRESS_MODE_BORDER,
	}
}

fn map_border_color(border_color: super::BorderColor) -> [f32; 4] {
	match border_color {
		super::BorderColor::TransparentBlack => [0.0, 0.0, 0.0, 0.0],
		super::BorderColor::OpaqueBlack      => [0.0, 0.0, 0.0, 1.0],
		super::BorderColor::White            => [1.0, 1.0, 1.0, 1.0],
	}
}

fn map_static_border_color(border_color: super::BorderColor) -> D3D12_STATIC_BORDER_COLOR {
	match border_color {
		super::BorderColor::TransparentBlack => D3D12_STATIC_BORDER_COLOR_TRANSPARENT_BLACK,
		super::BorderColor::OpaqueBlack      => D3D12_STATIC_BORDER_COLOR_OPAQUE_BLACK,
		super::BorderColor::White            => D3D12_STATIC_BORDER_COLOR_OPAQUE_WHITE,
	}
}

fn map_compare_op(compare_op: super::CompareOp) -> D3D12_COMPARISON_FUNC {
	match compare_op {
		super::CompareOp::Never        => D3D12_COMPARISON_FUNC_NEVER,
		super::CompareOp::Always       => D3D12_COMPARISON_FUNC_ALWAYS,
		super::CompareOp::Equal        => D3D12_COMPARISON_FUNC_EQUAL,
		super::CompareOp::NotEqual     => D3D12_COMPARISON_FUNC_NOT_EQUAL,
		super::CompareOp::Less         => D3D12_COMPARISON_FUNC_LESS,
		super::CompareOp::LessEqual    => D3D12_COMPARISON_FUNC_LESS_EQUAL,
		super::CompareOp::Greater      => D3D12_COMPARISON_FUNC_GREATER,
		super::CompareOp::GreaterEqual => D3D12_COMPARISON_FUNC_GREATER_EQUAL,
	}
}

fn map_address_compare_op(op: Option<super::CompareOp>) -> D3D12_COMPARISON_FUNC {
	op.map_or(D3D12_COMPARISON_FUNC_ALWAYS, map_compare_op)
}

fn map_resource_dimension(desc: &super::TextureDesc) -> D3D12_RESOURCE_DIMENSION {
	if desc.depth > 1 { return D3D12_RESOURCE_DIMENSION_TEXTURE3D; }
	if desc.height > 1 { return D3D12_RESOURCE_DIMENSION_TEXTURE2D; }
	return D3D12_RESOURCE_DIMENSION_TEXTURE1D;
}

fn map_srv_dimension(desc: &super::TextureDesc) -> D3D12_SRV_DIMENSION {
	if desc.depth > 1 { return D3D12_SRV_DIMENSION_TEXTURE3D; }
	if desc.height > 1 { return D3D12_SRV_DIMENSION_TEXTURE2D; }
	return D3D12_SRV_DIMENSION_TEXTURE1D;
}

fn map_texture_layout(layout: super::TextureLayout) -> D3D12_BARRIER_LAYOUT {
	match layout {
		super::TextureLayout::Common            => D3D12_BARRIER_LAYOUT_COMMON,
		super::TextureLayout::Present           => D3D12_BARRIER_LAYOUT_PRESENT,
		super::TextureLayout::CopySrc           => D3D12_BARRIER_LAYOUT_COPY_SOURCE,
		super::TextureLayout::CopyDst           => D3D12_BARRIER_LAYOUT_COPY_DEST,
		super::TextureLayout::ShaderResource    => D3D12_BARRIER_LAYOUT_SHADER_RESOURCE,
		super::TextureLayout::UnorderedAccess   => D3D12_BARRIER_LAYOUT_UNORDERED_ACCESS,
		super::TextureLayout::RenderTarget      => D3D12_BARRIER_LAYOUT_RENDER_TARGET,
		super::TextureLayout::DepthStencilWrite => D3D12_BARRIER_LAYOUT_DEPTH_STENCIL_WRITE,
		super::TextureLayout::DepthStencilRead  => D3D12_BARRIER_LAYOUT_DEPTH_STENCIL_READ,
	}
}

fn map_buffer_usage_flags(usage: super::BufferUsage) -> D3D12_RESOURCE_FLAGS {
	let mut dx_flags = D3D12_RESOURCE_FLAG_NONE;

	if usage.contains(super::BufferUsage::UNORDERED_ACCESS)       { dx_flags |= D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS; }
	if usage.contains(super::BufferUsage::ACCELERATION_STRUCTURE) { dx_flags |= D3D12_RESOURCE_FLAG_RAYTRACING_ACCELERATION_STRUCTURE; }

	dx_flags
}

fn map_texture_usage_flags(usage: super::TextureUsage) -> D3D12_RESOURCE_FLAGS {
	let mut dx_flags = D3D12_RESOURCE_FLAG_NONE;

	if usage.contains(super::TextureUsage::RENDER_TARGET)    { dx_flags |= D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET; }
	if usage.contains(super::TextureUsage::DEPTH_STENCIL)    { dx_flags |= D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL; }
	if usage.contains(super::TextureUsage::UNORDERED_ACCESS) { dx_flags |= D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS; }

	dx_flags
}

fn map_topology(topology: super::Topology) -> D3D_PRIMITIVE_TOPOLOGY {
	match topology {
		super::Topology::PointList        => D3D_PRIMITIVE_TOPOLOGY_POINTLIST,
		super::Topology::LineList         => D3D_PRIMITIVE_TOPOLOGY_LINELIST,
		super::Topology::LineStrip        => D3D_PRIMITIVE_TOPOLOGY_LINESTRIP,
		super::Topology::TriangleList     => D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
		super::Topology::TriangleStrip    => D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP,
	}
}

fn map_topology_type(topology: super::Topology) -> D3D12_PRIMITIVE_TOPOLOGY_TYPE {
	match topology {
		super::Topology::PointList        => D3D12_PRIMITIVE_TOPOLOGY_TYPE_POINT,
		super::Topology::LineList         => D3D12_PRIMITIVE_TOPOLOGY_TYPE_LINE,
		super::Topology::LineStrip        => D3D12_PRIMITIVE_TOPOLOGY_TYPE_LINE,
		super::Topology::TriangleList     => D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
		super::Topology::TriangleStrip    => D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
	}
}

fn map_polygon_mode(fill_mode: &super::PolygonMode) -> D3D12_FILL_MODE {
	match fill_mode {
		super::PolygonMode::Line => D3D12_FILL_MODE_WIREFRAME,
		super::PolygonMode::Fill => D3D12_FILL_MODE_SOLID,
	}
}

fn map_cull_mode(cull_mode: &super::CullMode) -> D3D12_CULL_MODE {
	match cull_mode {
		super::CullMode::None  => D3D12_CULL_MODE_NONE,
		super::CullMode::Front => D3D12_CULL_MODE_FRONT,
		super::CullMode::Back  => D3D12_CULL_MODE_BACK,
	}
}

fn map_stencil_op(stencil_op: &super::StencilOp) -> D3D12_STENCIL_OP {
	match stencil_op {
		super::StencilOp::Keep           => D3D12_STENCIL_OP_KEEP,
		super::StencilOp::Zero           => D3D12_STENCIL_OP_ZERO,
		super::StencilOp::Replace        => D3D12_STENCIL_OP_REPLACE,
		super::StencilOp::Invert         => D3D12_STENCIL_OP_INVERT,
		super::StencilOp::IncrementWrap  => D3D12_STENCIL_OP_INCR,
		super::StencilOp::IncrementClamp => D3D12_STENCIL_OP_INCR_SAT,
		super::StencilOp::DecrementWrap  => D3D12_STENCIL_OP_DECR,
		super::StencilOp::DecrementClamp => D3D12_STENCIL_OP_DECR_SAT,
	}
}

fn map_color_attachments(attachments: &[super::ColorAttachment]) -> [D3D12_RENDER_TARGET_BLEND_DESC; 8] {
	let mut dx_descs: [D3D12_RENDER_TARGET_BLEND_DESC; 8] = [D3D12_RENDER_TARGET_BLEND_DESC::default(); 8];

	for (attachment, dx) in attachments.iter().zip(dx_descs.iter_mut()) {
		dx.RenderTargetWriteMask = attachment.write_mask.bits();

		if let Some(blend) = &attachment.blend {
			dx.BlendEnable = true.into();
			dx.SrcBlend = map_blend_factor(&blend.src_color);
			dx.DestBlend = map_blend_factor(&blend.dst_color);
			dx.BlendOp = map_blend_op(&blend.color_op);
			dx.SrcBlendAlpha = map_blend_factor(&blend.src_alpha);
			dx.DestBlendAlpha = map_blend_factor(&blend.dst_alpha);
			dx.BlendOpAlpha = map_blend_op(&blend.alpha_op);
		}
	}

	dx_descs
}

fn map_blend_factor(blend_factor: &super::BlendFactor) -> D3D12_BLEND {
	match blend_factor {
		super::BlendFactor::Zero             => D3D12_BLEND_ZERO,
		super::BlendFactor::One              => D3D12_BLEND_ONE,
		super::BlendFactor::SrcColor         => D3D12_BLEND_SRC_COLOR,
		super::BlendFactor::InvSrcColor      => D3D12_BLEND_INV_SRC_COLOR,
		super::BlendFactor::SrcAlpha         => D3D12_BLEND_SRC_ALPHA,
		super::BlendFactor::InvSrcAlpha      => D3D12_BLEND_INV_SRC_ALPHA,
		super::BlendFactor::DstColor         => D3D12_BLEND_DEST_COLOR,
		super::BlendFactor::InvDstColor      => D3D12_BLEND_INV_DEST_COLOR,
		super::BlendFactor::DstAlpha         => D3D12_BLEND_DEST_ALPHA,
		super::BlendFactor::InvDstAlpha      => D3D12_BLEND_INV_DEST_ALPHA,
		super::BlendFactor::Src1Color        => D3D12_BLEND_SRC1_COLOR,
		super::BlendFactor::InvSrc1Color     => D3D12_BLEND_INV_SRC1_COLOR,
		super::BlendFactor::Src1Alpha        => D3D12_BLEND_SRC1_ALPHA,
		super::BlendFactor::InvSrc1Alpha     => D3D12_BLEND_INV_SRC1_ALPHA,
		super::BlendFactor::SrcAlphaSat      => D3D12_BLEND_SRC_ALPHA_SAT,
		super::BlendFactor::ConstantColor    => D3D12_BLEND_BLEND_FACTOR,
		super::BlendFactor::InvConstantColor => D3D12_BLEND_INV_BLEND_FACTOR,
	}
}

fn map_blend_op(blend_op: &super::BlendOp) -> D3D12_BLEND_OP {
	match blend_op {
		super::BlendOp::Add         => D3D12_BLEND_OP_ADD,
		super::BlendOp::Subtract    => D3D12_BLEND_OP_SUBTRACT,
		super::BlendOp::RevSubtract => D3D12_BLEND_OP_REV_SUBTRACT,
		super::BlendOp::Min         => D3D12_BLEND_OP_MIN,
		super::BlendOp::Max         => D3D12_BLEND_OP_MAX,
	}
}

fn map_acceleration_structure_build_flags(flags: super::AccelerationStructureBuildFlags) -> D3D12_RAYTRACING_ACCELERATION_STRUCTURE_BUILD_FLAGS {
	let mut dx_flags = D3D12_RAYTRACING_ACCELERATION_STRUCTURE_BUILD_FLAG_NONE;
	
	if flags.contains(super::AccelerationStructureBuildFlags::ALLOW_UPDATE)      { dx_flags |= D3D12_RAYTRACING_ACCELERATION_STRUCTURE_BUILD_FLAG_ALLOW_UPDATE; }
	if flags.contains(super::AccelerationStructureBuildFlags::ALLOW_COMPACTION)  { dx_flags |= D3D12_RAYTRACING_ACCELERATION_STRUCTURE_BUILD_FLAG_ALLOW_COMPACTION; }
	if flags.contains(super::AccelerationStructureBuildFlags::PREFER_FAST_TRACE) { dx_flags |= D3D12_RAYTRACING_ACCELERATION_STRUCTURE_BUILD_FLAG_PREFER_FAST_TRACE; }
	if flags.contains(super::AccelerationStructureBuildFlags::PREFER_FAST_BUILD) { dx_flags |= D3D12_RAYTRACING_ACCELERATION_STRUCTURE_BUILD_FLAG_PREFER_FAST_BUILD; }
	if flags.contains(super::AccelerationStructureBuildFlags::MINIMIZE_MEMORY)   { dx_flags |= D3D12_RAYTRACING_ACCELERATION_STRUCTURE_BUILD_FLAG_MINIMIZE_MEMORY; }

	dx_flags
}

fn map_acceleration_structure_bottom_level_flags(flags: super::AccelerationStructureBottomLevelFlags) -> D3D12_RAYTRACING_GEOMETRY_FLAGS {
	let mut dx_flags = D3D12_RAYTRACING_GEOMETRY_FLAG_NONE;

	if flags.contains(super::AccelerationStructureBottomLevelFlags::OPAQUE)                          { dx_flags |= D3D12_RAYTRACING_GEOMETRY_FLAG_OPAQUE; }
	if flags.contains(super::AccelerationStructureBottomLevelFlags::NO_DUPLICATE_ANY_HIT_INVOCATION) { dx_flags |= D3D12_RAYTRACING_GEOMETRY_FLAG_NO_DUPLICATE_ANYHIT_INVOCATION; }

	dx_flags
}

fn map_acceleration_structure_instance_flags(flags: super::AccelerationStructureInstanceFlags) -> D3D12_RAYTRACING_INSTANCE_FLAGS {
	let mut dx_flags = D3D12_RAYTRACING_INSTANCE_FLAG_NONE;

	if flags.contains(super::AccelerationStructureInstanceFlags::TRIANGLE_CULL_DISABLE) { dx_flags |= D3D12_RAYTRACING_INSTANCE_FLAG_TRIANGLE_CULL_DISABLE; }
	if flags.contains(super::AccelerationStructureInstanceFlags::TRIANGLE_FRONT_CCW)    { dx_flags |= D3D12_RAYTRACING_INSTANCE_FLAG_TRIANGLE_FRONT_COUNTERCLOCKWISE; }
	if flags.contains(super::AccelerationStructureInstanceFlags::FORCE_OPAQUE)          { dx_flags |= D3D12_RAYTRACING_INSTANCE_FLAG_FORCE_OPAQUE; }
	if flags.contains(super::AccelerationStructureInstanceFlags::FORCE_NON_OPAQUE)      { dx_flags |= D3D12_RAYTRACING_INSTANCE_FLAG_FORCE_NON_OPAQUE; }

	dx_flags
}

fn map_load_op<T: Default>(load_op: super::LoadOp<T>) -> (D3D12_RENDER_PASS_BEGINNING_ACCESS_TYPE, T) {
	match load_op {
		super::LoadOp::Load         => (D3D12_RENDER_PASS_BEGINNING_ACCESS_TYPE_PRESERVE, Default::default()),
		super::LoadOp::Clear(value) => (D3D12_RENDER_PASS_BEGINNING_ACCESS_TYPE_CLEAR, value),
		super::LoadOp::Discard      => (D3D12_RENDER_PASS_BEGINNING_ACCESS_TYPE_DISCARD, Default::default()),
	}
}

fn buffer_barrier(cmd: &ID3D12GraphicsCommandList7, buffer: &ID3D12Resource) {
	let dx_barrier = D3D12_BUFFER_BARRIER {
		SyncBefore: D3D12_BARRIER_SYNC_ALL,
		SyncAfter: D3D12_BARRIER_SYNC_ALL,
		AccessBefore: D3D12_BARRIER_ACCESS_COMMON,
		AccessAfter: D3D12_BARRIER_ACCESS_COMMON,
		pResource: unsafe { std::mem::transmute_copy(buffer) },
		Offset: 0,
		Size: u64::MAX,
	};

	let dx_barrier_group = D3D12_BARRIER_GROUP {
		Type: D3D12_BARRIER_TYPE_BUFFER,
		NumBarriers: 1,
		Anonymous: D3D12_BARRIER_GROUP_0 {
			pBufferBarriers: &dx_barrier,
		},
	};

	unsafe {
		cmd.Barrier(&[dx_barrier_group]);
	}
}

fn texture_barrier(cmd: &ID3D12GraphicsCommandList7, texture: &ID3D12Resource, old_layout: D3D12_BARRIER_LAYOUT, new_layout: D3D12_BARRIER_LAYOUT) {
	let dx_barrier = D3D12_TEXTURE_BARRIER {
		SyncBefore: D3D12_BARRIER_SYNC_ALL,
		SyncAfter: D3D12_BARRIER_SYNC_ALL,
		AccessBefore: D3D12_BARRIER_ACCESS_COMMON,
		AccessAfter: D3D12_BARRIER_ACCESS_COMMON,
		LayoutBefore: old_layout,
		LayoutAfter: new_layout,
		pResource: unsafe { std::mem::transmute_copy(texture) },
		Subresources: D3D12_BARRIER_SUBRESOURCE_RANGE {
			IndexOrFirstMipLevel: 0xffffffff, // All subresources
			NumMipLevels: 0,
			FirstArraySlice: 0,
			NumArraySlices: 0,
			FirstPlane: 0,
			NumPlanes: 0,
		},
		Flags: D3D12_TEXTURE_BARRIER_FLAG_NONE,
	};

	let dx_barrier_group = D3D12_BARRIER_GROUP {
		Type: D3D12_BARRIER_TYPE_TEXTURE,
		NumBarriers: 1,
		Anonymous: D3D12_BARRIER_GROUP_0 {
			pTextureBarriers: &dx_barrier,
		},
	};

	unsafe {
		cmd.Barrier(&[dx_barrier_group]);
	}
}

fn get_adapter(factory: &IDXGIFactory6, gpu_preference: DXGI_GPU_PREFERENCE) -> Result<(IDXGIAdapter1, super::AdapterInfo)> {
	unsafe {
		let mut adapter_index = None;
		let mut adapter_names = Vec::<String>::new();

		for i in 0.. {
			let adapter = factory.EnumAdapterByGpuPreference::<IDXGIAdapter1>(i, gpu_preference);
			if adapter.is_err() {
				break;
			}

			let mut desc = std::mem::zeroed();
			adapter.unwrap().GetDesc1(&mut desc)?;

			let adapter_name_len = desc.Description.iter().take_while(|&&c| c != 0).count();
			let adapter_name = String::from_utf16_lossy(&desc.Description[..adapter_name_len]);

			adapter_names.push(adapter_name);

			let adapter_flag = DXGI_ADAPTER_FLAG(desc.Flags as i32);
			if (adapter_flag & DXGI_ADAPTER_FLAG_SOFTWARE) == DXGI_ADAPTER_FLAG_NONE && adapter_index == None {
				adapter_index = Some(i as i32);
			}
		}

		let adapter_index = adapter_index.unwrap_or(0);

		let adapter = factory.EnumAdapterByGpuPreference::<IDXGIAdapter1>(adapter_index as u32, gpu_preference)?;
		let mut desc = std::mem::zeroed();
		adapter.GetDesc1(&mut desc)?;

		let adapter_info = super::AdapterInfo {
			name: adapter_names[adapter_index as usize].to_string(),
			dedicated_video_memory: desc.DedicatedVideoMemory,
			dedicated_system_memory: desc.DedicatedSystemMemory,
			shared_system_memory: desc.SharedSystemMemory,
			available: adapter_names,
		};

		return Ok((adapter, adapter_info));
	}
}

fn create_swap_chain_rtv(
	swap_chain: &IDXGISwapChain3,
	device: &mut Device,
	num_bb: u32,
) -> Vec<Texture> {
	unsafe {
		let mut textures: Vec<Texture> = Vec::new();
		for i in 0..num_bb {
			let render_target: ID3D12Resource = swap_chain.GetBuffer(i).unwrap();
			let h = device.rtv_heap.allocate();
			device.device.CreateRenderTargetView(&render_target, None, h);
			textures.push(Texture {
				desc: std::mem::zeroed(), // TODO: Fill this in.
				resource: render_target,
				rtv: Some(h),
				dsv: None,
				srv_index: None,
				uav_index: None,
			});
		}
		textures
	}
}

struct Heap {
	heap: ID3D12DescriptorHeap,
	base_address: usize,
	increment_size: usize,
	capacity: usize,
	offset: usize,
	free_list: Vec<usize>,
}

impl Heap {
	fn new(device: &ID3D12Device10, ty: D3D12_DESCRIPTOR_HEAP_TYPE, num_descriptors: usize) -> Heap {
		let heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
			Type: ty,
			NumDescriptors: std::cmp::max(num_descriptors, 1) as u32,
			Flags: match ty {
				D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV => D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
				D3D12_DESCRIPTOR_HEAP_TYPE_SAMPLER => D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
				D3D12_DESCRIPTOR_HEAP_TYPE_RTV => D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
				D3D12_DESCRIPTOR_HEAP_TYPE_DSV => D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
				_ => panic!(),
			},
			..Default::default()
		};

		let heap: ID3D12DescriptorHeap = unsafe { device.CreateDescriptorHeap(&heap_desc).unwrap() };
		let base_address = unsafe { heap.GetCPUDescriptorHandleForHeapStart().ptr } as usize;
		let increment_size = unsafe { device.GetDescriptorHandleIncrementSize(ty) } as usize;

		Heap {
			heap,
			base_address,
			increment_size,
			capacity: num_descriptors * increment_size,
			offset: 0,
			free_list: Vec::new(),
		}
	}

	fn allocate(&mut self) -> D3D12_CPU_DESCRIPTOR_HANDLE {
		if self.free_list.is_empty() {
			if self.offset >= self.capacity {
				panic!();
			}
			let ptr = self.base_address + self.offset;
			self.offset += self.increment_size;
			return D3D12_CPU_DESCRIPTOR_HANDLE { ptr };
		}

		D3D12_CPU_DESCRIPTOR_HANDLE {
			ptr: self.free_list.pop().unwrap(),
		}
	}

	fn deallocate(&mut self, handle: &D3D12_CPU_DESCRIPTOR_HANDLE) {
		self.free_list.push(handle.ptr);
	}

	fn index_to_handle(&self, index: usize) -> D3D12_CPU_DESCRIPTOR_HANDLE {
		let ptr = self.base_address + self.increment_size * index;
		D3D12_CPU_DESCRIPTOR_HANDLE { ptr }
	}

	fn handle_to_index(&self, handle: &D3D12_CPU_DESCRIPTOR_HANDLE) -> usize {
		let ptr = handle.ptr - self.base_address;
		ptr / self.increment_size
	}
}

unsafe extern "system" fn debug_callback(
	_category: D3D12_MESSAGE_CATEGORY,
	severity: D3D12_MESSAGE_SEVERITY,
	_id: D3D12_MESSAGE_ID,
	description: PCSTR,
	_context: *mut std::ffi::c_void,
) {
	let log_severity = match severity {
		D3D12_MESSAGE_SEVERITY_CORRUPTION => log::Level::Error,
		D3D12_MESSAGE_SEVERITY_ERROR => log::Level::Error,
		D3D12_MESSAGE_SEVERITY_WARNING => log::Level::Warn,
		D3D12_MESSAGE_SEVERITY_INFO => log::Level::Info,
		D3D12_MESSAGE_SEVERITY_MESSAGE => log::Level::Debug,
		_ => unreachable!(),
	};

	log::log!(target: "gpu::d3d12", log_severity, "{}", description.display());
}

impl Device {
	fn create_root_signature(&self, layout: &super::DescriptorLayout) -> result::Result<ID3D12RootSignature, super::Error> {
		let mut root_params: Vec<D3D12_ROOT_PARAMETER1> = Vec::new();

		if let Some(push_constants) = &layout.push_constants {
			assert_eq!(push_constants.size % 4, 0);
			root_params.push(D3D12_ROOT_PARAMETER1 {
				ParameterType: D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS,
				Anonymous: D3D12_ROOT_PARAMETER1_0 {
					Constants: D3D12_ROOT_CONSTANTS {
						ShaderRegister: 0, // TODO: Hardcoded because Vulkan can't configure this for push constants.
						RegisterSpace: 0,  // Maybe use different registers though, decide when implementing Vulkan.
						Num32BitValues: push_constants.size / 4,
					},
				},
				ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
			});
		}

		if let Some(bindings) = &layout.bindings {
			let mut ranges: Vec<D3D12_DESCRIPTOR_RANGE1> = Vec::new();

			for binding in bindings {
				let range = D3D12_DESCRIPTOR_RANGE1 {
					RangeType: match binding.binding_type {
						super::DescriptorType::ShaderResource => D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
						super::DescriptorType::UnorderedAccess => D3D12_DESCRIPTOR_RANGE_TYPE_UAV,
						super::DescriptorType::ConstantBuffer => D3D12_DESCRIPTOR_RANGE_TYPE_CBV,
						super::DescriptorType::Sampler => D3D12_DESCRIPTOR_RANGE_TYPE_SAMPLER,
					},
					NumDescriptors: binding.num_descriptors.unwrap_or(u32::MAX),
					BaseShaderRegister: binding.shader_register,
					RegisterSpace: binding.register_space,
					Flags: D3D12_DESCRIPTOR_RANGE_FLAG_DESCRIPTORS_VOLATILE | D3D12_DESCRIPTOR_RANGE_FLAG_DATA_VOLATILE, // TODO: Optimization potential?
					OffsetInDescriptorsFromTableStart: binding.offset.unwrap_or(D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND),
				};

				ranges.push(range);
			}

			root_params.push(D3D12_ROOT_PARAMETER1 {
				ParameterType: D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
				Anonymous: D3D12_ROOT_PARAMETER1_0 {
					DescriptorTable: D3D12_ROOT_DESCRIPTOR_TABLE1 {
						NumDescriptorRanges: ranges.len() as u32,
						pDescriptorRanges: ranges.as_ptr() as *mut _,
					},
				},
				ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
			});
		}

		let mut static_samplers: Vec<D3D12_STATIC_SAMPLER_DESC> = Vec::new();
		if let Some(samplers) = &layout.static_samplers {
			for sampler in samplers {
				static_samplers.push(D3D12_STATIC_SAMPLER_DESC {
					Filter: map_sampler_filter(&sampler.sampler_desc),
					AddressU: map_address_mode(sampler.sampler_desc.address_u),
					AddressV: map_address_mode(sampler.sampler_desc.address_v),
					AddressW: map_address_mode(sampler.sampler_desc.address_w),
					MipLODBias: sampler.sampler_desc.lod_bias,
					MaxAnisotropy: sampler.sampler_desc.max_anisotropy,
					ComparisonFunc: map_address_compare_op(sampler.sampler_desc.compare),
					BorderColor: sampler.sampler_desc.border_color.map_or(D3D12_STATIC_BORDER_COLOR_TRANSPARENT_BLACK, map_static_border_color),
					MinLOD: sampler.sampler_desc.min_lod,
					MaxLOD: sampler.sampler_desc.max_lod,
					ShaderRegister: sampler.shader_register,
					RegisterSpace: sampler.register_space,
					ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
				})
			}
		}

		let desc = D3D12_VERSIONED_ROOT_SIGNATURE_DESC {
			Version: D3D_ROOT_SIGNATURE_VERSION_1_1,
			Anonymous: D3D12_VERSIONED_ROOT_SIGNATURE_DESC_0 {
				Desc_1_1: D3D12_ROOT_SIGNATURE_DESC1 {
					NumParameters: root_params.len() as u32,
					pParameters: root_params.as_mut_ptr(),
					NumStaticSamplers: static_samplers.len() as u32,
					pStaticSamplers: static_samplers.as_mut_ptr(),
					Flags: D3D12_ROOT_SIGNATURE_FLAG_NONE,
				},
			},
		};

		unsafe {
			let mut signature = None;
			let mut error = None;

			D3D12SerializeVersionedRootSignature(&desc, &mut signature, Some(&mut error)).unwrap();

			if let Some(blob) = error {
				let error = String::from_raw_parts(blob.GetBufferPointer() as *mut _, blob.GetBufferSize(), blob.GetBufferSize());
				return Err(super::Error { error });
			}

			let sig = signature.unwrap();
			let slice = std::slice::from_raw_parts(sig.GetBufferPointer() as *mut u8, sig.GetBufferSize());
			let sig = self.device.CreateRootSignature(0, slice)?;
			Ok(sig)
		}
	}
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

	fn new(desc: &super::DeviceDesc) -> Device {
		unsafe {
			if !desc.validation.is_empty() {
				let mut debug: Option<ID3D12Debug1> = None;
				if let Some(debug) = D3D12GetDebugInterface(&mut debug).ok().and(debug) {
					debug.EnableDebugLayer();

					if desc.validation.contains(super::Validation::GPU) {
						debug.SetEnableGPUBasedValidation(true);
					}
				}
			}

			let dxgi_factory_flags = if !desc.validation.is_empty() {
				0
			} else {
				DXGI_CREATE_FACTORY_DEBUG
			};
			let dxgi_factory = CreateDXGIFactory2(dxgi_factory_flags).unwrap();

			let (adapter, adapter_info) = get_adapter(&dxgi_factory, map_power_preference(desc.power_preference)).unwrap();

			let feature_levels = [
				D3D_FEATURE_LEVEL_12_2,
				D3D_FEATURE_LEVEL_12_1,
				D3D_FEATURE_LEVEL_12_0,
				D3D_FEATURE_LEVEL_11_1,
				D3D_FEATURE_LEVEL_11_0,
			];

			let mut device: Option<ID3D12Device10> = None;
			for feature_level in feature_levels {
				if D3D12CreateDevice(&adapter, feature_level, &mut device).is_ok() {
					break;
				}
			}
			let device = device.unwrap();

			if let Ok(info_queue) = device.cast::<ID3D12InfoQueue1>() {
				let mut cookie = 0;
				info_queue.SetMuteDebugOutput(true);
				info_queue.RegisterMessageCallback(Some(debug_callback), D3D12_MESSAGE_CALLBACK_IGNORE_FILTERS, std::ptr::null_mut(), &mut cookie).unwrap();
			}

			let mut feature_options5 = D3D12_FEATURE_DATA_D3D12_OPTIONS5::default();
			let res = device.CheckFeatureSupport(
				D3D12_FEATURE_D3D12_OPTIONS5,
				&mut feature_options5 as *mut _ as *mut _,
				std::mem::size_of::<D3D12_FEATURE_DATA_D3D12_OPTIONS5>() as _,
			);

			let capabilities = super::Capabilities {
				raytracing: res.is_ok() && feature_options5.RaytracingTier == D3D12_RAYTRACING_TIER_1_1,
			};

			let command_allocator = device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT).unwrap();
			let command_list = device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &command_allocator, None).unwrap();

			let command_queue_desc = D3D12_COMMAND_QUEUE_DESC {
				Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
				NodeMask: 1,
				..Default::default()
			};
			let command_queue = device.CreateCommandQueue(&command_queue_desc).unwrap();

			let resource_heap = Heap::new(&device, D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV, 1_000_000); // Tier 1 limit
			let sampler_heap = Heap::new(&device, D3D12_DESCRIPTOR_HEAP_TYPE_SAMPLER, 2048); // Tier 1 limit
			let rtv_heap = Heap::new(&device, D3D12_DESCRIPTOR_HEAP_TYPE_RTV, 100); // TODO: Hardcoded
			let dsv_heap = Heap::new(&device, D3D12_DESCRIPTOR_HEAP_TYPE_DSV, 100); // TODO: Hardcoded

			let pix = desc.validation.contains(super::Validation::DEBUGGER)
				.then(|| WinPixEventRuntime::load()).flatten();

			Device {
				adapter_info,
				capabilities,
				dxgi_factory,
				device,
				command_allocator,
				command_list,
				command_queue,
				pix,
				resource_heap,
				sampler_heap,
				rtv_heap,
				dsv_heap,
			}
		}
	}

	fn create_swap_chain(&mut self, desc: &super::SwapChainDesc, window_handle: &NativeHandle) -> result::Result<SwapChain, super::Error> {
		unsafe {
			let flags = DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT.0 | DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING.0;
			let format = desc.format;
			let dxgi_format = map_format(format);

			let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
				BufferCount: desc.num_buffers,
				Width: desc.size[0],
				Height: desc.size[1],
				Format: dxgi_format,
				BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
				SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
				Flags: flags as u32,
				SampleDesc: DXGI_SAMPLE_DESC {
					Count: 1,
					Quality: 0,
				},
				..Default::default()
			};

			let swap_chain1 = self
				.dxgi_factory
				.CreateSwapChainForHwnd(
					&self.command_queue,
					HWND(window_handle.0),
					&swap_chain_desc,
					None,
					None,
				)?;
			let swap_chain: IDXGISwapChain3 = swap_chain1.cast()?;
			
			let textures = create_swap_chain_rtv(&swap_chain, self, desc.num_buffers);

			Ok(SwapChain {
				size: desc.size,
				num_buffers: desc.num_buffers,
				flags: flags as u32,
				bb_index: 0,
				fence: self.device.CreateFence(0, D3D12_FENCE_FLAG_NONE)?,
				fence_last_signalled_value: 0,
				fence_event: CreateEventA(None, false, false, None)?,
				swap_chain,
				backbuffer_textures: textures,
				frame_fence_value: vec![0; desc.num_buffers as usize],
				require_wait: vec![false; desc.num_buffers as usize],
			})
		}
	}

	fn create_cmd_list(&self, num_buffers: u32) -> CmdList {
		unsafe {
			let mut command_allocators: Vec<ID3D12CommandAllocator> = Vec::new();
			let mut command_lists: Vec<ID3D12GraphicsCommandList7> = Vec::new();

			for _ in 0..num_buffers as usize {
				let command_allocator = self.device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT).unwrap();
				let command_list = self.device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &command_allocator, None).unwrap();

				command_allocators.push(command_allocator);
				command_lists.push(command_list);
			}

			CmdList {
				bb_index: 0,
				command_allocator: command_allocators,
				command_list: command_lists,
				pix: self.pix,
				resource_heap_base: self.resource_heap.heap.GetGPUDescriptorHandleForHeapStart(),
			}
		}
	}

	fn create_buffer(&mut self, desc: &super::BufferDesc) -> result::Result<Buffer, super::Error> {
		let mut resource: Option<ID3D12Resource> = None;
		unsafe {
			self.device.CreateCommittedResource3(
				&D3D12_HEAP_PROPERTIES {
					Type: if matches!(desc.memory, super::Memory::CpuToGpu) {
						D3D12_HEAP_TYPE_UPLOAD
					} else {
						D3D12_HEAP_TYPE_DEFAULT
					},
					..Default::default()
				},
				D3D12_HEAP_FLAG_NONE,
				&D3D12_RESOURCE_DESC1 {
					Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
					Alignment: 0,
					Width: desc.size as u64,
					Height: 1,
					DepthOrArraySize: 1,
					MipLevels: 1,
					Format: DXGI_FORMAT_UNKNOWN,
					SampleDesc: DXGI_SAMPLE_DESC {
						Count: 1,
						Quality: 0,
					},
					Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
					Flags: map_buffer_usage_flags(desc.usage),
					SamplerFeedbackMipRegion: D3D12_MIP_REGION::default(),
				},
				D3D12_BARRIER_LAYOUT_UNDEFINED,
				None,
				None,
				None,
				&mut resource,
			)?;

			// TODO: We assume the srv is for a raw buffer (ByteAddressBuffer)
			// This is actually a 'Word' Buffer, since it must contain a multiple of 4 bytes, assert this!
			let srv_index = desc.usage.contains(super::BufferUsage::SHADER_RESOURCE).then(|| {
				let h = self.resource_heap.allocate();
				self.device.CreateShaderResourceView(
					&resource.clone().unwrap(),
					Some(&D3D12_SHADER_RESOURCE_VIEW_DESC {
						Format: DXGI_FORMAT_R32_TYPELESS,
						ViewDimension: D3D12_SRV_DIMENSION_BUFFER,
						Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
						Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
							Buffer: D3D12_BUFFER_SRV {
								FirstElement: 0, // offset / sizeof(u32)
								NumElements: (desc.size / std::mem::size_of::<u32>()) as u32, // (desc.size - offset) / sizeof(u32)
								StructureByteStride: 0,
								Flags: D3D12_BUFFER_SRV_FLAG_RAW,
							}
						},
					}),
					h,
				);
				self.resource_heap.handle_to_index(&h)
			});

			let uav_index = desc.usage.contains(super::BufferUsage::UNORDERED_ACCESS).then(|| {
				let h = self.resource_heap.allocate();
				self.device.CreateUnorderedAccessView(
					&resource.clone().unwrap(),
					None,
					Some(&D3D12_UNORDERED_ACCESS_VIEW_DESC {
						Format: DXGI_FORMAT_R32_TYPELESS,
						ViewDimension: D3D12_UAV_DIMENSION_BUFFER,
						Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
							Buffer: D3D12_BUFFER_UAV {
								FirstElement: 0,
								NumElements: (desc.size / std::mem::size_of::<u32>()) as u32,
								StructureByteStride: 0,
								CounterOffsetInBytes: 0,
								Flags: D3D12_BUFFER_UAV_FLAG_RAW,
							}
						},
					}),
					h,
				);
				self.resource_heap.handle_to_index(&h)
			});

			let mapped_ptr = {
				let mut ptr = std::ptr::null_mut();
				if !matches!(desc.memory, super::Memory::GpuOnly) {
					let read_range = matches!(desc.memory, super::Memory::CpuToGpu).then_some(&D3D12_RANGE::default() as *const _);
					resource.clone().unwrap().Map(0, read_range, Some(&mut ptr))?;
				}
				ptr as *mut u8
			};

			Ok(Buffer {
				resource: resource.unwrap(),
				size: desc.size as u64,
				srv_index,
				uav_index,
				mapped_ptr,
			})
		}
	}

	fn create_texture(&mut self, desc: &super::TextureDesc) -> result::Result<Texture, super::Error> {
		let mut resource: Option<ID3D12Resource> = None;
		let dxgi_format = map_format(desc.format);
		let initial_layout = map_texture_layout(desc.layout);
		unsafe {
			self.device.CreateCommittedResource3(
				&D3D12_HEAP_PROPERTIES {
					Type: D3D12_HEAP_TYPE_DEFAULT,
					..Default::default()
				},
				D3D12_HEAP_FLAG_NONE,
				&D3D12_RESOURCE_DESC1 {
					Dimension: map_resource_dimension(desc),
					Alignment: 0,
					Width: desc.width,
					Height: desc.height as u32,
					DepthOrArraySize: if desc.depth > 1 { desc.depth as u16 } else { desc.array_size as u16 },
					MipLevels: desc.mip_levels as u16,
					Format: dxgi_format,
					SampleDesc: DXGI_SAMPLE_DESC {
						Count: desc.samples,
						Quality: 0,
					},
					Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
					Flags: map_texture_usage_flags(desc.usage),
					SamplerFeedbackMipRegion: D3D12_MIP_REGION::default(),
				},
				initial_layout,
				None,
				None,
				None,
				&mut resource,
			)?;

			let srv_index = desc.usage.contains(super::TextureUsage::SHADER_RESOURCE).then(|| {
				let h = self.resource_heap.allocate();
				self.device.CreateShaderResourceView(
					&resource.clone().unwrap(),
					Some(&D3D12_SHADER_RESOURCE_VIEW_DESC {
						Format: dxgi_format,
						ViewDimension: map_srv_dimension(desc),
						Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
							Texture2D: D3D12_TEX2D_SRV {
								MipLevels: desc.mip_levels,
								MostDetailedMip: 0,
								..Default::default()
							},
						},
						Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
					}),
					h,
				);
				self.resource_heap.handle_to_index(&h)
			});

			let uav_index = desc.usage.contains(super::TextureUsage::UNORDERED_ACCESS).then(|| {
				let h = self.resource_heap.allocate();
				self.device.CreateUnorderedAccessView(&resource.clone().unwrap(), None, None, h);
				self.resource_heap.handle_to_index(&h)
			});

			let rtv = desc.usage.contains(super::TextureUsage::RENDER_TARGET).then(|| {
				let h = self.rtv_heap.allocate();
				self.device.CreateRenderTargetView(&resource.clone().unwrap(), None, h);
				h
			});

			let dsv = desc.usage.contains(super::TextureUsage::DEPTH_STENCIL).then(|| {
				let h = self.dsv_heap.allocate();
				self.device.CreateDepthStencilView(&resource.clone().unwrap(), None, h);
				h
			});

			Ok(Texture {
				desc: *desc,
				resource: resource.unwrap(),
				srv_index,
				uav_index,
				rtv,
				dsv,
			})
		}
	}

	fn create_sampler(&mut self, desc: &super::SamplerDesc) -> result::Result<Self::Sampler, super::Error> {
		let dx_desc = D3D12_SAMPLER_DESC {
			Filter: map_sampler_filter(desc),
			AddressU: map_address_mode(desc.address_u),
			AddressV: map_address_mode(desc.address_v),
			AddressW: map_address_mode(desc.address_w),
			MipLODBias: desc.lod_bias,
			MaxAnisotropy: desc.max_anisotropy,
			ComparisonFunc: map_address_compare_op(desc.compare),
			BorderColor: desc.border_color.map_or([0.0; 4], map_border_color),
			MinLOD: desc.min_lod,
			MaxLOD: desc.max_lod,
		};

		let h = self.sampler_heap.allocate();
		unsafe { self.device.CreateSampler(&dx_desc, h) };

		Ok(Sampler { _sampler: h })
	}

	fn create_acceleration_structure(&mut self, desc: &super::AccelerationStructureDesc<Self>) -> result::Result<AccelerationStructure, super::Error> {
		let srv_index = matches!(desc.kind, super::AccelerationStructureKind::TopLevel).then(|| {
			let h = self.resource_heap.allocate();

			unsafe {
				self.device.CreateShaderResourceView(
					None,
					Some(&D3D12_SHADER_RESOURCE_VIEW_DESC {
						Format: DXGI_FORMAT_UNKNOWN,
						ViewDimension: D3D12_SRV_DIMENSION_RAYTRACING_ACCELERATION_STRUCTURE,
						Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
							RaytracingAccelerationStructure: D3D12_RAYTRACING_ACCELERATION_STRUCTURE_SRV {
								Location: desc.buffer.resource.GetGPUVirtualAddress() + (desc.offset as u64),
							},
						},
						Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
					}),
					h,
				);
			}
		
			self.resource_heap.handle_to_index(&h)
		});

		Ok(AccelerationStructure {
			resource: desc.buffer.resource.clone(),
			srv_index,
		})
	}

	fn create_graphics_pipeline(&self, desc: &super::GraphicsPipelineDesc) -> result::Result<GraphicsPipeline, super::Error> {
		let root_signature = self.create_root_signature(&desc.descriptor_layout)?;

		let raster = &desc.rasterizer;
		let depth_stencil = &desc.depth_stencil;

		let mut dx_desc = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
			InputLayout: D3D12_INPUT_LAYOUT_DESC {
				pInputElementDescs: std::ptr::null(),
				NumElements: 0,
			},
			pRootSignature: unsafe { std::mem::transmute_copy(&root_signature) },
			VS: desc.vs.map_or(D3D12_SHADER_BYTECODE::default(), |vs| D3D12_SHADER_BYTECODE {
				pShaderBytecode: vs.as_ptr() as _,
				BytecodeLength: vs.len(),
			}),
			PS: desc.ps.map_or(D3D12_SHADER_BYTECODE::default(), |ps| D3D12_SHADER_BYTECODE {
				pShaderBytecode: ps.as_ptr() as _,
				BytecodeLength: ps.len(),
			}),
			RasterizerState: D3D12_RASTERIZER_DESC {
				FillMode: map_polygon_mode(&raster.polygon_mode),
				CullMode: map_cull_mode(&raster.cull_mode),
				FrontCounterClockwise: raster.front_ccw.into(),
				DepthBias: raster.depth_bias.constant as _,
				DepthBiasClamp: raster.depth_bias.clamp,
				SlopeScaledDepthBias: raster.depth_bias.slope,
				DepthClipEnable: raster.depth_clip_enable.into(),
				MultisampleEnable: raster.front_ccw.into(),
				AntialiasedLineEnable: raster.front_ccw.into(),
				ForcedSampleCount: raster.forced_sample_count,
				ConservativeRaster: if raster.conservative_rasterization_enable {
					D3D12_CONSERVATIVE_RASTERIZATION_MODE_ON
				} else {
					D3D12_CONSERVATIVE_RASTERIZATION_MODE_OFF
				},
			},
			BlendState: D3D12_BLEND_DESC {
				AlphaToCoverageEnable: false.into(),
				IndependentBlendEnable: true.into(),
				RenderTarget: map_color_attachments(&desc.color_attachments),
			},
			DepthStencilState: D3D12_DEPTH_STENCIL_DESC {
				DepthEnable: depth_stencil.depth_test_enable.into(),
				DepthWriteMask: if depth_stencil.depth_write_enable { D3D12_DEPTH_WRITE_MASK_ALL } else { D3D12_DEPTH_WRITE_MASK_ZERO },
				DepthFunc: map_compare_op(depth_stencil.depth_op),
				StencilEnable: depth_stencil.stencil_enable.into(),
				StencilReadMask: depth_stencil.stencil_read_mask,
				StencilWriteMask: depth_stencil.stencil_write_mask,
				FrontFace: D3D12_DEPTH_STENCILOP_DESC {
					StencilFailOp: map_stencil_op(&depth_stencil.front_face.fail),
					StencilDepthFailOp: map_stencil_op(&depth_stencil.front_face.depth_fail),
					StencilPassOp: map_stencil_op(&depth_stencil.front_face.pass),
					StencilFunc: map_compare_op(depth_stencil.front_face.func),
				},
				BackFace: D3D12_DEPTH_STENCILOP_DESC {
					StencilFailOp: map_stencil_op(&depth_stencil.back_face.fail),
					StencilDepthFailOp: map_stencil_op(&depth_stencil.back_face.depth_fail),
					StencilPassOp: map_stencil_op(&depth_stencil.back_face.pass),
					StencilFunc: map_compare_op(depth_stencil.back_face.func),
				},
			},
			SampleMask: 0xffffffff,
			PrimitiveTopologyType: map_topology_type(desc.topology),
			NumRenderTargets: desc.color_attachments.len() as u32,
			SampleDesc: DXGI_SAMPLE_DESC {
				Count: 1,
				Quality: 0,
			},
			..Default::default()
		};

		for i in 0..desc.color_attachments.len() as usize {
			dx_desc.RTVFormats[i] = map_format(desc.color_attachments[i].format);
		}
		
		dx_desc.DSVFormat = map_format(desc.depth_stencil.format);

		Ok(GraphicsPipeline {
			pipeline_state: unsafe { self.device.CreateGraphicsPipelineState(&dx_desc)? },
			root_signature,
			topology: map_topology(desc.topology),
		})
	}

	fn create_compute_pipeline(&self, desc: &super::ComputePipelineDesc) -> result::Result<ComputePipeline, super::Error> {
		let cs = &desc.cs;
		let root_signature = self.create_root_signature(&desc.descriptor_layout)?;

		let dx_desc = D3D12_COMPUTE_PIPELINE_STATE_DESC {
			CS: D3D12_SHADER_BYTECODE {
				pShaderBytecode: cs.as_ptr() as _,
				BytecodeLength: cs.len(),
			},
			pRootSignature: unsafe { std::mem::transmute_copy(&root_signature) },
			..Default::default()
		};

		unsafe {
			Ok(ComputePipeline {
				pipeline_state: self.device.CreateComputePipelineState(&dx_desc)?,
				root_signature,
			})
		}
	}

	fn create_raytracing_pipeline(&self, desc: &super::RaytracingPipelineDesc) -> result::Result<Self::RaytracingPipeline, super::Error> {
		let subobject_count = desc.libraries.len() + desc.groups.len() + 3; // TODO: Only count groups that are not general
		let mut subobjects: Vec<D3D12_STATE_SUBOBJECT> = Vec::with_capacity(subobject_count);

		let root_signature = self.create_root_signature(&desc.descriptor_layout)?;

		let pipeline_config = D3D12_RAYTRACING_PIPELINE_CONFIG {
			MaxTraceRecursionDepth: desc.max_trace_recursion_depth,
		};

		subobjects.push(D3D12_STATE_SUBOBJECT {
			Type: D3D12_STATE_SUBOBJECT_TYPE_RAYTRACING_PIPELINE_CONFIG,
			pDesc: &pipeline_config as *const _ as _,
		});

		let shader_config = D3D12_RAYTRACING_SHADER_CONFIG {
			MaxPayloadSizeInBytes: desc.max_payload_size,
			MaxAttributeSizeInBytes: desc.max_attribute_size,
		};

		subobjects.push(D3D12_STATE_SUBOBJECT {
			Type: D3D12_STATE_SUBOBJECT_TYPE_RAYTRACING_SHADER_CONFIG,
			pDesc: &shader_config as *const _ as _,
		});

		let global_root_signature = D3D12_GLOBAL_ROOT_SIGNATURE {
			pGlobalRootSignature: unsafe { std::mem::transmute_copy(&root_signature) },
		};

		subobjects.push(D3D12_STATE_SUBOBJECT {
			Type: D3D12_STATE_SUBOBJECT_TYPE_GLOBAL_ROOT_SIGNATURE,
			pDesc: &global_root_signature as *const _ as _,
		});

		let mut entries: Vec<HSTRING> = Vec::with_capacity(desc.libraries.len());
		let mut exports: Vec<D3D12_EXPORT_DESC> = Vec::with_capacity(desc.libraries.len());
		let mut libraries: Vec<D3D12_DXIL_LIBRARY_DESC> = Vec::with_capacity(desc.libraries.len());

		for library in &desc.libraries {
			entries.push(HSTRING::from(&library.entry));
		
			exports.push(D3D12_EXPORT_DESC {
				Name: PCWSTR(entries.last().unwrap().as_ptr()),
				ExportToRename: PCWSTR::null(),
				Flags: D3D12_EXPORT_FLAG_NONE,
			});

			libraries.push(D3D12_DXIL_LIBRARY_DESC {
				DXILLibrary: D3D12_SHADER_BYTECODE {
					pShaderBytecode: library.shader.as_ptr() as _,
					BytecodeLength: library.shader.len(),
				},
				NumExports: 1,
				pExports: exports.last().unwrap() as *const _ as _,
			});

			subobjects.push(D3D12_STATE_SUBOBJECT {
				Type: D3D12_STATE_SUBOBJECT_TYPE_DXIL_LIBRARY,
				pDesc: libraries.last().unwrap() as *const _ as _,
			});
		}

		let mut groups: Vec<D3D12_HIT_GROUP_DESC> = Vec::with_capacity(desc.groups.len());

		for group in &desc.groups {
			if group.ty == super::ShaderGroupType::General {
				continue;
			}

			let dx_group_type = match group.ty {
				super::ShaderGroupType::General | super::ShaderGroupType::Triangles => D3D12_HIT_GROUP_TYPE_TRIANGLES,
				super::ShaderGroupType::Procedural => D3D12_HIT_GROUP_TYPE_PROCEDURAL_PRIMITIVE,
			};

			entries.push(HSTRING::from(&group.name));

			groups.push(D3D12_HIT_GROUP_DESC {
				Type: dx_group_type,
				HitGroupExport: PCWSTR(entries.last().unwrap().as_ptr()),
				AnyHitShaderImport: group.any_hit.map_or(PCWSTR::null(), |i| exports[i as usize].Name),
				ClosestHitShaderImport: group.closest_hit.map_or(PCWSTR::null(), |i| exports[i as usize].Name),
				IntersectionShaderImport: group.intersection.map_or(PCWSTR::null(), |i| exports[i as usize].Name),
			});

			subobjects.push(D3D12_STATE_SUBOBJECT {
				Type: D3D12_STATE_SUBOBJECT_TYPE_HIT_GROUP,
				pDesc: groups.last().unwrap() as *const _ as _,
			});
		}

		let state_object_desc = D3D12_STATE_OBJECT_DESC {
			Type: D3D12_STATE_OBJECT_TYPE_RAYTRACING_PIPELINE,
			NumSubobjects: subobjects.len() as u32,
			pSubobjects: subobjects.as_ptr(),
		};

		let state_object: ID3D12StateObject = unsafe { self.device.CreateStateObject(&state_object_desc) }.unwrap();

		Ok(RaytracingPipeline {
			state_object,
			root_signature,
		})
	}

	fn create_texture_view(&mut self, desc: &super::TextureViewDesc, texture: &Texture) -> super::TextureView {
		let h = self.resource_heap.allocate();

		unsafe {
			let desc = D3D12_UNORDERED_ACCESS_VIEW_DESC {
				Format: texture.resource.GetDesc().Format,
				ViewDimension: D3D12_UAV_DIMENSION_TEXTURE2D,
				Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
					Texture2D: D3D12_TEX2D_UAV {
						MipSlice: desc.first_mip_level,
						PlaneSlice: 0,
					}
				},
			};

			self.device.CreateUnorderedAccessView(&texture.resource, None, Some(&desc), h);
		}

		let index = self.resource_heap.handle_to_index(&h);

		super::TextureView {
			index: index as u32,
		}
	}

	fn upload_buffer(&mut self, buffer: &Self::Buffer, data: &[u8]) {
		unsafe {
			let size = data.len();
			let mut upload: Option<ID3D12Resource> = None;
			self.device.CreateCommittedResource3(
				&D3D12_HEAP_PROPERTIES {
					Type: D3D12_HEAP_TYPE_UPLOAD,
					..Default::default()
				},
				D3D12_HEAP_FLAG_NONE,
				&D3D12_RESOURCE_DESC1 {
					Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
					Alignment: 0,
					Width: size as u64,
					Height: 1,
					DepthOrArraySize: 1,
					MipLevels: 1,
					Format: DXGI_FORMAT_UNKNOWN,
					SampleDesc: DXGI_SAMPLE_DESC {
						Count: 1,
						Quality: 0,
					},
					Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
					Flags: D3D12_RESOURCE_FLAG_NONE,
					SamplerFeedbackMipRegion: D3D12_MIP_REGION::default(),
				},
				D3D12_BARRIER_LAYOUT_UNDEFINED,
				None,
				None,
				None,
				&mut upload,
			).unwrap();

			let range = D3D12_RANGE {
				Begin: 0,
				End: size,
			};
			let mut map_data = std::ptr::null_mut();
			let res = upload.clone().unwrap();
			res.Map(0, Some(&range), Some(&mut map_data)).unwrap();
			if !map_data.is_null() {
				let src = data.as_ptr() as *mut u8;
				std::ptr::copy_nonoverlapping(src, map_data as *mut u8, size);
			}
			res.Unmap(0, None);

			let fence: ID3D12Fence = self.device.CreateFence(0, D3D12_FENCE_FLAG_NONE).unwrap();

			self.command_list.CopyResource(&buffer.resource, &upload.clone().unwrap());

			buffer_barrier(
				&self.command_list,
				&buffer.resource,
			);

			self.command_list.Close().unwrap();

			let cmd = Some(self.command_list.can_clone_into());
			self.command_queue.ExecuteCommandLists(&[cmd]);
			self.command_queue.Signal(&fence, 1).unwrap();

			let event = CreateEventA(None, false, false, None).unwrap();
			fence.SetEventOnCompletion(1, event).unwrap();
			WaitForSingleObject(event, INFINITE);

			self.command_list.Reset(&self.command_allocator, None).unwrap();
		}
	}

	fn upload_texture(&mut self, texture: &Self::Texture, data: &[u8]) {
		unsafe {
			let desc = &texture.desc;

			let size_bytes = desc.format.size(desc.width, desc.height, desc.depth) as usize;
			assert_eq!(size_bytes, data.len());

			// TODO: Relax constraints below, make texture uploading more robust in general.
			assert_eq!(desc.mip_levels, 1);
			assert_eq!(desc.depth, 1);

			let row_pitch = desc.format.row_pitch(desc.width);
			let upload_pitch = super::align_pow2(row_pitch, D3D12_TEXTURE_DATA_PITCH_ALIGNMENT as u64);
			let upload_size = desc.height * upload_pitch;
			let initial_layout = map_texture_layout(desc.layout);

			let mut upload: Option<ID3D12Resource> = None;
			self.device.CreateCommittedResource3(
				&D3D12_HEAP_PROPERTIES {
					Type: D3D12_HEAP_TYPE_UPLOAD,
					..Default::default()
				},
				D3D12_HEAP_FLAG_NONE,
				&D3D12_RESOURCE_DESC1 {
					Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
					Alignment: 0,
					Width: upload_size,
					Height: 1,
					DepthOrArraySize: 1,
					MipLevels: 1,
					Format: DXGI_FORMAT_UNKNOWN,
					SampleDesc: DXGI_SAMPLE_DESC {
						Count: 1,
						Quality: 0,
					},
					Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
					Flags: D3D12_RESOURCE_FLAG_NONE,
					SamplerFeedbackMipRegion: D3D12_MIP_REGION::default(),
				},
				D3D12_BARRIER_LAYOUT_UNDEFINED,
				None,
				None,
				None,
				&mut upload,
			).unwrap();

			let range = D3D12_RANGE {
				Begin: 0,
				End: upload_size as usize,
			};
			let mut map_data = std::ptr::null_mut();
			let res = upload.clone().unwrap();
			res.Map(0, Some(&range), Some(&mut map_data)).unwrap();
			if !map_data.is_null() {
				for y in 0..desc.height {
					let src = data.as_ptr().offset((y * row_pitch) as isize) as *const u8;
					let dst = (map_data as *mut u8).offset((y * upload_pitch) as isize);
					std::ptr::copy_nonoverlapping(src, dst, (row_pitch) as usize);
				}
			}
			res.Unmap(0, None);

			let fence: ID3D12Fence = self.device.CreateFence(0, D3D12_FENCE_FLAG_NONE).unwrap();

			let src = D3D12_TEXTURE_COPY_LOCATION {
				pResource: std::mem::transmute_copy(&upload.unwrap()),
				Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
				Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
					PlacedFootprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
						Offset: 0,
						Footprint: D3D12_SUBRESOURCE_FOOTPRINT {
							Format: map_format(desc.format),
							Width: desc.width as u32,
							Height: desc.height as u32,
							Depth: 1,
							RowPitch: upload_pitch as u32,
						},
					},
				},
			};

			let dst = D3D12_TEXTURE_COPY_LOCATION {
				pResource: std::mem::transmute_copy(&texture.resource),
				Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
				Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
					SubresourceIndex: 0,
				},
			};

			texture_barrier(
				&self.command_list,
				&texture.resource,
				initial_layout,
				D3D12_BARRIER_LAYOUT_COPY_DEST,
			);

			self.command_list.CopyTextureRegion(&dst, 0, 0, 0, &src, None);

			texture_barrier(
				&self.command_list,
				&texture.resource,
				D3D12_BARRIER_LAYOUT_COPY_DEST,
				initial_layout,
			);

			self.command_list.Close().unwrap();

			let cmd = Some(self.command_list.can_clone_into());
			self.command_queue.ExecuteCommandLists(&[cmd]);
			self.command_queue.Signal(&fence, 1).unwrap();

			let event = CreateEventA(None, false, false, None).unwrap();
			fence.SetEventOnCompletion(1, event).unwrap();
			WaitForSingleObject(event, INFINITE);
			self.command_list.Reset(&self.command_allocator, None).unwrap();
		}
	}

	fn submit(&self, cmd: &CmdList) {
		unsafe {
			let command_list = &cmd.command_list[cmd.bb_index];
			command_list.Close().unwrap();
			self.command_queue.ExecuteCommandLists(&[Some(command_list.cast().unwrap())]);
		}
	}

	fn adapter_info(&self) -> &super::AdapterInfo {
		&self.adapter_info
	}

	fn capabilities(&self) -> &super::Capabilities {
		&self.capabilities
	}

	fn acceleration_structure_sizes(&self, desc: &super::AccelerationStructureBuildInputs) -> super::AccelerationStructureSizes {
		let info = AccelerationStructureInfo::build(desc);

		let mut prebuild_info = D3D12_RAYTRACING_ACCELERATION_STRUCTURE_PREBUILD_INFO::default();
		unsafe { self.device.GetRaytracingAccelerationStructurePrebuildInfo(&info.desc, &mut prebuild_info); }

		super::AccelerationStructureSizes {
			acceleration_structure_size: prebuild_info.ResultDataMaxSizeInBytes as _,
			build_scratch_buffer_size: prebuild_info.ScratchDataSizeInBytes as _,
			update_scratch_buffer_size: prebuild_info.UpdateScratchDataSizeInBytes as _,
		}
	}
}

impl SwapChain {
	fn wait_for_frame(&mut self, frame_index: usize) {
		unsafe {
			let mut fv = self.frame_fence_value[frame_index];

			if fv != 0 {
				fv = 0;
				self.fence.SetEventOnCompletion(fv, self.fence_event).unwrap();
				WaitForMultipleObjects(&[self.swap_chain.GetFrameLatencyWaitableObject(), self.fence_event], true, INFINITE);
			} else {
				WaitForMultipleObjects(&[self.swap_chain.GetFrameLatencyWaitableObject()], true, INFINITE);
			}
		}
	}
}

impl super::SwapChainImpl<Device> for SwapChain {
	fn wait_for_last_frame(&mut self) {
		self.wait_for_frame(self.bb_index);
	}

	fn num_buffers(&self) -> u32 {
		self.num_buffers
	}

	fn update(&mut self, device: &mut Device, size: [u32; 2]) {
		if size == self.size {
			self.wait_for_frame(self.bb_index);
			return;
		}

		self.wait_for_frame(self.bb_index);

		for texture in &self.backbuffer_textures {
			if texture.rtv.is_some() {
				device.rtv_heap.deallocate(&texture.rtv.unwrap());
			}
		}

		unsafe {
			self.swap_chain.ResizeBuffers(self.num_buffers, size[0], size[1], DXGI_FORMAT_UNKNOWN, self.flags).unwrap();
		}

		self.backbuffer_textures = create_swap_chain_rtv(&self.swap_chain, device, self.num_buffers);

		self.size = size;
		self.bb_index = 0;
	}

	fn backbuffer_index(&self) -> u32 {
		self.bb_index as u32
	}

	fn backbuffer_texture(&self) -> &Texture {
		&self.backbuffer_textures[self.bb_index]
	}

	fn swap(&mut self, device: &Device) {
		unsafe {
			self.swap_chain.Present(0, DXGI_PRESENT_ALLOW_TEARING).ok().unwrap();

			let fv = self.fence_last_signalled_value + 1;
			device.command_queue.Signal(&self.fence, fv as u64).unwrap();

			self.fence_last_signalled_value = fv;
			self.frame_fence_value[self.bb_index as usize] = fv;
			self.require_wait[self.bb_index] = true;

			self.bb_index = (self.bb_index + 1) % self.num_buffers as usize;
		}
	}
}

impl CmdList {
	fn cmd(&self) -> &ID3D12GraphicsCommandList7 {
		&self.command_list[self.bb_index]
	}
}

impl super::CmdListImpl<Device> for CmdList {
	fn reset(&mut self, device: &Device, swap_chain: &SwapChain) {
		let bb = unsafe { swap_chain.swap_chain.GetCurrentBackBufferIndex() as usize };
		self.bb_index = bb;
		if swap_chain.frame_fence_value[bb] != 0 {
			unsafe {
				self.command_allocator[bb].Reset().unwrap();
				self.command_list[bb].Reset(&self.command_allocator[bb], None).unwrap();
			}
		}

		unsafe {
			self.cmd().SetDescriptorHeaps(&[
				Some(device.resource_heap.heap.clone()),
				Some(device.sampler_heap.heap.clone()),
			]);
		}
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
			self.cmd().CopyBufferRegion(&dst.resource, dst_offset, &src.resource, src_offset, size);
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
			self.cmd().CopyTextureRegion(
				&D3D12_TEXTURE_COPY_LOCATION {
					pResource: ManuallyDrop::new(Some(dst.resource.clone())),
					Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
					Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
						// TODO: Account for plane slices (aspects): color = 0, depth = 0, stencil = 1
						// https://learn.microsoft.com/en-us/windows/win32/direct3d12/subresources#plane-slice
						// subresource_index = mip_level + (array_slice + plane * desc.array_size) * desc.mip_levels
						// Also update the other copy_texture functions
						SubresourceIndex: dst_mip_level + dst_array_slice * dst.resource.GetDesc().MipLevels as u32,
					},
				},
				dst_offset[0],
				dst_offset[1],
				dst_offset[2],
				&D3D12_TEXTURE_COPY_LOCATION {
					pResource: ManuallyDrop::new(Some(src.resource.clone())),
					Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
					Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
						SubresourceIndex: src_mip_level + src_array_slice * src.resource.GetDesc().MipLevels as u32,
					},
				},
				Some(&map_box(&src_offset, &size)),
			);
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
		let src = &src.resource;
		let dst = &dst.resource;

		unsafe {
			self.cmd().CopyTextureRegion(
				&D3D12_TEXTURE_COPY_LOCATION {
					pResource: ManuallyDrop::new(Some(dst.clone())),
					Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
					Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
						SubresourceIndex: dst_mip_level + dst_array_slice * dst.GetDesc().MipLevels as u32,
					},
				},
				dst_offset[0],
				dst_offset[1],
				dst_offset[2],
				&D3D12_TEXTURE_COPY_LOCATION {
					pResource: ManuallyDrop::new(Some(src.clone())),
					Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
					Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
						PlacedFootprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
							Offset: src_offset,
							Footprint: D3D12_SUBRESOURCE_FOOTPRINT {
								Format: dst.GetDesc().Format,
								Width: size[0],
								Height: size[1],
								Depth: size[2],
								RowPitch: src_bytes_per_row,
							},
						},
					},
				},
				None,
			);
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
		let src = &src.resource;
		let dst = &dst.resource;

		unsafe {
			self.cmd().CopyTextureRegion(
				&D3D12_TEXTURE_COPY_LOCATION {
					pResource: ManuallyDrop::new(Some(src.clone())),
					Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
					Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
						PlacedFootprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
							Offset: dst_offset,
							Footprint: D3D12_SUBRESOURCE_FOOTPRINT {
								Format: src.GetDesc().Format,
								Width: size[0],
								Height: size[1],
								Depth: size[2],
								RowPitch: dst_bytes_per_row,
							},
						},
					},
				},
				0,
				0,
				0,
				&D3D12_TEXTURE_COPY_LOCATION {
					pResource: ManuallyDrop::new(Some(dst.clone())),
					Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
					Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
						SubresourceIndex: src_mip_level + src_array_slice * src.GetDesc().MipLevels as u32,
					},
				},
				Some(&map_box(&src_offset, &size)),
			);
		}
	}

	fn render_pass_begin(&self, desc: &super::RenderPassDesc<Device>) {
		let (rt_begin_type, rt_clear) = map_load_op(desc.color_load);
		let rt = desc.color_attachments.iter().map(|target| {
			let desc = unsafe { target.resource.GetDesc() };

			let begin = D3D12_RENDER_PASS_BEGINNING_ACCESS {
				Type: rt_begin_type,
				Anonymous: D3D12_RENDER_PASS_BEGINNING_ACCESS_0 {
					Clear: D3D12_RENDER_PASS_BEGINNING_ACCESS_CLEAR_PARAMETERS {
						ClearValue: D3D12_CLEAR_VALUE {
							Format: desc.Format,
							Anonymous: D3D12_CLEAR_VALUE_0 {
								Color: rt_clear.into(),
							},
						},
					},
				},
			};

			let end = D3D12_RENDER_PASS_ENDING_ACCESS {
				Type: D3D12_RENDER_PASS_ENDING_ACCESS_TYPE_PRESERVE,
				Anonymous: D3D12_RENDER_PASS_ENDING_ACCESS_0 {
					Resolve: Default::default(),
				},
			};

			D3D12_RENDER_PASS_RENDER_TARGET_DESC {
				cpuDescriptor: target.rtv.unwrap(),
				BeginningAccess: begin,
				EndingAccess: end,
			}
		}).collect::<Vec<_>>();

		let ds = desc.depth_stencil.map(|depth_stencil| {
			let (depth_begin_type, depth_clear) = map_load_op(desc.depth_load);
			let (stencil_begin_type, stencil_clear) = map_load_op(desc.stencil_load);

			let ds_desc = unsafe { depth_stencil.resource.GetDesc() };

			let depth_begin = D3D12_RENDER_PASS_BEGINNING_ACCESS {
				Type: depth_begin_type,
				Anonymous: D3D12_RENDER_PASS_BEGINNING_ACCESS_0 {
					Clear: D3D12_RENDER_PASS_BEGINNING_ACCESS_CLEAR_PARAMETERS {
						ClearValue: D3D12_CLEAR_VALUE {
							Format: ds_desc.Format,
							Anonymous: D3D12_CLEAR_VALUE_0 {
								DepthStencil: D3D12_DEPTH_STENCIL_VALUE {
									Depth: depth_clear,
									Stencil: stencil_clear,
								},
							},
						},
					},
				},
			};

			let depth_end = D3D12_RENDER_PASS_ENDING_ACCESS {
				Type: D3D12_RENDER_PASS_ENDING_ACCESS_TYPE_PRESERVE,
				Anonymous: D3D12_RENDER_PASS_ENDING_ACCESS_0 {
					Resolve: Default::default(),
				},
			};

			let stencil_begin = D3D12_RENDER_PASS_BEGINNING_ACCESS {
				Type: stencil_begin_type,
				Anonymous: D3D12_RENDER_PASS_BEGINNING_ACCESS_0 {
					Clear: D3D12_RENDER_PASS_BEGINNING_ACCESS_CLEAR_PARAMETERS {
						ClearValue: D3D12_CLEAR_VALUE {
							Format: ds_desc.Format,
							Anonymous: D3D12_CLEAR_VALUE_0 {
								DepthStencil: D3D12_DEPTH_STENCIL_VALUE {
									Depth: depth_clear,
									Stencil: stencil_clear,
								},
							},
						},
					},
				},
			};

			let stencil_end = D3D12_RENDER_PASS_ENDING_ACCESS {
				Type: D3D12_RENDER_PASS_ENDING_ACCESS_TYPE_PRESERVE,
				Anonymous: D3D12_RENDER_PASS_ENDING_ACCESS_0 {
					Resolve: Default::default(),
				},
			};

			D3D12_RENDER_PASS_DEPTH_STENCIL_DESC {
				cpuDescriptor: depth_stencil.dsv.unwrap(),
				DepthBeginningAccess: depth_begin,
				StencilBeginningAccess: stencil_begin,
				DepthEndingAccess: depth_end,
				StencilEndingAccess: stencil_end,
			}
		});

		unsafe {
			self.cmd().BeginRenderPass(Some(rt.as_slice()), ds.map_or(None, |ds| Some(&ds)), D3D12_RENDER_PASS_FLAG_NONE);
		}
	}

	fn render_pass_end(&self) {
		unsafe {
			self.cmd().EndRenderPass();
		}
	}

	fn barriers(&self, barriers: &super::Barriers<Device>) {
		let global_barriers = barriers.global.iter().map(|_| D3D12_GLOBAL_BARRIER {
			SyncBefore: D3D12_BARRIER_SYNC_ALL,
			SyncAfter: D3D12_BARRIER_SYNC_ALL,
			AccessBefore: D3D12_BARRIER_ACCESS_UNORDERED_ACCESS,
			AccessAfter: D3D12_BARRIER_ACCESS_UNORDERED_ACCESS,
		}).collect::<Vec<_>>();

		let buffer_barriers = barriers.buffer.iter().map(|_| {
			todo!()
		}).collect::<Vec<_>>();

		let texture_barriers = barriers.texture.iter().map(|barrier| D3D12_TEXTURE_BARRIER {
			SyncBefore: D3D12_BARRIER_SYNC_ALL,
			SyncAfter: D3D12_BARRIER_SYNC_ALL,
			AccessBefore: D3D12_BARRIER_ACCESS_COMMON,
			AccessAfter: D3D12_BARRIER_ACCESS_COMMON,
			LayoutBefore: map_texture_layout(barrier.old_layout),
			LayoutAfter: map_texture_layout(barrier.new_layout),
			pResource: unsafe { std::mem::transmute_copy(&barrier.texture.resource) },
			Subresources: D3D12_BARRIER_SUBRESOURCE_RANGE {
				IndexOrFirstMipLevel: 0xffffffff, // All subresources
				NumMipLevels: 0,
				FirstArraySlice: 0,
				NumArraySlices: 0,
				FirstPlane: 0,
				NumPlanes: 0,
			},
			Flags: D3D12_TEXTURE_BARRIER_FLAG_NONE,
		}).collect::<Vec<_>>();

		let barrier_groups = [
			D3D12_BARRIER_GROUP {
				Type: D3D12_BARRIER_TYPE_GLOBAL,
				NumBarriers: global_barriers.len() as u32,
				Anonymous: D3D12_BARRIER_GROUP_0 {
					pGlobalBarriers: global_barriers.as_ptr(),
				},
			},
			D3D12_BARRIER_GROUP {
				Type: D3D12_BARRIER_TYPE_BUFFER,
				NumBarriers: buffer_barriers.len() as u32,
				Anonymous: D3D12_BARRIER_GROUP_0 {
					pBufferBarriers: buffer_barriers.as_ptr(),
				},
			},
			D3D12_BARRIER_GROUP {
				Type: D3D12_BARRIER_TYPE_TEXTURE,
				NumBarriers: texture_barriers.len() as u32,
				Anonymous: D3D12_BARRIER_GROUP_0 {
					pTextureBarriers: texture_barriers.as_ptr(),
				},
			},
		];

		unsafe {
			self.cmd().Barrier(&barrier_groups);
		}
	}

	fn set_viewport(&self, rect: &super::Rect<f32>, depth: Range<f32>) {
		let dx_viewport = D3D12_VIEWPORT {
			TopLeftX: rect.left,
			TopLeftY: rect.top,
			Width: rect.right - rect.left,
			Height: rect.bottom - rect.top,
			MinDepth: depth.start,
			MaxDepth: depth.end,
		};

		unsafe {
			self.cmd().RSSetViewports(&[dx_viewport]);
		}
	}

	fn set_scissor(&self, rect: &super::Rect<u32>) {
		let dx_rect = RECT {
			left: rect.left as i32,
			top: rect.top as i32,
			right: rect.right as i32,
			bottom: rect.bottom as i32,
		};
		
		unsafe {
			self.cmd().RSSetScissorRects(&[dx_rect]);
		}
	}

	fn set_blend_constant(&self, color: super::Color<f32>) {
		unsafe {
			self.cmd().OMSetBlendFactor(Some(&color.into()));
		}
	}

	fn set_stencil_reference(&self, reference: u32) {
		unsafe {
			self.cmd().OMSetStencilRef(reference);
		}
	}

	fn set_index_buffer(&self, buffer: &Buffer, offset: u64, format: super::Format) {
		let ibv = D3D12_INDEX_BUFFER_VIEW {
			BufferLocation: unsafe { buffer.resource.GetGPUVirtualAddress() },
			SizeInBytes: (buffer.size - offset) as u32,
			Format: map_format(format),
		};

		unsafe {
			self.cmd().IASetIndexBuffer(Some(&ibv));
		}
	}

	fn set_graphics_pipeline(&self, pipeline: &GraphicsPipeline) {
		let cmd = self.cmd();
		unsafe {
			cmd.SetGraphicsRootSignature(&pipeline.root_signature);
			cmd.SetPipelineState(&pipeline.pipeline_state);
			cmd.IASetPrimitiveTopology(pipeline.topology);
			cmd.SetGraphicsRootDescriptorTable(1, self.resource_heap_base);
		}
	}

	fn set_compute_pipeline(&self, pipeline: &ComputePipeline) {
		let cmd = self.cmd();
		unsafe {
			cmd.SetComputeRootSignature(&pipeline.root_signature);
			cmd.SetPipelineState(&pipeline.pipeline_state);
			cmd.SetComputeRootDescriptorTable(1, self.resource_heap_base);
		}
	}

	fn set_raytracing_pipeline(&self, pipeline: &RaytracingPipeline) {
		let cmd = self.cmd();
		unsafe {
			cmd.SetComputeRootSignature(&pipeline.root_signature);
			cmd.SetPipelineState1(&pipeline.state_object);
			cmd.SetComputeRootDescriptorTable(1, self.resource_heap_base);
		}
	}

	fn graphics_push_constants(&self, offset: u32, data: &[u8]) {
		assert_eq!(offset % 4, 0);
		assert_eq!(data.len() % 4, 0);
		
		unsafe {
			self.cmd().SetGraphicsRoot32BitConstants(0, data.len() as u32 / 4, data.as_ptr() as *const _, offset / 4);
		}
	}

	fn compute_push_constants(&self, offset: u32, data: &[u8]) {
		assert_eq!(offset % 4, 0);
		assert_eq!(data.len() % 4, 0);

		unsafe {
			self.cmd().SetComputeRoot32BitConstants(0, data.len() as u32 / 4, data.as_ptr() as *const _, offset / 4);
		}
	}

	fn draw(&self, vertices: Range<u32>, instances: Range<u32>) {
		unsafe {
			self.cmd().DrawInstanced(vertices.len() as u32, instances.len() as u32, vertices.start, instances.start);
		}
	}

	fn draw_indexed(&self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
		unsafe {
			self.cmd().DrawIndexedInstanced(indices.len() as u32, instances.len() as u32, indices.start, base_vertex, instances.start);
		}
	}

	fn dispatch(&self, x: u32, y: u32, z: u32) {
		unsafe {
			self.cmd().Dispatch(x, y, z);
		}
	}

	fn dispatch_rays(&self, desc: &super::DispatchRaysDesc) {
		let dx_desc = D3D12_DISPATCH_RAYS_DESC {
			RayGenerationShaderRecord: desc.raygen.as_ref().map_or(Default::default(), |t| D3D12_GPU_VIRTUAL_ADDRESS_RANGE {
				StartAddress: t.ptr.0,
				SizeInBytes: t.size as _,
			}),
			MissShaderTable: desc.miss.as_ref().map_or(Default::default(), |t| D3D12_GPU_VIRTUAL_ADDRESS_RANGE_AND_STRIDE {
				StartAddress: t.ptr.0,
				SizeInBytes: t.size as _,
				StrideInBytes: t.stride as _,
			}),
			HitGroupTable: desc.hit_group.as_ref().map_or(Default::default(), |t| D3D12_GPU_VIRTUAL_ADDRESS_RANGE_AND_STRIDE {
				StartAddress: t.ptr.0,
				SizeInBytes: t.size as _,
				StrideInBytes: t.stride as _,
			}),
			CallableShaderTable: desc.callable.as_ref().map_or(Default::default(), |t| D3D12_GPU_VIRTUAL_ADDRESS_RANGE_AND_STRIDE {
				StartAddress: t.ptr.0,
				SizeInBytes: t.size as _,
				StrideInBytes: t.stride as _,
			}),
			Width: desc.width,
			Height: desc.height,
			Depth: desc.depth,
		};

		unsafe {
			self.cmd().DispatchRays(&dx_desc);
		}
	}

	fn build_acceleration_structure(&self, desc: &super::AccelerationStructureBuildDesc<Device>) {
		let info = AccelerationStructureInfo::build(&desc.inputs);

		// TODO: D3D12_RAYTRACING_ACCELERATION_STRUCTURE_BUILD_FLAG_PERFORM_UPDATE if desc.src.is_some()

		let dx_desc = D3D12_BUILD_RAYTRACING_ACCELERATION_STRUCTURE_DESC {
			DestAccelerationStructureData: unsafe { desc.dst.resource.GetGPUVirtualAddress() },
			Inputs: info.desc,
			SourceAccelerationStructureData: desc.src.map_or(0, |b| unsafe { b.resource.GetGPUVirtualAddress() }),
			ScratchAccelerationStructureData: desc.scratch_data.0,
		};

		unsafe {
			self.cmd().BuildRaytracingAccelerationStructure(&dx_desc, None)
		}
	}

	fn debug_marker(&self, name: &str, color: super::Color<u8>) {
		if let Some(pix) = &self.pix {
			let color = 0xff000000 | (color.r as u32) << 16 | (color.g as u32) << 8 | color.b as u32;
			pix.set_marker_on_command_list(self.cmd(), color as u64, name);
		}
	}

	fn debug_event_push(&self, name: &str, color: super::Color<u8>) {
		if let Some(pix) = &self.pix {
			let color = 0xff000000 | (color.r as u32) << 16 | (color.g as u32) << 8 | color.b as u32;
			pix.begin_event_on_command_list(self.cmd(), color as u64, name);
		}
	}

	fn debug_event_pop(&self) {
		if let Some(pix) = &self.pix {
			pix.end_event_on_command_list(self.cmd());
		}
	}
}

impl super::BufferImpl<Device> for Buffer {
	fn srv_index(&self) -> Option<u32> {
		self.srv_index.map(|i| i as u32)
	}

	fn uav_index(&self) -> Option<u32> {
		self.uav_index.map(|i| i as u32)
	}

	fn cpu_ptr(&self) -> *mut u8 {
		self.mapped_ptr
	}

	fn gpu_ptr(&self) -> super::GpuPtr {
		super::GpuPtr(unsafe {
			self.resource.GetGPUVirtualAddress()
		})
	}
}

impl super::TextureImpl<Device> for Texture {
	fn srv_index(&self) -> Option<u32> {
		self.srv_index.map(|i| i as u32)
	}

	fn uav_index(&self) -> Option<u32> {
		self.uav_index.map(|i| i as u32)
	}
}

impl super::SamplerImpl<Device> for Sampler {}

impl super::AccelerationStructureImpl<Device> for AccelerationStructure {
	fn srv_index(&self) -> Option<u32> {
		self.srv_index.map(|i| i as u32)
	}

	fn instance_descriptor_size() -> usize {
		std::mem::size_of::<D3D12_RAYTRACING_INSTANCE_DESC>()
	}

	fn write_instance_descriptor(instance: &super::AccelerationStructureInstance, slice: &mut [u8]) {
		let t = &instance.transform;

		let dx_instance = D3D12_RAYTRACING_INSTANCE_DESC {
			Transform: [
				t[0][0], t[0][1], t[0][2], t[0][3],
				t[1][0], t[1][1], t[1][2], t[1][3],
				t[2][0], t[2][1], t[2][2], t[2][3],
			],
			_bitfield1: (instance.user_id & 0xffffff) | (u32::from(instance.mask) << 24),
			_bitfield2: (instance.contribution_to_hit_group_index & 0xffffff) | ((map_acceleration_structure_instance_flags(instance.flags).0 as u32) << 24),
			AccelerationStructure: instance.bottom_level.0,
		};

		unsafe {
			std::ptr::copy_nonoverlapping(&dx_instance as *const _ as _, slice.as_mut_ptr(), std::mem::size_of::<D3D12_RAYTRACING_INSTANCE_DESC>());
		}
	}
}

impl super::GraphicsPipelineImpl<Device> for GraphicsPipeline {}
impl super::ComputePipelineImpl<Device> for ComputePipeline {}

impl super::RaytracingPipelineImpl<Device> for RaytracingPipeline {
	fn shader_identifier_size(&self) -> usize {
		D3D12_SHADER_IDENTIFIER_SIZE_IN_BYTES as usize
	}

	fn write_shader_identifier(&self, name: &str, slice: &mut [u8]) {
		let hstring = HSTRING::from(name);
		let state_object_properties: ID3D12StateObjectProperties = self.state_object.cast().unwrap();

		unsafe {
			let identifier = state_object_properties.GetShaderIdentifier(&hstring);
			std::ptr::copy_nonoverlapping(identifier as *const u8, slice.as_mut_ptr(), D3D12_SHADER_IDENTIFIER_SIZE_IN_BYTES as usize);
		}
	}
}

struct AccelerationStructureInfo {
	desc: D3D12_BUILD_RAYTRACING_ACCELERATION_STRUCTURE_INPUTS,
	_geometry_descs: Vec<D3D12_RAYTRACING_GEOMETRY_DESC>, // Here to keep the geometries alive
}

// TODO: Move into CmdList::build_acceleration_structure
impl AccelerationStructureInfo {
	fn build(inputs: &super::AccelerationStructureBuildInputs) -> Self {
		let geometries: Vec<D3D12_RAYTRACING_GEOMETRY_DESC> = inputs.geometry.iter().map(|g| match &g.part {
			super::GeometryPart::AABBs(aabbs) => D3D12_RAYTRACING_GEOMETRY_DESC {
				Type: D3D12_RAYTRACING_GEOMETRY_TYPE_PROCEDURAL_PRIMITIVE_AABBS,
				Flags: map_acceleration_structure_bottom_level_flags(g.flags),
				Anonymous: D3D12_RAYTRACING_GEOMETRY_DESC_0 {
					AABBs: D3D12_RAYTRACING_GEOMETRY_AABBS_DESC {
						AABBCount: aabbs.count as _,
						AABBs: D3D12_GPU_VIRTUAL_ADDRESS_AND_STRIDE {
							StartAddress: aabbs.data.0,
							StrideInBytes: aabbs.stride as _,
						},
					}
				},
			},
			super::GeometryPart::Triangles(triangles) => D3D12_RAYTRACING_GEOMETRY_DESC {
				Type: D3D12_RAYTRACING_GEOMETRY_TYPE_TRIANGLES,
				Flags: map_acceleration_structure_bottom_level_flags(g.flags),
				Anonymous: D3D12_RAYTRACING_GEOMETRY_DESC_0 {
					Triangles: D3D12_RAYTRACING_GEOMETRY_TRIANGLES_DESC {
						Transform3x4: triangles.transform.0,
						IndexFormat: map_format(triangles.index_format),
						VertexFormat: map_format(triangles.vertex_format),
						IndexCount: triangles.index_count as _,
						VertexCount: triangles.vertex_count as _,
						IndexBuffer: triangles.index_buffer.0,
						VertexBuffer: D3D12_GPU_VIRTUAL_ADDRESS_AND_STRIDE {
							StartAddress: triangles.vertex_buffer.0,
							StrideInBytes: triangles.vertex_stride as _,
						},
					},
				},
			},
		}).collect();

		let mut dx_input = D3D12_BUILD_RAYTRACING_ACCELERATION_STRUCTURE_INPUTS::default();

		dx_input.Flags = map_acceleration_structure_build_flags(inputs.flags);
		dx_input.DescsLayout = D3D12_ELEMENTS_LAYOUT_ARRAY;

		match inputs.kind {
			super::AccelerationStructureKind::BottomLevel => {
				dx_input.Type = D3D12_RAYTRACING_ACCELERATION_STRUCTURE_TYPE_BOTTOM_LEVEL;
				dx_input.NumDescs = inputs.geometry.len() as _;
				dx_input.Anonymous = D3D12_BUILD_RAYTRACING_ACCELERATION_STRUCTURE_INPUTS_0 {
					pGeometryDescs: geometries.as_ptr(),
				};
			},
			super::AccelerationStructureKind::TopLevel => {
				dx_input.Type = D3D12_RAYTRACING_ACCELERATION_STRUCTURE_TYPE_TOP_LEVEL;
				dx_input.NumDescs = inputs.instances.count as _;
				dx_input.Anonymous = D3D12_BUILD_RAYTRACING_ACCELERATION_STRUCTURE_INPUTS_0 {
					InstanceDescs: inputs.instances.data.0,
				};
			},
		};

		Self {
			desc: dx_input,
			_geometry_descs: geometries,
		}
	}
}
