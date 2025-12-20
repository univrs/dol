//! Common types used across the scheduler interface.
//!
//! This module defines shared data structures used by multiple
//! phases of the scheduling pipeline.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A node in the cluster available for scheduling.
///
/// Nodes represent compute resources (physical or virtual machines)
/// where containers can be scheduled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// Unique node identifier.
    pub id: String,

    /// Human-readable node name.
    pub name: String,

    /// Total capacity for each resource type.
    ///
    /// Maps resource type (cpu, memory, storage) to total capacity.
    /// Example: {"cpu": "16000m", "memory": "64Gi", "storage": "1Ti"}
    pub capacity: HashMap<String, String>,

    /// Currently allocated resources.
    ///
    /// Maps resource type to allocated quantity.
    /// Must be less than or equal to capacity.
    pub allocated: HashMap<String, String>,

    /// Node labels for filtering and affinity.
    ///
    /// Key-value pairs for node selection.
    /// Example: {"zone": "us-west-1a", "instance-type": "m5.xlarge"}
    pub labels: HashMap<String, String>,

    /// Node taints that prevent scheduling.
    ///
    /// Pods must have matching tolerations to be scheduled.
    /// Example: {"dedicated": "gpu-workload"}
    pub taints: HashMap<String, String>,

    /// Whether the node is currently schedulable.
    ///
    /// False if the node is cordoned, draining, or unavailable.
    pub schedulable: bool,
}

impl Node {
    /// Creates a new node.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique node identifier
    /// * `name` - Human-readable name
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            capacity: HashMap::new(),
            allocated: HashMap::new(),
            labels: HashMap::new(),
            taints: HashMap::new(),
            schedulable: true,
        }
    }

    /// Adds resource capacity to the node.
    ///
    /// # Arguments
    ///
    /// * `resource_type` - Type of resource (cpu, memory, etc.)
    /// * `capacity` - Total capacity
    pub fn with_capacity(mut self, resource_type: String, capacity: String) -> Self {
        self.capacity.insert(resource_type, capacity);
        self
    }

    /// Adds a label to the node.
    ///
    /// # Arguments
    ///
    /// * `key` - Label key
    /// * `value` - Label value
    pub fn with_label(mut self, key: String, value: String) -> Self {
        self.labels.insert(key, value);
        self
    }

    /// Adds a taint to the node.
    ///
    /// # Arguments
    ///
    /// * `key` - Taint key
    /// * `value` - Taint value
    pub fn with_taint(mut self, key: String, value: String) -> Self {
        self.taints.insert(key, value);
        self
    }

    /// Sets the schedulable status.
    ///
    /// # Arguments
    ///
    /// * `schedulable` - Whether the node accepts new pods
    pub fn with_schedulable(mut self, schedulable: bool) -> Self {
        self.schedulable = schedulable;
        self
    }

    /// Calculates available resources for a given resource type.
    ///
    /// Returns None if the resource type is not defined on this node.
    pub fn available(&self, resource_type: &str) -> Option<String> {
        let capacity = self.capacity.get(resource_type)?;
        let allocated = self.allocated.get(resource_type).cloned().unwrap_or_else(|| "0".to_string());

        // Note: In a real implementation, this would do proper resource arithmetic
        // For now, we just return the difference as a string
        Some(format!("{} (cap: {}, alloc: {})", resource_type, capacity, allocated))
    }

    /// Checks if the node has a specific label with the given value.
    pub fn has_label(&self, key: &str, value: &str) -> bool {
        self.labels.get(key).map(|v| v == value).unwrap_or(false)
    }

    /// Checks if the node has a specific taint.
    pub fn has_taint(&self, key: &str) -> bool {
        self.taints.contains_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::new("node-01".to_string(), "worker-1".to_string());

        assert_eq!(node.id, "node-01");
        assert_eq!(node.name, "worker-1");
        assert!(node.schedulable);
        assert!(node.capacity.is_empty());
        assert!(node.labels.is_empty());
    }

    #[test]
    fn test_node_builder_pattern() {
        let node = Node::new("node-01".to_string(), "worker-1".to_string())
            .with_capacity("cpu".to_string(), "16000m".to_string())
            .with_capacity("memory".to_string(), "64Gi".to_string())
            .with_label("zone".to_string(), "us-west-1a".to_string())
            .with_taint("dedicated".to_string(), "gpu".to_string())
            .with_schedulable(true);

        assert_eq!(node.capacity.get("cpu").unwrap(), "16000m");
        assert_eq!(node.capacity.get("memory").unwrap(), "64Gi");
        assert!(node.has_label("zone", "us-west-1a"));
        assert!(node.has_taint("dedicated"));
        assert!(node.schedulable);
    }

    #[test]
    fn test_node_has_label() {
        let node = Node::new("node-01".to_string(), "worker-1".to_string())
            .with_label("zone".to_string(), "us-west-1a".to_string());

        assert!(node.has_label("zone", "us-west-1a"));
        assert!(!node.has_label("zone", "us-east-1"));
        assert!(!node.has_label("region", "us-west"));
    }

    #[test]
    fn test_node_has_taint() {
        let node = Node::new("node-01".to_string(), "worker-1".to_string())
            .with_taint("dedicated".to_string(), "gpu".to_string());

        assert!(node.has_taint("dedicated"));
        assert!(!node.has_taint("other"));
    }
}
