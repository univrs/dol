//! DID resolver for P2P peer verification
//!
//! This module provides DID resolution capabilities with local caching
//! and support for did:peer:2 derivation.
//!
//! # Examples
//!
//! ```
//! use vudo_identity::{DidResolver, Did};
//! use ed25519_dalek::SigningKey;
//! use x25519_dalek::{StaticSecret, PublicKey};
//! use rand::rngs::OsRng;
//!
//! # async fn example() -> vudo_identity::error::Result<()> {
//! let resolver = DidResolver::new();
//!
//! // Create and resolve a did:peer
//! let signing_key = SigningKey::generate(&mut OsRng);
//! let encryption_secret = StaticSecret::random_from_rng(&mut OsRng);
//! let encryption_public = PublicKey::from(&encryption_secret);
//! let did = Did::from_keys(signing_key.verifying_key(), &encryption_public)?;
//!
//! let doc = resolver.resolve(&did).await?;
//! println!("Resolved DID document: {}", doc.id);
//! # Ok(())
//! # }
//! ```

use crate::did::{Did, DidDocument};
use crate::error::{Error, Result};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

/// DID resolver with caching
#[derive(Debug, Clone)]
pub struct DidResolver {
    /// Local cache of DID documents
    cache: Arc<DashMap<String, CachedDocument>>,

    /// Cache TTL (time-to-live) in seconds
    cache_ttl: u64,
}

impl DidResolver {
    /// Create a new DID resolver
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            cache_ttl: 3600, // 1 hour default
        }
    }

    /// Create a new DID resolver with custom cache TTL
    pub fn with_ttl(cache_ttl: u64) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            cache_ttl,
        }
    }

    /// Resolve DID to document
    pub async fn resolve(&self, did: &Did) -> Result<DidDocument> {
        let did_str = did.as_str();

        // Check local cache
        if let Some(cached) = self.cache.get(did_str) {
            if !cached.is_expired(self.cache_ttl) {
                debug!("DID resolved from cache: {}", did_str);
                return Ok(cached.document.clone());
            } else {
                debug!("Cached DID document expired: {}", did_str);
            }
        }

        // For did:peer, derive document from DID itself
        if did.method() == "peer" {
            let doc = did.to_document();
            self.cache_document(did_str, doc.clone());
            debug!("DID resolved from derivation: {}", did_str);
            return Ok(doc);
        }

        // For other methods, would query P2P network here
        // For now, return error
        Err(Error::Resolution(format!(
            "Resolution not supported for method: {}",
            did.method()
        )))
    }

    /// Resolve DID from string
    pub async fn resolve_str(&self, did_str: &str) -> Result<DidDocument> {
        let did = Did::parse(did_str)?;
        self.resolve(&did).await
    }

    /// Cache a DID document
    pub fn cache_document(&self, did: &str, document: DidDocument) {
        let cached = CachedDocument {
            document,
            cached_at: Self::current_timestamp(),
        };
        self.cache.insert(did.to_string(), cached);
        debug!("Cached DID document: {}", did);
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        self.cache.clear();
        debug!("DID resolver cache cleared");
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Remove expired entries from cache
    pub fn prune_cache(&self) {
        let now = Self::current_timestamp();
        self.cache.retain(|_, cached| {
            let age = now.saturating_sub(cached.cached_at);
            age < self.cache_ttl
        });
        debug!("Pruned expired entries from DID cache");
    }

    /// Get current Unix timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

impl Default for DidResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Cached DID document
#[derive(Debug, Clone)]
struct CachedDocument {
    document: DidDocument,
    cached_at: u64,
}

impl CachedDocument {
    /// Check if cache entry is expired
    fn is_expired(&self, ttl: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let age = now.saturating_sub(self.cached_at);
        age >= ttl
    }
}

/// Batch DID resolver for efficient resolution of multiple DIDs
#[derive(Debug, Clone)]
pub struct BatchDidResolver {
    resolver: DidResolver,
}

impl BatchDidResolver {
    /// Create a new batch resolver
    pub fn new(resolver: DidResolver) -> Self {
        Self { resolver }
    }

    /// Resolve multiple DIDs concurrently
    pub async fn resolve_batch(&self, dids: Vec<Did>) -> Result<Vec<DidDocument>> {
        let mut handles = Vec::new();

        for did in dids {
            let resolver = self.resolver.clone();
            let handle = tokio::spawn(async move { resolver.resolve(&did).await });
            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(doc)) => results.push(doc),
                Ok(Err(e)) => return Err(e),
                Err(e) => {
                    return Err(Error::Resolution(format!("Task join error: {}", e)))
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use x25519_dalek::{PublicKey, StaticSecret};

    fn create_test_did() -> Did {
        let signing_key = SigningKey::generate(&mut OsRng);
        let encryption_secret = StaticSecret::random_from_rng(&mut OsRng);
        let encryption_public = PublicKey::from(&encryption_secret);
        Did::from_keys(signing_key.verifying_key(), &encryption_public).unwrap()
    }

    #[tokio::test]
    async fn test_resolver_creation() {
        let resolver = DidResolver::new();
        assert_eq!(resolver.cache_size(), 0);
    }

    #[tokio::test]
    async fn test_did_peer_resolution() {
        let resolver = DidResolver::new();
        let did = create_test_did();

        let doc = resolver.resolve(&did).await.unwrap();
        assert_eq!(doc.id, did.as_str());
        assert_eq!(doc.authentication.len(), 1);
        assert_eq!(doc.key_agreement.len(), 1);
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let resolver = DidResolver::new();
        let did = create_test_did();

        // First resolution
        let doc1 = resolver.resolve(&did).await.unwrap();

        // Second resolution should hit cache
        let doc2 = resolver.resolve(&did).await.unwrap();

        assert_eq!(doc1.id, doc2.id);
        assert_eq!(resolver.cache_size(), 1);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        use std::time::Duration;

        let resolver = DidResolver::with_ttl(1); // 1 second TTL
        let did = create_test_did();

        // Resolve and cache
        resolver.resolve(&did).await.unwrap();
        assert_eq!(resolver.cache_size(), 1);

        // Wait for expiration
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Prune cache
        resolver.prune_cache();
        assert_eq!(resolver.cache_size(), 0);
    }

    #[tokio::test]
    async fn test_batch_resolution() {
        let resolver = DidResolver::new();
        let batch_resolver = BatchDidResolver::new(resolver);

        let dids = vec![create_test_did(), create_test_did(), create_test_did()];

        let docs = batch_resolver.resolve_batch(dids.clone()).await.unwrap();
        assert_eq!(docs.len(), 3);

        for (did, doc) in dids.iter().zip(docs.iter()) {
            assert_eq!(did.as_str(), doc.id);
        }
    }

    #[tokio::test]
    async fn test_resolve_from_string() {
        let resolver = DidResolver::new();
        let did = create_test_did();
        let did_str = did.as_str();

        let doc = resolver.resolve_str(did_str).await.unwrap();
        assert_eq!(doc.id, did_str);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let resolver = DidResolver::new();
        let did1 = create_test_did();
        let did2 = create_test_did();

        resolver.resolve(&did1).await.unwrap();
        resolver.resolve(&did2).await.unwrap();
        assert_eq!(resolver.cache_size(), 2);

        resolver.clear_cache();
        assert_eq!(resolver.cache_size(), 0);
    }
}
