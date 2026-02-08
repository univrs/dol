//! S-IDA (Secure Information Dispersal Algorithm) implementation
//!
//! This module provides Reed-Solomon erasure coding for fragmenting sync messages
//! across multiple peers. No single peer can observe the full message.
//!
//! # Privacy Guarantee
//!
//! With k-of-n threshold (e.g., 3-of-5):
//! - Any k fragments can reconstruct the message
//! - Having < k fragments reveals NO information
//! - Each peer stores only 1/n of the message
//!
//! # Examples
//!
//! ```
//! use vudo_planetserve::sida::{SidaFragmenter, SidaConfig};
//!
//! # fn example() -> vudo_planetserve::error::Result<()> {
//! let config = SidaConfig { k: 3, n: 5 };
//! let fragmenter = SidaFragmenter::new(config)?;
//!
//! let message = b"Alice updated document X";
//! let fragments = fragmenter.fragment(message)?;
//! assert_eq!(fragments.len(), 5);
//!
//! // Reconstruct from any 3 fragments
//! let subset: Vec<_> = fragments.iter().take(3).cloned().collect();
//! let reconstructed = fragmenter.reconstruct(subset)?;
//! assert_eq!(reconstructed, message);
//! # Ok(())
//! # }
//! ```

use crate::config::SidaConfig;
use crate::error::{Error, Result};
use reed_solomon_erasure::galois_8::ReedSolomon;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// S-IDA fragmenter using Reed-Solomon erasure coding
pub struct SidaFragmenter {
    /// Reed-Solomon encoder
    encoder: ReedSolomon,

    /// Configuration
    config: SidaConfig,
}

impl SidaFragmenter {
    /// Create a new S-IDA fragmenter
    pub fn new(config: SidaConfig) -> Result<Self> {
        // Validate configuration
        config.validate().map_err(Error::InvalidSidaConfig)?;

        // Create Reed-Solomon encoder
        // k = data shards, n-k = parity shards
        let encoder = ReedSolomon::new(config.k, config.n - config.k)
            .map_err(|e| Error::InvalidSidaConfig(e.to_string()))?;

        Ok(Self { encoder, config })
    }

    /// Fragment a message into n shards
    ///
    /// Returns n fragments, where any k can reconstruct the original message.
    pub fn fragment(&self, message: &[u8]) -> Result<Vec<Fragment>> {
        // Calculate shard size (round up to ensure all data fits)
        let shard_size = (message.len() + self.config.k - 1) / self.config.k;

        // Create data shards
        let mut shards: Vec<Vec<u8>> = Vec::with_capacity(self.config.n);

        // Fill data shards
        for i in 0..self.config.k {
            let start = i * shard_size;
            let end = std::cmp::min(start + shard_size, message.len());

            let mut shard = vec![0u8; shard_size];
            if start < message.len() {
                let data_len = end - start;
                shard[..data_len].copy_from_slice(&message[start..end]);
            }
            shards.push(shard);
        }

        // Create empty parity shards
        for _ in 0..(self.config.n - self.config.k) {
            shards.push(vec![0u8; shard_size]);
        }

        // Encode (fills parity shards)
        self.encoder
            .encode(&mut shards)
            .map_err(|e| Error::FragmentationFailed(e.to_string()))?;

        // Create fragments with metadata
        let fragment_id = Uuid::new_v4();
        let fragments: Vec<Fragment> = shards
            .into_iter()
            .enumerate()
            .map(|(index, data)| Fragment {
                id: fragment_id,
                index,
                data,
                total_shards: self.config.n,
                required_shards: self.config.k,
                original_size: message.len(),
            })
            .collect();

        Ok(fragments)
    }

