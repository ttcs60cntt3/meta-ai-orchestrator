#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]

//! Agent implementations for various LLM providers

pub mod openai;
pub mod claude;
pub mod copilot;
pub mod cursor;
pub mod codewhisperer;
pub mod selector;

pub use openai::OpenAIAgent;
pub use claude::ClaudeAgent;
pub use copilot::CopilotAgent;
pub use cursor::CursorAgent;
pub use codewhisperer::CodeWhispererAgent;
pub use selector::DefaultAgentSelector;