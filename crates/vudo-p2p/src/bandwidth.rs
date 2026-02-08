//! Bandwidth management and adaptive sync rate limiting.

use crate::error::{P2PError, Result};
use crate::sync_protocol::PeerId;
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Sync task priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SyncPriority {
    /// Low priority (background sync).
    Low = 0,
    /// Normal priority.
    Normal = 1,
    /// High priority (user-initiated).
    High = 2,
    /// Urgent priority.
    Urgent = 3,
}

/// Sync task.
#[derive(Debug, Clone)]
pub struct SyncTask {
    /// Task ID.
    pub id: u64,
    /// Peer ID.
    pub peer_id: PeerId,
    /// Document namespace.
    pub namespace: String,
    /// Document ID.
    pub doc_id: String,
    /// Priority.
    pub priority: SyncPriority,
    /// Created at.
    pub created_at: Instant,
    /// Estimated size (bytes).
    pub estimated_size: usize,
}

/// Bandwidth statistics.
#[derive(Debug, Clone)]
pub struct BandwidthStats {
    /// Bytes sent in current window.
    pub bytes_sent: u64,
    /// Bytes received in current window.
    pub bytes_received: u64,
    /// Current send rate (bytes/sec).
    pub send_rate: u64,
    /// Current receive rate (bytes/sec).
    pub receive_rate: u64,
    /// Is connection metered?
    pub is_metered: bool,
    /// Current rate limit (bytes/sec).
    pub rate_limit: u64,
}

/// Priority queue for sync tasks.
struct PriorityQueue {
    /// Tasks by priority.
    tasks: HashMap<SyncPriority, VecDeque<SyncTask>>,
    /// Next task ID.
    next_id: u64,
}

impl PriorityQueue {
    fn new() -> Self {
        let mut tasks = HashMap::new();
        tasks.insert(SyncPriority::Low, VecDeque::new());
        tasks.insert(SyncPriority::Normal, VecDeque::new());
        tasks.insert(SyncPriority::High, VecDeque::new());
        tasks.insert(SyncPriority::Urgent, VecDeque::new());

        Self { tasks, next_id: 0 }
    }

    fn enqueue(&mut self, mut task: SyncTask) -> u64 {
        task.id = self.next_id;
        self.next_id += 1;

        let id = task.id;
        self.tasks
            .get_mut(&task.priority)
            .unwrap()
            .push_back(task);

        id
    }

    fn dequeue(&mut self) -> Option<SyncTask> {
        // Dequeue highest priority task
        for priority in [
            SyncPriority::Urgent,
            SyncPriority::High,
            SyncPriority::Normal,
            SyncPriority::Low,
        ] {
            if let Some(task) = self.tasks.get_mut(&priority).unwrap().pop_front() {
                return Some(task);
            }
        }

        None
    }

