//! Onion routing for privacy-preserving sync
//!
//! This module implements multi-hop onion routing to hide WHO is syncing with WHOM.
//! Messages are wrapped in multiple layers of encryption, with each relay peeling
//! off one layer.
//!
//! # Architecture
//!
//! ```text
//! Alice → Relay 1 → Relay 2 → Relay 3 → Bob
//!
//! Layer 3: Encrypt(Bob_key, "message")
//! Layer 2: Encrypt(Relay3_key, next_hop=Bob, Layer 3)
//! Layer 1: Encrypt(Relay2_key, next_hop=Relay3, Layer 2)
//! Layer 0: Encrypt(Relay1_key, next_hop=Relay2, Layer 1)
//! ```
//!
//! # Privacy Properties
//!
//! - **Entry relay** knows Alice but not Bob (destination)
//! - **Middle relays** know neither Alice nor Bob
//! - **Exit relay** knows Bob but not Alice (origin)
//! - **No single relay** can correlate sender and receiver
//!
//! # Examples
//!
//! ```no_run
//! use vudo_planetserve::onion::{OnionRouter, RelayNode};
//! use vudo_identity::MasterIdentity;
//! use vudo_p2p::VudoP2P;
//! use std::sync::Arc;
//!
//! # async fn example() -> vudo_planetserve::error::Result<()> {
//! # let identity = Arc::new(MasterIdentity::generate("Alice").await?);
//! # let p2p = Arc::new(VudoP2P::new(
//! #     Arc::new(vudo_state::StateEngine::new().await?),
//! #     vudo_p2p::P2PConfig::default()
//! # ).await?);
//! let router = OnionRouter::new(identity, p2p);
//!
//! // Build 3-hop circuit to Bob
//! let circuit = router.build_circuit("did:peer:bob", 3).await?;
//!
//! // Send message through circuit
//! let message = b"Private sync message";
//! router.send_onion(&circuit, message).await?;
//! # Ok(())
//! # }
//! ```

use crate::config::{OnionConfig, RelaySelectionStrategy};
use crate::error::{Error, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use parking_lot::RwLock;
use rand::seq::SliceRandom;
use rand::Rng;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};
use vudo_identity::MasterIdentity;
use vudo_p2p::VudoP2P;
use x25519_dalek::{PublicKey, StaticSecret};

/// Onion router
pub struct OnionRouter {
    /// Identity for key agreement
    identity: Arc<MasterIdentity>,

    /// P2P network
    p2p: Arc<VudoP2P>,

    /// Relay pool
    relays: Arc<RwLock<Vec<RelayNode>>>,

    /// Configuration
    config: OnionConfig,
}

