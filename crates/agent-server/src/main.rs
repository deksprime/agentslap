//! Agent Server - REST API for agent management
//!
//! Provides HTTP endpoints for creating agents, sending messages,
//! and streaming responses.

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod models;
mod state;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "agent_server=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Get API keys from environment
    let openai_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY must be set");
    let anthropic_key = std::env::var("ANTHROPIC_API_KEY").ok();

    // Create shared state
    let state = AppState::new(openai_key, anthropic_key).await?;

    // Build router
    let app = Router::new()
        .route("/", get(handlers::health))
        .route("/agents", post(handlers::create_agent))
        .route("/agents", get(handlers::list_agents))
        .route("/agents/:agent_id", get(handlers::get_agent))
        .route("/agents/:agent_id", axum::routing::delete(handlers::delete_agent))
        .route("/agents/:agent_id/messages", post(handlers::send_message))
        .route("/agents/:agent_id/messages", get(handlers::get_messages))
        .route("/agents/:agent_id/stream", post(handlers::stream_message))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("🚀 Agent Server listening on http://{}", addr);
    tracing::info!("📚 Endpoints:");
    tracing::info!("  POST   /agents - Create new agent");
    tracing::info!("  GET    /agents - List all agents");
    tracing::info!("  GET    /agents/:id - Get agent details");
    tracing::info!("  DELETE /agents/:id - Stop agent");
    tracing::info!("  POST   /agents/:id/messages - Send message (JSON)");
    tracing::info!("  POST   /agents/:id/stream - Send message (SSE stream)");
    tracing::info!("  GET    /agents/:id/messages - Get message history");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
