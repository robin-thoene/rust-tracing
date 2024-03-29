use opentelemetry::{
    global::{self},
    trace::TraceError,
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{propagation::TraceContextPropagator, runtime, trace, Resource};

pub fn init_tracer(service_name: String) -> Result<opentelemetry_sdk::trace::Tracer, TraceError> {
    global::set_text_map_propagator(TraceContextPropagator::new());
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(
            trace::config().with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                service_name,
            )])),
        )
        .install_batch(runtime::Tokio)
}
