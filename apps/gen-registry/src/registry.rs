//! Registry core implementation
//!
//! Manages module metadata, versioning, and CRDT state

use crate::{
    error::{Error, Result},
    models::{Dependency, GenModule, InstalledModule, ModuleVersion, Rating, SearchIndex},
    search::{SearchEngine, SearchQuery, SearchResult},
    sync::P2PSync,
    version::VersionResolver,
    wasm::{WasmModule, WasmValidator},
};
use automerge::{transaction::Transactable, Automerge, ObjType, ReadDoc, ROOT};
use dashmap::DashMap;
use parking_lot::RwLock;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::Arc,
};
use tracing::{debug, info, warn};
use vudo_state::StateEngine;

/// Registry configuration
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    pub registry_id: String,
    pub owner_did: String,
    pub data_dir: String,
    pub enable_p2p: bool,
    pub enable_search: bool,
    pub auto_sync: bool,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            registry_id: "gen.community".to_string(),
            owner_did: String::new(),
            data_dir: "./gen-registry-data".to_string(),
            enable_p2p: true,
            enable_search: true,
            auto_sync: true,
        }
    }
}

/// Gen Registry
pub struct Registry {
    config: RegistryConfig,
    state_engine: Arc<StateEngine>,
    modules: Arc<DashMap<String, GenModule>>,
    installed: Arc<DashMap<String, InstalledModule>>,
    ratings: Arc<DashMap<String, Vec<Rating>>>,
    search_engine: Option<Arc<SearchEngine>>,
    p2p_sync: Option<Arc<P2PSync>>,
    version_resolver: Arc<VersionResolver>,
    wasm_validator: Arc<WasmValidator>,
    doc: Arc<RwLock<Automerge>>,
}

impl Registry {
    /// Create new registry
    pub async fn new(owner_did: impl Into<String>) -> Result<Self> {
        let mut config = RegistryConfig::default();
        config.owner_did = owner_did.into();
        Self::with_config(config).await
    }

    /// Create registry with custom config
    pub async fn with_config(config: RegistryConfig) -> Result<Self> {
        info!("Initializing Gen Registry: {}", config.registry_id);

        // Create VUDO state engine
        let state_engine = Arc::new(
            StateEngine::new()
                .await
                .map_err(|e| Error::VudoStateError(e.to_string()))?,
        );

        // Initialize Automerge document
        let doc = Arc::new(RwLock::new(Automerge::new()));

        // Initialize modules map
        let modules = Arc::new(DashMap::new());
        let installed = Arc::new(DashMap::new());
        let ratings = Arc::new(DashMap::new());

        // Create search engine
        let search_engine = if config.enable_search {
            Some(Arc::new(SearchEngine::new(&config.data_dir).await?))
        } else {
            None
        };

        // Create P2P sync
        let p2p_sync = if config.enable_p2p {
            Some(Arc::new(
                P2PSync::new(Arc::clone(&state_engine), config.clone()).await?,
            ))
        } else {
            None
        };

        let version_resolver = Arc::new(VersionResolver::new());
        let wasm_validator = Arc::new(WasmValidator::new());

        Ok(Self {
            config,
            state_engine,
            modules,
            installed,
            ratings,
            search_engine,
            p2p_sync,
            version_resolver,
            wasm_validator,
            doc,
        })
    }

    /// Publish a new module version
    pub async fn publish(
        &self,
        mut module: GenModule,
        version: &str,
        wasm_path: &Path,
        changelog: &str,
    ) -> Result<()> {
        info!("Publishing {}@{}", module.id, version);

        // Validate module ID
        if !module.validate_id() {
            return Err(Error::InvalidModuleId(module.id.clone()));
        }

        // Validate WASM
        let wasm_module = WasmModule::from_file(wasm_path).await?;
        self.wasm_validator.validate(&wasm_module)?;

        // Create version
        let wasm_hash = wasm_module.hash();
        let signature = self.sign_module(&module.id, version, &wasm_hash).await?;

        let mut module_version = ModuleVersion::new(
            version,
            wasm_hash.clone(),
            wasm_module.size(),
            changelog,
            signature,
        );

        // Extract capabilities from WASM
        let capabilities = wasm_module.extract_capabilities()?;
        for cap in capabilities {
            module_version.add_capability(cap);
        }

        // Add version to module
        module.add_version(module_version);

        // Store in CRDT
        self.update_module_crdt(&module).await?;

        // Store in local cache
        self.modules.insert(module.id.clone(), module.clone());

        // Update search index
        if let Some(search) = &self.search_engine {
            let index = SearchIndex::new(&module);
            search.index_module(&index).await?;
        }

        // Sync to P2P network
        if let Some(sync) = &self.p2p_sync {
            if self.config.auto_sync {
                sync.sync_module(&module.id).await?;
            }
        }

        info!("Successfully published {}@{}", module.id, version);
        Ok(())
    }

