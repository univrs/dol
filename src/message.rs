//! Spirit-to-Spirit message wire format for ABI communication.
//!
//! This module defines the message types and serialization protocol used for
//! communication between Spirits in the DOL runtime. Messages follow a
//! structured format with headers and typed payloads.
//!
//! # Message Structure
//!
//! Each message consists of:
//! - **Header**: Routing and metadata (sender, receiver, timestamp, sequence)
//! - **Payload**: The actual message content (Text, Binary, or JSON)
//!
//! # Examples
//!
//! ```rust
//! # #[cfg(feature = "serde")]
//! # {
//! use metadol::message::{Message, MessageHeader, MessagePayload};
//! use std::time::{SystemTime, UNIX_EPOCH};
//!
//! let now = SystemTime::now()
//!     .duration_since(UNIX_EPOCH)
//!     .unwrap()
//!     .as_secs_f64();
//!
//! let header = MessageHeader {
//!     sender: "spirit.alice".to_string(),
//!     receiver: "spirit.bob".to_string(),
//!     timestamp: now,
//!     sequence: 1,
//! };
//!
//! let payload = MessagePayload::Text("Hello, Bob!".to_string());
//! let message = Message::new(header, payload);
//!
//! // Serialize to bytes
//! let bytes = message.serialize().unwrap();
//!
//! // Deserialize from bytes
//! let restored = Message::deserialize(&bytes).unwrap();
//! assert_eq!(message.header.sender, restored.header.sender);
//! # }
//! ```

use crate::error::AbiError;
use serde::{Deserialize, Serialize};

/// Message header containing routing and metadata information.
///
/// The header identifies the sender and receiver of a message, captures
/// the time of creation, and maintains a sequence number for ordering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageHeader {
    /// The Spirit ID of the sender (e.g., "spirit.alice")
    pub sender: String,

    /// The Spirit ID of the receiver (e.g., "spirit.bob")
    pub receiver: String,

    /// Timestamp when the message was created (seconds since Unix epoch).
    ///
    /// This uses a floating-point representation to allow fractional seconds.
    pub timestamp: f64,

    /// Message sequence number for ordering and deduplication.
    ///
    /// This number should be monotonically increasing for messages from
    /// a given sender to a given receiver.
    pub sequence: u64,
}

impl MessageHeader {
    /// Creates a new message header.
    ///
    /// # Arguments
    ///
    /// * `sender` - The Spirit ID of the sender
    /// * `receiver` - The Spirit ID of the receiver
    /// * `timestamp` - The message creation timestamp in seconds since Unix epoch
    /// * `sequence` - The message sequence number
    ///
    /// # Returns
    ///
    /// A new `MessageHeader` instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use metadol::message::MessageHeader;
    /// use std::time::{SystemTime, UNIX_EPOCH};
    ///
    /// let now = SystemTime::now()
    ///     .duration_since(UNIX_EPOCH)
    ///     .unwrap()
    ///     .as_secs_f64();
    ///
    /// let header = MessageHeader::new(
    ///     "spirit.alice".to_string(),
    ///     "spirit.bob".to_string(),
    ///     now,
    ///     1,
    /// );
    /// ```
    pub fn new(sender: String, receiver: String, timestamp: f64, sequence: u64) -> Self {
        Self {
            sender,
            receiver,
            timestamp,
            sequence,
        }
    }

    /// Validates the message header.
    ///
    /// Checks that the sender and receiver have valid Spirit ID formats
    /// and that the timestamp is non-negative.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the header is valid, or an `AbiError` if validation fails.
    pub fn validate(&self) -> Result<(), AbiError> {
        if self.sender.is_empty() {
            return Err(AbiError::InvalidMessage(
                "sender cannot be empty".to_string(),
            ));
        }

        if self.receiver.is_empty() {
            return Err(AbiError::InvalidMessage(
                "receiver cannot be empty".to_string(),
            ));
        }

        if !self
            .sender
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
        {
            return Err(AbiError::InvalidMessage(format!(
                "sender '{}' contains invalid characters",
                self.sender
            )));
        }

        if !self
            .receiver
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
        {
            return Err(AbiError::InvalidMessage(format!(
                "receiver '{}' contains invalid characters",
                self.receiver
            )));
        }

        if self.timestamp < 0.0 {
            return Err(AbiError::InvalidMessage(
                "timestamp cannot be negative".to_string(),
            ));
        }

        Ok(())
    }
}

/// Message payload content with multiple supported formats.
///
/// Messages can carry different types of payloads depending on the
/// communication needs. Each variant represents a different encoding.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessagePayload {
    /// Text-encoded payload (UTF-8 string)
    ///
    /// Use this for human-readable messages or simple text data.
    Text(String),

    /// Binary-encoded payload (raw bytes)
    ///
    /// Use this for binary data like serialized structures or raw buffers.
    Binary(Vec<u8>),

    /// JSON-encoded payload (semi-structured data)
    ///
    /// Use this for structured data that needs human readability
    /// and flexibility.
    Json(serde_json::Value),
}

