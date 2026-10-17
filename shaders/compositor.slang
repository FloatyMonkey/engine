import view_transform;

[[vk::push_constant]] cbuffer ConstantBuffer {
	Texture2D<float4>.Handle input;
	Texture2D<float4>.Handle overlay;
	RWTexture2D<float4>.Handle output;
	uint2 output_res;
}

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 id : SV_DispatchThreadID) {
	let pixel = id.xy;
	if (any(pixel >= output_res)) return;

	var color = input[pixel].xyz;
	color = agx(color);
	//color = agx_golden(color);
	//color = agx_punchy(color);
	color = agx_eotf(color);

	let alpha = overlay[pixel].w;
	output[pixel] = float4(overlay[pixel].xyz * alpha + color * (1.0 - alpha), 1.0);
}
