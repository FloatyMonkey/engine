implementing light;

import math;
import onb;
import pathtracer.sampling;
import sample_generator;

public struct SphereLight: Light {
	public float3 emission;
	public float3 position;
	public float radius;

	public func sample_li<S: SampleGenerator>(p: float3, inout sg: S) -> LightLiSample {
		let u = sample_next_2d(sg);

		let r_sq = sqr(radius);
		let d_sq = distance_sq(p, position);

		// TODO: No light emitted if point is inside sphere.
		if (d_sq <= r_sq) {
			return {};
		}

		let sin2_theta_max = r_sq / d_sq;
		let sin_theta_max = safe_sqrt(sin2_theta_max);
		let cos_theta_max = safe_sqrt(1.0 - sin2_theta_max);
		var one_minus_cos_theta_max = 1.0 - cos_theta_max;

		var cos_theta = (cos_theta_max - 1.0) * u[0] + 1.0;
		var sin2_theta = 1.0 - sqr(cos_theta);
		if (sin2_theta_max < sqr(sin(radians(1.5)))) {
			// Use Taylor series expansion for small angles.
			sin2_theta = sin2_theta_max * u[0];
			cos_theta = sqrt(1.0 - sin2_theta);
			one_minus_cos_theta_max = sin2_theta_max / 2.0;
		}

		let cos_alpha = sin2_theta / sin_theta_max + cos_theta * safe_sqrt(1.0 - sin2_theta / sin2_theta_max);
		let sin_alpha = safe_sqrt(1.0 - sqr(cos_alpha));

		let phi = 2.0 * PI * u[1];
		let w = spherical_direction(sin_alpha, cos_alpha, phi);
		let frame = Onb::from_z(normalize(position - p));
		let n = frame.local_to_world(-w);
		let p_on_sphere = position + radius * n;

		var ls: LightLiSample;
		ls.li = emission;
		ls.wi = normalize(p_on_sphere - p);
		ls.distance = distance(p_on_sphere, p);
		ls.pdf = 1.0 / (2.0 * PI * one_minus_cos_theta_max);

		return ls;
	}

	public func pdf_li(p: float3, wi: float3) -> float {
		return 0.0; // TODO: implement
	}
}

func spherical_direction(sin_theta: float, cos_theta: float, phi: float) -> float3 {
	return float3(clamp(sin_theta, -1, 1) * cos(phi), clamp(sin_theta, -1, 1) * sin(phi), clamp(cos_theta, -1, 1));
}
