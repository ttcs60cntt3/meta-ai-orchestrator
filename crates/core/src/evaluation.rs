//! Evaluation and validation subsystem

use async_trait::async_trait;
use meta_ai_common::{
    types::{Task, TaskStatus, LlmResponse},
    error::Result,
};

/// Evaluator trait for model validation and quality assurance
#[async_trait]
pub trait Evaluator: Send + Sync {
    /// Pre-task validation
    async fn pre_task_validation(&self, task: &Task) -> Result<ValidationResult>;
    
    /// Post-task validation
    async fn post_task_validation(&self, task: &Task, status: &TaskStatus) -> Result<ValidationResult>;
    
    /// Validate LLM response
    async fn validate_response(&self, response: &LlmResponse) -> Result<ValidationResult>;
    
    /// Get current accuracy
    async fn get_accuracy(&self) -> Result<f64>;
    
    /// Get current bug rate
    async fn get_bug_rate(&self) -> Result<f64>;
    
    /// Run self-check cycle
    async fn self_check(&self) -> Result<SelfCheckResult>;
    
    /// Run fuzzing tests
    async fn fuzz_test(&self, iterations: u32) -> Result<FuzzingResult>;
    
    /// Check for model drift
    async fn check_drift(&self) -> Result<DriftAnalysis>;
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub score: f64,
    pub issues: Vec<ValidationIssue>,
    pub metadata: meta_ai_common::types::Metadata,
}

/// Validation issue
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub description: String,
    pub suggestion: Option<String>,
}

/// Issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Issue category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueCategory {
    InputValidation,
    OutputQuality,
    SafetyViolation,
    PerformanceIssue,
    ConsistencyError,
    FormatError,
}

/// Self-check result
#[derive(Debug)]
pub struct SelfCheckResult {
    pub passed: bool,
    pub accuracy: f64,
    pub bug_rate: f64,
    pub tests_run: u32,
    pub tests_passed: u32,
    pub duration_ms: u64,
    pub issues: Vec<ValidationIssue>,
}

/// Fuzzing test result
#[derive(Debug)]
pub struct FuzzingResult {
    pub iterations: u32,
    pub failures: u32,
    pub crash_count: u32,
    pub timeout_count: u32,
    pub unique_errors: Vec<String>,
    pub coverage_percent: f64,
}

/// Drift analysis result
#[derive(Debug)]
pub struct DriftAnalysis {
    pub drift_detected: bool,
    pub drift_score: f64,
    pub baseline_accuracy: f64,
    pub current_accuracy: f64,
    pub performance_change: f64,
    pub recommendations: Vec<String>,
}

/// Test case for evaluation
#[derive(Debug, Clone)]
pub struct TestCase {
    pub id: String,
    pub name: String,
    pub input: String,
    pub expected_output: Option<String>,
    pub validation_criteria: Vec<ValidationCriterion>,
    pub tags: Vec<String>,
}

/// Validation criterion
#[derive(Debug, Clone)]
pub enum ValidationCriterion {
    /// Exact match
    ExactMatch(String),
    /// Contains substring
    Contains(String),
    /// Matches regex
    Regex(String),
    /// Custom validation function
    Custom(String),
    /// Semantic similarity threshold
    SemanticSimilarity(f64),
    /// Response time limit
    ResponseTime(u64),
    /// Token count limit
    TokenLimit(u32),
}

/// Benchmark suite for evaluation
#[derive(Debug)]
pub struct BenchmarkSuite {
    pub name: String,
    pub test_cases: Vec<TestCase>,
    pub pass_threshold: f64,
    pub time_limit_ms: Option<u64>,
}

/// Evaluation metrics
#[derive(Debug, Default, Clone)]
pub struct EvaluationMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub average_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub bug_rate: f64,
    pub error_rate: f64,
}

/// Quality gate definition
#[derive(Debug, Clone)]
pub struct QualityGate {
    pub name: String,
    pub metric: QualityMetric,
    pub operator: ComparisonOperator,
    pub threshold: f64,
    pub action: GateAction,
}

/// Quality metrics
#[derive(Debug, Clone, Copy)]
pub enum QualityMetric {
    Accuracy,
    BugRate,
    Latency,
    ErrorRate,
    TokenUsage,
    Cost,
}

/// Comparison operators
#[derive(Debug, Clone, Copy)]
pub enum ComparisonOperator {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
    NotEqual,
}

/// Gate actions
#[derive(Debug, Clone, Copy)]
pub enum GateAction {
    Warn,
    Block,
    Retry,
    Fallback,
}

/// A/B test configuration
#[derive(Debug, Clone)]
pub struct AbTestConfig {
    pub name: String,
    pub control_provider: meta_ai_common::types::LlmProvider,
    pub experiment_provider: meta_ai_common::types::LlmProvider,
    pub traffic_split: f64,
    pub minimum_samples: u32,
    pub metrics_to_track: Vec<QualityMetric>,
}