//! Dynamic Routing Table and Role Matching for Multi-Agent Swarms.

pub mod consistent_hash;

use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct DynamicRouter {
    routes: RwLock<HashMap<String, String>>,
}

impl DynamicRouter {
    pub fn new() -> Self {
        let mut routes = HashMap::new();
        routes.insert("ast".into(), "actor_cognitive_architect".into());
        routes.insert("codegen".into(), "actor_systems_coder".into());
        routes.insert("verify".into(), "actor_verification_engine".into());
        routes.insert("security".into(), "actor_security_auditor".into());

        Self {
            routes: RwLock::new(routes),
        }
    }

    pub async fn resolve_target(&self, capability: &str) -> Option<String> {
        let guard = self.routes.read().await;
        guard.get(capability).cloned()
    }

    pub async fn register_route(&self, capability: String, actor_id: String) {
        let mut guard = self.routes.write().await;
        guard.insert(capability, actor_id);
    }
}
