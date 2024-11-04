module light;

__include lights.dome_light;
__include lights.rect_light;
__include lights.sphere_light;

import sample_generator;
import spectrum;

public struct LightLiSample {
	/// Incident radiance at the shading point (unshadowed).
	public SampledSpectrum li;
	/// Normalized direction from the shading point to the sampled point on the light in world space.
	public float3 wi;
	/// Distance from the shading point to the sampled point on the light.
	public float distance;
	/// Probability density with respect to solid angle to sample direction `wi`.
	public float pdf;
}

public interface Light {
	/// Samples an incident direction at point `p` along which illumination from the light may be arriving.
	func sample_li<S: SampleGenerator>(p: float3, inout sg: S) -> LightLiSample;

	/// Probability density with respect to solid angle to sample direction `wi` from point `p`.
	func pdf_li(p: float3, wi: float3) -> float;
}
