import bxdf;
import math;
import onb;
import spectrum;
import sample_generator;
import pathtracer.sampling;

struct DiffuseBxdf: Bxdf {
	SampledSpectrum albedo;

	func f(wo: float3, wi: float3) -> SampledSpectrum {
		if (!onb::same_hemisphere(wo, wi)) {
			return SampledSpectrum(0.0);
		}

		return albedo / PI;
	}

	func sample_f<S: SampleGenerator>(wo: float3, inout sg: S) -> BxdfSample {
		var wi = sample_cosine_hemisphere(sample_next_2d(sg));

		if (wo.z < 0.0) {
			wi.z *= -1.0;
		}

		let pdf = cosine_hemisphere_pdf(onb::abs_cos_theta(wi));

		return { wi, pdf, albedo / PI, BxdfLobe::DiffuseReflection };
	}

	func pdf(wo: float3, wi: float3) -> float {
		if (!onb::same_hemisphere(wo, wi)) {
			return 0.0;
		}

		return cosine_hemisphere_pdf(onb::abs_cos_theta(wi));
	}
}
