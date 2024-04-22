use core::panic;
use std::io;

use reqwest::{Error, Response};

use opentelemetry::global::shutdown_tracer_provider;
use shared::{traceable_http_client, tracer::init_tracer};

enum Api {
    AxumApi,
    AxumDownstreamApi,
    DotnetApi,
}

async fn perform_request(api: &Api, relative_path: &str) -> Result<Response, Error> {
    let port = match api {
        Api::AxumApi => 5000,
        Api::AxumDownstreamApi => 9000,
        Api::DotnetApi => 5240,
    };
    let http_client = traceable_http_client::TraceableHttpClient::new(
        traceable_http_client::UriScheme::Http,
        "localhost".to_string(),
        Some(port),
    );
    http_client.get(relative_path).await
}

#[tokio::main]
async fn main() {
    let _tracer = init_tracer("cli-client".to_string()).expect("Failed to initialize tracer.");

    loop {
        let mut input = String::new();
        println!("Choose what to do:");
        println!("\"q\" - quit");
        println!("\"1\" - send request -> axum-api -> downstream-api-status");
        println!("\"2\" - send request -> axum-api -> greet/foo/bar");
        println!("\"3\" - send request -> axum-downstream-api -> status");

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        match input.trim().to_lowercase().as_str() {
            "q" => break,
            "1" => {
                println!("Sending http request ...");
                let response = perform_request(&Api::AxumApi, "downstream-api-status?q=foo").await;
                match response {
                    Ok(response) => {
                        let data = response.text().await;
                        println!("Response: {:?}", data);
                    }
                    Err(err) => println!("Error: {}", err),
                }
            }
            "2" => {
                println!("Sending http request ...");
                let response = perform_request(&Api::AxumApi, "greet/foo/bar").await;
                match response {
                    Ok(response) => {
                        let data = response.text().await;
                        println!("Response: {:?}", data);
                    }
                    Err(err) => println!("Error: {}", err),
                }
            }
            "3" => {
                println!("Sending http request ...");
                let response = perform_request(&Api::AxumDownstreamApi, "status").await;
                match response {
                    Ok(response) => {
                        let data = response.text().await;
                        println!("Response: {:?}", data);
                    }
                    Err(err) => println!("Error: {}", err),
                }
            }
            "4" => {
                println!("Sending http request ...");
                let response = perform_request(&Api::DotnetApi, "weatherforecast").await;
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
