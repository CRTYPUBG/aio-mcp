use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GrantState {
    Pending,
    Granted,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermissionGrant {
    pub principal_id: String,
    pub scope: String,
    pub state: GrantState,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PermissionError {
    #[error("principal id cannot be empty")]
    EmptyPrincipal,
    #[error("scope cannot be empty")]
    EmptyScope,
    #[error("grant not found")]
    NotFound,
}

#[derive(Debug, Default)]
pub struct PermissionStore {
    grants: HashMap<(String, String), PermissionGrant>,
}

impl PermissionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn request(
        &mut self,
        principal_id: &str,
        scope: &str,
    ) -> Result<&PermissionGrant, PermissionError> {
        if principal_id.trim().is_empty() {
            return Err(PermissionError::EmptyPrincipal);
        }
        if scope.trim().is_empty() {
            return Err(PermissionError::EmptyScope);
        }

        let key = (principal_id.to_string(), scope.to_string());
        self.grants.insert(
            key,
            PermissionGrant {
                principal_id: principal_id.to_string(),
                scope: scope.to_string(),
                state: GrantState::Pending,
            },
        );

        Ok(self
            .grants
            .get(&(principal_id.to_string(), scope.to_string()))
            .expect("grant should exist right after insert"))
    }

    pub fn set_state(
        &mut self,
        principal_id: &str,
        scope: &str,
        state: GrantState,
    ) -> Result<&PermissionGrant, PermissionError> {
        let key = (principal_id.to_string(), scope.to_string());
        let grant = self.grants.get_mut(&key).ok_or(PermissionError::NotFound)?;
        grant.state = state;

        Ok(self
            .grants
            .get(&key)
            .expect("grant should exist right after update"))
    }

    pub fn can_access(&self, principal_id: &str, scope: &str) -> bool {
        self.grants
            .get(&(principal_id.to_string(), scope.to_string()))
            .map(|grant| grant.state == GrantState::Granted)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grants_after_approval() {
        let mut store = PermissionStore::new();
        store
            .request("user-1", "plugin.install")
            .expect("request should succeed");

        assert!(!store.can_access("user-1", "plugin.install"));

        store
            .set_state("user-1", "plugin.install", GrantState::Granted)
            .expect("approval should succeed");

        assert!(store.can_access("user-1", "plugin.install"));
    }

    #[test]
    fn rejects_empty_scope() {
        let mut store = PermissionStore::new();
        let error = store
            .request("user-1", "")
            .expect_err("empty scope should fail");

        assert_eq!(error, PermissionError::EmptyScope);
    }
}
