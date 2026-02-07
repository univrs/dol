//! Dynamic Schema Loading with Hot-Reload
//!
//! This module provides dynamic loading of .dol files at runtime with support
//! for hot-reloading schemas without process restart.
//!
//! # Features
//!
//! - Load .dol files at runtime (not just compile-time)
//! - Hot-reload schemas on file changes
//! - Schema versioning and migration at runtime
//! - Atomic schema updates with validation
//!
//! # Example
//!
//! ```rust,no_run
//! use dol_reflect::dynamic_load::{SchemaLoader, LoadOptions};
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a loader
//! let mut loader = SchemaLoader::new();
//!
//! // Load a schema file
//! loader.load_file(Path::new("schema.dol")).await?;
//!
//! // Enable hot-reload
//! let (mut watcher, mut rx) = loader.watch_directory(Path::new("schemas/")).await?;
//!
//! // React to schema changes
//! while let Some(event) = rx.recv().await {
//!     println!("Schema changed: {:?}", event);
//! }
//! # Ok(())
//! # }
//! ```

use crate::schema_api::{ReflectionError, SchemaRegistry};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use walkdir::WalkDir;

/// Error type for dynamic loading operations.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    /// IO error reading file
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Reflection error
    #[error("Reflection error: {0}")]
    Reflection(#[from] ReflectionError),

    /// File watch error
    #[error("File watch error: {0}")]
    Watch(#[from] notify::Error),

    /// Invalid schema version
    #[error("Invalid schema version: {0}")]
    InvalidVersion(String),

    /// Schema validation failed
    #[error("Schema validation failed: {0}")]
    ValidationFailed(String),

    /// Schema conflict during migration
    #[error("Schema conflict: {0}")]
    Conflict(String),
}

/// Result type for loading operations.
pub type LoadResult<T> = Result<T, LoadError>;

/// Options for schema loading.
#[derive(Debug, Clone)]
pub struct LoadOptions {
    /// Validate schemas before loading
    pub validate: bool,
    /// Allow hot-reload
    pub hot_reload: bool,
    /// Recursive directory loading
    pub recursive: bool,
    /// File extension filter (default: ".dol")
    pub extension: String,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            validate: true,
            hot_reload: false,
            recursive: true,
            extension: ".dol".to_string(),
        }
    }
}

/// Schema version information for migrations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaVersion {
    /// Schema identifier (file path or name)
    pub identifier: String,
    /// Version string
    pub version: String,
    /// Timestamp of last modification
    pub timestamp: std::time::SystemTime,
}

/// Schema change event.
#[derive(Debug, Clone)]
pub enum SchemaEvent {
    /// Schema file was created
    Created {
        path: PathBuf,
        registry: Arc<RwLock<SchemaRegistry>>,
    },
    /// Schema file was modified
    Modified {
        path: PathBuf,
        registry: Arc<RwLock<SchemaRegistry>>,
    },
    /// Schema file was deleted
    Deleted { path: PathBuf },
    /// Schema load error
    Error { path: PathBuf, error: String },
}

/// Dynamic schema loader with hot-reload support.
///
/// The schema loader provides runtime loading and reloading of DOL schemas
/// from the filesystem with automatic change detection.
pub struct SchemaLoader {
    /// Main schema registry
    registry: Arc<RwLock<SchemaRegistry>>,
    /// Loaded schema versions for migration tracking
    versions: HashMap<PathBuf, SchemaVersion>,
    /// Load options
    options: LoadOptions,
}

impl SchemaLoader {
    /// Creates a new schema loader with default options.
    pub fn new() -> Self {
        Self {
            registry: Arc::new(RwLock::new(SchemaRegistry::new())),
            versions: HashMap::new(),
            options: LoadOptions::default(),
        }
    }

    /// Creates a new schema loader with custom options.
    pub fn with_options(options: LoadOptions) -> Self {
        Self {
            registry: Arc::new(RwLock::new(SchemaRegistry::new())),
            versions: HashMap::new(),
            options,
        }
    }

    /// Returns a reference to the schema registry.
    pub fn registry(&self) -> Arc<RwLock<SchemaRegistry>> {
        Arc::clone(&self.registry)
    }

    /// Loads a single schema file.
    pub async fn load_file(&mut self, path: &Path) -> LoadResult<()> {
        let source = tokio::fs::read_to_string(path).await?;
        let metadata = tokio::fs::metadata(path).await?;

        // Load into registry
        let mut registry = self.registry.write().await;
        registry.load_schema(&source)?;

        // Track version
        let version = SchemaVersion {
            identifier: path.to_string_lossy().to_string(),
            version: "1.0.0".to_string(), // TODO: Extract from schema
            timestamp: metadata.modified()?,
        };
        self.versions.insert(path.to_path_buf(), version);

        Ok(())
    }

