import math;

/// [Gulbrandsen 2014, *Artist Friendly Metallic Fresnel*](http://jcgt.org/published/0003/04/03)
func artistic_ior(reflectivity: float3, edge_color: float3, out ior: float3, out extinction: float3) {
	let r = clamp(reflectivity, 0.0, 0.99);
	let r_sqrt = sqrt(r);
	let n_min = (1.0 - r) / (1.0 + r);
	let n_max = (1.0 + r_sqrt) / (1.0 - r_sqrt);
	ior = lerp(n_max, n_min, edge_color);

	let np1 = ior + 1.0;
	let nm1 = ior - 1.0;
	let k2 = (np1 * np1 * r - nm1 * nm1) / (1.0 - r);
	extinction = safe_sqrt(k2);
}

func fr_conductor(cos_theta_i: float, eta: float, k: float) -> float {
	float cos_theta_i_sq = cos_theta_i * cos_theta_i;
	float sin_theta_i_sq = max(1.0 - cos_theta_i_sq, 0.0);
	float sin_theta_i_qu = sin_theta_i_sq * sin_theta_i_sq;

	float inner_term = eta * eta - k * k - sin_theta_i_sq;
	float a_sq_plus_b_sq = sqrt(max(inner_term * inner_term + 4.0 * eta * eta * k * k, 0.0));
	float a = sqrt(max((a_sq_plus_b_sq + inner_term) * 0.5, 0.0));

	float rs = ((a_sq_plus_b_sq + cos_theta_i_sq) - (2.0 * a * cos_theta_i))/
			   ((a_sq_plus_b_sq + cos_theta_i_sq) + (2.0 * a * cos_theta_i));
	float rp = ((cos_theta_i_sq * a_sq_plus_b_sq + sin_theta_i_qu) - (2.0 * a * cos_theta_i * sin_theta_i_sq))/
			   ((cos_theta_i_sq * a_sq_plus_b_sq + sin_theta_i_qu) + (2.0 * a * cos_theta_i * sin_theta_i_sq));

	return 0.5f * (rs + rs * rp);
}

func fr_conductor(cos_theta_i: float, eta: float3, k: float3) -> float3 {
	return float3(
		fr_conductor(cos_theta_i, eta.x, k.x),
		fr_conductor(cos_theta_i, eta.y, k.y),
		fr_conductor(cos_theta_i, eta.z, k.z)
	);
}
