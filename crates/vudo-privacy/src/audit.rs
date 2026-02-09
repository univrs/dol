//! Audit trail for GDPR deletion requests.
//!
//! This module provides comprehensive audit logging for all GDPR deletion
//! requests, as required by data protection regulations. The audit log proves
//! compliance and provides an immutable record of all deletion operations.
//!
//! # Requirements
//!
//! Under GDPR, organizations must maintain records of:
//! - What data was deleted
//! - When it was deleted
//! - Who requested the deletion
//! - How it was deleted (method)
//! - Proof that deletion occurred
//!
//! # Architecture
//!
//! The audit log is append-only and includes cryptographic proofs (deletion
//! receipts) when cryptographic deletion is used.
//!
//! # Example
//!
//! ```rust
//! use vudo_privacy::audit::{DeletionAuditLog, DataCategory, DeletionMethod};
//!
//! # fn example() -> vudo_privacy::error::Result<()> {
//! let mut audit_log = DeletionAuditLog::new();
//!
//! // Record a deletion
//! let request_id = audit_log.record_deletion(
//!     "did:peer:alice",
//!     vec![DataCategory::PersonalData],
//!     DeletionMethod::CryptographicErasure,
//!     None, // Could include deletion receipt
//! );
//!
//! // Query audit log
//! let entries = audit_log.get_entries_for_user("did:peer:alice");
//! assert_eq!(entries.len(), 1);
//! # Ok(())
//! # }
//! ```

use crate::crypto::DeletionReceipt;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;

/// Category of data being deleted.
///
/// Different data categories may have different legal retention requirements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataCategory {
    /// Personal data encrypted with DEK (can be cryptographically deleted).
    PersonalData,

    /// Public data (can be tombstoned in Willow).
    PublicData,

    /// Transaction history (may need to be retained for tax/legal reasons).
    TransactionHistory,

    /// Account metadata (usernames, public profiles).
    AccountMetadata,

    /// System logs (may need to be anonymized instead of deleted).
    SystemLogs,
}

/// Method used for data deletion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeletionMethod {
    /// Cryptographic erasure (DEK deletion).
    CryptographicErasure,

    /// Willow true-deletion (tombstone).
    Tombstone,

    /// Anonymization (replace DID with pseudonym, retain data).
    Anonymization,

    /// Physical deletion (not recommended for CRDTs).
    PhysicalDeletion,
}

/// Audit log entry for a deletion request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionLogEntry {
    /// Unique request ID.
    pub request_id: String,

    /// User DID.
    pub user_did: String,

    /// Deletion timestamp (Unix seconds).
    pub deleted_at: u64,

    /// Data categories deleted.
    pub categories: Vec<DataCategory>,

    /// Deletion method used.
    pub method: DeletionMethod,

    /// Cryptographic proof (DEK deletion receipt).
    pub proof: Option<DeletionReceipt>,

    /// Optional notes or rationale.
    pub notes: Option<String>,
}

impl DeletionLogEntry {
    /// Create a new deletion log entry.
    pub fn new(
        user_did: String,
        categories: Vec<DataCategory>,
        method: DeletionMethod,
        proof: Option<DeletionReceipt>,
    ) -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            user_did,
            deleted_at: Utc::now().timestamp() as u64,
            categories,
            method,
            proof,
            notes: None,
        }
    }

    /// Add notes to this entry.
    pub fn with_notes(mut self, notes: String) -> Self {
        self.notes = Some(notes);
        self
    }

    /// Check if this entry has cryptographic proof.
    pub fn has_proof(&self) -> bool {
        self.proof.is_some()
    }

    /// Get age of deletion in seconds.
    pub fn age_seconds(&self) -> u64 {
        let now = Utc::now().timestamp() as u64;
        now.saturating_sub(self.deleted_at)
    }
}

/// Audit log for GDPR deletion requests.
///
/// Provides an append-only, immutable record of all deletion operations.
/// This is critical for GDPR compliance and regulatory audits.
pub struct DeletionAuditLog {
    /// Log entries (append-only).
    logs: Arc<RwLock<Vec<DeletionLogEntry>>>,
}

impl DeletionAuditLog {
    /// Create a new empty audit log.
    pub fn new() -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record a deletion operation.
    ///
    /// # Arguments
    ///
    /// * `user_did` - The DID of the user whose data was deleted
    /// * `categories` - The categories of data that were deleted
    /// * `method` - The method used for deletion
    /// * `proof` - Optional cryptographic proof (deletion receipt)
    ///
    /// # Returns
    ///
    /// The unique request ID for this deletion.
    pub fn record_deletion(
        &self,
        user_did: &str,
        categories: Vec<DataCategory>,
        method: DeletionMethod,
        proof: Option<DeletionReceipt>,
    ) -> String {
        let entry = DeletionLogEntry::new(
            user_did.to_string(),
            categories,
            method,
            proof,
        );

        let request_id = entry.request_id.clone();
        self.logs.write().push(entry);
        request_id
    }

