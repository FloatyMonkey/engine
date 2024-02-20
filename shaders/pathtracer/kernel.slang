import bxdf;
import light;
import sample_generator;
import onb;
import math;
import sampling;
import spectrum;

import bxdfs.diffuse;
import bxdfs.conductor;
import lights.dome_light;
import lights.rect_light;
import lights.sphere_light;

ByteAddressBuffer buffers[] : register(t0, space1);
RaytracingAccelerationStructure acceleration_structures[] : register(t0, space2);
Texture2D<float4> textures_2d_float4[] : register(t0, space3);
Texture2D<float> textures_2d_float[] : register(t0, space4);
RWTexture2D<float4> rw_textures_2d_float4[] : register(u0, space5);

SamplerState linear_sampler : register(s0, space0);

static const uint INVALID_ID = 0xffffffff;

enum LightType {
	Dome = 0,
	Rect = 1,
	Sphere = 2,
}

static const uint LIGHT_SIZE = 64; // Size of cpu side `GpuLight` union

struct GpuCamera {
	float3 u;
	float scale_u;
	float3 v;
	float scale_v;
	float3 w;
	float scale_w;
	float3 position;
	float aperture_radius;
}

cbuffer ConstantBuffer : register(b0, space0) {
	GpuCamera camera;
	uint tlas_index;
	uint combined_index;
	uint instance_data_index;
	uint light_data_index;
	uint light_count;
	uint infinite_light_count;
	uint seed;
	float accumulation_factor;
}

struct Vertex {
	float3 position;
	float3 normal;
}

struct Instance {
	uint vertex_buffer_id;
	uint index_buffer_id;
	uint material_offset;
}

struct RayPayload {
	SampledSpectrum radiance;
	SampledSpectrum throughput;
	float pdf;
	uint depth;
	float3 origin;
	float3 direction;
	PCG32si sg;
}

func power_heuristic(f_n: float, f_pdf: float, g_n: float, g_pdf: float) -> float {
	let f = f_n * f_pdf;
	let g = g_n * g_pdf;
	return (f * f) / (f * f + g * g);
}

/// Traces a shadow ray and returns true if there was an intersection.
func occluded(tlas: RaytracingAccelerationStructure, ray: RayDesc) -> bool {
	RayQuery<RAY_FLAG_CULL_NON_OPAQUE | RAY_FLAG_SKIP_PROCEDURAL_PRIMITIVES | RAY_FLAG_ACCEPT_FIRST_HIT_AND_END_SEARCH> q;
	q.TraceRayInline(tlas, RAY_FLAG_NONE, 0xff, ray);
	q.Proceed();
	return q.CommittedStatus() == COMMITTED_TRIANGLE_HIT;
}

struct Ray {
	float3 origin;
	float3 direction;
}

func lens(pixel: uint2, resolution: uint2, inout sg: PCG32si) -> Ray {
	let p = (pixel + sample_next_2d(sg)) / resolution;
	let uv = float2(p.x * 2.0 - 1.0, -p.y * 2.0 + 1.0);

	let focus_dir = uv.x * camera.u * camera.scale_u + uv.y * camera.v * camera.scale_v + camera.w * camera.scale_w;
	let u = sample_uniform_disk_concentric(sample_next_2d(sg));
	let coc = camera.aperture_radius * (u.x * camera.u + u.y * camera.v);

	let origin = camera.position + coc;
	let direction = normalize(focus_dir - coc);

	return { origin, direction };
}

[shader("raygeneration")]
func raygen() {
	let tlas = acceleration_structures[tlas_index];
	let combined = rw_textures_2d_float4[combined_index];

	let rand_seed = DispatchRaysIndex().x + DispatchRaysDimensions().x * DispatchRaysIndex().y;

	const int BOUNCES = 4;

	var sg = PCG32si(rand_seed ^ seed);
	
	let camera_ray = lens(DispatchRaysIndex().xy, DispatchRaysDimensions().xy, sg);

	var payload: RayPayload;
	payload.radiance = 0.0;
	payload.throughput = 1.0;
	payload.pdf = 1.0;
	payload.depth = 0;
	payload.origin = camera_ray.origin;
	payload.direction = camera_ray.direction;
	payload.sg = sg;

	for (int bounce = 0; bounce < BOUNCES; bounce++) {
		let ray: RayDesc = { payload.origin, 0.0, payload.direction, 10000.0 };
		TraceRay(tlas, RAY_FLAG_CULL_BACK_FACING_TRIANGLES, 0xff, 0, 0, 0, ray, payload);

		if (russian_roulette(payload.throughput, sample_next_1d(payload.sg))) {
			break;
		}

		if (!any(payload.throughput)) {
			break;
		}
	}

	let color = lerp(combined[DispatchRaysIndex().xy].xyz, payload.radiance, accumulation_factor);
	combined[DispatchRaysIndex().xy] = float4(color, 1.0);
}

