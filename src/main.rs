use anyhow::Result;
use axum::{
    Router,
    extract::State,
    routing::{get, post},
};
use sqlx::SqlitePool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

mod api;
mod dashboard;
mod db;
mod delivery;
mod models;
mod nvidia;
mod websocket;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("STRIPE_SECRET_KEY").is_err() {
        eprintln!("[clawtrade] WARNING: STRIPE_SECRET_KEY not set. Stripe payments will fail.");
    }

    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("clawtrade");
    let db_path = data_dir.join("clawtrade.db");
    eprintln!("[clawtrade] database: {}", db_path.display());

    let pool = db::init_db(&db_path).await?;
    let state = Arc::new(pool);

    let api_routes = Router::new()
        .route("/api/services", get(api::services::list_services).post(api::services::create_service))
        .route("/api/services/{id}", get(api::services::get_service))
        .route("/api/agents", get(api::agents::list_agents).post(api::agents::create_agent))
        .route("/api/agents/{id}", get(api::agents::get_agent))
        .route("/api/transactions", get(api::transactions::list_transactions).post(api::transactions::create_transaction))
        .route("/api/transactions/{id}", get(api::transactions::get_transaction))
        .route("/api/transactions/{id}/release", post(api::transactions::release_escrow))
        .route("/api/transactions/{id}/dispute", post(api::transactions::dispute_transaction))
        .route("/api/checkout", get(api::stripe::create_checkout))
        .route("/api/webhooks/stripe", post(api::stripe::stripe_webhook))
        .route("/api/stripe/connect", post(api::stripe::create_connect_account))
        .route("/api/stripe/account_link", post(api::stripe::create_account_link))
        .route("/api/deliverables/{id}", get(api::deliverables::get_deliverable))
        .route("/api/reviews", post(api::reviews::create_review))
        .route("/api/agents/{id}/reviews", get(api::reviews::list_reviews))
        .route("/api/llm/summarize", post(api::llm::summarize))
        .route("/api/llm/analyze", post(api::llm::analyze))
        .route("/api/v1/agents/spawn", post(api::hosting::spawn_agent))
        .route("/api/v1/agents/{id}/status", get(api::hosting::agent_status))
        .route("/api/v1/agents/{id}/stop", post(api::hosting::stop_agent))
        .route("/api/v1/agents/{id}/logs", get(api::hosting::agent_logs))
        .route("/api/v1/market/trends", get(api::pricing::market_trends))
        .route("/api/v1/pricing/recommendations", get(api::pricing::pricing_recommendations))
        .route("/api/v1/templates", get(api::templates::list_templates))
        .route("/api/v1/templates/{id}", get(api::templates::get_template))
        .route("/api/v1/templates/{id}/deploy", post(api::templates::deploy_template))
        .route("/ws", get(websocket::ws_handler))
        .with_state(state.clone());

    let dashboard_routes = Router::new()
        .route("/", get(dashboard::index_handler))
        .route("/services", get(dashboard::services_page))
        .route("/agents", get(dashboard::agents_page))
        .route("/transactions", get(dashboard::transactions_page))
        .route("/success", get(dashboard::success_page))
        .route("/cancel", get(dashboard::cancel_page))
        .with_state(state.clone());

    let app = Router::new()
        .merge(api_routes)
        .merge(dashboard_routes)
        .layer(CorsLayer::permissive());

    let api_addr = std::env::var("CLAWTRADE_API_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let dashboard_addr = std::env::var("CLAWTRADE_DASHBOARD_ADDR").unwrap_or_else(|_| "127.0.0.1:8746".to_string());

    eprintln!("[clawtrade] API server starting on http://{}", api_addr);
    eprintln!("[clawtrade] Dashboard starting on http://{}", dashboard_addr);

    let api_listener = tokio::net::TcpListener::bind(&api_addr).await?;
    let api_handle = tokio::spawn(async move {
        axum::serve(api_listener, app).await
    });

    let dashboard_app = dashboard::dashboard_router(state.clone());
    let dashboard_listener = tokio::net::TcpListener::bind(&dashboard_addr).await?;
    let dashboard_handle = tokio::spawn(async move {
        axum::serve(dashboard_listener, dashboard_app).await
    });

    tokio::select! {
        r = api_handle => r??,
        r = dashboard_handle => r??,
    }

    Ok(())
}