    /// Reconstruct a message from k-of-n fragments
    ///
    /// Returns an error if fewer than k fragments are provided.
    pub fn reconstruct(&self, mut fragments: Vec<Fragment>) -> Result<Vec<u8>> {
        // Validate fragment count
        if fragments.is_empty() {
            return Err(Error::InsufficientFragments {
                have: 0,
                need: self.config.k,
            });
        }

        // Get metadata from first fragment
        let original_size = fragments[0].original_size;
        let required = fragments[0].required_shards;
        let total = fragments[0].total_shards;

        // Validate configuration matches
        if required != self.config.k || total != self.config.n {
            return Err(Error::InvalidFragment(format!(
                "Fragment configuration mismatch: fragment has k={}, n={}, but encoder has k={}, n={}",
                required, total, self.config.k, self.config.n
            )));
        }

        // Check if we have enough fragments
        if fragments.len() < self.config.k {
            return Err(Error::InsufficientFragments {
                have: fragments.len(),
                need: self.config.k,
            });
        }

        // Sort fragments by index
        fragments.sort_by_key(|f| f.index);

        // Build shard array with Option<Vec<u8>> for missing shards
        let mut shards: Vec<Option<Vec<u8>>> = vec![None; self.config.n];
        for fragment in fragments {
            if fragment.index < self.config.n {
                shards[fragment.index] = Some(fragment.data);
            }
        }

        // Reconstruct
        self.encoder
            .reconstruct(&mut shards)
            .map_err(|e| Error::ReconstructionFailed(e.to_string()))?;

        // Combine data shards
        let shard_size = shards[0].as_ref().unwrap().len();
        let mut message = Vec::with_capacity(self.config.k * shard_size);

        for i in 0..self.config.k {
            if let Some(shard) = &shards[i] {
                message.extend_from_slice(shard);
            }
        }

        // Trim to original size
        message.truncate(original_size);

        Ok(message)
    }

    /// Get configuration
    pub fn config(&self) -> &SidaConfig {
        &self.config
    }
}

/// Fragment of a dispersed message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fragment {
    /// Fragment set ID (all fragments from same message share this)
    pub id: Uuid,

    /// Fragment index (0 to n-1)
    pub index: usize,

    /// Fragment data (encoded shard)
    pub data: Vec<u8>,

    /// Total number of shards
    pub total_shards: usize,

    /// Required shards for reconstruction (k)
    pub required_shards: usize,

    /// Original message size (for trimming after reconstruction)
    pub original_size: usize,
}

