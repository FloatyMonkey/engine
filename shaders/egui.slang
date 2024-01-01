cbuffer ConstantBuffer : register(b0) {
	float2 screen_size;
	uint vb_index;
	uint texture_index;
};

ByteAddressBuffer buffers[] : register(t0, space1);
Texture2D<float4> textures_2d_float4[] : register(t0, space2);

SamplerState linear_sampler : register(s0);

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
};

struct PsInput {
	float4 position : SV_POSITION;
	float4 color    : COLOR0;
	float2 uv       : TEXCOORD0;
};

[shader("vertex")]
PsInput main_vs(VsInput input) {
	let vertex_buffer = buffers[vb_index];
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
	let texture = textures_2d_float4[texture_index];

	return input.color * texture.Sample(linear_sampler, input.uv);
}
