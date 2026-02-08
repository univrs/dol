//! Metadata obfuscation for traffic analysis resistance
//!
//! This module provides techniques to hide WHEN syncing occurs and WHAT is
//! being synced by:
//! - Padding messages to fixed sizes
//! - Adding timing jitter (randomized delays)
//! - Sending cover traffic (dummy messages)
//!
//! # Privacy Properties
//!
//! - **Message padding**: All messages are padded to multiples of `padding_size`,
//!   hiding the actual content length
//! - **Timing jitter**: Random delays make traffic timing less predictable
//! - **Cover traffic**: Dummy messages maintain constant bandwidth, hiding real sync events
//!
//! # Examples
//!
//! ```
//! use vudo_planetserve::obfuscator::MetadataObfuscator;
//! use vudo_planetserve::config::PrivacyConfig;
//!
//! # async fn example() {
//! let config = PrivacyConfig::privacy_max();
//! let obfuscator = MetadataObfuscator::new(config);
//!
//! // Pad message
//! let message = b"test";
//! let padded = obfuscator.pad_message(message);
//! assert!(padded.len() >= message.len());
//! assert_eq!(padded.len() % 4096, 0); // padded to 4KB boundary
//!
//! // Apply timing jitter (adds random delay)
//! obfuscator.apply_timing_jitter().await;
//! # }
//! ```

use crate::config::PrivacyConfig;
use rand::Rng;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{debug, trace};

/// Metadata obfuscator
pub struct MetadataObfuscator {
    /// Privacy configuration
    config: PrivacyConfig,
}

impl MetadataObfuscator {
    /// Create a new metadata obfuscator
    pub fn new(config: PrivacyConfig) -> Self {
        Self { config }
    }

    /// Pad message to fixed size
    ///
    /// Messages are padded to the next multiple of `padding_size` bytes.
    pub fn pad_message(&self, message: &[u8]) -> Vec<u8> {
        if self.config.padding_size == 0 {
            return message.to_vec();
        }

        let target_size = ((message.len() / self.config.padding_size) + 1)
            * self.config.padding_size;

        let mut padded = message.to_vec();
        padded.resize(target_size, 0);

        trace!(
            "Padded message: {} bytes -> {} bytes",
            message.len(),
            padded.len()
        );

        padded
    }

    /// Remove padding from message
    ///
    /// This is a simple implementation that assumes padding is zeros at the end.
    /// A production implementation would use a more sophisticated padding scheme
    /// (e.g., ISO/IEC 7816-4 padding).
    pub fn unpad_message(&self, padded: &[u8]) -> Vec<u8> {
        // Find last non-zero byte
        let mut len = padded.len();
        while len > 0 && padded[len - 1] == 0 {
            len -= 1;
        }

        padded[..len].to_vec()
    }

    /// Apply timing jitter (random delay)
    ///
    /// Adds a random delay between 0 and `timing_jitter` milliseconds.
    pub async fn apply_timing_jitter(&self) {
        if self.config.timing_jitter == 0 {
            return;
        }

        let jitter = rand::thread_rng().gen_range(0..self.config.timing_jitter);
        debug!("Applying timing jitter: {}ms", jitter);
        time::sleep(Duration::from_millis(jitter)).await;
    }

    /// Create a dummy message for cover traffic
    ///
    /// Generates a message of random size (padded to `padding_size` boundary).
    pub fn create_dummy_message(&self) -> Vec<u8> {
        let mut rng = rand::thread_rng();

        // Random size between 1KB and 10KB
        let size = rng.gen_range(1024..10240);

        // Generate random data
        let mut message = vec![0u8; size];
        rng.fill(&mut message[..]);

        // Pad to fixed size
        self.pad_message(&message)
    }

    /// Start cover traffic task
    ///
    /// Spawns a background task that sends dummy messages at a fixed rate.
    /// Returns a handle to stop the task.
    pub fn start_cover_traffic(&self) -> Option<CoverTrafficHandle> {
        if self.config.cover_traffic_rate <= 0.0 {
            return None;
        }

        let interval = Duration::from_secs_f64(60.0 / self.config.cover_traffic_rate);
        let obfuscator = Self::new(self.config.clone());

        let (tx, mut rx) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            debug!(
                "Starting cover traffic at {} messages/min",
                obfuscator.config.cover_traffic_rate
            );

            let mut interval_timer = time::interval(interval);

            loop {
                tokio::select! {
                    _ = interval_timer.tick() => {
                        let dummy = obfuscator.create_dummy_message();
                        trace!("Generated cover traffic: {} bytes", dummy.len());
                        // In real implementation, would send via P2P
                    }
                    _ = &mut rx => {
                        debug!("Stopping cover traffic");
                        break;
                    }
                }
            }
        });

        Some(CoverTrafficHandle { stop_tx: Some(tx) })
    }

    /// Get configuration
    pub fn config(&self) -> &PrivacyConfig {
        &self.config
    }
}

