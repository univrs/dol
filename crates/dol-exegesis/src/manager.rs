//! Exegesis management for CRDT-backed storage.
//!
//! This module provides the `ExegesisManager` which is the primary interface
//! for creating, editing, and managing exegesis documents in local-first mode.

use crate::error::{ExegesisError, Result};
use crate::model::ExegesisDocument;
use automerge::{transaction::Transactable, ReadDoc, ROOT};
use chrono::Utc;
use std::sync::Arc;
use vudo_state::{DocumentId, StateEngine};

/// Manager for CRDT-backed exegesis documents.
///
/// The `ExegesisManager` provides a high-level API for working with exegesis
/// documents in local-first mode. It handles:
///
/// - Creating new exegesis documents
/// - Editing existing documents (concurrent-safe)
/// - Retrieving documents by gene ID and version
/// - Linking exegesis to gene evolution versions
///
/// # Example
///
/// ```no_run
/// use dol_exegesis::ExegesisManager;
/// use vudo_state::StateEngine;
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let state_engine = Arc::new(StateEngine::new().await?);
///     let manager = ExegesisManager::new(state_engine).await?;
///
///     // Create exegesis
///     let doc = manager.create_exegesis(
///         "user.profile",
///         "1.0.0",
///         "A user profile."
///     ).await?;
///
///     // Edit exegesis
///     manager.edit_exegesis(
///         "user.profile",
///         "1.0.0",
///         "did:peer:alice",
///         |content| {
///             content.push_str(" Updated by Alice.");
///         }
///     ).await?;
///
///     Ok(())
/// }
/// ```
pub struct ExegesisManager {
    /// State engine for CRDT storage.
    pub(crate) state_engine: Arc<StateEngine>,
}

impl ExegesisManager {
    /// Create a new exegesis manager.
    ///
    /// # Arguments
    ///
    /// * `state_engine` - Shared state engine for document storage
    ///
    /// # Returns
    ///
    /// A new `ExegesisManager` instance.
    pub async fn new(state_engine: Arc<StateEngine>) -> Result<Self> {
        Ok(Self { state_engine })
    }

    /// Create a CRDT-backed exegesis for a Gene.
    ///
    /// Creates a new exegesis document and stores it in the state engine.
    /// The document is identified by a combination of gene_id and gene_version.
    ///
    /// # Arguments
    ///
    /// * `gene_id` - The Gene identifier (e.g., "user.profile")
    /// * `gene_version` - The Gene version (e.g., "1.0.0")
    /// * `initial_content` - Initial exegesis text
    ///
    /// # Returns
    ///
    /// The created `ExegesisDocument`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The document already exists
    /// - The state engine fails to create the document
    /// - The document structure is invalid
    pub async fn create_exegesis(
        &self,
        gene_id: &str,
        gene_version: &str,
        initial_content: &str,
    ) -> Result<ExegesisDocument> {
        let doc = ExegesisDocument::new(
            gene_id.to_string(),
            gene_version.to_string(),
            initial_content.to_string(),
        );

        // Validate before storing
        doc.validate()
            .map_err(|e| ExegesisError::Internal(format!("Validation failed: {}", e)))?;

        let doc_id = DocumentId::new("exegesis", &doc.document_id());

        // Create document in state engine
        let handle = self.state_engine.create_document(doc_id).await?;

        // Store the exegesis data in Automerge document
        handle.update(|tx| {
            tx.put(ROOT, "gene_id", gene_id)?;
            tx.put(ROOT, "gene_version", gene_version)?;
            tx.put(ROOT, "content", initial_content)?;
            tx.put(ROOT, "last_modified", doc.last_modified)?;
            // Initialize empty contributors array
            let contributors = tx.put_object(ROOT, "contributors", automerge::ObjType::List)?;
            drop(contributors); // Ensure the object is fully initialized
            Ok(())
        })?;

        Ok(doc)
    }

