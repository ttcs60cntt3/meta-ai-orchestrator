//! Task dispatcher for request routing and load balancing

use dashmap::DashMap;
use meta_ai_common::{
    error::{Error, Result},
    types::{LlmRequest, LlmResponse, TaskId, LlmProvider},
    metrics::MetricsCollector,
};
use meta_ai_core::agent::{Agent, SelectionStrategy};
use std::{
    sync::{Arc, atomic::{AtomicU64, Ordering}},
    time::Instant,
};
use tokio::sync::Semaphore;
use tracing::{info, warn, instrument};

/// Task dispatcher for routing requests to agents
pub struct TaskDispatcher {
    active_requests: Arc<DashMap<TaskId, DispatchedRequest>>,
    request_counter: Arc<AtomicU64>,
    max_concurrent: usize,
    semaphore: Arc<Semaphore>,
}

/// Dispatched request tracking
#[derive(Debug)]
struct DispatchedRequest {
    agent_provider: LlmProvider,
    started_at: Instant,
    attempt: u32,
}

impl TaskDispatcher {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            active_requests: Arc::new(DashMap::new()),
            request_counter: Arc::new(AtomicU64::new(0)),
            max_concurrent,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }
    
    /// Dispatch request to best available agent
    #[instrument(skip(self, request, agents))]
    pub async fn dispatch(
        &self,
        request: LlmRequest,
        agents: &[Box<dyn Agent>],
        selection_strategy: SelectionStrategy,
    ) -> Result<LlmResponse> {
        // Acquire semaphore permit
        let _permit = self.semaphore.acquire().await
            .map_err(|_| Error::Internal("Failed to acquire dispatcher permit".to_string()))?;
        
        let request_id = request.id;
        let task_id = request.task_id;
        
        // Select agent based on strategy
        let agent = self.select_agent(&request, agents, selection_strategy).await?;
        
        // Track request
        let dispatched = DispatchedRequest {
            agent_provider: agent.provider(),
            started_at: Instant::now(),
            attempt: 1,
        };
        self.active_requests.insert(task_id, dispatched);
        
        // Execute request
        let result = agent.submit(request).await;
        
        // Remove from tracking
        self.active_requests.remove(&task_id);
        
        match result {
            Ok(response) => {
                info!("Request {} completed successfully", request_id);
                Ok(response)
            }
            Err(e) => {
                warn!("Request {} failed: {}", request_id, e);
                Err(e)
            }
        }
    }
    
    /// Select agent based on strategy
    async fn select_agent(
        &self,
        request: &LlmRequest,
        agents: &[Box<dyn Agent>],
        strategy: SelectionStrategy,
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
        
        match strategy {
            SelectionStrategy::RoundRobin => {
                let count = self.request_counter.fetch_add(1, Ordering::Relaxed);
                let index = (count as usize) % available_agents.len();
                Ok(available_agents[index])
            }
            
            SelectionStrategy::LowestLatency => {
                // Get health info for all agents
                let mut best_agent = available_agents[0];
                let mut best_latency = f64::MAX;
                
                for agent in available_agents {
                    if let Ok(health) = agent.health_check().await {
                        if health.average_latency_ms < best_latency {
                            best_latency = health.average_latency_ms;
                            best_agent = agent;
                        }
                    }
                }
                
                Ok(best_agent)
            }
            
            SelectionStrategy::BestMatch => {
                // Select based on provider preferences
                let preferred_provider = request.provider;
                
                // Try to find exact match first
                for agent in &available_agents {
                    if agent.provider() == preferred_provider {
                        return Ok(agent);
                    }
                }
                
                // Fall back to first available
                Ok(available_agents[0])
            }
            
            SelectionStrategy::CostOptimized => {
                // For now, use round-robin (would need cost data)
                let count = self.request_counter.fetch_add(1, Ordering::Relaxed);
                let index = (count as usize) % available_agents.len();
                Ok(available_agents[index])
            }
            
            SelectionStrategy::Random => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..available_agents.len());
                Ok(available_agents[index])
            }
        }
    }
    
    /// Get current dispatch statistics
    pub fn get_stats(&self) -> DispatchStats {
        let active_count = self.active_requests.len();
        let available_permits = self.semaphore.available_permits();
        
        DispatchStats {
            active_requests: active_count,
            available_capacity: available_permits,
            total_capacity: self.max_concurrent,
            utilization: (self.max_concurrent - available_permits) as f64 / self.max_concurrent as f64,
        }
    }
}