    fn len(&self) -> usize {
        self.tasks.values().map(|q| q.len()).sum()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Bandwidth manager.
pub struct BandwidthManager {
    /// Is connection metered?
    is_metered: Arc<AtomicBool>,
    /// Current rate limit (bytes/sec).
    rate_limit: Arc<AtomicU64>,
    /// Bytes sent counter.
    bytes_sent: Arc<AtomicU64>,
    /// Bytes received counter.
    bytes_received: Arc<AtomicU64>,
    /// Sync task queue.
    task_queue: Arc<RwLock<PriorityQueue>>,
    /// Rate calculation window.
    window_duration: Duration,
    /// Timestamp samples for rate calculation.
    samples: Arc<RwLock<VecDeque<(Instant, u64, u64)>>>, // (timestamp, bytes_sent, bytes_received)
}

impl BandwidthManager {
    /// Create a new bandwidth manager.
    pub fn new() -> Self {
        Self {
            is_metered: Arc::new(AtomicBool::new(false)),
            rate_limit: Arc::new(AtomicU64::new(u64::MAX)), // Unlimited by default
            bytes_sent: Arc::new(AtomicU64::new(0)),
            bytes_received: Arc::new(AtomicU64::new(0)),
            task_queue: Arc::new(RwLock::new(PriorityQueue::new())),
            window_duration: Duration::from_secs(10),
            samples: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Detect if connection is metered.
    pub async fn detect_metered_connection(&self) -> Result<bool> {
        #[cfg(target_arch = "wasm32")]
        {
            // In browser, try to use NetworkInformation API
            self.detect_metered_wasm().await
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // On native platforms, check environment hints
            // This is a simplified implementation
            // In production, you'd check OS-specific APIs
            Ok(false)
        }
    }

    #[cfg(target_arch = "wasm32")]
    async fn detect_metered_wasm(&self) -> Result<bool> {
        use wasm_bindgen::JsCast;
        use web_sys::window;

        let window = window().ok_or_else(|| P2PError::Internal("No window".to_string()))?;

        let navigator = window.navigator();

        // Try to access NetworkInformation API
        // Note: This API is not universally supported
        if let Ok(connection) = js_sys::Reflect::get(&navigator, &"connection".into()) {
            if !connection.is_undefined() {
                if let Ok(metered) = js_sys::Reflect::get(&connection, &"saveData".into()) {
                    if let Some(is_metered) = metered.as_bool() {
                        self.set_metered(is_metered);
                        return Ok(is_metered);
                    }
                }
            }
        }

        // Default to false if API not available
        Ok(false)
    }

    /// Set metered connection status.
    pub fn set_metered(&self, is_metered: bool) {
        self.is_metered.store(is_metered, Ordering::SeqCst);

        // Adjust rate limit based on metered status
        let new_limit = if is_metered {
            1024 * 1024 // 1 MB/s for metered connections
        } else {
            u64::MAX // Unlimited for non-metered
        };

        self.rate_limit.store(new_limit, Ordering::SeqCst);

        info!(
            "Connection metered status: {} (rate limit: {} bytes/sec)",
            is_metered,
            if new_limit == u64::MAX {
                "unlimited".to_string()
            } else {
                new_limit.to_string()
            }
        );
    }

    /// Set custom rate limit.
    pub fn set_rate_limit(&self, bytes_per_sec: u64) {
        self.rate_limit.store(bytes_per_sec, Ordering::SeqCst);
        info!("Rate limit set to {} bytes/sec", bytes_per_sec);
    }

    /// Record bytes sent.
    pub fn record_sent(&self, bytes: usize) {
        self.bytes_sent.fetch_add(bytes as u64, Ordering::SeqCst);
        self.add_sample();
    }

    /// Record bytes received.
    pub fn record_received(&self, bytes: usize) {
        self.bytes_received.fetch_add(bytes as u64, Ordering::SeqCst);
        self.add_sample();
    }

    /// Add a sample for rate calculation.
    fn add_sample(&self) {
        let mut samples = self.samples.write();
        let now = Instant::now();
        let sent = self.bytes_sent.load(Ordering::SeqCst);
        let received = self.bytes_received.load(Ordering::SeqCst);

        samples.push_back((now, sent, received));

        // Keep only samples within window
        let cutoff = now - self.window_duration;
        while let Some((timestamp, _, _)) = samples.front() {
            if *timestamp < cutoff {
                samples.pop_front();
            } else {
                break;
            }
        }
    }

    /// Calculate current send rate.
    pub fn send_rate(&self) -> u64 {
        self.calculate_rate(true)
    }

    /// Calculate current receive rate.
    pub fn receive_rate(&self) -> u64 {
        self.calculate_rate(false)
    }

    /// Calculate rate (send or receive).
    fn calculate_rate(&self, send: bool) -> u64 {
        let samples = self.samples.read();

        if samples.len() < 2 {
            return 0;
        }

        let (first_time, first_sent, first_received) = samples.front().unwrap();
        let (last_time, last_sent, last_received) = samples.back().unwrap();

        let duration = last_time.duration_since(*first_time).as_secs_f64();
        if duration == 0.0 {
            return 0;
        }

        let bytes = if send {
            last_sent - first_sent
        } else {
            last_received - first_received
        };

        (bytes as f64 / duration) as u64
    }

    /// Check if we can send given number of bytes without exceeding rate limit.
    pub fn can_send(&self, bytes: usize) -> bool {
        let rate_limit = self.rate_limit.load(Ordering::SeqCst);
        if rate_limit == u64::MAX {
            return true;
        }

        let current_rate = self.send_rate();
        current_rate + bytes as u64 <= rate_limit
    }

    /// Schedule a sync task.
    pub async fn schedule_sync(&self, task: SyncTask) -> Result<u64> {
        debug!(
            "Scheduling sync task: {}/{} (priority: {:?})",
            task.namespace, task.doc_id, task.priority
        );

        let id = self.task_queue.write().enqueue(task);
        Ok(id)
    }

    /// Get next task to execute.
    pub async fn next_task(&self) -> Option<SyncTask> {
        self.task_queue.write().dequeue()
    }

    /// Get task queue length.
    pub fn queue_length(&self) -> usize {
        self.task_queue.read().len()
    }

    /// Get bandwidth statistics.
    pub fn stats(&self) -> BandwidthStats {
        BandwidthStats {
            bytes_sent: self.bytes_sent.load(Ordering::SeqCst),
            bytes_received: self.bytes_received.load(Ordering::SeqCst),
            send_rate: self.send_rate(),
            receive_rate: self.receive_rate(),
            is_metered: self.is_metered.load(Ordering::SeqCst),
            rate_limit: self.rate_limit.load(Ordering::SeqCst),
        }
    }

    /// Reset counters.
    pub fn reset(&self) {
        self.bytes_sent.store(0, Ordering::SeqCst);
        self.bytes_received.store(0, Ordering::SeqCst);
        self.samples.write().clear();
    }
}

impl Default for BandwidthManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bandwidth_manager_creation() {
        let manager = BandwidthManager::new();
        let stats = manager.stats();

        assert_eq!(stats.bytes_sent, 0);
        assert_eq!(stats.bytes_received, 0);
        assert!(!stats.is_metered);
    }

    #[test]
    fn test_record_bytes() {
        let manager = BandwidthManager::new();

        manager.record_sent(1000);
        manager.record_received(500);

        let stats = manager.stats();
        assert_eq!(stats.bytes_sent, 1000);
        assert_eq!(stats.bytes_received, 500);
    }

    #[test]
    fn test_metered_connection() {
        let manager = BandwidthManager::new();

        assert!(!manager.is_metered.load(Ordering::SeqCst));

        manager.set_metered(true);
        assert!(manager.is_metered.load(Ordering::SeqCst));

        let stats = manager.stats();
        assert!(stats.is_metered);
        assert_eq!(stats.rate_limit, 1024 * 1024); // 1 MB/s
    }

    #[test]
    fn test_custom_rate_limit() {
        let manager = BandwidthManager::new();

        manager.set_rate_limit(500_000); // 500 KB/s

        let stats = manager.stats();
        assert_eq!(stats.rate_limit, 500_000);
    }

    #[tokio::test]
    async fn test_priority_queue() {
        let manager = BandwidthManager::new();

        // Add tasks with different priorities
        let task_low = SyncTask {
            id: 0,
            peer_id: "peer1".to_string(),
            namespace: "users".to_string(),
            doc_id: "alice".to_string(),
            priority: SyncPriority::Low,
            created_at: Instant::now(),
            estimated_size: 1000,
        };

        let task_high = SyncTask {
            id: 0,
            peer_id: "peer1".to_string(),
            namespace: "users".to_string(),
            doc_id: "bob".to_string(),
            priority: SyncPriority::High,
            created_at: Instant::now(),
            estimated_size: 1000,
        };

        manager.schedule_sync(task_low).await.unwrap();
        manager.schedule_sync(task_high).await.unwrap();

        // High priority should come first
        let next = manager.next_task().await.unwrap();
        assert_eq!(next.priority, SyncPriority::High);
        assert_eq!(next.doc_id, "bob");

        let next = manager.next_task().await.unwrap();
        assert_eq!(next.priority, SyncPriority::Low);
        assert_eq!(next.doc_id, "alice");
    }

    #[test]
    fn test_can_send() {
        let manager = BandwidthManager::new();

        // Unlimited by default
        assert!(manager.can_send(1_000_000));

        // Set a limit
        manager.set_rate_limit(1000);

        // Should be able to send small amounts
        assert!(manager.can_send(100));
    }
}
