use std::time::{SystemTime, UNIX_EPOCH};

use uuid::Uuid;

use crate::error::{AnyListError, Result};

pub(crate) fn encode_operation_list<M: prost::Message>(operation_list: &M) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    operation_list
        .encode(&mut buf)
        .map_err(|e| AnyListError::ProtobufError(format!("Failed to encode operation: {}", e)))?;
    Ok(buf)
}

pub fn generate_id() -> String {
    Uuid::simple(Uuid::new_v4()).to_string()
}

pub fn current_timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}
