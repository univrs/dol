//! Resource management types for container scheduling.
//!
//! This module implements the `scheduling.resources` gene, providing types and operations
//! for managing node and container resources in a Kubernetes-style scheduler.

use serde::{Deserialize, Serialize};

/// Physical resources available on a node.
///
/// Represents the total capacity of a node in the cluster.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeResources {
    /// CPU capacity in cores (e.g., 4.0 for 4 cores)
    pub cpu_capacity: f64,
    /// Memory capacity in bytes
    pub memory_capacity: u64,
    /// Disk capacity in bytes
    pub disk_capacity: u64,
    /// Number of GPU devices
    pub gpu_count: u32,
}

impl NodeResources {
    /// Creates a new NodeResources instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use scheduler_interface::resources::NodeResources;
    ///
    /// let node = NodeResources::new(8.0, 16 * 1024 * 1024 * 1024, 500 * 1024 * 1024 * 1024, 2);
    /// assert_eq!(node.cpu_capacity, 8.0);
    /// ```
    pub fn new(
        cpu_capacity: f64,
        memory_capacity: u64,
        disk_capacity: u64,
        gpu_count: u32,
    ) -> Self {
        Self {
            cpu_capacity,
            memory_capacity,
            disk_capacity,
            gpu_count,
        }
    }

    /// Returns zero resources.
    pub fn zero() -> Self {
        Self {
            cpu_capacity: 0.0,
            memory_capacity: 0,
            disk_capacity: 0,
            gpu_count: 0,
        }
    }
}

/// Resource requests and limits for a container.
///
/// Follows Kubernetes resource model with requests (guaranteed) and limits (max allowed).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContainerResources {
    /// CPU request in cores (guaranteed allocation)
    pub cpu_request: f64,
    /// CPU limit in cores (maximum allowed)
    pub cpu_limit: f64,
    /// Memory request in bytes (guaranteed allocation)
    pub memory_request: u64,
    /// Memory limit in bytes (maximum allowed)
    pub memory_limit: u64,
}

impl ContainerResources {
    /// Creates a new ContainerResources instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use scheduler_interface::resources::ContainerResources;
    ///
    /// let container = ContainerResources::new(1.0, 2.0, 512 * 1024 * 1024, 1024 * 1024 * 1024);
    /// assert_eq!(container.cpu_request, 1.0);
    /// assert_eq!(container.cpu_limit, 2.0);
    /// ```
    pub fn new(cpu_request: f64, cpu_limit: f64, memory_request: u64, memory_limit: u64) -> Self {
        Self {
            cpu_request,
            cpu_limit,
            memory_request,
            memory_limit,
        }
    }

    /// Returns zero resource requests.
    pub fn zero() -> Self {
        Self {
            cpu_request: 0.0,
            cpu_limit: 0.0,
            memory_request: 0,
            memory_limit: 0,
        }
    }

    /// Determines the QoS class based on resource requests and limits.
    ///
    /// # Examples
    ///
    /// ```
    /// use scheduler_interface::resources::{ContainerResources, QoSClass};
    ///
    /// // Guaranteed: requests == limits and all set
    /// let guaranteed = ContainerResources::new(1.0, 1.0, 1024, 1024);
    /// assert_eq!(guaranteed.qos_class(), QoSClass::Guaranteed);
    ///
    /// // BestEffort: no requests or limits
    /// let best_effort = ContainerResources::zero();
    /// assert_eq!(best_effort.qos_class(), QoSClass::BestEffort);
    /// ```
    pub fn qos_class(&self) -> QoSClass {
        if self.cpu_request == 0.0 && self.memory_request == 0 {
            QoSClass::BestEffort
        } else if self.cpu_request == self.cpu_limit && self.memory_request == self.memory_limit {
            QoSClass::Guaranteed
        } else {
            QoSClass::Burstable
        }
    }
}

/// Tracks allocatable (available) resources on a node.
///
/// This represents resources that are currently available for allocation,
/// calculated as total capacity minus already allocated resources.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AllocatableResources {
    /// Available CPU in cores
    pub cpu_available: f64,
    /// Available memory in bytes
    pub memory_available: u64,
    /// Available disk in bytes
    pub disk_available: u64,
    /// Available GPU count
    pub gpu_available: u32,
}

impl AllocatableResources {
    /// Creates a new AllocatableResources instance.
    pub fn new(
        cpu_available: f64,
        memory_available: u64,
        disk_available: u64,
        gpu_available: u32,
    ) -> Self {
        Self {
            cpu_available,
            memory_available,
            disk_available,
            gpu_available,
        }
    }

    /// Creates allocatable resources from node resources (all available).
    pub fn from_node(node: &NodeResources) -> Self {
        Self {
            cpu_available: node.cpu_capacity,
            memory_available: node.memory_capacity,
            disk_available: node.disk_capacity,
            gpu_available: node.gpu_count,
        }
    }

    /// Adds resources back to the available pool.
    ///
    /// # Examples
    ///
    /// ```
    /// use scheduler_interface::resources::{AllocatableResources, ContainerResources};
    ///
    /// let mut allocatable = AllocatableResources::new(4.0, 8192, 10000, 1);
    /// let container = ContainerResources::new(1.0, 2.0, 2048, 4096);
    /// allocatable.add(&container);
    /// assert_eq!(allocatable.cpu_available, 5.0);
    /// assert_eq!(allocatable.memory_available, 10240);
    /// ```
    pub fn add(&mut self, container: &ContainerResources) {
        self.cpu_available += container.cpu_request;
        self.memory_available += container.memory_request;
    }

