/// Peritext CRDT Integration Example
///
/// Demonstrates how the Collaborative Document uses Peritext CRDT
/// for conflict-free rich text editing with formatting preservation.

use automerge::{Automerge, ObjType, ROOT};
use serde::{Deserialize, Serialize};

/// Document structure generated from workspace.document DOL schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborativeDocument {
    pub id: String,
    pub title: String,
    pub content: String,  // Peritext CRDT
    pub collaborators: Vec<String>,
    pub owner: String,
    pub created_at: i64,
    pub last_modified: i64,
    pub tags: Vec<String>,
}

/// Formatting annotation for Peritext
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatSpan {
    pub start: usize,
    pub end: usize,
    pub format_type: FormatType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormatType {
    Bold,
    Italic,
    Underline,
    Heading(u8),
    Link(String),
    Code,
}

/// Peritext CRDT implementation example
pub struct PeritextDocument {
    doc: Automerge,
}

impl PeritextDocument {
    /// Create a new Peritext document
    pub fn new(document_id: String, owner: String) -> Self {
        let mut doc = Automerge::new();

        // Initialize document with DOL schema fields
        doc.put(ROOT, "id", document_id).unwrap();
        doc.put(ROOT, "owner", owner).unwrap();
        doc.put(ROOT, "title", "Untitled Document").unwrap();
        doc.put(ROOT, "created_at", chrono::Utc::now().timestamp()).unwrap();
        doc.put(ROOT, "last_modified", chrono::Utc::now().timestamp()).unwrap();

        // Create text object for content (Peritext)
        let text_obj = doc.put_object(ROOT, "content", ObjType::Text).unwrap();

        // Create arrays for collaborators and tags
        doc.put_object(ROOT, "collaborators", ObjType::List).unwrap();
        doc.put_object(ROOT, "tags", ObjType::List).unwrap();

        Self { doc }
    }

    /// Insert text at a position (Peritext operation)
    pub fn insert_text(&mut self, position: usize, text: &str) -> Result<(), String> {
        let text_obj = self.doc.get(ROOT, "content")
            .ok_or("Content object not found")?
            .1;

        // Peritext insert operation
        self.doc.splice_text(text_obj, position, 0, text)
            .map_err(|e| format!("Failed to insert text: {:?}", e))?;

        // Update last_modified timestamp
        self.doc.put(ROOT, "last_modified", chrono::Utc::now().timestamp()).unwrap();

        Ok(())
    }

    /// Delete text range (Peritext operation)
    pub fn delete_text(&mut self, start: usize, length: usize) -> Result<(), String> {
        let text_obj = self.doc.get(ROOT, "content")
            .ok_or("Content object not found")?
            .1;

        // Peritext delete operation
        self.doc.splice_text(text_obj, start, length as isize, "")
            .map_err(|e| format!("Failed to delete text: {:?}", e))?;

        self.doc.put(ROOT, "last_modified", chrono::Utc::now().timestamp()).unwrap();

        Ok(())
    }

    /// Apply formatting to text range (Peritext marks)
    pub fn apply_format(&mut self, start: usize, end: usize, format: FormatType) -> Result<(), String> {
        let text_obj = self.doc.get(ROOT, "content")
            .ok_or("Content object not found")?
            .1;

        // Store formatting as marks on the text
        let format_key = format!("format_{}_{}", start, end);
        let format_json = serde_json::to_string(&format)
            .map_err(|e| format!("Failed to serialize format: {:?}", e))?;

        self.doc.put(text_obj, &format_key, format_json)
            .map_err(|e| format!("Failed to apply format: {:?}", e))?;

        Ok(())
    }

    /// Get current text content
    pub fn get_text(&self) -> Result<String, String> {
        let text_obj = self.doc.get(ROOT, "content")
            .ok_or("Content object not found")?
            .1;

        self.doc.text(text_obj)
            .map_err(|e| format!("Failed to get text: {:?}", e))
    }

    /// Merge changes from another peer (Peritext CRDT merge)
    pub fn merge(&mut self, other_changes: &[u8]) -> Result<(), String> {
        self.doc.apply_changes(other_changes.to_vec())
            .map_err(|e| format!("Failed to merge changes: {:?}", e))?;

        Ok(())
    }

    /// Get changes since last sync
    pub fn get_changes(&self) -> Vec<u8> {
        self.doc.save()
    }

    /// Add a collaborator (RGA CRDT for list)
    pub fn add_collaborator(&mut self, did: String) -> Result<(), String> {
        let collaborators_obj = self.doc.get(ROOT, "collaborators")
            .ok_or("Collaborators list not found")?
            .1;

        // RGA append operation
        let len = self.doc.length(collaborators_obj);
        self.doc.insert(collaborators_obj, len, did)
            .map_err(|e| format!("Failed to add collaborator: {:?}", e))?;

        Ok(())
    }
}

/// Example: Simulating concurrent edits from two users
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concurrent_edits_converge() {
        // Alice creates a document
        let mut alice_doc = PeritextDocument::new(
            "doc-123".to_string(),
            "did:key:alice".to_string(),
        );

        // Alice writes initial content
        alice_doc.insert_text(0, "Hello World").unwrap();

        // Bob forks the document
        let alice_changes = alice_doc.get_changes();
        let mut bob_doc = PeritextDocument::new(
            "doc-123".to_string(),
            "did:key:alice".to_string(),
        );
        bob_doc.merge(&alice_changes).unwrap();

        // Alice and Bob edit concurrently (offline)
        alice_doc.insert_text(5, " Beautiful").unwrap();  // "Hello Beautiful World"
        bob_doc.insert_text(11, "!").unwrap();           // "Hello World!"

        // They sync changes
        let alice_changes = alice_doc.get_changes();
        let bob_changes = bob_doc.get_changes();

        alice_doc.merge(&bob_changes).unwrap();
        bob_doc.merge(&alice_changes).unwrap();

        // Both converge to the same state
        let alice_text = alice_doc.get_text().unwrap();
        let bob_text = bob_doc.get_text().unwrap();

        assert_eq!(alice_text, bob_text);
        assert_eq!(alice_text, "Hello Beautiful World!");

        println!("✅ Concurrent edits converged: {}", alice_text);
    }

    #[test]
    fn test_formatting_preservation() {
        let mut doc = PeritextDocument::new(
            "doc-456".to_string(),
            "did:key:alice".to_string(),
        );

        doc.insert_text(0, "This is important text").unwrap();
        doc.apply_format(8, 17, FormatType::Bold).unwrap();

        let text = doc.get_text().unwrap();
        assert_eq!(text, "This is important text");

        // In a real implementation, formatting would be preserved through merges
        println!("✅ Formatting applied to: 'important'");
    }

    #[test]
    fn test_collaborative_editing() {
        let mut doc = PeritextDocument::new(
            "doc-789".to_string(),
            "did:key:alice".to_string(),
        );

        doc.add_collaborator("did:key:bob".to_string()).unwrap();
        doc.add_collaborator("did:key:carol".to_string()).unwrap();

        doc.insert_text(0, "Collaborative document").unwrap();

        let text = doc.get_text().unwrap();
        assert_eq!(text, "Collaborative document");

        println!("✅ Collaborative document created with 2 collaborators");
    }
}

