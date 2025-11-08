//! Workspace and package management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Build error types
#[derive(Debug, Error)]
pub enum BuildError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML parse error
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// Invalid manifest
    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    /// Build failed
    #[error("Build failed: {0}")]
    BuildFailed(String),
}

/// Result type for build operations
pub type Result<T> = std::result::Result<T, BuildError>;

/// Package metadata from Aurora.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Package authors
    pub authors: Vec<String>,
    /// Package edition (e.g., "2025")
    pub edition: String,
    /// Package description
    #[serde(default)]
    pub description: String,
    /// Package license
    #[serde(default)]
    pub license: String,
}

/// Dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    /// Simple version string
    Simple(String),
    /// Detailed dependency
    Detailed {
        /// Version requirement
        version: String,
        /// Path to local dependency
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<PathBuf>,
        /// Git repository URL
        #[serde(skip_serializing_if = "Option::is_none")]
        git: Option<String>,
        /// Optional features
        #[serde(default)]
        features: Vec<String>,
    },
}

/// Aurora.toml manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Package metadata
    pub package: PackageMetadata,
    /// Dependencies
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
    /// Dev dependencies
    #[serde(default, rename = "dev-dependencies")]
    pub dev_dependencies: HashMap<String, Dependency>,
    /// Build profiles
    #[serde(default)]
    pub profile: HashMap<String, ProfileConfig>,
}

/// Build profile configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// Optimization level (0-3)
    #[serde(default)]
    pub opt_level: u8,
    /// Include debug info
    #[serde(default)]
    pub debug: bool,
    /// Link-time optimization
    #[serde(default)]
    pub lto: bool,
}

impl Manifest {
    /// Load manifest from path
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let manifest = toml::from_str(&content)?;
        Ok(manifest)
    }

    /// Find manifest in directory or parents
    pub fn find_in_dir(dir: &Path) -> Result<(Self, PathBuf)> {
        let mut current = dir.to_path_buf();
        loop {
            let manifest_path = current.join("Aurora.toml");
            if manifest_path.exists() {
                let manifest = Self::load(&manifest_path)?;
                return Ok((manifest, current));
            }

            if !current.pop() {
                return Err(BuildError::InvalidManifest(
                    "Aurora.toml not found in directory tree".to_string()
                ));
            }
        }
    }

    /// Save manifest to path
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| BuildError::InvalidManifest(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Build cache for incremental compilation
#[derive(Debug, Default)]
pub struct BuildCache {
    /// Content-addressed cache entries
    entries: HashMap<String, CacheEntry>,
}

/// Cache entry for a build artifact
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Content hash
    pub hash: String,
    /// Artifact path
    pub path: PathBuf,
    /// Timestamp
    pub timestamp: u64,
}

impl BuildCache {
    /// Create new cache
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert cache entry
    pub fn insert(&mut self, key: String, entry: CacheEntry) {
        self.entries.insert(key, entry);
    }

    /// Get cache entry
    pub fn get(&self, key: &str) -> Option<&CacheEntry> {
        self.entries.get(key)
    }

    /// Check if cache is valid for source
    pub fn is_valid(&self, key: &str, source_modified: u64) -> bool {
        if let Some(entry) = self.get(key) {
            entry.timestamp >= source_modified
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_metadata() {
        let pkg = PackageMetadata {
            name: "test".to_string(),
            version: "0.1.0".to_string(),
            authors: vec!["Test Author".to_string()],
            edition: "2025".to_string(),
            description: "Test package".to_string(),
            license: "MIT".to_string(),
        };
        assert_eq!(pkg.name, "test");
        assert_eq!(pkg.version, "0.1.0");
    }

    #[test]
    fn test_build_cache() {
        let mut cache = BuildCache::new();
        let entry = CacheEntry {
            hash: "abc123".to_string(),
            path: PathBuf::from("/tmp/test.o"),
            timestamp: 1000,
        };

        cache.insert("test.ax".to_string(), entry.clone());
        assert!(cache.get("test.ax").is_some());
        assert!(cache.is_valid("test.ax", 500));
        assert!(!cache.is_valid("test.ax", 2000));
    }

    #[test]
    fn test_dependency_simple() {
        let dep = Dependency::Simple("1.0.0".to_string());
        match dep {
            Dependency::Simple(v) => assert_eq!(v, "1.0.0"),
            _ => panic!("Expected Simple variant"),
        }
    }

    #[test]
    fn test_profile_config() {
        let profile = ProfileConfig {
            opt_level: 2,
            debug: true,
            lto: false,
        };
        assert_eq!(profile.opt_level, 2);
        assert!(profile.debug);
        assert!(!profile.lto);
    }
}