    /// Subtracts resources from the available pool.
    ///
    /// # Examples
    ///
    /// ```
    /// use scheduler_interface::resources::{AllocatableResources, ContainerResources};
    ///
    /// let mut allocatable = AllocatableResources::new(4.0, 8192, 10000, 1);
    /// let container = ContainerResources::new(1.0, 2.0, 2048, 4096);
    /// allocatable.subtract(&container);
    /// assert_eq!(allocatable.cpu_available, 3.0);
    /// assert_eq!(allocatable.memory_available, 6144);
    /// ```
    pub fn subtract(&mut self, container: &ContainerResources) {
        self.cpu_available -= container.cpu_request;
        self.memory_available = self
            .memory_available
            .saturating_sub(container.memory_request);
    }

    /// Checks if container resources fit within available resources.
    ///
    /// # Examples
    ///
    /// ```
    /// use scheduler_interface::resources::{AllocatableResources, ContainerResources};
    ///
    /// let allocatable = AllocatableResources::new(4.0, 8192, 10000, 1);
    /// let small = ContainerResources::new(1.0, 2.0, 2048, 4096);
    /// let large = ContainerResources::new(8.0, 16.0, 16384, 32768);
    ///
    /// assert!(allocatable.fits(&small));
    /// assert!(!allocatable.fits(&large));
    /// ```
    pub fn fits(&self, container: &ContainerResources) -> bool {
        self.cpu_available >= container.cpu_request
            && self.memory_available >= container.memory_request
    }

    /// Returns the resource utilization as a percentage (0.0 to 1.0).
    ///
    /// Calculated based on the most constrained resource.
    pub fn utilization(&self, total: &NodeResources) -> f64 {
        let cpu_util = 1.0 - (self.cpu_available / total.cpu_capacity);
        let mem_util = 1.0 - (self.memory_available as f64 / total.memory_capacity as f64);
        cpu_util.max(mem_util).clamp(0.0, 1.0)
    }
}

/// Quality of Service class for containers.
///
/// Determines scheduling and eviction priority following Kubernetes QoS model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QoSClass {
    /// Guaranteed: requests == limits for all resources.
    /// Highest priority, last to be evicted.
    Guaranteed,

    /// Burstable: requests < limits or only some resources specified.
    /// Medium priority.
    Burstable,

    /// BestEffort: no requests or limits specified.
    /// Lowest priority, first to be evicted.
    BestEffort,
}

impl QoSClass {
    /// Returns the eviction priority (higher = evicted first).
    ///
    /// # Examples
    ///
    /// ```
    /// use scheduler_interface::resources::QoSClass;
    ///
    /// assert!(QoSClass::BestEffort.eviction_priority() > QoSClass::Burstable.eviction_priority());
    /// assert!(QoSClass::Burstable.eviction_priority() > QoSClass::Guaranteed.eviction_priority());
    /// ```
    pub fn eviction_priority(&self) -> u8 {
        match self {
            QoSClass::BestEffort => 3,
            QoSClass::Burstable => 2,
            QoSClass::Guaranteed => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_resources_creation() {
        let node = NodeResources::new(8.0, 16384, 500000, 2);
        assert_eq!(node.cpu_capacity, 8.0);
        assert_eq!(node.memory_capacity, 16384);
        assert_eq!(node.disk_capacity, 500000);
        assert_eq!(node.gpu_count, 2);
    }

    #[test]
    fn test_container_qos_guaranteed() {
        let container = ContainerResources::new(2.0, 2.0, 4096, 4096);
        assert_eq!(container.qos_class(), QoSClass::Guaranteed);
    }

    #[test]
    fn test_container_qos_burstable() {
        let container = ContainerResources::new(1.0, 2.0, 2048, 4096);
        assert_eq!(container.qos_class(), QoSClass::Burstable);
    }

    #[test]
    fn test_container_qos_besteffort() {
        let container = ContainerResources::zero();
        assert_eq!(container.qos_class(), QoSClass::BestEffort);
    }

    #[test]
    fn test_allocatable_fits() {
        let allocatable = AllocatableResources::new(4.0, 8192, 10000, 1);
        let small = ContainerResources::new(1.0, 2.0, 2048, 4096);
        let large = ContainerResources::new(8.0, 16.0, 16384, 32768);

        assert!(allocatable.fits(&small));
        assert!(!allocatable.fits(&large));
    }

    #[test]
    fn test_allocatable_add_subtract() {
        let mut allocatable = AllocatableResources::new(4.0, 8192, 10000, 1);
        let container = ContainerResources::new(1.0, 2.0, 2048, 4096);

        allocatable.subtract(&container);
        assert_eq!(allocatable.cpu_available, 3.0);
        assert_eq!(allocatable.memory_available, 6144);

        allocatable.add(&container);
        assert_eq!(allocatable.cpu_available, 4.0);
        assert_eq!(allocatable.memory_available, 8192);
    }

    #[test]
    fn test_qos_eviction_priority() {
        assert!(QoSClass::BestEffort.eviction_priority() > QoSClass::Burstable.eviction_priority());
        assert!(QoSClass::Burstable.eviction_priority() > QoSClass::Guaranteed.eviction_priority());
    }
}
