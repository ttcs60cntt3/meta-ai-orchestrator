//! Common types used across the orchestrator

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Request ID type
pub type RequestId = Uuid;

/// Agent ID type
pub type AgentId = String;

/// Task ID type
pub type TaskId = Uuid;

/// LLM provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmProvider {
    OpenAI,
    Claude,
    Copilot,
    Cursor,
    CodeWhisperer,
    Local,
}

impl LlmProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenAI => "openai",
            Self::Claude => "claude",
            Self::Copilot => "copilot",
            Self::Cursor => "cursor",
            Self::CodeWhisperer => "codewhisperer",
            Self::Local => "local",
        }
    }
}

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

/// Generic metadata type
pub type Metadata = HashMap<String, serde_json::Value>;

/// Task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub name: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: Priority,
    pub provider: Option<LlmProvider>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Metadata,
}

impl Default for Task {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4(),
            name: "Unnamed Task".to_string(),
            description: None,
            status: TaskStatus::Pending,
            priority: Priority::Medium,
            provider: None,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }
}

/// LLM request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub id: RequestId,
    pub task_id: TaskId,
    pub provider: LlmProvider,
    pub prompt: String,
    pub parameters: LlmParameters,
    pub timeout_ms: Option<u64>,
    pub metadata: Metadata,
}

/// LLM parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmParameters {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub stream: bool,
}

impl Default for LlmParameters {
    fn default() -> Self {
        Self {
            temperature: Some(0.7),
            max_tokens: Some(2048),
            top_p: Some(0.9),
            frequency_penalty: None,
            presence_penalty: None,
            stop_sequences: None,
            stream: false,
        }
    }
}

/// LLM response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub request_id: RequestId,
    pub content: String,
    pub usage: TokenUsage,
    pub latency_ms: u64,
    pub provider: LlmProvider,
    pub metadata: Metadata,
}

/// Token usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Embedding vector
pub type Embedding = Vec<f32>;

/// Document for RAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub embedding: Option<Embedding>,
    pub metadata: Metadata,
    pub created_at: DateTime<Utc>,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document: Document,
    pub score: f32,
    pub metadata: Metadata,
}