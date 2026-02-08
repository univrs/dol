//! Gossip overlay for presence and document discovery.

use crate::error::{P2PError, Result};
use crate::sync_protocol::PeerId;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Gossip topic identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Topic(String);

impl Topic {
    /// Create a new topic.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Create a topic for a document.
    pub fn document(namespace: &str, id: &str) -> Self {
        Self(format!("doc:{}:{}", namespace, id))
    }

    /// Create a topic for presence.
    pub fn presence() -> Self {
        Self("presence".to_string())
    }

    /// Get the topic name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for Topic {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Topic {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Gossip message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GossipMessage {
    /// Presence announcement.
    Presence {
        /// Peer ID.
        peer_id: PeerId,
        /// Available documents.
        documents: Vec<(String, String)>, // (namespace, id)
        /// Timestamp.
        timestamp: u64,
    },

    /// Document announcement.
    DocumentAnnouncement {
        /// Peer ID.
        peer_id: PeerId,
        /// Document namespace.
        namespace: String,
        /// Document ID.
        id: String,
        /// Document version.
        version: u64,
        /// Timestamp.
        timestamp: u64,
    },

    /// Document update notification.
    DocumentUpdate {
        /// Peer ID.
        peer_id: PeerId,
        /// Document namespace.
        namespace: String,
        /// Document ID.
        id: String,
        /// New version.
        version: u64,
        /// Timestamp.
        timestamp: u64,
    },
}

impl GossipMessage {
    /// Serialize message to bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(P2PError::from)
    }

    /// Deserialize message from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(P2PError::from)
    }
}

/// Subscription handle.
pub struct Subscription {
    /// Subscription ID.
    id: SubscriptionId,
    /// Message receiver.
    rx: mpsc::UnboundedReceiver<GossipMessage>,
}

impl Subscription {
    /// Receive next message.
    pub async fn recv(&mut self) -> Option<GossipMessage> {
        self.rx.recv().await
    }

    /// Get subscription ID.
    pub fn id(&self) -> SubscriptionId {
        self.id
    }
}

/// Subscription ID.
pub type SubscriptionId = u64;

/// Gossip overlay manager.
pub struct GossipOverlay {
    /// Topic subscriptions.
    subscriptions: Arc<RwLock<HashMap<Topic, Vec<(SubscriptionId, mpsc::UnboundedSender<GossipMessage>)>>>>,
    /// Next subscription ID.
    next_sub_id: Arc<RwLock<SubscriptionId>>,
    /// Peer interests (which peers are interested in which topics).
    peer_interests: Arc<RwLock<HashMap<PeerId, HashSet<Topic>>>>,
}

impl GossipOverlay {
    /// Create a new gossip overlay.
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            next_sub_id: Arc::new(RwLock::new(0)),
            peer_interests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Subscribe to a topic.
    pub async fn subscribe(&self, topic: Topic) -> Result<Subscription> {
        let id = {
            let mut next_id = self.next_sub_id.write();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let (tx, rx) = mpsc::unbounded_channel();

        self.subscriptions
            .write()
            .entry(topic.clone())
            .or_insert_with(Vec::new)
            .push((id, tx));

        info!("Subscribed to topic: {} (id: {})", topic.as_str(), id);

        Ok(Subscription { id, rx })
    }

    /// Unsubscribe from a topic.
    pub async fn unsubscribe(&self, subscription_id: SubscriptionId) -> Result<()> {
        let mut subscriptions = self.subscriptions.write();

        for (topic, subs) in subscriptions.iter_mut() {
            if let Some(pos) = subs.iter().position(|(id, _)| *id == subscription_id) {
                subs.remove(pos);
                info!("Unsubscribed from topic: {} (id: {})", topic.as_str(), subscription_id);
                return Ok(());
            }
        }

        Err(P2PError::Internal(format!(
            "Subscription not found: {}",
            subscription_id
        )))
    }

    /// Publish a message to a topic.
    pub async fn publish(&self, topic: Topic, message: GossipMessage) -> Result<()> {
        debug!("Publishing to topic: {}", topic.as_str());

        let subscriptions = self.subscriptions.read();

        if let Some(subs) = subscriptions.get(&topic) {
            for (id, tx) in subs {
                if tx.send(message.clone()).is_err() {
                    warn!("Failed to send to subscriber {}", id);
                }
            }
        }

        Ok(())
    }

    /// Announce document presence.
    pub async fn announce_document(&self, peer_id: PeerId, namespace: &str, id: &str, version: u64) -> Result<()> {
        let topic = Topic::document(namespace, id);

        let message = GossipMessage::DocumentAnnouncement {
            peer_id,
            namespace: namespace.to_string(),
            id: id.to_string(),
            version,
            timestamp: current_timestamp(),
        };

        self.publish(topic, message).await
    }

    /// Announce document update.
    pub async fn announce_update(&self, peer_id: PeerId, namespace: &str, id: &str, version: u64) -> Result<()> {
        let topic = Topic::document(namespace, id);

        let message = GossipMessage::DocumentUpdate {
            peer_id,
            namespace: namespace.to_string(),
            id: id.to_string(),
            version,
            timestamp: current_timestamp(),
        };

        self.publish(topic, message).await
    }

    /// Announce presence with available documents.
    pub async fn announce_presence(&self, peer_id: PeerId, documents: Vec<(String, String)>) -> Result<()> {
        let topic = Topic::presence();

        let message = GossipMessage::Presence {
            peer_id,
            documents,
            timestamp: current_timestamp(),
        };

        self.publish(topic, message).await
    }

    /// Subscribe to document updates.
    pub async fn subscribe_document(&self, namespace: &str, id: &str) -> Result<Subscription> {
        let topic = Topic::document(namespace, id);
        self.subscribe(topic).await
    }

    /// Subscribe to presence announcements.
    pub async fn subscribe_presence(&self) -> Result<Subscription> {
        let topic = Topic::presence();
        self.subscribe(topic).await
    }

    /// Track peer interest in a topic.
    pub fn add_peer_interest(&self, peer_id: &PeerId, topic: Topic) {
        self.peer_interests
            .write()
            .entry(peer_id.clone())
            .or_insert_with(HashSet::new)
            .insert(topic);
    }

    /// Remove peer interest.
    pub fn remove_peer_interest(&self, peer_id: &PeerId, topic: &Topic) {
        if let Some(interests) = self.peer_interests.write().get_mut(peer_id) {
            interests.remove(topic);
        }
    }

    /// Get peers interested in a topic.
    pub fn get_interested_peers(&self, topic: &Topic) -> Vec<PeerId> {
        self.peer_interests
            .read()
            .iter()
            .filter(|(_, interests)| interests.contains(topic))
            .map(|(peer_id, _)| peer_id.clone())
            .collect()
    }

    /// Get subscription count.
    pub fn subscription_count(&self) -> usize {
        self.subscriptions.read().values().map(|v| v.len()).sum()
    }

    /// Get topic count.
    pub fn topic_count(&self) -> usize {
        self.subscriptions.read().len()
    }
}

impl Default for GossipOverlay {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current timestamp in milliseconds.
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_creation() {
        let topic = Topic::new("test");
        assert_eq!(topic.as_str(), "test");

        let doc_topic = Topic::document("users", "alice");
        assert_eq!(doc_topic.as_str(), "doc:users:alice");

        let presence_topic = Topic::presence();
        assert_eq!(presence_topic.as_str(), "presence");
    }

