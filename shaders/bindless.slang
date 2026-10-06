[[vk::binding(0, 0)]] ByteAddressBuffer buffers[] : register(t0, space1);
[[vk::binding(0, 0)]] RWByteAddressBuffer rw_buffers[] : register(u0, space2);

[[vk::binding(0, 1)]] Texture2D<float4> textures_2d_float4[] : register(t0, space3);
[[vk::binding(0, 1)]] Texture2D<float> textures_2d_float[] : register(t0, space4);

[[vk::binding(0, 2)]] RWTexture2D<float4> rw_textures_2d_float4[] : register(u0, space5);
[[vk::binding(0, 2)]] RWTexture2D<float> rw_textures_2d_float[] : register(u0, space6);

[[vk::binding(0, 3)]] RaytracingAccelerationStructure acceleration_structures[] : register(t0, space7);

[[vk::binding(0, 4)]] SamplerState samplers[] : register(s0, space8);

public func get_buffer(index: uint) -> ByteAddressBuffer { return buffers[index]; }
public func get_rw_buffer(index: uint) -> RWByteAddressBuffer { return rw_buffers[index]; }

public func get_texture2d_float4(index: uint) -> Texture2D<float4> { return textures_2d_float4[index]; }
public func get_texture2d_float(index: uint) -> Texture2D<float> { return textures_2d_float[index]; }

public func get_rw_texture2d_float4(index: uint) -> RWTexture2D<float4> { return rw_textures_2d_float4[index]; }
public func get_rw_texture2d_float(index: uint) -> RWTexture2D<float> { return rw_textures_2d_float[index]; }

public func get_acceleration_structure(index: uint) -> RaytracingAccelerationStructure { return acceleration_structures[index]; }

public func get_sampler(index: uint) -> SamplerState { return samplers[index]; }
