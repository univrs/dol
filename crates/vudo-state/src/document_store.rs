//! Document store for managing Automerge documents.

use crate::error::{Result, StateError};
use automerge::AutoCommit;
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Document identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentId {
    /// Namespace (e.g., "users", "posts").
    pub namespace: String,
    /// Document key within namespace.
    pub key: String,
}

impl DocumentId {
    /// Create a new document ID.
    pub fn new(namespace: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            key: key.into(),
        }
    }

    /// Parse a document ID from a string (format: "namespace/key").
    pub fn parse(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(StateError::InvalidDocumentId(s.to_string()));
        }
        Ok(Self {
            namespace: parts[0].to_string(),
            key: parts[1].to_string(),
        })
    }

    /// Convert to string representation.
    pub fn to_string(&self) -> String {
        format!("{}/{}", self.namespace, self.key)
    }
}

impl std::fmt::Display for DocumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.namespace, self.key)
    }
}

/// Metadata about a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Document ID.
    pub id: DocumentId,
    /// Creation timestamp (Unix epoch milliseconds).
    pub created_at: u64,
    /// Last modification timestamp (Unix epoch milliseconds).
    pub last_modified: u64,
    /// Document size in bytes.
    pub size: usize,
    /// Document version (number of changes).
    pub version: u64,
}

/// A handle to an Automerge document.
#[derive(Clone)]
pub struct DocumentHandle {
    /// Document ID.
    pub id: DocumentId,
    /// Automerge document.
    pub(crate) doc: Arc<RwLock<AutoCommit>>,
    /// Metadata.
    pub(crate) metadata: Arc<RwLock<DocumentMetadata>>,
}

impl DocumentHandle {
    /// Create a new document handle.
    fn new(id: DocumentId, mut doc: AutoCommit) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let size = doc.save().len();
        let version = doc.get_heads().len() as u64;

        let metadata = DocumentMetadata {
            id: id.clone(),
            created_at: now,
            last_modified: now,
            size,
            version,
        };

        Self {
            id,
            doc: Arc::new(RwLock::new(doc)),
            metadata: Arc::new(RwLock::new(metadata)),
        }
    }

    /// Get document metadata.
    pub fn metadata(&self) -> DocumentMetadata {
        self.metadata.read().clone()
    }

    /// Update a document with a transaction function.
    pub fn update<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut AutoCommit) -> Result<T>,
    {
        let mut doc = self.doc.write();
        let result = f(&mut *doc)?;

        // Update metadata
        let mut meta = self.metadata.write();
        meta.last_modified = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        meta.size = doc.save().len();
        meta.version += 1;

        Ok(result)
    }

    /// Read from the document.
    pub fn read<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&AutoCommit) -> Result<T>,
    {
        let doc = self.doc.read();
        f(&*doc)
    }

    /// Save the document to bytes.
    pub fn save(&self) -> Vec<u8> {
        self.doc.write().save()
    }

    /// Get the number of changes in the document.
    pub fn change_count(&self) -> usize {
        self.doc.write().get_changes(&[]).len()
    }
}

/// Document store for managing multiple Automerge documents.
pub struct DocumentStore {
    /// Map of document ID to document handle.
    documents: DashMap<DocumentId, DocumentHandle>,
}

impl DocumentStore {
    /// Create a new document store.
    pub fn new() -> Self {
        Self {
            documents: DashMap::new(),
        }
    }

    /// Create a new document.
    pub fn create(&self, id: DocumentId) -> Result<DocumentHandle> {
        if self.documents.contains_key(&id) {
            return Err(StateError::DocumentAlreadyExists(id.to_string()));
        }

        let doc = AutoCommit::new();
        let handle = DocumentHandle::new(id.clone(), doc);
        self.documents.insert(id, handle.clone());
        Ok(handle)
    }

    /// Load a document from bytes.
    pub fn load(&self, id: DocumentId, bytes: &[u8]) -> Result<DocumentHandle> {
        if self.documents.contains_key(&id) {
            return Err(StateError::DocumentAlreadyExists(id.to_string()));
        }

        let doc = AutoCommit::load(bytes)?;
        let handle = DocumentHandle::new(id.clone(), doc);
        self.documents.insert(id, handle.clone());
        Ok(handle)
    }