struct ShadingCtx {
	float3 position;
	float3 shading_point;
	float3 normal;
	float3 geometric_normal;

	static func init(attributes: BuiltInTriangleIntersectionAttributes) -> ShadingCtx {
		// Load mesh data
		let instance_data_buffer = buffers[instance_data_index];
		let instance_data = instance_data_buffer.Load<Instance>(InstanceIndex() * sizeof(Instance));

		let vertex_buffer = buffers[NonUniformResourceIndex(instance_data.vertex_buffer_id)];
		let index_buffer = buffers[NonUniformResourceIndex(instance_data.index_buffer_id)];

		let indices = index_buffer.Load<uint3>(PrimitiveIndex() * sizeof(uint3));

		let v0 = vertex_buffer.Load<Vertex>(indices[0] * sizeof(Vertex));
		let v1 = vertex_buffer.Load<Vertex>(indices[1] * sizeof(Vertex));
		let v2 = vertex_buffer.Load<Vertex>(indices[2] * sizeof(Vertex));

		let barycentrics = float3(1.0 - attributes.barycentrics.x - attributes.barycentrics.y, attributes.barycentrics.x, attributes.barycentrics.y);

		// Interpolate vertex data
		let local_position = barycentrics.x * v0.position + barycentrics.y * v1.position + barycentrics.z * v2.position;
		let local_normal = normalize(barycentrics.x * v0.normal + barycentrics.y * v1.normal + barycentrics.z * v2.normal);
		let local_geometric_normal = normalize(cross(v1.position - v0.position, v2.position - v0.position));

		var shading_point = offset_shading_point(local_position, { v0.position, v1.position, v2.position }, { v0.normal, v1.normal, v2.normal }, barycentrics);
		shading_point = mul(ObjectToWorld3x4(), float4(shading_point, 1.0));

		// Transform to world space
		let position = mul(ObjectToWorld3x4(), float4(local_position, 1.0));
		let normal = normalize(mul((float3x3)ObjectToWorld3x4(), local_normal));
		let geometric_normal = normalize(mul((float3x3)ObjectToWorld3x4(), local_geometric_normal));

		return { position, shading_point, normal, geometric_normal };
	}
}

func light_pmf(light_index: uint) -> float {
	return 1.0 / light_count;
}

func sample_light<S: SampleGenerator>(position: float3, inout sg: S, out light_pmf: float) -> LightLiSample {
	let light_index = min(uint(sample_next_1d(sg) * light_count), light_count - 1);
	let sampled_light_pmf = 1.0 / light_count;

	let light_data_buffer = buffers[light_data_index];
	let light_type = LightType(light_data_buffer.Load<uint>(light_index * LIGHT_SIZE));

	var ls: LightLiSample;

	switch (light_type) {
		case LightType.Dome: {
			let light_data = light_data_buffer.Load<DomeLightData>(light_index * LIGHT_SIZE + 4);

			let env_map = textures_2d_float4[light_data.env_map_id];
			let importance_map = textures_2d_float[light_data.importance_map_id];
			let light: DomeLight = { env_map, linear_sampler, { importance_map, light_data.base_mip } };

			ls = light.sample_li(position, sg);
		} break;

		case LightType.Rect: {
			let light = light_data_buffer.Load<RectLight>(light_index * LIGHT_SIZE + 4);
			ls = light.sample_li(position, sg);
		} break;

		case LightType.Sphere: {
			let light = light_data_buffer.Load<SphereLight>(light_index * LIGHT_SIZE + 4);
			ls = light.sample_li(position, sg);
		} break;
	}

	light_pmf = sampled_light_pmf;
	return ls;
}

