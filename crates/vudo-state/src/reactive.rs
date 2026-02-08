//! Reactive subscriptions for document changes.

use crate::document_store::{DocumentHandle, DocumentId};
use crate::error::{Result, StateError};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};

/// Subscription ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(u64);

impl SubscriptionId {
    /// Generate a new subscription ID.
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

/// Change event describing a document mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEvent {
    /// Document ID.
    pub document_id: DocumentId,
    /// Timestamp of the change (Unix epoch milliseconds).
    pub timestamp: u64,
    /// Change hash (Automerge heads).
    pub change_hash: Vec<u8>,
    /// Path affected (if path-specific subscription).
    pub path: Option<String>,
}

/// Subscription handle that can be used to unsubscribe.
pub struct Subscription {
    /// Subscription ID.
    pub id: SubscriptionId,
    /// Receiver for change events.
    pub receiver: mpsc::UnboundedReceiver<ChangeEvent>,
}

impl Subscription {
    /// Receive the next change event.
    pub async fn recv(&mut self) -> Option<ChangeEvent> {
        self.receiver.recv().await
    }

    /// Try to receive a change event without blocking.
    pub fn try_recv(&mut self) -> std::result::Result<ChangeEvent, mpsc::error::TryRecvError> {
        self.receiver.try_recv()
    }
}

/// Subscription filter.
#[derive(Debug, Clone)]
pub enum SubscriptionFilter {
    /// Subscribe to all changes on a document.
    Document(DocumentId),
    /// Subscribe to changes on a specific path (e.g., "users/*/name").
    Path(DocumentId, String),
}

impl SubscriptionFilter {
    /// Check if a change matches this filter.
    pub fn matches(&self, event: &ChangeEvent) -> bool {
        match self {
            SubscriptionFilter::Document(doc_id) => event.document_id == *doc_id,
            SubscriptionFilter::Path(doc_id, path) => {
                event.document_id == *doc_id
                    && event
                        .path
                        .as_ref()
                        .map(|p| path_matches(path, p))
                        .unwrap_or(false)
            }
        }
    }
}

/// Path matching with wildcard support.
fn path_matches(pattern: &str, path: &str) -> bool {
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let path_parts: Vec<&str> = path.split('/').collect();

    if pattern_parts.len() != path_parts.len() {
        return false;
    }

    for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
        if *pattern_part != "*" && *pattern_part != *path_part {
            return false;
        }
    }

    true
}

/// Internal subscription data.
struct SubscriptionData {
    /// Filter for this subscription.
    filter: SubscriptionFilter,
    /// Sender for change events.
    sender: mpsc::UnboundedSender<ChangeEvent>,
}

/// Change event batcher to coalesce rapid changes.
struct EventBatcher {
    /// Pending events.
    pending: Vec<ChangeEvent>,
    /// Last flush time.
    last_flush: Instant,
    /// Batch duration.
    batch_duration: Duration,
}

impl EventBatcher {
    /// Create a new event batcher.
    fn new(batch_duration: Duration) -> Self {
        Self {
            pending: Vec::new(),
            last_flush: Instant::now(),
            batch_duration,
        }
    }

    /// Add an event to the batch.
    fn add(&mut self, event: ChangeEvent) {
        self.pending.push(event);
    }

    /// Check if the batch should be flushed.
    fn should_flush(&self) -> bool {
        self.last_flush.elapsed() >= self.batch_duration || self.pending.len() >= 100
    }

    /// Flush pending events.
    fn flush(&mut self) -> Vec<ChangeEvent> {
        self.last_flush = Instant::now();
        std::mem::take(&mut self.pending)
    }
}

/// Observable pattern for change notifications.
pub struct ChangeObservable {
    /// Active subscriptions.
    subscriptions: Arc<DashMap<SubscriptionId, SubscriptionData>>,
    /// Event batcher.
    batcher: Arc<parking_lot::Mutex<EventBatcher>>,
}