impl MessagePayload {
    /// Returns the content type of this payload.
    ///
    /// # Returns
    ///
    /// A string describing the payload type: "text", "binary", or "json"
    pub fn content_type(&self) -> &'static str {
        match self {
            MessagePayload::Text(_) => "text/plain",
            MessagePayload::Binary(_) => "application/octet-stream",
            MessagePayload::Json(_) => "application/json",
        }
    }

    /// Gets the payload size in bytes (after serialization).
    ///
    /// Note: This is an estimate and may not be perfectly accurate
    /// for the final serialized representation.
    pub fn size(&self) -> usize {
        match self {
            MessagePayload::Text(s) => s.len(),
            MessagePayload::Binary(b) => b.len(),
            MessagePayload::Json(v) => serde_json::to_string(v).unwrap_or_default().len(),
        }
    }

    /// Validates the payload.
    ///
    /// Checks that binary payloads are not excessively large
    /// and that JSON payloads are properly formed.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the payload is valid, or an `AbiError` if validation fails.
    pub fn validate(&self) -> Result<(), AbiError> {
        const MAX_BINARY_SIZE: usize = 1024 * 1024 * 10; // 10 MB

        match self {
            MessagePayload::Binary(b) if b.len() > MAX_BINARY_SIZE => {
                Err(AbiError::InvalidMessage(format!(
                    "binary payload exceeds maximum size of {} bytes",
                    MAX_BINARY_SIZE
                )))
            }
            _ => Ok(()),
        }
    }
}

/// A complete Spirit-to-Spirit message.
///
/// This struct combines a message header with a payload to form
/// a complete message that can be serialized and transmitted.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    /// The message header containing routing information
    pub header: MessageHeader,

    /// The message payload containing the actual data
    pub payload: MessagePayload,
}

impl Message {
    /// Creates a new message.
    ///
    /// # Arguments
    ///
    /// * `header` - The message header
    /// * `payload` - The message payload
    ///
    /// # Returns
    ///
    /// A new `Message` instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "serde")]
    /// # {
    /// use metadol::message::{Message, MessageHeader, MessagePayload};
    /// use std::time::{SystemTime, UNIX_EPOCH};
    ///
    /// let now = SystemTime::now()
    ///     .duration_since(UNIX_EPOCH)
    ///     .unwrap()
    ///     .as_secs_f64();
    ///
    /// let header = MessageHeader::new(
    ///     "spirit.alice".to_string(),
    ///     "spirit.bob".to_string(),
    ///     now,
    ///     1,
    /// );
    /// let payload = MessagePayload::Text("Hello".to_string());
    /// let msg = Message::new(header, payload);
    /// # }
    /// ```
    pub fn new(header: MessageHeader, payload: MessagePayload) -> Self {
        Self { header, payload }
    }

    /// Serializes the message to bytes.
    ///
    /// Uses bincode encoding for efficient, compact representation.
    /// The resulting bytes can be deserialized with [`deserialize`](Self::deserialize).
    ///
    /// # Returns
    ///
    /// A `Result` containing the serialized bytes on success, or an `AbiError` on failure.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "serde")]
    /// # {
    /// use metadol::message::{Message, MessageHeader, MessagePayload};
    /// use std::time::{SystemTime, UNIX_EPOCH};
    ///
    /// let now = SystemTime::now()
    ///     .duration_since(UNIX_EPOCH)
    ///     .unwrap()
    ///     .as_secs_f64();
    ///
    /// let header = MessageHeader::new("alice".to_string(), "bob".to_string(), now, 1);
    /// let payload = MessagePayload::Text("Test".to_string());
    /// let msg = Message::new(header, payload);
    /// let bytes = msg.serialize().unwrap();
    /// assert!(!bytes.is_empty());
    /// # }
    /// ```
    pub fn serialize(&self) -> Result<Vec<u8>, AbiError> {
        // Validate before serializing
        self.header.validate()?;
        self.payload.validate()?;

        bincode::serialize(self)
            .map_err(|e| AbiError::EffectFailed(format!("bincode error: {}", e)))
    }

    /// Serializes the message to a JSON string.
    ///
    /// Uses serde_json for human-readable JSON encoding.
    /// The resulting string can be deserialized with [`deserialize_json`](Self::deserialize_json).
    ///
    /// # Returns
    ///
    /// A `Result` containing the JSON string on success, or an `AbiError` on failure.
    pub fn serialize_json(&self) -> Result<String, AbiError> {
        // Validate before serializing
        self.header.validate()?;
        self.payload.validate()?;

        serde_json::to_string(self)
            .map_err(|e| AbiError::EffectFailed(format!("JSON error: {}", e)))
    }