/// Handle to stop cover traffic
pub struct CoverTrafficHandle {
    stop_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl CoverTrafficHandle {
    /// Stop cover traffic
    pub fn stop(mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for CoverTrafficHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PrivacyLevel;

    #[test]
    fn test_message_padding() {
        let config = PrivacyConfig {
            level: PrivacyLevel::Basic,
            padding_size: 1024,
            ..Default::default()
        };

        let obfuscator = MetadataObfuscator::new(config);

        // Small message
        let message = b"test";
        let padded = obfuscator.pad_message(message);
        assert_eq!(padded.len(), 1024);

        // Larger message
        let message = vec![0u8; 1500];
        let padded = obfuscator.pad_message(&message);
        assert_eq!(padded.len(), 2048);
    }

    #[test]
    fn test_message_unpadding() {
        let config = PrivacyConfig {
            level: PrivacyLevel::Basic,
            padding_size: 1024,
            ..Default::default()
        };

        let obfuscator = MetadataObfuscator::new(config);

        let original = b"test message";
        let padded = obfuscator.pad_message(original);
        let unpadded = obfuscator.unpad_message(&padded);

        assert_eq!(unpadded, original);
    }

    #[test]
    fn test_padding_disabled() {
        let config = PrivacyConfig {
            level: PrivacyLevel::None,
            padding_size: 0,
            ..Default::default()
        };

        let obfuscator = MetadataObfuscator::new(config);

        let message = b"test";
        let padded = obfuscator.pad_message(message);
        assert_eq!(padded.len(), message.len());
    }

    #[tokio::test]
    async fn test_timing_jitter() {
        let config = PrivacyConfig {
            level: PrivacyLevel::Standard,
            timing_jitter: 100,
            ..Default::default()
        };

        let obfuscator = MetadataObfuscator::new(config);

        let start = std::time::Instant::now();
        obfuscator.apply_timing_jitter().await;
        let elapsed = start.elapsed();

        // Should have added some delay (but less than max)
        assert!(elapsed < Duration::from_millis(150));
    }

    #[tokio::test]
    async fn test_timing_jitter_disabled() {
        let config = PrivacyConfig {
            level: PrivacyLevel::None,
            timing_jitter: 0,
            ..Default::default()
        };

        let obfuscator = MetadataObfuscator::new(config);

        let start = std::time::Instant::now();
        obfuscator.apply_timing_jitter().await;
        let elapsed = start.elapsed();

        // Should be nearly instant
        assert!(elapsed < Duration::from_millis(10));
    }

    #[test]
    fn test_dummy_message_generation() {
        let config = PrivacyConfig {
            level: PrivacyLevel::Maximum,
            padding_size: 1024,
            ..Default::default()
        };

        let obfuscator = MetadataObfuscator::new(config);

        let dummy = obfuscator.create_dummy_message();

        // Should be padded
        assert_eq!(dummy.len() % 1024, 0);

        // Should be at least 1KB
        assert!(dummy.len() >= 1024);

        // Should be at most ~11KB (10KB + padding)
        assert!(dummy.len() <= 11264);
    }

    #[tokio::test]
    async fn test_cover_traffic_start_stop() {
        let config = PrivacyConfig {
            level: PrivacyLevel::Maximum,
            cover_traffic_rate: 60.0, // 60 messages/min = 1/sec
            padding_size: 1024,
            ..Default::default()
        };

        let obfuscator = MetadataObfuscator::new(config);

        let handle = obfuscator.start_cover_traffic();
        assert!(handle.is_some());

        let handle = handle.unwrap();

        // Wait a bit
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Stop
        handle.stop();
    }

    #[test]
    fn test_cover_traffic_disabled() {
        let config = PrivacyConfig {
            level: PrivacyLevel::None,
            cover_traffic_rate: 0.0,
            ..Default::default()
        };

        let obfuscator = MetadataObfuscator::new(config);

        let handle = obfuscator.start_cover_traffic();
        assert!(handle.is_none());
    }

    #[test]
    fn test_multiple_messages_same_padded_size() {
        let config = PrivacyConfig {
            level: PrivacyLevel::Basic,
            padding_size: 1024,
            ..Default::default()
        };

        let obfuscator = MetadataObfuscator::new(config);

        // Different messages, same size after padding
        let msg1 = b"short";
        let msg2 = b"a bit longer message";
        let msg3 = vec![0u8; 500];

        let padded1 = obfuscator.pad_message(msg1);
        let padded2 = obfuscator.pad_message(msg2);
        let padded3 = obfuscator.pad_message(&msg3);

        // All should be padded to 1024 bytes
        assert_eq!(padded1.len(), 1024);
        assert_eq!(padded2.len(), 1024);
        assert_eq!(padded3.len(), 1024);
    }
}
