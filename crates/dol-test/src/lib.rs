//! Property-based testing framework for DOL CRDT convergence guarantees
//!
//! This crate provides a comprehensive framework for testing CRDT implementations
//! against the formal properties defined in RFC-001. It includes:
//!
//! - Property test generators for CRDT operations
//! - Verification of the 13 formal theorems from RFC-001
//! - Network topology simulation for partition testing
//! - Test case generation from DOL files
//!
//! # Example
//!
//! ```rust
//! use dol_test::properties::*;
//! use proptest::prelude::*;
//!
//! proptest! {
//!     #[test]
//!     fn test_merge_commutativity(
//!         ops_a in vec(any::<Operation>(), 1..100),
//!         ops_b in vec(any::<Operation>(), 1..100)
//!     ) {
//!         let mut replica_a = CrdtState::default();
//!         let mut replica_b = CrdtState::default();
//!
//!         for op in ops_a { replica_a.apply(op); }
//!         for op in ops_b { replica_b.apply(op); }
//!
//!         // Merge in both directions
//!         let mut ab = replica_a.clone();
//!         ab.merge(&replica_b);
//!
//!         let mut ba = replica_b.clone();
//!         ba.merge(&replica_a);
//!
//!         // Theorem 1: Commutativity
//!         assert_eq!(ab, ba);
//!     }
//! }
//! ```
//!
//! # Architecture
//!
//! The framework is organized into three main modules:
//!
//! - [`properties`]: CRDT property definitions and verification functions
//! - [`generators`]: Arbitrary generators for CRDT operations and network topologies
//! - [`harness`]: Test harness utilities for running property tests
//!
//! # Testing Strategy
//!
//! The framework tests all 7 CRDT strategies defined in RFC-001:
//!
//! 1. **Immutable**: Write-once semantics
//! 2. **LWW** (Last-Write-Wins): Timestamp-based conflict resolution
//! 3. **OR-Set** (Observed-Remove Set): Add-wins set semantics
//! 4. **PN-Counter** (Positive-Negative Counter): Distributed counting
//! 5. **RGA** (Replicated Growable Array): Ordered sequence with causality
//! 6. **MV-Register** (Multi-Value Register): Preserves concurrent values
//! 7. **Peritext**: Collaborative rich text editing
//!
//! Each strategy is tested against the fundamental CRDT properties:
//!
//! - Commutativity: `merge(a, b) = merge(b, a)`
//! - Associativity: `merge(merge(a, b), c) = merge(a, merge(b, c))`
//! - Idempotency: `merge(a, a) = a`

pub mod properties;
pub mod generators;
pub mod harness;

use thiserror::Error;

/// Errors that can occur during property-based testing
#[derive(Debug, Error)]
pub enum TestError {
    /// CRDT property violation detected
    #[error("Property violation: {property} - {details}")]
    PropertyViolation {
        property: String,
        details: String,
    },

    /// Convergence failure after merge
    #[error("Convergence failure: replicas did not reach same state")]
    ConvergenceFailure,

    /// Invalid operation sequence
    #[error("Invalid operation sequence: {reason}")]
    InvalidOperationSequence {
        reason: String,
    },

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Code generation error
    #[error("Code generation error: {0}")]
    CodegenError(String),
}

/// Result type for property tests
pub type TestResult<T> = Result<T, TestError>;

/// Test configuration for property-based tests
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Number of test cases to generate
    pub num_cases: u32,

    /// Maximum number of operations per test case
    pub max_operations: usize,

    /// Number of replicas to simulate
    pub num_replicas: usize,

    /// Enable network partition simulation
    pub simulate_partitions: bool,

    /// Enable Byzantine fault injection
    pub simulate_byzantine: bool,

    /// Random seed for reproducibility
    pub seed: Option<u64>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            num_cases: 1000,
            max_operations: 100,
            num_replicas: 3,
            simulate_partitions: true,
            simulate_byzantine: false,
            seed: None,
        }
    }
}

