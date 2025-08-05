//! Agent trait and common functionality

use async_trait::async_trait;
use meta_ai_common::{
    types::{LlmProvider, LlmRequest, LlmResponse},
    error::Result,
};

/// Agent trait for LLM providers
#[async_trait]
pub trait Agent: Send + Sync {
    /// Get agent name/provider
    fn name(&self) -> &str;
    
    /// Get provider type
    fn provider(&self) -> LlmProvider;
    
    /// Check if agent is available
    async fn is_available(&self) -> bool;
    
    /// Submit a request to the agent
    async fn submit(&self, request: LlmRequest) -> Result<LlmResponse>;
    
    /// Get agent capabilities
    fn capabilities(&self) -> AgentCapabilities;
    
    /// Health check
    async fn health_check(&self) -> Result<AgentHealth>;
    
    /// Get rate limit info
    async fn rate_limit_info(&self) -> Result<RateLimitInfo>;
}

/// Agent capabilities
#[derive(Debug, Clone)]
pub struct AgentCapabilities {
    pub max_tokens: u32,
    pub supports_streaming: bool,
    pub supports_function_calling: bool,
    pub supports_vision: bool,
    pub supports_code_execution: bool,
    pub supports_web_search: bool,
    pub context_window: u32,
    pub languages: Vec<String>,
    pub specializations: Vec<String>,
}

impl Default for AgentCapabilities {
    fn default() -> Self {
        Self {
            max_tokens: 4096,
            supports_streaming: true,
            supports_function_calling: false,
            supports_vision: false,
            supports_code_execution: false,
            supports_web_search: false,
            context_window: 128000,
            languages: vec!["en".to_string()],
            specializations: vec!["general".to_string()],
        }
    }
}

/// Agent health information
#[derive(Debug, Clone)]
pub struct AgentHealth {
    pub healthy: bool,
    pub latency_ms: Option<f64>,
    pub requests_per_minute: f64,
    pub average_latency_ms: f64,
    pub error_rate: f64,
    pub last_error: Option<String>,
}

/// Rate limit information
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub requests_remaining: Option<u32>,
    pub requests_limit: Option<u32>,
    pub reset_time: Option<chrono::DateTime<chrono::Utc>>,
    pub tokens_remaining: Option<u32>,
    pub tokens_limit: Option<u32>,
}

/// Agent selection strategy
#[derive(Debug, Clone, Copy)]
pub enum SelectionStrategy {
    /// Round-robin selection
    RoundRobin,
    /// Select based on lowest latency
    LowestLatency,
    /// Select based on highest capability match
    BestMatch,
    /// Select based on cost optimization
    CostOptimized,
    /// Random selection
    Random,
}

/// Agent selector trait
#[async_trait]
pub trait AgentSelector: Send + Sync {
    /// Select an agent for a request
    async fn select_agent(
        &self,
        request: &LlmRequest,
        agents: &[Box<dyn Agent>],
    ) -> Result<&Box<dyn Agent>>;
    
    /// Get selection strategy
    fn strategy(&self) -> SelectionStrategy;
}

/// Task routing rules
#[derive(Debug, Clone)]
pub struct RoutingRule {
    pub pattern: String,
    pub preferred_provider: LlmProvider,
    pub required_capabilities: Vec<String>,
    pub priority: i32,
}

/// Agent priority for different task types
#[derive(Debug, Clone)]
pub struct AgentPriority {
    pub provider: LlmProvider,
    pub task_type: TaskType,
    pub priority: i32,
}

/// Task types for routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskType {
    Reasoning,
    CodeGeneration,
    Documentation,
    Analysis,
    Creative,
    Translation,
    Summarization,
    QA,
}

impl TaskType {
    /// Get preferred providers for task type
    pub fn preferred_providers(&self) -> Vec<LlmProvider> {
        match self {
            Self::Reasoning => vec![LlmProvider::Claude, LlmProvider::OpenAI],
            Self::CodeGeneration => vec![LlmProvider::OpenAI, LlmProvider::Cursor, LlmProvider::Copilot],
            Self::Documentation => vec![LlmProvider::Claude, LlmProvider::OpenAI],
            Self::Analysis => vec![LlmProvider::Claude, LlmProvider::OpenAI],
            Self::Creative => vec![LlmProvider::Claude, LlmProvider::OpenAI],
            Self::Translation => vec![LlmProvider::OpenAI, LlmProvider::Claude],
            Self::Summarization => vec![LlmProvider::Claude, LlmProvider::OpenAI],
            Self::QA => vec![LlmProvider::Claude, LlmProvider::OpenAI],
        }
    }
}