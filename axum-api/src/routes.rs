use axum::{extract::Path, http::HeaderMap};
use opentelemetry::global;
use opentelemetry_http::HeaderExtractor;
use shared::traceable_http_client;

pub async fn greet_handler(Path((first_name, last_name)): Path<(String, String)>) -> String {
    format!(
        "Hello {} {}, the axum-api server greets you!",
        first_name, last_name
    )
}

pub async fn get_axum_downstream_api_status(headers: HeaderMap) -> String {
    let parent_cx = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(&headers))
    });
    // Create the traceable HTTP client.
    let http_client = traceable_http_client::TraceableHttpClient::new(
        traceable_http_client::UriScheme::Http,
        "localhost".to_string(),
        Some(9000),
    );
    // Fetch the downstream API.
    let response = http_client.get("status", Some(parent_cx)).await;
    // Parse the response and ensure that the result will be a string.
    let result = match response {
        Ok(res) => match res.text().await {
            Ok(value) => value,
            Err(err) => err.to_string(),
        },
        Err(err) => err.to_string(),
    };
    // Return the result.
    result
}