    /// Get all log entries.
    pub fn get_all_entries(&self) -> Vec<DeletionLogEntry> {
        self.logs.read().clone()
    }

    /// Get log entries for a specific user.
    ///
    /// # Arguments
    ///
    /// * `user_did` - The DID of the user
    ///
    /// # Returns
    ///
    /// All deletion log entries for this user.
    pub fn get_entries_for_user(&self, user_did: &str) -> Vec<DeletionLogEntry> {
        self.logs
            .read()
            .iter()
            .filter(|entry| entry.user_did == user_did)
            .cloned()
            .collect()
    }

    /// Get a specific log entry by request ID.
    pub fn get_entry(&self, request_id: &str) -> Option<DeletionLogEntry> {
        self.logs
            .read()
            .iter()
            .find(|entry| entry.request_id == request_id)
            .cloned()
    }

    /// Get entries by deletion method.
    pub fn get_entries_by_method(&self, method: DeletionMethod) -> Vec<DeletionLogEntry> {
        self.logs
            .read()
            .iter()
            .filter(|entry| entry.method == method)
            .cloned()
            .collect()
    }

    /// Get entries by data category.
    pub fn get_entries_by_category(&self, category: DataCategory) -> Vec<DeletionLogEntry> {
        self.logs
            .read()
            .iter()
            .filter(|entry| entry.categories.contains(&category))
            .cloned()
            .collect()
    }

    /// Get total number of deletions.
    pub fn total_deletions(&self) -> usize {
        self.logs.read().len()
    }

    /// Export audit log to JSON.
    pub fn export_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&*self.logs.read())
    }

    /// Import audit log from JSON.
    pub fn import_json(json: &str) -> serde_json::Result<Self> {
        let entries: Vec<DeletionLogEntry> = serde_json::from_str(json)?;
        Ok(Self {
            logs: Arc::new(RwLock::new(entries)),
        })
    }

    /// Clear all entries (for testing only).
    #[cfg(test)]
    pub fn clear(&self) {
        self.logs.write().clear();
    }
}

impl Default for DeletionAuditLog {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for DeletionAuditLog {
    fn clone(&self) -> Self {
        Self {
            logs: Arc::clone(&self.logs),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_deletion() {
        let audit_log = DeletionAuditLog::new();

        let request_id = audit_log.record_deletion(
            "did:peer:alice",
            vec![DataCategory::PersonalData],
            DeletionMethod::CryptographicErasure,
            None,
        );

        assert!(!request_id.is_empty());
        assert_eq!(audit_log.total_deletions(), 1);
    }

    #[test]
    fn test_get_entries_for_user() {
        let audit_log = DeletionAuditLog::new();

        audit_log.record_deletion(
            "did:peer:alice",
            vec![DataCategory::PersonalData],
            DeletionMethod::CryptographicErasure,
            None,
        );

        audit_log.record_deletion(
            "did:peer:bob",
            vec![DataCategory::PublicData],
            DeletionMethod::Tombstone,
            None,
        );

        let alice_entries = audit_log.get_entries_for_user("did:peer:alice");
        assert_eq!(alice_entries.len(), 1);
        assert_eq!(alice_entries[0].user_did, "did:peer:alice");
    }

    #[test]
    fn test_get_entry_by_id() {
        let audit_log = DeletionAuditLog::new();

        let request_id = audit_log.record_deletion(
            "did:peer:alice",
            vec![DataCategory::PersonalData],
            DeletionMethod::CryptographicErasure,
            None,
        );

        let entry = audit_log.get_entry(&request_id);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().request_id, request_id);
    }

    #[test]
    fn test_get_entries_by_method() {
        let audit_log = DeletionAuditLog::new();

        audit_log.record_deletion(
            "did:peer:alice",
            vec![DataCategory::PersonalData],
            DeletionMethod::CryptographicErasure,
            None,
        );

        audit_log.record_deletion(
            "did:peer:bob",
            vec![DataCategory::PublicData],
            DeletionMethod::Tombstone,
            None,
        );

        let crypto_entries = audit_log.get_entries_by_method(DeletionMethod::CryptographicErasure);
        assert_eq!(crypto_entries.len(), 1);
    }

    #[test]
    fn test_export_import_json() {
        let audit_log = DeletionAuditLog::new();

        audit_log.record_deletion(
            "did:peer:alice",
            vec![DataCategory::PersonalData],
            DeletionMethod::CryptographicErasure,
            None,
        );

        let json = audit_log.export_json().unwrap();
        let imported = DeletionAuditLog::import_json(&json).unwrap();

        assert_eq!(imported.total_deletions(), 1);
    }
}