    #[tokio::test]
    async fn test_subscribe_unsubscribe() {
        let overlay = GossipOverlay::new();

        let topic = Topic::new("test");
        let sub = overlay.subscribe(topic.clone()).await.unwrap();
        assert_eq!(overlay.subscription_count(), 1);

        overlay.unsubscribe(sub.id()).await.unwrap();
        assert_eq!(overlay.subscription_count(), 0);
    }

    #[tokio::test]
    async fn test_publish_subscribe() {
        let overlay = GossipOverlay::new();

        let topic = Topic::new("test");
        let mut sub = overlay.subscribe(topic.clone()).await.unwrap();

        let message = GossipMessage::Presence {
            peer_id: "peer1".to_string(),
            documents: vec![],
            timestamp: 12345,
        };

        overlay.publish(topic, message.clone()).await.unwrap();

        let received = sub.recv().await.unwrap();
        match received {
            GossipMessage::Presence { peer_id, .. } => {
                assert_eq!(peer_id, "peer1");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[tokio::test]
    async fn test_announce_document() {
        let overlay = GossipOverlay::new();

        let mut sub = overlay.subscribe_document("users", "alice").await.unwrap();

        overlay
            .announce_document("peer1".to_string(), "users", "alice", 1)
            .await
            .unwrap();

        let received = sub.recv().await.unwrap();
        match received {
            GossipMessage::DocumentAnnouncement {
                peer_id,
                namespace,
                id,
                version,
                ..
            } => {
                assert_eq!(peer_id, "peer1");
                assert_eq!(namespace, "users");
                assert_eq!(id, "alice");
                assert_eq!(version, 1);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_peer_interests() {
        let overlay = GossipOverlay::new();

        let topic1 = Topic::new("topic1");
        let topic2 = Topic::new("topic2");

        overlay.add_peer_interest(&"peer1".to_string(), topic1.clone());
        overlay.add_peer_interest(&"peer1".to_string(), topic2.clone());
        overlay.add_peer_interest(&"peer2".to_string(), topic1.clone());

        let interested = overlay.get_interested_peers(&topic1);
        assert_eq!(interested.len(), 2);

        overlay.remove_peer_interest(&"peer1".to_string(), &topic1);
        let interested = overlay.get_interested_peers(&topic1);
        assert_eq!(interested.len(), 1);
    }

    #[test]
    fn test_gossip_message_serialization() {
        let msg = GossipMessage::Presence {
            peer_id: "peer1".to_string(),
            documents: vec![("users".to_string(), "alice".to_string())],
            timestamp: 12345,
        };

        let bytes = msg.to_bytes().unwrap();
        let decoded = GossipMessage::from_bytes(&bytes).unwrap();

        match decoded {
            GossipMessage::Presence {
                peer_id, documents, ..
            } => {
                assert_eq!(peer_id, "peer1");
                assert_eq!(documents.len(), 1);
            }
            _ => panic!("Wrong message type"),
        }
    }
}
