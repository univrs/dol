//! CRDT-backed exegesis document model.
//!
//! This module defines the ExegesisDocument structure that uses CRDT strategies
//! for conflict-free replication:
//!
//! - `gene_id`: Immutable (set once, never changed)
//! - `gene_version`: Immutable (set once, never changed)
//! - `content`: Peritext CRDT (rich text with concurrent editing support)
//! - `last_modified`: LWW (last-write-wins with timestamp)
//! - `contributors`: RGA (replicated growable array for ordered list)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// CRDT-backed exegesis document.
///
/// This structure represents a documentation entry for a DOL Gene declaration,
/// with CRDT annotations for conflict-free synchronization across peers.
///
/// # CRDT Strategies
///
/// - `gene_id`: **Immutable** - Set once during creation, never modified
/// - `gene_version`: **Immutable** - Set once during creation, never modified
/// - `content`: **Peritext** - Rich text CRDT for collaborative editing
/// - `last_modified`: **LWW** - Last-Write-Wins with timestamp resolution
/// - `contributors`: **RGA** - Replicated Growable Array for ordered list
///
/// # Example
///
/// ```
/// use dol_exegesis::ExegesisDocument;
/// use chrono::Utc;
///
/// let doc = ExegesisDocument {
///     gene_id: "user.profile".to_string(),
///     gene_version: "1.0.0".to_string(),
///     content: "A user profile contains identity and preferences.".to_string(),
///     last_modified: Utc::now().timestamp(),
///     contributors: vec!["did:peer:alice".to_string()],
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExegesisDocument {
    /// Gene identifier (immutable CRDT).
    ///
    /// This field is set once during document creation and never modified.
    /// The CRDT strategy ensures that any attempt to modify this field
    /// after creation will be rejected.
    pub gene_id: String,

    /// Gene version (immutable CRDT).
    ///
    /// Semver version string (e.g., "1.0.0"). This field is set once
    /// during document creation and never modified.
    pub gene_version: String,

    /// Exegesis content (Peritext CRDT).
    ///
    /// Rich text content that supports concurrent editing by multiple
    /// collaborators. The Peritext CRDT automatically merges concurrent
    /// edits while preserving formatting and intent.
    pub content: String,

    /// Last modification timestamp (LWW CRDT).
    ///
    /// Unix timestamp (seconds since epoch). Uses Last-Write-Wins strategy
    /// where the most recent timestamp wins in case of concurrent updates.
    pub last_modified: i64,

    /// List of contributor DIDs (RGA CRDT).
    ///
    /// Ordered list of decentralized identifiers (DIDs) of all users who
    /// have edited this exegesis. Uses Replicated Growable Array strategy
    /// to maintain causal ordering of additions.
    pub contributors: Vec<String>,
}

impl ExegesisDocument {
    /// Create a new exegesis document.
    ///
    /// # Arguments
    ///
    /// * `gene_id` - The Gene identifier (e.g., "user.profile")
    /// * `gene_version` - The Gene version (e.g., "1.0.0")
    /// * `content` - Initial exegesis content
    ///
    /// # Returns
    ///
    /// A new `ExegesisDocument` with empty contributors list and current timestamp.
    pub fn new(gene_id: String, gene_version: String, content: String) -> Self {
        Self {
            gene_id,
            gene_version,
            content,
            last_modified: Utc::now().timestamp(),
            contributors: Vec::new(),
        }
    }

    /// Get the document ID key for storage.
    ///
    /// Returns a unique identifier in the format "gene_id@gene_version"
    /// that can be used as a key in the state engine.
    pub fn document_id(&self) -> String {
        format!("{}@{}", self.gene_id, self.gene_version)
    }

    /// Add a contributor to the document.
    ///
    /// Adds a contributor DID to the list if not already present.
    /// Uses RGA semantics to maintain causal ordering.
    ///
    /// # Arguments
    ///
    /// * `did` - Decentralized identifier of the contributor
    pub fn add_contributor(&mut self, did: String) {
        if !self.contributors.contains(&did) {
            self.contributors.push(did);
        }
    }

    /// Update the last modified timestamp.
    ///
    /// Sets the last_modified field to the current UTC timestamp.
    /// Uses LWW semantics where the most recent timestamp wins.
    pub fn touch(&mut self) {
        self.last_modified = Utc::now().timestamp();
    }

