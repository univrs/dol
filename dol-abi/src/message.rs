//! Message types for host-to-ABI communication

use serde::{Deserialize, Serialize};

/// A message from the host to the ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message identifier
    pub id: String,
    /// Message type
    pub msg_type: String,
    /// Message payload
    pub payload: serde_json::Value,
}

impl Message {
    /// Create a new message
    pub fn new(
        id: impl Into<String>,
        msg_type: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: id.into(),
            msg_type: msg_type.into(),
            payload,
        }
    }
}

/// A response message from the ABI to the host
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Response identifier (corresponds to request id)
    pub id: String,
    /// Whether the operation succeeded
    pub success: bool,
    /// Response data
    pub data: serde_json::Value,
    /// Optional error message
    pub error: Option<String>,
}

impl Response {
    /// Create a successful response
    pub fn success(id: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            id: id.into(),
            success: true,
            data,
            error: None,
        }
    }

    /// Create a failed response
    pub fn error(id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            success: false,
            data: serde_json::Value::Null,
            error: Some(error.into()),
        }
    }
}
