//! Device escrow allocation and management

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{CreditError, Result};

/// Per-device escrow allocation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceEscrow {
    /// Device ID
    pub device_id: String,

    /// Allocated escrow amount (in cents)
    pub allocated: i64,

    /// Spent from escrow (in cents)
    pub spent: i64,

    /// Remaining = allocated - spent (in cents)
    pub remaining: i64,

    /// Escrow granted timestamp (Unix epoch seconds)
    pub granted_at: u64,

    /// Escrow expiry timestamp (Unix epoch seconds)
    pub expires_at: u64,
}

impl DeviceEscrow {
    /// Create a new device escrow
    pub fn new(device_id: String, allocated: i64, duration_days: u64) -> Self {
        let now = Utc::now().timestamp() as u64;
        let expires_at = now + (duration_days * 24 * 60 * 60);

        Self {
            device_id,
            allocated,
            spent: 0,
            remaining: allocated,
            granted_at: now,
            expires_at,
        }
    }

    /// Check if escrow has expired
    pub fn is_expired(&self) -> bool {
        let now = Utc::now().timestamp() as u64;
        now >= self.expires_at
    }

    /// Check if escrow is low (below threshold)
    pub fn is_low(&self, threshold_percent: u8) -> bool {
        if self.allocated == 0 {
            return true;
        }
        let threshold = (self.allocated * threshold_percent as i64) / 100;
        self.remaining <= threshold
    }

    /// Spend from escrow
    pub fn spend(&mut self, amount: i64) -> Result<()> {
        if self.is_expired() {
            return Err(CreditError::EscrowExpired {
                expired_at: self.expires_at,
            });
        }

        if self.remaining < amount {
            return Err(CreditError::InsufficientEscrow {
                available: self.remaining,
                requested: amount,
            });
        }

        self.spent += amount;
        self.remaining -= amount;

        Ok(())
    }

    /// Refund to escrow (e.g., for reversed transactions)
    pub fn refund(&mut self, amount: i64) {
        self.spent = self.spent.saturating_sub(amount);
        self.remaining = self.remaining.saturating_add(amount).min(self.allocated);
    }

    /// Refresh escrow with new allocation
    pub fn refresh(&mut self, new_allocated: i64, duration_days: u64) {
        let now = Utc::now().timestamp() as u64;
        self.allocated = new_allocated;
        self.spent = 0;
        self.remaining = new_allocated;
        self.granted_at = now;
        self.expires_at = now + (duration_days * 24 * 60 * 60);
    }

    /// Get time until expiry in seconds
    pub fn time_until_expiry(&self) -> i64 {
        let now = Utc::now().timestamp() as i64;
        self.expires_at as i64 - now
    }
}

/// Escrow manager for tracking device escrows
pub struct EscrowManager {
    /// Local escrow cache
    escrows: Arc<parking_lot::RwLock<HashMap<String, DeviceEscrow>>>,
}

