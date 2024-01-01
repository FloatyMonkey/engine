/// [Duff et al. 2017, *Building an Orthonormal Basis, Revisited*](https://jcgt.org/published/0006/01/01/paper.pdf)
func orthonormal_basis(n: float3) -> Onb {
	let sign = n.z < 0.0 ? -1.0 : 1.0;
	let p = -1.0 / (sign + n.z);
	let q = n.x * n.y * p;

	let t = float3(1.0 + sign * n.x * n.x * p, sign * q, -sign * n.x);
	let b = float3(q, sign + n.y * n.y * p, -n.y);

	return { t, b, n };
}

struct Onb {
	float3 x, y, z;

	static func from_z(n: float3) -> Onb {
		return orthonormal_basis(n);
	}
	
	func local_to_world(v: float3) -> float3 {
		return v.x * x + v.y * y + v.z * z;
	}

	func world_to_local(v: float3) -> float3 {
		return float3(dot(v, x), dot(v, y), dot(v, z));
	}
}

namespace onb {
	func cos_theta(w: float3) -> float {
		return w.z;
	}

	func cos2_theta(w: float3) -> float {
		return w.z * w.z;
	}

	func sin_theta(w: float3) -> float {
		return sqrt(sin2_theta(w));
	}

	func sin2_theta(w: float3) -> float {
		return max(0.0, 1.0 - cos2_theta(w));
	}

	func tan_theta(w: float3) -> float {
		return sin_theta(w) / cos_theta(w);
	}

	func tan2_theta(w: float3) -> float {
		return sin2_theta(w) / cos2_theta(w);
	}

	func cos_phi(w: float3) -> float {
		float st = sin_theta(w);
		return (st == 0) ? 1 : clamp(w.x / st, -1, 1);
	}

	func sin_phi(w: float3) -> float {
		float st = sin_theta(w);
		return (st == 0) ? 0 : clamp(w.y / st, -1, 1);
	}

	func cos2_phi(w: float3) -> float {
		return cos_phi(w) * cos_phi(w);
	}

	func sin2_phi(w: float3) -> float {
		return sin_phi(w) * sin_phi(w);
	}

	func abs_cos_theta(w: float3) -> float {
		return abs(cos_theta(w));
	}

	func same_hemisphere(w: float3, wp: float3) -> bool {
		return w.z * wp.z > 0.0;
	}
}
