pub fn next_15_minutes() -> u64 {
	use std::time::{Duration, SystemTime, UNIX_EPOCH};

	(SystemTime::now().duration_since(UNIX_EPOCH).unwrap() + Duration::from_secs(15 * 60)).as_secs()
}
