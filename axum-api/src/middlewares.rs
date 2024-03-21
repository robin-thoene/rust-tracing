use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, MatchedPath},
    headers::{Host, UserAgent},
    http::{HeaderMap, Request},
    middleware::Next,
    response::Response,
    TypedHeader,
};
use opentelemetry::{
    global,
    trace::{Span, SpanKind, Tracer},
    KeyValue,
};
use opentelemetry_http::HeaderExtractor;

pub async fn otel_tracing_middleware<B>(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    TypedHeader(host): TypedHeader<Host>,
    headers: HeaderMap,
    matched_path: MatchedPath,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    let tracer = global::tracer("tracing-jaeger");
    let parent_cx = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(&headers))
    });
    let mut span = tracer
        .span_builder(format!(
            "{} {}",
            request.method().to_string(),
            matched_path.as_str().to_string()
        ))
        .with_kind(SpanKind::Server)
        .start_with_context(&tracer, &parent_cx);
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
        KeyValue::new(
            opentelemetry_semantic_conventions::trace::HTTP_REQUEST_METHOD,
            request.method().to_string(),
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::trace::USER_AGENT_ORIGINAL,
            user_agent.to_string(),
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::trace::SERVER_ADDRESS,
            host.hostname().to_string(),
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::trace::HTTP_ROUTE,
            matched_path.as_str().to_string(),
        ),
    ]);
    // If the host port can be determined, trace it.
    if let Some(port) = host.port() {
        span.set_attribute(KeyValue::new(
            opentelemetry_semantic_conventions::trace::SERVER_PORT,
            port.to_string(),
        ));
    }
    // If the scheme can be parsed, trace it.
    if let Some(scheme_str) = &request.uri().scheme_str() {
        span.set_attribute(KeyValue::new(
            opentelemetry_semantic_conventions::trace::URL_SCHEME,
            scheme_str.to_string(),
        ));
    }
    // If the request contains a query, add it to the span as well.
    if let Some(query) = request.uri().query() {
        span.set_attribute(KeyValue::new(
            opentelemetry_semantic_conventions::trace::URL_QUERY,
            query.to_string(),
        ));
    }
    // Process the request.
    let response = next.run(request).await;
    // Add response related data to the span.
    span.set_attribute(KeyValue::new(
        opentelemetry_semantic_conventions::trace::HTTP_RESPONSE_STATUS_CODE,
        response.status().as_str().to_string(),
    ));
    // End the span.
    span.end();
    // Return the response.
    response
}
