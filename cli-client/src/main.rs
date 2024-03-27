use std::io;

use reqwest::{Error, Response};

use opentelemetry::{
    global::{self, shutdown_tracer_provider},
    trace::TraceError,
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{propagation::TraceContextPropagator, runtime, trace, Resource};

mod http_client;

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
    let http_client = http_client::TraceableHttpClient::new(
        http_client::UriScheme::Http,
        String::from("localhost"),
        Some(5000),
    );
    return http_client.get("greet/foo/bar?q=test").await;
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
