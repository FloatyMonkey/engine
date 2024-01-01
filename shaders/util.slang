func pack_float4(f: float4) -> uint {
	uint u = 0;

	u |= (uint)(f.r * 255.0) << 24;
	u |= (uint)(f.g * 255.0) << 16;
	u |= (uint)(f.b * 255.0) << 8;
	u |= (uint)(f.a * 255.0) << 0;

	return u;
}

func unpack_float4(u: uint) -> float4 {
	float4 f;

	f.r = (float)((u >> 24) & 0xff) / 255.0;
	f.g = (float)((u >> 16) & 0xff) / 255.0;
	f.b = (float)((u >>  8) & 0xff) / 255.0;
	f.a = (float)((u >>  0) & 0xff) / 255.0;

	return f;
}

func pack_float4_signed(f: float4) -> uint {
	return pack_float4(f * 0.5 + 0.5);
}

func unpack_float4_signed(u: uint) -> float4 {
	return unpack_float4(u) * 2.0 - 1.0;
}
