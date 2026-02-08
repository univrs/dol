//! Data models for Gen Registry
//!
//! These models correspond to the DOL schemas in schemas/gen_module.dol

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Gen Module metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenModule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author_did: String,
    pub license: String,
    pub tags: HashSet<String>,
    pub versions: Vec<ModuleVersion>,
    pub latest_version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub download_count: i64,
    pub dependencies: Vec<Dependency>,
}

impl GenModule {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        author_did: impl Into<String>,
        license: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            author_did: author_did.into(),
            license: license.into(),
            tags: HashSet::new(),
            versions: Vec::new(),
            latest_version: "0.0.0".to_string(),
            created_at: now,
            updated_at: now,
            download_count: 0,
            dependencies: Vec::new(),
        }
    }

    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.tags.insert(tag.into());
    }

    pub fn add_version(&mut self, version: ModuleVersion) {
        self.versions.push(version.clone());
        self.latest_version = version.version;
        self.updated_at = Utc::now();
    }

    pub fn add_dependency(&mut self, dep: Dependency) {
        self.dependencies.push(dep);
    }

    pub fn validate_id(&self) -> bool {
        // Reverse-domain notation: io.univrs.user
        let parts: Vec<&str> = self.id.split('.').collect();
        parts.len() >= 2 && parts.iter().all(|p| !p.is_empty())
    }
}

/// Module version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleVersion {
    pub version: String,
    pub published_at: DateTime<Utc>,
    pub wasm_hash: String,
    pub wasm_size: u64,
    pub changelog: String,
    pub signature: String,
    pub capabilities: Vec<Capability>,
    pub deprecated: bool,
    pub yanked: bool,
}

impl ModuleVersion {
    pub fn new(
        version: impl Into<String>,
        wasm_hash: impl Into<String>,
        wasm_size: u64,
        changelog: impl Into<String>,
        signature: impl Into<String>,
    ) -> Self {
        Self {
            version: version.into(),
            published_at: Utc::now(),
            wasm_hash: wasm_hash.into(),
            wasm_size,
            changelog: changelog.into(),
            signature: signature.into(),
            capabilities: Vec::new(),
            deprecated: false,
            yanked: false,
        }
    }

    pub fn add_capability(&mut self, cap: Capability) {
        self.capabilities.push(cap);
    }
}

/// Module dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub module_id: String,
    pub version_requirement: String,
    pub optional: bool,
}

impl Dependency {
    pub fn new(module_id: impl Into<String>, version_requirement: impl Into<String>) -> Self {
        Self {
            module_id: module_id.into(),
            version_requirement: version_requirement.into(),
            optional: false,
        }
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
}

/// Exported capability (function/type)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub kind: CapabilityKind,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CapabilityKind {
    Function,
    Type,
    Trait,
}

impl Capability {
    pub fn function(name: impl Into<String>, signature: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: CapabilityKind::Function,
            signature: signature.into(),
        }
    }

    pub fn type_def(name: impl Into<String>, signature: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: CapabilityKind::Type,
            signature: signature.into(),
        }
    }
}

/// Search index entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndex {
    pub module_id: String,
    pub keywords: HashSet<String>,
    pub popularity_score: i64,
    pub last_indexed: DateTime<Utc>,
}

impl SearchIndex {
    pub fn new(module: &GenModule) -> Self {
        let mut keywords = HashSet::new();

        // Extract keywords from name
        for word in module.name.split_whitespace() {
            keywords.insert(word.to_lowercase());
        }

        // Extract from description
        for word in module.description.split_whitespace() {
            keywords.insert(word.to_lowercase());
        }

        // Add tags
        keywords.extend(module.tags.iter().cloned());

        Self {
            module_id: module.id.clone(),
            keywords,
            popularity_score: module.download_count,
            last_indexed: Utc::now(),
        }
    }
}

/// User rating
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rating {
    pub module_id: String,
    pub user_did: String,
    pub stars: u8,
    pub review: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Rating {
    pub fn new(module_id: impl Into<String>, user_did: impl Into<String>, stars: u8) -> Self {
        let now = Utc::now();
        Self {
            module_id: module_id.into(),
            user_did: user_did.into(),
            stars: stars.clamp(1, 5),
            review: String::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_review(mut self, review: impl Into<String>) -> Self {
        self.review = review.into();
        self
    }
}

/// Installed module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledModule {
    pub module_id: String,
    pub version: String,
    pub installed_at: DateTime<Utc>,
    pub auto_update: bool,
    pub update_history: Vec<UpdateRecord>,
}

impl InstalledModule {
    pub fn new(module_id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            module_id: module_id.into(),
            version: version.into(),
            installed_at: Utc::now(),
            auto_update: false,
            update_history: Vec::new(),
        }
    }

    pub fn record_update(&mut self, from: String, to: String, success: bool) {
        self.update_history.push(UpdateRecord {
            from_version: from,
            to_version: to.clone(),
            updated_at: Utc::now(),
            success,
        });
        if success {
            self.version = to;
        }
    }
}

/// Update record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRecord {
    pub from_version: String,
    pub to_version: String,
    pub updated_at: DateTime<Utc>,
    pub success: bool,
}

/// Publish capability (Meadowcap)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishCapability {
    pub capability_id: String,
    pub namespace_id: String,
    pub delegate_did: String,
    pub granted_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub allowed_modules: HashSet<String>,
    pub revoked: bool,
}

impl PublishCapability {
    pub fn can_publish(&self, module_id: &str) -> bool {
        if self.revoked {
            return false;
        }
        if Utc::now() > self.expires_at {
            return false;
        }
        self.allowed_modules.is_empty() || self.allowed_modules.contains(module_id)
    }
}

/// Sync state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub peer_id: String,
    pub last_sync: DateTime<Utc>,
    pub bytes_sent: i64,
    pub bytes_received: i64,
    pub modules_synced: i64,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncStatus {
    Idle,
    Syncing,
    Error,
}

impl SyncState {
    pub fn new(peer_id: impl Into<String>) -> Self {
        Self {
            peer_id: peer_id.into(),
            last_sync: Utc::now(),
            bytes_sent: 0,
            bytes_received: 0,
            modules_synced: 0,
            sync_status: SyncStatus::Idle,
        }
    }
}