/// Dispatch statistics
#[derive(Debug, Clone)]
pub struct DispatchStats {
    pub active_requests: usize,
    pub available_capacity: usize,
    pub total_capacity: usize,
    pub utilization: f64,
}

/// Load balancer for multiple dispatchers
pub struct LoadBalancer {
    dispatchers: Vec<Arc<TaskDispatcher>>,
    current_index: Arc<AtomicU64>,
}

impl LoadBalancer {
    pub fn new(dispatchers: Vec<Arc<TaskDispatcher>>) -> Self {
        Self {
            dispatchers,
            current_index: Arc::new(AtomicU64::new(0)),
        }
    }
    
    /// Get next dispatcher using round-robin
    pub fn next_dispatcher(&self) -> &Arc<TaskDispatcher> {
        let index = self.current_index.fetch_add(1, Ordering::Relaxed) as usize;
        &self.dispatchers[index % self.dispatchers.len()]
    }
    
    /// Get dispatcher with least load
    pub fn least_loaded_dispatcher(&self) -> &Arc<TaskDispatcher> {
        self.dispatchers
            .iter()
            .min_by_key(|d| d.get_stats().active_requests)
            .unwrap_or(&self.dispatchers[0])
    }
    
    /// Get overall stats
    pub fn get_overall_stats(&self) -> LoadBalancerStats {
        let stats: Vec<_> = self.dispatchers.iter().map(|d| d.get_stats()).collect();
        
        let total_active = stats.iter().map(|s| s.active_requests).sum();
        let total_capacity = stats.iter().map(|s| s.total_capacity).sum();
        let average_utilization = stats.iter().map(|s| s.utilization).sum::<f64>() / stats.len() as f64;
        
        LoadBalancerStats {
            total_dispatchers: self.dispatchers.len(),
            total_active_requests: total_active,
            total_capacity,
            average_utilization,
            dispatcher_stats: stats,
        }
    }
}

/// Load balancer statistics
#[derive(Debug, Clone)]
pub struct LoadBalancerStats {
    pub total_dispatchers: usize,
    pub total_active_requests: usize,
    pub total_capacity: usize,
    pub average_utilization: f64,
    pub dispatcher_stats: Vec<DispatchStats>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use meta_ai_common::types::*;
    use mockall::mock;
    use uuid::Uuid;
    
    mock! {
        TestAgent {}
        
        #[async_trait::async_trait]
        impl Agent for TestAgent {
            fn name(&self) -> &str;
            fn provider(&self) -> LlmProvider;
            async fn is_available(&self) -> bool;
            async fn submit(&self, request: LlmRequest) -> Result<LlmResponse>;
            fn capabilities(&self) -> meta_ai_core::agent::AgentCapabilities;
            async fn health_check(&self) -> Result<meta_ai_core::agent::AgentHealth>;
            async fn rate_limit_info(&self) -> Result<meta_ai_core::agent::RateLimitInfo>;
        }
    }
    
    #[tokio::test]
    async fn test_dispatcher_creation() {
        let dispatcher = TaskDispatcher::new(10);
        let stats = dispatcher.get_stats();
        
        assert_eq!(stats.total_capacity, 10);
        assert_eq!(stats.active_requests, 0);
        assert_eq!(stats.available_capacity, 10);
    }
    
    #[tokio::test]
    async fn test_agent_selection() {
        // Test implementation would go here
    }
}