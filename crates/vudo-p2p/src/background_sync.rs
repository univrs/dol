//! Background sync task management.

use crate::bandwidth::{BandwidthManager, SyncPriority, SyncTask};
use crate::error::{P2PError, Result};
use crate::sync_protocol::{PeerId, SyncMessage, SyncProtocol};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};
use vudo_state::DocumentId;

/// Background sync configuration.
#[derive(Debug, Clone)]
pub struct BackgroundSyncConfig {
    /// Sync interval.
    pub sync_interval: Duration,
    /// Maximum retry attempts.
    pub max_retries: u32,
    /// Retry backoff base duration.
    pub retry_backoff: Duration,
    /// Enable exponential backoff.
    pub exponential_backoff: bool,
}

impl Default for BackgroundSyncConfig {
    fn default() -> Self {
        Self {
            sync_interval: Duration::from_secs(30),
            max_retries: 3,
            retry_backoff: Duration::from_secs(5),
            exponential_backoff: true,
        }
    }
}

/// Sync task state.
#[derive(Debug, Clone)]
struct SyncTaskState {
    /// Task.
    task: SyncTask,
    /// Retry count.
    retry_count: u32,
    /// Last attempt timestamp.
    last_attempt: Option<std::time::Instant>,
}

/// Background sync manager.
pub struct BackgroundSync {
    /// Configuration.
    config: BackgroundSyncConfig,
    /// Is running?
    is_running: Arc<AtomicBool>,
    /// Bandwidth manager.
    bandwidth_manager: Arc<BandwidthManager>,
    /// Pending tasks.
    pending_tasks: Arc<RwLock<HashMap<String, SyncTaskState>>>, // Key: "peer_id:namespace:doc_id"
}

impl BackgroundSync {
    /// Create a new background sync manager.
    pub fn new(config: BackgroundSyncConfig, bandwidth_manager: Arc<BandwidthManager>) -> Self {
        Self {
            config,
            is_running: Arc::new(AtomicBool::new(false)),
            bandwidth_manager,
            pending_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start background sync.
    pub fn start(&self) {
        if self.is_running.swap(true, Ordering::SeqCst) {
            warn!("Background sync already running");
            return;
        }

        info!("Starting background sync");

        #[cfg(target_arch = "wasm32")]
        {
            self.spawn_worker();
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.spawn_task();
        }
    }

    /// Stop background sync.
    pub fn stop(&self) {
        info!("Stopping background sync");
        self.is_running.store(false, Ordering::SeqCst);
    }

    /// Add a document to background sync.
    pub fn add_document(&self, peer_id: PeerId, namespace: String, doc_id: String) {
        let task = SyncTask {
            id: 0,
            peer_id: peer_id.clone(),
            namespace: namespace.clone(),
            doc_id: doc_id.clone(),
            priority: SyncPriority::Low, // Background tasks are low priority
            created_at: std::time::Instant::now(),
            estimated_size: 0, // Unknown size
        };

        let key = format!("{}:{}:{}", peer_id, namespace, doc_id);

        self.pending_tasks.write().insert(
            key.clone(),
            SyncTaskState {
                task,
                retry_count: 0,
                last_attempt: None,
            },
        );

        debug!("Added document to background sync: {}", key);
    }

    /// Remove a document from background sync.
    pub fn remove_document(&self, peer_id: &PeerId, namespace: &str, doc_id: &str) {
        let key = format!("{}:{}:{}", peer_id, namespace, doc_id);
        self.pending_tasks.write().remove(&key);
        debug!("Removed document from background sync: {}", key);
    }

    /// Get number of pending tasks.
    pub fn pending_count(&self) -> usize {
        self.pending_tasks.read().len()
    }

    /// Spawn background sync task (native platforms).
    #[cfg(not(target_arch = "wasm32"))]
    fn spawn_task(&self) {
        let is_running = self.is_running.clone();
        let sync_interval = self.config.sync_interval;
        let pending_tasks = self.pending_tasks.clone();
        let bandwidth_manager = self.bandwidth_manager.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            info!("Background sync task started");

            while is_running.load(Ordering::SeqCst) {
                // Process pending tasks
                let tasks: Vec<(String, SyncTaskState)> = {
                    let pending = pending_tasks.read();
                    pending.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
                };

                for (key, mut state) in tasks {
                    // Check if we should retry this task
                    if let Some(last_attempt) = state.last_attempt {
                        let backoff = if config.exponential_backoff {
                            config.retry_backoff * 2u32.pow(state.retry_count)
                        } else {
                            config.retry_backoff
                        };

                        if last_attempt.elapsed() < backoff {
                            continue; // Too soon to retry
                        }
                    }

                    // Check retry limit
                    if state.retry_count >= config.max_retries {
                        warn!("Max retries reached for task: {}", key);
                        pending_tasks.write().remove(&key);
                        continue;
                    }

                    // Schedule task
                    debug!("Scheduling background sync task: {}", key);

                    match bandwidth_manager.schedule_sync(state.task.clone()).await {
                        Ok(_) => {
                            // Update state
                            state.last_attempt = Some(std::time::Instant::now());
                            state.retry_count += 1;
                            pending_tasks.write().insert(key.clone(), state);
                        }
                        Err(e) => {
                            warn!("Failed to schedule task {}: {}", key, e);
                        }
                    }
                }

                // Wait for next sync interval
                tokio::time::sleep(sync_interval).await;
            }

            info!("Background sync task stopped");
        });
    }

