import sample_generator;
import spectrum;

struct LightLiSample {
	/// Incident radiance at the shading point (unshadowed).
	SampledSpectrum li;
	/// Normalized direction from the shading point to the sampled point on the light in world space.
	float3 wi;
	/// Distance from the shading point to the sampled point on the light.
	float distance;
	/// Probability density with respect to solid angle to sample direction `wi`.
	float pdf;
}

interface Light {
	/// Samples an incident direction at point `p` along which illumination from the light may be arriving.
	func sample_li<S: SampleGenerator>(p: float3, inout sg: S) -> LightLiSample;

	/// Probability density with respect to solid angle to sample direction `wi` from point `p`.
	func pdf_li(p: float3, wi: float3) -> float;
}
