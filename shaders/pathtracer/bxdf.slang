import onb;
import sample_generator;
import spectrum;

public enum BxdfLobe {
	Reflection   = 1 << 0,
	Transmission = 1 << 1,

	Diffuse      = 1 << 2,
	Glossy       = 1 << 3,
	Singular     = 1 << 4,

	DiffuseReflection    = Diffuse  | Reflection,
	DiffuseTransmission  = Diffuse  | Transmission,
	GlossyReflection     = Glossy   | Reflection,
	GlossyTransmission   = Glossy   | Transmission,
	SingularReflection   = Singular | Reflection,
	SingularTransmission = Singular | Transmission,
}

public struct BxdfEval {
	public SampledSpectrum f;
	public float pdf;
}

public struct BxdfSample {
	/// Incident direction.
	public float3 wo;
	/// Probability density with respect to solid angle to sample direction `wo`.
	public float pdf;
	/// BxDF value.
	public SampledSpectrum f; // TODO: Optimize by making this the samples `weight` ie. f * cos_theta / pdf.
	/// Sampled lobe.
	public BxdfLobe lobe;
}

/// Bidirectional Distribution Function.
///
/// # Conventions:
/// - Operations are done in a local coordinate frame with normal N=(0,0,1), tangent T=(1,0,0) and bitangent B=(0,1,0).
/// - The incident and outgoing direction point away from the shading location.
/// - The outgoing direction `wo` is sampled.
public interface Bxdf {
	/// Evaluates the BxDF for incident direction `wi` and outgoing direction `wo`.
	func eval(wi: float3, wo: float3) -> BxdfEval;

	/// Samples an outgoing direction for incident direction `wi`.
	func sample<S: SampleGenerator>(wi: float3, inout sg: S) -> BxdfSample;
}

public struct Bsdf<B: Bxdf> {
	B bxdf;
	Onb basis;

	public func eval(wi: float3, wo: float3) -> BxdfEval {
		let wi_local = basis.world_to_local(wi);
		let wo_local = basis.world_to_local(wo);
		return bxdf.eval(wi_local, wo_local);
	}

	public func sample<S: SampleGenerator>(wi: float3, inout sg: S) -> BxdfSample {
		let wi_local = basis.world_to_local(wi);
		var sample = bxdf.sample(wi_local, sg);
		sample.wo = basis.local_to_world(sample.wo);
		return sample;
	}
}
