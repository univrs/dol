//! SQLite-based storage adapter implementation.

use async_trait::async_trait;
use bytes::Bytes;
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::task;
use vudo_storage::{Operation, QueryFilter, Result, StorageAdapter, StorageError, StorageStats};

/// SQLite storage adapter.
///
/// Uses SQLite with WAL mode for high-performance concurrent access.
/// Internally uses a connection pool with multiple readers and a single writer.
pub struct SqliteAdapter {
    /// Database file path.
    path: PathBuf,
    /// Shared connection for reads and writes (protected by mutex).
    /// We use a single connection with WAL mode which allows concurrent reads.
    connection: Arc<Mutex<Connection>>,
}

impl SqliteAdapter {
    /// Get the database file path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Create a new SQLite adapter.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the SQLite database file
    pub async fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let path_clone = path.clone();

        // Open connection in a blocking task
        let connection = task::spawn_blocking(move || {
            let conn = Connection::open(&path_clone)
                .map_err(|e| StorageError::Database(e.to_string()))?;

            // Enable WAL mode for better concurrency
            conn.pragma_update(None, "journal_mode", "WAL")
                .map_err(|e| StorageError::Database(e.to_string()))?;

            // Enable foreign keys
            conn.pragma_update(None, "foreign_keys", "ON")
                .map_err(|e| StorageError::Database(e.to_string()))?;

            // Optimize for performance
            conn.pragma_update(None, "synchronous", "NORMAL")
                .map_err(|e| StorageError::Database(e.to_string()))?;

            // Increase cache size (10MB)
            conn.pragma_update(None, "cache_size", -10000)
                .map_err(|e| StorageError::Database(e.to_string()))?;

            Ok::<_, StorageError>(conn)
        })
        .await
        .map_err(|e| StorageError::Internal(format!("Task join error: {}", e)))??;

