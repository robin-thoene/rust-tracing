use std::io;

use reqwest::{Error, Response};

use opentelemetry::global::shutdown_tracer_provider;
use shared::{traceable_http_client, tracer::init_tracer};

async fn perform_request() -> Result<Response, Error> {
    let http_client = traceable_http_client::TraceableHttpClient::new(
        traceable_http_client::UriScheme::Http,
        "localhost".to_string(),
        Some(5000),
    );
    http_client.get("downstream-api-status?q=foo").await
}

#[tokio::main]
async fn main() {
    let _tracer = init_tracer("cli-client".to_string()).expect("Failed to initialize tracer.");

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