    /// Deserializes a message from bytes.
    ///
    /// The bytes must have been produced by [`serialize`](Self::serialize).
    ///
    /// # Arguments
    ///
    /// * `bytes` - The serialized message bytes
    ///
    /// # Returns
    ///
    /// A `Result` containing the deserialized message on success, or an `AbiError` on failure.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "serde")]
    /// # {
    /// use metadol::message::{Message, MessageHeader, MessagePayload};
    /// use std::time::{SystemTime, UNIX_EPOCH};
    ///
    /// let now = SystemTime::now()
    ///     .duration_since(UNIX_EPOCH)
    ///     .unwrap()
    ///     .as_secs_f64();
    ///
    /// let header = MessageHeader::new("alice".to_string(), "bob".to_string(), now, 1);
    /// let payload = MessagePayload::Text("Test".to_string());
    /// let original = Message::new(header, payload);
    /// let bytes = original.serialize().unwrap();
    /// let restored = Message::deserialize(&bytes).unwrap();
    /// assert_eq!(original, restored);
    /// # }
    /// ```
    pub fn deserialize(bytes: &[u8]) -> Result<Self, AbiError> {
        bincode::deserialize(bytes)
            .map_err(|e| AbiError::InvalidMessage(format!("bincode error: {}", e)))
            .and_then(|msg: Message| {
                // Validate after deserializing
                msg.header.validate()?;
                msg.payload.validate()?;
                Ok(msg)
            })
    }

    /// Deserializes a message from a JSON string.
    ///
    /// The string must have been produced by [`serialize_json`](Self::serialize_json).
    ///
    /// # Arguments
    ///
    /// * `json_str` - The JSON-encoded message string
    ///
    /// # Returns
    ///
    /// A `Result` containing the deserialized message on success, or an `AbiError` on failure.
    pub fn deserialize_json(json_str: &str) -> Result<Self, AbiError> {
        serde_json::from_str(json_str)
            .map_err(|e| AbiError::InvalidMessage(format!("JSON error: {}", e)))
            .and_then(|msg: Message| {
                // Validate after deserializing
                msg.header.validate()?;
                msg.payload.validate()?;
                Ok(msg)
            })
    }

    /// Gets the total size of the message (header + payload).
    ///
    /// This is an estimate of the serialized size.
    pub fn size(&self) -> usize {
        // Rough estimate: header fields + payload
        self.header.sender.len()
            + self.header.receiver.len()
            + 8 // timestamp (f64)
            + 8 // sequence (u64)
            + self.payload.size()
    }

