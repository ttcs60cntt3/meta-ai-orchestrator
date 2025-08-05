//! Configuration management with hot-reload support

use serde::{Deserialize, Serialize};
use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use std::path::Path;
use std::collections::HashMap;
use crate::types::LlmProvider;
use secrecy::{Secret, ExposeSecret};

/// Main configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub orchestrator: OrchestratorConfig,
    pub agents: HashMap<LlmProvider, AgentConfig>,
    pub rag: RagConfig,
    pub evaluation: EvaluationConfig,
    pub observability: ObservabilityConfig,
    pub security: SecurityConfig,
}

/// Server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub max_connections: u32,
    pub request_timeout_ms: u64,
}

/// Orchestrator configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrchestratorConfig {
    pub max_concurrent_tasks: usize,
    pub task_queue_size: usize,
    pub default_timeout_ms: u64,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
    pub dag_max_depth: usize,
}

/// Agent configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentConfig {
    pub api_key: Secret<String>,
    pub base_url: String,
    pub model: String,
    pub max_retries: u32,
    pub timeout_ms: u64,
    pub rate_limit_rpm: Option<u32>,
    pub enabled: bool,
}

/// RAG configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RagConfig {
    pub qdrant_url: String,
    pub collection_name: String,
    pub embedding_model: String,
    pub embedding_dimension: usize,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub top_k: usize,
}

/// Evaluation configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EvaluationConfig {
    pub accuracy_threshold: f64,
    pub bug_rate_threshold: f64,
    pub self_check_interval_ms: u64,
    pub fuzzing_enabled: bool,
    pub fuzzing_iterations: u32,
}

/// Observability configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ObservabilityConfig {
    pub metrics_enabled: bool,
    pub metrics_port: u16,
    pub tracing_enabled: bool,
    pub otlp_endpoint: Option<String>,
    pub log_level: String,
    pub log_format: LogFormat,
}

/// Security configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    pub auth_enabled: bool,
    pub api_key_header: String,
    pub rate_limit_enabled: bool,
    pub rate_limit_requests_per_minute: u32,
    pub request_signature_validation: bool,
    pub sandbox_enabled: bool,
}

/// Log format
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

impl Config {
    /// Load configuration from files and environment
    pub fn load() -> Result<Self, ConfigError> {
        let mut builder = ConfigBuilder::builder()
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 8080)?
            .set_default("server.request_timeout_ms", 30000)?
            .set_default("orchestrator.max_concurrent_tasks", 100)?
            .set_default("orchestrator.task_queue_size", 1000)?
            .set_default("evaluation.accuracy_threshold", 0.9999)?
            .set_default("evaluation.bug_rate_threshold", 0.0005)?
            .set_default("security.rate_limit_requests_per_minute", 60)?;
        
        // Load from config file if exists
        if Path::new("config.toml").exists() {
            builder = builder.add_source(File::with_name("config"));
        }
        
        // Override with environment variables
        builder = builder.add_source(
            Environment::with_prefix("META_AI")
                .separator("__")
                .try_parsing(true)
        );
        
        builder.build()?.try_deserialize()
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // Validate accuracy threshold
        if self.evaluation.accuracy_threshold < 0.0 || self.evaluation.accuracy_threshold > 1.0 {
            errors.push("Accuracy threshold must be between 0.0 and 1.0".to_string());
        }
        
        // Validate bug rate threshold
        if self.evaluation.bug_rate_threshold < 0.0 || self.evaluation.bug_rate_threshold > 1.0 {
            errors.push("Bug rate threshold must be between 0.0 and 1.0".to_string());
        }
        
        // Validate at least one agent is enabled
        let enabled_agents = self.agents.values().filter(|a| a.enabled).count();
        if enabled_agents == 0 {
            errors.push("At least one agent must be enabled".to_string());
        }
        
        // Validate server configuration
        if self.server.port == 0 {
            errors.push("Server port must be greater than 0".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: None,
                max_connections: 1000,
                request_timeout_ms: 30000,
            },
            orchestrator: OrchestratorConfig {
                max_concurrent_tasks: 100,
                task_queue_size: 1000,
                default_timeout_ms: 60000,
                retry_attempts: 3,
                retry_delay_ms: 1000,
                dag_max_depth: 10,
            },
            agents: HashMap::new(),
            rag: RagConfig {
                qdrant_url: "http://localhost:6333".to_string(),
                collection_name: "meta_ai_docs".to_string(),
                embedding_model: "BAAI/bge-base-en-v1.5".to_string(),
                embedding_dimension: 768,
                chunk_size: 512,
                chunk_overlap: 128,
                top_k: 5,
            },
            evaluation: EvaluationConfig {
                accuracy_threshold: 0.9999,
                bug_rate_threshold: 0.0005,
                self_check_interval_ms: 60000,
                fuzzing_enabled: true,
                fuzzing_iterations: 100,
            },
            observability: ObservabilityConfig {
                metrics_enabled: true,
                metrics_port: 9090,
                tracing_enabled: true,
                otlp_endpoint: None,
                log_level: "info".to_string(),
                log_format: LogFormat::Json,
            },
            security: SecurityConfig {
                auth_enabled: true,
                api_key_header: "X-API-Key".to_string(),
                rate_limit_enabled: true,
                rate_limit_requests_per_minute: 60,
                request_signature_validation: true,
                sandbox_enabled: true,
            },
        }
    }
}