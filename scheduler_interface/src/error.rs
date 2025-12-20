//! Error types for container scheduling operations.
//!
//! This module defines the error types that can occur during scheduling,
//! resource allocation, and node binding operations.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during scheduling operations.
#[derive(Error, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SchedulerError {
    /// Insufficient resources available to fulfill request.
    ///
    /// This error occurs when a container's resource requests exceed
    /// what is available on any feasible node.
    #[error("Insufficient {resource}: requested {requested}, available {available}")]
    InsufficientResources {
        /// The resource type (e.g., "CPU", "memory", "disk", "GPU")
        resource: String,
        /// Amount of resource requested
        requested: u64,
        /// Amount of resource available
        available: u64,
    },

    /// No nodes meet the scheduling constraints.
    ///
    /// This error occurs when no nodes in the cluster satisfy the
    /// pod's affinity, anti-affinity, or other placement constraints.
    #[error("No feasible nodes found for scheduling")]
    NoFeasibleNodes,

    /// Conflict when attempting to bind a pod to a node.
    ///
    /// This error occurs when the binding operation fails due to
    /// the node being in an invalid state or the pod already being bound.
    #[error("Binding conflict for node {node_id}")]
    BindingConflict {
        /// ID of the node that could not be bound
        node_id: String,
    },

    /// Timeout while waiting for a resource reservation.
    ///
    /// This error occurs when a reserved allocation is not confirmed
    /// within the expected timeout period.
    #[error("Reservation timeout for reservation {reservation_id}")]
    ReservationTimeout {
        /// ID of the reservation that timed out
        reservation_id: String,
    },

    /// Validation error for scheduling request.
    ///
    /// This error occurs when the scheduling request itself is invalid,
    /// such as negative resource values or malformed constraints.
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Resource allocation already exists for the given entity.
    ///
    /// This error occurs when attempting to allocate resources for
    /// a pod that already has an allocation.
    #[error("Resource allocation already exists for {entity_id}")]
    AllocationExists {
        /// ID of the entity (e.g., pod ID)
        entity_id: String,
    },

    /// Resource allocation not found.
    ///
    /// This error occurs when attempting to release or modify an
    /// allocation that doesn't exist.
    #[error("Resource allocation not found for {entity_id}")]
    AllocationNotFound {
        /// ID of the entity (e.g., pod ID)
        entity_id: String,
    },

    /// Node not found in the cluster.
    ///
    /// This error occurs when referencing a node that doesn't exist
    /// or has been removed from the cluster.
    #[error("Node not found: {node_id}")]
    NodeNotFound {
        /// ID of the missing node
        node_id: String,
    },

    /// Node is in an unhealthy state and cannot accept pods.
    ///
    /// This error occurs when attempting to schedule to a node that
    /// is marked as NotReady, Cordoned, or otherwise unhealthy.
    #[error("Node {node_id} is unhealthy: {reason}")]
    NodeUnhealthy {
        /// ID of the unhealthy node
        node_id: String,
        /// Reason for the unhealthy state
        reason: String,
    },

    /// Scheduling constraint violation.
    ///
    /// This error occurs when a scheduling decision would violate
    /// a hard constraint (e.g., required anti-affinity).
    #[error("Constraint violation: {constraint}")]
    ConstraintViolation {
        /// Description of the violated constraint
        constraint: String,
    },

    /// Priority preemption failed.
    ///
    /// This error occurs when the scheduler cannot make room for
    /// a high-priority pod by preempting lower-priority pods.
    #[error("Preemption failed: {reason}")]
    PreemptionFailed {
        /// Reason preemption failed
        reason: String,
    },
}

impl SchedulerError {
    /// Creates a new InsufficientResources error.
    ///
    /// # Examples
    ///
    /// ```
    /// use scheduler_interface::error::SchedulerError;
    ///
    /// let error = SchedulerError::insufficient_resources("CPU", 8, 4);
    /// assert_eq!(error.to_string(), "Insufficient CPU: requested 8, available 4");
    /// ```
    pub fn insufficient_resources(
        resource: impl Into<String>,
        requested: u64,
        available: u64,
    ) -> Self {
        Self::InsufficientResources {
            resource: resource.into(),
            requested,
            available,
        }
    }

    /// Creates a new BindingConflict error.
    pub fn binding_conflict(node_id: impl Into<String>) -> Self {
        Self::BindingConflict {
            node_id: node_id.into(),
        }
    }

