mod d3d12;
mod vulkan;

use std::ops::Range;

use crate::os::NativeHandle;
use slang;

type Error = super::Error;

// TODO: This hardcodes the backend to D3D12. Make it dynamic.
pub type Device = d3d12::Device;
pub type SwapChain = <d3d12::Device as DeviceImpl>::SwapChain;
pub type CmdList = <d3d12::Device as DeviceImpl>::CmdList;
pub type Buffer = <d3d12::Device as DeviceImpl>::Buffer;
pub type Texture = <d3d12::Device as DeviceImpl>::Texture;
pub type Sampler = <d3d12::Device as DeviceImpl>::Sampler;
pub type GraphicsPipeline = <d3d12::Device as DeviceImpl>::GraphicsPipeline;
pub type ComputePipeline = <d3d12::Device as DeviceImpl>::ComputePipeline;
pub type RaytracingPipeline = <d3d12::Device as DeviceImpl>::RaytracingPipeline;
pub type AccelerationStructure = <d3d12::Device as DeviceImpl>::AccelerationStructure;

pub struct ShaderCompiler {
	session: slang::GlobalSession,
}

impl ShaderCompiler {
	pub fn new() -> Self {
		Self{ session: slang::GlobalSession::new() }
	}

	pub fn compile(&self, file: &str, entry_point: &str) -> Vec<u8> {
		let mut compile_request = self.session.create_compile_request();

		compile_request
			.set_optimization_level(slang::OptimizationLevel::High)
			.set_codegen_target(slang::CompileTarget::Dxil)
			.set_target_profile(self.session.find_profile("sm_6_5"))
			.set_matrix_layout_mode(slang::MatrixLayoutMode::RowMajor)
			.add_search_path("shaders");

		let ep = compile_request
			.add_translation_unit(slang::SourceLanguage::Slang, None)
			.add_source_file(file)
			.add_entry_point(entry_point, slang::Stage::None);

		let res = compile_request.compile()
			.expect("Shader compilation errors");

		res.get_entry_point_code(ep).into()
	}
}

/// Returns the highest mip number for a texture with the given resolution.
/// This number is one less than the total number of mip levels.
pub fn max_mip_level(resolution: u32) -> u32 {
	resolution.next_power_of_two().trailing_zeros()
}

/// Returns the resolution of a mip level for a texture with the given resolution.
/// The mip level 0 is the full resolution.
pub fn at_mip_level(resolution: u32, mip_level: u32) -> u32 {
	(resolution >> mip_level).max(1)
}

#[derive(Clone, Copy, Default)]
pub struct Color<T> {
	pub r: T,
	pub g: T,
	pub b: T,
	pub a: T,
}

impl Color<u8> {
	pub fn to_u32(&self) -> u32 {
		(self.r as u32) << 24 | (self.g as u32) << 16 | (self.b as u32) << 8 | (self.a as u32)
	}

	pub fn to_f32(&self) -> Color<f32> {
		Color {
			r: self.r as f32 / 255.0,
			g: self.g as f32 / 255.0,
			b: self.b as f32 / 255.0,
			a: self.a as f32 / 255.0,
		}
	}
}

impl<T> From<Color<T>> for [T; 4] {
	fn from(c: Color<T>) -> Self {
		[c.r, c.g, c.b, c.a]
	}
}

#[derive(Clone, Copy)]
pub struct Rect<T> {
	pub left: T,
	pub top: T,
	pub right: T,
	pub bottom: T,
}

impl<T: Copy + Default> Rect<T> {
	pub fn from_size(size: [T; 2]) -> Self {
		Self {
			left: Default::default(),
			top: Default::default(),
			right: size[0],
			bottom: size[1],
		}
	}
}

impl From<Rect<u32>> for Rect<f32> {
	fn from(r: Rect<u32>) -> Self {
		Self {
			left: r.left as f32,
			top: r.top as f32,
			right: r.right as f32,
			bottom: r.bottom as f32,
		}
	}
}

/// Defines resource formats.
///
/// Each format has one or more components:
/// - R: Red
/// - G: Green
/// - B: Blue
/// - D: Depth
/// - S: Stencil
/// - X: Unused
/// 
/// The number after every component indicates how many bits it occupies.
///
/// Each format also has a type specifier at the end:
/// - UNorm: Unsigned normalized integer.
/// - SNorm: Signed normalized integer.
/// - UInt:  Unsigned integer.
/// - SInt:  Signed integer.
/// - Float: Floating-point value.
#[derive(Clone, Copy, Default)]
pub enum Format {
	#[default]
	Unknown,

	// R Color

	R8UNorm,
	R8SNorm,
	R8UInt,
	R8SInt,

	R16UNorm,
	R16SNorm,
	R16UInt,
	R16SInt,
	R16Float,

	R32UInt,
	R32SInt,
	R32Float,

	// RG Color

