import onb;
import sample_generator;
import spectrum;

enum BxdfLobe {
	Reflection   = 1 << 0,
	Transmission = 1 << 1,

	Diffuse      = 1 << 2,
	Glossy       = 1 << 3,
	Specular     = 1 << 4,

	DiffuseReflection    = (uint)BxdfLobe::Diffuse  | (uint)BxdfLobe::Reflection,
	DiffuseTransmission  = (uint)BxdfLobe::Diffuse  | (uint)BxdfLobe::Transmission,
	GlossyReflection     = (uint)BxdfLobe::Glossy   | (uint)BxdfLobe::Reflection,
	GlossyTransmission   = (uint)BxdfLobe::Glossy   | (uint)BxdfLobe::Transmission,
	SpecularReflection   = (uint)BxdfLobe::Specular | (uint)BxdfLobe::Reflection,
	SpecularTransmission = (uint)BxdfLobe::Specular | (uint)BxdfLobe::Transmission,
}

struct BxdfSample {
	/// Incident direction.
	float3 wi;
	/// Probability density with respect to solid angle to sample direction `wi`.
	float pdf;
	/// BxDF value.
	SampledSpectrum f; // TODO: Optimize by making this the samples `weight` ie. f * cos_theta / pdf.
	/// Sampled lobe.
	BxdfLobe lobe;
}

/// Bidirectional Distribution Function.
///
/// # Conventions:
/// - Operations are done in a local coordinate frame with normal N=(0,0,1), tangent T=(1,0,0) and bitangent B=(0,1,0).
/// - The incident and outgoing direction point away from the shading location.
/// - The incident direction `wi` is sampled.
interface Bxdf {
	/// Evaluates the BxDF for incident direction `wi` and outgoing direction `wo`.
	func f(wo: float3, wi: float3) -> SampledSpectrum;

	/// Samples an incident direction for outgoing direction `wo`.
	func sample_f<S: SampleGenerator>(wo: float3, inout sg: S) -> BxdfSample;

	/// Probability density with respect to solid angle to sample incident direction `wi` from outgoing direction `wo`.
	func pdf(wo: float3, wi: float3) -> float;
}

struct Bsdf<B: Bxdf> {
	B bxdf;
	Onb basis;

	func f(wo: float3, wi: float3) -> SampledSpectrum {
		let wo_local = basis.world_to_local(wo);
		let wi_local = basis.world_to_local(wi);
		return bxdf.f(wo_local, wi_local);
	}

	func sample_f<S: SampleGenerator>(wo: float3, inout sg: S) -> BxdfSample {
		let wo_local = basis.world_to_local(wo);
		var sample = bxdf.sample_f(wo_local, sg);
		sample.wi = basis.local_to_world(sample.wi);
		return sample;
	}

	func pdf(wo: float3, wi: float3) -> float {
		let wo_local = basis.world_to_local(wo);
		let wi_local = basis.world_to_local(wi);
		return bxdf.pdf(wo_local, wi_local);
	}
}
