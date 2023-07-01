pub struct Random {
	a: i64,
	c: i64,
	previous: i64,
}

impl Random {
	pub fn new() -> Random {
			Random {
					a: 25214903917,
					c: 11,
					previous: 0,
			}
	}

	pub fn rseed(&mut self, seed: i64) {
			self.previous = seed;
	}

	pub fn rand(&mut self) -> i64 {
			let r = self.a * self.previous + self.c;
			self.previous = r;
			return r;
	}
}