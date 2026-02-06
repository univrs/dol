//! Reputation tier system for credit limits

use serde::{Deserialize, Serialize};

use crate::error::{CreditError, Result};

/// Reputation tier (0-5)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReputationTier(u8);

impl ReputationTier {
    /// Minimum tier
    pub const MIN: u8 = 0;

    /// Maximum tier
    pub const MAX: u8 = 5;

    /// Create a new reputation tier
    pub fn new(tier: u8) -> Result<Self> {
        if tier > Self::MAX {
            return Err(CreditError::InvalidReputationTier(tier));
        }
        Ok(Self(tier))
    }

    /// Get tier value
    pub fn value(&self) -> u8 {
        self.0
    }

    /// Create tier 0 (new user)
    pub fn new_user() -> Self {
        Self(0)
    }

    /// Check if can upgrade
    pub fn can_upgrade(&self) -> bool {
        self.0 < Self::MAX
    }

    /// Check if can downgrade
    pub fn can_downgrade(&self) -> bool {
        self.0 > Self::MIN
    }

    /// Upgrade tier
    pub fn upgrade(&mut self) -> Result<()> {
        if !self.can_upgrade() {
            return Err(CreditError::InvalidOperation(
                "Already at maximum tier".to_string(),
            ));
        }
        self.0 += 1;
        Ok(())
    }

    /// Downgrade tier
    pub fn downgrade(&mut self) -> Result<()> {
        if !self.can_downgrade() {
            return Err(CreditError::InvalidOperation(
                "Already at minimum tier".to_string(),
            ));
        }
        self.0 -= 1;
        Ok(())
    }

    /// Get tier name
    pub fn name(&self) -> &'static str {
        match self.0 {
            0 => "New User",
            1 => "Trusted",
            2 => "Established",
            3 => "Highly Trusted",
            4 => "Community Pillar",
            5 => "Unlimited Trust",
            _ => unreachable!(),
        }
    }
}

impl Default for ReputationTier {
    fn default() -> Self {
        Self::new_user()
    }
}

impl std::fmt::Display for ReputationTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name(), self.0)
    }
}

/// Reputation manager for calculating credit limits
pub struct ReputationManager;

impl ReputationManager {
    /// Get credit limit in cents based on reputation tier
    pub fn credit_limit(tier: ReputationTier) -> i64 {
        match tier.value() {
            0 => 100,           // New user: $1.00
            1 => 1_000,         // Trusted: $10.00
            2 => 10_000,        // Established: $100.00
            3 => 100_000,       // Highly trusted: $1,000.00
            4 => 1_000_000,     // Community pillar: $10,000.00
            5 => 10_000_000,    // Unlimited trust: $100,000.00
            _ => 0,
        }
    }

    /// Get escrow limit in cents (10% of credit limit)
    pub fn escrow_limit(tier: ReputationTier) -> i64 {
        Self::credit_limit(tier) / 10
    }

    /// Get escrow low threshold percentage
    pub fn escrow_low_threshold_percent(tier: ReputationTier) -> u8 {
        match tier.value() {
            0 => 50, // New users should refresh at 50%
            1 => 40,
            2 => 30,
            3 => 25,
            4 => 20,
            5 => 10,
            _ => 50,
        }
    }

    /// Calculate recommended escrow duration in days
    pub fn escrow_duration_days(tier: ReputationTier) -> u64 {
        match tier.value() {
            0 => 1,  // New users: 1 day
            1 => 3,  // Trusted: 3 days
            2 => 7,  // Established: 1 week
            3 => 14, // Highly trusted: 2 weeks
            4 => 30, // Community pillar: 1 month
            5 => 90, // Unlimited trust: 3 months
            _ => 1,
        }
    }

    /// Format credit limit as currency string
    pub fn format_credit_limit(tier: ReputationTier) -> String {
        let cents = Self::credit_limit(tier);
        format!("${:.2}", cents as f64 / 100.0)
    }

    /// Format escrow limit as currency string
    pub fn format_escrow_limit(tier: ReputationTier) -> String {
        let cents = Self::escrow_limit(tier);
        format!("${:.2}", cents as f64 / 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reputation_tier_new() {
        assert!(ReputationTier::new(0).is_ok());
        assert!(ReputationTier::new(5).is_ok());
        assert!(ReputationTier::new(6).is_err());
    }

    #[test]
    fn test_reputation_tier_names() {
        assert_eq!(ReputationTier::new(0).unwrap().name(), "New User");
        assert_eq!(ReputationTier::new(1).unwrap().name(), "Trusted");
        assert_eq!(ReputationTier::new(2).unwrap().name(), "Established");
        assert_eq!(ReputationTier::new(3).unwrap().name(), "Highly Trusted");
        assert_eq!(ReputationTier::new(4).unwrap().name(), "Community Pillar");
        assert_eq!(ReputationTier::new(5).unwrap().name(), "Unlimited Trust");
    }

    #[test]
    fn test_reputation_tier_upgrade() {
        let mut tier = ReputationTier::new(0).unwrap();
        assert!(tier.can_upgrade());
        tier.upgrade().unwrap();
        assert_eq!(tier.value(), 1);
    }

    #[test]
    fn test_reputation_tier_downgrade() {
        let mut tier = ReputationTier::new(3).unwrap();
        assert!(tier.can_downgrade());
        tier.downgrade().unwrap();
        assert_eq!(tier.value(), 2);
    }

    #[test]
    fn test_reputation_tier_max() {
        let mut tier = ReputationTier::new(5).unwrap();
        assert!(!tier.can_upgrade());
        assert!(tier.upgrade().is_err());
    }

    #[test]
    fn test_reputation_tier_min() {
        let mut tier = ReputationTier::new(0).unwrap();
        assert!(!tier.can_downgrade());
        assert!(tier.downgrade().is_err());
    }

    #[test]
    fn test_credit_limits() {
        assert_eq!(
            ReputationManager::credit_limit(ReputationTier::new(0).unwrap()),
            100
        );
        assert_eq!(
            ReputationManager::credit_limit(ReputationTier::new(1).unwrap()),
            1_000
        );
        assert_eq!(
            ReputationManager::credit_limit(ReputationTier::new(2).unwrap()),
            10_000
        );
        assert_eq!(
            ReputationManager::credit_limit(ReputationTier::new(3).unwrap()),
            100_000
        );
        assert_eq!(
            ReputationManager::credit_limit(ReputationTier::new(4).unwrap()),
            1_000_000
        );
        assert_eq!(
            ReputationManager::credit_limit(ReputationTier::new(5).unwrap()),
            10_000_000
        );
    }

    #[test]
    fn test_escrow_limits() {
        assert_eq!(
            ReputationManager::escrow_limit(ReputationTier::new(0).unwrap()),
            10
        );
        assert_eq!(
            ReputationManager::escrow_limit(ReputationTier::new(3).unwrap()),
            10_000
        );
    }

    #[test]
    fn test_escrow_duration() {
        assert_eq!(
            ReputationManager::escrow_duration_days(ReputationTier::new(0).unwrap()),
            1
        );
        assert_eq!(
            ReputationManager::escrow_duration_days(ReputationTier::new(2).unwrap()),
            7
        );
        assert_eq!(
            ReputationManager::escrow_duration_days(ReputationTier::new(5).unwrap()),
            90
        );
    }

    #[test]
    fn test_format_credit_limit() {
        assert_eq!(
            ReputationManager::format_credit_limit(ReputationTier::new(0).unwrap()),
            "$1.00"
        );
        assert_eq!(
            ReputationManager::format_credit_limit(ReputationTier::new(3).unwrap()),
            "$1000.00"
        );
    }
}
