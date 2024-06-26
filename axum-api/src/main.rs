use std::net::SocketAddr;

use axum::{middleware, routing::get, Router};
use opentelemetry::global::shutdown_tracer_provider;
use shared::{middlewares::otel_tracing_middleware, tracer::init_tracer};

mod routes;

#[tokio::main]
async fn main() {
    let _tracer = init_tracer("axum-api".to_string()).expect("Failed to initialize tracer.");

    let app = Router::new()
        .route("/greet/:first_name/:last_name", get(routes::greet_handler))
        .route(
            "/downstream-api-status",
            get(routes::get_axum_downstream_api_status),
        )
        .layer(middleware::from_fn(otel_tracing_middleware));
    let addr = SocketAddr::from(([127, 0, 0, 1], 5000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("Server is expected to start.");
    shutdown_tracer_provider();
}