        Ok(Self {
            path,
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    /// Create an in-memory SQLite adapter (for testing).
    pub async fn in_memory() -> Result<Self> {
        let connection = task::spawn_blocking(|| {
            let conn = Connection::open_in_memory()
                .map_err(|e| StorageError::Database(e.to_string()))?;

            conn.pragma_update(None, "foreign_keys", "ON")
                .map_err(|e| StorageError::Database(e.to_string()))?;

            Ok::<_, StorageError>(conn)
        })
        .await
        .map_err(|e| StorageError::Internal(format!("Task join error: {}", e)))??;

        Ok(Self {
            path: PathBuf::from(":memory:"),
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    /// Execute a query in a blocking task.
    async fn execute<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let conn = Arc::clone(&self.connection);
        task::spawn_blocking(move || {
            let conn = conn.lock();
            f(&conn)
        })
        .await
        .map_err(|e| StorageError::Internal(format!("Task join error: {}", e)))?
    }
}

#[async_trait]
impl StorageAdapter for SqliteAdapter {
    async fn init(&self) -> Result<()> {
        self.execute(|conn| {
            // Documents table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS documents (
                    namespace TEXT NOT NULL,
                    id TEXT NOT NULL,
                    data BLOB NOT NULL,
                    updated_at INTEGER NOT NULL,
                    PRIMARY KEY (namespace, id)
                )",
                [],
            )
            .map_err(|e| StorageError::Database(e.to_string()))?;

            // Create index on updated_at for time-based queries
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_documents_updated
                 ON documents(namespace, updated_at)",
                [],
            )
            .map_err(|e| StorageError::Database(e.to_string()))?;

            // Operations table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS operations (
                    id INTEGER PRIMARY KEY,
                    data BLOB NOT NULL,
                    timestamp INTEGER NOT NULL
                )",
                [],
            )
            .map_err(|e| StorageError::Database(e.to_string()))?;

            // Create index on operations timestamp
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_operations_timestamp
                 ON operations(timestamp)",
                [],
            )
            .map_err(|e| StorageError::Database(e.to_string()))?;

            // Snapshots table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS snapshots (
                    namespace TEXT NOT NULL,
                    id TEXT NOT NULL,
                    version INTEGER NOT NULL,
                    data BLOB NOT NULL,
                    created_at INTEGER NOT NULL,
                    PRIMARY KEY (namespace, id, version)
                )",
                [],
            )
            .map_err(|e| StorageError::Database(e.to_string()))?;

            Ok(())
        })
        .await
    }

    async fn save(&self, namespace: &str, id: &str, data: Bytes) -> Result<()> {
        let namespace = namespace.to_string();
        let id = id.to_string();
        let data_vec = data.to_vec();

        self.execute(move |conn| {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64;

            conn.execute(
                "INSERT OR REPLACE INTO documents (namespace, id, data, updated_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![namespace, id, data_vec, timestamp],
            )
            .map_err(|e| StorageError::Database(e.to_string()))?;

            Ok(())
        })
        .await
    }

    async fn load(&self, namespace: &str, id: &str) -> Result<Option<Bytes>> {
        let namespace = namespace.to_string();
        let id = id.to_string();

        self.execute(move |conn| {
            let result: Option<Vec<u8>> = conn
                .query_row(
                    "SELECT data FROM documents WHERE namespace = ?1 AND id = ?2",
                    params![namespace, id],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|e| StorageError::Database(e.to_string()))?;

            Ok(result.map(Bytes::from))
        })
        .await
    }

    async fn delete(&self, namespace: &str, id: &str) -> Result<()> {
        let namespace = namespace.to_string();
        let id = id.to_string();

        self.execute(move |conn| {
            conn.execute(
                "DELETE FROM documents WHERE namespace = ?1 AND id = ?2",
                params![namespace, id],
            )
            .map_err(|e| StorageError::Database(e.to_string()))?;

            Ok(())
        })
        .await
    }

    async fn list(&self, namespace: &str) -> Result<Vec<String>> {
        let namespace = namespace.to_string();

        self.execute(move |conn| {
            let mut stmt = conn
                .prepare("SELECT id FROM documents WHERE namespace = ?1 ORDER BY id")
                .map_err(|e| StorageError::Database(e.to_string()))?;

            let ids = stmt
                .query_map(params![namespace], |row| row.get::<_, String>(0))
                .map_err(|e| StorageError::Database(e.to_string()))?
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|e| StorageError::Database(e.to_string()))?;

            Ok(ids)
        })
        .await
    }

    async fn save_operations(&self, ops: &[Operation]) -> Result<()> {
        let ops_json = serde_json::to_vec(ops)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        self.execute(move |conn| {
            // Clear existing operations
            conn.execute("DELETE FROM operations", [])
                .map_err(|e| StorageError::Database(e.to_string()))?;

            // Deserialize and insert each operation
            let operations: Vec<Operation> = serde_json::from_slice(&ops_json)
                .map_err(|e| StorageError::Serialization(e.to_string()))?;

            for op in operations {
                let op_json = serde_json::to_vec(&op)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;

                conn.execute(
                    "INSERT INTO operations (id, data, timestamp) VALUES (?1, ?2, ?3)",
                    params![op.id as i64, op_json, op.timestamp as i64],
                )
                .map_err(|e| StorageError::Database(e.to_string()))?;
            }

            Ok(())
        })
        .await
    }

    async fn load_operations(&self) -> Result<Vec<Operation>> {
        self.execute(|conn| {
            let mut stmt = conn
                .prepare("SELECT data FROM operations ORDER BY timestamp, id")
                .map_err(|e| StorageError::Database(e.to_string()))?;

            let ops = stmt
                .query_map([], |row| {
                    let data: Vec<u8> = row.get(0)?;
                    Ok(data)
                })
                .map_err(|e| StorageError::Database(e.to_string()))?
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|e| StorageError::Database(e.to_string()))?;

            let operations = ops
                .into_iter()
                .map(|data| {
                    serde_json::from_slice(&data)
                        .map_err(|e| StorageError::Serialization(e.to_string()))
                })
                .collect::<Result<Vec<_>>>()?;

            Ok(operations)
        })
        .await
    }

    async fn save_snapshot(
        &self,
        namespace: &str,
        id: &str,
        version: u64,
        data: Bytes,
    ) -> Result<()> {
        let namespace = namespace.to_string();
        let id = id.to_string();
        let data_vec = data.to_vec();

        self.execute(move |conn| {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64;

            conn.execute(
                "INSERT OR REPLACE INTO snapshots (namespace, id, version, data, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![namespace, id, version as i64, data_vec, timestamp],
            )
            .map_err(|e| StorageError::Database(e.to_string()))?;

            Ok(())
        })
        .await
    }

    async fn load_snapshot(&self, namespace: &str, id: &str) -> Result<Option<(u64, Bytes)>> {
        let namespace = namespace.to_string();
        let id = id.to_string();

        self.execute(move |conn| {
            let result: Option<(i64, Vec<u8>)> = conn
                .query_row(
                    "SELECT version, data FROM snapshots
                     WHERE namespace = ?1 AND id = ?2
                     ORDER BY version DESC LIMIT 1",
                    params![namespace, id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()
                .map_err(|e| StorageError::Database(e.to_string()))?;

            Ok(result.map(|(v, d)| (v as u64, Bytes::from(d))))
        })
        .await
    }

    async fn query(&self, namespace: &str, filter: QueryFilter) -> Result<Vec<(String, Bytes)>> {
        let namespace = namespace.to_string();

        self.execute(move |conn| {
            let (sql, params) = build_query_sql(&namespace, &filter)?;
            let mut stmt = conn
                .prepare(&sql)
                .map_err(|e| StorageError::Database(e.to_string()))?;

            let results = stmt
                .query_map(rusqlite::params_from_iter(params.iter()), |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
                })
                .map_err(|e| StorageError::Database(e.to_string()))?
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|e| StorageError::Database(e.to_string()))?;

            Ok(results
                .into_iter()
                .map(|(id, data)| (id, Bytes::from(data)))
                .collect())
        })
        .await
    }

    async fn stats(&self) -> Result<StorageStats> {
        self.execute(|conn| {
            let document_count: i64 = conn
                .query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))
                .map_err(|e| StorageError::Database(e.to_string()))?;

            let total_document_size: i64 = conn
                .query_row(
                    "SELECT COALESCE(SUM(LENGTH(data)), 0) FROM documents",
                    [],
                    |row| row.get(0),
                )
                .map_err(|e| StorageError::Database(e.to_string()))?;

            let operation_count: i64 = conn
                .query_row("SELECT COUNT(*) FROM operations", [], |row| row.get(0))
                .map_err(|e| StorageError::Database(e.to_string()))?;

            let snapshot_count: i64 = conn
                .query_row("SELECT COUNT(*) FROM snapshots", [], |row| row.get(0))
                .map_err(|e| StorageError::Database(e.to_string()))?;

            let total_snapshot_size: i64 = conn
                .query_row(
                    "SELECT COALESCE(SUM(LENGTH(data)), 0) FROM snapshots",
                    [],
                    |row| row.get(0),
                )
                .map_err(|e| StorageError::Database(e.to_string()))?;

            Ok(StorageStats {
                document_count: document_count as usize,
                total_document_size: total_document_size as usize,
                operation_count: operation_count as usize,
                snapshot_count: snapshot_count as usize,
                total_snapshot_size: total_snapshot_size as usize,
            })
        })
        .await
    }

    async fn clear(&self) -> Result<()> {
        self.execute(|conn| {
            conn.execute("DELETE FROM documents", [])
                .map_err(|e| StorageError::Database(e.to_string()))?;
            conn.execute("DELETE FROM operations", [])
                .map_err(|e| StorageError::Database(e.to_string()))?;
            conn.execute("DELETE FROM snapshots", [])
                .map_err(|e| StorageError::Database(e.to_string()))?;
            Ok(())
        })
        .await
    }
}

