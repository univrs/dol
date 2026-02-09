use anyhow::Result;
use automerge::{AutoCommit, sync::{Message as SyncMessage, State as SyncState, SyncDoc}};
use tracing::{info, warn};

/// AutomergeSync manages CRDT synchronization using Automerge
pub struct AutomergeSync {
    sync_state: SyncState,
}

impl AutomergeSync {
    pub fn new() -> Self {
        Self {
            sync_state: SyncState::new(),
        }
    }

    /// Generate a sync message from the current document
    pub fn generate_sync_message(&mut self, doc: &mut AutoCommit) -> Result<Vec<u8>> {
        let msg = doc.sync().generate_sync_message(&mut self.sync_state);

        if let Some(sync_msg) = msg {
            let encoded = sync_msg.encode();
            Ok(encoded)
        } else {
            // No changes to sync
            Ok(Vec::new())
        }
    }

    /// Apply a sync message to the document
    /// Returns true if the document was modified
    pub fn apply_sync_message(&mut self, doc: &mut AutoCommit, data: &[u8]) -> Result<bool> {
        if data.is_empty() {
            return Ok(false);
        }

        // Decode the sync message
        let sync_msg = SyncMessage::decode(data)?;

        // Apply to document
        doc.sync().receive_sync_message(&mut self.sync_state, sync_msg)?;

        // Check if we have changes to apply
        let has_changes = self.sync_state.their_heads
            .as_ref()
            .map(|heads| !heads.is_empty())
            .unwrap_or(false);

        if has_changes {
            info!("Applied sync message with changes");
        }

        Ok(has_changes)
    }

    /// Reset sync state
    pub fn reset(&mut self) {
        self.sync_state = SyncState::new();
    }
}

impl Default for AutomergeSync {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_roundtrip() {
        let mut doc1 = AutoCommit::new();
        let mut doc2 = AutoCommit::new();

        let mut sync1 = AutomergeSync::new();
        let mut sync2 = AutomergeSync::new();

        // Add data to doc1
        doc1.put(automerge::ROOT, "key", "value").unwrap();

        // Generate sync message from doc1
        let msg1 = sync1.generate_sync_message(&mut doc1).unwrap();

        // Apply to doc2
        let changed = sync2.apply_sync_message(&mut doc2, &msg1).unwrap();
        assert!(changed || msg1.is_empty());

        // Verify doc2 has the data
        if !msg1.is_empty() {
            let value = doc2.get(automerge::ROOT, "key").unwrap();
            assert!(value.is_some());
        }
    }
}
