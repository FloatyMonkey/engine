import math;

cbuffer ConstantBuffer : register(b0) {
	uint env_map_id;
	uint importance_map_id;

	uint2 output_res;
	uint2 output_res_in_samples;
	uint2 num_samples;
	float inv_samples;
}

Texture2D<float4> textures_2d_float4[] : register(t0, space1);
RWTexture2D<float> rw_textures_2d_float[] : register(u0, space2);

SamplerState linear_sampler : register(s0, space0);

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 id : SV_DispatchThreadID) {
	let pixel = id.xy;
	if (any(pixel >= output_res)) return;

	let env_map = textures_2d_float4[env_map_id];
	let importance_map = rw_textures_2d_float[importance_map_id];

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
