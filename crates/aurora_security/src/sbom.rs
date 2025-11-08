//! Software Bill of Materials (SBOM) generation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SBOM format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SbomFormat {
    /// CycloneDX format
    CycloneDX,
    /// SPDX format
    SPDX,
}

/// Component type in SBOM
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentType {
    /// Library
    Library,
    /// Application
    Application,
    /// Framework
    Framework,
    /// Operating System
    OperatingSystem,
}

/// Dependency component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    /// Component name
    pub name: String,
    /// Component version
    pub version: String,
    /// Component type
    #[serde(rename = "type")]
    pub component_type: ComponentType,
    /// License identifier (SPDX)
    pub license: Option<String>,
    /// Package URL (purl)
    pub purl: Option<String>,
    /// Hashes (algorithm -> value)
    #[serde(default)]
    pub hashes: HashMap<String, String>,
}

impl Component {
    /// Create new component
    pub fn new(name: String, version: String, component_type: ComponentType) -> Self {
        Self {
            name,
            version,
            component_type,
            license: None,
            purl: None,
            hashes: HashMap::new(),
        }
    }

    /// Add hash
    pub fn with_hash(mut self, algorithm: String, hash: String) -> Self {
        self.hashes.insert(algorithm, hash);
        self
    }

    /// Set license
    pub fn with_license(mut self, license: String) -> Self {
        self.license = Some(license);
        self
    }

    /// Set package URL
    pub fn with_purl(mut self, purl: String) -> Self {
        self.purl = Some(purl);
        self
    }
}

/// Software Bill of Materials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sbom {
    /// SBOM metadata
    pub metadata: SbomMetadata,
    /// Components list
    pub components: Vec<Component>,
    /// Dependencies graph
    #[serde(default)]
    pub dependencies: HashMap<String, Vec<String>>,
}

/// SBOM metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomMetadata {
    /// Tool name
    pub tool: String,
    /// Tool version
    pub version: String,
    /// Timestamp
    pub timestamp: String,
}

impl Sbom {
    /// Create new SBOM
    pub fn new(tool: String, version: String) -> Self {
        Self {
            metadata: SbomMetadata {
                tool,
                version,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            components: Vec::new(),
            dependencies: HashMap::new(),
        }
    }

    /// Add component
    pub fn add_component(&mut self, component: Component) {
        self.components.push(component);
    }

    /// Add dependency relationship
    pub fn add_dependency(&mut self, from: String, to: String) {
        self.dependencies.entry(from).or_default().push(to);
    }

    /// Export to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Get component count
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Verify all components have hashes
    pub fn verify_hashes(&self) -> Result<(), String> {
        for component in &self.components {
            if component.hashes.is_empty() {
                return Err(format!("Component {} has no hashes", component.name));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_creation() {
        let comp = Component::new(
            "test-lib".to_string(),
            "1.0.0".to_string(),
            ComponentType::Library,
        );
        assert_eq!(comp.name, "test-lib");
        assert_eq!(comp.version, "1.0.0");
    }

    #[test]
    fn test_component_with_hash() {
        let comp = Component::new(
            "test-lib".to_string(),
            "1.0.0".to_string(),
            ComponentType::Library,
        )
        .with_hash("sha256".to_string(), "abc123".to_string());

        assert_eq!(comp.hashes.get("sha256"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_component_with_license() {
        let comp = Component::new(
            "test-lib".to_string(),
            "1.0.0".to_string(),
            ComponentType::Library,
        )
        .with_license("MIT".to_string());

        assert_eq!(comp.license, Some("MIT".to_string()));
    }

    #[test]
    fn test_sbom_creation() {
        let sbom = Sbom::new("aurora".to_string(), "0.1.0".to_string());
        assert_eq!(sbom.metadata.tool, "aurora");
        assert_eq!(sbom.component_count(), 0);
    }

    #[test]
    fn test_sbom_add_component() {
        let mut sbom = Sbom::new("aurora".to_string(), "0.1.0".to_string());
        let comp = Component::new(
            "dep1".to_string(),
            "1.0.0".to_string(),
            ComponentType::Library,
        );

        sbom.add_component(comp);
        assert_eq!(sbom.component_count(), 1);
    }

    #[test]
    fn test_sbom_add_dependency() {
        let mut sbom = Sbom::new("aurora".to_string(), "0.1.0".to_string());
        sbom.add_dependency("app".to_string(), "lib1".to_string());
        sbom.add_dependency("app".to_string(), "lib2".to_string());

        assert_eq!(sbom.dependencies.get("app").unwrap().len(), 2);
    }

    #[test]
    fn test_sbom_verify_hashes_ok() {
        let mut sbom = Sbom::new("aurora".to_string(), "0.1.0".to_string());
        let comp = Component::new(
            "dep1".to_string(),
            "1.0.0".to_string(),
            ComponentType::Library,
        )
        .with_hash("sha256".to_string(), "abc".to_string());

        sbom.add_component(comp);
        assert!(sbom.verify_hashes().is_ok());
    }

    #[test]
    fn test_sbom_verify_hashes_fail() {
        let mut sbom = Sbom::new("aurora".to_string(), "0.1.0".to_string());
        let comp = Component::new(
            "dep1".to_string(),
            "1.0.0".to_string(),
            ComponentType::Library,
        );

        sbom.add_component(comp);
        assert!(sbom.verify_hashes().is_err());
    }

    #[test]
    fn test_sbom_to_json() {
        let sbom = Sbom::new("aurora".to_string(), "0.1.0".to_string());
        let json = sbom.to_json();
        assert!(json.is_ok());
        assert!(json.unwrap().contains("aurora"));
    }
}
