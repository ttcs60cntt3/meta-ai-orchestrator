//! Metrics collection and reporting

use prometheus::{
    register_counter_vec, register_histogram_vec, register_gauge_vec,
    CounterVec, HistogramVec, GaugeVec, Registry,
};
use once_cell::sync::Lazy;
use crate::error::Result;

/// Global metrics registry
pub static METRICS_REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

/// Request counter
pub static REQUEST_COUNTER: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "meta_ai_requests_total",
        "Total number of requests",
        &["provider", "status", "task_type"]
    ).unwrap()
});

/// Request duration histogram
pub static REQUEST_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "meta_ai_request_duration_seconds",
        "Request duration in seconds",
        &["provider", "task_type"],
        vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0]
    ).unwrap()
});

/// Token usage counter
pub static TOKEN_USAGE: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "meta_ai_tokens_total",
        "Total tokens used",
        &["provider", "token_type"]
    ).unwrap()
});

/// Active tasks gauge
pub static ACTIVE_TASKS: Lazy<GaugeVec> = Lazy::new(|| {
    register_gauge_vec!(
        "meta_ai_active_tasks",
        "Number of active tasks",
        &["provider", "priority"]
    ).unwrap()
});

/// Error counter
pub static ERROR_COUNTER: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "meta_ai_errors_total",
        "Total number of errors",
        &["error_type", "severity", "provider"]
    ).unwrap()
});

/// Accuracy gauge (for evaluation)
pub static ACCURACY_GAUGE: Lazy<GaugeVec> = Lazy::new(|| {
    register_gauge_vec!(
        "meta_ai_accuracy",
        "Model accuracy percentage",
        &["provider", "task_type"]
    ).unwrap()
});

/// Bug rate gauge
pub static BUG_RATE_GAUGE: Lazy<GaugeVec> = Lazy::new(|| {
    register_gauge_vec!(
        "meta_ai_bug_rate",
        "Bug rate per 1000 requests",
        &["provider", "error_type"]
    ).unwrap()
});

/// Initialize all metrics
pub fn init_metrics() -> Result<()> {
    METRICS_REGISTRY.register(Box::new(REQUEST_COUNTER.clone()))?;
    METRICS_REGISTRY.register(Box::new(REQUEST_DURATION.clone()))?;
    METRICS_REGISTRY.register(Box::new(TOKEN_USAGE.clone()))?;
    METRICS_REGISTRY.register(Box::new(ACTIVE_TASKS.clone()))?;
    METRICS_REGISTRY.register(Box::new(ERROR_COUNTER.clone()))?;
    METRICS_REGISTRY.register(Box::new(ACCURACY_GAUGE.clone()))?;
    METRICS_REGISTRY.register(Box::new(BUG_RATE_GAUGE.clone()))?;
    
    Ok(())
}

/// Metrics collector trait
pub trait MetricsCollector {
    /// Record request metrics
    fn record_request(&self, provider: &str, status: &str, duration_secs: f64);
    
    /// Record token usage
    fn record_tokens(&self, provider: &str, prompt_tokens: u32, completion_tokens: u32);
    
    /// Record error
    fn record_error(&self, error_type: &str, severity: &str, provider: &str);
    
    /// Update accuracy
    fn update_accuracy(&self, provider: &str, task_type: &str, accuracy: f64);
    
    /// Update bug rate
    fn update_bug_rate(&self, provider: &str, error_type: &str, rate: f64);
}

/// Default metrics collector implementation
#[derive(Clone)]
pub struct DefaultMetricsCollector;

impl MetricsCollector for DefaultMetricsCollector {
    fn record_request(&self, provider: &str, status: &str, duration_secs: f64) {
        REQUEST_COUNTER
            .with_label_values(&[provider, status, "default"])
            .inc();
        
        REQUEST_DURATION
            .with_label_values(&[provider, "default"])
            .observe(duration_secs);
    }
    
    fn record_tokens(&self, provider: &str, prompt_tokens: u32, completion_tokens: u32) {
        TOKEN_USAGE
            .with_label_values(&[provider, "prompt"])
            .inc_by(f64::from(prompt_tokens));
        
        TOKEN_USAGE
            .with_label_values(&[provider, "completion"])
            .inc_by(f64::from(completion_tokens));
    }
    
    fn record_error(&self, error_type: &str, severity: &str, provider: &str) {
        ERROR_COUNTER
            .with_label_values(&[error_type, severity, provider])
            .inc();
    }
    
    fn update_accuracy(&self, provider: &str, task_type: &str, accuracy: f64) {
        ACCURACY_GAUGE
            .with_label_values(&[provider, task_type])
            .set(accuracy);
    }
    
    fn update_bug_rate(&self, provider: &str, error_type: &str, rate: f64) {
        BUG_RATE_GAUGE
            .with_label_values(&[provider, error_type])
            .set(rate);
    }
}