    /// Search for modules
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        match &self.search_engine {
            Some(search) => {
                let query = SearchQuery::new(query);
                search.search(&query).await
            }
            None => Err(Error::SearchIndexError(
                "Search engine not enabled".to_string(),
            )),
        }
    }

    /// Search by tags
    pub async fn search_by_tags(&self, tags: &[&str]) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        for module in self.modules.iter() {
            if tags.iter().any(|t| module.tags.contains(*t)) {
                results.push(SearchResult {
                    module_id: module.id.clone(),
                    name: module.name.clone(),
                    description: module.description.clone(),
                    version: module.latest_version.clone(),
                    score: 1.0,
                });
            }
        }
        Ok(results)
    }

    /// Install a module
    pub async fn install(&self, module_id: &str, version: Option<&str>) -> Result<()> {
        info!("Installing {}", module_id);

        // Fetch module metadata
        let module = self.get_module(module_id).await?;

        // Resolve version
        let version_str = match version {
            Some(v) => v.to_string(),
            None => module.latest_version.clone(),
        };

        // Check if already installed
        if let Some(installed) = self.installed.get(module_id) {
            if installed.version == version_str {
                info!("{} already installed at {}", module_id, version_str);
                return Ok(());
            }
        }

        // Resolve dependencies
        let resolved = self
            .version_resolver
            .resolve_dependencies(&module, &version_str)
            .await?;

        // Install dependencies first
        for dep in resolved {
            if dep.module_id != module_id {
                self.install(&dep.module_id, Some(&dep.version)).await?;
            }
        }

        // Download WASM
        let wasm_module = self.download_wasm(module_id, &version_str).await?;

        // Validate WASM
        self.wasm_validator.validate(&wasm_module)?;

        // Store locally
        let installed = InstalledModule::new(module_id, version_str);
        self.installed.insert(module_id.to_string(), installed);

        info!("Successfully installed {}", module_id);
        Ok(())
    }

    /// Install with auto-update
    pub async fn install_with_auto_update(
        &self,
        module_id: &str,
        version_req: &str,
    ) -> Result<()> {
        self.install(module_id, None).await?;

        if let Some(mut installed) = self.installed.get_mut(module_id) {
            installed.auto_update = true;
        }

        Ok(())
    }

    /// Get module by ID
    pub async fn get_module(&self, module_id: &str) -> Result<GenModule> {
        // Check local cache
        if let Some(module) = self.modules.get(module_id) {
            return Ok(module.clone());
        }

        // Fetch from P2P network
        if let Some(sync) = &self.p2p_sync {
            let module = sync.fetch_module(module_id).await?;
            self.modules.insert(module_id.to_string(), module.clone());
            return Ok(module);
        }

        Err(Error::ModuleNotFound(module_id.to_string()))
    }

    /// Rate a module
    pub async fn rate(&self, module_id: &str, stars: u8, review: Option<&str>) -> Result<()> {
        let mut rating = Rating::new(module_id, &self.config.owner_did, stars);
        if let Some(r) = review {
            rating = rating.with_review(r);
        }

        // Store rating
        self.ratings
            .entry(module_id.to_string())
            .or_insert_with(Vec::new)
            .push(rating.clone());

        // Update CRDT
        self.update_rating_crdt(&rating).await?;

        // Sync to P2P
        if let Some(sync) = &self.p2p_sync {
            if self.config.auto_sync {
                sync.sync_ratings(module_id).await?;
            }
        }

        Ok(())
    }

    /// Get average rating for a module
    pub fn get_average_rating(&self, module_id: &str) -> Option<f64> {
        self.ratings.get(module_id).map(|ratings| {
            let sum: u32 = ratings.iter().map(|r| r.stars as u32).sum();
            sum as f64 / ratings.len() as f64
        })
    }

    /// List installed modules
    pub fn list_installed(&self) -> Vec<InstalledModule> {
        self.installed.iter().map(|e| e.value().clone()).collect()
    }

    /// Start P2P sync
    pub async fn start_sync(&self) -> Result<()> {
        if let Some(sync) = &self.p2p_sync {
            sync.start().await?;
            info!("P2P sync started");
        }
        Ok(())
    }

    /// Discover peers
    pub async fn discover_peers(&self) -> Result<Vec<String>> {
        if let Some(sync) = &self.p2p_sync {
            sync.discover_peers().await
        } else {
            Ok(Vec::new())
        }
    }

    /// Sync specific module
    pub async fn sync_module(&self, module_id: &str) -> Result<()> {
        if let Some(sync) = &self.p2p_sync {
            sync.sync_module(module_id).await
        } else {
            Err(Error::SyncError("P2P sync not enabled".to_string()))
        }
    }

    // Private methods

    async fn update_module_crdt(&self, module: &GenModule) -> Result<()> {
        let mut doc = self.doc.write();
        let mut tx = doc.transaction();

        // Store module in Automerge
        let module_obj = tx
            .put_object(ROOT, &module.id, ObjType::Map)
            .map_err(|e| Error::AutomergeError(e.to_string()))?;

        tx.put(&module_obj, "name", module.name.as_str())
            .map_err(|e| Error::AutomergeError(e.to_string()))?;
        tx.put(&module_obj, "description", module.description.as_str())
            .map_err(|e| Error::AutomergeError(e.to_string()))?;
        tx.put(&module_obj, "latest_version", module.latest_version.as_str())
            .map_err(|e| Error::AutomergeError(e.to_string()))?;
        tx.put(&module_obj, "download_count", module.download_count)
            .map_err(|e| Error::AutomergeError(e.to_string()))?;

        tx.commit();

        debug!("Updated CRDT for module {}", module.id);
        Ok(())
    }

    async fn update_rating_crdt(&self, rating: &Rating) -> Result<()> {
        let mut doc = self.doc.write();
        let mut tx = doc.transaction();

        let ratings_key = format!("{}_ratings", rating.module_id);
        let ratings_obj = tx
            .put_object(ROOT, &ratings_key, ObjType::List)
            .map_err(|e| Error::AutomergeError(e.to_string()))?;

        let rating_obj = tx
            .insert_object(&ratings_obj, 0, ObjType::Map)
            .map_err(|e| Error::AutomergeError(e.to_string()))?;

        tx.put(&rating_obj, "user_did", rating.user_did.as_str())
            .map_err(|e| Error::AutomergeError(e.to_string()))?;
        tx.put(&rating_obj, "stars", rating.stars as i64)
            .map_err(|e| Error::AutomergeError(e.to_string()))?;

        tx.commit();

        Ok(())
    }

    async fn sign_module(&self, module_id: &str, version: &str, hash: &str) -> Result<String> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(module_id.as_bytes());
        hasher.update(version.as_bytes());
        hasher.update(hash.as_bytes());
        hasher.update(self.config.owner_did.as_bytes());

        Ok(format!("{:x}", hasher.finalize()))
    }

    async fn download_wasm(&self, module_id: &str, version: &str) -> Result<WasmModule> {
        // In a real implementation, this would download from P2P network
        // For now, return a placeholder
        warn!(
            "download_wasm not fully implemented for {}@{}",
            module_id, version
        );
        Err(Error::NetworkError("Not implemented".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_registry() {
        let registry = Registry::new("did:key:test").await.unwrap();
        assert_eq!(registry.config.owner_did, "did:key:test");
    }

    #[tokio::test]
    async fn test_module_validation() {
        let module = GenModule::new(
            "io.univrs.test",
            "Test Module",
            "A test module",
            "did:key:alice",
            "MIT",
        );
        assert!(module.validate_id());

        let invalid = GenModule::new(
            "invalid",
            "Invalid",
            "Invalid module",
            "did:key:alice",
            "MIT",
        );
        assert!(!invalid.validate_id());
    }
}
