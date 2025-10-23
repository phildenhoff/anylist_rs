use std::fmt;

/// Custom error type for AnyList API operations
#[derive(Debug)]
pub enum AnyListError {
    AuthenticationFailed(String),
    NetworkError(String),
    InvalidResponse(String),
    NotFound(String),
    PermissionDenied(String),
    /// Protocol buffer decoding error
    ProtobufError(String),
    /// Generic error
    Other(String),
}

impl fmt::Display for AnyListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnyListError::AuthenticationFailed(msg) => write!(f, "Authentication failed: {}", msg),
            AnyListError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            AnyListError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            AnyListError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AnyListError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            AnyListError::ProtobufError(msg) => write!(f, "Protobuf error: {}", msg),
            AnyListError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for AnyListError {}

impl From<reqwest::Error> for AnyListError {
    fn from(err: reqwest::Error) -> Self {
        AnyListError::NetworkError(err.to_string())
    }
}

impl From<prost::DecodeError> for AnyListError {
    fn from(err: prost::DecodeError) -> Self {
        AnyListError::ProtobufError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AnyListError>;