    /// Validates the entire message (header and payload).
    ///
    /// # Returns
    ///
    /// `Ok(())` if the message is valid, or an `AbiError` if validation fails.
    pub fn validate(&self) -> Result<(), AbiError> {
        self.header.validate()?;
        self.payload.validate()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_header_creation() {
        let header = MessageHeader::new(
            "spirit.alice".to_string(),
            "spirit.bob".to_string(),
            1234567890.5,
            42,
        );
        assert_eq!(header.sender, "spirit.alice");
        assert_eq!(header.receiver, "spirit.bob");
        assert_eq!(header.timestamp, 1234567890.5);
        assert_eq!(header.sequence, 42);
    }

    #[test]
    fn test_message_header_validation_valid() {
        let header = MessageHeader::new(
            "spirit.alice".to_string(),
            "spirit.bob".to_string(),
            1234567890.5,
            42,
        );
        assert!(header.validate().is_ok());
    }

    #[test]
    fn test_message_header_validation_empty_sender() {
        let header = MessageHeader::new(String::new(), "spirit.bob".to_string(), 1234567890.5, 42);
        assert!(header.validate().is_err());
    }

    #[test]
    fn test_message_header_validation_empty_receiver() {
        let header =
            MessageHeader::new("spirit.alice".to_string(), String::new(), 1234567890.5, 42);
        assert!(header.validate().is_err());
    }

    #[test]
    fn test_message_header_validation_negative_timestamp() {
        let header = MessageHeader::new(
            "spirit.alice".to_string(),
            "spirit.bob".to_string(),
            -123.0,
            42,
        );
        assert!(header.validate().is_err());
    }

    #[test]
    fn test_message_header_validation_invalid_sender_chars() {
        let header = MessageHeader::new(
            "spirit@alice".to_string(),
            "spirit.bob".to_string(),
            1234567890.5,
            42,
        );
        assert!(header.validate().is_err());
    }

    #[test]
    fn test_payload_text_content_type() {
        let payload = MessagePayload::Text("hello".to_string());
        assert_eq!(payload.content_type(), "text/plain");
    }

    #[test]
    fn test_payload_binary_content_type() {
        let payload = MessagePayload::Binary(vec![1, 2, 3]);
        assert_eq!(payload.content_type(), "application/octet-stream");
    }

    #[test]
    fn test_payload_json_content_type() {
        let payload = MessagePayload::Json(serde_json::json!({"key": "value"}));
        assert_eq!(payload.content_type(), "application/json");
    }

    #[test]
    fn test_payload_text_size() {
        let payload = MessagePayload::Text("hello".to_string());
        assert_eq!(payload.size(), 5);
    }

    #[test]
    fn test_payload_binary_size() {
        let payload = MessagePayload::Binary(vec![1, 2, 3, 4, 5]);
        assert_eq!(payload.size(), 5);
    }

    #[test]
    fn test_payload_validation_valid() {
        let payload = MessagePayload::Text("hello".to_string());
        assert!(payload.validate().is_ok());
    }

    #[test]
    fn test_message_creation() {
        let header = MessageHeader::new("alice".to_string(), "bob".to_string(), 1234567890.5, 1);
        let payload = MessagePayload::Text("test".to_string());
        let msg = Message::new(header, payload);

        assert_eq!(msg.header.sender, "alice");
        assert_eq!(msg.header.receiver, "bob");
    }

    #[test]
    fn test_message_serialize_deserialize_text() {
        let header = MessageHeader::new("alice".to_string(), "bob".to_string(), 1234567890.5, 1);
        let payload = MessagePayload::Text("hello".to_string());
        let original = Message::new(header, payload);

        let bytes = original.serialize().unwrap();
        let restored = Message::deserialize(&bytes).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn test_message_serialize_deserialize_binary() {
        let header = MessageHeader::new("alice".to_string(), "bob".to_string(), 1234567890.5, 2);
        let payload = MessagePayload::Binary(vec![0x01, 0x02, 0x03, 0xFF]);
        let original = Message::new(header, payload);

        let bytes = original.serialize().unwrap();
        let restored = Message::deserialize(&bytes).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn test_message_serialize_json_roundtrip() {
        let header = MessageHeader::new("alice".to_string(), "bob".to_string(), 1234567890.5, 3);
        let payload = MessagePayload::Json(serde_json::json!({"name": "test", "value": 42}));
        let original = Message::new(header, payload);

        // Test JSON serialization (bincode doesn't support serde_json::Value)
        let json_str = original.serialize_json().unwrap();
        let restored = Message::deserialize_json(&json_str).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn test_message_serialize_json() {
        let header = MessageHeader::new("alice".to_string(), "bob".to_string(), 1234567890.5, 1);
        let payload = MessagePayload::Text("hello".to_string());
        let msg = Message::new(header, payload);

        let json_str = msg.serialize_json().unwrap();
        assert!(json_str.contains("alice"));
        assert!(json_str.contains("bob"));
        assert!(json_str.contains("hello"));
    }

    #[test]
    fn test_message_deserialize_json() {
        let json_str = r#"{"header":{"sender":"alice","receiver":"bob","timestamp":1234567890.5,"sequence":1},"payload":{"Text":"hello"}}"#;
        let msg = Message::deserialize_json(json_str).unwrap();

        assert_eq!(msg.header.sender, "alice");
        assert_eq!(msg.header.receiver, "bob");
        match msg.payload {
            MessagePayload::Text(s) => assert_eq!(s, "hello"),
            _ => panic!("expected text payload"),
        }
    }

    #[test]
    fn test_message_validation() {
        let header = MessageHeader::new("alice".to_string(), "bob".to_string(), 1234567890.5, 1);
        let payload = MessagePayload::Text("hello".to_string());
        let msg = Message::new(header, payload);

        assert!(msg.validate().is_ok());
    }

    #[test]
    fn test_message_size() {
        let header = MessageHeader::new("alice".to_string(), "bob".to_string(), 1234567890.5, 1);
        let payload = MessagePayload::Text("hello".to_string());
        let msg = Message::new(header, payload);

        let size = msg.size();
        // alice (5) + bob (3) + 8 + 8 + hello (5) = 29 at minimum
        assert!(size > 0);
    }

    #[test]
    fn test_message_deserialize_invalid_data() {
        let invalid_bytes = vec![0xFF, 0xFE, 0xFD];
        assert!(Message::deserialize(&invalid_bytes).is_err());
    }

    #[test]
    fn test_message_with_special_characters_in_ids() {
        let header = MessageHeader::new(
            "spirit.alice-v1".to_string(),
            "spirit.bob_v2".to_string(),
            1234567890.5,
            1,
        );
        assert!(header.validate().is_ok());
    }
}
