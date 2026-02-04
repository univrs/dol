//! Common ABI types for Metal DOL.
//!
//! This module defines shared types used across the DOL ecosystem,
//! including logging levels, result codes, and effect tracking.
//!
//! These types are essential for interoperability between DOL-generated
//! code and runtime systems.
//!
//! # Example
//!
//! ```rust
//! use metadol::types::{LogLevel, ResultCode, StandardEffect};
//!
//! // Create a log level
//! let level = LogLevel::Info;
//! assert_eq!(level.as_str(), "INFO");
//!
//! // Create a result code
//! let result = ResultCode::Success;
//! assert_eq!(result as u32, 0);
//!
//! // Create an effect (requires serde feature)
//! # #[cfg(feature = "serde")]
//! # {
//! # use serde_json::json;
//! let effect = StandardEffect {
//!     effect_type: "DOM_UPDATE".to_string(),
//!     payload: json!({"selector": "#app", "content": "Hello"}),
//!     timestamp: 1234567890.0,
//! };
//! # }
//! ```

use std::convert::TryFrom;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
use serde_json;

/// Logging level enumeration for DOL runtime diagnostics.
///
/// The logging level controls which diagnostic messages are recorded
/// and emitted by the DOL runtime system.
///
/// # Variant Ordering
///
/// Variants are ordered by severity, from least to most severe:
/// - `Debug` (0): Detailed diagnostic information for development
/// - `Info` (1): General informational messages
/// - `Warn` (2): Warning messages for potentially problematic conditions
/// - `Error` (3): Error messages for failures
///
/// # Examples
///
/// ```rust
/// use metadol::types::LogLevel;
///
/// let debug_level = LogLevel::Debug;
/// assert_eq!(debug_level as u32, 0);
/// assert_eq!(debug_level.as_str(), "DEBUG");
///
/// let error_level = LogLevel::Error;
/// assert_eq!(error_level as u32, 3);
/// assert_eq!(error_level.as_str(), "ERROR");
///
/// // Convert from u32
/// let level: LogLevel = 1.into();
/// assert_eq!(level, LogLevel::Info);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "UPPERCASE"))]
pub enum LogLevel {
    /// Debug level (0) - Detailed diagnostic information
    Debug = 0,
    /// Info level (1) - General informational messages
    Info = 1,
    /// Warn level (2) - Warning messages
    Warn = 2,
    /// Error level (3) - Error messages
    Error = 3,
}

impl LogLevel {
    /// Convert LogLevel to its string representation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use metadol::types::LogLevel;
    ///
    /// assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
    /// assert_eq!(LogLevel::Info.as_str(), "INFO");
    /// assert_eq!(LogLevel::Warn.as_str(), "WARN");
    /// assert_eq!(LogLevel::Error.as_str(), "ERROR");
    /// ```
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }

    /// Check if this level is at least as severe as another level.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use metadol::types::LogLevel;
    ///
    /// assert!(LogLevel::Error.is_at_least(LogLevel::Warn));
    /// assert!(LogLevel::Warn.is_at_least(LogLevel::Warn));
    /// assert!(!LogLevel::Info.is_at_least(LogLevel::Warn));
    /// ```
    #[inline]
    pub fn is_at_least(self, other: LogLevel) -> bool {
        self as u32 >= other as u32
    }
}

