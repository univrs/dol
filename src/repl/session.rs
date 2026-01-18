//! REPL Session - Persistent state and configuration
//!
//! Manages REPL sessions including:
//! - Session configuration
//! - Save/restore state
//! - Session history

use std::path::PathBuf;

/// Configuration for a REPL session.
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Session name
    pub name: String,

    /// Enable tree shaking
    pub tree_shaking: bool,

    /// Enable optimization
    pub optimize: bool,

    /// Include debug info
    pub debug_info: bool,

    /// Auto-save session
    pub auto_save: bool,

    /// Session file path (for persistence)
    pub session_file: Option<PathBuf>,

    /// Maximum history entries
    pub max_history: usize,

    /// Verbose output
    pub verbose: bool,

    /// Output format
    pub output_format: OutputFormat,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            tree_shaking: true,
            optimize: false,
            debug_info: true,
            auto_save: false,
            session_file: None,
            max_history: 1000,
            verbose: false,
            output_format: OutputFormat::Pretty,
        }
    }
}

impl SessionConfig {
    /// Create a new session config with a name.
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }

    /// Enable tree shaking.
    pub fn with_tree_shaking(mut self, enable: bool) -> Self {
        self.tree_shaking = enable;
        self
    }

    /// Enable optimization.
    pub fn with_optimization(mut self, enable: bool) -> Self {
        self.optimize = enable;
        self
    }

    /// Enable debug info.
    pub fn with_debug_info(mut self, enable: bool) -> Self {
        self.debug_info = enable;
        self
    }

    /// Set session file for persistence.
    pub fn with_session_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.session_file = Some(path.into());
        self.auto_save = true;
        self
    }

    /// Enable verbose output.
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }
}

/// Output format for REPL results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Pretty-printed output
    Pretty,
    /// JSON output
    Json,
    /// Minimal output
    Minimal,
}

/// A REPL session with persistent state.
#[derive(Debug)]
pub struct ReplSession {
    /// Session configuration
    config: SessionConfig,

    /// Session start time
    started_at: std::time::Instant,

    /// Total evaluations in this session
    eval_count: usize,

    /// Successful evaluations
    success_count: usize,

    /// Error count
    error_count: usize,
}

impl ReplSession {
    /// Create a new session with default config.
    pub fn new() -> Self {
        Self::with_config(SessionConfig::default())
    }

    /// Create a session with custom config.
    pub fn with_config(config: SessionConfig) -> Self {
        Self {
            config,
            started_at: std::time::Instant::now(),
            eval_count: 0,
            success_count: 0,
            error_count: 0,
        }
    }