	RG8UNorm,
	RG8SNorm,
	RG8UInt,
	RG8SInt,

	RG16UNorm,
	RG16SNorm,
	RG16UInt,
	RG16SInt,
	RG16Float,

	RG32UInt,
	RG32SInt,
	RG32Float,

	// RGB Color

	RGB32UInt,
	RGB32SInt,
	RGB32Float,

	// RGBA Color

	RGBA8UNorm,
	RGBA8SNorm,
	RGBA8UInt,
	RGBA8SInt,

	RGBA16UNorm,
	RGBA16SNorm,
	RGBA16UInt,
	RGBA16SInt,
	RGBA16Float,

	RGBA32UInt,
	RGBA32SInt,
	RGBA32Float,

	// Other formats
	BGRA8UNorm,

	// Depth Stencil
	D16UNorm,
	D24UNormS8UInt,
	D32Float,
	D32FloatS8UIntX24,
}

bitflags! {
	pub struct Validation : u8 {
		/// Enable cpu based command list validation.
		const CPU = 1 << 0;
		/// Enable gpu based address and state validation. Requires `Validation::CPU`.
		const GPU = 1 << 1;
		/// Enable graphics debugger object and event naming.
		const DEBUGGER = 1 << 2;
	}
}

#[derive(Clone, Copy, Default)]
pub enum PowerPreference {
	/// No preference.
	#[default]
	None,
	/// Prefer GPU that uses the least power. This is often an integrated GPU.
	LowPower,
	/// Prefer GPU that has the highest performance. This is often a discrete or external GPU.
	HighPerformance,
}

pub struct DeviceDesc {
	pub validation: Validation,
	pub power_preference: PowerPreference,
}

pub struct AdapterInfo {
	pub name: String,
	pub dedicated_video_memory: usize,
	pub dedicated_system_memory: usize,
	pub shared_system_memory: usize,
	/// List of all available adapters.
	pub available: Vec<String>,
}

pub struct SwapChainDesc {
	pub size: [u32; 2],
	pub num_buffers: u32,
	pub format: Format,
}

#[derive(Clone, Copy)]
pub enum Memory {
	GpuOnly,
	CpuToGpu,
	GpuToCpu,
}

bitflags! {
	#[derive(Clone, Copy)]
	pub struct BufferUsage: u32 {
		const INDEX = 1 << 0;
		const SHADER_RESOURCE = 1 << 1;
		const UNORDERED_ACCESS = 1 << 2;
		const ACCELERATION_STRUCTURE = 1 << 3;
	}
}

#[derive(Clone, Copy)]
pub struct BufferDesc {
	pub size: usize,
	pub usage: BufferUsage,
	pub memory: Memory,
}

#[derive(Clone, Copy)]
pub enum TextureLayout {
	Common,
	Present,
	CopySrc,
	CopyDst,
	ShaderResource,
	UnorderedAccess,
	RenderTarget,
	DepthStencilWrite,
	DepthStencilRead,
}

bitflags! {
	#[derive(Clone, Copy)]
	pub struct TextureUsage: u32 {
		const SHADER_RESOURCE = 1 << 0;
		const UNORDERED_ACCESS = 1 << 1;
		const RENDER_TARGET = 1 << 2;
		const DEPTH_STENCIL = 1 << 3;
	}
}

#[derive(Clone, Copy)]
pub struct TextureDesc {
	pub width: u64,
	pub height: u64,
	pub depth: u32,

	pub array_size: u32,
	pub mip_levels: u32,
	pub samples: u32,

	pub format: Format,
	pub usage: TextureUsage,
	pub layout: TextureLayout,
}

pub struct TextureView {
	pub index: u32,
}

pub struct TextureViewDesc {
	pub first_mip_level: u32,
	pub mip_level_count: u32,
}

#[derive(Clone, Copy)]
pub enum ShaderType {
	Vertex,
	Pixel,
	Compute,

	Raygen,
	Miss,
	Intersection,
	ClosestHit,
	AnyHit,
	Callable,
}

bitflags! {
	pub struct ColorWriteMask : u8 {
		const RED   = 1 << 0;
		const GREEN = 1 << 1;
		const BLUE  = 1 << 2;
		const ALPHA = 1 << 3;
		const ALL   = Self::RED.bits() | Self::GREEN.bits() | Self::BLUE.bits() | Self::ALPHA.bits();
	}
}

pub enum DescriptorType {
	ConstantBuffer,
	ShaderResource,
	UnorderedAccess,
	Sampler,
}

pub struct PushConstantBinding {
	/// Number of bytes to push
	pub size: u32,
}

pub struct DescriptorBinding {
	pub shader_register: u32,
	pub register_space: u32,
	pub binding_type: DescriptorType,
	/// Number of descriptors in this table, use `None` for unbounded.
	pub num_descriptors: Option<u32>,
	/// Offset in decriptors, use `None` to append to previous range.
	pub offset: Option<u32>,
}

