//! Orchestrator trait for task scheduling and DAG execution

use async_trait::async_trait;
use meta_ai_common::{
    types::{Task, TaskId, TaskStatus, LlmRequest, LlmResponse},
    error::Result,
};
use std::collections::HashMap;

/// Orchestrator trait for managing task execution
#[async_trait]
pub trait Orchestrator: Send + Sync {
    /// Execute a task
    async fn execute_task(&self, task: Task) -> Result<TaskStatus>;
    
    /// Submit a direct LLM request
    async fn submit_request(&self, request: LlmRequest) -> Result<LlmResponse>;
    
    /// Get task status
    async fn get_task_status(&self, task_id: TaskId) -> Result<TaskStatus>;
    
    /// Cancel a task
    async fn cancel_task(&mut self, task_id: TaskId) -> Result<()>;
    
    /// List active tasks
    async fn list_active_tasks(&self) -> Result<Vec<Task>>;
}

/// DAG (Directed Acyclic Graph) for task dependencies
#[derive(Debug, Clone)]
pub struct TaskDag {
    pub nodes: HashMap<TaskId, DagNode>,
    pub edges: Vec<DagEdge>,
}

/// DAG node representing a task
#[derive(Debug, Clone)]
pub struct DagNode {
    pub task_id: TaskId,
    pub task: Task,
    pub dependencies: Vec<TaskId>,
    pub dependents: Vec<TaskId>,
    pub status: TaskStatus,
}

/// DAG edge representing dependency
#[derive(Debug, Clone)]
pub struct DagEdge {
    pub from: TaskId,
    pub to: TaskId,
    pub condition: Option<EdgeCondition>,
}

/// Edge condition for conditional execution
#[derive(Debug, Clone)]
pub enum EdgeCondition {
    /// Execute if source task succeeded
    OnSuccess,
    /// Execute if source task failed
    OnFailure,
    /// Execute regardless of source task status
    Always,
    /// Custom condition
    Custom(String),
}

/// DAG validation result
#[derive(Debug)]
pub struct DagValidation {
    pub valid: bool,
    pub has_cycles: bool,
    pub unreachable_nodes: Vec<TaskId>,
    pub max_depth: usize,
}

/// DAG executor trait
#[async_trait]
pub trait DagExecutor: Send + Sync {
    /// Execute a DAG
    async fn execute_dag(&self, dag: &TaskDag) -> Result<DagExecutionResult>;
    
    /// Validate a DAG
    fn validate_dag(&self, dag: &TaskDag) -> DagValidation;
    
    /// Get execution order for DAG
    fn topological_sort(&self, dag: &TaskDag) -> Result<Vec<TaskId>>;
}

/// DAG execution result
#[derive(Debug)]
pub struct DagExecutionResult {
    pub completed_tasks: Vec<TaskId>,
    pub failed_tasks: Vec<TaskId>,
    pub skipped_tasks: Vec<TaskId>,
    pub total_duration_ms: u64,
}

/// Task scheduler trait
#[async_trait]
pub trait TaskScheduler: Send + Sync {
    /// Schedule a task for execution
    async fn schedule_task(&mut self, task: Task) -> Result<()>;
    
    /// Get next task to execute
    async fn next_task(&mut self) -> Result<Option<Task>>;
    
    /// Return a task to the queue (e.g., after failure)
    async fn requeue_task(&mut self, task: Task) -> Result<()>;
    
    /// Get queue statistics
    async fn queue_stats(&self) -> QueueStats;
}

/// Queue statistics
#[derive(Debug, Clone)]
pub struct QueueStats {
    pub pending_tasks: usize,
    pub running_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub average_wait_time_ms: f64,
    pub average_execution_time_ms: f64,
}

/// Execution strategy for orchestration
#[derive(Debug, Clone, Copy)]
pub enum ExecutionStrategy {
    /// Execute tasks sequentially
    Sequential,
    /// Execute tasks in parallel when possible
    Parallel,
    /// Execute with adaptive parallelism based on resources
    Adaptive,
    /// Execute with priority-based scheduling
    Priority,
}

/// Resource constraints for execution
#[derive(Debug, Clone)]
pub struct ResourceConstraints {
    pub max_concurrent_tasks: usize,
    pub max_memory_mb: usize,
    pub max_cpu_percent: f32,
    pub max_tokens_per_minute: u32,
}

impl Default for ResourceConstraints {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 100,
            max_memory_mb: 8192,
            max_cpu_percent: 80.0,
            max_tokens_per_minute: 1_000_000,
        }
    }
}