//! Test harness utilities for running property tests
//!
//! This module provides high-level utilities for orchestrating property-based tests,
//! including test execution, reporting, and integration with DOL code generation.

use crate::properties::{Mergeable, Operable, PropertyResult, PropertyTestSuite};
use crate::{PropertyViolation, TestConfig, TestReport, TestResult};
use std::time::Instant;

/// Test harness for running property-based tests on CRDT implementations
pub struct TestHarness {
    config: TestConfig,
}

impl TestHarness {
    /// Creates a new test harness with the given configuration
    pub fn new(config: TestConfig) -> Self {
        Self { config }
    }

    /// Creates a test harness with default configuration
    pub fn default_config() -> Self {
        Self {
            config: TestConfig::default(),
        }
    }

    /// Runs fundamental property tests (commutativity, associativity, idempotency)
    pub fn test_fundamental_properties<T>(
        &self,
        states: Vec<T>,
    ) -> TestResult<TestReport>
    where
        T: Mergeable,
    {
        let start = Instant::now();
        let mut report = TestReport::new(&self.config);

        if states.len() < 3 {
            return Err(crate::TestError::InvalidOperationSequence {
                reason: "Need at least 3 states to test all fundamental properties".to_string(),
            });
        }

        // Test all combinations of states
        for i in 0..states.len() {
            for j in 0..states.len() {
                for k in 0..states.len() {
                    let results = PropertyTestSuite::<T>::test_fundamental_properties(
                        &states[i],
                        &states[j],
                        &states[k],
                    );

                    for (property, result) in results {
                        match result {
                            PropertyResult::Pass => {
                                report.record_pass();
                            }
                            PropertyResult::Fail { reason } => {
                                report.record_failure(PropertyViolation {
                                    property: property.clone(),
                                    strategy: "fundamental".to_string(),
                                    details: reason,
                                    reproducer: None,
                                });
                            }
                            PropertyResult::Inconclusive { .. } => {
                                // Don't count as pass or fail
                            }
                        }
                    }
                }
            }
        }

        report.duration_ms = start.elapsed().as_millis() as u64;
        Ok(report)
    }

    /// Runs convergence tests with random operations
    pub fn test_convergence<T, Op>(
        &self,
        operations: Vec<Op>,
    ) -> TestResult<TestReport>
    where
        T: Mergeable + Operable<Op> + Default,
        Op: Clone,
    {
        let start = Instant::now();
        let mut report = TestReport::new(&self.config);

        // Create replicas
        let mut replicas: Vec<T> = (0..self.config.num_replicas)
            .map(|_| T::default())
            .collect();

        // Apply operations to each replica
        for (idx, replica) in replicas.iter_mut().enumerate() {
            let mut ops = operations.clone();

            // Apply operations in different orders to simulate network conditions
            if idx % 2 == 1 {
                ops.reverse();
            }

            for op in ops {
                replica.apply(op)?;
            }
        }

        // Merge all replicas pairwise and verify convergence
        for i in 0..replicas.len() {
            for j in i + 1..replicas.len() {
                let mut replica_i = replicas[i].clone();
                let mut replica_j = replicas[j].clone();

                replica_i.merge(&replicas[j])?;
                replica_j.merge(&replicas[i])?;

                if replica_i == replica_j {
                    report.record_pass();
                } else {
                    report.record_failure(PropertyViolation {
                        property: "convergence".to_string(),
                        strategy: "general".to_string(),
                        details: format!(
                            "Replicas {} and {} did not converge after merge",
                            i, j
                        ),
                        reproducer: None,
                    });
                }
            }
        }

        report.duration_ms = start.elapsed().as_millis() as u64;
        Ok(report)
    }

    /// Returns the test configuration
    pub fn config(&self) -> &TestConfig {
        &self.config
    }
}

/// Builder for constructing test scenarios
pub struct TestScenarioBuilder<T, Op> {
    initial_state: Option<T>,
    operations: Vec<Op>,
    num_replicas: usize,
    enable_partition: bool,
}

impl<T, Op> Default for TestScenarioBuilder<T, Op> {
    fn default() -> Self {
        Self {
            initial_state: None,
            operations: Vec::new(),
            num_replicas: 3,
            enable_partition: false,
        }
    }
}