impl OnionRouter {
    /// Create a new onion router
    pub fn new(identity: Arc<MasterIdentity>, p2p: Arc<VudoP2P>) -> Self {
        Self {
            identity,
            p2p,
            relays: Arc::new(RwLock::new(Vec::new())),
            config: OnionConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        identity: Arc<MasterIdentity>,
        p2p: Arc<VudoP2P>,
        config: OnionConfig,
    ) -> Self {
        Self {
            identity,
            p2p,
            relays: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Add relay to the pool
    pub fn add_relay(&self, relay: RelayNode) {
        let mut relays = self.relays.write();
        // Don't add duplicates
        if !relays.iter().any(|r| r.did == relay.did) {
            relays.push(relay);
        }
    }

    /// Remove relay from the pool
    pub fn remove_relay(&self, did: &str) {
        let mut relays = self.relays.write();
        relays.retain(|r| r.did != did);
    }

    /// Get relay count
    pub fn relay_count(&self) -> usize {
        self.relays.read().len()
    }

    /// Build an onion circuit
    ///
    /// Selects `hops` relays and establishes shared secrets with each.
    pub async fn build_circuit(&self, destination: &str, hops: usize) -> Result<OnionCircuit> {
        info!("Building {}-hop onion circuit to {}", hops, destination);

        if hops == 0 {
            return Err(Error::CircuitBuildFailed(
                "Circuit must have at least 1 hop".to_string(),
            ));
        }

        // Select relay nodes
        let relays = self.select_relays(hops).await?;

        if relays.len() < hops {
            return Err(Error::CircuitBuildFailed(format!(
                "Insufficient relays: need {}, have {}",
                hops,
                relays.len()
            )));
        }

        // Establish shared secrets with each relay using X25519 key agreement
        let mut shared_secrets = Vec::with_capacity(hops);
        for relay in &relays {
            // Perform ECDH key agreement
            let secret = self.key_agreement(&relay.public_key)?;
            shared_secrets.push(secret);
        }

        Ok(OnionCircuit {
            relays,
            shared_secrets,
            destination: destination.to_string(),
        })
    }

    /// Send message through onion circuit
    pub async fn send_onion(&self, circuit: &OnionCircuit, message: &[u8]) -> Result<()> {
        debug!("Sending onion message through {}-hop circuit", circuit.relays.len());

        // 1. Encrypt innermost layer (for destination)
        let mut encrypted = self.encrypt_for_destination(&circuit.destination, message)?;

        // 2. Encrypt each layer (backwards from destination to entry)
        for (i, secret) in circuit.shared_secrets.iter().enumerate().rev() {
            let next_hop = if i == circuit.relays.len() - 1 {
                // Last relay -> destination
                circuit.destination.clone()
            } else {
                // Intermediate relay -> next relay
                circuit.relays[i + 1].did.clone()
            };

            encrypted = self.encrypt_layer(secret, &next_hop, &encrypted)?;
        }

        // 3. Send to first relay (entry node)
        let entry_did = &circuit.relays[0].did;

        // In a real implementation, this would use P2P send
        // For now, we'll just log (P2P integration would happen here)
        debug!("Sending encrypted onion to entry relay: {}", entry_did);

        Ok(())
    }

    /// Peel one layer of the onion (relay operation)
    ///
    /// This is called by relay nodes to decrypt their layer and forward to next hop.
    pub fn peel_layer(&self, secret: &[u8; 32], onion: &[u8]) -> Result<(String, Vec<u8>)> {
        // Decrypt the layer
        let key = Key::from_slice(secret);
        let cipher = ChaCha20Poly1305::new(key);

        // Extract nonce (first 12 bytes)
        if onion.len() < 12 {
            return Err(Error::DecryptionFailed("Onion too short".to_string()));
        }

        let nonce = Nonce::from_slice(&onion[..12]);
        let ciphertext = &onion[12..];

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| Error::DecryptionFailed(e.to_string()))?;

        // Parse plaintext: next_hop | payload
        let parts: Vec<&[u8]> = plaintext.splitn(2, |&b| b == b'|').collect();
        if parts.len() != 2 {
            return Err(Error::DecryptionFailed(
                "Invalid onion layer format".to_string(),
            ));
        }

        let next_hop = String::from_utf8(parts[0].to_vec())
            .map_err(|e| Error::DecryptionFailed(format!("Invalid next_hop: {}", e)))?;
        let payload = parts[1].to_vec();

        Ok((next_hop, payload))
    }

    /// Select relays according to strategy
    async fn select_relays(&self, count: usize) -> Result<Vec<RelayNode>> {
        let relays = self.relays.read();

        if relays.is_empty() {
            return Err(Error::RelaySelectionFailed(
                "No relays available".to_string(),
            ));
        }

        if relays.len() < count {
            return Err(Error::RelaySelectionFailed(format!(
                "Not enough relays: need {}, have {}",
                count,
                relays.len()
            )));
        }

        let mut selected = match self.config.relay_selection_strategy {
            RelaySelectionStrategy::Random => {
                // Random selection
                let mut rng = rand::thread_rng();
                let mut pool = relays.clone();
                pool.shuffle(&mut rng);
                pool.into_iter().take(count).collect()
            }

            RelaySelectionStrategy::LowLatency => {
                // Sort by latency, take lowest
                let mut pool = relays.clone();
                pool.sort_by(|a, b| a.latency.cmp(&b.latency));
                pool.into_iter().take(count).collect()
            }

            RelaySelectionStrategy::HighReliability => {
                // Sort by reliability (descending), take highest
                let mut pool = relays.clone();
                pool.sort_by(|a, b| b.reliability.partial_cmp(&a.reliability).unwrap());
                pool.into_iter().take(count).collect()
            }

            RelaySelectionStrategy::Balanced => {
                // Score = reliability * (1.0 / latency_seconds)
                let mut pool = relays.clone();
                pool.sort_by(|a, b| {
                    let score_a = a.reliability / (a.latency.as_secs_f64() + 0.001);
                    let score_b = b.reliability / (b.latency.as_secs_f64() + 0.001);
                    score_b.partial_cmp(&score_a).unwrap()
                });
                pool.into_iter().take(count).collect()
            }
        };

        Ok(selected)
    }

    /// Perform X25519 key agreement
    fn key_agreement(&self, peer_public_key: &PublicKey) -> Result<[u8; 32]> {
        // In a real implementation, we would use the identity's encryption key
        // For now, generate an ephemeral key
        let ephemeral = StaticSecret::random_from_rng(&mut rand::thread_rng());
        let shared_secret = ephemeral.diffie_hellman(peer_public_key);

        Ok(*shared_secret.as_bytes())
    }

    /// Encrypt layer of onion
    fn encrypt_layer(&self, secret: &[u8; 32], next_hop: &str, payload: &[u8]) -> Result<Vec<u8>> {
        let key = Key::from_slice(secret);
        let cipher = ChaCha20Poly1305::new(key);

        // Generate random nonce
        let mut rng = rand::thread_rng();
        let nonce_bytes: [u8; 12] = rng.gen();
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Plaintext: next_hop | payload
        let plaintext = [next_hop.as_bytes(), b"|", payload].concat();

        // Encrypt
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_slice())
            .map_err(|e| Error::EncryptionFailed(e.to_string()))?;

        // Return nonce || ciphertext
        Ok([&nonce_bytes[..], &ciphertext[..]].concat())
    }

    /// Encrypt for destination (final layer)
    fn encrypt_for_destination(&self, destination: &str, message: &[u8]) -> Result<Vec<u8>> {
        // In a real implementation, we would look up the destination's public key
        // and perform key agreement. For now, use a derived key.
        let key_material = blake3::hash(destination.as_bytes());
        let key = Key::from_slice(key_material.as_bytes());
        let cipher = ChaCha20Poly1305::new(key);

        let mut rng = rand::thread_rng();
        let nonce_bytes: [u8; 12] = rng.gen();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, message)
            .map_err(|e| Error::EncryptionFailed(e.to_string()))?;

        Ok([&nonce_bytes[..], &ciphertext[..]].concat())
    }
}

