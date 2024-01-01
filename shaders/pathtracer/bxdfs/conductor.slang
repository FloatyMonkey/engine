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

	func f(wo: float3, wi: float3) -> SampledSpectrum {
		if (!onb::same_hemisphere(wo, wi)) {
			return SampledSpectrum(0.0);
		}

		if (distribution.is_singular()) {
			return SampledSpectrum(0.0);
		}

		let cos_theta_o = onb::abs_cos_theta(wo);
		let cos_theta_i = onb::abs_cos_theta(wi);

		if (cos_theta_o == 0.0 || cos_theta_i == 0.0) {
			return SampledSpectrum(0.0);
		}

		let wm = normalize(wo + wi);

		let fr = fr_conductor(abs(dot(wo, wm)), eta, k);

		return distribution.d(wm) * fr * g2(distribution, wo, wi) / (4.0 * cos_theta_o * cos_theta_i);
	}

	func sample_f<S: SampleGenerator>(wo: float3, inout sg: S) -> BxdfSample {
		if (distribution.is_singular()) {
			let wi = float3(-wo.x, -wo.y, wo.z);
			let f = fr_conductor(onb::abs_cos_theta(wi), eta, k) / onb::abs_cos_theta(wi);

			return { wi, 1.0, f, BxdfLobe::SpecularReflection };
		}

		if (onb::cos_theta(wo) == 0.0) {
			return {};
		}

		let wm = distribution.sample_wm(wo, sg);
		let wi = reflect(-wo, wm);

		if (!onb::same_hemisphere(wo, wi)) {
			return {};
		}

		let pdf = distribution.pdf(wo, wm) / (4.0 * abs(dot(wo, wm)));

		let cos_theta_o = onb::abs_cos_theta(wo);
		let cos_theta_i = onb::abs_cos_theta(wi);

		if (cos_theta_o == 0.0 || cos_theta_i == 0.0) {
			return {};
		}

		let fr = fr_conductor(abs(dot(wo, wm)), eta, k);

		let f = distribution.d(wm) * fr * g2(distribution, wo, wi) / (4.0 * cos_theta_o * cos_theta_i);

		return { wi, pdf, f, BxdfLobe::GlossyReflection };
	}

	func pdf(wo: float3, wi: float3) -> float {
		if (!onb::same_hemisphere(wo, wi)) {
			return 0.0;
		}

		if (distribution.is_singular()) {
			return 0.0;
		}

		let wm = normalize(wo + wi);
		return distribution.pdf(wo, wm) / (4.0 * abs(dot(wo, wm)));
	}
}
