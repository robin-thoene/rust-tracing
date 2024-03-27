use opentelemetry::{
    global,
    trace::{Span, SpanKind, TraceContextExt, Tracer},
    Context, KeyValue,
};
use opentelemetry_http::HeaderInjector;
use reqwest::{header::USER_AGENT, Client, Error, Method, Response};

pub enum UriScheme {
    Http,
    Https,
}
pub struct TraceableHttpClient {
    http_client: Client,
    host: String,
    port: Option<u32>,
    base_url: String,
}

impl TraceableHttpClient {
    pub fn new(scheme: UriScheme, host: String, port: Option<u32>) -> Self {
        let port_str = if let Some(port) = port {
            format!(":{}", port)
        } else {
            "".to_string()
        };
        let scheme_str = match scheme {
            UriScheme::Http => "http",
            UriScheme::Https => "https",
        };
        TraceableHttpClient {
            http_client: Client::new(),
            host: host.clone(),
            port,
            base_url: format!("{}://{}{}", scheme_str, host.clone(), port_str),
        }
    }

    pub async fn get(&self, rel_endpoint_url: &str) -> Result<Response, Error> {
        // Build the full URL.
        let url_full = format!("{}/{}", self.base_url, rel_endpoint_url);
        // Get the tracer.
        let tracer = global::tracer("tracing-jaeger");
        // Create a new span.
        let mut span = tracer
            .span_builder("TODO")
            .with_kind(SpanKind::Client)
            .start(&tracer);
        // Set the default span attributes using the gathered information.
        span.set_attributes(vec![
            KeyValue::new(
                opentelemetry_semantic_conventions::trace::SERVER_ADDRESS,
                self.host.clone(),
            ),
            KeyValue::new(
                opentelemetry_semantic_conventions::trace::URL_FULL,
                url_full.clone(),
            ),
        ]);
        // If a server port is specified, trace it as well.
        if let Some(p) = self.port {
            span.set_attribute(KeyValue::new(
                opentelemetry_semantic_conventions::trace::SERVER_PORT,
                p.to_string(),
            ));
        }
        // Build the request.
        let mut request = self
            .http_client
            .request(Method::GET, url_full)
            .header(USER_AGENT, "http-client")
            .build()
            .expect("Expect the request to be build for now.");
        // Inject the created span context into the request headers.
        let cx = Context::current_with_span(span);
        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&cx, &mut HeaderInjector(request.headers_mut()))
        });
        // Send the request.
        let response = self.http_client.execute(request).await;
        // Return the response.
        response
    }
}
