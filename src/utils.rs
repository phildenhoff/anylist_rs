use std::time::{SystemTime, UNIX_EPOCH};

use uuid::Uuid;

pub fn generate_id() -> String {
    Uuid::simple(Uuid::new_v4()).to_string()
}

pub fn current_timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}
