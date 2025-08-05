#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

//! Core AI model and interfaces for Meta-AI Orchestrator
//!
//! This crate defines the core abstractions and traits used throughout the system.

use async_trait::async_trait;
use meta_ai_common::{
    types::{LlmRequest, LlmResponse, Task, TaskId, TaskStatus, Priority},
    error::Result,
};
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod agent;
pub mod orchestrator;
pub mod rag;
pub mod evaluation;

pub use agent::Agent;
pub use orchestrator::Orchestrator;
pub use rag::RagEngine;
pub use evaluation::Evaluator;

/// Core AI trait - the main interface for all AI operations
#[async_trait]
pub trait CoreAI: Send + Sync {
    /// Process a task using the appropriate agent
    async fn process_task(&self, task: Task) -> Result<TaskStatus>;
    
    /// Submit an LLM request
    async fn submit_request(&self, request: LlmRequest) -> Result<LlmResponse>;
    
    /// Get task status
    async fn get_task_status(&self, task_id: TaskId) -> Result<TaskStatus>;
    
    /// Cancel a task
    async fn cancel_task(&self, task_id: TaskId) -> Result<()>;
    
    /// List all active tasks
    async fn list_active_tasks(&self) -> Result<Vec<Task>>;
    
    /// Get system health status
    async fn health_check(&self) -> Result<HealthStatus>;
}

/// System health status
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub healthy: bool,
    pub uptime_seconds: u64,
    pub active_tasks: usize,
    pub accuracy: f64,
    pub bug_rate: f64,
    pub agent_status: Vec<AgentStatus>,
}

/// Agent health status
#[derive(Debug, Clone)]
pub struct AgentStatus {
    pub name: String,
    pub healthy: bool,
    pub requests_per_minute: f64,
    pub average_latency_ms: f64,
    pub error_rate: f64,
}

/// Default Core AI implementation
pub struct MetaAICore {
    orchestrator: Arc<RwLock<Box<dyn Orchestrator>>>,
    agents: Arc<RwLock<Vec<Box<dyn Agent>>>>,
    rag_engine: Arc<RwLock<Box<dyn RagEngine>>>,
    evaluator: Arc<RwLock<Box<dyn Evaluator>>>,
    start_time: std::time::Instant,
}

impl MetaAICore {
    /// Create a new Core AI instance
    pub fn new(
        orchestrator: Box<dyn Orchestrator>,
        agents: Vec<Box<dyn Agent>>,
        rag_engine: Box<dyn RagEngine>,
        evaluator: Box<dyn Evaluator>,
    ) -> Self {
        Self {
            orchestrator: Arc::new(RwLock::new(orchestrator)),
            agents: Arc::new(RwLock::new(agents)),
            rag_engine: Arc::new(RwLock::new(rag_engine)),
            evaluator: Arc::new(RwLock::new(evaluator)),
            start_time: std::time::Instant::now(),
        }
    }
}

#[async_trait]
impl CoreAI for MetaAICore {
    async fn process_task(&self, task: Task) -> Result<TaskStatus> {
        // Run evaluation checks
        let evaluator = self.evaluator.read().await;
        evaluator.pre_task_validation(&task).await?;
        
        // Process through orchestrator
        let orchestrator = self.orchestrator.read().await;
        let status = orchestrator.execute_task(task.clone()).await?;
        
        // Post-task evaluation
        evaluator.post_task_validation(&task, &status).await?;
        
        Ok(status)
    }
    
    async fn submit_request(&self, request: LlmRequest) -> Result<LlmResponse> {
        let orchestrator = self.orchestrator.read().await;
        orchestrator.submit_request(request).await
    }
    
    async fn get_task_status(&self, task_id: TaskId) -> Result<TaskStatus> {
        let orchestrator = self.orchestrator.read().await;
        orchestrator.get_task_status(task_id).await
    }
    
    async fn cancel_task(&self, task_id: TaskId) -> Result<()> {
        let orchestrator = self.orchestrator.write().await;
        orchestrator.cancel_task(task_id).await
    }
    
    async fn list_active_tasks(&self) -> Result<Vec<Task>> {
        let orchestrator = self.orchestrator.read().await;
        orchestrator.list_active_tasks().await
    }
    
    async fn health_check(&self) -> Result<HealthStatus> {
        let evaluator = self.evaluator.read().await;
        let accuracy = evaluator.get_accuracy().await?;
        let bug_rate = evaluator.get_bug_rate().await?;
        
        let agents = self.agents.read().await;
        let mut agent_status = Vec::new();
        
        for agent in agents.iter() {
            let status = agent.health_check().await?;
            agent_status.push(AgentStatus {
                name: agent.name().to_string(),
                healthy: status.healthy,
                requests_per_minute: status.requests_per_minute,
                average_latency_ms: status.average_latency_ms,
                error_rate: status.error_rate,
            });
        }
        
        let orchestrator = self.orchestrator.read().await;
        let active_tasks = orchestrator.list_active_tasks().await?.len();
        
        Ok(HealthStatus {
            healthy: accuracy >= 0.9999 && bug_rate <= 0.0005,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            active_tasks,
            accuracy,
            bug_rate,
            agent_status,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    
    mock! {
        TestOrchestrator {}
        
        #[async_trait]
        impl Orchestrator for TestOrchestrator {
            async fn execute_task(&self, task: Task) -> Result<TaskStatus>;
            async fn submit_request(&self, request: LlmRequest) -> Result<LlmResponse>;
            async fn get_task_status(&self, task_id: TaskId) -> Result<TaskStatus>;
            async fn cancel_task(&mut self, task_id: TaskId) -> Result<()>;
            async fn list_active_tasks(&self) -> Result<Vec<Task>>;
        }
    }
    
    #[tokio::test]
    async fn test_core_ai_creation() {
        // Test will be implemented
    }
}