    /// Spawn Web Worker for background sync (browser).
    #[cfg(target_arch = "wasm32")]
    fn spawn_worker(&self) {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::spawn_local;
        use web_sys::Worker;

        let is_running = self.is_running.clone();
        let sync_interval = self.config.sync_interval;
        let pending_tasks = self.pending_tasks.clone();
        let bandwidth_manager = self.bandwidth_manager.clone();
        let config = self.config.clone();

        spawn_local(async move {
            info!("Background sync worker started (WASM)");

            // Note: In a real implementation, you'd create a Web Worker
            // For now, we'll use a simple async loop

            while is_running.load(Ordering::SeqCst) {
                // Process pending tasks (same logic as native)
                let tasks: Vec<(String, SyncTaskState)> = {
                    let pending = pending_tasks.read();
                    pending.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
                };

                for (key, mut state) in tasks {
                    if let Some(last_attempt) = state.last_attempt {
                        let backoff = if config.exponential_backoff {
                            config.retry_backoff * 2u32.pow(state.retry_count)
                        } else {
                            config.retry_backoff
                        };

                        if last_attempt.elapsed() < backoff {
                            continue;
                        }
                    }

                    if state.retry_count >= config.max_retries {
                        warn!("Max retries reached for task: {}", key);
                        pending_tasks.write().remove(&key);
                        continue;
                    }

                    debug!("Scheduling background sync task: {}", key);

                    match bandwidth_manager.schedule_sync(state.task.clone()).await {
                        Ok(_) => {
                            state.last_attempt = Some(std::time::Instant::now());
                            state.retry_count += 1;
                            pending_tasks.write().insert(key.clone(), state);
                        }
                        Err(e) => {
                            warn!("Failed to schedule task {}: {}", key, e);
                        }
                    }
                }

                // Sleep (using WASM-compatible sleep)
                let millis = sync_interval.as_millis() as i32;
                gloo_timers::future::sleep(Duration::from_millis(millis as u64)).await;
            }

            info!("Background sync worker stopped (WASM)");
        });
    }

    /// Manually trigger sync for a specific document.
    pub async fn sync_now(&self, peer_id: &PeerId, namespace: &str, doc_id: &str) -> Result<()> {
        let key = format!("{}:{}:{}", peer_id, namespace, doc_id);

        let task_state = self
            .pending_tasks
            .read()
            .get(&key)
            .cloned()
            .ok_or_else(|| P2PError::Internal(format!("Task not found: {}", key)))?;

        // Schedule with high priority for immediate execution
        let mut task = task_state.task;
        task.priority = SyncPriority::High;

        self.bandwidth_manager.schedule_sync(task).await?;

        Ok(())
    }

    /// Clear all pending tasks.
    pub fn clear(&self) {
        self.pending_tasks.write().clear();
        info!("Cleared all background sync tasks");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_background_sync_creation() {
        let config = BackgroundSyncConfig::default();
        let bandwidth_manager = Arc::new(BandwidthManager::new());
        let sync = BackgroundSync::new(config, bandwidth_manager);

        assert!(!sync.is_running.load(Ordering::SeqCst));
        assert_eq!(sync.pending_count(), 0);
    }

    #[test]
    fn test_add_remove_document() {
        let config = BackgroundSyncConfig::default();
        let bandwidth_manager = Arc::new(BandwidthManager::new());
        let sync = BackgroundSync::new(config, bandwidth_manager);

        sync.add_document("peer1".to_string(), "users".to_string(), "alice".to_string());
        assert_eq!(sync.pending_count(), 1);

        sync.remove_document(&"peer1".to_string(), "users", "alice");
        assert_eq!(sync.pending_count(), 0);
    }

    #[tokio::test]
    async fn test_start_stop() {
        let config = BackgroundSyncConfig::default();
        let bandwidth_manager = Arc::new(BandwidthManager::new());
        let sync = BackgroundSync::new(config, bandwidth_manager);

        sync.start();
        assert!(sync.is_running.load(Ordering::SeqCst));

        sync.stop();
        assert!(!sync.is_running.load(Ordering::SeqCst));
    }

    #[test]
    fn test_clear() {
        let config = BackgroundSyncConfig::default();
        let bandwidth_manager = Arc::new(BandwidthManager::new());
        let sync = BackgroundSync::new(config, bandwidth_manager);

        sync.add_document("peer1".to_string(), "users".to_string(), "alice".to_string());
        sync.add_document("peer2".to_string(), "posts".to_string(), "post1".to_string());
        assert_eq!(sync.pending_count(), 2);

        sync.clear();
        assert_eq!(sync.pending_count(), 0);
    }

    #[tokio::test]
    async fn test_sync_now() {
        let config = BackgroundSyncConfig::default();
        let bandwidth_manager = Arc::new(BandwidthManager::new());
        let sync = BackgroundSync::new(config, bandwidth_manager);

        sync.add_document("peer1".to_string(), "users".to_string(), "alice".to_string());

        let result = sync.sync_now(&"peer1".to_string(), "users", "alice").await;
        assert!(result.is_ok());
    }
}
