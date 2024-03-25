use std::io;

use reqwest::{header::USER_AGENT, Client, Error, Method, Response};

use opentelemetry::{
    global::{self, shutdown_tracer_provider},
    trace::{Span, SpanKind, TraceContextExt, TraceError, Tracer},
    Context, KeyValue,
};
use opentelemetry_http::HeaderInjector;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{propagation::TraceContextPropagator, runtime, trace, Resource};

fn init_tracer() -> Result<opentelemetry_sdk::trace::Tracer, TraceError> {
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
                "cli-client",
            )])),
        )
        .install_batch(runtime::Tokio)
}

async fn perform_request() -> Result<Response, Error> {
    // Get the tracer.
    let tracer = global::tracer("tracing-jaeger");
    // Create a new span.
    let mut span = tracer
        .span_builder("test")
        .with_kind(SpanKind::Client)
        .start(&tracer);
    // Define what resource to access.
    let server_address = "localhost";
    let route = "greet/foo/bar?test=100";
    let server_port = "5000";
    let url_full = format!("http://{}:{}/{}", server_address, server_port, route);
    // Set the default span attributes using the gathered information.
    span.set_attributes(vec![
        KeyValue::new(
            opentelemetry_semantic_conventions::trace::SERVER_ADDRESS,
            server_address,
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::trace::SERVER_PORT,
            server_port,
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::trace::URL_FULL,
            url_full.clone(),
        ),
    ]);
    let http_client = Client::new();
    let mut request = http_client
        .request(Method::GET, url_full)
        .header(USER_AGENT, "terminal")
        .build()
        .expect("Expect the request to be build.");
    let cx = Context::current_with_span(span);
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, &mut HeaderInjector(request.headers_mut()))
    });
    let response = http_client.execute(request).await;
    return response;
}

#[tokio::main]
async fn main() {
    let _tracer = init_tracer().expect("Failed to initialize tracer.");

    loop {
        let mut input = String::new();
        println!("Choose what to do:");
        println!("\"q\" - quit");
        println!("\"s\" - send request");
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        match input.trim().to_lowercase().as_str() {
            "q" => break,
            "s" => {
                println!("Sending http request ...");
                let response = perform_request().await;
                match response {
                    Ok(response) => {
                        let data = response.text().await;
                        println!("Response: {:?}", data);
                    }
                    Err(err) => println!("Error: {}", err),
                }
            }
            _ => println!("Invalid option, try again."),
        }
    }
    shutdown_tracer_provider();
}