pub struct SamplerBinding {
	pub shader_register: u32,
	pub register_space: u32,
	pub sampler_desc: SamplerDesc
}

#[derive(Default)]
pub struct DescriptorLayout {
	pub push_constants: Option<PushConstantBinding>,
	pub bindings: Option<Vec<DescriptorBinding>>,
	pub static_samplers: Option<Vec<SamplerBinding>>,
}

#[derive(Clone, Copy, Default)]
pub enum FilterMode {
	/// Take the nearest texture sample.
	#[default]
	Nearest,
	/// Linear interpolate between multiple texture samples.
	Linear,
}

/// Describes how texture coordinates outside of the [0, 1] range are handled.
#[derive(Clone, Copy, Default)]
pub enum AddressMode {
	/// Clamps the texture coordinates to the [0, 1] range.
	#[default]
	Clamp,
	/// Repeat the texture at every integer junction.
	Repeat,
	/// Repeat the texture and mirror at every integer junction.
	Mirror,
	/// Repeat the texture and mirror only once around the origin.
	MirrorOnce,
	/// Use the color from [`SamplerDesc::border_color`].
	Border,
}

#[derive(Clone, Copy)]
pub enum BorderColor {
	TransparentBlack,
	OpaqueBlack,
	White,
}

#[derive(Clone, Copy)]
pub struct SamplerDesc {
	pub address_u: AddressMode,
	pub address_v: AddressMode,
	pub address_w: AddressMode,

	pub filter_min: FilterMode,
	pub filter_mag: FilterMode,
	pub filter_mip: FilterMode,

	pub min_lod: f32,
	pub max_lod: f32,
	pub lod_bias: f32,

	pub compare: Option<CompareOp>,
	/// Must be at least 1. If higher, all filter modes must be [`FilterMode::Nearest`].
	pub max_anisotropy: u32,
	/// Border color to use if address mode is [`AddressMode::Border`].
	pub border_color: Option<BorderColor>,
}

#[derive(Clone, Copy)]
pub enum CompareOp {
	/// Pass never.
	Never,
	/// Pass always.
	Always,
	/// Pass if source data is equal to destination data.
	Equal,
	/// Pass if source data is not equal to destination data.
	NotEqual,
	/// Pass if source data is less than destination data.
	Less,
	/// Pass if source data is less than or equal to destination data.
	LessEqual,
	/// Pass if source data is greater than destination data.
	Greater,
	/// Pass if source data is greater than or equal to destination data.
	GreaterEqual,
}

#[derive(Clone, Copy)]
pub enum Topology {
	/// Every vertex represents a single point.
	PointList,
	/// Every pair of vertices represents a single line.
	LineList,
	/// Every vertex creates a new line while the previous vertex is used as starting point.
	LineStrip,
	/// Every trio of vertices represent a single triangle.
	TriangleList,
	/// Every vertex creates a new triangle with an alternating winding.
	TriangleStrip,
}

// ----------------------------------------------------------------
// DEPTH STENCIL DESC
// ----------------------------------------------------------------

/// Operation to perform on stencil value.
pub enum StencilOp {
	/// Keep existing stencil value.
	Keep,
	/// Set stencil value to 0.
	Zero,
	/// Replace stencil value with value from [`CmdListImpl::set_stencil_reference`].
	Replace,
	/// Bitwise invert stencil value.
	Invert,
	/// Increment stencil value by 1, wrap on overflow.
	IncrementWrap,
	/// Increment stencil value by 1, clamp on overflow.
	IncrementClamp,
	/// Decrement stencil value by 1, wrap on underflow.
	DecrementWrap,
	/// Decrement stencil value by 1, clamp on underflow.
	DecrementClamp,
}

pub struct DepthStencilFaceDesc {
	pub func: CompareOp,
	pub fail: StencilOp,
	pub depth_fail: StencilOp,
	pub pass: StencilOp,
}

pub struct DepthStencilDesc {
	pub format: Format,

	pub depth_test_enable: bool,
	pub depth_write_enable: bool,
	pub depth_op: CompareOp,

	pub stencil_enable: bool,
	pub stencil_read_mask: u8,
	pub stencil_write_mask: u8,

	pub front_face: DepthStencilFaceDesc,
	pub back_face: DepthStencilFaceDesc,
}

// ----------------------------------------------------------------
// RASTERIZER DESC
// ----------------------------------------------------------------

pub enum PolygonMode {
	/// Draw lines connecting the vertices.
	Line,
	/// Fill the polygons formed by the vertices.
	Fill,
}

pub enum CullMode {
	/// Draw all polygons.
	None,
	/// Don't draw front-facing polygons.
	Front,
	/// Don't draw back-facing polygons.
	Back,
}

pub struct DepthBias {
	/// Constant depth value added to a pixel.
	pub constant: f32,
	/// Scalar factor applied to a pixel's slope.
	pub slope: f32,
	/// Maximum depth bias of a pixel.
	pub clamp: f32,
}