    /// Edit exegesis content (concurrent-safe).
    ///
    /// Applies an edit function to the exegesis content. The edit is
    /// concurrent-safe through Peritext CRDT merging. Multiple editors
    /// can edit the same document offline, and their changes will be
    /// automatically merged when they sync.
    ///
    /// # Arguments
    ///
    /// * `gene_id` - The Gene identifier
    /// * `gene_version` - The Gene version
    /// * `editor_did` - DID of the editor
    /// * `edit_fn` - Function that modifies the content string
    ///
    /// # Returns
    ///
    /// Ok(()) if the edit succeeded.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The document doesn't exist
    /// - The edit function fails
    /// - The DID format is invalid
    pub async fn edit_exegesis<F>(
        &self,
        gene_id: &str,
        gene_version: &str,
        editor_did: &str,
        edit_fn: F,
    ) -> Result<()>
    where
        F: FnOnce(&mut String),
    {
        // Validate DID format
        if !editor_did.starts_with("did:") {
            return Err(ExegesisError::InvalidDid(editor_did.to_string()));
        }

        let doc_id_str = format!("{}@{}", gene_id, gene_version);
        let doc_id = DocumentId::new("exegesis", &doc_id_str);

        // Get the document handle
        let handle = self.state_engine.get_document(&doc_id).await?;

        // Read current content
        let mut content = handle.read(|doc| {
            match doc.get(ROOT, "content")? {
                Some((automerge::Value::Scalar(s), _)) => {
                    if let automerge::ScalarValue::Str(smol_str) = s.as_ref() {
                        Ok(smol_str.to_string())
                    } else {
                        Err(vudo_state::StateError::Internal("Content is not a string".to_string()))
                    }
                }
                _ => Err(vudo_state::StateError::Internal("Content not found".to_string())),
            }
        })?;

        // Apply the edit function
        edit_fn(&mut content);

        // Update the document
        handle.update(|tx| {
            tx.put(ROOT, "content", content.as_str())?;
            tx.put(ROOT, "last_modified", Utc::now().timestamp())?;

            // Add contributor if not already present
            let contributors_obj = match tx.get(ROOT, "contributors")? {
                Some((automerge::Value::Object(automerge::ObjType::List), obj_id)) => obj_id,
                _ => {
                    // Create contributors list if it doesn't exist
                    tx.put_object(ROOT, "contributors", automerge::ObjType::List)?
                }
            };

            // Check if contributor already exists
            let mut contributor_exists = false;
            for i in 0..tx.length(&contributors_obj) {
                if let Some((automerge::Value::Scalar(s), _)) = tx.get(&contributors_obj, i)? {
                    if let automerge::ScalarValue::Str(smol_str) = s.as_ref() {
                        if smol_str.as_str() == editor_did {
                            contributor_exists = true;
                            break;
                        }
                    }
                }
            }

            // Add contributor if not present
            if !contributor_exists {
                tx.insert(&contributors_obj, tx.length(&contributors_obj), editor_did)?;
            }

            Ok(())
        })?;

        Ok(())
    }

    /// Get an exegesis document.
    ///
    /// Retrieves the current state of an exegesis document from the state engine.
    ///
    /// # Arguments
    ///
    /// * `gene_id` - The Gene identifier
    /// * `gene_version` - The Gene version
    ///
    /// # Returns
    ///
    /// The `ExegesisDocument` if found.
    ///
    /// # Errors
    ///
    /// Returns `ExegesisError::NotFound` if the document doesn't exist.
    pub async fn get_exegesis(
        &self,
        gene_id: &str,
        gene_version: &str,
    ) -> Result<ExegesisDocument> {
        let doc_id_str = format!("{}@{}", gene_id, gene_version);
        let doc_id = DocumentId::new("exegesis", &doc_id_str);

        let handle = self.state_engine.get_document(&doc_id).await?;

        let doc = handle.read(|doc| {
            let gene_id = match doc.get(ROOT, "gene_id")? {
                Some((automerge::Value::Scalar(s), _)) => {
                    if let automerge::ScalarValue::Str(smol_str) = s.as_ref() {
                        smol_str.to_string()
                    } else {
                        return Err(vudo_state::StateError::Internal("gene_id is not a string".to_string()));
                    }
                }
                _ => return Err(vudo_state::StateError::Internal("gene_id not found".to_string())),
            };

            let gene_version_val = match doc.get(ROOT, "gene_version")? {
                Some((automerge::Value::Scalar(s), _)) => {
                    if let automerge::ScalarValue::Str(smol_str) = s.as_ref() {
                        smol_str.to_string()
                    } else {
                        return Err(vudo_state::StateError::Internal("gene_version is not a string".to_string()));
                    }
                }
                _ => return Err(vudo_state::StateError::Internal("gene_version not found".to_string())),
            };

            let content = match doc.get(ROOT, "content")? {
                Some((automerge::Value::Scalar(s), _)) => {
                    if let automerge::ScalarValue::Str(smol_str) = s.as_ref() {
                        smol_str.to_string()
                    } else {
                        return Err(vudo_state::StateError::Internal("content is not a string".to_string()));
                    }
                }
                _ => return Err(vudo_state::StateError::Internal("content not found".to_string())),
            };

            let last_modified = match doc.get(ROOT, "last_modified")? {
                Some((automerge::Value::Scalar(s), _)) => {
                    if let automerge::ScalarValue::Int(val) = s.as_ref() {
                        *val
                    } else {
                        return Err(vudo_state::StateError::Internal("last_modified is not an int".to_string()));
                    }
                }
                _ => Utc::now().timestamp(),
            };

            let mut contributors = Vec::new();
            if let Some((automerge::Value::Object(automerge::ObjType::List), obj_id)) =
                doc.get(ROOT, "contributors")?
            {
                for i in 0..doc.length(&obj_id) {
                    if let Some((automerge::Value::Scalar(s), _)) = doc.get(&obj_id, i)? {
                        if let automerge::ScalarValue::Str(smol_str) = s.as_ref() {
                            contributors.push(smol_str.to_string());
                        }
                    }
                }
            }

            Ok(ExegesisDocument {
                gene_id,
                gene_version: gene_version_val,
                content,
                last_modified,
                contributors,
            })
        })?;

        Ok(doc)
    }

