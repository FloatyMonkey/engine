interface SampleGenerator {
	[mutating]
	func next() -> uint;
}

/// Returns a random number in [0,1).
func sample_next_1d<S: SampleGenerator>(inout sg: S) -> float {
	// Divide upper 24 bits by 2^24 to get a number u in [0,1).
	// In floating-point precision this also ensures that 1.0 - u != 0.0.
	uint bits = sg.next();
	return (bits >> 8) * 0x1p-24;
}

/// Returns two random numbers in [0,1).
func sample_next_2d<S: SampleGenerator>(inout sg: S) -> float2 {
	return float2(sample_next_1d(sg), sample_next_1d(sg));
}

struct PCG32si: SampleGenerator {
	uint state;

	__init(uint seed) {
		state = seed;
		pcg_oneseq_32_step_r();
		state += seed;
		pcg_oneseq_32_step_r();
	}

	[mutating]
	func pcg_oneseq_32_step_r() {
		state = (state * 747796405u) + 2891336453u;
	}

	static func pcg_output_rxs_m_xs_32_32(state: uint) -> uint {
		let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
		return (word >> 22u) ^ word;
	}

	[mutating]
	func next() -> uint {
		let old_state = state;
		pcg_oneseq_32_step_r();
		return pcg_output_rxs_m_xs_32_32(old_state);
	}
};