pub struct RasterizerDesc {
	pub front_ccw: bool,
	pub polygon_mode: PolygonMode,
	pub cull_mode: CullMode,
	pub depth_bias: DepthBias,

	pub depth_clip_enable: bool,
	pub multisample_enable: bool,
	pub antialiased_line_enable: bool,
	pub conservative_rasterization_enable: bool,

	pub forced_sample_count: u32,
}

// ----------------------------------------------------------------
// BLEND DESC
// ----------------------------------------------------------------

pub enum BlendFactor {
	/// 0
	Zero,
	/// 1
	One,
	/// src
	SrcColor,
	/// 1 - src
	InvSrcColor,
	/// src.a
	SrcAlpha,
	/// 1 - src.a
	InvSrcAlpha,
	/// dst
	DstColor,
	/// 1 - dst
	InvDstColor,
	/// dst.a
	DstAlpha,
	/// 1 - dst.a
	InvDstAlpha,
	/// src1
	Src1Color,
	/// 1 - src1
	InvSrc1Color,
	/// src1.a
	Src1Alpha,
	/// 1 - src1.a
	InvSrc1Alpha,
	/// min(src.a, 1 - dst.a)
	SrcAlphaSat,
	/// Value from [`CmdListImpl::set_blend_constant`].
	ConstantColor,
	/// 1 - value from [`CmdListImpl::set_blend_constant`].
	InvConstantColor,
}

pub enum BlendOp {
	/// src + dst
	Add,
	/// src - dst
	Subtract,
	/// dst - src
	RevSubtract,
	/// min(src, dst)
	Min,
	/// max(src, dst)
	Max,
}

pub struct BlendDesc {
	pub src_color: BlendFactor,
	pub dst_color: BlendFactor,
	pub color_op: BlendOp,

	pub src_alpha: BlendFactor,
	pub dst_alpha: BlendFactor,
	pub alpha_op: BlendOp,
}

pub struct ColorAttachment {
	pub format: Format,
	pub blend: Option<BlendDesc>,
	pub write_mask: ColorWriteMask,
}

// ----------------------------------------------------------------
// RENDER PIPELINE DESC
// ----------------------------------------------------------------

pub struct GraphicsPipelineDesc<'a> {
	pub vs: Option<&'a [u8]>,
	pub ps: Option<&'a [u8]>,
	
	pub descriptor_layout: DescriptorLayout,
	pub rasterizer: RasterizerDesc,
	pub depth_stencil: DepthStencilDesc,
	pub color_attachments: &'a [ColorAttachment],
	pub topology: Topology,
}

pub struct ComputePipelineDesc<'a> {
	pub cs: &'a [u8],
	pub descriptor_layout: &'a DescriptorLayout,
}

#[derive(Clone, Copy)]
pub enum LoadOp<T> {
	Load,
	Clear(T),
	Discard,
}

pub struct RenderPassDesc<'a, D: DeviceImpl> {
	pub depth_stencil: Option<&'a D::Texture>,
	pub color_attachments: &'a [&'a D::Texture],

	pub color_load: LoadOp<Color<f32>>,
	pub depth_load: LoadOp<f32>,
	pub stencil_load: LoadOp<u8>,
}

bitflags! {
	pub struct StageFlags : u32 {
		
	}

	pub struct AccessFlags : u32 {
	
	}
}

pub struct GlobalBarrier {}

pub struct BufferBarrier<'a, D: DeviceImpl> {
	pub buffer: &'a D::Buffer,
}

pub struct TextureBarrier<'a, D: DeviceImpl> {
	pub texture: &'a D::Texture,
	pub old_layout: TextureLayout,
	pub new_layout: TextureLayout,
}

pub struct Barriers<'a, D: DeviceImpl> {
	pub global: &'a [GlobalBarrier],
	pub buffer: &'a [BufferBarrier<'a, D>],
	pub texture: &'a [TextureBarrier<'a, D>],
}

impl<'a, D: DeviceImpl> Barriers<'a, D> {
	pub fn global() -> Barriers<'static, D> {
		Barriers {
			global: &[GlobalBarrier {}],
			buffer: &[],
			texture: &[],
		}
	}

	pub fn texture(texture: &'a [TextureBarrier<'a, D>]) -> Self {
		Self {
			global: &[],
			buffer: &[],
			texture,
		}
	}
}

pub trait BufferImpl<D: DeviceImpl> {
	fn srv_index(&self) -> Option<u32>;
	fn uav_index(&self) -> Option<u32>;

	fn cpu_ptr(&self) -> *mut u8;
	fn gpu_ptr(&self) -> GpuPtr;
}

pub trait TextureImpl<D: DeviceImpl> {
	fn srv_index(&self) -> Option<u32>;
	fn uav_index(&self) -> Option<u32>;
}