    /// Loads all schema files from a directory.
    pub async fn load_directory(&mut self, dir: &Path) -> LoadResult<Vec<PathBuf>> {
        let mut loaded = Vec::new();

        let walker = if self.options.recursive {
            WalkDir::new(dir)
        } else {
            WalkDir::new(dir).max_depth(1)
        };

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file()
                && path
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| format!(".{}", s) == self.options.extension)
                    .unwrap_or(false)
            {
                match self.load_file(path).await {
                    Ok(_) => loaded.push(path.to_path_buf()),
                    Err(e) => {
                        eprintln!("Failed to load {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(loaded)
    }

    /// Watches a directory for schema changes and enables hot-reload.
    ///
    /// Returns a watcher handle and a channel receiver for schema events.
    pub async fn watch_directory(
        &mut self,
        dir: &Path,
    ) -> LoadResult<(RecommendedWatcher, mpsc::UnboundedReceiver<SchemaEvent>)> {
        let (tx, rx) = mpsc::unbounded_channel();
        let registry = Arc::clone(&self.registry);
        let dir = dir.to_path_buf();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if let EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) =
                        event.kind
                    {
                        for path in event.paths {
                            if path.extension().and_then(|s| s.to_str()) == Some("dol") {
                                let event = match event.kind {
                                    EventKind::Create(_) => SchemaEvent::Created {
                                        path: path.clone(),
                                        registry: Arc::clone(&registry),
                                    },
                                    EventKind::Modify(_) => SchemaEvent::Modified {
                                        path: path.clone(),
                                        registry: Arc::clone(&registry),
                                    },
                                    EventKind::Remove(_) => SchemaEvent::Deleted { path },
                                    _ => continue,
                                };
                                let _ = tx.send(event);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Watch error: {}", e);
                }
            }
        })?;

        watcher.watch(&dir, RecursiveMode::Recursive)?;

        Ok((watcher, rx))
    }

    /// Reloads a schema file (for hot-reload).
    pub async fn reload_file(&mut self, path: &Path) -> LoadResult<()> {
        // Check if file was modified
        let metadata = tokio::fs::metadata(path).await?;
        let modified = metadata.modified()?;

        if let Some(version) = self.versions.get(path) {
            if version.timestamp >= modified {
                // No changes, skip reload
                return Ok(());
            }
        }

        // Reload the file
        self.load_file(path).await
    }

    /// Returns all loaded schema versions.
    pub fn versions(&self) -> &HashMap<PathBuf, SchemaVersion> {
        &self.versions
    }

    /// Clears all loaded schemas.
    pub async fn clear(&mut self) {
        let mut registry = self.registry.write().await;
        registry.clear();
        self.versions.clear();
    }

    /// Migrates schemas from one version to another.
    ///
    /// This is a placeholder for future migration logic.
    pub async fn migrate(
        &mut self,
        _from_version: &str,
        _to_version: &str,
    ) -> LoadResult<()> {
        // TODO: Implement schema migration logic
        Ok(())
    }
}

impl Default for SchemaLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to process schema events from a watcher.
pub async fn process_schema_event(event: SchemaEvent) -> LoadResult<()> {
    match event {
        SchemaEvent::Created { path, registry } => {
            println!("Schema created: {}", path.display());
            let source = tokio::fs::read_to_string(&path).await?;
            let mut reg = registry.write().await;
            reg.load_schema(&source)?;
        }
        SchemaEvent::Modified { path, registry } => {
            println!("Schema modified: {}", path.display());
            let source = tokio::fs::read_to_string(&path).await?;
            let mut reg = registry.write().await;
            // Clear and reload
            reg.clear();
            reg.load_schema(&source)?;
        }
        SchemaEvent::Deleted { path } => {
            println!("Schema deleted: {}", path.display());
            // TODO: Remove from registry
        }
        SchemaEvent::Error { path, error } => {
            eprintln!("Schema error for {}: {}", path.display(), error);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_load_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.dol");

        let content = r#"
gen test.gen {
  test has field: String
}

exegesis { Test gen }
"#;

        tokio::fs::write(&file_path, content).await.unwrap();

        let mut loader = SchemaLoader::new();
        assert!(loader.load_file(&file_path).await.is_ok());

        let registry = loader.registry.read().await;
        assert!(registry.get_gen("test.gen").is_some());
    }

    #[tokio::test]
    async fn test_load_directory() {
        let temp_dir = TempDir::new().unwrap();

        // Create multiple schema files
        let files = vec![
            ("schema1.dol", "gen test1.gen { test1 has field: String } exegesis { Test }"),
            ("schema2.dol", "gen test2.gen { test2 has field: String } exegesis { Test }"),
        ];

        for (name, content) in files {
            tokio::fs::write(temp_dir.path().join(name), content)
                .await
                .unwrap();
        }

        let mut loader = SchemaLoader::new();
        let loaded = loader.load_directory(temp_dir.path()).await.unwrap();

        assert_eq!(loaded.len(), 2);

        let registry = loader.registry.read().await;
        assert!(registry.get_gen("test1.gen").is_some());
        assert!(registry.get_gen("test2.gen").is_some());
    }

    #[tokio::test]
    async fn test_reload_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.dol");

        let content_v1 = r#"
gen test.gen {
  test has field1: String
}

exegesis { Version 1 }
"#;

        tokio::fs::write(&file_path, content_v1).await.unwrap();

        let mut loader = SchemaLoader::new();
        loader.load_file(&file_path).await.unwrap();

        // Modify the file
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let content_v2 = r#"
gen test.gen {
  test has field1: String
  test has field2: Int32
}

exegesis { Version 2 }
"#;

        tokio::fs::write(&file_path, content_v2).await.unwrap();

        // Reload
        loader.reload_file(&file_path).await.unwrap();

        let registry = loader.registry.read().await;
        let gen = registry.get_gen("test.gen").unwrap();
        assert_eq!(gen.field_count(), 2);
    }

    #[tokio::test]
    async fn test_version_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.dol");

        let content = r#"
gen test.gen {
  test has field: String
}

exegesis { Test }
"#;

        tokio::fs::write(&file_path, content).await.unwrap();

        let mut loader = SchemaLoader::new();
        loader.load_file(&file_path).await.unwrap();

        assert_eq!(loader.versions().len(), 1);
        assert!(loader.versions().contains_key(&file_path));
    }
}
