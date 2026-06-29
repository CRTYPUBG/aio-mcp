use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDescriptor {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EngineEventKind {
    ServiceRegistered,
    ServiceUpdated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EngineEvent {
    pub kind: EngineEventKind,
    pub service_id: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreEngineError {
    #[error("service id cannot be empty")]
    EmptyServiceId,
    #[error("service version cannot be empty")]
    EmptyServiceVersion,
}

#[derive(Debug, Default)]
pub struct CoreEngine {
    services: HashMap<String, ServiceDescriptor>,
    events: Vec<EngineEvent>,
}

impl CoreEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_service(&mut self, service: ServiceDescriptor) -> Result<(), CoreEngineError> {
        if service.id.trim().is_empty() {
            return Err(CoreEngineError::EmptyServiceId);
        }

        if service.version.trim().is_empty() {
            return Err(CoreEngineError::EmptyServiceVersion);
        }

        let event_kind = if self.services.contains_key(&service.id) {
            EngineEventKind::ServiceUpdated
        } else {
            EngineEventKind::ServiceRegistered
        };

        self.events.push(EngineEvent {
            kind: event_kind,
            service_id: service.id.clone(),
        });

        self.services.insert(service.id.clone(), service);
        Ok(())
    }

    pub fn service_count(&self) -> usize {
        self.services.len()
    }

    pub fn get_service(&self, service_id: &str) -> Option<&ServiceDescriptor> {
        self.services.get(service_id)
    }

    pub fn take_events(&mut self) -> Vec<EngineEvent> {
        std::mem::take(&mut self.events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_service() {
        let mut engine = CoreEngine::new();
        engine
            .register_service(ServiceDescriptor {
            id: "plugin-manager".to_string(),
            version: "0.1.0".to_string(),
        })
            .expect("service should register");

        assert_eq!(engine.service_count(), 1);
        let events = engine.take_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EngineEventKind::ServiceRegistered);
    }

    #[test]
    fn updates_existing_service() {
        let mut engine = CoreEngine::new();
        engine
            .register_service(ServiceDescriptor {
                id: "plugin-manager".to_string(),
                version: "0.1.0".to_string(),
            })
            .expect("service should register");
        engine.take_events();

        engine
            .register_service(ServiceDescriptor {
                id: "plugin-manager".to_string(),
                version: "0.2.0".to_string(),
            })
            .expect("service should update");

        assert_eq!(engine.service_count(), 1);
        assert_eq!(
            engine
                .get_service("plugin-manager")
                .expect("service should exist")
                .version,
            "0.2.0"
        );

        let events = engine.take_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EngineEventKind::ServiceUpdated);
    }

    #[test]
    fn rejects_invalid_service() {
        let mut engine = CoreEngine::new();
        let error = engine
            .register_service(ServiceDescriptor {
                id: "".to_string(),
                version: "0.1.0".to_string(),
            })
            .expect_err("empty id should fail");

        assert_eq!(error, CoreEngineError::EmptyServiceId);
    }
}