    /// Get session configuration.
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }

    /// Get mutable session configuration.
    pub fn config_mut(&mut self) -> &mut SessionConfig {
        &mut self.config
    }

    /// Record a successful evaluation.
    pub fn record_success(&mut self) {
        self.eval_count += 1;
        self.success_count += 1;
    }

    /// Record an error.
    pub fn record_error(&mut self) {
        self.eval_count += 1;
        self.error_count += 1;
    }

    /// Get session duration.
    pub fn duration(&self) -> std::time::Duration {
        self.started_at.elapsed()
    }

    /// Get evaluation count.
    pub fn eval_count(&self) -> usize {
        self.eval_count
    }

    /// Get success count.
    pub fn success_count(&self) -> usize {
        self.success_count
    }

    /// Get error count.
    pub fn error_count(&self) -> usize {
        self.error_count
    }

    /// Get success rate.
    pub fn success_rate(&self) -> f64 {
        if self.eval_count == 0 {
            1.0
        } else {
            self.success_count as f64 / self.eval_count as f64
        }
    }

    /// Get session summary.
    pub fn summary(&self) -> SessionSummary {
        SessionSummary {
            name: self.config.name.clone(),
            duration: self.duration(),
            eval_count: self.eval_count,
            success_count: self.success_count,
            error_count: self.error_count,
            success_rate: self.success_rate(),
        }
    }

    /// Save session to file.
    #[cfg(feature = "serde")]
    pub fn save(&self) -> Result<(), SessionError> {
        let path = self
            .config
            .session_file
            .as_ref()
            .ok_or(SessionError::NoSessionFile)?;

        let data = serde_json::json!({
            "name": self.config.name,
            "eval_count": self.eval_count,
            "success_count": self.success_count,
            "error_count": self.error_count,
        });

        std::fs::write(path, serde_json::to_string_pretty(&data).unwrap())
            .map_err(|e| SessionError::Io(e.to_string()))?;

        Ok(())
    }

    /// Save session to file (stub when serde is not available).
    #[cfg(not(feature = "serde"))]
    pub fn save(&self) -> Result<(), SessionError> {
        let path = self
            .config
            .session_file
            .as_ref()
            .ok_or(SessionError::NoSessionFile)?;

        // Simple text format when serde is not available
        let content = format!(
            "name={}\neval_count={}\nsuccess_count={}\nerror_count={}\n",
            self.config.name, self.eval_count, self.success_count, self.error_count
        );

        std::fs::write(path, content).map_err(|e| SessionError::Io(e.to_string()))?;

        Ok(())
    }

    /// Load session from file.
    #[cfg(feature = "serde")]
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, SessionError> {
        let path = path.into();
        let content =
            std::fs::read_to_string(&path).map_err(|e| SessionError::Io(e.to_string()))?;

        let data: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| SessionError::Parse(e.to_string()))?;

        let name = data["name"].as_str().unwrap_or("loaded").to_string();
        let eval_count = data["eval_count"].as_u64().unwrap_or(0) as usize;
        let success_count = data["success_count"].as_u64().unwrap_or(0) as usize;
        let error_count = data["error_count"].as_u64().unwrap_or(0) as usize;

        let config = SessionConfig::with_name(name).with_session_file(path);

        Ok(Self {
            config,
            started_at: std::time::Instant::now(),
            eval_count,
            success_count,
            error_count,
        })
    }

    /// Load session from file (simple parser when serde is not available).
    #[cfg(not(feature = "serde"))]
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, SessionError> {
        let path = path.into();
        let content =
            std::fs::read_to_string(&path).map_err(|e| SessionError::Io(e.to_string()))?;

        let mut name = "loaded".to_string();
        let mut eval_count = 0usize;
        let mut success_count = 0usize;
        let mut error_count = 0usize;

        for line in content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "name" => name = value.trim().to_string(),
                    "eval_count" => eval_count = value.trim().parse().unwrap_or(0),
                    "success_count" => success_count = value.trim().parse().unwrap_or(0),
                    "error_count" => error_count = value.trim().parse().unwrap_or(0),
                    _ => {}
                }
            }
        }

        let config = SessionConfig::with_name(name).with_session_file(path);

        Ok(Self {
            config,
            started_at: std::time::Instant::now(),
            eval_count,
            success_count,
            error_count,
        })
    }
}

impl Default for ReplSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Session summary statistics.
#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub name: String,
    pub duration: std::time::Duration,
    pub eval_count: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub success_rate: f64,
}

impl std::fmt::Display for SessionSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Session: {}", self.name)?;
        writeln!(f, "  Duration: {:.2}s", self.duration.as_secs_f64())?;
        writeln!(f, "  Evaluations: {}", self.eval_count)?;
        writeln!(f, "  Successes: {}", self.success_count)?;
        writeln!(f, "  Errors: {}", self.error_count)?;
        writeln!(f, "  Success rate: {:.1}%", self.success_rate * 100.0)?;
        Ok(())
    }
}

/// Session error types.
#[derive(Debug, Clone)]
pub enum SessionError {
    /// No session file configured
    NoSessionFile,
    /// I/O error
    Io(String),
    /// Parse error
    Parse(String),
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::NoSessionFile => write!(f, "No session file configured"),
            SessionError::Io(msg) => write!(f, "I/O error: {}", msg),
            SessionError::Parse(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for SessionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_new() {
        let session = ReplSession::new();
        assert_eq!(session.config.name, "default");
        assert_eq!(session.eval_count(), 0);
    }

    #[test]
    fn test_session_record_success() {
        let mut session = ReplSession::new();
        session.record_success();
        session.record_success();

        assert_eq!(session.eval_count(), 2);
        assert_eq!(session.success_count(), 2);
        assert_eq!(session.error_count(), 0);
        assert_eq!(session.success_rate(), 1.0);
    }

    #[test]
    fn test_session_record_error() {
        let mut session = ReplSession::new();
        session.record_success();
        session.record_error();

        assert_eq!(session.eval_count(), 2);
        assert_eq!(session.success_count(), 1);
        assert_eq!(session.error_count(), 1);
        assert_eq!(session.success_rate(), 0.5);
    }

    #[test]
    fn test_session_config_builder() {
        let config = SessionConfig::with_name("test")
            .with_tree_shaking(true)
            .with_optimization(true)
            .verbose();

        assert_eq!(config.name, "test");
        assert!(config.tree_shaking);
        assert!(config.optimize);
        assert!(config.verbose);
    }

    #[test]
    fn test_session_summary() {
        let session = ReplSession::new();
        let summary = session.summary();

        assert_eq!(summary.name, "default");
        assert_eq!(summary.eval_count, 0);
    }
}
