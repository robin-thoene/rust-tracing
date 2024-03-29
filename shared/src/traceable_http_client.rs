use opentelemetry::{
    global,
    trace::{Span, SpanKind, TraceContextExt, Tracer},
    Context, KeyValue,
};
use opentelemetry_http::HeaderInjector;
use reqwest::{header::USER_AGENT, Client, Error, Method, Response};

pub enum UriScheme {
    Http,
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
        };
        TraceableHttpClient {
            http_client: Client::new(),
            host: host.clone(),
            port,
            base_url: format!("{}://{}{}", scheme_str, host.clone(), port_str),
        }
    }

    pub async fn get(
        &self,
        rel_endpoint_url: &str,
        tracing_context: Option<Context>,
    ) -> Result<Response, Error> {
        // Build the full URL.
        let url_full = format!("{}/{}", self.base_url, rel_endpoint_url);
        // Get the tracer.
        let tracer = global::tracer("tracing-jaeger");
        // Create a new span.
        let mut span = if let Some(context) = tracing_context {
            tracer
                .span_builder(Method::GET.as_str())
                .with_kind(SpanKind::Client)
                .start_with_context(&tracer, &context)
        } else {
            tracer
                .span_builder(Method::GET.as_str())
                .with_kind(SpanKind::Client)
                .start(&tracer)
        };
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
            KeyValue::new(
                opentelemetry_semantic_conventions::trace::HTTP_REQUEST_METHOD,
                Method::GET.as_str(),
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
            .build()?;
        // Inject the created span context into the request headers.
        let cx = Context::current_with_span(span);
        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&cx, &mut HeaderInjector(request.headers_mut()))
        });
        // Send the request.
        let response = self.http_client.execute(request).await;
        match response {
            Ok(ref response) => {
                // If the request was send and a response was retrieved successful, trace the
                // response status code.
                cx.span().set_attribute(KeyValue::new(
                    opentelemetry_semantic_conventions::trace::HTTP_RESPONSE_STATUS_CODE,
                    response.status().as_str().to_string(),
                ));
            }
            _ => {}
        }
        // End the span.
        cx.span().end();
        // Return the response.
        response
    }
}
