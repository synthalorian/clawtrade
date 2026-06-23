use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

mod agent_loop;
mod api;
mod dashboard;
mod db;
mod delivery;
mod models;
mod monitor;
mod nvidia;
mod service_catalog;
mod websocket;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub llm: Arc<nvidia::LlmClient>,
    pub agent_loop: Arc<Mutex<agent_loop::AgentLoop>>,
}

impl AppState {
    pub fn new(pool: SqlitePool) -> Self {
        let llm = Arc::new(nvidia::LlmClient::new());
        let agent_loop = Arc::new(Mutex::new(agent_loop::AgentLoop::new(pool.clone())));
        Self {
            pool,
            llm,
            agent_loop,
        }
    }
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
    let state = Arc::new(AppState::new(pool));

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
        .route("/api/demo/purchase", post(api::stripe::demo_purchase))
        .route("/api/webhooks/stripe", post(api::stripe::stripe_webhook))
        .route("/api/stripe/connect", post(api::stripe::create_connect_account))
        .route("/api/stripe/account_link", post(api::stripe::create_account_link))
        .route("/api/deliverables/{id}", get(api::deliverables::get_deliverable))
        .route("/api/services/{id}/execute", post(api::deliverables::execute_service))
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
        .route("/api/monitor/catalog", get(api::monitor::get_catalog))
        .route("/api/monitor/demonstrate/{service_id}", get(api::monitor::demonstrate))
        .route("/api/monitor/stats", get(api::monitor::marketplace_stats))
        .route("/api/agents/tick", post(api::monitor::agent_tick))
        .route("/api/agents/states", get(api::monitor::agent_states))
        .route("/api/agents/{id}/create-service", post(api::monitor::agent_create_service))
        .route("/api/agents/{id}/review", post(api::monitor::agent_leave_review))
        .route("/api/activity", get(api::activity::global_activity))
        .route("/api/activity/stats", get(api::activity::activity_stats))
        .route("/api/activity/agent/{id}", get(api::activity::agent_activity))
        .route("/api/activity/tx/{id}", get(api::activity::tx_activity))
        .route("/ws", get(websocket::ws_handler))
        .with_state(state.clone());

    let dashboard_routes = Router::new()
        .route("/", get(dashboard::index_handler))
        .route("/services", get(dashboard::services_page))
        .route("/agents", get(dashboard::agents_page))
        .route("/transactions", get(dashboard::transactions_page))
        .route("/my-purchases", get(dashboard::my_purchases_page))
        .route("/deliverable/{id}", get(dashboard::deliverable_page))
        .route("/success", get(dashboard::success_page))
        .route("/cancel", get(dashboard::cancel_page))
        .route("/monitor", get(dashboard::monitor_page))
        .route("/agent-loop", get(dashboard::agent_loop_page))
        .with_state(state.clone());

    let api_addr = std::env::var("CLAWTRADE_API_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let dashboard_addr = std::env::var("CLAWTRADE_DASHBOARD_ADDR").unwrap_or_else(|_| "127.0.0.1:8746".to_string());

    eprintln!("[clawtrade] API server starting on http://{}", api_addr);
    eprintln!("[clawtrade] Dashboard starting on http://{}", dashboard_addr);

    // CORS: permissive in debug, restricted in release
    #[cfg(debug_assertions)]
    let cors = CorsLayer::permissive();
    #[cfg(not(debug_assertions))]
    let cors = CorsLayer::new();

    // API server: API routes + dashboard routes (for full functionality)
    let app = Router::new()
        .merge(api_routes.clone())
        .merge(dashboard_routes)
        .layer(cors);

    let api_listener = tokio::net::TcpListener::bind(&api_addr).await?;
    let api_handle = tokio::spawn(async move {
        axum::serve(api_listener, app).await
    });

    // Dashboard server also needs API routes for same-origin frontend calls
    let dashboard_app = dashboard::dashboard_router(state.clone())
        .merge(api_routes);
    let dashboard_listener = tokio::net::TcpListener::bind(&dashboard_addr).await?;
    let dashboard_handle = tokio::spawn(async move {
        axum::serve(dashboard_listener, dashboard_app).await
    });

    // Auto-tick: agents trade autonomously every 30 seconds
    let tick_state = state.clone();
    let tick_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let loop_engine = tick_state.agent_loop.lock().await;
            match loop_engine.tick().await {
                Ok(results) => {
                    if !results.is_empty() {
                        let summary: Vec<String> = results.iter().map(|r| format!("{}: {}", r.interaction_type, r.message)).collect();
                        eprintln!("[autotick] {} actions: {}", results.len(), summary.join(" | "));
                    }
                }
                Err(e) => eprintln!("[autotick] error: {}", e),
            }
        }
    });

    tokio::select! {
        r = api_handle => r??,
        r = dashboard_handle => r??,
        _ = tick_handle => {},
    }
    Ok(())
}