    /// Creates a new ReservationTimeout error.
    pub fn reservation_timeout(reservation_id: impl Into<String>) -> Self {
        Self::ReservationTimeout {
            reservation_id: reservation_id.into(),
        }
    }

    /// Creates a new ValidationError.
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::ValidationError(message.into())
    }

    /// Creates a new AllocationExists error.
    pub fn allocation_exists(entity_id: impl Into<String>) -> Self {
        Self::AllocationExists {
            entity_id: entity_id.into(),
        }
    }

    /// Creates a new AllocationNotFound error.
    pub fn allocation_not_found(entity_id: impl Into<String>) -> Self {
        Self::AllocationNotFound {
            entity_id: entity_id.into(),
        }
    }

    /// Creates a new NodeNotFound error.
    pub fn node_not_found(node_id: impl Into<String>) -> Self {
        Self::NodeNotFound {
            node_id: node_id.into(),
        }
    }

    /// Creates a new NodeUnhealthy error.
    pub fn node_unhealthy(node_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::NodeUnhealthy {
            node_id: node_id.into(),
            reason: reason.into(),
        }
    }

    /// Creates a new ConstraintViolation error.
    pub fn constraint_violation(constraint: impl Into<String>) -> Self {
        Self::ConstraintViolation {
            constraint: constraint.into(),
        }
    }

    /// Creates a new PreemptionFailed error.
    pub fn preemption_failed(reason: impl Into<String>) -> Self {
        Self::PreemptionFailed {
            reason: reason.into(),
        }
    }

    /// Returns true if this error is retryable.
    ///
    /// Some errors, like temporary resource shortages, may succeed on retry.
    /// Others, like validation errors, will not.
    ///
    /// # Examples
    ///
    /// ```
    /// use scheduler_interface::error::SchedulerError;
    ///
    /// let retryable = SchedulerError::insufficient_resources("CPU", 8, 4);
    /// assert!(retryable.is_retryable());
    ///
    /// let not_retryable = SchedulerError::validation_error("Invalid pod spec");
    /// assert!(!not_retryable.is_retryable());
    /// ```
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            SchedulerError::InsufficientResources { .. }
                | SchedulerError::NoFeasibleNodes
                | SchedulerError::BindingConflict { .. }
                | SchedulerError::ReservationTimeout { .. }
                | SchedulerError::NodeUnhealthy { .. }
        )
    }
}

/// Result type alias for scheduler operations.
pub type SchedulerResult<T> = Result<T, SchedulerError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insufficient_resources_error() {
        let error = SchedulerError::insufficient_resources("CPU", 8, 4);
        assert_eq!(
            error.to_string(),
            "Insufficient CPU: requested 8, available 4"
        );
        assert!(error.is_retryable());
    }

    #[test]
    fn test_validation_error() {
        let error = SchedulerError::validation_error("Invalid resource spec");
        assert_eq!(error.to_string(), "Validation error: Invalid resource spec");
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_no_feasible_nodes_error() {
        let error = SchedulerError::NoFeasibleNodes;
        assert_eq!(error.to_string(), "No feasible nodes found for scheduling");
        assert!(error.is_retryable());
    }

    #[test]
    fn test_binding_conflict_error() {
        let error = SchedulerError::binding_conflict("node-123");
        assert_eq!(error.to_string(), "Binding conflict for node node-123");
        assert!(error.is_retryable());
    }

    #[test]
    fn test_reservation_timeout_error() {
        let error = SchedulerError::reservation_timeout("rsv-456");
        assert_eq!(
            error.to_string(),
            "Reservation timeout for reservation rsv-456"
        );
        assert!(error.is_retryable());
    }

    #[test]
    fn test_node_not_found_error() {
        let error = SchedulerError::node_not_found("node-789");
        assert_eq!(error.to_string(), "Node not found: node-789");
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_node_unhealthy_error() {
        let error = SchedulerError::node_unhealthy("node-123", "DiskPressure");
        assert_eq!(
            error.to_string(),
            "Node node-123 is unhealthy: DiskPressure"
        );
        assert!(error.is_retryable());
    }

    #[test]
    fn test_constraint_violation_error() {
        let error = SchedulerError::constraint_violation("Required anti-affinity violated");
        assert_eq!(
            error.to_string(),
            "Constraint violation: Required anti-affinity violated"
        );
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_error_serialization() {
        let error = SchedulerError::insufficient_resources("memory", 16384, 8192);
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: SchedulerError = serde_json::from_str(&json).unwrap();
        assert_eq!(error, deserialized);
    }
}
