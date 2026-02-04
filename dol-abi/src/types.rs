//! Type definitions for the DOL ABI

use serde::{Deserialize, Serialize};

/// A qualified identifier in DOL (e.g., "domain.property.version")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QualifiedId {
    /// Domain part
    pub domain: String,
    /// Property part
    pub property: String,
    /// Version part (optional)
    pub version: Option<String>,
}

impl QualifiedId {
    /// Create a new qualified identifier
    pub fn new(domain: impl Into<String>, property: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            property: property.into(),
            version: None,
        }
    }

    /// Create a new qualified identifier with version
    pub fn with_version(
        domain: impl Into<String>,
        property: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Self {
            domain: domain.into(),
            property: property.into(),
            version: Some(version.into()),
        }
    }
}

impl std::fmt::Display for QualifiedId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.domain, self.property)?;
        if let Some(v) = &self.version {
            write!(f, ".{}", v)?;
        }
        Ok(())
    }
}
