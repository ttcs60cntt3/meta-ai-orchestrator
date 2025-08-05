#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

//! Orchestrator implementation for Meta-AI

use async_trait::async_trait;
use dashmap::DashMap;
use meta_ai_common::{
    error::{Error, Result},
    types::{LlmRequest, LlmResponse, Task, TaskId, TaskStatus, Priority},
    metrics::{MetricsCollector, DefaultMetricsCollector},
};
use meta_ai_core::{
    agent::{Agent, AgentSelector, SelectionStrategy},
    orchestrator::{Orchestrator, TaskScheduler, DagExecutor, ExecutionStrategy, ResourceConstraints},
};
use parking_lot::RwLock;
use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, Semaphore};
use tracing::{info, warn, error, instrument};
use uuid::Uuid;

pub mod dag;
pub mod scheduler;
pub mod dispatcher;

use dag::DagExecutorImpl;
use scheduler::PriorityScheduler;
use dispatcher::TaskDispatcher;

/// Main orchestrator implementation
pub struct MetaAIOrchestrator {
    agents: Arc<Vec<Box<dyn Agent>>>,
    agent_selector: Arc<Box<dyn AgentSelector>>,
    scheduler: Arc<RwLock<Box<dyn TaskScheduler>>>,
    dag_executor: Arc<Box<dyn DagExecutor>>,
    dispatcher: Arc<TaskDispatcher>,
    active_tasks: Arc<DashMap<TaskId, Task>>,
    task_semaphore: Arc<Semaphore>,
    metrics: Arc<dyn MetricsCollector>,
    config: OrchestratorConfig,
}

/// Orchestrator configuration
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    pub max_concurrent_tasks: usize,
    pub task_timeout: Duration,
    pub retry_attempts: u32,
    pub retry_delay: Duration,
    pub execution_strategy: ExecutionStrategy,
    pub resource_constraints: ResourceConstraints,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 100,
            task_timeout: Duration::from_secs(60),
            retry_attempts: 3,
            retry_delay: Duration::from_secs(1),
            execution_strategy: ExecutionStrategy::Adaptive,
            resource_constraints: ResourceConstraints::default(),
        }
    }
}

impl MetaAIOrchestrator {
    /// Create new orchestrator instance
    pub fn new(
        agents: Vec<Box<dyn Agent>>,
        agent_selector: Box<dyn AgentSelector>,
        config: OrchestratorConfig,
    ) -> Self {
        let max_concurrent = config.max_concurrent_tasks;
        
        Self {
            agents: Arc::new(agents),
            agent_selector: Arc::new(agent_selector),
            scheduler: Arc::new(RwLock::new(Box::new(PriorityScheduler::new(1000)))),
            dag_executor: Arc::new(Box::new(DagExecutorImpl::new())),
            dispatcher: Arc::new(TaskDispatcher::new(max_concurrent)),
            active_tasks: Arc::new(DashMap::new()),
            task_semaphore: Arc::new(Semaphore::new(max_concurrent)),
            metrics: Arc::new(DefaultMetricsCollector),
            config,
        }
    }
    
