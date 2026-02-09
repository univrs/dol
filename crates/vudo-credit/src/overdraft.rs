//! Overdraft detection and resolution

use serde::{Deserialize, Serialize};

use crate::transaction::TransactionId;

/// Overdraft information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Overdraft {
    /// Transaction ID that contributed to overdraft
    pub transaction_id: TransactionId,

    /// Transaction amount
    pub amount: i64,

    /// Deficit amount (how much over the limit)
    pub deficit: i64,

    /// Transaction timestamp
    pub timestamp: u64,
}

impl Overdraft {
    /// Create a new overdraft
    pub fn new(transaction_id: TransactionId, amount: i64, deficit: i64, timestamp: u64) -> Self {
        Self {
            transaction_id,
            amount,
            deficit,
            timestamp,
        }
    }
}

/// Overdraft resolution strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OverdraftResolution {
    /// Reverse the transaction
    Reverse,

    /// Approve and extend credit (requires BFT vote)
    Approve,

    /// Split resolution between parties
    Split { sender_pays: i64, receiver_pays: i64 },

    /// Defer resolution (mark as disputed)
    Defer,
}

/// Overdraft resolver
pub struct OverdraftResolver;

impl OverdraftResolver {
    /// Detect overdrafts in a list of transactions
    ///
    /// Given a confirmed balance and a list of pending transactions,
    /// identify which transactions cause the balance to go negative.
    pub fn detect_overdrafts(
        confirmed_balance: i64,
        transactions: &[(TransactionId, i64, u64)], // (id, amount, timestamp)
    ) -> Vec<Overdraft> {
        let mut running_balance = confirmed_balance;
        let mut overdrafts = Vec::new();

        for (tx_id, amount, timestamp) in transactions {
            running_balance -= amount;
            if running_balance < 0 {
                overdrafts.push(Overdraft {
                    transaction_id: tx_id.clone(),
                    amount: *amount,
                    deficit: running_balance.abs(),
                    timestamp: *timestamp,
                });
            }
        }

        overdrafts
    }

    /// Calculate total deficit from overdrafts
    pub fn total_deficit(overdrafts: &[Overdraft]) -> i64 {
        overdrafts.iter().map(|o| o.deficit).sum()
    }

    /// Get most severe overdraft (highest deficit)
    pub fn most_severe(overdrafts: &[Overdraft]) -> Option<&Overdraft> {
        overdrafts.iter().max_by_key(|o| o.deficit)
    }

    /// Suggest resolution strategy based on overdraft severity
    pub fn suggest_resolution(overdraft: &Overdraft, confirmed_balance: i64) -> OverdraftResolution {
        let overdraft_ratio = overdraft.deficit as f64 / confirmed_balance as f64;

        if overdraft_ratio < 0.1 {
            // Less than 10% overdraft: suggest approval with extended credit
            OverdraftResolution::Approve
        } else if overdraft_ratio < 0.5 {
            // 10-50% overdraft: suggest split resolution
            let half_deficit = overdraft.deficit / 2;
            OverdraftResolution::Split {
                sender_pays: half_deficit,
                receiver_pays: overdraft.deficit - half_deficit,
            }
        } else {
            // More than 50% overdraft: suggest reversal
            OverdraftResolution::Reverse
        }
    }

    /// Validate resolution
    pub fn validate_resolution(
        overdraft: &Overdraft,
        resolution: &OverdraftResolution,
    ) -> Result<(), String> {
        match resolution {
            OverdraftResolution::Reverse => Ok(()),
            OverdraftResolution::Approve => Ok(()),
            OverdraftResolution::Split {
                sender_pays,
                receiver_pays,
            } => {
                if sender_pays + receiver_pays != overdraft.deficit {
                    return Err(format!(
                        "Split resolution doesn't sum to deficit: {} + {} != {}",
                        sender_pays, receiver_pays, overdraft.deficit
                    ));
                }
                if *sender_pays < 0 || *receiver_pays < 0 {
                    return Err("Split amounts cannot be negative".to_string());
                }
                Ok(())
            }
            OverdraftResolution::Defer => Ok(()),
        }
    }