pub trait SamplerImpl<D: DeviceImpl> {}
pub trait GraphicsPipelineImpl<D: DeviceImpl> {}
pub trait ComputePipelineImpl<D: DeviceImpl> {}

pub trait RaytracingPipelineImpl<D: DeviceImpl> {
	fn shader_identifier_size(&self) -> usize;
	fn write_shader_identifier(&self, name: &str, slice: &mut [u8]);
}

pub trait AccelerationStructureImpl<D: DeviceImpl> {
	/// Only valid for top-level acceleration structures.
	fn srv_index(&self) -> Option<u32>;

	fn instance_descriptor_size() -> usize;
	fn write_instance_descriptor(instance: &AccelerationStructureInstance, slice: &mut [u8]);
}

pub trait DeviceImpl: 'static + Send + Sync + Sized {
	type SwapChain: SwapChainImpl<Self>;
	type CmdList: CmdListImpl<Self>;
	type Buffer: BufferImpl<Self>;
	type Texture: TextureImpl<Self>;
	type Sampler: SamplerImpl<Self>;
	type AccelerationStructure: AccelerationStructureImpl<Self>;
	type GraphicsPipeline: GraphicsPipelineImpl<Self>;
	type ComputePipeline: ComputePipelineImpl<Self>;
	type RaytracingPipeline: RaytracingPipelineImpl<Self>;

	fn new(desc: &DeviceDesc) -> Self;

	fn create_swap_chain(&mut self, desc: &SwapChainDesc, window_handle: &NativeHandle) -> Result<Self::SwapChain, Error>;
	fn create_cmd_list(&self, num_buffers: u32) -> Self::CmdList;
	fn create_buffer(&mut self, desc: &BufferDesc) -> Result<Self::Buffer, Error>;
	fn create_texture(&mut self, desc: &TextureDesc) -> Result<Self::Texture, Error>;
	fn create_sampler(&mut self, desc: &SamplerDesc) -> Result<Self::Sampler, Error>;
	fn create_acceleration_structure(&mut self, desc: &AccelerationStructureDesc<Self>) -> Result<Self::AccelerationStructure, Error>;

	fn create_graphics_pipeline(&self, desc: &GraphicsPipelineDesc) -> Result<Self::GraphicsPipeline, Error>;
	fn create_compute_pipeline(&self, desc: &ComputePipelineDesc) -> Result<Self::ComputePipeline, Error>;
	fn create_raytracing_pipeline(&self, desc: &RaytracingPipelineDesc) -> Result<Self::RaytracingPipeline, Error>;

	// TODO: Only supports 2D texture UAVs
	fn create_texture_view(&mut self, desc: &TextureViewDesc, texture: &Self::Texture) -> TextureView;

	fn upload_buffer(&mut self, buffer: &Self::Buffer, data: &[u8]);
	fn upload_texture(&mut self, texture: &Self::Texture, data: &[u8]);

	fn submit(&self, cmd: &Self::CmdList);
	fn adapter_info(&self) -> &AdapterInfo;

	fn acceleration_structure_sizes(&self, desc: &AccelerationStructureBuildInputs) -> AccelerationStructureSizes;
}

pub trait SwapChainImpl<D: DeviceImpl>: 'static + Sized {
	fn update(&mut self, device: &mut D, size: [u32; 2]);
	fn wait_for_last_frame(&mut self);
	fn num_buffers(&self) -> u32;
	fn backbuffer_index(&self) -> u32;
	fn backbuffer_texture(&self) -> &D::Texture;
	fn swap(&mut self, device: &D);
}

pub trait CmdListImpl<D: DeviceImpl> {
	fn reset(&mut self, device: &D, swap_chain: &D::SwapChain);

	fn copy_buffer(
		&self,
		src: &D::Buffer,
		src_offset: u64,
		dst: &D::Buffer,
		dst_offset: u64,
		size: u64,
	);

	fn copy_texture(
		&self,
		src: &D::Texture,
		src_mip_level: u32,
		src_array_slice: u32,
		src_offset: [u32; 3],
		dst: &D::Texture,
		dst_mip_level: u32,
		dst_array_slice: u32,
		dst_offset: [u32; 3],
		size: [u32; 3],
	);

	fn copy_buffer_to_texture(
		&self,
		src: &D::Buffer,
		src_offset: u64,
		src_bytes_per_row: u32,
		dst: &D::Texture,
		dst_mip_level: u32,
		dst_array_slice: u32,
		dst_offset: [u32; 3],
		size: [u32; 3],
	);

	fn copy_texture_to_buffer(
		&self,
		src: &D::Texture,
		src_mip_level: u32,
		src_array_slice: u32,
		src_offset: [u32; 3],
		dst: &D::Buffer,
		dst_offset: u64,
		dst_bytes_per_row: u32,
		size: [u32; 3],
	);

