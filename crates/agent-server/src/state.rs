//! Shared application state

use agent_comms::{AgentCoordinator, InProcessTransport};
use agent_coordination::AgentFactory;
use agent_runtime::Agent;
use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use std::sync::Arc;

pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub model: String,
    pub agent: Arc<Agent>,
    pub created_at: DateTime<Utc>,
    pub messages: Arc<DashMap<usize, crate::models::MessageResponse>>,
}

#[derive(Clone)]
pub struct AppState {
    pub factory: Arc<AgentFactory>,
    pub coordinator: Arc<AgentCoordinator>,
    pub agents: Arc<DashMap<String, AgentInfo>>,
    pub openai_key: String,
    pub anthropic_key: Option<String>,
}

impl AppState {
    pub async fn new(openai_key: String, anthropic_key: Option<String>) -> Result<Self> {
        let transport = Arc::new(InProcessTransport::new());
        let coordinator = Arc::new(AgentCoordinator::new(transport.clone()));
        let factory = Arc::new(AgentFactory::new(
            coordinator.clone(),
            Some(openai_key.clone()),
            anthropic_key.clone(),
        ));

        Ok(Self {
            factory,
            coordinator,
            agents: Arc::new(DashMap::new()),
            openai_key,
            anthropic_key,
        })
    }
}
