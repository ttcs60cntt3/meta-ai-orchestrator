#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]

//! Evaluation and quality assurance

pub mod evaluator;
pub mod fuzzer;
pub mod metrics;

pub use evaluator::MetaEvaluator;
pub use fuzzer::FuzzingEngine;
pub use metrics::EvaluationMetrics;