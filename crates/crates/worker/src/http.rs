use std::sync::Arc;

use axum::{Json, extract::State};
use tower_http::cors::CorsLayer;

use crate::script::{Script, ScriptOutput};

#[derive(Debug, Clone)]
struct HttpState {
    semaphore: Arc<tokio::sync::Semaphore>,
}

pub async fn run_http_worker(host: String, port: u16, count: u16) -> Result<(), anyhow::Error> {
    let state = HttpState {
        semaphore: Arc::new(tokio::sync::Semaphore::new(count as usize)),
    };

    let cors_layer = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = axum::Router::new()
        .route("/execute", axum::routing::post(handle_event))
        .layer(cors_layer)
        .with_state(state);

    let addr: std::net::SocketAddr = format!("{}:{}", host, port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Starting HTTP worker on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

#[axum::debug_handler]
async fn handle_event(
    State(state): State<HttpState>,
    Json(script): Json<Script>,
) -> Result<axum::Json<ScriptOutput>, axum::http::StatusCode> {
    let _permit = state.semaphore.acquire().await.unwrap();
    let output = script.execute().await.map_err(|e| {
        tracing::error!("Error executing script: {}", e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(axum::Json(output))
}
