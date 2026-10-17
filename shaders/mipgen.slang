import math;

[[vk::push_constant]] cbuffer ConstantBuffer {
	Texture2D<float>.Handle input;
	RWTexture2D<float>.Handle output;
	SamplerState.Handle linear_sampler;
	uint2 output_res;
	uint input_mip;
}

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 id : SV_DispatchThreadID) {
	let pixel = id.xy;
	if (any(pixel >= output_res)) return;

	let uv = ((float2)pixel + 0.5) / (float2)output_res;
	let color = input.SampleLevel(linear_sampler, uv, input_mip);
	output[pixel] = color;
}