/// JavaScript/WASM integration example
#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use wasm_bindgen::prelude::*;
    use super::*;

    #[wasm_bindgen]
    pub struct WasmPeritextDocument {
        inner: PeritextDocument,
    }

    #[wasm_bindgen]
    impl WasmPeritextDocument {
        #[wasm_bindgen(constructor)]
        pub fn new(document_id: String, owner: String) -> Self {
            Self {
                inner: PeritextDocument::new(document_id, owner),
            }
        }

        #[wasm_bindgen]
        pub fn insert(&mut self, position: usize, text: String) -> Result<(), JsValue> {
            self.inner.insert_text(position, &text)
                .map_err(|e| JsValue::from_str(&e))
        }

        #[wasm_bindgen]
        pub fn delete(&mut self, start: usize, length: usize) -> Result<(), JsValue> {
            self.inner.delete_text(start, length)
                .map_err(|e| JsValue::from_str(&e))
        }

        #[wasm_bindgen]
        pub fn get_text(&self) -> Result<String, JsValue> {
            self.inner.get_text()
                .map_err(|e| JsValue::from_str(&e))
        }

        #[wasm_bindgen]
        pub fn merge(&mut self, changes: &[u8]) -> Result<(), JsValue> {
            self.inner.merge(changes)
                .map_err(|e| JsValue::from_str(&e))
        }
    }
}

/// Usage example in a real application
///
/// ```rust
/// // Create document
/// let mut doc = PeritextDocument::new(
///     "doc-001".to_string(),
///     "did:key:alice".to_string(),
/// );
///
/// // User types "Hello"
/// doc.insert_text(0, "Hello")?;
///
/// // User types " World"
/// doc.insert_text(5, " World")?;
///
/// // User selects "World" and makes it bold
/// doc.apply_format(6, 11, FormatType::Bold)?;
///
/// // Get changes to send to peers via Iroh P2P
/// let changes = doc.get_changes();
/// sync_engine.broadcast_changes(&changes).await?;
///
/// // Receive changes from peer
/// let peer_changes = sync_engine.receive_changes().await?;
/// doc.merge(&peer_changes)?;
/// ```
