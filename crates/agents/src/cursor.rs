//! Cursor agent stub

use async_trait::async_trait;
use meta_ai_common::{error::{Error, Result}, types::{LlmProvider, LlmRequest, LlmResponse}};
use meta_ai_core::agent::{Agent, AgentCapabilities, AgentHealth, RateLimitInfo};

pub struct CursorAgent;

impl CursorAgent {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Agent for CursorAgent {
    fn name(&self) -> &str { "Cursor" }
    fn provider(&self) -> LlmProvider { LlmProvider::Cursor }
    async fn is_available(&self) -> bool { true }
    async fn submit(&self, _request: LlmRequest) -> Result<LlmResponse> {
        Err(Error::Agent("Not implemented".to_string()))
    }
    fn capabilities(&self) -> AgentCapabilities { AgentCapabilities::default() }
    async fn health_check(&self) -> Result<AgentHealth> {
        Ok(AgentHealth {
            healthy: true, latency_ms: Some(80.0), requests_per_minute: 120.0,
            average_latency_ms: 100.0, error_rate: 0.015, last_error: None,
        })
    }
    async fn rate_limit_info(&self) -> Result<RateLimitInfo> {
        Ok(RateLimitInfo {
            requests_remaining: Some(150), requests_limit: Some(1000),
            reset_time: None, tokens_remaining: Some(70000), tokens_limit: Some(100000),
        })
    }
}