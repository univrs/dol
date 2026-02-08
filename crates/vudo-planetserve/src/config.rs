//! Configuration types for PlanetServe privacy levels and parameters

use serde::{Deserialize, Serialize};

/// Privacy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    /// Privacy level
    pub level: PrivacyLevel,

    /// S-IDA configuration
    pub sida: SidaConfig,

    /// Onion routing configuration
    pub onion: OnionConfig,

    /// Message padding (multiple of bytes)
    pub padding_size: usize,

    /// Timing noise (jitter in milliseconds)
    pub timing_jitter: u64,

    /// Cover traffic rate (messages per minute)
    pub cover_traffic_rate: f64,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            level: PrivacyLevel::Standard,
            sida: SidaConfig::default(),
            onion: OnionConfig::default(),
            padding_size: 1024,
            timing_jitter: 100,
            cover_traffic_rate: 0.0,
        }
    }
}

impl PrivacyConfig {
    /// Fast-open mode (no privacy, direct sync)
    pub fn fast_open() -> Self {
        Self {
            level: PrivacyLevel::None,
            ..Default::default()
        }
    }

    /// Privacy-max mode (all privacy features enabled)
    pub fn privacy_max() -> Self {
        Self {
            level: PrivacyLevel::Maximum,
            sida: SidaConfig {
                k: 3,
                n: 5,
            },
            onion: OnionConfig {
                hops: 3,
                relay_selection_strategy: RelaySelectionStrategy::LowLatency,
            },
            padding_size: 4096,
            timing_jitter: 500,
            cover_traffic_rate: 5.0,
        }
    }

    /// Basic privacy (padding only)
    pub fn basic() -> Self {
        Self {
            level: PrivacyLevel::Basic,
            ..Default::default()
        }
    }
}

/// Privacy level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrivacyLevel {
    /// No privacy (direct sync)
    None,

    /// Basic privacy (padding only)
    Basic,

    /// Standard privacy (padding + timing jitter)
    Standard,

    /// Maximum privacy (S-IDA + onion + cover traffic)
    Maximum,
}

/// S-IDA configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SidaConfig {
    /// Data shards (minimum required for reconstruction)
    pub k: usize,

    /// Total shards (k data + parity)
    pub n: usize,
}

impl Default for SidaConfig {
    fn default() -> Self {
        Self {
            k: 2,
            n: 3,
        }
    }
}

impl SidaConfig {
    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.k == 0 {
            return Err("k must be > 0".to_string());
        }
        if self.k >= self.n {
            return Err(format!("k ({}) must be < n ({})", self.k, self.n));
        }
        if self.n > 10 {
            return Err("n must be <= 10 (too many fragments)".to_string());
        }
        Ok(())
    }
}

/// Onion routing configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OnionConfig {
    /// Number of hops in circuit
    pub hops: usize,

    /// Relay selection strategy
    pub relay_selection_strategy: RelaySelectionStrategy,
}

impl Default for OnionConfig {
    fn default() -> Self {
        Self {
            hops: 2,
            relay_selection_strategy: RelaySelectionStrategy::Random,
        }
    }
}

/// Relay selection strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelaySelectionStrategy {
    /// Random selection
    Random,

    /// Prefer low-latency relays
    LowLatency,

    /// Prefer high-reliability relays
    HighReliability,

    /// Balance latency and reliability
    Balanced,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sida_config_validation() {
        let valid = SidaConfig { k: 2, n: 3 };
        assert!(valid.validate().is_ok());

        let invalid_k_gte_n = SidaConfig { k: 3, n: 3 };
        assert!(invalid_k_gte_n.validate().is_err());

        let invalid_k_zero = SidaConfig { k: 0, n: 3 };
        assert!(invalid_k_zero.validate().is_err());

        let invalid_n_too_large = SidaConfig { k: 2, n: 11 };
        assert!(invalid_n_too_large.validate().is_err());
    }

    #[test]
    fn test_privacy_config_presets() {
        let fast_open = PrivacyConfig::fast_open();
        assert_eq!(fast_open.level, PrivacyLevel::None);

        let basic = PrivacyConfig::basic();
        assert_eq!(basic.level, PrivacyLevel::Basic);

        let privacy_max = PrivacyConfig::privacy_max();
        assert_eq!(privacy_max.level, PrivacyLevel::Maximum);
        assert_eq!(privacy_max.sida.k, 3);
        assert_eq!(privacy_max.sida.n, 5);
        assert_eq!(privacy_max.onion.hops, 3);
    }
}
