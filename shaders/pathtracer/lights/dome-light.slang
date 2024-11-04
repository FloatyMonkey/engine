implementing light;

import math;
import pathtracer.importance_map;
import sample_generator;

public struct DomeLightData {
	public uint env_map_id;
	public uint importance_map_id;
	public uint base_mip;
}

public struct DomeLight: Light {
	public Texture2D<float4> env_map;
	public SamplerState linear_sampler;
	public HierarchicalImportanceMap importance_map;

	public func le(dir: float3, lod: float = 0.0) -> float3 {
		let uv = ndir_to_equirect_unorm(dir);
		return env_map.SampleLevel(linear_sampler, uv, lod).rgb;
	}

	public func sample_li<S: SampleGenerator>(p: float3, inout sg: S) -> LightLiSample {
		var pdf: float;
		let uv = importance_map.sample(sample_next_2d(sg), pdf);

		let wi = oct_to_ndir_equal_area_unorm(uv);

		var result: LightLiSample;
		result.li = le(wi);
		result.wi = wi;
		result.distance = float::maxValue;
		result.pdf = pdf / (4.0 * PI);

		return result;
	}

	public func pdf_li(p: float3, wi: float3) -> float {
		let uv = ndir_to_oct_equal_area_unorm(wi);
		return importance_map.pdf(uv) / (4.0 * PI);
	}
}
