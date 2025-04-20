import view_transform;

[[vk::push_constant]] cbuffer ConstantBuffer : register(b0) {
	uint input_id;
	uint output_id;
	uint2 output_res;
	uint overlay_id;
}

[[vk::binding(0, 1)]] Texture2D<float4> textures_2d_float4[] : register(t0, space1);
[[vk::binding(0, 2)]] RWTexture2D<float4> rw_textures_2d_float4[] : register(u0, space2);

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 id : SV_DispatchThreadID) {
	let pixel = id.xy;
	if (any(pixel >= output_res)) return;

	let input = textures_2d_float4[input_id];
	let overlay = textures_2d_float4[overlay_id];
	let output = rw_textures_2d_float4[output_id];

	var color = input[pixel].xyz;
	color = agx(color);
	//color = agx_golden(color);
	//color = agx_punchy(color);
	color = agx_eotf(color);

	let alpha = overlay[pixel].w;
	output[pixel] = float4(overlay[pixel].xyz * alpha + color * (1.0 - alpha), 1.0);
}