    /// Calculate recovery amount for resolution
    pub fn recovery_amount(resolution: &OverdraftResolution) -> i64 {
        match resolution {
            OverdraftResolution::Reverse => 0, // Transaction reversed, no recovery
            OverdraftResolution::Approve => 0, // Credit extended, no recovery
            OverdraftResolution::Split {
                sender_pays,
                receiver_pays,
            } => sender_pays + receiver_pays,
            OverdraftResolution::Defer => 0, // Deferred, no immediate recovery
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_overdrafts_none() {
        let transactions = vec![
            ("tx1".to_string(), 100, 1000),
            ("tx2".to_string(), 200, 1001),
        ];
        let overdrafts = OverdraftResolver::detect_overdrafts(1000, &transactions);
        assert!(overdrafts.is_empty());
    }

    #[test]
    fn test_detect_overdrafts_single() {
        let transactions = vec![
            ("tx1".to_string(), 500, 1000),
            ("tx2".to_string(), 600, 1001), // This causes overdraft
        ];
        let overdrafts = OverdraftResolver::detect_overdrafts(1000, &transactions);
        assert_eq!(overdrafts.len(), 1);
        assert_eq!(overdrafts[0].transaction_id, "tx2");
        assert_eq!(overdrafts[0].deficit, 100);
    }

    #[test]
    fn test_detect_overdrafts_multiple() {
        let transactions = vec![
            ("tx1".to_string(), 700, 1000),
            ("tx2".to_string(), 400, 1001), // Overdraft by 100
            ("tx3".to_string(), 200, 1002), // Overdraft by 300
        ];
        let overdrafts = OverdraftResolver::detect_overdrafts(1000, &transactions);
        assert_eq!(overdrafts.len(), 2);
        assert_eq!(overdrafts[0].transaction_id, "tx2");
        assert_eq!(overdrafts[0].deficit, 100);
        assert_eq!(overdrafts[1].transaction_id, "tx3");
        assert_eq!(overdrafts[1].deficit, 300);
    }

    #[test]
    fn test_total_deficit() {
        let overdrafts = vec![
            Overdraft::new("tx1".to_string(), 100, 50, 1000),
            Overdraft::new("tx2".to_string(), 200, 150, 1001),
        ];
        assert_eq!(OverdraftResolver::total_deficit(&overdrafts), 200);
    }

    #[test]
    fn test_most_severe() {
        let overdrafts = vec![
            Overdraft::new("tx1".to_string(), 100, 50, 1000),
            Overdraft::new("tx2".to_string(), 200, 150, 1001),
            Overdraft::new("tx3".to_string(), 300, 100, 1002),
        ];
        let most_severe = OverdraftResolver::most_severe(&overdrafts).unwrap();
        assert_eq!(most_severe.transaction_id, "tx2");
    }

    #[test]
    fn test_suggest_resolution_small_overdraft() {
        let overdraft = Overdraft::new("tx1".to_string(), 1000, 50, 1000);
        let resolution = OverdraftResolver::suggest_resolution(&overdraft, 1000);
        assert_eq!(resolution, OverdraftResolution::Approve);
    }

    #[test]
    fn test_suggest_resolution_medium_overdraft() {
        let overdraft = Overdraft::new("tx1".to_string(), 1000, 300, 1000);
        let resolution = OverdraftResolver::suggest_resolution(&overdraft, 1000);
        match resolution {
            OverdraftResolution::Split {
                sender_pays,
                receiver_pays,
            } => {
                assert_eq!(sender_pays + receiver_pays, 300);
            }
            _ => panic!("Expected Split resolution"),
        }
    }

    #[test]
    fn test_suggest_resolution_large_overdraft() {
        let overdraft = Overdraft::new("tx1".to_string(), 1000, 600, 1000);
        let resolution = OverdraftResolver::suggest_resolution(&overdraft, 1000);
        assert_eq!(resolution, OverdraftResolution::Reverse);
    }

    #[test]
    fn test_validate_resolution_split_valid() {
        let overdraft = Overdraft::new("tx1".to_string(), 1000, 100, 1000);
        let resolution = OverdraftResolution::Split {
            sender_pays: 60,
            receiver_pays: 40,
        };
        assert!(OverdraftResolver::validate_resolution(&overdraft, &resolution).is_ok());
    }

    #[test]
    fn test_validate_resolution_split_invalid() {
        let overdraft = Overdraft::new("tx1".to_string(), 1000, 100, 1000);
        let resolution = OverdraftResolution::Split {
            sender_pays: 60,
            receiver_pays: 50, // Doesn't sum to 100
        };
        assert!(OverdraftResolver::validate_resolution(&overdraft, &resolution).is_err());
    }
}
