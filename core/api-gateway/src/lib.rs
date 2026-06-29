use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Route {
    pub method: String,
    pub path: String,
    pub target_service: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GatewayError {
    #[error("route method cannot be empty")]
    EmptyMethod,
    #[error("route path cannot be empty")]
    EmptyPath,
    #[error("target service cannot be empty")]
    EmptyTarget,
    #[error("route not found")]
    RouteNotFound,
}

#[derive(Debug, Default)]
pub struct RouteTable {
    routes: HashMap<(String, String), Route>,
}

impl RouteTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, route: Route) -> Result<(), GatewayError> {
        if route.method.trim().is_empty() {
            return Err(GatewayError::EmptyMethod);
        }
        if route.path.trim().is_empty() {
            return Err(GatewayError::EmptyPath);
        }
        if route.target_service.trim().is_empty() {
            return Err(GatewayError::EmptyTarget);
        }

        let key = (route.method.to_uppercase(), route.path.clone());
        self.routes.insert(key, route);
        Ok(())
    }

    pub fn resolve(&self, method: &str, path: &str) -> Result<&Route, GatewayError> {
        self.routes
            .get(&(method.to_uppercase(), path.to_string()))
            .ok_or(GatewayError::RouteNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_registered_route() {
        let mut table = RouteTable::new();
        table
            .register(Route {
                method: "GET".to_string(),
                path: "/v1/health".to_string(),
                target_service: "health-monitoring".to_string(),
            })
            .expect("route should register");

        let route = table
            .resolve("get", "/v1/health")
            .expect("route should resolve");
        assert_eq!(route.target_service, "health-monitoring");
    }

    #[test]
    fn fails_for_missing_route() {
        let table = RouteTable::new();
        let error = table
            .resolve("GET", "/v1/unknown")
            .expect_err("missing route should fail");

        assert_eq!(error, GatewayError::RouteNotFound);
    }
}