impl From<u32> for LogLevel {
    /// Convert a u32 to LogLevel.
    ///
    /// Values outside the range [0, 3] will be clamped:
    /// - Values < 0 (impossible with u32) → Debug
    /// - Values 0 → Debug
    /// - Values 1 → Info
    /// - Values 2 → Warn
    /// - Values 3+ → Error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use metadol::types::LogLevel;
    ///
    /// assert_eq!(LogLevel::from(0u32), LogLevel::Debug);
    /// assert_eq!(LogLevel::from(2u32), LogLevel::Warn);
    /// assert_eq!(LogLevel::from(5u32), LogLevel::Error); // Clamped
    /// ```
    fn from(value: u32) -> Self {
        match value {
            0 => LogLevel::Debug,
            1 => LogLevel::Info,
            2 => LogLevel::Warn,
            _ => LogLevel::Error,
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Result code enumeration for DOL operation outcomes.
///
/// Represents the outcome of operations executed by DOL runtime systems,
/// including success, failure, pending, and timeout states.
///
/// # Variant Ordering
///
/// - `Success` (0): Operation completed successfully
/// - `Error` (1): Operation failed with an error
/// - `Pending` (2): Operation is still pending
/// - `Timeout` (3): Operation timed out before completion
///
/// # Examples
///
/// ```rust
/// use metadol::types::ResultCode;
///
/// let success = ResultCode::Success;
/// assert_eq!(success as u32, 0);
///
/// let error: ResultCode = 1.into();
/// assert_eq!(error, ResultCode::Error);
///
/// let timeout: ResultCode = 3.into();
/// assert_eq!(timeout, ResultCode::Timeout);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "UPPERCASE"))]
pub enum ResultCode {
    /// Success (0) - Operation completed successfully
    Success = 0,
    /// Error (1) - Operation failed with an error
    Error = 1,
    /// Pending (2) - Operation is still pending
    Pending = 2,
    /// Timeout (3) - Operation timed out
    Timeout = 3,
}

impl ResultCode {
    /// Check if this result represents success.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use metadol::types::ResultCode;
    ///
    /// assert!(ResultCode::Success.is_success());
    /// assert!(!ResultCode::Error.is_success());
    /// assert!(!ResultCode::Pending.is_success());
    /// ```
    #[inline]
    pub fn is_success(self) -> bool {
        self == ResultCode::Success
    }

    /// Check if this result represents an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use metadol::types::ResultCode;
    ///
    /// assert!(ResultCode::Error.is_error());
    /// assert!(!ResultCode::Success.is_error());
    /// ```
    #[inline]
    pub fn is_error(self) -> bool {
        self == ResultCode::Error
    }

    /// Check if this result indicates the operation is still pending.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use metadol::types::ResultCode;
    ///
    /// assert!(ResultCode::Pending.is_pending());
    /// assert!(!ResultCode::Success.is_pending());
    /// ```
    #[inline]
    pub fn is_pending(self) -> bool {
        self == ResultCode::Pending
    }

    /// Check if this result indicates a timeout.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use metadol::types::ResultCode;
    ///
    /// assert!(ResultCode::Timeout.is_timeout());
    /// assert!(!ResultCode::Success.is_timeout());
    /// ```
    #[inline]
    pub fn is_timeout(self) -> bool {
        self == ResultCode::Timeout
    }

    /// Convert ResultCode to its string representation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use metadol::types::ResultCode;
    ///
    /// assert_eq!(ResultCode::Success.as_str(), "SUCCESS");
    /// assert_eq!(ResultCode::Error.as_str(), "ERROR");
    /// assert_eq!(ResultCode::Pending.as_str(), "PENDING");
    /// assert_eq!(ResultCode::Timeout.as_str(), "TIMEOUT");
    /// ```
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            ResultCode::Success => "SUCCESS",
            ResultCode::Error => "ERROR",
            ResultCode::Pending => "PENDING",
            ResultCode::Timeout => "TIMEOUT",
        }
    }
}

impl From<u32> for ResultCode {
    /// Convert a u32 to ResultCode.
    ///
    /// Values outside the range [0, 3] will be clamped:
    /// - Values 0 → Success
    /// - Values 1 → Error
    /// - Values 2 → Pending
    /// - Values 3+ → Timeout
    ///
    /// # Examples
    ///
    /// ```rust
    /// use metadol::types::ResultCode;
    ///
    /// assert_eq!(ResultCode::from(0u32), ResultCode::Success);
    /// assert_eq!(ResultCode::from(2u32), ResultCode::Pending);
    /// assert_eq!(ResultCode::from(10u32), ResultCode::Timeout); // Clamped
    /// ```
    fn from(value: u32) -> Self {
        match value {
            0 => ResultCode::Success,
            1 => ResultCode::Error,
            2 => ResultCode::Pending,
            _ => ResultCode::Timeout,
        }
    }
}

impl std::fmt::Display for ResultCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Standard effect descriptor for DOL side-effects (requires `serde` feature).
///
/// Describes an effect that occurred in the DOL runtime,
/// including its type, associated payload, and timestamp.
///
/// Effects are tracked to understand and validate the side-effects
/// produced by DOL-generated code.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "serde")]
/// # {
/// use metadol::types::StandardEffect;
/// use serde_json::json;
///
/// let effect = StandardEffect {
///     effect_type: "DOM_UPDATE".to_string(),
///     payload: json!({
///         "selector": "#app",
///         "content": "Hello, World!"
///     }),
///     timestamp: 1234567890.123,
/// };
///
/// assert_eq!(effect.effect_type, "DOM_UPDATE");
/// assert_eq!(effect.timestamp, 1234567890.123);
/// # }
/// ```
#[cfg(feature = "serde")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StandardEffect {
    /// The type of effect (e.g., "DOM_UPDATE", "HTTP_REQUEST", "FILE_WRITE").
    ///
    /// This categorizes the kind of side-effect that occurred.
    pub effect_type: String,

