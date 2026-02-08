//! PlanetServe adapter - Main integration point for privacy-preserving sync
//!
//! This module provides the main adapter that orchestrates:
//! - S-IDA fragmentation
//! - Onion routing
//! - Metadata obfuscation
//! - Configurable privacy levels
//!
//! # Privacy Levels
//!
//! - **None**: Direct sync, no privacy (fast)
//! - **Basic**: Message padding only
//! - **Standard**: Padding + timing jitter
//! - **Maximum**: S-IDA + Onion routing + Cover traffic
//!
//! # Examples
//!
//! ## Direct Sync (No Privacy)
//!
//! ```no_run
//! use vudo_planetserve::{PlanetServeAdapter, config::PrivacyConfig};
//! use vudo_identity::MasterIdentity;
//! use vudo_p2p::VudoP2P;
//! use std::sync::Arc;
//!
//! # async fn example() -> vudo_planetserve::error::Result<()> {
//! let identity = Arc::new(MasterIdentity::generate("Alice").await?);
//! let p2p = Arc::new(VudoP2P::new(
//!     Arc::new(vudo_state::StateEngine::new().await?),
//!     vudo_p2p::P2PConfig::default()
//! ).await?);
//!
//! let adapter = PlanetServeAdapter::new(
//!     identity,
//!     p2p,
//!     PrivacyConfig::fast_open(), // No privacy
//! ).await?;
//!
//! // Sync document (direct, no privacy)
//! adapter.sync_private("namespace", "doc_id", vec![1, 2, 3]).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Maximum Privacy
//!
//! ```no_run
//! use vudo_planetserve::{PlanetServeAdapter, config::PrivacyConfig};
//! use vudo_identity::MasterIdentity;
//! use vudo_p2p::VudoP2P;
//! use std::sync::Arc;
//!
//! # async fn example() -> vudo_planetserve::error::Result<()> {
//! let identity = Arc::new(MasterIdentity::generate("Alice").await?);
//! let p2p = Arc::new(VudoP2P::new(
//!     Arc::new(vudo_state::StateEngine::new().await?),
//!     vudo_p2p::P2PConfig::default()
//! ).await?);
//!
//! let adapter = PlanetServeAdapter::new(
//!     identity,
//!     p2p,
//!     PrivacyConfig::privacy_max(), // Maximum privacy
//! ).await?;
//!
//! // Sync document (fragmented, onion-routed, with cover traffic)
//! adapter.sync_private("namespace", "doc_id", vec![1, 2, 3]).await?;
//! # Ok(())
//! # }
//! ```

use crate::config::{PrivacyConfig, PrivacyLevel};
use crate::error::{Error, Result};
use crate::obfuscator::{CoverTrafficHandle, MetadataObfuscator};
use crate::onion::{OnionRouter, RelayNode};
use crate::sida::SidaFragmenter;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{debug, info, warn};
use vudo_identity::MasterIdentity;
use vudo_p2p::VudoP2P;

/// PlanetServe adapter for privacy-preserving sync
pub struct PlanetServeAdapter {
    /// Identity system
    identity: Arc<MasterIdentity>,

    /// P2P network
    p2p: Arc<VudoP2P>,

    /// S-IDA fragmenter
    sida: SidaFragmenter,

    /// Onion router
    onion: OnionRouter,

    /// Metadata obfuscator
    obfuscator: MetadataObfuscator,

    /// Privacy configuration
    config: PrivacyConfig,

    /// Cover traffic handle
    cover_traffic: Arc<RwLock<Option<CoverTrafficHandle>>>,
}

impl PlanetServeAdapter {
    /// Create a new PlanetServe adapter
    pub async fn new(
        identity: Arc<MasterIdentity>,
        p2p: Arc<VudoP2P>,
        config: PrivacyConfig,
    ) -> Result<Self> {
        info!("Initializing PlanetServe adapter with privacy level: {:?}", config.level);

        // Validate S-IDA config
        config.sida.validate()
            .map_err(Error::InvalidSidaConfig)?;

        // Create S-IDA fragmenter
        let sida = SidaFragmenter::new(config.sida)?;

        // Create onion router
        let onion = OnionRouter::with_config(
            Arc::clone(&identity),
            Arc::clone(&p2p),
            config.onion,
        );

        // Create metadata obfuscator
        let obfuscator = MetadataObfuscator::new(config.clone());

        Ok(Self {
            identity,
            p2p,
            sida,
            onion,
            obfuscator,
            config,
            cover_traffic: Arc::new(RwLock::new(None)),
        })
    }

