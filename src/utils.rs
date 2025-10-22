use std::time::{SystemTime, UNIX_EPOCH};

/// Generate a unique ID using timestamp and random suffix
/// This is a simple alternative to UUID v4 for environments where the uuid crate is not available
pub fn generate_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    // Create a simple pseudo-random suffix using the timestamp
    let random_suffix = (timestamp % 1000000) as u32;

    format!("{:x}-{:06x}", timestamp, random_suffix)
}

pub fn current_timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}
