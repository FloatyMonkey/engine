import bxdf;
import math;
import onb;
import spectrum;
import sample_generator;
import pathtracer.sampling;

struct DiffuseBxdf: Bxdf {
	SampledSpectrum albedo;

	func eval(wi: float3, wo: float3) -> BxdfEval {
		if (!onb::same_hemisphere(wi, wo)) {
			return {};
		}

		let pdf = cosine_hemisphere_pdf(onb::abs_cos_theta(wo));

		return { albedo / PI, pdf };
	}

	func sample<S: SampleGenerator>(wi: float3, inout sg: S) -> BxdfSample {
		var wo = sample_cosine_hemisphere(sample_next_2d(sg));

		if (wi.z < 0.0) {
			wo.z *= -1.0;
		}

		let pdf = cosine_hemisphere_pdf(onb::abs_cos_theta(wo));

		return { wo, pdf, albedo / PI, BxdfLobe::DiffuseReflection };
	}
}