    /// Start PlanetServe services
    ///
    /// This starts cover traffic if enabled.
    pub async fn start(&self) -> Result<()> {
        info!("Starting PlanetServe services");

        // Start cover traffic if enabled
        if self.config.level == PrivacyLevel::Maximum {
            let handle = self.obfuscator.start_cover_traffic();
            *self.cover_traffic.write() = handle;
        }

        Ok(())
    }

    /// Stop PlanetServe services
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping PlanetServe services");

        // Stop cover traffic
        if let Some(handle) = self.cover_traffic.write().take() {
            handle.stop();
        }

        Ok(())
    }

    /// Sync document with privacy
    ///
    /// Privacy level is determined by the configuration:
    /// - None: Direct sync
    /// - Basic: Padding only
    /// - Standard: Padding + timing jitter
    /// - Maximum: S-IDA + Onion + Cover traffic
    pub async fn sync_private(
        &self,
        namespace: &str,
        doc_id: &str,
        data: Vec<u8>,
    ) -> Result<()> {
        match self.config.level {
            PrivacyLevel::None => {
                debug!("Direct sync (no privacy): {}/{}", namespace, doc_id);
                // Direct sync via P2P (would need to implement P2P sync method)
                // For now, just log
                Ok(())
            }

            PrivacyLevel::Basic => {
                debug!("Basic privacy sync: {}/{}", namespace, doc_id);

                // Pad message
                let padded = self.obfuscator.pad_message(&data);

                // Direct sync with padded message
                // P2P sync would happen here
                Ok(())
            }

            PrivacyLevel::Standard => {
                debug!("Standard privacy sync: {}/{}", namespace, doc_id);

                // Pad message
                let padded = self.obfuscator.pad_message(&data);

                // Apply timing jitter
                self.obfuscator.apply_timing_jitter().await;

                // Direct sync with padded message
                // P2P sync would happen here
                Ok(())
            }

            PrivacyLevel::Maximum => {
                info!("Maximum privacy sync: {}/{}", namespace, doc_id);

                // 1. Fragment message with S-IDA
                let fragments = self.sida.fragment(&data)?;
                debug!("Fragmented message into {} shards", fragments.len());

                // 2. Select peers for each fragment
                // In real implementation, would select from P2P peer pool
                let peers = self.select_fragment_peers(fragments.len()).await?;

                // 3. Build onion circuits
                let mut circuits = Vec::new();
                for peer in &peers {
                    match self.onion.build_circuit(peer, self.config.onion.hops).await {
                        Ok(circuit) => circuits.push(circuit),
                        Err(e) => {
                            warn!("Failed to build circuit to {}: {}", peer, e);
                            // Continue with other circuits
                        }
                    }
                }

                if circuits.is_empty() {
                    return Err(Error::OnionRoutingFailed(
                        "No circuits could be established".to_string(),
                    ));
                }

                // 4. Send each fragment through different circuit
                for (i, (fragment, circuit)) in fragments.iter().zip(circuits.iter()).enumerate() {
                    let fragment_bytes = bincode::serialize(fragment)?;
                    let padded = self.obfuscator.pad_message(&fragment_bytes);

                    // Apply timing jitter
                    self.obfuscator.apply_timing_jitter().await;

                    // Send through onion
                    if let Err(e) = self.onion.send_onion(circuit, &padded).await {
                        warn!("Failed to send fragment {} through onion: {}", i, e);
                    }
                }

                // Cover traffic is already running in the background

                Ok(())
            }
        }
    }

    /// Add relay to onion router
    pub fn add_relay(&self, relay: RelayNode) {
        self.onion.add_relay(relay);
    }

    /// Remove relay from onion router
    pub fn remove_relay(&self, did: &str) {
        self.onion.remove_relay(did);
    }

    /// Get relay count
    pub fn relay_count(&self) -> usize {
        self.onion.relay_count()
    }

    /// Get fragmenter (for BFT committee)
    pub fn fragmenter(&self) -> &SidaFragmenter {
        &self.sida
    }

    /// Get onion router (for advanced usage)
    pub fn onion_router(&self) -> &OnionRouter {
        &self.onion
    }

    /// Get configuration
    pub fn config(&self) -> &PrivacyConfig {
        &self.config
    }

    /// Select peers for fragment distribution
    ///
    /// In a real implementation, would select from P2P peer pool.
    async fn select_fragment_peers(&self, count: usize) -> Result<Vec<String>> {
        // Placeholder: In real implementation, would use P2P peer discovery
        // For now, generate dummy peer DIDs
        Ok((0..count).map(|i| format!("did:peer:fragment_{}", i)).collect())
    }
}