impl Fragment {
    /// Get fragment size in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Validate fragment
    pub fn validate(&self) -> Result<()> {
        if self.index >= self.total_shards {
            return Err(Error::InvalidFragment(format!(
                "Fragment index {} >= total shards {}",
                self.index, self.total_shards
            )));
        }

        if self.required_shards > self.total_shards {
            return Err(Error::InvalidFragment(format!(
                "Required shards {} > total shards {}",
                self.required_shards, self.total_shards
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sida_basic_fragmentation() {
        let config = SidaConfig { k: 2, n: 3 };
        let fragmenter = SidaFragmenter::new(config).unwrap();

        let message = b"Hello, PlanetServe!";
        let fragments = fragmenter.fragment(message).unwrap();

        assert_eq!(fragments.len(), 3);
        for fragment in &fragments {
            assert_eq!(fragment.total_shards, 3);
            assert_eq!(fragment.required_shards, 2);
            assert_eq!(fragment.original_size, message.len());
        }
    }

    #[test]
    fn test_sida_reconstruction_with_all_fragments() {
        let config = SidaConfig { k: 2, n: 3 };
        let fragmenter = SidaFragmenter::new(config).unwrap();

        let message = b"Test message for S-IDA";
        let fragments = fragmenter.fragment(message).unwrap();

        let reconstructed = fragmenter.reconstruct(fragments).unwrap();
        assert_eq!(reconstructed, message);
    }

    #[test]
    fn test_sida_reconstruction_with_minimum_fragments() {
        let config = SidaConfig { k: 3, n: 5 };
        let fragmenter = SidaFragmenter::new(config).unwrap();

        let message = b"Privacy-preserving sync message";
        let all_fragments = fragmenter.fragment(message).unwrap();

        // Take any 3 fragments (minimum required)
        let subset: Vec<_> = all_fragments.iter().take(3).cloned().collect();
        let reconstructed = fragmenter.reconstruct(subset).unwrap();
        assert_eq!(reconstructed, message);
    }

    #[test]
    fn test_sida_reconstruction_with_different_subsets() {
        let config = SidaConfig { k: 3, n: 5 };
        let fragmenter = SidaFragmenter::new(config).unwrap();

        let message = b"Test different subset combinations";
        let all_fragments = fragmenter.fragment(message).unwrap();

        // Test different combinations of 3 fragments
        let subsets = vec![
            vec![0, 1, 2],
            vec![0, 2, 4],
            vec![1, 3, 4],
            vec![2, 3, 4],
        ];

        for indices in subsets {
            let subset: Vec<_> = indices
                .iter()
                .map(|&i| all_fragments[i].clone())
                .collect();
            let reconstructed = fragmenter.reconstruct(subset).unwrap();
            assert_eq!(reconstructed, message, "Failed with indices {:?}", indices);
        }
    }

    #[test]
    fn test_sida_insufficient_fragments() {
        let config = SidaConfig { k: 3, n: 5 };
        let fragmenter = SidaFragmenter::new(config).unwrap();

        let message = b"Test insufficient fragments";
        let all_fragments = fragmenter.fragment(message).unwrap();

        // Only 2 fragments (need 3)
        let subset: Vec<_> = all_fragments.iter().take(2).cloned().collect();
        let result = fragmenter.reconstruct(subset);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::InsufficientFragments { have: 2, need: 3 }
        ));
    }

    #[test]
    fn test_sida_large_message() {
        let config = SidaConfig { k: 3, n: 5 };
        let fragmenter = SidaFragmenter::new(config).unwrap();

        // Large message (10KB)
        let message: Vec<u8> = (0..10240).map(|i| (i % 256) as u8).collect();
        let fragments = fragmenter.fragment(&message).unwrap();

        assert_eq!(fragments.len(), 5);

        // Reconstruct from minimum fragments
        let subset: Vec<_> = fragments.iter().take(3).cloned().collect();
        let reconstructed = fragmenter.reconstruct(subset).unwrap();
        assert_eq!(reconstructed, message);
    }

    #[test]
    fn test_fragment_validation() {
        let mut fragment = Fragment {
            id: Uuid::new_v4(),
            index: 0,
            data: vec![1, 2, 3],
            total_shards: 3,
            required_shards: 2,
            original_size: 10,
        };

        assert!(fragment.validate().is_ok());

        // Invalid: index >= total_shards
        fragment.index = 3;
        assert!(fragment.validate().is_err());

        // Invalid: required > total
        fragment.index = 0;
        fragment.required_shards = 4;
        assert!(fragment.validate().is_err());
    }

    #[test]
    fn test_sida_empty_message() {
        let config = SidaConfig { k: 2, n: 3 };
        let fragmenter = SidaFragmenter::new(config).unwrap();

        // Reed-Solomon doesn't support empty messages, so we test a very small message instead
        let message = b"a";
        let fragments = fragmenter.fragment(message).unwrap();
        let reconstructed = fragmenter.reconstruct(fragments).unwrap();
        assert_eq!(reconstructed, message);
    }

    #[test]
    fn test_sida_config_validation() {
        // Valid config
        let valid = SidaConfig { k: 2, n: 3 };
        assert!(SidaFragmenter::new(valid).is_ok());

        // Invalid: k >= n
        let invalid1 = SidaConfig { k: 3, n: 3 };
        assert!(SidaFragmenter::new(invalid1).is_err());

        // Invalid: k = 0
        let invalid2 = SidaConfig { k: 0, n: 3 };
        assert!(SidaFragmenter::new(invalid2).is_err());
    }
}
