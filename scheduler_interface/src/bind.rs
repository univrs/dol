//! Binding phase of the scheduling algorithm.
//!
//! This module implements the `scheduling.bind` trait from the Metal DOL
//! scheduling ontology. It handles the final step of committing a container
//! to a selected node.
//!
//! # Binding Modes
//!
//! The binder supports three binding modes:
//!
//! - **Optimistic**: Bind immediately, assuming the reservation is valid
//! - **Pessimistic**: Verify reservation and resources before binding
//! - **TwoPhase**: Prepare and commit in two phases for maximum consistency
//!
//! # Example
//!
//! ```rust
//! use scheduler_interface::bind::{Binder, BindingMode, BindRequest, BindResult};
//! use std::collections::HashMap;
//!
//! // Implement the Binder trait for your scheduler
//! struct MyBinder;
//!
//! impl Binder for MyBinder {
//!     fn bind(
//!         &self,
//!         request: BindRequest,
//!         mode: BindingMode,
//!     ) -> Result<BindResult, scheduler_interface::SchedulerError> {
//!         // Validate reservation
//!         // Update node resources
//!         // Commit binding
//!         todo!()
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Binding mode controlling consistency guarantees.
///
/// Different modes provide different trade-offs between performance
/// and consistency during the binding phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindingMode {
    /// Optimistic binding with minimal validation.
    ///
    /// Assumes the reservation is valid and resources are available.
    /// Fastest mode but may fail if assumptions are violated.
    /// Best for: Single-scheduler deployments, low contention.
    Optimistic,

    /// Pessimistic binding with full validation.
    ///
    /// Verifies reservation validity and resource availability before binding.
    /// Slower but more reliable in concurrent environments.
    /// Best for: Multi-scheduler deployments, high contention.
    Pessimistic,

    /// Two-phase binding for maximum consistency.
    ///
    /// Phase 1: Prepare and lock resources
    /// Phase 2: Commit or rollback
    /// Slowest but provides ACID-like guarantees.
    /// Best for: Critical workloads, distributed schedulers.
    TwoPhase,
}

impl Default for BindingMode {
    fn default() -> Self {
        BindingMode::Pessimistic
    }
}

/// Request to bind a container to a node.
///
/// Contains all information needed to commit the scheduling decision
/// and update resource allocations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BindRequest {
    /// Container/Pod identifier to bind.
    pub container_id: String,

    /// Target node identifier.
    pub node_id: String,

    /// Reservation ID from the selection phase.
    ///
    /// Must match an active, non-expired reservation on the target node.
    pub reservation_id: String,

    /// Resources to allocate on the node.
    ///
    /// Maps resource type to quantity.
    /// Example: {"cpu": "2000m", "memory": "4Gi", "storage": "10Gi"}
    pub resources: HashMap<String, String>,
}

impl BindRequest {
    /// Creates a new bind request.
    ///
    /// # Arguments
    ///
    /// * `container_id` - Container identifier
    /// * `node_id` - Target node
    /// * `reservation_id` - Active reservation
    /// * `resources` - Resources to allocate
    pub fn new(
        container_id: String,
        node_id: String,
        reservation_id: String,
        resources: HashMap<String, String>,
    ) -> Self {
        Self {
            container_id,
            node_id,
            reservation_id,
            resources,
        }
    }
}

/// Result of a binding operation.
///
/// Indicates whether the binding succeeded and provides details
/// about resource updates on the target node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BindResult {
    /// Whether the binding succeeded.
    pub success: bool,

    /// Timestamp when binding was committed (Unix timestamp in seconds).
    ///
    /// Only set if `success` is true.
    pub bound_at: Option<u64>,

    /// Resource updates applied to the node.
    ///
    /// Maps resource type to the new allocated amount.
    /// Useful for verification and auditing.
    pub resource_updates: HashMap<String, ResourceUpdate>,
}

impl BindResult {
    /// Creates a successful bind result.
    ///
    /// # Arguments
    ///
    /// * `bound_at` - Timestamp when binding occurred
    /// * `resource_updates` - Applied resource updates
    pub fn success(bound_at: u64, resource_updates: HashMap<String, ResourceUpdate>) -> Self {
        Self {
            success: true,
            bound_at: Some(bound_at),
            resource_updates,
        }
    }

