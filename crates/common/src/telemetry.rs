//! Telemetry and observability setup

use opentelemetry::{
    global,
    sdk::{
        export::trace::stdout,
        propagation::TraceContextPropagator,
        trace::{self, RandomIdGenerator, Sampler},
        Resource,
    },
    KeyValue,
};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Registry,
};
use crate::config::{ObservabilityConfig, LogFormat};
use crate::error::Result;

/// Initialize telemetry subsystems
pub fn init_telemetry(config: &ObservabilityConfig) -> Result<()> {
    // Set up tracing propagation
    global::set_text_map_propagator(TraceContextPropagator::new());
    
    // Initialize tracing subscriber
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.log_level));
    
    let fmt_layer = match config.log_format {
        LogFormat::Json => fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_list(true)
            .boxed(),
        LogFormat::Pretty => fmt::layer()
            .pretty()
            .with_span_events(FmtSpan::CLOSE)
            .boxed(),
        LogFormat::Compact => fmt::layer()
            .compact()
            .boxed(),
    };
    
    let registry = Registry::default()
        .with(env_filter)
        .with(fmt_layer);
    
    // Add OpenTelemetry layer if enabled
    if config.tracing_enabled {
        let tracer = init_tracer(config)?;
        let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
        registry.with(telemetry_layer).init();
    } else {
        registry.init();
    }
    
    Ok(())
}

/// Initialize OpenTelemetry tracer
fn init_tracer(config: &ObservabilityConfig) -> Result<impl opentelemetry::trace::Tracer> {
    let resource = Resource::new(vec![
        KeyValue::new("service.name", "meta-ai-orchestrator"),
        KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
    ]);
    
    let tracer = if let Some(endpoint) = &config.otlp_endpoint {
        // Use OTLP exporter
        opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(endpoint)
                    .with_protocol(Protocol::Grpc),
            )
            .with_trace_config(
                trace::config()
                    .with_sampler(Sampler::AlwaysOn)
                    .with_id_generator(RandomIdGenerator::default())
                    .with_max_events_per_span(64)
                    .with_max_attributes_per_span(16)
                    .with_resource(resource),
            )
            .install_batch(opentelemetry::runtime::Tokio)?
    } else {
        // Use stdout exporter for development
        opentelemetry::sdk::export::trace::stdout::new_pipeline()
            .with_trace_config(
                trace::config()
                    .with_sampler(Sampler::AlwaysOn)
                    .with_id_generator(RandomIdGenerator::default())
                    .with_resource(resource),
            )
            .install_simple()?
    };
    
    Ok(tracer)
}

/// Shutdown telemetry gracefully
pub fn shutdown_telemetry() {
    global::shutdown_tracer_provider();
}

/// Helper macros for structured logging
#[macro_export]
macro_rules! log_event {
    ($level:expr, $message:expr, $($key:expr => $value:expr),*) => {
        tracing::event!(
            $level,
            message = $message,
            $($key = ?$value,)*
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($error:expr, $($key:expr => $value:expr),*) => {
        tracing::error!(
            error = ?$error,
            error_type = std::any::type_name_of_val(&$error),
            $($key = ?$value,)*
        );
    };
}

/// Span builder for consistent span creation
pub struct SpanBuilder {
    name: String,
    attributes: Vec<KeyValue>,
}

impl SpanBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            attributes: Vec::new(),
        }
    }
    
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push(KeyValue::new(key.into(), value.into()));
        self
    }
    
    pub fn with_provider(self, provider: &str) -> Self {
        self.with_attribute("llm.provider", provider)
    }
    
    pub fn with_task_id(self, task_id: &str) -> Self {
        self.with_attribute("task.id", task_id)
    }
    
    pub fn build(self) -> tracing::Span {
        let span = tracing::info_span!(&self.name);
        for attr in self.attributes {
            span.record(attr.key.as_str(), &attr.value.as_str());
        }
        span
    }
}