use serde::{Deserialize, Serialize};
use std::time::Duration;

/// ConnectionMetrics tracks P2P connection performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMetrics {
    pub total_connections: u64,
    pub active_connections: u64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub avg_connection_time_ms: f64,
    pub avg_sync_latency_ms: f64,
    pub reconnection_count: u64,
    pub avg_reconnection_time_ms: f64,
}

impl ConnectionMetrics {
    pub fn new() -> Self {
        Self {
            total_connections: 0,
            active_connections: 0,
            total_bytes_sent: 0,
            total_bytes_received: 0,
            avg_connection_time_ms: 0.0,
            avg_sync_latency_ms: 0.0,
            reconnection_count: 0,
            avg_reconnection_time_ms: 0.0,
        }
    }

    pub fn record_connection_time(&mut self, duration: Duration) {
        self.total_connections += 1;
        self.active_connections += 1;

        let new_time_ms = duration.as_millis() as f64;
        self.avg_connection_time_ms = (self.avg_connection_time_ms
            * (self.total_connections - 1) as f64
            + new_time_ms)
            / self.total_connections as f64;
    }

    pub fn record_sync_latency(&mut self, duration: Duration) {
        let new_latency_ms = duration.as_millis() as f64;
        let count = self.total_bytes_received.max(1);

        self.avg_sync_latency_ms =
            (self.avg_sync_latency_ms * (count - 1) as f64 + new_latency_ms) / count as f64;
    }

    pub fn record_bytes_sent(&mut self, bytes: usize) {
        self.total_bytes_sent += bytes as u64;
    }

    pub fn record_bytes_received(&mut self, bytes: usize) {
        self.total_bytes_received += bytes as u64;
    }

    pub fn record_reconnection(&mut self, duration: Duration) {
        self.reconnection_count += 1;

        let new_time_ms = duration.as_millis() as f64;
        self.avg_reconnection_time_ms = (self.avg_reconnection_time_ms
            * (self.reconnection_count - 1) as f64
            + new_time_ms)
            / self.reconnection_count as f64;
    }

    pub fn record_disconnection(&mut self) {
        if self.active_connections > 0 {
            self.active_connections -= 1;
        }
    }

    /// Get throughput in bytes per second
    pub fn throughput_bps(&self) -> f64 {
        // Simple estimation based on sync latency
        if self.avg_sync_latency_ms > 0.0 {
            (self.total_bytes_sent + self.total_bytes_received) as f64
                / (self.avg_sync_latency_ms / 1000.0)
        } else {
            0.0
        }
    }

    /// Pretty print metrics
    pub fn print_summary(&self) {
        println!("=== Connection Metrics ===");
        println!("Total Connections:       {}", self.total_connections);
        println!("Active Connections:      {}", self.active_connections);
        println!("Bytes Sent:              {}", self.total_bytes_sent);
        println!("Bytes Received:          {}", self.total_bytes_received);
        println!(
            "Avg Connection Time:     {:.2}ms",
            self.avg_connection_time_ms
        );
        println!("Avg Sync Latency:        {:.2}ms", self.avg_sync_latency_ms);
        println!("Reconnection Count:      {}", self.reconnection_count);
        println!(
            "Avg Reconnection Time:   {:.2}ms",
            self.avg_reconnection_time_ms
        );
        println!("Throughput:              {:.2} bytes/s", self.throughput_bps());
        println!("==========================");
    }

    /// Check if metrics meet acceptance criteria
    pub fn meets_acceptance_criteria(&self) -> TestResult {
        let mut result = TestResult {
            passed: true,
            failures: Vec::new(),
        };

        // Connection establishment < 3 seconds (direct)
        if self.avg_connection_time_ms > 3000.0 {
            result.passed = false;
            result.failures.push(format!(
                "Connection time {}ms exceeds 3000ms threshold",
                self.avg_connection_time_ms
            ));
        }

        // Reconnection < 5 seconds
        if self.reconnection_count > 0 && self.avg_reconnection_time_ms > 5000.0 {
            result.passed = false;
            result.failures.push(format!(
                "Reconnection time {}ms exceeds 5000ms threshold",
                self.avg_reconnection_time_ms
            ));
        }

        result
    }
}

impl Default for ConnectionMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct TestResult {
    pub passed: bool,
    pub failures: Vec<String>,
}

impl TestResult {
    pub fn print(&self) {
        if self.passed {
            println!("✓ All acceptance criteria met");
        } else {
            println!("✗ Test failed:");
            for failure in &self.failures {
                println!("  - {}", failure);
            }
        }
    }
}