[shader("closesthit")]
func closesthit(inout RayPayload payload, attributes: BuiltInTriangleIntersectionAttributes) {
	payload.depth++;

	let tlas = acceleration_structures[tlas_index];

	let ctx = ShadingCtx.init(attributes);

	var bxdf: DiffuseBxdf;
	bxdf.albedo = SampledSpectrum(0.74);
	/*var bxdf: ConductorBxdf;
	bxdf.eta = SampledSpectrum(0.1803, 0.38405, 1.3661);
	bxdf.k = SampledSpectrum(3.4671, 2.4245, 1.7681);
	bxdf.distribution.alpha = 0.5 * 0.5;*/

	let onb = orthonormal_basis(ctx.normal);
	let bsdf: Bsdf<DiffuseBxdf> = { bxdf, onb };

	let wo = -payload.direction;

	// Sample light
	if (light_count > 0) {
		var sampled_light_pmf: float;
		let ls = sample_light(ctx.position, payload.sg, sampled_light_pmf);

		if (ls.pdf > 0.0) {
			let f = bsdf.f(wo, ls.wi) * abs(dot(ls.wi, ctx.normal));

			let light_ray: RayDesc = { ctx.shading_point, 1e-4, ls.wi, ls.distance - 1e-4 };

			if (!occluded(tlas, light_ray)) {
				let p_l = sampled_light_pmf * ls.pdf;
				let p_b = bsdf.pdf(wo, ls.wi);

				let w_l = power_heuristic(1.0, p_l, 1.0, p_b);

				payload.radiance += payload.throughput * (w_l * ls.li * f / p_l);
			}
		}
	}

	// Sample bsdf
	{
		let sampled_f = bsdf.sample_f(wo, payload.sg);

		if (sampled_f.pdf < 1e-5) {
			payload.throughput = 0.0;
			return;
		}

		payload.throughput *= sampled_f.f * abs(dot(sampled_f.wi, ctx.normal)) / sampled_f.pdf;
		payload.pdf = sampled_f.pdf;

		payload.origin = offset_ray_origin(ctx.position, ctx.geometric_normal);
		payload.direction = sampled_f.wi;
	}
}

[shader("miss")]
func miss(inout RayPayload payload) {
	for (uint i = 0; i < infinite_light_count; i++) {
		let light_data_buffer = buffers[light_data_index];
		let light_type = LightType(light_data_buffer.Load<uint>(i * LIGHT_SIZE));

		if (light_type != LightType.Dome) {
			continue;
		}

		let light_data = light_data_buffer.Load<DomeLightData>(i * LIGHT_SIZE + 4);

		let env_map = textures_2d_float4[light_data.env_map_id];
		let importance_map = textures_2d_float[light_data.importance_map_id];
		let light: DomeLight = { env_map, linear_sampler, { importance_map, light_data.base_mip } };

		var w_b: float = 1.0;
		if (payload.depth != 0) { // TODO: && !specular bounce
			let p_b = payload.pdf;
			let p_l = light_pmf(i) * light.pdf_li(0.0, payload.direction); // TODO: Pass correct position instead of 0.0
			w_b = power_heuristic(1.0, p_b, 1.0, p_l);
		}

		let le = light.le(payload.direction);

		payload.radiance += payload.throughput * le * w_b;
	}

	payload.throughput = 0.0;
}

/// Returns true if a path should terminate early using russian roulette.
func russian_roulette(inout throughput: SampledSpectrum, u: float) -> bool {
	let rr_throughput = max(throughput.r, max(throughput.g, throughput.b));
	let q = max(0.0, 1.0 - rr_throughput);

	if (u < q) {
		throughput = SampledSpectrum(0.0);
		return true;
	}

	throughput /= 1.0 - q;
	return false;
}

/// Offsets ray origin `p` along geometric normal `ng` to avoid self-intersections caused by numerical precision issues.
/// [Wächter & Binder 2019, *A Fast and Robust Method for Avoiding Self-Intersection*]
func offset_ray_origin(p: float3, ng: float3) -> float3 {
	const float ORIGIN = 1.0 / 32.0;
	const float FLOAT_SCALE = 3.0 / 65536.0;
	const float INT_SCALE = 256.0;

	int3 i_offset = int3(ng * INT_SCALE);
	float3 i_p = asfloat(asint(p) + select(p < 0.0, -i_offset, i_offset));

	float3 f_offset = ng * FLOAT_SCALE;
	return select(abs(p) < ORIGIN, p + f_offset, i_p);
}

/// Offsets shading point `p` to avoid hard self-shadowing caused by low tessellation.
/// [Hanika 2021, *Hacking the Shadow Terminator*]
func offset_shading_point(p: float3, positions: float3[3], normals: float3[3], barycentrics: float3) -> float3 {
	var tmpu = p - positions[0];
	var tmpv = p - positions[1];
	var tmpw = p - positions[2];

	// TODO: Transmitted rays may need to apply the offset the other way around, i.e. flip all the vertex normals before evaluation.
	tmpu -= min(0.0, dot(tmpu, normals[0])) * normals[0];
	tmpv -= min(0.0, dot(tmpv, normals[1])) * normals[1];
	tmpw -= min(0.0, dot(tmpw, normals[2])) * normals[2];

	return p + barycentrics.x * tmpu + barycentrics.y * tmpv + barycentrics.z * tmpw;
}