impl ChangeObservable {
    /// Create a new change observable.
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(DashMap::new()),
            batcher: Arc::new(parking_lot::Mutex::new(EventBatcher::new(
                Duration::from_millis(16), // One animation frame
            ))),
        }
    }

    /// Subscribe to document changes.
    pub fn subscribe(&self, filter: SubscriptionFilter) -> Subscription {
        let id = SubscriptionId::new();
        let (sender, receiver) = mpsc::unbounded_channel();

        self.subscriptions.insert(
            id,
            SubscriptionData {
                filter,
                sender,
            },
        );

        Subscription { id, receiver }
    }

    /// Unsubscribe from changes.
    pub fn unsubscribe(&self, id: SubscriptionId) -> Result<()> {
        self.subscriptions
            .remove(&id)
            .ok_or(StateError::SubscriptionNotFound(format!("{:?}", id)))?;
        Ok(())
    }

    /// Notify subscribers of a change.
    pub fn notify(&self, event: ChangeEvent) {
        // Add to batch
        self.batcher.lock().add(event.clone());

        // Check if we should flush
        if self.batcher.lock().should_flush() {
            self.flush_batch();
        }
    }

    /// Flush the event batch immediately.
    pub fn flush_batch(&self) {
        let events = self.batcher.lock().flush();

        for event in events {
            for entry in self.subscriptions.iter() {
                if entry.value().filter.matches(&event) {
                    // Ignore send errors (subscriber may have dropped)
                    let _ = entry.value().sender.send(event.clone());
                }
            }
        }
    }

    /// Get the number of active subscriptions.
    pub fn subscription_count(&self) -> usize {
        self.subscriptions.len()
    }

    /// Clear all subscriptions.
    pub fn clear(&self) {
        self.subscriptions.clear();
    }
}

