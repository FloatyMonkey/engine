import util;

cbuffer ConstantBuffer : register(b0) {
	float4x4 view_projection;
	float2 screen_size;
	uint vb_index;
}

ByteAddressBuffer buffers[] : register(t0, space1);

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
	var color = input.color;

	var d = abs(input.edge_distance) / input.size;
	d = smoothstep(1.0, 1.0 - (ANTIALIASING / input.size), d);
	color.a *= d;

	return color;
}