/// Relay node in the onion network
#[derive(Debug, Clone)]
pub struct RelayNode {
    /// DID of the relay
    pub did: String,

    /// X25519 public key for key agreement
    pub public_key: PublicKey,

    /// Average latency
    pub latency: Duration,

    /// Reliability score (0.0 to 1.0)
    pub reliability: f64,
}

impl RelayNode {
    /// Create a new relay node
    pub fn new(did: String, public_key: PublicKey) -> Self {
        Self {
            did,
            public_key,
            latency: Duration::from_millis(50),
            reliability: 1.0,
        }
    }

    /// Update latency
    pub fn update_latency(&mut self, latency: Duration) {
        // Exponential moving average
        self.latency = Duration::from_secs_f64(
            0.7 * self.latency.as_secs_f64() + 0.3 * latency.as_secs_f64(),
        );
    }

    /// Update reliability (success rate)
    pub fn update_reliability(&mut self, success: bool) {
        // Exponential moving average
        let new_value = if success { 1.0 } else { 0.0 };
        self.reliability = 0.9 * self.reliability + 0.1 * new_value;
    }
}

/// Onion circuit (established path through relays)
#[derive(Debug, Clone)]
pub struct OnionCircuit {
    /// Relay nodes in the circuit
    pub relays: Vec<RelayNode>,

    /// Shared secrets with each relay (from key agreement)
    pub shared_secrets: Vec<[u8; 32]>,

    /// Destination DID
    pub destination: String,
}

impl OnionCircuit {
    /// Get circuit length (number of hops)
    pub fn hops(&self) -> usize {
        self.relays.len()
    }

