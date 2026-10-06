import bindless;

[[vk::push_constant]] cbuffer ConstantBuffer : register(b0) {
	uint num_vertices;

	uint in_vertices;
	uint out_vertices;

	uint lookup_stream;
	uint weight_stream;
	uint bone_matrices;
}

struct Vertex {
	float3 position;
	float3 normal;
}

func get_bone_transform(vertex_id: uint, lookup: ByteAddressBuffer, weights: ByteAddressBuffer, bones: ByteAddressBuffer) -> float3x4 {
	let offsets = lookup.Load<uint2>(vertex_id * sizeof(uint));
	let weights_offset = offsets[0];
	let weights_count = offsets[1] - offsets[0];

	var bone_transform = (float3x4)0.0;

	for (uint i = 0; i < weights_count; i++) {
		let index_weight = weights.Load<uint>((weights_offset + i) * sizeof(uint));
		let index = index_weight & 0xffff;
		let weight = float(index_weight >> 16) / 65535.0;

		bone_transform += weight * bones.Load<float3x4>(index * sizeof(float3x4));
	}

	return bone_transform;
}

[shader("compute")]
[numthreads(32, 1, 1)]
func main(uint id : SV_DispatchThreadID) {
	if (id >= num_vertices) return;

	let in_vertices = get_buffer(in_vertices);
	let out_vertices = get_rw_buffer(out_vertices);

	let lookup = get_buffer(lookup_stream);
	let weights = get_buffer(weight_stream);
	let bones = get_buffer(bone_matrices);

	var vertex = in_vertices.Load<Vertex>(id * sizeof(Vertex));

	let transform = get_bone_transform(id, lookup, weights, bones);

	vertex.position = mul(transform, float4(vertex.position, 1.0));
	vertex.normal = normalize(mul((float3x3)transform, vertex.normal));

	out_vertices.Store(id * sizeof(Vertex), vertex);
}
