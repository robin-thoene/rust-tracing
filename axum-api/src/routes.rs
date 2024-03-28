use axum::extract::Path;

pub async fn greet_handler(Path((first_name, last_name)): Path<(String, String)>) -> String {
    format!(
        "Hello {} {}, the axum-api server greets you!",
        first_name, last_name
    )
}

pub async fn get_axum_downstream_api_status() -> String {
    // TODO: Fetch the downstream API and return the result.
    "TODO".to_string()
}