    /// Get a document by ID.
    pub fn get(&self, id: &DocumentId) -> Result<DocumentHandle> {
        self.documents
            .get(id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| StateError::DocumentNotFound(id.to_string()))
    }

    /// Check if a document exists.
    pub fn exists(&self, id: &DocumentId) -> bool {
        self.documents.contains_key(id)
    }

    /// Delete a document.
    pub fn delete(&self, id: &DocumentId) -> Result<()> {
        self.documents
            .remove(id)
            .ok_or_else(|| StateError::DocumentNotFound(id.to_string()))?;
        Ok(())
    }

    /// List all document IDs in a namespace.
    pub fn list_namespace(&self, namespace: &str) -> Vec<DocumentId> {
        self.documents
            .iter()
            .filter(|entry| entry.key().namespace == namespace)
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// List all document IDs.
    pub fn list_all(&self) -> Vec<DocumentId> {
        self.documents
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Get the number of documents in the store.
    pub fn count(&self) -> usize {
        self.documents.len()
    }

    /// Clear all documents.
    pub fn clear(&self) {
        self.documents.clear();
    }

    /// Get total size of all documents in bytes.
    pub fn total_size(&self) -> usize {
        self.documents
            .iter()
            .map(|entry| entry.value().metadata().size)
            .sum()
    }
}

impl Default for DocumentStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use automerge::{transaction::Transactable, ReadDoc, ROOT, ScalarValue};

    fn get_string(doc: &impl ReadDoc, obj: automerge::ObjId, key: &str) -> Result<String> {
        match doc.get(&obj, key)? {
            Some((automerge::Value::Scalar(s), _)) => {
                if let ScalarValue::Str(smol_str) = s.as_ref() {
                    Ok(smol_str.to_string())
                } else {
                    Err(StateError::Internal("Expected string value".to_string()))
                }
            }
            _ => Err(StateError::Internal("Value not found".to_string())),
        }
    }

    fn get_i64(doc: &impl ReadDoc, obj: automerge::ObjId, key: &str) -> Result<i64> {
        match doc.get(&obj, key)? {
            Some((automerge::Value::Scalar(s), _)) => {
                if let ScalarValue::Int(val) = s.as_ref() {
                    Ok(*val)
                } else {
                    Err(StateError::Internal("Expected int value".to_string()))
                }
            }
            _ => Err(StateError::Internal("Value not found".to_string())),
        }
    }

    #[test]
    fn test_document_id_new() {
        let id = DocumentId::new("users", "alice");
        assert_eq!(id.namespace, "users");
        assert_eq!(id.key, "alice");
    }

    #[test]
    fn test_document_id_parse() {
        let id = DocumentId::parse("users/alice").unwrap();
        assert_eq!(id.namespace, "users");
        assert_eq!(id.key, "alice");
    }

    #[test]
    fn test_document_id_parse_invalid() {
        assert!(DocumentId::parse("invalid").is_err());
        assert!(DocumentId::parse("invalid/too/many/parts").is_err());
    }

    #[test]
    fn test_document_id_to_string() {
        let id = DocumentId::new("users", "alice");
        assert_eq!(id.to_string(), "users/alice");
    }

    #[test]
    fn test_document_store_create() {
        let store = DocumentStore::new();
        let id = DocumentId::new("users", "alice");
        let handle = store.create(id.clone()).unwrap();
        assert_eq!(handle.id, id);
    }

    #[test]
    fn test_document_store_create_duplicate() {
        let store = DocumentStore::new();
        let id = DocumentId::new("users", "alice");
        store.create(id.clone()).unwrap();
        let result = store.create(id);
        assert!(matches!(result, Err(StateError::DocumentAlreadyExists(_))));
    }

    #[test]
    fn test_document_store_get() {
        let store = DocumentStore::new();
        let id = DocumentId::new("users", "alice");
        store.create(id.clone()).unwrap();
        let handle = store.get(&id).unwrap();
        assert_eq!(handle.id, id);
    }

    #[test]
    fn test_document_store_get_not_found() {
        let store = DocumentStore::new();
        let id = DocumentId::new("users", "alice");
        let result = store.get(&id);
        assert!(matches!(result, Err(StateError::DocumentNotFound(_))));
    }

    #[test]
    fn test_document_store_delete() {
        let store = DocumentStore::new();
        let id = DocumentId::new("users", "alice");
        store.create(id.clone()).unwrap();
        store.delete(&id).unwrap();
        assert!(!store.exists(&id));
    }

    #[test]
    fn test_document_store_exists() {
        let store = DocumentStore::new();
        let id = DocumentId::new("users", "alice");
        assert!(!store.exists(&id));
        store.create(id.clone()).unwrap();
        assert!(store.exists(&id));
    }

    #[test]
    fn test_document_handle_update() {
        let store = DocumentStore::new();
        let id = DocumentId::new("users", "alice");
        let handle = store.create(id).unwrap();

        handle
            .update(|doc| {
                doc.put(ROOT, "name", "Alice")?;
                doc.put(ROOT, "age", 30i64)?;
                Ok(())
            })
            .unwrap();

        handle
            .read(|doc| {
                let name = get_string(doc, ROOT, "name")?;
                let age = get_i64(doc, ROOT, "age")?;
                assert_eq!(name, "Alice");
                assert_eq!(age, 30);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn test_document_handle_save_load() {
        let store = DocumentStore::new();
        let id1 = DocumentId::new("users", "alice");
        let handle1 = store.create(id1).unwrap();

        handle1
            .update(|doc| {
                doc.put(ROOT, "name", "Alice")?;
                Ok(())
            })
            .unwrap();

        let bytes = handle1.save();

        let id2 = DocumentId::new("users", "alice_copy");
        let handle2 = store.load(id2, &bytes).unwrap();

        handle2
            .read(|doc| {
                let name = get_string(doc, ROOT, "name")?;
                assert_eq!(name, "Alice");
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn test_document_store_list_namespace() {
        let store = DocumentStore::new();
        store.create(DocumentId::new("users", "alice")).unwrap();
        store.create(DocumentId::new("users", "bob")).unwrap();
        store.create(DocumentId::new("posts", "1")).unwrap();

        let users = store.list_namespace("users");
        assert_eq!(users.len(), 2);

        let posts = store.list_namespace("posts");
        assert_eq!(posts.len(), 1);
    }

    #[test]
    fn test_document_store_list_all() {
        let store = DocumentStore::new();
        store.create(DocumentId::new("users", "alice")).unwrap();
        store.create(DocumentId::new("posts", "1")).unwrap();

        let all = store.list_all();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_document_store_count() {
        let store = DocumentStore::new();
        assert_eq!(store.count(), 0);

        store.create(DocumentId::new("users", "alice")).unwrap();
        assert_eq!(store.count(), 1);

        store.create(DocumentId::new("users", "bob")).unwrap();
        assert_eq!(store.count(), 2);
    }

    #[test]
    fn test_document_store_clear() {
        let store = DocumentStore::new();
        store.create(DocumentId::new("users", "alice")).unwrap();
        store.create(DocumentId::new("users", "bob")).unwrap();
        assert_eq!(store.count(), 2);

        store.clear();
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_document_metadata() {
        let store = DocumentStore::new();
        let id = DocumentId::new("users", "alice");
        let handle = store.create(id.clone()).unwrap();

        let meta = handle.metadata();
        assert_eq!(meta.id, id);
        assert!(meta.created_at > 0);
        assert_eq!(meta.created_at, meta.last_modified);

        // Update the document
        std::thread::sleep(std::time::Duration::from_millis(10));
        handle
            .update(|doc| {
                doc.put(ROOT, "name", "Alice")?;
                Ok(())
            })
            .unwrap();

        let meta2 = handle.metadata();
        assert!(meta2.last_modified > meta.last_modified);
        assert!(meta2.version > meta.version);
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let store = Arc::new(DocumentStore::new());
        let id = DocumentId::new("counter", "shared");
        store.create(id.clone()).unwrap();

        let mut handles = vec![];
        for i in 0..10 {
            let store_clone = Arc::clone(&store);
            let id_clone = id.clone();
            let handle = thread::spawn(move || {
                let doc = store_clone.get(&id_clone).unwrap();
                doc.update(|d| {
                    d.put(ROOT, format!("key{}", i), i as i64)?;
                    Ok(())
                })
                .unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let doc = store.get(&id).unwrap();
        doc.read(|d| {
            for i in 0..10 {
                let val = get_i64(d, ROOT, &format!("key{}", i))?;
                assert_eq!(val, i);
            }
            Ok(())
        })
        .unwrap();
    }
}