/// Build SQL query from filter.
fn build_query_sql(namespace: &str, filter: &QueryFilter) -> Result<(String, Vec<String>)> {
    match filter {
        QueryFilter::All => {
            let sql = "SELECT id, data FROM documents WHERE namespace = ?1 ORDER BY id"
                .to_string();
            Ok((sql, vec![namespace.to_string()]))
        }
        QueryFilter::UpdatedAfter(timestamp) => {
            let sql = "SELECT id, data FROM documents WHERE namespace = ?1 AND updated_at > ?2 ORDER BY updated_at".to_string();
            Ok((
                sql,
                vec![namespace.to_string(), timestamp.to_string()],
            ))
        }
        QueryFilter::UpdatedBefore(timestamp) => {
            let sql = "SELECT id, data FROM documents WHERE namespace = ?1 AND updated_at < ?2 ORDER BY updated_at".to_string();
            Ok((
                sql,
                vec![namespace.to_string(), timestamp.to_string()],
            ))
        }
        QueryFilter::UpdatedBetween { start, end } => {
            let sql = "SELECT id, data FROM documents WHERE namespace = ?1 AND updated_at >= ?2 AND updated_at <= ?3 ORDER BY updated_at".to_string();
            Ok((
                sql,
                vec![namespace.to_string(), start.to_string(), end.to_string()],
            ))
        }
        _ => Err(StorageError::Unsupported(
            "Complex query filters not yet implemented".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sqlite_adapter_new() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let adapter = SqliteAdapter::new(&db_path).await.unwrap();
        assert_eq!(adapter.path, db_path);
    }

    #[tokio::test]
    async fn test_sqlite_adapter_in_memory() {
        let adapter = SqliteAdapter::in_memory().await.unwrap();
        assert_eq!(adapter.path, PathBuf::from(":memory:"));
    }

    #[tokio::test]
    async fn test_sqlite_adapter_init() {
        let adapter = SqliteAdapter::in_memory().await.unwrap();
        adapter.init().await.unwrap();

        // Init should be idempotent
        adapter.init().await.unwrap();
    }

    #[tokio::test]
    async fn test_sqlite_adapter_save_and_load() {
        let adapter = SqliteAdapter::in_memory().await.unwrap();
        adapter.init().await.unwrap();

        let data = Bytes::from("test document");
        adapter.save("users", "alice", data.clone()).await.unwrap();

        let loaded = adapter.load("users", "alice").await.unwrap();
        assert_eq!(loaded, Some(data));
    }

    #[tokio::test]
    async fn test_sqlite_adapter_load_nonexistent() {
        let adapter = SqliteAdapter::in_memory().await.unwrap();
        adapter.init().await.unwrap();

        let loaded = adapter.load("users", "bob").await.unwrap();
        assert_eq!(loaded, None);
    }

    #[tokio::test]
    async fn test_sqlite_adapter_delete() {
        let adapter = SqliteAdapter::in_memory().await.unwrap();
        adapter.init().await.unwrap();

        let data = Bytes::from("test document");
        adapter.save("users", "alice", data).await.unwrap();

        adapter.delete("users", "alice").await.unwrap();

        let loaded = adapter.load("users", "alice").await.unwrap();
        assert_eq!(loaded, None);
    }

    #[tokio::test]
    async fn test_sqlite_adapter_list() {
        let adapter = SqliteAdapter::in_memory().await.unwrap();
        adapter.init().await.unwrap();

        adapter
            .save("users", "alice", Bytes::from("data1"))
            .await
            .unwrap();
        adapter
            .save("users", "bob", Bytes::from("data2"))
            .await
            .unwrap();
        adapter
            .save("posts", "post1", Bytes::from("data3"))
            .await
            .unwrap();

        let users = adapter.list("users").await.unwrap();
        assert_eq!(users, vec!["alice", "bob"]);

        let posts = adapter.list("posts").await.unwrap();
        assert_eq!(posts, vec!["post1"]);
    }

    #[tokio::test]
    async fn test_sqlite_adapter_operations() {
        let adapter = SqliteAdapter::in_memory().await.unwrap();
        adapter.init().await.unwrap();

        let ops = vec![
            Operation::new(1, "users", "alice", vudo_storage::operation::OperationType::Create),
            Operation::new(2, "users", "bob", vudo_storage::operation::OperationType::Create),
        ];

        adapter.save_operations(&ops).await.unwrap();

        let loaded_ops = adapter.load_operations().await.unwrap();
        assert_eq!(loaded_ops.len(), 2);
        assert_eq!(loaded_ops[0].id, 1);
        assert_eq!(loaded_ops[1].id, 2);
    }

    #[tokio::test]
    async fn test_sqlite_adapter_snapshots() {
        let adapter = SqliteAdapter::in_memory().await.unwrap();
        adapter.init().await.unwrap();

        let data1 = Bytes::from("snapshot v1");
        let data2 = Bytes::from("snapshot v2");

        adapter
            .save_snapshot("users", "alice", 1, data1.clone())
            .await
            .unwrap();
        adapter
            .save_snapshot("users", "alice", 2, data2.clone())
            .await
            .unwrap();

        let snapshot = adapter.load_snapshot("users", "alice").await.unwrap();
        assert_eq!(snapshot, Some((2, data2))); // Should get latest version
    }

    #[tokio::test]
    async fn test_sqlite_adapter_query_all() {
        let adapter = SqliteAdapter::in_memory().await.unwrap();
        adapter.init().await.unwrap();

        adapter
            .save("users", "alice", Bytes::from("data1"))
            .await
            .unwrap();
        adapter
            .save("users", "bob", Bytes::from("data2"))
            .await
            .unwrap();

        let results = adapter.query("users", QueryFilter::All).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_sqlite_adapter_query_updated_after() {
        let adapter = SqliteAdapter::in_memory().await.unwrap();
        adapter.init().await.unwrap();

        let before_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        adapter
            .save("users", "alice", Bytes::from("data1"))
            .await
            .unwrap();

        let results = adapter
            .query("users", QueryFilter::updated_after(before_timestamp))
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "alice");
    }

    #[tokio::test]
    async fn test_sqlite_adapter_stats() {
        let adapter = SqliteAdapter::in_memory().await.unwrap();
        adapter.init().await.unwrap();

        adapter
            .save("users", "alice", Bytes::from("test"))
            .await
            .unwrap();

        let stats = adapter.stats().await.unwrap();
        assert_eq!(stats.document_count, 1);
        assert!(stats.total_document_size > 0);
    }

    #[tokio::test]
    async fn test_sqlite_adapter_clear() {
        let adapter = SqliteAdapter::in_memory().await.unwrap();
        adapter.init().await.unwrap();

        adapter
            .save("users", "alice", Bytes::from("data"))
            .await
            .unwrap();

        adapter.clear().await.unwrap();

        let loaded = adapter.load("users", "alice").await.unwrap();
        assert_eq!(loaded, None);
    }

    #[tokio::test]
    async fn test_sqlite_adapter_concurrent_writes() {
        let adapter = Arc::new(SqliteAdapter::in_memory().await.unwrap());
        adapter.init().await.unwrap();

        let mut handles = vec![];

        for i in 0..10 {
            let adapter_clone = Arc::clone(&adapter);
            let handle = tokio::spawn(async move {
                let data = Bytes::from(format!("data{}", i));
                adapter_clone
                    .save("users", &format!("user{}", i), data)
                    .await
                    .unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let users = adapter.list("users").await.unwrap();
        assert_eq!(users.len(), 10);
    }
}
