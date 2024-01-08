import math;

/// Uniform sampling of the unit disk using polar coordinates.
func sample_uniform_disk_polar(u: float2) -> float2 {
	let r = sqrt(u[0]);
	let theta = 2.0 * PI * u[1];
	return float2(r * cos(theta), r * sin(theta));
}

/// Uniform sampling of the unit disk using Shirley's concentric mapping.
func sample_uniform_disk_concentric(u: float2) -> float2 {
	let uo = 2.0 * u - 1.0;
	if (uo[0] == 0.0 && uo[1] == 0.0) return 0.0;

	var theta: float;
	var r: float;

	if (abs(uo[0]) > abs(uo[1])) {
		r = uo[0];
		theta = (PI / 4.0) * (uo[1] / uo[0]);
	} else {
		r = uo[1];
		theta = (PI / 2.0) - (PI / 4.0) * (uo[0] / uo[1]);
	}

	return float2(r * cos(theta), r * sin(theta));
}

/// Uniform sampling of the unit sphere using spherical coordinates.
func sample_uniform_sphere(u: float2) -> float3 {
	let z = 1.0 - 2.0 * u[0];
	let r = safe_sqrt(1.0 - sqr(z));
	let phi = 2.0 * PI * u[1];
	return float3(r * cos(phi), r * sin(phi), z);
}

/// PDF for `sample_uniform_sphere()`.
func uniform_sphere_pdf() -> float {
	return 1.0 / (4.0 * PI);
}

/// Uniform sampling of the unit hemisphere using spherical coordinates.
func sample_uniform_hemisphere(u: float2) -> float3 {
	let z = u[0];
	let r = safe_sqrt(1.0 - sqr(z));
	let phi = 2.0 * PI * u[1];
	return float3(r * cos(phi), r * sin(phi), z);
}

/// PDF for `sample_uniform_hemisphere()`.
func uniform_hemisphere_pdf() -> float {
	return 1.0 / (2.0 * PI);
}

/// Cosine-weighted sampling of the unit hemisphere using Shirley's concentric maping.
func sample_cosine_hemisphere(u: float2) -> float3 {
	let d = sample_uniform_disk_concentric(u);
	let z = safe_sqrt(1.0 - sqr(d.x) - sqr(d.y));
	return float3(d.x, d.y, z);
}

/// PDF for `sample_cosine_hemisphere()`.
func cosine_hemisphere_pdf(cos_theta: float) -> float {
	return cos_theta / PI;
}

/// Uniform sampling of direction within a cone.
func sample_uniform_cone(u: float2, cos_theta_max: float) -> float3 {
	let z = u[0] * (1.0 - cos_theta_max) + cos_theta_max;
	let r = safe_sqrt(1.0 - sqr(z));
	let phi = 2.0 * PI * u[1];
	return float3(r * cos(phi), r * sin(phi), z);
}

/// PDF for `sample_uniform_cone()`.
func uniform_cone_pdf(cos_theta_max: float) -> float {
	return 1.0 / (2.0 * PI * (1.0 - cos_theta_max));
}

/// Uniform sampling of point on a triangle.
/// [Heitz 2019, *A Low-Distortion Map Between Triangle and Square*](https://hal.science/hal-02073696v2/document)
func sample_uniform_triangle(u: float2) -> float3 {
	var b0: float;
	var b1: float;

	if (u[0] < u[1]) {
		b0 = u[0] / 2.0;
		b1 = u[1] - b0;
	} else {
		b1 = u[1] / 2.0;
		b0 = u[0] - b1;
	}

	return float3(1.0 - b0 - b1, b0, b1);
}

/// Uniform sampling of point on a regular polygon.
func sample_uniform_regular_polygon(corners: uint, rotation: float, u: float2) -> float2 {
	var v = u[1];
	var u = u[0];

	// Sample corner number and reuse u
	let corner = floor(u * corners);
	u = u * corners - corner;

	// Uniform sampled triangle weights
	u = sqrt(u);
	v = v * u;
	u = 1.0 - u;

	// Point in triangle
	let angle = PI / corners;
	float2 p = float2((u + v) * cos(angle), (u - v) * sin(angle));

	// Rotate
	let rot = rotation + corner * 2.0 * angle;

	let cr = cos(rot);
	let sr = sin(rot);

	return float2(p.x * cr - p.y * sr, p.x * sr + p.y * cr);
}
