import math;

cbuffer ConstantBuffer : register(b0) {
	uint output_id;
	uint2 output_res;
	uint input_id;
	uint input_mip;
}

Texture2D<float> textures_2d_float[] : register(t0, space1);
RWTexture2D<float> rw_textures_2d_float[] : register(u0, space2);

SamplerState linear_sampler : register(s0, space0);

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 id : SV_DispatchThreadID) {
	let pixel = id.xy;
	if (any(pixel >= output_res)) return;

	let input = textures_2d_float[input_id];
	let output = rw_textures_2d_float[output_id];

	let uv = ((float2)pixel + 0.5) / (float2)output_res;
	let color = input.SampleLevel(linear_sampler, uv, input_mip);
	output[pixel] = color;
}