    /// Start background task processing
    pub fn start(&self) -> tokio::task::JoinHandle<()> {
        let scheduler = Arc::clone(&self.scheduler);
        let dispatcher = Arc::clone(&self.dispatcher);
        let active_tasks = Arc::clone(&self.active_tasks);
        let task_semaphore = Arc::clone(&self.task_semaphore);
        let agents = Arc::clone(&self.agents);
        let agent_selector = Arc::clone(&self.agent_selector);
        let metrics = Arc::clone(&self.metrics);
        let config = self.config.clone();
        
        tokio::spawn(async move {
            loop {
                // Get next task from scheduler
                let task = {
                    let mut sched = scheduler.write();
                    sched.next_task().await
                };
                
                match task {
                    Ok(Some(task)) => {
                        let task_id = task.id;
                        active_tasks.insert(task_id, task.clone());
                        
                        // Acquire semaphore permit
                        let permit = task_semaphore.clone().acquire_owned().await.unwrap();
                        
                        // Spawn task execution
                        let agents = Arc::clone(&agents);
                        let agent_selector = Arc::clone(&agent_selector);
                        let active_tasks = Arc::clone(&active_tasks);
                        let metrics = Arc::clone(&metrics);
                        let config = config.clone();
                        
                        tokio::spawn(async move {
                            let start = Instant::now();
                            let result = execute_task_with_retry(
                                task,
                                &agents,
                                &*agent_selector,
                                &config,
                                &*metrics,
                            ).await;
                            
                            // Update task status
                            if let Some((_, mut task)) = active_tasks.remove(&task_id) {
                                task.status = match result {
                                    Ok(_) => TaskStatus::Completed,
                                    Err(_) => TaskStatus::Failed,
                                };
                                task.updated_at = chrono::Utc::now();
                            }
                            
                            // Release permit
                            drop(permit);
                            
                            // Record metrics
                            let duration = start.elapsed().as_secs_f64();
                            let status = if result.is_ok() { "success" } else { "failed" };
                            metrics.record_request("orchestrator", status, duration);
                        });
                    }
                    Ok(None) => {
                        // No tasks available, sleep briefly
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                    Err(e) => {
                        error!("Failed to get next task: {}", e);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        })
    }
}

#[async_trait]
impl Orchestrator for MetaAIOrchestrator {
    #[instrument(skip(self))]
    async fn execute_task(&self, task: Task) -> Result<TaskStatus> {
        info!("Executing task: {} ({})", task.id, task.name);
        
        // Add task to scheduler
        {
            let mut scheduler = self.scheduler.write();
            scheduler.schedule_task(task.clone()).await?;
        }
        
        // Wait for task completion (simplified for now)
        let task_id = task.id;
        let timeout = self.config.task_timeout;
        
        tokio::time::timeout(timeout, async {
            loop {
                if let Some(task) = self.active_tasks.get(&task_id) {
                    match task.status {
                        TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled => {
                            return Ok(task.status);
                        }
                        _ => {}
                    }
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
        .await
        .map_err(|_| Error::Timeout(format!("Task {} timed out", task_id)))?
    }
    
    #[instrument(skip(self))]
    async fn submit_request(&self, request: LlmRequest) -> Result<LlmResponse> {
        let start = Instant::now();
        
        // Select agent
        let agent = self.agent_selector
            .select_agent(&request, &self.agents)
            .await?;
        
        // Submit request to agent
        let response = agent.submit(request.clone()).await?;
        
        // Record metrics
        let duration = start.elapsed().as_secs_f64();
        self.metrics.record_request(agent.provider().as_str(), "success", duration);
        self.metrics.record_tokens(
            agent.provider().as_str(),
            response.usage.prompt_tokens,
            response.usage.completion_tokens,
        );
        
        Ok(response)
    }
    
    async fn get_task_status(&self, task_id: TaskId) -> Result<TaskStatus> {
        self.active_tasks
            .get(&task_id)
            .map(|task| task.status)
            .ok_or_else(|| Error::Internal(format!("Task {} not found", task_id)))
    }
    
    async fn cancel_task(&mut self, task_id: TaskId) -> Result<()> {
        if let Some(mut task) = self.active_tasks.get_mut(&task_id) {
            task.status = TaskStatus::Cancelled;
            task.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err(Error::Internal(format!("Task {} not found", task_id)))
        }
    }
    
    async fn list_active_tasks(&self) -> Result<Vec<Task>> {
        Ok(self.active_tasks
            .iter()
            .map(|entry| entry.value().clone())
            .collect())
    }
}

/// Execute task with retry logic
async fn execute_task_with_retry(
    task: Task,
    agents: &[Box<dyn Agent>],
    agent_selector: &dyn AgentSelector,
    config: &OrchestratorConfig,
    metrics: &dyn MetricsCollector,
) -> Result<LlmResponse> {
    let mut attempts = 0;
    let mut last_error = None;
    
    while attempts < config.retry_attempts {
        attempts += 1;
        
        // Create LLM request from task
        let request = LlmRequest {
            id: Uuid::new_v4(),
            task_id: task.id,
            provider: task.provider.unwrap_or(meta_ai_common::types::LlmProvider::OpenAI),
            prompt: task.description.unwrap_or_default(),
            parameters: Default::default(),
            timeout_ms: Some(config.task_timeout.as_millis() as u64),
            metadata: task.metadata.clone(),
        };
        
        // Select agent and execute
        match agent_selector.select_agent(&request, agents).await {
            Ok(agent) => {
                match agent.submit(request).await {
                    Ok(response) => return Ok(response),
                    Err(e) => {
                        warn!("Task {} attempt {} failed: {}", task.id, attempts, e);
                        last_error = Some(e);
                        
                        if attempts < config.retry_attempts {
                            tokio::time::sleep(config.retry_delay).await;
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to select agent for task {}: {}", task.id, e);
                last_error = Some(e);
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| Error::Internal("Max retry attempts reached".to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_orchestrator_creation() {
        // Test implementation
    }
}