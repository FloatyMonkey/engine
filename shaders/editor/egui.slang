
[[vk::push_constant]] cbuffer ConstantBuffer {
	ByteAddressBuffer.Handle vertex_buffer;
	Texture2D<float4>.Handle texture;
	SamplerState.Handle linear_sampler;
	float2 screen_size;
}

func unpack_float4(u: uint) -> float4 {
	float4 f;

	f.a = (float)((u >> 24) & 0xff) / 255.0;
	f.b = (float)((u >> 16) & 0xff) / 255.0;
	f.g = (float)((u >> 8 ) & 0xff) / 255.0;
	f.r = (float)((u >> 0 ) & 0xff) / 255.0;

	return f;
}

struct Vertex {
	float2 position;
	float2 uv;
	uint color;
}

struct VsInput {
	uint vertex_id : SV_VertexID;
}

struct PsInput {
	float4 position : SV_POSITION;
	float4 color    : COLOR0;
	float2 uv       : TEXCOORD0;
}

[shader("vertex")]
PsInput main_vs(VsInput input) {
	let vertex = vertex_buffer.Load<Vertex>(input.vertex_id * sizeof(Vertex));

	var output: PsInput;
	output.position = float4(
		2.0 * vertex.position.x / screen_size.x - 1.0,
		1.0 - 2.0 * vertex.position.y / screen_size.y,
		0.0,
		1.0
	);
	output.color = unpack_float4(vertex.color);
	output.uv = vertex.uv;

	return output;
}

[shader("pixel")]
float4 main_ps(PsInput input) : SV_Target {
	return input.color * texture.Sample(linear_sampler, input.uv);
}