    /// Creates a failed bind result.
    ///
    /// # Arguments
    ///
    /// * `resource_updates` - Attempted resource updates (may be partial)
    pub fn failure(resource_updates: HashMap<String, ResourceUpdate>) -> Self {
        Self {
            success: false,
            bound_at: None,
            resource_updates,
        }
    }
}

/// Details of a resource update during binding.
///
/// Tracks the before and after state of a resource allocation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceUpdate {
    /// Resource type (cpu, memory, storage, etc.).
    pub resource_type: String,

    /// Amount allocated before this binding.
    pub previous_allocated: String,

    /// Amount allocated after this binding.
    pub new_allocated: String,

    /// Total capacity of this resource on the node.
    pub total_capacity: String,
}

impl ResourceUpdate {
    /// Creates a new resource update record.
    ///
    /// # Arguments
    ///
    /// * `resource_type` - Type of resource
    /// * `previous_allocated` - Previous allocation
    /// * `new_allocated` - New allocation
    /// * `total_capacity` - Total capacity
    pub fn new(
        resource_type: String,
        previous_allocated: String,
        new_allocated: String,
        total_capacity: String,
    ) -> Self {
        Self {
            resource_type,
            previous_allocated,
            new_allocated,
            total_capacity,
        }
    }
}

/// Trait for implementing node binding logic.
///
/// The binder is responsible for:
/// 1. Validating reservations and resource availability
/// 2. Committing the container to the selected node
/// 3. Updating resource allocations atomically
/// 4. Handling binding failures and rollback
///
/// Implementations should ensure:
/// - Atomic resource updates (all or nothing)
/// - Proper handling of reservation expiration
/// - Accurate resource tracking
/// - Idempotent binding operations
pub trait Binder: Send + Sync {
    /// Binds a container to a node.
    ///
    /// # Arguments
    ///
    /// * `request` - Binding request with container and node details
    /// * `mode` - Binding mode controlling consistency level
    ///
    /// # Returns
    ///
    /// A `BindResult` indicating success or failure with resource updates,
    /// or a `SchedulerError` for unexpected failures.
    ///
    /// # Errors
    ///
    /// - `ReservationExpired` - The reservation has expired
    /// - `InsufficientResources` - Not enough resources available
    /// - `NodeNotFound` - Target node does not exist
    /// - `BindingFailed` - Generic binding failure
    fn bind(
        &self,
        request: BindRequest,
        mode: BindingMode,
    ) -> Result<BindResult, super::SchedulerError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binding_mode_default() {
        assert_eq!(BindingMode::default(), BindingMode::Pessimistic);
    }

    #[test]
    fn test_bind_request_creation() {
        let mut resources = HashMap::new();
        resources.insert("cpu".to_string(), "2000m".to_string());
        resources.insert("memory".to_string(), "4Gi".to_string());

        let request = BindRequest::new(
            "container-123".to_string(),
            "node-01".to_string(),
            "res-456".to_string(),
            resources.clone(),
        );

        assert_eq!(request.container_id, "container-123");
        assert_eq!(request.node_id, "node-01");
        assert_eq!(request.reservation_id, "res-456");
        assert_eq!(request.resources.get("cpu").unwrap(), "2000m");
    }

    #[test]
    fn test_bind_result_success() {
        let mut updates = HashMap::new();
        updates.insert(
            "cpu".to_string(),
            ResourceUpdate::new(
                "cpu".to_string(),
                "8000m".to_string(),
                "10000m".to_string(),
                "16000m".to_string(),
            ),
        );

        let result = BindResult::success(1000000, updates);

        assert!(result.success);
        assert_eq!(result.bound_at, Some(1000000));
        assert!(result.resource_updates.contains_key("cpu"));
    }

    #[test]
    fn test_bind_result_failure() {
        let updates = HashMap::new();
        let result = BindResult::failure(updates);

        assert!(!result.success);
        assert_eq!(result.bound_at, None);
    }

    #[test]
    fn test_resource_update_creation() {
        let update = ResourceUpdate::new(
            "memory".to_string(),
            "32Gi".to_string(),
            "36Gi".to_string(),
            "64Gi".to_string(),
        );

        assert_eq!(update.resource_type, "memory");
        assert_eq!(update.previous_allocated, "32Gi");
        assert_eq!(update.new_allocated, "36Gi");
        assert_eq!(update.total_capacity, "64Gi");
    }
}