/// Test report containing results of property-based testing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestReport {
    /// Total number of test cases executed
    pub total_cases: u32,

    /// Number of passed test cases
    pub passed: u32,

    /// Number of failed test cases
    pub failed: u32,

    /// List of property violations
    pub violations: Vec<PropertyViolation>,

    /// Test duration in milliseconds
    pub duration_ms: u64,

    /// Configuration used for tests
    pub config_summary: String,
}

/// A property violation detected during testing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PropertyViolation {
    /// Name of the violated property
    pub property: String,

    /// CRDT strategy being tested
    pub strategy: String,

    /// Description of the violation
    pub details: String,

    /// Minimal reproducing operation sequence
    pub reproducer: Option<String>,
}

impl TestReport {
    /// Creates a new test report
    pub fn new(config: &TestConfig) -> Self {
        Self {
            total_cases: 0,
            passed: 0,
            failed: 0,
            violations: Vec::new(),
            duration_ms: 0,
            config_summary: format!(
                "cases={}, max_ops={}, replicas={}",
                config.num_cases, config.max_operations, config.num_replicas
            ),
        }
    }

    /// Records a passed test case
    pub fn record_pass(&mut self) {
        self.total_cases += 1;
        self.passed += 1;
    }

    /// Records a failed test case
    pub fn record_failure(&mut self, violation: PropertyViolation) {
        self.total_cases += 1;
        self.failed += 1;
        self.violations.push(violation);
    }

    /// Returns true if all tests passed
    pub fn is_success(&self) -> bool {
        self.failed == 0
    }

    /// Returns the pass rate as a percentage
    pub fn pass_rate(&self) -> f64 {
        if self.total_cases == 0 {
            0.0
        } else {
            (self.passed as f64 / self.total_cases as f64) * 100.0
        }
    }

    /// Generates a summary report
    pub fn summary(&self) -> String {
        format!(
            "Test Report: {} total, {} passed, {} failed ({:.2}% pass rate)\nConfig: {}\nDuration: {}ms",
            self.total_cases,
            self.passed,
            self.failed,
            self.pass_rate(),
            self.config_summary,
            self.duration_ms
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = TestConfig::default();
        assert_eq!(config.num_cases, 1000);
        assert_eq!(config.max_operations, 100);
        assert_eq!(config.num_replicas, 3);
        assert!(config.simulate_partitions);
        assert!(!config.simulate_byzantine);
    }

    #[test]
    fn test_report_creation() {
        let config = TestConfig::default();
        let report = TestReport::new(&config);
        assert_eq!(report.total_cases, 0);
        assert_eq!(report.passed, 0);
        assert_eq!(report.failed, 0);
        assert!(report.is_success());
    }

    #[test]
    fn test_report_pass_recording() {
        let config = TestConfig::default();
        let mut report = TestReport::new(&config);

        report.record_pass();
        assert_eq!(report.total_cases, 1);
        assert_eq!(report.passed, 1);
        assert_eq!(report.failed, 0);
        assert!(report.is_success());
    }

    #[test]
    fn test_report_failure_recording() {
        let config = TestConfig::default();
        let mut report = TestReport::new(&config);

        let violation = PropertyViolation {
            property: "commutativity".to_string(),
            strategy: "lww".to_string(),
            details: "merge(a,b) != merge(b,a)".to_string(),
            reproducer: None,
        };

        report.record_failure(violation);
        assert_eq!(report.total_cases, 1);
        assert_eq!(report.passed, 0);
        assert_eq!(report.failed, 1);
        assert!(!report.is_success());
    }

    #[test]
    fn test_pass_rate_calculation() {
        let config = TestConfig::default();
        let mut report = TestReport::new(&config);

        report.record_pass();
        report.record_pass();
        report.record_pass();
        report.record_failure(PropertyViolation {
            property: "test".to_string(),
            strategy: "test".to_string(),
            details: "test".to_string(),
            reproducer: None,
        });

        assert_eq!(report.total_cases, 4);
        assert_eq!(report.pass_rate(), 75.0);
    }
}