impl<T, Op> TestScenarioBuilder<T, Op> {
    /// Creates a new scenario builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the initial state
    pub fn with_initial_state(mut self, state: T) -> Self {
        self.initial_state = Some(state);
        self
    }

    /// Adds operations to the scenario
    pub fn with_operations(mut self, ops: Vec<Op>) -> Self {
        self.operations = ops;
        self
    }

    /// Sets the number of replicas
    pub fn with_replicas(mut self, num: usize) -> Self {
        self.num_replicas = num;
        self
    }

    /// Enables network partition simulation
    pub fn with_partition(mut self, enable: bool) -> Self {
        self.enable_partition = enable;
        self
    }

    /// Builds the scenario and runs the test
    pub fn build_and_test(self) -> TestResult<TestReport>
    where
        T: Mergeable + Operable<Op> + Default,
        Op: Clone,
    {
        let config = TestConfig {
            num_replicas: self.num_replicas,
            simulate_partitions: self.enable_partition,
            ..Default::default()
        };

        let harness = TestHarness::new(config);
        harness.test_convergence::<T, Op>(self.operations)
    }
}

/// Helper function to create a test report from property results
pub fn results_to_report(
    results: Vec<(String, PropertyResult)>,
    config: &TestConfig,
) -> TestReport {
    let mut report = TestReport::new(config);

    for (property, result) in results {
        match result {
            PropertyResult::Pass => {
                report.record_pass();
            }
            PropertyResult::Fail { reason } => {
                report.record_failure(PropertyViolation {
                    property,
                    strategy: "unknown".to_string(),
                    details: reason,
                    reproducer: None,
                });
            }
            PropertyResult::Inconclusive { .. } => {
                // Inconclusive results don't count
            }
        }
    }

    report
}

/// Runs a single property test and returns the result
pub fn run_property_test<F>(test_fn: F, property_name: &str) -> PropertyResult
where
    F: FnOnce() -> bool,
{
    if test_fn() {
        PropertyResult::Pass
    } else {
        PropertyResult::Fail {
            reason: format!("{} property violated", property_name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::properties::{Mergeable, MonotonicState, Operable};

    // Test CRDT implementation
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct SimpleCounter {
        value: i64,
    }

    impl Default for SimpleCounter {
        fn default() -> Self {
            Self { value: 0 }
        }
    }

    impl Mergeable for SimpleCounter {
        fn merge(&mut self, other: &Self) -> TestResult<()> {
            self.value = self.value.max(other.value);
            Ok(())
        }

        fn replica_id(&self) -> String {
            "test".to_string()
        }
    }

    #[derive(Debug, Clone)]
    struct Inc;

    impl Operable<Inc> for SimpleCounter {
        fn apply(&mut self, _op: Inc) -> TestResult<()> {
            self.value += 1;
            Ok(())
        }
    }

    impl MonotonicState for SimpleCounter {
        fn measure(&self) -> usize {
            self.value.max(0) as usize
        }
    }

    #[test]
    fn test_harness_creation() {
        let harness = TestHarness::default_config();
        assert_eq!(harness.config().num_replicas, 3);
    }

    #[test]
    fn test_fundamental_properties() {
        let harness = TestHarness::default_config();
        let states = vec![
            SimpleCounter { value: 10 },
            SimpleCounter { value: 20 },
            SimpleCounter { value: 15 },
        ];

        let report = harness.test_fundamental_properties(states).unwrap();
        assert!(report.is_success());
    }

    #[test]
    fn test_convergence() {
        let harness = TestHarness::default_config();
        let operations = vec![Inc, Inc, Inc];

        let report = harness.test_convergence::<SimpleCounter, Inc>(operations).unwrap();
        assert!(report.passed > 0);
    }

    #[test]
    fn test_scenario_builder() {
        let scenario = TestScenarioBuilder::<SimpleCounter, Inc>::new()
            .with_operations(vec![Inc, Inc])
            .with_replicas(2)
            .build_and_test();

        assert!(scenario.is_ok());
        let report = scenario.unwrap();
        assert!(report.passed > 0);
    }

    #[test]
    fn test_run_property_test_pass() {
        let result = run_property_test(|| true, "test_property");
        assert!(result.is_pass());
    }

    #[test]
    fn test_run_property_test_fail() {
        let result = run_property_test(|| false, "test_property");
        assert!(result.is_fail());
    }
}
