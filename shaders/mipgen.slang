import bindless;
import math;

[[vk::push_constant]] cbuffer ConstantBuffer : register(b0) {
	uint2 output_res;
	uint output_id;
	uint input_id;
	uint input_mip;
	uint sampler_index;
}

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 id : SV_DispatchThreadID) {
	let pixel = id.xy;
	if (any(pixel >= output_res)) return;

	let input = get_texture2d_float(input_id);
	let output = get_rw_texture2d_float(output_id);
	let linear_sampler = get_sampler(sampler_index);

	let uv = ((float2)pixel + 0.5) / (float2)output_res;
	let color = input.SampleLevel(linear_sampler, uv, input_mip);
	output[pixel] = color;
}
