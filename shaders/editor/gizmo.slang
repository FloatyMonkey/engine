import util;

[[vk::push_constant]] cbuffer ConstantBuffer : register(b0) {
	float4x4 view_projection;
	float2 screen_size;
	uint vb_index;
	uint depth_texture_id;
}

[[vk::binding(0, 0)]] ByteAddressBuffer buffers[] : register(t0, space1);
[[vk::binding(0, 1)]] Texture2D<float> textures_2d_float[] : register(t0, space2);

[[vk::binding(0, 4)]] SamplerState linear_sampler : register(s0, space0);

static const float ANTIALIASING = 1.0;

static const float2[4] POSITIONS = {
	float2(-1.0, -1.0),
	float2( 1.0, -1.0),
	float2(-1.0,  1.0),
	float2( 1.0,  1.0),
};

struct Vertex {
	float3 position;
	float size;
	uint color;
}

struct VsInput {
	uint vertex_id   : SV_VertexID;
	uint instance_id : SV_InstanceID;
}

struct PsInput {
	float4 position : SV_POSITION;
	float4 color : COLOR0;
	float2 uv : TEXCOORD0;
	float size : SIZE;
	float edge_distance : EDGE_DISTANCE;
}

[shader("vertex")]
PsInput main_vs(VsInput input) {
	let vertex_buffer = buffers[vb_index];

	let position = POSITIONS[input.vertex_id];

	int v0_id = input.instance_id * 2;
	int v1_id = v0_id + 1;

	let v0 = vertex_buffer.Load<Vertex>(v0_id * sizeof(Vertex));
	let v1 = vertex_buffer.Load<Vertex>(v1_id * sizeof(Vertex));
	let v = (input.vertex_id % 2 == 0) ? v0 : v1;

	var output: PsInput;

	output.color = unpack_float4(v.color);
	output.size = max(v.size, ANTIALIASING);
	output.edge_distance = output.size * position.y;

	let p0 = mul(view_projection, float4(v0.position, 1.0));
	let p1 = mul(view_projection, float4(v1.position, 1.0));
	var dir = (p0.xy / p0.w) - (p1.xy / p1.w);
	dir = normalize(float2(dir.x, dir.y * screen_size.y / screen_size.x));
	let tng = float2(-dir.y, dir.x) * output.size / screen_size;

	output.position = (input.vertex_id % 2 == 0) ? p0 : p1;
	output.position.xy += tng * position.y * output.position.w;

	return output;
}

[shader("pixel")]
float4 main_ps(PsInput input) : SV_Target {
	let scene_depth_texture = textures_2d_float[depth_texture_id];

	float2 uv = input.position.xy / screen_size;
	float scene_depth = scene_depth_texture.Sample(linear_sampler, uv).r;

	float gizmo_depth = input.position.w;

	let alpha_multiplier = gizmo_depth > scene_depth ? 0.25 : 1.0;

	let color = input.color;

	var d = abs(input.edge_distance) / input.size;
	d = smoothstep(1.0, 1.0 - (ANTIALIASING / input.size), d);

	return float4(color.rgb, color.a * d * alpha_multiplier);
}
