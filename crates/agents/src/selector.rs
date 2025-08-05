//! Agent selector implementation

use async_trait::async_trait;
use meta_ai_common::{error::{Error, Result}, types::LlmRequest};
use meta_ai_core::agent::{Agent, AgentSelector, SelectionStrategy};

/// Default agent selector implementation
pub struct DefaultAgentSelector {
    strategy: SelectionStrategy,
}

impl DefaultAgentSelector {
    pub fn new(strategy: SelectionStrategy) -> Self {
        Self { strategy }
    }
}

#[async_trait]
impl AgentSelector for DefaultAgentSelector {
    async fn select_agent(
        &self,
        request: &LlmRequest,
        agents: &[Box<dyn Agent>],
    ) -> Result<&Box<dyn Agent>> {
        let available_agents: Vec<_> = futures::future::join_all(
            agents.iter().map(|agent| async {
                (agent, agent.is_available().await)
            })
        ).await.into_iter()
            .filter_map(|(agent, available)| if available { Some(agent) } else { None })
            .collect();
        
        if available_agents.is_empty() {
            return Err(Error::Agent("No available agents".to_string()));
        }
        
        match self.strategy {
            SelectionStrategy::RoundRobin => {
                // Simple round-robin for now
                Ok(available_agents[0])
            }
            
            SelectionStrategy::BestMatch => {
                // Try to match provider preference
                for agent in &available_agents {
                    if agent.provider() == request.provider {
                        return Ok(agent);
                    }
                }
                // Fall back to first available
                Ok(available_agents[0])
            }
            
            _ => Ok(available_agents[0]),
        }
    }
    
    fn strategy(&self) -> SelectionStrategy {
        self.strategy
    }
}