//! Integration tests for dol-abi crate
//!
//! These tests verify that all modules work together correctly and that
//! re-exports are properly configured.

use dol_abi::*;

#[test]
fn test_abi_version_constant() {
    assert_eq!(ABI_VERSION, "0.1.0");
}

#[test]
fn test_import_module_constant() {
    assert_eq!(IMPORT_MODULE, "vudo");
}

#[test]
fn test_qualified_id_construction() {
    let id = QualifiedId::new("domain", "property");
    assert_eq!(id.domain, "domain");
    assert_eq!(id.property, "property");
    assert_eq!(id.version, None);
}

#[test]
fn test_qualified_id_with_version() {
    let id = QualifiedId::with_version("domain", "property", "1.0.0");
    assert_eq!(id.domain, "domain");
    assert_eq!(id.property, "property");
    assert_eq!(id.version, Some("1.0.0".to_string()));
}

#[test]
fn test_qualified_id_display() {
    let id1 = QualifiedId::new("domain", "property");
    assert_eq!(id1.to_string(), "domain.property");

    let id2 = QualifiedId::with_version("domain", "property", "1.0.0");
    assert_eq!(id2.to_string(), "domain.property.1.0.0");
}

#[test]
fn test_qualified_id_equality() {
    let id1 = QualifiedId::new("domain", "property");
    let id2 = QualifiedId::new("domain", "property");
    let id3 = QualifiedId::new("other", "property");

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn test_qualified_id_clone() {
    let id1 = QualifiedId::with_version("domain", "property", "1.0.0");
    let id2 = id1.clone();

    assert_eq!(id1, id2);
    assert_eq!(id1.domain, id2.domain);
    assert_eq!(id1.property, id2.property);
    assert_eq!(id1.version, id2.version);
}

#[test]
fn test_qualified_id_serialization() {
    let id = QualifiedId::with_version("test", "prop", "2.0.0");

    // Test that it can be serialized
    let json = serde_json::to_string(&id).expect("Failed to serialize");
    assert!(json.contains("test"));
    assert!(json.contains("prop"));
    assert!(json.contains("2.0.0"));

    // Test that it can be deserialized
    let deserialized: QualifiedId = serde_json::from_str(&json)
        .expect("Failed to deserialize");
    assert_eq!(id, deserialized);
}

#[test]
fn test_message_creation() {
    let msg = message::Message::new("test-id", "test-type", serde_json::json!({"key": "value"}));

    assert_eq!(msg.id, "test-id");
    assert_eq!(msg.msg_type, "test-type");
    assert_eq!(msg.payload["key"], "value");
}

#[test]
fn test_message_serialization() {
    let msg = message::Message::new(
        "msg-123",
        "command",
        serde_json::json!({"action": "execute", "args": [1, 2, 3]})
    );

    let json = serde_json::to_string(&msg).expect("Failed to serialize message");
    let deserialized: message::Message = serde_json::from_str(&json)
        .expect("Failed to deserialize message");

    assert_eq!(msg.id, deserialized.id);
    assert_eq!(msg.msg_type, deserialized.msg_type);
    assert_eq!(msg.payload, deserialized.payload);
}

#[test]
fn test_response_success_creation() {
    let response = message::Response::success("req-123", serde_json::json!({"result": "ok"}));

    assert_eq!(response.id, "req-123");
    assert!(response.success);
    assert_eq!(response.data["result"], "ok");
    assert!(response.error.is_none());
}

#[test]
fn test_response_error_creation() {
    let response = message::Response::error("req-456", "Something went wrong");

    assert_eq!(response.id, "req-456");
    assert!(!response.success);
    assert_eq!(response.error, Some("Something went wrong".to_string()));
    assert_eq!(response.data, serde_json::Value::Null);
}

#[test]
fn test_response_serialization() {
    let success_response = message::Response::success("id-1", serde_json::json!(42));
    let json = serde_json::to_string(&success_response).expect("Failed to serialize");
    let deserialized: message::Response = serde_json::from_str(&json)
        .expect("Failed to deserialize");

    assert_eq!(success_response.id, deserialized.id);
    assert_eq!(success_response.success, deserialized.success);

    let error_response = message::Response::error("id-2", "error message");
    let json = serde_json::to_string(&error_response).expect("Failed to serialize");
    let deserialized: message::Response = serde_json::from_str(&json)
        .expect("Failed to deserialize");

    assert_eq!(error_response.id, deserialized.id);
    assert_eq!(error_response.success, deserialized.success);
    assert_eq!(error_response.error, deserialized.error);
}

#[test]
fn test_error_types_display() {
    let errors = vec![
        Error::InvalidConfig("bad config".to_string()),
        Error::InvalidMessage("bad message".to_string()),
        Error::HostError("host failed".to_string()),
        Error::TypeMismatch("type error".to_string()),
        Error::Other("other error".to_string()),
    ];

    for error in errors {
        let display = error.to_string();
        assert!(!display.is_empty());
    }
}

#[test]
fn test_error_serialization() {
    let error = Error::InvalidConfig("test config error".to_string());

    let json = serde_json::to_string(&error).expect("Failed to serialize error");
    let deserialized: Error = serde_json::from_str(&json)
        .expect("Failed to deserialize error");

    assert!(matches!(deserialized, Error::InvalidConfig(_)));
}

#[test]
fn test_result_type_usage() {
    fn success_function() -> Result<i32> {
        Ok(42)
    }

    fn error_function() -> Result<i32> {
        Err(Error::Other("failed".to_string()))
    }

    assert!(success_function().is_ok());
    assert_eq!(success_function().unwrap(), 42);

    assert!(error_function().is_err());
}

#[test]
fn test_error_type_implements_std_error() {
    let error: Box<dyn std::error::Error> = Box::new(Error::InvalidConfig("test".to_string()));
    assert!(!error.to_string().is_empty());
}

#[test]
fn test_message_response_roundtrip() {
    // Create a message, process it, and respond
    let request = message::Message::new(
        "request-1",
        "query",
        serde_json::json!({"query": "SELECT * FROM users"})
    );

    // Simulate processing
    let response = if request.msg_type == "query" {
        message::Response::success(
            request.id.clone(),
            serde_json::json!({"rows": 10, "data": []})
        )
    } else {
        message::Response::error(request.id.clone(), "Unknown message type")
    };

    assert_eq!(request.id, response.id);
    assert!(response.success);
}

#[test]
fn test_complex_payload_handling() {
    let complex_payload = serde_json::json!({
        "user": {
            "id": 123,
            "name": "Alice",
            "roles": ["admin", "user"]
        },
        "timestamp": 1234567890,
        "metadata": {
            "version": "1.0",
            "flags": {
                "enabled": true,
                "priority": 5
            }
        }
    });

    let msg = message::Message::new("complex-1", "user_update", complex_payload);
    let json = serde_json::to_string(&msg).expect("Serialization failed");
    let deserialized: message::Message = serde_json::from_str(&json)
        .expect("Deserialization failed");

    assert_eq!(msg.id, deserialized.id);
    assert_eq!(msg.payload["user"]["name"], "Alice");
    assert_eq!(deserialized.payload["metadata"]["flags"]["enabled"], true);
}

#[test]
fn test_error_clone() {
    let error1 = Error::HostError("test error".to_string());
    let error2 = error1.clone();

    match (&error1, &error2) {
        (Error::HostError(msg1), Error::HostError(msg2)) => {
            assert_eq!(msg1, msg2);
        }
        _ => panic!("Error types don't match"),
    }
}

#[test]
fn test_type_conversions() {
    // Test that types can be converted as expected
    let id = QualifiedId::new("test", "prop");
    let id_string = id.to_string();
    assert!(id_string.contains("test"));
    assert!(id_string.contains("prop"));

    // Test Result type usage
    let result: Result<String> = Ok("success".to_string());
    assert!(result.is_ok());

    let result: Result<String> = Err(Error::Other("failure".to_string()));
    assert!(result.is_err());
}

#[test]
fn test_message_type_string_conversion() {
    let msg = message::Message::new(
        String::from("id"),
        String::from("type"),
        serde_json::json!(null)
    );
    assert_eq!(msg.id, "id");
    assert_eq!(msg.msg_type, "type");
}

#[test]
fn test_response_id_string_conversion() {
    let resp = message::Response::success(String::from("response-id"), serde_json::json!(true));
    assert_eq!(resp.id, "response-id");

    let err = message::Response::error(String::from("error-id"), String::from("error"));
    assert_eq!(err.id, "error-id");
}

#[test]
fn test_null_payload_handling() {
    let msg = message::Message::new("null-test", "test", serde_json::Value::Null);
    assert_eq!(msg.payload, serde_json::Value::Null);

    let json = serde_json::to_string(&msg).expect("Serialization failed");
    let deserialized: message::Message = serde_json::from_str(&json)
        .expect("Deserialization failed");
    assert_eq!(deserialized.payload, serde_json::Value::Null);
}

#[test]
fn test_empty_strings() {
    let id = QualifiedId::new("", "");
    assert_eq!(id.domain, "");
    assert_eq!(id.property, "");
    assert_eq!(id.to_string(), ".");

    let msg = message::Message::new("", "", serde_json::Value::Null);
    assert_eq!(msg.id, "");
    assert_eq!(msg.msg_type, "");
}

#[test]
fn test_multiple_qualified_ids_with_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(QualifiedId::new("a", "b"));
    set.insert(QualifiedId::new("c", "d"));
    set.insert(QualifiedId::new("a", "b")); // Duplicate

    assert_eq!(set.len(), 2);
    assert!(set.contains(&QualifiedId::new("a", "b")));
    assert!(set.contains(&QualifiedId::new("c", "d")));
}

#[test]
fn test_module_re_exports() {
    // Verify that types are properly re-exported from lib.rs
    let _error: Error = Error::Other("test".to_string());
    let _id: QualifiedId = QualifiedId::new("test", "prop");

    // This test ensures the re-exports in lib.rs work correctly
    assert!(true);
}
