use anyhow::Result;
use axum::Router;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

mod agent_loop;
mod api;
mod dashboard;
mod db;
mod delivery;
mod hermes_bridge;
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
    pub hermes: Arc<hermes_bridge::HermesBridge>,
}

impl AppState {
    pub fn new(pool: SqlitePool) -> Self {
        let llm = Arc::new(nvidia::LlmClient::new());
        let agent_loop = Arc::new(Mutex::new(agent_loop::AgentLoop::new(pool.clone())));
        let hermes = Arc::new(hermes_bridge::HermesBridge::new(llm.clone()));
        Self {
            pool,
            llm,
            agent_loop,
            hermes,
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
    
    // Seed demo data if fresh database
    let agent_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agents")
        .fetch_one(&pool)
        .await
        .unwrap_or(0);
    if agent_count == 0 {
        eprintln!("[clawtrade] Seeding demo agents and services...");
        db::seed_demo_data(&pool).await?;
    }

    let state = Arc::new(AppState::new(pool));

    // Wire Hermes bridge into agent loop
    {
        let mut loop_guard = state.agent_loop.lock().await;
        loop_guard.hermes = Some(state.hermes.clone());
    }

    // Use centralized API routes
    let api_routes = api::routes(state.clone());
    
    let dashboard_routes = dashboard::dashboard_router(state.clone());

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

    // Auto-tick: agents trade autonomously every 15 seconds
    let tick_state = state.clone();
    let tick_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(15));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let loop_engine = tick_state.agent_loop.lock().await;
            match loop_engine.tick().await {
                Ok(results) => {
                    if !results.is_empty() {
                        eprintln!("[agent_loop] {} interactions", results.len());
                    }
                }
                Err(e) => eprintln!("[agent_loop] tick failed: {}", e),
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
