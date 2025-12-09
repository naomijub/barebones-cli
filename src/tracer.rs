use std::{error::Error, time::Duration};

use opentelemetry::{KeyValue, global, trace::TracerProvider};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::{
    Resource,
    logs::SdkLoggerProvider,
    metrics::{PeriodicReader, SdkMeterProvider, Temporality},
    trace::SdkTracerProvider,
};
use tracing::instrument::WithSubscriber;
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::APP_NAME;

/// Telemetry configuration options
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub service_name: String,
    pub use_stdout: bool,
    pub use_otlp: bool,
    pub otlp_endpoint: Option<String>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            service_name: APP_NAME.to_string(),
            use_stdout: true,
            use_otlp: false,
            otlp_endpoint: None,
        }
    }
}

/// Telemetry shutdown handlers
pub struct TelemetryShutdown {
    meter_provider: SdkMeterProvider,
    tracer_provider: SdkTracerProvider,
    logger_provider: SdkLoggerProvider,
}

impl TelemetryShutdown {
    pub fn shutdown(self) -> Result<(), Box<dyn Error>> {
        self.tracer_provider.shutdown()?;
        self.logger_provider.shutdown()?;
        self.meter_provider.shutdown()?;
        Ok(())
    }
}

///Telemetry stack initializer (logs, traces, metrics)
pub fn init_telemetry(
    config: TelemetryConfig,
    logger: fmt::SubscriberBuilder<
        fmt::format::DefaultFields,
        fmt::format::Format<fmt::format::Compact, ()>,
    >,
) -> Result<TelemetryShutdown, Box<dyn Error>> {
    let resource = Resource::builder()
        .with_service_name(config.service_name.clone())
        .with_attributes([
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            KeyValue::new("service.OS", std::env::consts::OS),
        ])
        .build();

    let (tracer_provider, logger_provider) =
        init_tracing_logger(&config, resource.clone(), logger)?;
    let meter_provider = init_metrics(&config, resource);

    Ok(TelemetryShutdown {
        meter_provider,
        tracer_provider,
        logger_provider,
    })
}

/// Initialize metrics with stdout (and/or OTLP - NOT IMPLEMENTED)
fn init_metrics(config: &TelemetryConfig, resource: Resource) -> SdkMeterProvider {
    let exporter = opentelemetry_stdout::MetricExporterBuilder::default()
        .with_temporality(Temporality::Cumulative)
        .build();
    let reader = PeriodicReader::builder(exporter)
        .with_interval(Duration::from_secs(1)) // Export every second
        .build();
    let mut builder = SdkMeterProvider::builder()
        .with_resource(resource)
        .with_reader(reader);

    if config.use_stdout {
        let exporter = opentelemetry_stdout::MetricExporterBuilder::default()
            .with_temporality(Temporality::Cumulative)
            .build();
        builder = builder.with_periodic_exporter(exporter);
    }

    if config.use_otlp {
        todo!("OTLP metrics exporter will be configured here");
    }

    let provider = builder.build();
    global::set_meter_provider(provider.clone());

    provider
}

/// Initialize tracing with stdout (and/or OTLP - NOT IMPLEMENTED) exporters
fn init_tracing_logger(
    config: &TelemetryConfig,
    resource: Resource,
    logger: fmt::SubscriberBuilder<
        fmt::format::DefaultFields,
        fmt::format::Format<fmt::format::Compact, ()>,
    >,
) -> Result<(SdkTracerProvider, SdkLoggerProvider), Box<dyn Error>> {
    let mut log_builder = SdkLoggerProvider::builder().with_resource(resource.clone());
    let mut tracing_builder = SdkTracerProvider::builder().with_resource(resource);

    if config.use_stdout {
        let exporter = opentelemetry_stdout::SpanExporter::default();
        tracing_builder = tracing_builder.with_simple_exporter(exporter);
        let exporter = opentelemetry_stdout::LogExporter::default();
        log_builder = log_builder.with_simple_exporter(exporter);
    }

    if config.use_otlp {
        todo!("OTLP tracing exporter will be configured here");
    }

    // === TRACING ===
    let tracing_provider = tracing_builder.build();
    let tracer = tracing_provider.tracer(config.service_name.clone());

    // Create tracing layer
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Configure filters
    let tracing_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"))
        .add_directive("opentelemetry=debug".parse()?)
        .add_directive("hyper=off".parse()?)
        .add_directive("tonic=off".parse()?)
        .add_directive("h2=off".parse()?)
        .add_directive("reqwest=off".parse()?);

    // Create fmt layer for console output
    let fmt_layer = tracing_subscriber::fmt::layer().with_filter(tracing_filter);

    // === LOG ===
    let log_provider = log_builder.build();

    // Add OpenTelemetry log appender to existing tracing subscriber
    let log_filter = EnvFilter::new("info")
        .add_directive("reqwest=off".parse()?)
        .add_directive("hyper=off".parse()?)
        .add_directive("tonic=off".parse()?);

    let otel_layer = OpenTelemetryTracingBridge::new(&log_provider).with_filter(log_filter);

    let dispatcher = tracing_subscriber::registry()
        .with(fmt_layer)
        .with(telemetry_layer)
        .with(otel_layer)
        .with_subscriber(logger);
    let dispatcher = dispatcher.dispatcher();
    dispatcher.clone().init();

    Ok((tracing_provider, log_provider))
}