	fn render_pass_begin(&self, desc: &RenderPassDesc<D>);
	fn render_pass_end(&self);
	fn barriers(&self, barriers: &Barriers<D>);
	fn set_viewport(&self, rect: &Rect<f32>, depth: Range<f32>);
	fn set_scissor(&self, rect: &Rect<u32>);
	fn set_blend_constant(&self, color: Color<f32>);
	fn set_stencil_reference(&self, reference: u32);
	fn set_index_buffer(&self, buffer: &D::Buffer, offset: u64, format: Format);
	fn set_graphics_pipeline(&self, pipeline: &D::GraphicsPipeline);
	fn set_compute_pipeline(&self, pipeline: &D::ComputePipeline);
	fn set_raytracing_pipeline(&self, pipeline: &D::RaytracingPipeline);

	fn graphics_push_constants(&self, offset: u32, data: &[u8]);
	fn compute_push_constants(&self, offset: u32, data: &[u8]);

	fn draw(&self, vertices: Range<u32>, instances: Range<u32>);
	/// NOTE: base_vertex and instances.start aren't added to SV_VertexID, pass them manually when needed!
	fn draw_indexed(&self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>);

	fn dispatch(&self, x: u32, y: u32, z: u32);
	fn dispatch_rays(&self, desc: &DispatchRaysDesc);

	fn build_acceleration_structure(&self, desc: &AccelerationStructureBuildDesc<D>);

	fn debug_marker(&self, name: &str, color: Color<u8>);
	fn debug_event_push(&self, name: &str, color: Color<u8>);
	fn debug_event_pop(&self);
}

/// Converts a Sized type to a u8 slice.
pub fn as_u8_slice<T: Sized>(p: &T) -> &[u8] {
	unsafe {
		std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
	}
}

/// Converts a Sized slice to a u8 slice.
pub fn slice_as_u8_slice<T: Sized>(p: &[T]) -> &[u8] {
	unsafe {
		std::slice::from_raw_parts((p.as_ptr() as *const T) as *const u8, std::mem::size_of::<T>() * p.len())
	}
}

/// Aligns value to the alignment specified by align, value must be a power of 2.
pub fn align_pow2(value: u64, align: u64) -> u64 {
	(value + (align - 1)) & !(align - 1)
}

/// Aligns value to the alignment specified by align, value can be non-power of 2.
pub fn align(value: u64, align: u64) -> u64 {
	let div = value / align;
	let rem = value % align;
	if rem != 0 {
		return (div + 1) * align;
	}
	value
}

impl Default for SamplerDesc {
	fn default() -> Self {
		Self {
			address_u: Default::default(),
			address_v: Default::default(),
			address_w: Default::default(),
			filter_min: Default::default(),
			filter_mag: Default::default(),
			filter_mip: Default::default(),
			min_lod: 0.0,
			max_lod: 32.0,
			lod_bias: 0.0,
			compare: None,
			max_anisotropy: 1,
			border_color: None,
		}
	}
}

impl Default for RasterizerDesc {
	fn default() -> Self {
		Self {
			front_ccw: false,
			polygon_mode: PolygonMode::Fill,
			cull_mode: CullMode::None,
			depth_bias: DepthBias {
				constant: 0.0,
				slope: 0.0,
				clamp: 0.0,
			},
			depth_clip_enable: true,
			multisample_enable: false,
			antialiased_line_enable: false,
			conservative_rasterization_enable: false,
			forced_sample_count: 0,
		}
	}
}

impl Default for DepthStencilFaceDesc {
	fn default() -> Self {
		Self {
			func: CompareOp::Never,
			fail: StencilOp::Keep,
			depth_fail: StencilOp::Keep,
			pass: StencilOp::Keep,
		}
	}
}

impl Default for DepthStencilDesc {
	fn default() -> Self {
		Self {
			format: Format::Unknown,
			depth_test_enable: false,
			depth_write_enable: false,
			depth_op: CompareOp::Never,
			stencil_enable: false,
			stencil_read_mask: 0xff,
			stencil_write_mask: 0xff,
			front_face: Default::default(),
			back_face: Default::default(),
		}
	}
}

impl DescriptorBinding {
	pub fn bindless_srv(space: u32) -> Self {
		Self {
			binding_type: DescriptorType::ShaderResource,
			num_descriptors: None,
			shader_register: 0,
			register_space: space,
			offset: Some(0),
		}
	}

	pub fn bindless_uav(space: u32) -> Self {
		Self {
			binding_type: DescriptorType::UnorderedAccess,
			num_descriptors: None,
			shader_register: 0,
			register_space: space,
			offset: Some(0),
		}
	}
}

pub struct FormatInfo {
	pub components: u8,
	pub block_size: u8,
	pub block_dimensions: [u8; 2],
}

impl Format {
	/// Returns the size in bytes of a texel, compressed block of texels or buffer element.
	pub fn block_size(&self) -> u32 {
		self.info().block_size as u32
	}

