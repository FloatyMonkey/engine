static const float PI = 3.14159265358979323846;

static const float FLT_MAX = 3.402823466e+38;
static const float FLT_MIN = 1.175494351e-38;

func sqr(x: float) -> float {
	return x * x;
}

func safe_sqrt(x: float) -> float {
	return sqrt(max(0.0, x));
}

func safe_sqrt(x: float3) -> float3 {
	return sqrt(max(0.0, x));
}

func length_sq(x: float3) -> float {
	return dot(x, x);
}

func distance_sq(a: float3, b: float3) -> float {
	return dot(a - b, a - b);
}

func srgb_from_linear(float3 rgb) -> float3 {
	return select(rgb <= 0.0031308, rgb * 12.92, 1.055 * pow(rgb, 1.0 / 2.4) - 0.055);
}

func linear_from_srgb(float3 rgb) -> float3 {
	return select(rgb <= 0.040449936, rgb / 12.92, pow((rgb + 0.055) / 1.055, 2.4));
}

/// Returns a relative luminance of an input linear RGB color in the ITU-R BT.709 color space.
func luminance(rgb: float3) -> float {
	return dot(rgb, float3(0.2126f, 0.7152f, 0.0722f));
}

/// Converts normalized direction `n` to position in [0, 1] in the equirectangular map (unsigned normalized).
func ndir_to_equirect_unorm(n: float3) -> float2 {
	let phi = atan2(n.y, n.x);
	let theta = acos(n.z);
	return float2(0.5 - phi / (2.0 * PI), theta / PI);
}

/// Converts normalized direction `n` to position in [0, 1] in the octahedral map (equal-area, unsigned normalized).
func ndir_to_oct_equal_area_unorm(n: float3) -> float2 {
	let r = sqrt(1.0 - abs(n.z));
	let phi = atan2(abs(n.y), abs(n.x));

	float2 p;
	p.y = r * phi * (2.0 / PI);
	p.x = r - p.y;

	if (n.z < 0.0) {
		p = 1.0 - p.yx;
	}
	p *= sign(n.xy);

	return p * 0.5 + 0.5;
}

/// Converts position `p` in [0, 1] in the octahedral map to normalized direction (equal area, unsigned normalized).
func oct_to_ndir_equal_area_unorm(float2 p) -> float3 {
	p = p * 2.0 - 1.0;

	let d = 1.0 - (abs(p.x) + abs(p.y));
	let r = 1.0 - abs(d);

	let phi = (r > 0.0) ? ((abs(p.y) - abs(p.x)) / r + 1.0) * (PI / 4.0) : 0.0;

	let f = r * sqrt(2.0 - r * r);
	let x = f * sign(p.x) * cos(phi);
	let y = f * sign(p.y) * sin(phi);
	let z = sign(d) * (1.0 - r * r);

	return float3(x, y, z);
}
