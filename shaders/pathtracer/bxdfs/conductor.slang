import bxdf;
import math;
import onb;
import sample_generator;
import spectrum;
import pathtracer.sampling;
import pathtracer.microfacet;
import pathtracer.fresnel;

struct ConductorBxdf: Bxdf {
	TrowbridgeReitzDistribution distribution;
	SampledSpectrum eta;
	SampledSpectrum k;

	func eval(wi: float3, wo: float3) -> BxdfEval {
		if (!onb::same_hemisphere(wi, wo)) {
			return {};
		}

		if (distribution.is_singular()) {
			return {};
		}

		let cos_theta_i = onb::abs_cos_theta(wi);
		let cos_theta_o = onb::abs_cos_theta(wo);

		if (cos_theta_i == 0.0 || cos_theta_o == 0.0) {
			return {};
		}

		let wm = normalize(wi + wo);

		let fr = fr_conductor(abs(dot(wi, wm)), eta, k);

		let f = distribution.d(wm) * fr * g2(distribution, wi, wo) / (4.0 * cos_theta_i * cos_theta_o);
		let pdf = distribution.pdf(wi, wm) / (4.0 * abs(dot(wi, wm)));

		return { f, pdf };
	}

	func sample<S: SampleGenerator>(wi: float3, inout sg: S) -> BxdfSample {
		if (distribution.is_singular()) {
			let wo = float3(-wi.x, -wi.y, wi.z);
			let f = fr_conductor(onb::abs_cos_theta(wo), eta, k) / onb::abs_cos_theta(wo);

			return { wo, 1.0, f, BxdfLobe::SingularReflection };
		}

		if (onb::cos_theta(wi) == 0.0) {
			return {};
		}

		let wm = distribution.sample_wm(wi, sg);
		let wo = reflect(-wi, wm);

		if (!onb::same_hemisphere(wi, wo)) {
			return {};
		}

		let pdf = distribution.pdf(wi, wm) / (4.0 * abs(dot(wi, wm)));

		let cos_theta_i = onb::abs_cos_theta(wi);
		let cos_theta_o = onb::abs_cos_theta(wo);

		if (cos_theta_i == 0.0 || cos_theta_o == 0.0) {
			return {};
		}

		let fr = fr_conductor(abs(dot(wi, wm)), eta, k);

		let f = distribution.d(wm) * fr * g2(distribution, wi, wo) / (4.0 * cos_theta_i * cos_theta_o);

		return { wo, pdf, f, BxdfLobe::GlossyReflection };
	}
}
