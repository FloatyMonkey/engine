import math;
import onb;
import sample_generator;
import sampling;

/// Converts isotropic alpha and anisotropy factor into anisotropic Trowbridge-Reitz alpha.
/// @param alpha Squared perceptual roughness.
/// @param anisotropy Factor in range [0, 1] which increases roughness along tangent and decreases it along bitangent.
/// @return Alpha along tangent and bitangent.
func anisotropic_roughness(float alpha, float anisotropy) -> float2 {
	// [Kulla 2017, *Revisiting Physically Based Shading at Imageworks*]
	return float2(
		max(1e-3, alpha * (1.0 + anisotropy)),
		max(1e-3, alpha * (1.0 - anisotropy))
	);
}

interface MicrofacetDistribution {
	/// Returns true if the distribution can't be sampled reliably.
	func is_singular() -> bool;

	/// Normal Distribution Function (NDF), differential area of microfacets with surface normal `wm`.
	func d(wm: float3) -> float;

	/// Invisible masked microfacet area per visible microfacet area.
	func lambda(w: float3) -> float;

	/// Samples the distribution of normals visible from direction `w`.
	func sample_wm<S: SampleGenerator>(w: float3, inout sg: S) -> float3;

	/// Probability density for sampling normal `wm` from `d(w)`.
	func pdf(w: float3, wm: float3) -> float;
}

func g1<M: MicrofacetDistribution>(m: M, w: float3) -> float {
	return 1.0 / (1.0 + m.lambda(w));
}

func g2<M: MicrofacetDistribution>(m: M, wo: float3, wi: float3) -> float {
	return 1.0 / (1.0 + m.lambda(wo) + m.lambda(wi));
}

/// Anisotropic Trowbridge-Reitz (GGX) Microfacet Distribution.
/// [Walter et al. 2007, *Microfacet Models for Refraction through Rough Surfaces*]
struct TrowbridgeReitzDistribution: MicrofacetDistribution {
	float2 alpha;

	func is_singular() -> bool {
		return all(alpha < 1e-3);
	}

	func d(wm: float3) -> float {
		let tan2_theta = onb::tan2_theta(wm);
		if (isinf(tan2_theta)) {
			return 0.0;
		}

		let cos4_theta = sqr(onb::cos2_theta(wm));
		if (cos4_theta < 1e-16) {
			return 0.0;
		}

		let e = tan2_theta * (sqr(onb::cos_phi(wm) / alpha.x) + sqr(onb::sin_phi(wm) / alpha.y));
		return 1.0 / (PI * alpha.x * alpha.y * cos4_theta * sqr(1.0 + e));
	}

	func lambda(w: float3) -> float {
		let tan2_theta = onb::tan2_theta(w);
		if (isinf(tan2_theta)) {
			return 0.0;
		}

		let alpha2 = sqr(onb::cos_phi(w) * alpha.x) + sqr(onb::sin_phi(w) * alpha.y);
		return (sqrt(1.0 + alpha2 * tan2_theta) - 1.0) / 2.0;
	}

	func sample_wm<S: SampleGenerator>(w: float3, inout sg: S) -> float3 {
		let u = sample_next_2d(sg);

		// Transform `w` to hemisphere configuration.
		let wh = normalize(float3(w.xy * alpha, w.z));
		
		// Sample the visible hemisphere as half vectors.
		// [Dupey & Benyoub 2023, *Sampling Visible GGX Normals with Spherical Caps*](https://arxiv.org/pdf/2306.05044.pdf)
		let phi = 2.0 * PI * u[0];
		let z = (1.0 - u[1]) * (1.0 + wh.z) - wh.z;
		let sin_theta = safe_sqrt(1.0 - sqr(z));
		let c = float3(sin_theta * cos(phi), sin_theta * sin(phi), z);
		let h = c + wh;

		// Transform back to ellipsoid configuration.
		return normalize(float3(h.xy * alpha, h.z));
	}

	func pdf(w: float3, wm: float3) -> float {
		return g1(this, w) / onb::abs_cos_theta(w) * d(wm) * abs(dot(w, wm));
	}
}
