struct HierarchicalImportanceMap {
	/// Hierarchical importance map, entire mip chain.
	Texture2D<float> importance_map;
	/// Mip level for 1x1 resolution of `importance_map`.
	uint base_mip;

	func sample(u: float2, out pdf: float) -> float2 {
		var p = u;
		var pos: uint2 = 0;

		// Loop over mips of 2x2...NxN resolution.
		for (int mip = base_mip - 1; mip >= 0; mip--) {
			pos *= 2;

			let w: float[] = {
				importance_map.Load(int3(pos, mip)),
				importance_map.Load(int3(pos + uint2(1, 0), mip)),
				importance_map.Load(int3(pos + uint2(0, 1), mip)),
				importance_map.Load(int3(pos + uint2(1, 1), mip))
			};

			let q: float[] = {
				w[0] + w[2],
				w[1] + w[3]
			};

			var off: uint2;

			let d = q[0] / (q[0] + q[1]); // TODO: Prevent division by zero?

			if (p.x < d) {
				off.x = 0;
				p.x = p.x / d;
			} else {
				off.x = 1;
				p.x = (p.x - d) / (1.0 - d);
			}

			let e = off.x == 0 ? (w[0] / q[0]) : (w[1] / q[1]);

			if (p.y < e) {
				off.y = 0;
				p.y = p.y / e;
			} else {
				off.y = 1;
				p.y = (p.y - e) / (1.0 - e);
			}

			pos += off;
		}

		let uv = ((float2)pos + p) / (1u << base_mip);

		/// TODO: 1x1 mip holds integral over the entire importance map.
		/// Rescale the entire map so that the integral is 1.0,
		/// this allows to remove the texture `Load()` here and in `pdf()`.
		let avg_w = importance_map.Load(int3(0, 0, base_mip));
		pdf = importance_map[pos] / avg_w;

		return uv;
	}

	func pdf(uv: float2) -> float {
		let pos = uint2(uv * (1u << base_mip));
		let avg_w = importance_map.Load(int3(0, 0, base_mip));
		let pdf = importance_map[pos] / avg_w;
		return pdf;
	}
}