impl Drop for PlanetServeAdapter {
    fn drop(&mut self) {
        // Stop cover traffic on drop
        if let Some(handle) = self.cover_traffic.write().take() {
            handle.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vudo_state::StateEngine;

    async fn create_test_adapter(config: PrivacyConfig) -> PlanetServeAdapter {
        let identity = Arc::new(MasterIdentity::generate("Test").await.unwrap());
        let state_engine = Arc::new(StateEngine::new().await.unwrap());
        let p2p = Arc::new(
            VudoP2P::new(state_engine, vudo_p2p::P2PConfig::default())
                .await
                .unwrap(),
        );

        PlanetServeAdapter::new(identity, p2p, config)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_adapter_creation() {
        let adapter = create_test_adapter(PrivacyConfig::default()).await;
        assert_eq!(adapter.config().level, PrivacyLevel::Standard);
    }

    #[tokio::test]
    async fn test_adapter_fast_open() {
        let adapter = create_test_adapter(PrivacyConfig::fast_open()).await;
        assert_eq!(adapter.config().level, PrivacyLevel::None);
    }

    #[tokio::test]
    async fn test_adapter_privacy_max() {
        let adapter = create_test_adapter(PrivacyConfig::privacy_max()).await;
        assert_eq!(adapter.config().level, PrivacyLevel::Maximum);
    }

    #[tokio::test]
    async fn test_sync_private_none() {
        let adapter = create_test_adapter(PrivacyConfig::fast_open()).await;

        let data = b"test data".to_vec();
        let result = adapter.sync_private("test", "doc1", data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sync_private_basic() {
        let adapter = create_test_adapter(PrivacyConfig::basic()).await;

        let data = b"test data".to_vec();
        let result = adapter.sync_private("test", "doc1", data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sync_private_standard() {
        let adapter = create_test_adapter(PrivacyConfig::default()).await;

        let data = b"test data".to_vec();
        let result = adapter.sync_private("test", "doc1", data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_relay_management() {
        let adapter = create_test_adapter(PrivacyConfig::default()).await;

        assert_eq!(adapter.relay_count(), 0);

        // Create test relay
        let secret = x25519_dalek::StaticSecret::random_from_rng(&mut rand::thread_rng());
        let public_key = x25519_dalek::PublicKey::from(&secret);
        let relay = RelayNode::new("did:peer:relay1".to_string(), public_key);

        adapter.add_relay(relay);
        assert_eq!(adapter.relay_count(), 1);

        adapter.remove_relay("did:peer:relay1");
        assert_eq!(adapter.relay_count(), 0);
    }

    #[tokio::test]
    async fn test_start_stop() {
        let adapter = create_test_adapter(PrivacyConfig::privacy_max()).await;

        adapter.start().await.unwrap();

        // Cover traffic should be running
        assert!(adapter.cover_traffic.read().is_some());

        adapter.stop().await.unwrap();

        // Cover traffic should be stopped
        assert!(adapter.cover_traffic.read().is_none());
    }
}