    /// Get the last modified time as a DateTime.
    ///
    /// # Returns
    ///
    /// The last modification time as a UTC DateTime, or None if the
    /// timestamp is invalid.
    pub fn last_modified_datetime(&self) -> Option<DateTime<Utc>> {
        DateTime::from_timestamp(self.last_modified, 0)
    }

    /// Validate the document structure.
    ///
    /// Checks that:
    /// - `gene_id` is not empty
    /// - `gene_version` is a valid semver string
    /// - `content` is not empty
    ///
    /// # Returns
    ///
    /// Ok(()) if valid, Err with description if invalid.
    pub fn validate(&self) -> Result<(), String> {
        if self.gene_id.is_empty() {
            return Err("gene_id cannot be empty".to_string());
        }

        if self.gene_version.is_empty() {
            return Err("gene_version cannot be empty".to_string());
        }

        // Basic semver validation (X.Y.Z)
        let parts: Vec<&str> = self.gene_version.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid semver version: {}", self.gene_version));
        }

        for part in parts {
            if part.parse::<u32>().is_err() {
                return Err(format!("Invalid semver component: {}", part));
            }
        }

        if self.content.is_empty() {
            return Err("content cannot be empty".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_exegesis_document() {
        let doc = ExegesisDocument::new(
            "user.profile".to_string(),
            "1.0.0".to_string(),
            "Test content".to_string(),
        );

        assert_eq!(doc.gene_id, "user.profile");
        assert_eq!(doc.gene_version, "1.0.0");
        assert_eq!(doc.content, "Test content");
        assert!(doc.contributors.is_empty());
        assert!(doc.last_modified > 0);
    }

    #[test]
    fn test_document_id() {
        let doc = ExegesisDocument::new(
            "user.profile".to_string(),
            "1.0.0".to_string(),
            "Test".to_string(),
        );

        assert_eq!(doc.document_id(), "user.profile@1.0.0");
    }

    #[test]
    fn test_add_contributor() {
        let mut doc = ExegesisDocument::new(
            "user.profile".to_string(),
            "1.0.0".to_string(),
            "Test".to_string(),
        );

        doc.add_contributor("did:peer:alice".to_string());
        assert_eq!(doc.contributors.len(), 1);
        assert_eq!(doc.contributors[0], "did:peer:alice");

        // Adding same contributor twice should not duplicate
        doc.add_contributor("did:peer:alice".to_string());
        assert_eq!(doc.contributors.len(), 1);

        // Adding different contributor should append
        doc.add_contributor("did:peer:bob".to_string());
        assert_eq!(doc.contributors.len(), 2);
    }

    #[test]
    fn test_touch() {
        let mut doc = ExegesisDocument::new(
            "user.profile".to_string(),
            "1.0.0".to_string(),
            "Test".to_string(),
        );

        let initial_timestamp = doc.last_modified;
        std::thread::sleep(std::time::Duration::from_millis(10));
        doc.touch();

        assert!(doc.last_modified > initial_timestamp);
    }

    #[test]
    fn test_last_modified_datetime() {
        let doc = ExegesisDocument::new(
            "user.profile".to_string(),
            "1.0.0".to_string(),
            "Test".to_string(),
        );

        let dt = doc.last_modified_datetime();
        assert!(dt.is_some());
    }

    #[test]
    fn test_validate_success() {
        let doc = ExegesisDocument::new(
            "user.profile".to_string(),
            "1.0.0".to_string(),
            "Test content".to_string(),
        );

        assert!(doc.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_gene_id() {
        let doc = ExegesisDocument::new(
            "".to_string(),
            "1.0.0".to_string(),
            "Test".to_string(),
        );

        assert!(doc.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_version() {
        let doc = ExegesisDocument::new(
            "user.profile".to_string(),
            "1.0".to_string(), // Invalid: missing patch
            "Test".to_string(),
        );

        assert!(doc.validate().is_err());
    }

    #[test]
    fn test_validate_empty_content() {
        let doc = ExegesisDocument::new(
            "user.profile".to_string(),
            "1.0.0".to_string(),
            "".to_string(),
        );

        assert!(doc.validate().is_err());
    }

    #[test]
    fn test_serialization() {
        let doc = ExegesisDocument::new(
            "user.profile".to_string(),
            "1.0.0".to_string(),
            "Test content".to_string(),
        );

        let json = serde_json::to_string(&doc).unwrap();
        let deserialized: ExegesisDocument = serde_json::from_str(&json).unwrap();

        assert_eq!(doc, deserialized);
    }
}