    /// The effect payload containing operation-specific data.
    ///
    /// The structure and content of this value depends on the `effect_type`.
    /// For example, a DOM_UPDATE effect might contain selector and content properties.
    #[serde(default)]
    pub payload: serde_json::Value,

    /// The timestamp when the effect occurred (seconds since epoch).
    ///
    /// This is a floating-point number to support sub-second precision.
    pub timestamp: f64,
}

#[cfg(feature = "serde")]
impl StandardEffect {
    /// Create a new StandardEffect with the given type and timestamp.
    ///
    /// The payload is initialized to `null`.
    ///
    /// # Arguments
    ///
    /// * `effect_type` - The type of effect
    /// * `timestamp` - The timestamp when the effect occurred
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "serde")]
    /// # {
    /// use metadol::types::StandardEffect;
    /// use serde_json::json;
    ///
    /// let effect = StandardEffect::new("LOG".to_string(), 1234567890.0);
    /// assert_eq!(effect.effect_type, "LOG");
    /// assert_eq!(effect.timestamp, 1234567890.0);
    /// assert_eq!(effect.payload, json!(null));
    /// # }
    /// ```
    pub fn new(effect_type: String, timestamp: f64) -> Self {
        StandardEffect {
            effect_type,
            payload: serde_json::Value::Null,
            timestamp,
        }
    }

    /// Create a new StandardEffect with the given type, payload, and timestamp.
    ///
    /// # Arguments
    ///
    /// * `effect_type` - The type of effect
    /// * `payload` - The effect payload
    /// * `timestamp` - The timestamp when the effect occurred
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "serde")]
    /// # {
    /// use metadol::types::StandardEffect;
    /// use serde_json::json;
    ///
    /// let effect = StandardEffect::with_payload(
    ///     "HTTP_REQUEST".to_string(),
    ///     json!({"method": "POST", "url": "https://example.com"}),
    ///     1234567890.123,
    /// );
    /// assert_eq!(effect.effect_type, "HTTP_REQUEST");
    /// assert_eq!(effect.timestamp, 1234567890.123);
    /// # }
    /// ```
    pub fn with_payload(effect_type: String, payload: serde_json::Value, timestamp: f64) -> Self {
        StandardEffect {
            effect_type,
            payload,
            timestamp,
        }
    }

    /// Get a reference to the effect payload as a JSON value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "serde")]
    /// # {
    /// use metadol::types::StandardEffect;
    /// use serde_json::json;
    ///
    /// let effect = StandardEffect::with_payload(
    ///     "UPDATE".to_string(),
    ///     json!({"key": "value"}),
    ///     0.0,
    /// );
    /// assert_eq!(effect.get_payload().get("key").and_then(|v| v.as_str()), Some("value"));
    /// # }
    /// ```
    #[inline]
    pub fn get_payload(&self) -> &serde_json::Value {
        &self.payload
    }

    /// Get a mutable reference to the effect payload.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "serde")]
    /// # {
    /// use metadol::types::StandardEffect;
    /// use serde_json::json;
    ///
    /// let mut effect = StandardEffect::new("UPDATE".to_string(), 0.0);
    /// if let Some(obj) = effect.get_payload_mut().as_object_mut() {
    ///     obj.insert("key".to_string(), json!("value"));
    /// }
    /// # }
    /// ```
    #[inline]
    pub fn get_payload_mut(&mut self) -> &mut serde_json::Value {
        &mut self.payload
    }
}

