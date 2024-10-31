use crate::asset::{Asset, UntypedAssetId, AssetId, AssetServer};
use crate::ecs::World;
use crate::gpu::{self, BufferImpl, CmdListImpl, DeviceImpl, TextureImpl, AccelerationStructureImpl};
use crate::math::transform::Transform3;
use crate::math::{Vec3, Mat3x4, Mat4};
use crate::scene::{DomeLight, RectLight, SphereLight, Renderable};
use super::acceleration_structure::{Blas, Tlas};
use super::camera::Camera;
use super::env_map::ImportanceMap;
use crate::geometry::{mesh, io};

const MAX_LIGHTS: usize = 100;
const MAX_INSTANCES: usize = 1000;

#[repr(C)]
pub struct Vertex {
	position: Vec3,
	normal: Vec3,
}

#[repr(C)]
struct Instance {
	vertex_buffer_id: u32,
	index_buffer_id: u32,
	material_offset: u32,
}

#[repr(C)]
enum GpuLightType {
	Dome = 0,
	Rect = 1,
	Sphere = 2,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct GpuDomeLight {
	ty: u32,
	env_map_id: u32,
	importance_map_id: u32,
	base_mip: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct GpuSphereLight {
	ty: u32,
	emission: Vec3,
	position: Vec3,
	radius: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct GpuRectLight {
	ty: u32,
	emission: Vec3,
	position: Vec3,
	area_scaled_normal: Vec3,
	x: Vec3,
	y: Vec3,
}

#[repr(C)]
union GpuLight {
	dome: GpuDomeLight,
	rect: GpuRectLight,
	sphere: GpuSphereLight,
}

#[derive(Clone, Debug)]
pub struct Image {
	pub width: u32,
	pub height: u32,
	pub data: Vec<[f32; 4]>,
}

impl Image {
	pub fn new(width: u32, height: u32, data: Vec<[f32; 4]>) -> Self {
		assert_eq!((width * height) as usize, data.len());
		Self { width, height, data }
	}
}

struct GpuMeshData {
	vertex_buffer: gpu::Buffer,
	index_buffer: gpu::Buffer,
	blas: Blas,
}

impl GpuMeshData {
	fn from_mesh(device: &mut gpu::Device, mesh: &mesh::Mesh) -> Self {
		let vertices: Vec<Vertex> = mesh.vertices.iter().map(|v| Vertex { position: v.p, normal: v.n }).collect();
		let indices: Vec<u32> = mesh.indices.iter().map(|i| *i as u32).collect();

		let vertex_buffer = device.create_buffer(&gpu::BufferDesc {
			size: size_of::<Vertex>() * vertices.len(),
			usage: gpu::BufferUsage::SHADER_RESOURCE,
			memory: gpu::Memory::GpuOnly,
		}).unwrap();

		let index_buffer = device.create_buffer(&gpu::BufferDesc {
			size: size_of::<u32>() * indices.len(),
			usage: gpu::BufferUsage::SHADER_RESOURCE,
			memory: gpu::Memory::GpuOnly,
		}).unwrap();

		gpu::upload_buffer(device, &vertex_buffer, gpu::slice_as_u8_slice(&vertices));
		gpu::upload_buffer(device, &index_buffer, gpu::slice_as_u8_slice(&indices));

		let blas = Blas::create(device, &vertex_buffer, &index_buffer, vertices.len(), indices.len(), size_of::<Vertex>());

		Self {
			vertex_buffer,
			index_buffer,
			blas,
		}
	}
}

pub struct Scene {
	pub tlas: Tlas,
	pub camera: Camera,
	pub camera_transform: Transform3,
	pub instance_data_buffer: gpu::Buffer,
	pub light_data_buffer: gpu::Buffer,
	pub light_count: usize,
	pub infinite_light_count: usize,
	pub importance_map: ImportanceMap,

	texture_cache: std::collections::HashMap<UntypedAssetId, gpu::Texture>,
	mesh_cache: std::collections::HashMap<UntypedAssetId, GpuMeshData>,
}

impl Scene {
	pub fn new(device: &mut gpu::Device, shader_compiler: &gpu::ShaderCompiler) -> Self {
		let tlas = Tlas::create(device, MAX_INSTANCES);

		let instance_data_buffer = device.create_buffer(&gpu::BufferDesc {
			size: size_of::<Instance>() * MAX_INSTANCES,
			usage: gpu::BufferUsage::SHADER_RESOURCE,
			memory: gpu::Memory::CpuToGpu,
		}).unwrap();

		let light_data_buffer = device.create_buffer(&gpu::BufferDesc {
			size: size_of::<GpuLight>() * MAX_LIGHTS,
			usage: gpu::BufferUsage::SHADER_RESOURCE,
			memory: gpu::Memory::CpuToGpu,
		}).unwrap();

		let importance_map = ImportanceMap::setup(device, shader_compiler);

		Self {
			tlas,
			camera: Camera::default(),
			camera_transform: Transform3::IDENTITY,
			instance_data_buffer,
			light_data_buffer,
			light_count: 0,
			infinite_light_count: 0,
			importance_map,
			texture_cache: std::collections::HashMap::new(),
			mesh_cache: std::collections::HashMap::new(),
		}
	}

	pub fn update(&mut self, world: &mut World, assets: &AssetServer, device: &mut gpu::Device, cmd: &mut gpu::CmdList) {
		// CAMERA
		// TODO: Handle properly when there's no camera in the scene.
		if let Some((transform, camera)) = world.query::<(&Transform3, &Camera)>().iter().next() {
			self.camera = *camera;
			self.camera_transform = *transform;
		}

		// LIGHTS

		// Gpu assumes infinite lights are at the start of the array
		// light_count contains the number of all lights
		// infinite_light_count only contains the number of infinite lights

		let lights = unsafe { std::slice::from_raw_parts_mut(self.light_data_buffer.cpu_ptr() as *mut GpuLight, MAX_LIGHTS) };
		let mut light_index = 0;
		let mut infinite_light_count = 0;

		for light in &world.query::<&DomeLight>() {
			let env_map_srv_index = self.get_texture_from_cache(&light.image, device, assets).srv_index().unwrap();

			// TODO: Currently only supports single dome light
			self.importance_map.update(cmd, env_map_srv_index);

			lights[light_index].dome = GpuDomeLight {
				ty: GpuLightType::Dome as _,
				env_map_id: env_map_srv_index,
				importance_map_id: self.importance_map.importance_map.srv_index().unwrap(),
				base_mip: self.importance_map.base_mip(),
			};

			light_index += 1;
			infinite_light_count += 1;
		}

		for (transform, light) in &world.query::<(&Transform3, &RectLight)>() {
			let x = transform.rotation *  Vec3::X * transform.scale.x * light.width;
			let y = transform.rotation *  Vec3::Y * transform.scale.y * light.height;
			let z = transform.rotation * -Vec3::Z * transform.scale.z.signum();

			let area = light.width * light.height;

			lights[light_index].rect = GpuRectLight {
				ty: GpuLightType::Rect as _,
				emission: Vec3::new(light.emission[0], light.emission[1], light.emission[2]),
				position: transform.translation,
				area_scaled_normal: z * area,
				x,
				y,
			};

			light_index += 1;
		}

		for (transform, light) in &world.query::<(&Transform3, &SphereLight)>() {
			lights[light_index].sphere = GpuSphereLight {
				ty: GpuLightType::Sphere as _,
				emission: Vec3::new(light.emission[0], light.emission[1], light.emission[2]),
				position: transform.translation,
				radius: light.radius,
			};

			light_index += 1;
		}

		self.light_count = light_index;
		self.infinite_light_count = infinite_light_count;

		// INSTANCES

		let instance_descriptor_size = gpu::AccelerationStructure::instance_descriptor_size();

		let instance_data = unsafe { std::slice::from_raw_parts_mut(self.instance_data_buffer.cpu_ptr() as *mut Instance, MAX_INSTANCES) };
		let instance_descriptors = unsafe { std::slice::from_raw_parts_mut(self.tlas.instance_buffer.cpu_ptr(), MAX_INSTANCES * instance_descriptor_size) };

		let mut instance_index = 0;

		for (transform, renderable) in &world.query::<(&Transform3, &Renderable)>() {
			let mesh_data = self.get_mesh_from_cache(&renderable.mesh, device, cmd, assets);

			instance_data[instance_index] = Instance {
				vertex_buffer_id: mesh_data.vertex_buffer.srv_index().unwrap(),
				index_buffer_id: mesh_data.index_buffer.srv_index().unwrap(),
				material_offset: 0,
			};

			let instance_desc = gpu::AccelerationStructureInstance {
				transform: Mat3x4::from(Mat4::from(*transform)).data,
				user_id: 0,
				mask: 0xff,
				contribution_to_hit_group_index: 0,
				flags: gpu::AccelerationStructureInstanceFlags::empty(),
				bottom_level: mesh_data.blas.accel.gpu_ptr(),
			};

			gpu::AccelerationStructure::write_instance_descriptor(&instance_desc, &mut instance_descriptors[instance_index * instance_descriptor_size..]);

			instance_index += 1;
		}

		self.tlas.build_inputs.entries = gpu::AccelerationStructureEntries::Instances(
			gpu::AccelerationStructureInstances { data: self.tlas.instance_buffer.gpu_ptr(), count: instance_index }
		);

		// ACCELERATION STRUCTURES

		cmd.barriers(&gpu::Barriers::global()); // Global barrier to ensure the BLASes are visible to TLAS
		self.tlas.build(cmd);
		cmd.barriers(&gpu::Barriers::global()); // Global barrier to ensure the TLAS is visible to the raytracing pipeline
	}

	fn get_texture_from_cache(&mut self, asset: &AssetId<Image>, device: &mut gpu::Device, assets: &AssetServer) -> &gpu::Texture {
		self.texture_cache.entry(asset.id()).or_insert_with(|| {
			let image = assets.get(asset).unwrap();

			let texture_desc = gpu::TextureDesc {
				width: image.width as u64,
				height: image.height as u64,
				depth: 1,
				array_size: 1,
				mip_levels: 1,
				format: gpu::Format::RGBA32Float,
				usage: gpu::TextureUsage::SHADER_RESOURCE,
				layout: gpu::TextureLayout::ShaderResource,
			};

			let texture = device.create_texture(&texture_desc).unwrap();
			gpu::upload_texture(device, &texture, &texture_desc, gpu::slice_as_u8_slice(&image.data));

			texture
		})
	}

	fn get_mesh_from_cache(&mut self, asset: &AssetId<mesh::Mesh>, device: &mut gpu::Device, cmd: &gpu::CmdList, assets: &AssetServer) -> &GpuMeshData {
		self.mesh_cache.entry(asset.id()).or_insert_with(|| {
			let mesh = assets.get(asset).unwrap();
			let mut gpu_mesh_data = GpuMeshData::from_mesh(device, mesh);
			gpu_mesh_data.blas.build(cmd);
			gpu_mesh_data
		})
	}
}

impl Asset for mesh::Mesh {
	fn load(path: impl AsRef<std::path::Path>) -> Self
		where
			Self: Sized {
		io::load_mesh(path.as_ref().to_str().unwrap())
	}
}

impl Asset for Image {
	fn load(path: impl AsRef<std::path::Path>) -> Self
		where
			Self: Sized {
		exr::prelude::read_first_rgba_layer_from_file(
				path,
				|resolution, _| Image::new(
					resolution.width() as u32,
					resolution.height() as u32,
					vec![[0.0, 0.0, 0.0, 0.0]; resolution.width() * resolution.height()],
				),
				|image, position, (r, g, b, a): (f32, f32, f32, f32)| {
					image.data[image.width as usize * position.y() + position.x()] = [r, g, b, a];
				},
			).unwrap().layer_data.channel_data.pixels
	}
}
