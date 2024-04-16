use std::fmt::Display;
use std::ops::Deref;

pub struct Number<N>(N);

impl<N> Deref for Number<N> {
	type Target = N;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl From<u64> for Number<u64> {
	fn from(n: u64) -> Self {
		Self(n)
	}
}

impl From<u32> for Number<u32> {
	fn from(n: u32) -> Self {
		Self(n)
	}
}

impl From<usize> for Number<usize> {
	fn from(n: usize) -> Self {
		Self(n)
	}
}

impl Display for Number<u64> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.0 >= 1_000_000_000 {
			write!(f, "{:.2}B", self.0 as f64 / 1_000_000_000.0)
		} else if self.0 >= 1_000_000 {
			write!(f, "{:.2}M", self.0 as f64 / 1_000_000.0)
		} else if self.0 >= 1_000 {
			write!(f, "{:.2}k", self.0 as f64 / 1_000.0)
		} else {
			write!(f, "{}", self.0)
		}
	}
}

impl Display for Number<u32> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.0 >= 1_000_000_000 {
			write!(f, "{:.2}B", self.0 as f64 / 1_000_000_000.0)
		} else if self.0 >= 1_000_000 {
			write!(f, "{:.2}M", self.0 as f64 / 1_000_000.0)
		} else if self.0 >= 1_000 {
			write!(f, "{:.2}k", self.0 as f64 / 1_000.0)
		} else {
			write!(f, "{}", self.0)
		}
	}
}

impl Display for Number<usize> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.0 >= 1_000_000_000 {
			write!(f, "{:.2}B", self.0 as f64 / 1_000_000_000.0)
		} else if self.0 >= 1_000_000 {
			write!(f, "{:.2}M", self.0 as f64 / 1_000_000.0)
		} else if self.0 >= 1_000 {
			write!(f, "{:.2}k", self.0 as f64 / 1_000.0)
		} else {
			write!(f, "{}", self.0)
		}
	}
}
