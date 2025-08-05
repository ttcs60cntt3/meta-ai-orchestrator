//! Copilot agent stub

use async_trait::async_trait;
use meta_ai_common::{error::{Error, Result}, types::{LlmProvider, LlmRequest, LlmResponse}};
use meta_ai_core::agent::{Agent, AgentCapabilities, AgentHealth, RateLimitInfo};

pub struct CopilotAgent;

impl CopilotAgent {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Agent for CopilotAgent {
    fn name(&self) -> &str { "Copilot" }
    fn provider(&self) -> LlmProvider { LlmProvider::Copilot }
    async fn is_available(&self) -> bool { true }
    async fn submit(&self, _request: LlmRequest) -> Result<LlmResponse> {
        Err(Error::Agent("Not implemented".to_string()))
    }
    fn capabilities(&self) -> AgentCapabilities { AgentCapabilities::default() }
    async fn health_check(&self) -> Result<AgentHealth> {
        Ok(AgentHealth {
            healthy: true, latency_ms: Some(90.0), requests_per_minute: 100.0,
            average_latency_ms: 120.0, error_rate: 0.02, last_error: None,
        })
    }
    async fn rate_limit_info(&self) -> Result<RateLimitInfo> {
        Ok(RateLimitInfo {
            requests_remaining: Some(200), requests_limit: Some(1000),
            reset_time: None, tokens_remaining: Some(60000), tokens_limit: Some(100000),
        })
    }
}