use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigKey {
    pub scope: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub key: ConfigKey,
    pub value: Value,
    pub revision: u64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigurationError {
    #[error("scope cannot be empty")]
    EmptyScope,
    #[error("key cannot be empty")]
    EmptyKey,
    #[error("revision mismatch: expected {expected}, got {actual}")]
    RevisionMismatch { expected: u64, actual: u64 },
}

#[derive(Debug, Default)]
pub struct ConfigurationStore {
    entries: HashMap<(String, String), ConfigEntry>,
}

impl ConfigurationStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, scope: &str, key: &str) -> Option<&ConfigEntry> {
        self.entries.get(&(scope.to_string(), key.to_string()))
    }

    pub fn upsert(
        &mut self,
        scope: &str,
        key: &str,
        value: Value,
        expected_revision: Option<u64>,
    ) -> Result<&ConfigEntry, ConfigurationError> {
        if scope.trim().is_empty() {
            return Err(ConfigurationError::EmptyScope);
        }
        if key.trim().is_empty() {
            return Err(ConfigurationError::EmptyKey);
        }

        let map_key = (scope.to_string(), key.to_string());
        let current_revision = self.entries.get(&map_key).map(|entry| entry.revision).unwrap_or(0);

        if let Some(expected) = expected_revision {
            if expected != current_revision {
                return Err(ConfigurationError::RevisionMismatch {
                    expected,
                    actual: current_revision,
                });
            }
        }

        let new_revision = current_revision + 1;
        self.entries.insert(
            map_key,
            ConfigEntry {
                key: ConfigKey {
                    scope: scope.to_string(),
                    key: key.to_string(),
                },
                value,
                revision: new_revision,
            },
        );

        Ok(self
            .entries
            .get(&(scope.to_string(), key.to_string()))
            .expect("entry should exist right after insert"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn upserts_with_revision_checks() {
        let mut store = ConfigurationStore::new();

        let first = store
            .upsert("workspace", "theme", json!("light"), Some(0))
            .expect("first insert should succeed");
        assert_eq!(first.revision, 1);

        let second = store
            .upsert("workspace", "theme", json!("dark"), Some(1))
            .expect("update should succeed");
        assert_eq!(second.revision, 2);
    }

    #[test]
    fn rejects_revision_mismatch() {
        let mut store = ConfigurationStore::new();
        store
            .upsert("workspace", "theme", json!("light"), Some(0))
            .expect("insert should succeed");

        let error = store
            .upsert("workspace", "theme", json!("dark"), Some(0))
            .expect_err("stale revision should fail");

        assert_eq!(
            error,
            ConfigurationError::RevisionMismatch {
                expected: 0,
                actual: 1,
            }
        );
    }
}