#[cfg(feature = "serde")]
impl std::fmt::Display for StandardEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}@{} with payload: {}",
            self.effect_type, self.timestamp, self.payload
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        assert_eq!(LogLevel::Debug as u32, 0);
        assert_eq!(LogLevel::Info as u32, 1);
        assert_eq!(LogLevel::Warn as u32, 2);
        assert_eq!(LogLevel::Error as u32, 3);
    }

    #[test]
    fn test_log_level_as_str() {
        assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Warn.as_str(), "WARN");
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
    }

    #[test]
    fn test_log_level_from_u32() {
        assert_eq!(LogLevel::from(0u32), LogLevel::Debug);
        assert_eq!(LogLevel::from(1u32), LogLevel::Info);
        assert_eq!(LogLevel::from(2u32), LogLevel::Warn);
        assert_eq!(LogLevel::from(3u32), LogLevel::Error);
        assert_eq!(LogLevel::from(100u32), LogLevel::Error); // Clamped
    }

    #[test]
    fn test_log_level_is_at_least() {
        assert!(LogLevel::Error.is_at_least(LogLevel::Warn));
        assert!(LogLevel::Warn.is_at_least(LogLevel::Warn));
        assert!(!LogLevel::Info.is_at_least(LogLevel::Warn));
        assert!(!LogLevel::Debug.is_at_least(LogLevel::Info));
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Debug.to_string(), "DEBUG");
        assert_eq!(LogLevel::Error.to_string(), "ERROR");
    }

    #[test]
    fn test_result_code_ordering() {
        assert_eq!(ResultCode::Success as u32, 0);
        assert_eq!(ResultCode::Error as u32, 1);
        assert_eq!(ResultCode::Pending as u32, 2);
        assert_eq!(ResultCode::Timeout as u32, 3);
    }

    #[test]
    fn test_result_code_as_str() {
        assert_eq!(ResultCode::Success.as_str(), "SUCCESS");
        assert_eq!(ResultCode::Error.as_str(), "ERROR");
        assert_eq!(ResultCode::Pending.as_str(), "PENDING");
        assert_eq!(ResultCode::Timeout.as_str(), "TIMEOUT");
    }

    #[test]
    fn test_result_code_from_u32() {
        assert_eq!(ResultCode::from(0u32), ResultCode::Success);
        assert_eq!(ResultCode::from(1u32), ResultCode::Error);
        assert_eq!(ResultCode::from(2u32), ResultCode::Pending);
        assert_eq!(ResultCode::from(3u32), ResultCode::Timeout);
        assert_eq!(ResultCode::from(100u32), ResultCode::Timeout); // Clamped
    }

    #[test]
    fn test_result_code_predicates() {
        assert!(ResultCode::Success.is_success());
        assert!(!ResultCode::Error.is_success());

        assert!(ResultCode::Error.is_error());
        assert!(!ResultCode::Success.is_error());

        assert!(ResultCode::Pending.is_pending());
        assert!(!ResultCode::Success.is_pending());

        assert!(ResultCode::Timeout.is_timeout());
        assert!(!ResultCode::Success.is_timeout());
    }

    #[test]
    fn test_result_code_display() {
        assert_eq!(ResultCode::Success.to_string(), "SUCCESS");
        assert_eq!(ResultCode::Timeout.to_string(), "TIMEOUT");
    }

    #[test]
    fn test_standard_effect_new() {
        let effect = StandardEffect::new("TEST".to_string(), 123.456);
        assert_eq!(effect.effect_type, "TEST");
        assert_eq!(effect.timestamp, 123.456);
        assert_eq!(effect.payload, serde_json::Value::Null);
    }

    #[test]
    fn test_standard_effect_with_payload() {
        let payload = serde_json::json!({"key": "value"});
        let effect = StandardEffect::with_payload("TEST".to_string(), payload.clone(), 789.0);
        assert_eq!(effect.effect_type, "TEST");
        assert_eq!(effect.timestamp, 789.0);
        assert_eq!(effect.payload, payload);
    }

    #[test]
    fn test_standard_effect_getters() {
        let effect = StandardEffect::with_payload(
            "UPDATE".to_string(),
            serde_json::json!({"data": 42}),
            1.0,
        );
        assert_eq!(
            effect.get_payload().get("data").and_then(|v| v.as_u64()),
            Some(42)
        );
    }

    #[test]
    fn test_standard_effect_mutable_access() {
        let mut effect = StandardEffect::new("TEST".to_string(), 0.0);
        if let Some(obj) = effect.get_payload_mut().as_object_mut() {
            obj.insert("new_key".to_string(), serde_json::json!("new_value"));
        }
        assert_eq!(
            effect.get_payload().get("new_key").and_then(|v| v.as_str()),
            Some("new_value")
        );
    }

    #[test]
    fn test_standard_effect_display() {
        let effect =
            StandardEffect::with_payload("DOM".to_string(), serde_json::json!({"test": 1}), 5.0);
        let display_str = effect.to_string();
        assert!(display_str.contains("DOM"));
        assert!(display_str.contains("5"));
    }

    #[test]
    fn test_standard_effect_clone() {
        let effect = StandardEffect::with_payload(
            "CLONE_TEST".to_string(),
            serde_json::json!({"x": 10}),
            99.9,
        );
        let cloned = effect.clone();
        assert_eq!(cloned, effect);
        assert_eq!(cloned.effect_type, "CLONE_TEST");
    }
}
