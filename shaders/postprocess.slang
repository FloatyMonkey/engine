import math;

/// [Wrensch 2023, *Minimal AgX Implementation*](https://iolite-engine.com/blog_posts/minimal_agx_implementation)
func agx_default_contrast_approx(x: float3) -> float3 {
	// Mean error^2: 3.6705141e-06
	//return (((((15.5 * x - 40.14) * x + 31.96) * x - 6.868) * x + 0.4298) * x + 0.1191) * x - 0.00232;

	// Mean error^2: 1.85907662e-06
	return ((((((-17.86 * x + 78.01) * x - 126.7) * x + 92.06) * x - 28.72) * x + 4.361) * x - 0.1718) * x + 0.002857;
}

func agx(val: float3) -> float3 {
	var res: float3 = val;

	/// Input transform (inset)
	const float3x3 agx_mat = {
		0.842479062253094,  0.0784335999999992, 0.0792237451477643,
		0.0423282422610123, 0.878468636469772,  0.0791661274605434,
		0.0423756549057051, 0.0784336,          0.879142973793104
	};

	res = mul(agx_mat, res);

	/// Log2 space encoding
	const float min_ev = -12.47393;
	const float max_ev = 4.026069;

	res = clamp(log2(res), min_ev, max_ev);
	res = (res - min_ev) / (max_ev - min_ev);
  
	// Sigmoid function approximation
	res = agx_default_contrast_approx(res);

	return res;
}

func agx_eotf(val: float3) -> float3 {
	var res: float3 = val;

	// Inverse input transform (outset)
	const float3x3 agx_mat_inv = {
		 1.19687900512017,   -0.0980208811401368, -0.0990297440797205,
		-0.0528968517574562,  1.15190312990417,   -0.0989611768448433,
		-0.0529716355144438, -0.0980434501171241,  1.15107367264116
	};

	res = mul(agx_mat_inv, res);

	return res;
}

/// American Society of Cinematographers Color Decision List (ASC CDL) transform.
func cdl_transform(i: float3, slope: float3, offset: float3, power: float3, saturation: float) -> float3 {
	let luma = luminance(i);
	let v = pow(i * slope + offset, power);
	return luma + saturation * (v - luma);
}

/// A golden tinted, slightly washed look.
func agx_golden(c: float3) -> float3 {
	return cdl_transform(c, float3(1.0, 0.9, 0.5), 0.0, 0.8, 1.3);
}

/// A punchy and more chroma laden look.
func agx_punchy(c: float3) -> float3 {
	return cdl_transform(c, 1.0, 0.0, 1.35, 1.4);
}

cbuffer ConstantBuffer : register(b0) {
	uint input_id;
	uint output_id;
	uint2 output_res;
}

Texture2D<float4> textures_2d_float4[] : register(t0, space1);
RWTexture2D<float4> rw_textures_2d_float4[] : register(u0, space2);

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 id : SV_DispatchThreadID) {
	let pixel = id.xy;
	if (any(pixel >= output_res)) return;

	let input = textures_2d_float4[input_id];
	let output = rw_textures_2d_float4[output_id];

	var color = input[pixel].xyz;
	color = agx(color);
	//color = agx_golden(color);
	//color = agx_punchy(color);
	color = agx_eotf(color);

	output[pixel] = float4(color, 1.0);
}