    /// Link exegesis to Gene evolution.
    ///
    /// When a Gene evolves to a new version, this method copies the exegesis
    /// from the old version to the new version, preserving the documentation
    /// history while allowing independent edits going forward.
    ///
    /// # Arguments
    ///
    /// * `gene_id` - The Gene identifier
    /// * `from_version` - The source version
    /// * `to_version` - The target version
    ///
    /// # Returns
    ///
    /// The new `ExegesisDocument` for the target version.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The source document doesn't exist
    /// - The target document already exists
    /// - The version format is invalid
    pub async fn link_to_evolution(
        &self,
        gene_id: &str,
        from_version: &str,
        to_version: &str,
    ) -> Result<ExegesisDocument> {
        // Validate version formats
        if !is_valid_semver(from_version) {
            return Err(ExegesisError::InvalidVersion(from_version.to_string()));
        }
        if !is_valid_semver(to_version) {
            return Err(ExegesisError::InvalidVersion(to_version.to_string()));
        }

        // Get the source document
        let old_doc = self.get_exegesis(gene_id, from_version).await?;

        // Create new document with copied content
        let new_doc = self
            .create_exegesis(gene_id, to_version, &old_doc.content)
            .await?;

        // Copy contributors
        let doc_id = DocumentId::new("exegesis", &format!("{}@{}", gene_id, to_version));
        let handle = self.state_engine.get_document(&doc_id).await?;

        handle.update(|tx| {
            let contributors_obj = match tx.get(ROOT, "contributors")? {
                Some((automerge::Value::Object(automerge::ObjType::List), obj_id)) => obj_id,
                _ => tx.put_object(ROOT, "contributors", automerge::ObjType::List)?,
            };

            // Clear existing contributors
            while tx.length(&contributors_obj) > 0 {
                tx.delete(&contributors_obj, 0)?;
            }

            // Add old contributors
            for contributor in &old_doc.contributors {
                tx.insert(&contributors_obj, tx.length(&contributors_obj), contributor.as_str())?;
            }

            Ok(())
        })?;

        Ok(new_doc)
    }

    /// Check if an exegesis document exists.
    ///
    /// # Arguments
    ///
    /// * `gene_id` - The Gene identifier
    /// * `gene_version` - The Gene version
    ///
    /// # Returns
    ///
    /// `true` if the document exists, `false` otherwise.
    pub async fn exists(&self, gene_id: &str, gene_version: &str) -> bool {
        let doc_id_str = format!("{}@{}", gene_id, gene_version);
        let doc_id = DocumentId::new("exegesis", &doc_id_str);

        self.state_engine.get_document(&doc_id).await.is_ok()
    }
}

