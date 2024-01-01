import light;
import sample_generator;

struct RectLight: Light {
	float3 emission;
	float3 position;
	float3 area_scaled_normal;
	float3 x;
	float3 y;

	func sample_li<S: SampleGenerator>(p: float3, inout sg: S) -> LightLiSample {
		let u = sample_next_2d(sg);

		let on_light = position + x * (u.x - 0.5) + y * (u.y - 0.5);

		let to_light = on_light - p;

		let dist_squared = dot(to_light, to_light);

		let dist = sqrt(dist_squared);

		let dir = to_light / dist;

		let area_scaled_cos_theta = dot(-dir, area_scaled_normal);

		if (area_scaled_cos_theta <= 0) {
			return {};
		}

		let pdf = dist_squared / area_scaled_cos_theta;

		var ls: LightLiSample;
		ls.li = emission;
		ls.wi = dir;
		ls.distance = dist;
		ls.pdf = pdf;

		return ls;
	}

	func pdf_li(p: float3, wi: float3) -> float {
		let to_light = position - p;

		let dist_squared = dot(to_light, to_light);

		let dist = sqrt(dist_squared);

		let dir = to_light / dist;

		let area_scaled_cos_theta = dot(-dir, area_scaled_normal);

		if (area_scaled_cos_theta <= 0) {
			return 0;
		}

		let pdf = dist_squared / area_scaled_cos_theta;

		return pdf;
	}
}