impl EscrowManager {
    /// Create a new escrow manager
    pub fn new() -> Self {
        Self {
            escrows: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }

    /// Get escrow for account/device
    pub fn get(&self, account_id: &str, device_id: &str) -> Result<DeviceEscrow> {
        let key = format!("{}:{}", account_id, device_id);
        self.escrows
            .read()
            .get(&key)
            .cloned()
            .ok_or_else(|| CreditError::NoEscrowAllocated {
                account_id: account_id.to_string(),
                device_id: device_id.to_string(),
            })
    }

    /// Set escrow for account/device
    pub fn set(&self, account_id: &str, device_id: &str, escrow: DeviceEscrow) {
        let key = format!("{}:{}", account_id, device_id);
        self.escrows.write().insert(key, escrow);
    }

    /// Remove escrow for account/device
    pub fn remove(&self, account_id: &str, device_id: &str) {
        let key = format!("{}:{}", account_id, device_id);
        self.escrows.write().remove(&key);
    }

    /// Spend from escrow
    pub fn spend(&self, account_id: &str, device_id: &str, amount: i64) -> Result<()> {
        let key = format!("{}:{}", account_id, device_id);
        let mut escrows = self.escrows.write();
        let escrow = escrows.get_mut(&key).ok_or_else(|| {
            CreditError::NoEscrowAllocated {
                account_id: account_id.to_string(),
                device_id: device_id.to_string(),
            }
        })?;

        escrow.spend(amount)
    }

    /// Refund to escrow
    pub fn refund(&self, account_id: &str, device_id: &str, amount: i64) -> Result<()> {
        let key = format!("{}:{}", account_id, device_id);
        let mut escrows = self.escrows.write();
        let escrow = escrows.get_mut(&key).ok_or_else(|| {
            CreditError::NoEscrowAllocated {
                account_id: account_id.to_string(),
                device_id: device_id.to_string(),
            }
        })?;

        escrow.refund(amount);
        Ok(())
    }

    /// Check if escrow is low
    pub fn is_low(&self, account_id: &str, device_id: &str, threshold_percent: u8) -> Result<bool> {
        let escrow = self.get(account_id, device_id)?;
        Ok(escrow.is_low(threshold_percent))
    }

    /// Get all escrows for an account
    pub fn get_all_for_account(&self, account_id: &str) -> Vec<DeviceEscrow> {
        let prefix = format!("{}:", account_id);
        self.escrows
            .read()
            .iter()
            .filter(|(key, _)| key.starts_with(&prefix))
            .map(|(_, escrow)| escrow.clone())
            .collect()
    }

    /// Get total allocated for an account
    pub fn total_allocated(&self, account_id: &str) -> i64 {
        self.get_all_for_account(account_id)
            .iter()
            .map(|e| e.allocated)
            .sum()
    }

    /// Get total remaining for an account
    pub fn total_remaining(&self, account_id: &str) -> i64 {
        self.get_all_for_account(account_id)
            .iter()
            .map(|e| e.remaining)
            .sum()
    }

    /// Clean up expired escrows
    pub fn cleanup_expired(&self) {
        self.escrows.write().retain(|_, escrow| !escrow.is_expired());
    }
}

impl Default for EscrowManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_escrow_new() {
        let escrow = DeviceEscrow::new("device1".to_string(), 10000, 7);
        assert_eq!(escrow.allocated, 10000);
        assert_eq!(escrow.spent, 0);
        assert_eq!(escrow.remaining, 10000);
        assert!(!escrow.is_expired());
    }

    #[test]
    fn test_device_escrow_spend() {
        let mut escrow = DeviceEscrow::new("device1".to_string(), 10000, 7);
        escrow.spend(3000).unwrap();
        assert_eq!(escrow.spent, 3000);
        assert_eq!(escrow.remaining, 7000);
    }

    #[test]
    fn test_device_escrow_insufficient() {
        let mut escrow = DeviceEscrow::new("device1".to_string(), 10000, 7);
        let result = escrow.spend(15000);
        assert!(result.is_err());
    }

    #[test]
    fn test_device_escrow_refund() {
        let mut escrow = DeviceEscrow::new("device1".to_string(), 10000, 7);
        escrow.spend(5000).unwrap();
        escrow.refund(2000);
        assert_eq!(escrow.spent, 3000);
        assert_eq!(escrow.remaining, 7000);
    }

    #[test]
    fn test_device_escrow_is_low() {
        let escrow = DeviceEscrow::new("device1".to_string(), 10000, 7);
        assert!(!escrow.is_low(20)); // 20% threshold = 2000

        let mut low_escrow = DeviceEscrow::new("device1".to_string(), 10000, 7);
        low_escrow.spend(9000).unwrap();
        assert!(low_escrow.is_low(20)); // remaining = 1000 < 2000
    }

    #[test]
    fn test_escrow_manager() {
        let manager = EscrowManager::new();
        let escrow = DeviceEscrow::new("device1".to_string(), 10000, 7);

        manager.set("alice", "device1", escrow.clone());
        let retrieved = manager.get("alice", "device1").unwrap();
        assert_eq!(retrieved, escrow);
    }

    #[test]
    fn test_escrow_manager_spend() {
        let manager = EscrowManager::new();
        let escrow = DeviceEscrow::new("device1".to_string(), 10000, 7);
        manager.set("alice", "device1", escrow);

        manager.spend("alice", "device1", 3000).unwrap();
        let updated = manager.get("alice", "device1").unwrap();
        assert_eq!(updated.remaining, 7000);
    }

    #[test]
    fn test_escrow_manager_total_allocated() {
        let manager = EscrowManager::new();
        manager.set("alice", "device1", DeviceEscrow::new("device1".to_string(), 10000, 7));
        manager.set("alice", "device2", DeviceEscrow::new("device2".to_string(), 5000, 7));
        manager.set("bob", "device3", DeviceEscrow::new("device3".to_string(), 8000, 7));

        assert_eq!(manager.total_allocated("alice"), 15000);
        assert_eq!(manager.total_allocated("bob"), 8000);
    }
}
