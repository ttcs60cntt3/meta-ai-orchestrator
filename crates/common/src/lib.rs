#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

//! Common types and utilities for Meta-AI Orchestrator
//!
//! This crate provides shared functionality across all orchestrator components.

pub mod error;
pub mod metrics;
pub mod types;
pub mod config;
pub mod telemetry;

pub use error::{Error, Result};
pub use types::*;
pub use config::Config;

/// Re-export commonly used external types
pub use uuid::Uuid;
pub use chrono::{DateTime, Utc};
pub use serde::{Deserialize, Serialize};