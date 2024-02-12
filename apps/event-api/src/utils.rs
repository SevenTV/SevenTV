use std::time::Duration;

use rand::Rng;

/// Jitter a duration by up to 10%.
/// This function is useful for adding jitter to timeouts.
/// You might want to add jitter to timeouts to avoid thundering herds, where
/// many threads all wake up at the same time when a timeout expires.
pub fn jitter(duration: Duration) -> Duration {
	let mut rng = rand::thread_rng();
	let jitter = rng.gen_range(0..=duration.as_millis() / 10);
	duration + Duration::from_millis(jitter as u64)
}