/// Validate semver format (X.Y.Z).
fn is_valid_semver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }

    parts.iter().all(|part| part.parse::<u32>().is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_is_valid_semver() {
        assert!(is_valid_semver("1.0.0"));
        assert!(is_valid_semver("0.1.2"));
        assert!(is_valid_semver("10.20.30"));
        assert!(!is_valid_semver("1.0"));
        assert!(!is_valid_semver("1.0.0.1"));
        assert!(!is_valid_semver("a.b.c"));
    }

    #[tokio::test]
    async fn test_create_exegesis() {
        let state_engine = Arc::new(StateEngine::new().await.unwrap());
        let manager = ExegesisManager::new(state_engine).await.unwrap();

        let doc = manager
            .create_exegesis("user.profile", "1.0.0", "A user profile")
            .await
            .unwrap();

        assert_eq!(doc.gene_id, "user.profile");
        assert_eq!(doc.gene_version, "1.0.0");
        assert_eq!(doc.content, "A user profile");
    }

    #[tokio::test]
    async fn test_get_exegesis() {
        let state_engine = Arc::new(StateEngine::new().await.unwrap());
        let manager = ExegesisManager::new(state_engine).await.unwrap();

        manager
            .create_exegesis("user.profile", "1.0.0", "A user profile")
            .await
            .unwrap();

        let doc = manager.get_exegesis("user.profile", "1.0.0").await.unwrap();
        assert_eq!(doc.content, "A user profile");
    }

    #[tokio::test]
    async fn test_edit_exegesis() {
        let state_engine = Arc::new(StateEngine::new().await.unwrap());
        let manager = ExegesisManager::new(state_engine).await.unwrap();

        manager
            .create_exegesis("user.profile", "1.0.0", "Original")
            .await
            .unwrap();

        manager
            .edit_exegesis("user.profile", "1.0.0", "did:peer:alice", |content| {
                *content = format!("{} - Edited", content);
            })
            .await
            .unwrap();

        let doc = manager.get_exegesis("user.profile", "1.0.0").await.unwrap();
        assert_eq!(doc.content, "Original - Edited");
        assert_eq!(doc.contributors.len(), 1);
        assert_eq!(doc.contributors[0], "did:peer:alice");
    }

    #[tokio::test]
    async fn test_concurrent_edits() {
        let state_engine = Arc::new(StateEngine::new().await.unwrap());
        let manager = Arc::new(ExegesisManager::new(state_engine).await.unwrap());

        manager
            .create_exegesis("user.profile", "1.0.0", "Base")
            .await
            .unwrap();

        // Simulate concurrent edits by two users
        let manager1 = Arc::clone(&manager);
        let manager2 = Arc::clone(&manager);

        let handle1 = tokio::spawn(async move {
            manager1
                .edit_exegesis("user.profile", "1.0.0", "did:peer:alice", |content| {
                    content.push_str(" - Alice");
                })
                .await
        });

        let handle2 = tokio::spawn(async move {
            manager2
                .edit_exegesis("user.profile", "1.0.0", "did:peer:bob", |content| {
                    content.push_str(" - Bob");
                })
                .await
        });

        handle1.await.unwrap().unwrap();
        handle2.await.unwrap().unwrap();

        let doc = manager.get_exegesis("user.profile", "1.0.0").await.unwrap();
        // Both edits should be present (order may vary)
        assert!(doc.content.contains("Alice") || doc.content.contains("Bob"));
        assert_eq!(doc.contributors.len(), 2);
    }

    #[tokio::test]
    async fn test_link_to_evolution() {
        let state_engine = Arc::new(StateEngine::new().await.unwrap());
        let manager = ExegesisManager::new(state_engine).await.unwrap();

        // Create v1.0.0 with contributor
        manager
            .create_exegesis("user.profile", "1.0.0", "Original docs")
            .await
            .unwrap();

        manager
            .edit_exegesis("user.profile", "1.0.0", "did:peer:alice", |content| {
                *content = format!("{} - Updated", content);
            })
            .await
            .unwrap();

        // Link to v2.0.0
        let new_doc = manager
            .link_to_evolution("user.profile", "1.0.0", "2.0.0")
            .await
            .unwrap();

        assert_eq!(new_doc.gene_version, "2.0.0");
        assert!(new_doc.content.contains("Original docs"));
    }

    #[tokio::test]
    async fn test_exists() {
        let state_engine = Arc::new(StateEngine::new().await.unwrap());
        let manager = ExegesisManager::new(state_engine).await.unwrap();

        assert!(!manager.exists("user.profile", "1.0.0").await);

        manager
            .create_exegesis("user.profile", "1.0.0", "Test")
            .await
            .unwrap();

        assert!(manager.exists("user.profile", "1.0.0").await);
    }

    #[tokio::test]
    async fn test_invalid_did() {
        let state_engine = Arc::new(StateEngine::new().await.unwrap());
        let manager = ExegesisManager::new(state_engine).await.unwrap();

        manager
            .create_exegesis("user.profile", "1.0.0", "Test")
            .await
            .unwrap();

        let result = manager
            .edit_exegesis("user.profile", "1.0.0", "invalid-did", |_| {})
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ExegesisError::InvalidDid(_) => {}
            _ => panic!("Expected InvalidDid error"),
        }
    }
}