	/// Returns the row pitch of an image in bytes: width * block size.
	pub fn row_pitch(&self, width: u64) -> u64 {
		self.block_size() as u64 * width
	}

	/// Returns the slice pitch of an image in bytes: width * height * block size.
	/// A slice is a single 2D image or a single slice of a 3D texture or texture array.
	pub fn slice_pitch(&self, width: u64, height: u64) -> u64 {
		self.block_size() as u64 * width * height
	}

	/// Returns the size in bytes of a 3D resource: width * height * depth * block size.
	pub fn size(&self, width: u64, height: u64, depth: u32) -> u64 {
		self.block_size() as u64 * width * height * depth as u64
	}

	pub fn info(&self) -> FormatInfo {
		let (components, block_size, block_dimensions) = match self {
			Self::Unknown => (0, 0, [0, 0]),
		
			Self::R8UNorm  => (1, 1, [1, 1]),
			Self::R8SNorm  => (1, 1, [1, 1]),
			Self::R8UInt   => (1, 1, [1, 1]),
			Self::R8SInt   => (1, 1, [1, 1]),
		
			Self::R16SNorm => (1, 2, [1, 1]),
			Self::R16UNorm => (1, 2, [1, 1]),
			Self::R16UInt  => (1, 2, [1, 1]),
			Self::R16SInt  => (1, 2, [1, 1]),
			Self::R16Float => (1, 2, [1, 1]),
		
			Self::R32UInt  => (1, 4, [1, 1]),
			Self::R32SInt  => (1, 4, [1, 1]),
			Self::R32Float => (1, 4, [1, 1]),
		
			Self::RG8UNorm  => (2, 2, [1, 1]),
			Self::RG8SNorm  => (2, 2, [1, 1]),
			Self::RG8UInt   => (2, 2, [1, 1]),
			Self::RG8SInt   => (2, 2, [1, 1]),

			Self::RG16UNorm => (2, 4, [1, 1]),
			Self::RG16SNorm => (2, 4, [1, 1]),
			Self::RG16UInt  => (2, 4, [1, 1]),
			Self::RG16SInt  => (2, 4, [1, 1]),
			Self::RG16Float => (2, 4, [1, 1]),

			Self::RG32UInt  => (2, 8, [1, 1]),
			Self::RG32SInt  => (2, 8, [1, 1]),
			Self::RG32Float => (2, 8, [1, 1]),

			Self::RGB32UInt  => (3, 12, [1, 1]),
			Self::RGB32SInt  => (3, 12, [1, 1]),
			Self::RGB32Float => (3, 12, [1, 1]),

			Self::RGBA8UNorm  => (4, 4, [1, 1]),
			Self::RGBA8SNorm  => (4, 4, [1, 1]),
			Self::RGBA8UInt   => (4, 4, [1, 1]),
			Self::RGBA8SInt   => (4, 4, [1, 1]),

			Self::RGBA16UNorm => (4, 8, [1, 1]),
			Self::RGBA16SNorm => (4, 8, [1, 1]),
			Self::RGBA16UInt  => (4, 8, [1, 1]),
			Self::RGBA16SInt  => (4, 8, [1, 1]),
			Self::RGBA16Float => (4, 8, [1, 1]),

			Self::RGBA32UInt  => (4, 16, [1, 1]),
			Self::RGBA32SInt  => (4, 16, [1, 1]),
			Self::RGBA32Float => (4, 16, [1, 1]),

			Self::BGRA8UNorm => (4, 16, [1, 1]),

			Self::D16UNorm          => (1, 2, [1, 1]),
			Self::D24UNormS8UInt    => (2, 4, [1, 1]),
			Self::D32Float          => (1, 4, [1, 1]),
			Self::D32FloatS8UIntX24 => (2, 4, [1, 1]),
		};

		FormatInfo {
			components,
			block_size,
			block_dimensions,
		}
	}
}

bitflags! {
	#[derive(Clone, Copy)]
	pub struct AccelerationStructureBuildFlags : u8 {
		const ALLOW_UPDATE      = 1 << 0;
		const ALLOW_COMPACTION  = 1 << 1;
		const PREFER_FAST_TRACE = 1 << 2;
		const PREFER_FAST_BUILD = 1 << 3;
		const MINIMIZE_MEMORY   = 1 << 4;
	}

	#[derive(Clone, Copy)]
	pub struct AccelerationStructureBottomLevelFlags : u8 {
		const OPAQUE                          = 1 << 0;
		const NO_DUPLICATE_ANY_HIT_INVOCATION = 1 << 1;
	}

	#[derive(Clone, Copy)]
	pub struct AccelerationStructureInstanceFlags : u8 {
		const TRIANGLE_CULL_DISABLE = 1 << 0;
		const TRIANGLE_FRONT_CCW    = 1 << 1;
		const FORCE_OPAQUE          = 1 << 2;
		const FORCE_NON_OPAQUE      = 1 << 3;
	}
}

