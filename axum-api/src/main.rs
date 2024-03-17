use std::net::SocketAddr;

use axum::{
    extract::Path,
    http::Request,
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use opentelemetry::{
    global::{self, shutdown_tracer_provider},
    trace::{Span, SpanKind, TraceError, Tracer},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{runtime, trace, Resource};

fn init_tracer() -> Result<opentelemetry_sdk::trace::Tracer, TraceError> {
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
                "axum-api",
            )])),
        )
        .install_batch(runtime::Tokio)
}

async fn otel_tracing_middleware<B>(request: Request<B>, next: Next<B>) -> Response {
    let tracer = global::tracer("tracing-jaeger");
    let mut span = tracer
        .span_builder("todo-replace")
        .with_kind(SpanKind::Server)
        .start(&tracer);
    // TODO: handle tracing that is related to the request.
    let response = next.run(request).await;
    span.end();
    // TODO: handle tracing that is related to the response.
    response
}

async fn greet_handler(Path((first_name, last_name)): Path<(String, String)>) -> String {
    format!(
        "Hello {} {}, the axum-api server greets you!",
        first_name, last_name
    )
}

#[tokio::main]
async fn main() {
    let _tracer = init_tracer().expect("Failed to initialize tracer.");

    let app = Router::new()
        .route("/greet/:first_name/:last_name", get(greet_handler))
        .layer(middleware::from_fn(otel_tracing_middleware));
    let addr = SocketAddr::from(([127, 0, 0, 1], 5000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    shutdown_tracer_provider();
}
