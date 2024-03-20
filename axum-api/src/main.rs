use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Path},
    http::{
        header::{HOST, USER_AGENT},
        HeaderMap, Request,
    },
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
use opentelemetry_http::HeaderExtractor;
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

async fn otel_tracing_middleware<B>(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    let tracer = global::tracer("tracing-jaeger");
    let parent_cx = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(&headers))
    });
    let mut span = tracer
        .span_builder("todo-replace")
        .with_kind(SpanKind::Server)
        .start_with_context(&tracer, &parent_cx);
    // TODO: handle tracing that is related to the request.
    // Set some of the conventional span attributes.
    span.set_attributes(vec![
        KeyValue::new(
            opentelemetry_semantic_conventions::trace::CLIENT_ADDRESS,
            addr.ip().to_string(),
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::trace::CLIENT_PORT,
            addr.port().to_string(),
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::trace::URL_PATH,
            request.uri().path().to_string(),
        ),
    ]);
    // If the scheme can be parsed, trace it.
    if let Some(scheme_str) = &request.uri().scheme_str() {
        span.set_attribute(KeyValue::new(
            opentelemetry_semantic_conventions::trace::URL_SCHEME,
            scheme_str.to_string(),
        ));
    }
    // If the request headers contain the user agent header, trace it.
    if let Some(user_agent_header) = &headers.get(USER_AGENT) {
        if let Ok(user_agent_header) = user_agent_header.to_str() {
            span.set_attribute(KeyValue::new(
                opentelemetry_semantic_conventions::trace::USER_AGENT_ORIGINAL,
                user_agent_header.to_string(),
            ));
        }
    }
    // If the request contains a query, add it to the span as well.
    if let Some(query) = request.uri().query() {
        span.set_attribute(KeyValue::new(
            opentelemetry_semantic_conventions::trace::URL_QUERY,
            query.to_string(),
        ));
    }
    // If the request contains the host header, parse the host domain/ip and the port.
    if let Some(host_header) = &headers.get(HOST) {
        if let Ok(host_header) = host_header.to_str() {
            let split: Vec<&str> = host_header.split(':').collect();
            if let Some(server_address) = split.get(0) {
                span.set_attribute(KeyValue::new(
                    opentelemetry_semantic_conventions::trace::SERVER_ADDRESS,
                    server_address.to_string(),
                ));
            }
            if let Some(server_port) = split.get(1) {
                span.set_attribute(KeyValue::new(
                    opentelemetry_semantic_conventions::trace::SERVER_PORT,
                    server_port.to_string(),
                ));
            }
        }
    }
    // Process the request.
    let response = next.run(request).await;
    // TODO: handle tracing that is related to the response.
    // End the span.
    span.end();
    // Return the response.
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
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
    shutdown_tracer_provider();
}