impl Default for ChangeObservable {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for DocumentHandle to support reactive updates.
pub trait ReactiveDocument {
    /// Update the document and notify subscribers.
    fn update_reactive<F, T>(
        &self,
        observable: &ChangeObservable,
        f: F,
    ) -> Result<T>
    where
        F: FnOnce(&mut automerge::AutoCommit) -> Result<T>;
}

impl ReactiveDocument for DocumentHandle {
    fn update_reactive<F, T>(
        &self,
        observable: &ChangeObservable,
        f: F,
    ) -> Result<T>
    where
        F: FnOnce(&mut automerge::AutoCommit) -> Result<T>,
    {
        let result = self.update(f)?;

        // Notify subscribers
        let change_hash = self.doc.write().get_heads().iter().map(|h| h.0.to_vec()).flatten().collect();
        let event = ChangeEvent {
            document_id: self.id.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            change_hash,
            path: None,
        };
        observable.notify(event);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document_store::DocumentStore;
    use automerge::{transaction::Transactable, ROOT};

    #[test]
    fn test_subscription_id() {
        let id1 = SubscriptionId::new();
        let id2 = SubscriptionId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_path_matches() {
        assert!(path_matches("users/*/name", "users/alice/name"));
        assert!(path_matches("users/*/name", "users/bob/name"));
        assert!(!path_matches("users/*/name", "users/alice/age"));
        assert!(!path_matches("users/*/name", "posts/1/title"));
        assert!(path_matches("*/*/*", "a/b/c"));
        assert!(!path_matches("*/*/*", "a/b"));
    }

    #[test]
    fn test_subscription_filter_document() {
        let doc_id = DocumentId::new("users", "alice");
        let filter = SubscriptionFilter::Document(doc_id.clone());

        let event = ChangeEvent {
            document_id: doc_id.clone(),
            timestamp: 0,
            change_hash: vec![],
            path: None,
        };

        assert!(filter.matches(&event));

        let event2 = ChangeEvent {
            document_id: DocumentId::new("users", "bob"),
            timestamp: 0,
            change_hash: vec![],
            path: None,
        };

        assert!(!filter.matches(&event2));
    }

    #[test]
    fn test_subscription_filter_path() {
        let doc_id = DocumentId::new("users", "alice");
        let filter = SubscriptionFilter::Path(doc_id.clone(), "profile/*/name".to_string());

        let event = ChangeEvent {
            document_id: doc_id.clone(),
            timestamp: 0,
            change_hash: vec![],
            path: Some("profile/public/name".to_string()),
        };

        assert!(filter.matches(&event));

        let event2 = ChangeEvent {
            document_id: doc_id.clone(),
            timestamp: 0,
            change_hash: vec![],
            path: Some("profile/public/age".to_string()),
        };

        assert!(!filter.matches(&event2));
    }

    #[test]
    fn test_observable_subscribe_unsubscribe() {
        let observable = ChangeObservable::new();
        let doc_id = DocumentId::new("users", "alice");
        let filter = SubscriptionFilter::Document(doc_id);

        let sub = observable.subscribe(filter);
        assert_eq!(observable.subscription_count(), 1);

        observable.unsubscribe(sub.id).unwrap();
        assert_eq!(observable.subscription_count(), 0);
    }

    #[tokio::test]
    async fn test_observable_notify() {
        let observable = ChangeObservable::new();
        let doc_id = DocumentId::new("users", "alice");
        let filter = SubscriptionFilter::Document(doc_id.clone());

        let mut sub = observable.subscribe(filter);

        let event = ChangeEvent {
            document_id: doc_id,
            timestamp: 0,
            change_hash: vec![],
            path: None,
        };

        observable.notify(event.clone());
        observable.flush_batch(); // Force immediate flush

        let received = sub.recv().await.unwrap();
        assert_eq!(received.document_id, event.document_id);
    }

    #[tokio::test]
    async fn test_observable_multiple_subscribers() {
        let observable = ChangeObservable::new();
        let doc_id = DocumentId::new("users", "alice");
        let filter = SubscriptionFilter::Document(doc_id.clone());

        let mut sub1 = observable.subscribe(filter.clone());
        let mut sub2 = observable.subscribe(filter);

        let event = ChangeEvent {
            document_id: doc_id,
            timestamp: 0,
            change_hash: vec![],
            path: None,
        };

        observable.notify(event.clone());
        observable.flush_batch();

        let received1 = sub1.recv().await.unwrap();
        let received2 = sub2.recv().await.unwrap();

        assert_eq!(received1.document_id, event.document_id);
        assert_eq!(received2.document_id, event.document_id);
    }

    #[tokio::test]
    async fn test_reactive_document_update() {
        let store = DocumentStore::new();
        let observable = ChangeObservable::new();
        let doc_id = DocumentId::new("users", "alice");
        let handle = store.create(doc_id.clone()).unwrap();

        let filter = SubscriptionFilter::Document(doc_id);
        let mut sub = observable.subscribe(filter);

        handle
            .update_reactive(&observable, |doc| {
                doc.put(ROOT, "name", "Alice")?;
                Ok(())
            })
            .unwrap();

        observable.flush_batch();

        let event = tokio::time::timeout(Duration::from_secs(1), sub.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(event.document_id.namespace, "users");
        assert_eq!(event.document_id.key, "alice");
    }

    #[test]
    fn test_event_batcher() {
        let mut batcher = EventBatcher::new(Duration::from_millis(100));

        let event = ChangeEvent {
            document_id: DocumentId::new("users", "alice"),
            timestamp: 0,
            change_hash: vec![],
            path: None,
        };

        batcher.add(event.clone());
        assert!(!batcher.should_flush());

        // Add many events to trigger count-based flush
        for _ in 0..100 {
            batcher.add(event.clone());
        }
        assert!(batcher.should_flush());

        let flushed = batcher.flush();
        assert_eq!(flushed.len(), 101);
    }

    #[tokio::test]
    async fn test_subscription_try_recv() {
        let observable = ChangeObservable::new();
        let doc_id = DocumentId::new("users", "alice");
        let filter = SubscriptionFilter::Document(doc_id.clone());

        let mut sub = observable.subscribe(filter);

        // Should be empty initially
        assert!(sub.try_recv().is_err());

        let event = ChangeEvent {
            document_id: doc_id,
            timestamp: 0,
            change_hash: vec![],
            path: None,
        };

        observable.notify(event);
        observable.flush_batch();

        // Should now have an event
        assert!(sub.try_recv().is_ok());
    }

    #[tokio::test]
    async fn test_observable_clear() {
        let observable = ChangeObservable::new();
        let doc_id = DocumentId::new("users", "alice");
        let filter = SubscriptionFilter::Document(doc_id);

        let _sub1 = observable.subscribe(filter.clone());
        let _sub2 = observable.subscribe(filter);

        assert_eq!(observable.subscription_count(), 2);

        observable.clear();
        assert_eq!(observable.subscription_count(), 0);
    }
}
