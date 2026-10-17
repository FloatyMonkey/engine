import math;

[[vk::push_constant]] cbuffer ConstantBuffer {
	Texture2D<float4>.Handle env_map;
	RWTexture2D<float>.Handle importance_map;
	SamplerState.Handle linear_sampler;

	uint2 output_res;
	uint2 output_res_in_samples;
	uint2 num_samples;
	float inv_samples;
}

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 id : SV_DispatchThreadID) {
	let pixel = id.xy;
	if (any(pixel >= output_res)) return;

	var L = 0.0;
	for (uint y = 0; y < num_samples.y; y++) {
		for (uint x = 0; x < num_samples.x; x++) {
			let sample_pos = pixel * num_samples + uint2(x, y);
			let p = ((float2)sample_pos + 0.5) / output_res_in_samples;

			let dir = oct_to_ndir_equal_area_unorm(p);
			let uv = ndir_to_equirect_unorm(dir);

			let radiance = env_map.SampleLevel(linear_sampler, uv, 0).rgb;
			L += luminance(radiance);
		}
	}

	importance_map[pixel] = L * inv_samples;
}
