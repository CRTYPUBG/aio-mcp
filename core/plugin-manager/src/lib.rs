use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginManifest {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PluginRegistryError {
    #[error("plugin id cannot be empty")]
    EmptyPluginId,
    #[error("plugin version cannot be empty")]
    EmptyPluginVersion,
}

#[derive(Debug, Default)]
pub struct PluginRegistry {
    plugins: HashMap<String, PluginManifest>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, manifest: PluginManifest) -> Result<(), PluginRegistryError> {
        if manifest.id.trim().is_empty() {
            return Err(PluginRegistryError::EmptyPluginId);
        }

        if manifest.version.trim().is_empty() {
            return Err(PluginRegistryError::EmptyPluginVersion);
        }

        self.plugins.insert(manifest.id.clone(), manifest);
        Ok(())
    }

    pub fn list(&self) -> Vec<&PluginManifest> {
        self.plugins.values().collect()
    }

    pub fn get(&self, plugin_id: &str) -> Option<&PluginManifest> {
        self.plugins.get(plugin_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_manifest() {
        let mut registry = PluginRegistry::new();
        registry
            .add(PluginManifest {
                id: "official.github".to_string(),
                version: "1.0.0".to_string(),
            })
            .expect("manifest should store");

        assert_eq!(registry.list().len(), 1);
        assert_eq!(
            registry
                .get("official.github")
                .expect("manifest should exist")
                .version,
            "1.0.0"
        );
    }

    #[test]
    fn updates_existing_manifest() {
        let mut registry = PluginRegistry::new();
        registry
            .add(PluginManifest {
                id: "official.github".to_string(),
                version: "1.0.0".to_string(),
            })
            .expect("manifest should store");

        registry
            .add(PluginManifest {
                id: "official.github".to_string(),
                version: "1.1.0".to_string(),
            })
            .expect("manifest should update");

        assert_eq!(registry.list().len(), 1);
        assert_eq!(
            registry
                .get("official.github")
                .expect("manifest should exist")
                .version,
            "1.1.0"
        );
    }
}
