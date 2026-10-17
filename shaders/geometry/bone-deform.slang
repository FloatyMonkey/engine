
[[vk::push_constant]] cbuffer ConstantBuffer {
	ByteAddressBuffer.Handle in_vertices;
	RWByteAddressBuffer.Handle out_vertices;

	ByteAddressBuffer.Handle lookup_stream;
	ByteAddressBuffer.Handle weight_stream;
	ByteAddressBuffer.Handle bone_matrices;

	uint num_vertices;
}

struct Vertex {
	float3 position;
	float3 normal;
}

func get_bone_transform(vertex_id: uint, lookup: ByteAddressBuffer.Handle, weights: ByteAddressBuffer.Handle, bones: ByteAddressBuffer.Handle) -> float3x4 {
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

	var vertex = in_vertices.Load<Vertex>(id * sizeof(Vertex));

	let transform = get_bone_transform(id, lookup_stream, weight_stream, bone_matrices);

	vertex.position = mul(transform, float4(vertex.position, 1.0));
	vertex.normal = normalize(mul((float3x3)transform, vertex.normal));

	out_vertices.Store(id * sizeof(Vertex), vertex);
}
