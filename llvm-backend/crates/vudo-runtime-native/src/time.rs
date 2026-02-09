//! Time host functions implementation

use std::time::{Duration, SystemTime, UNIX_EPOCH, Instant};
use std::sync::OnceLock;

static START_TIME: OnceLock<Instant> = OnceLock::new();

pub fn now_impl() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as i64
}

pub fn sleep_impl(millis: i64) {
    if millis > 0 {
        std::thread::sleep(Duration::from_millis(millis as u64));
    }
}

pub fn monotonic_now_impl() -> i64 {
    let start = START_TIME.get_or_init(Instant::now);
    start.elapsed().as_millis() as i64
}
