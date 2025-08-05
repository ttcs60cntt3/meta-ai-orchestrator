//! Error types for Meta-AI Orchestrator

use thiserror::Error;

/// Main error type for the orchestrator
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Agent-related errors
    #[error("Agent error: {0}")]
    Agent(String),
    
    /// RAG system errors
    #[error("RAG error: {0}")]
    Rag(String),
    
    /// Orchestration errors
    #[error("Orchestration error: {0}")]
    Orchestration(String),
    
    /// Evaluation errors
    #[error("Evaluation error: {0}")]
    Evaluation(String),
    
    /// Timeout errors
    #[error("Operation timed out: {0}")]
    Timeout(String),
    
    /// Rate limiting errors
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),
    
    /// Authentication errors
    #[error("Authentication failed: {0}")]
    Auth(String),
    
    /// Validation errors
    #[error("Validation failed: {0}")]
    Validation(String),
    
    /// Network errors
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Generic errors
    #[error("Internal error: {0}")]
    Internal(String),
    
    /// Unknown errors
    #[error("Unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Error::Network(_) | Error::Timeout(_) | Error::RateLimit(_)
        )
    }
    
    /// Get error severity for metrics
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Error::Config(_) | Error::Auth(_) => ErrorSeverity::Critical,
            Error::Agent(_) | Error::Orchestration(_) => ErrorSeverity::High,
            Error::Timeout(_) | Error::RateLimit(_) => ErrorSeverity::Medium,
            Error::Validation(_) => ErrorSeverity::Low,
            _ => ErrorSeverity::Medium,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}