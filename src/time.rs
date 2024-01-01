pub struct Time {
	start: std::time::Instant,
	delta: std::time::Duration,
}

impl Time {
	pub fn new() -> Self {
		let now = std::time::Instant::now();
		
		Time {
			start: now,
			delta: now.duration_since(now),
		}
	}

	pub fn update(&mut self) {
		let now = std::time::Instant::now();
		self.delta = now.duration_since(self.start);
		self.start = now;
	}

	pub fn delta_seconds(&self) -> f32 {
		self.delta.as_secs_f32()
	}
}