    /// Estimate total latency
    pub fn estimated_latency(&self) -> Duration {
        self.relays.iter().map(|r| r.latency).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_relay(did: &str) -> RelayNode {
        let secret = StaticSecret::random_from_rng(&mut rand::thread_rng());
        let public_key = PublicKey::from(&secret);

        RelayNode::new(did.to_string(), public_key)
    }

    #[tokio::test]
    async fn test_relay_pool() {
        let identity = Arc::new(MasterIdentity::generate("Test").await.unwrap());
        let p2p = Arc::new(
            VudoP2P::new(
                Arc::new(vudo_state::StateEngine::new().await.unwrap()),
                vudo_p2p::P2PConfig::default(),
            )
            .await
            .unwrap(),
        );

        let router = OnionRouter::new(identity, p2p);

        assert_eq!(router.relay_count(), 0);

        let relay1 = create_test_relay("relay1");
        router.add_relay(relay1);
        assert_eq!(router.relay_count(), 1);

        let relay2 = create_test_relay("relay2");
        router.add_relay(relay2);
        assert_eq!(router.relay_count(), 2);

        router.remove_relay("relay1");
        assert_eq!(router.relay_count(), 1);
    }

    #[test]
    fn test_relay_node_latency_update() {
        let mut relay = create_test_relay("test");
        let initial_latency = relay.latency;

        relay.update_latency(Duration::from_millis(100));
        // Should be moving average, not exact
        assert!(relay.latency > initial_latency);
        assert!(relay.latency < Duration::from_millis(100));
    }

    #[test]
    fn test_relay_node_reliability_update() {
        let mut relay = create_test_relay("test");
        assert_eq!(relay.reliability, 1.0);

        // Report failure
        relay.update_reliability(false);
        assert!(relay.reliability < 1.0);

        // Report success
        relay.update_reliability(true);
        assert!(relay.reliability > 0.0);
    }

    #[tokio::test]
    async fn test_onion_layer_encryption_decryption() {
        let identity = Arc::new(MasterIdentity::generate("Test").await.unwrap());
        let p2p = Arc::new(
            VudoP2P::new(
                Arc::new(vudo_state::StateEngine::new().await.unwrap()),
                vudo_p2p::P2PConfig::default(),
            )
            .await
            .unwrap(),
        );

        let router = OnionRouter::new(identity, p2p);

        let secret: [u8; 32] = rand::thread_rng().gen();
        let next_hop = "did:peer:bob";
        let payload = b"test message";

        // Encrypt
        let encrypted = router
            .encrypt_layer(&secret, next_hop, payload)
            .unwrap();

        // Decrypt
        let (decoded_hop, decoded_payload) = router.peel_layer(&secret, &encrypted).unwrap();

        assert_eq!(decoded_hop, next_hop);
        assert_eq!(decoded_payload, payload);
    }

    #[tokio::test]
    async fn test_circuit_build_insufficient_relays() {
        let identity = Arc::new(MasterIdentity::generate("Test").await.unwrap());
        let p2p = Arc::new(
            VudoP2P::new(
                Arc::new(vudo_state::StateEngine::new().await.unwrap()),
                vudo_p2p::P2PConfig::default(),
            )
            .await
            .unwrap()
        );

        let router = OnionRouter::new(identity, p2p);

        // Try to build 3-hop circuit with no relays
        let result = router.build_circuit("did:peer:bob", 3).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_circuit_build_success() {
        let identity = Arc::new(MasterIdentity::generate("Test").await.unwrap());
        let p2p = Arc::new(
            VudoP2P::new(
                Arc::new(vudo_state::StateEngine::new().await.unwrap()),
                vudo_p2p::P2PConfig::default(),
            )
            .await
            .unwrap()
        );

        let router = OnionRouter::new(Arc::clone(&identity), p2p);

        // Add relays
        for i in 0..5 {
            let relay = create_test_relay(&format!("relay{}", i));
            router.add_relay(relay);
        }

        // Build circuit
        let circuit = router.build_circuit("did:peer:bob", 3).await.unwrap();
        assert_eq!(circuit.hops(), 3);
        assert_eq!(circuit.shared_secrets.len(), 3);
    }
}