// TODO: Support multiple entry points per library
// Pass shader by reference or id instead of Vec<u8>
pub struct ShaderLibrary {
	pub ty: ShaderType,
	pub entry: String,
	pub shader: Vec<u8>,
}

#[derive(PartialEq)]
pub enum ShaderGroupType {
	General,
	Triangles,
	Procedural,
}

pub struct ShaderGroup {
	pub ty: ShaderGroupType,
	pub name: String,
	pub general: Option<u32>,
	pub closest_hit: Option<u32>,
	pub any_hit: Option<u32>,
	pub intersection: Option<u32>,
}

pub struct RaytracingPipelineDesc {
	pub max_trace_recursion_depth: u32,
	pub max_attribute_size: u32,
	pub max_payload_size: u32,

	pub libraries: Vec<ShaderLibrary>,
	pub groups: Vec<ShaderGroup>,

	pub descriptor_layout: DescriptorLayout,
}

pub struct ShaderTable {
	pub ptr: GpuPtr,
	pub size: usize,
	pub stride: usize,
}

pub struct DispatchRaysDesc {
	pub raygen: Option<ShaderTable>,
	pub miss: Option<ShaderTable>,
	pub hit_group: Option<ShaderTable>,
	pub callable: Option<ShaderTable>,

	pub width: u32,
	pub height: u32,
	pub depth: u32,
}

#[repr(C)]
pub struct AccelerationStructureAABB {
	pub min: [f32; 3],
	pub max: [f32; 3],
}

pub struct AccelerationStructureInstance {
	pub transform: [[f32; 4]; 3], // 3x4 row-major matrix
	pub user_id: u32,
	pub mask: u8,
	pub contribution_to_hit_group_index: u32,
	pub flags: AccelerationStructureInstanceFlags,
	pub bottom_level: GpuPtr, // TODO: Just use an index? MTL: accelerationStructureIndex
}

/// TODO: Metal doesn't support gpu addresses, it requires an MTLBuffer and offset.
/// Maybe change GpuPtr to contain a resource (eg. buffer handle) and offset.
/// This way the GpuPtr is type safe and pointer arithmetic can be implemented by overloading operators.
/// ```
/// pub struct GpuPtr<R: GpuResource> {
/// 	resource: R,
/// 	offset: usize,
/// }
/// ```
pub struct GpuPtr(u64);

impl GpuPtr {
	pub const NULL: Self = Self(0);

	pub fn new(ptr: u64) -> Self {
		Self(ptr)
	}

	pub fn offset(&self, offset: usize) -> Self {
		Self(self.0 + offset as u64)
	}
}

pub struct AccelerationStructureAABBsDesc {
	pub data: GpuPtr,
	pub count: usize,
	pub stride: usize,
}

pub struct AccelerationStructureTrianglesDesc {
	pub vertex_buffer: GpuPtr,
	pub vertex_format: Format,
	pub vertex_count: usize,
	pub vertex_stride: usize,

	pub index_buffer: GpuPtr,
	pub index_format: Format,
	pub index_count: usize,

	pub transform: GpuPtr,
}

pub struct AccelerationStructureInstancesDesc {
	pub data: GpuPtr,
	pub count: usize,
}

pub struct AccelerationStructureSizes {
	pub acceleration_structure_size: usize,
	pub build_scratch_buffer_size: usize,
	pub update_scratch_buffer_size: usize,
}

pub enum GeometryPart {
	AABBs(AccelerationStructureAABBsDesc),
	Triangles(AccelerationStructureTrianglesDesc),
}

pub struct GeometryDesc {
	pub flags: AccelerationStructureBottomLevelFlags,
	pub part: GeometryPart,
}

pub enum AccelerationStructureKind {
	BottomLevel,
	TopLevel,
}

pub struct AccelerationStructureBuildInputs {
	pub kind: AccelerationStructureKind,
	pub flags: AccelerationStructureBuildFlags,

	pub instances: AccelerationStructureInstancesDesc,
	pub geometry: Vec<GeometryDesc>,
}

pub struct AccelerationStructureDesc<'a, D: DeviceImpl> {
	pub kind: AccelerationStructureKind,
	// TODO: In Metal we can't specify the buffer (accel struct has internal buffer)
	// They have makeAccelerationStructure(size:) and makeAccelerationStructure(descriptor:)
	// The backend should allocate this buffer behind the scenes
	pub buffer: &'a D::Buffer,
	pub offset: usize,
}

pub struct AccelerationStructureBuildDesc<'a, D: DeviceImpl> {
	pub inputs: &'a AccelerationStructureBuildInputs,

	pub dst: &'a D::AccelerationStructure,
	pub src: Option<&'a D::AccelerationStructure>,

	pub scratch_data: GpuPtr,
}
