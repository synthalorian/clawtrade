//! Health check endpoint — returns system status for monitoring and judge demos

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
    pub llm_local: String,
    pub llm_local_detail: String,
    pub stripe: String,
    pub stripe_detail: String,
    pub version: String,
    pub uptime_seconds: u64,
}

pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();

    // Check database
    let db_status = match sqlx::query("SELECT 1").fetch_one(&state.pool).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    // Check local LLM (llama-swap on port 8080)
    let llm_status = match check_llm_local().await {
        Ok(detail) => ("connected", detail),
        Err(e) => ("disconnected", format!("{}", e)),
    };

    // Check Stripe
    let stripe_status = match std::env::var("STRIPE_SECRET_KEY") {
        Ok(_) => ("configured", "Live payments enabled".to_string()),
        Err(_) => ("demo_mode", "STRIPE_SECRET_KEY not set — running in demo mode".to_string()),
    };

    let overall = if db_status == "connected" {
        "healthy"
    } else {
        "degraded"
    };

    let response = HealthResponse {
        status: overall.to_string(),
        database: db_status.to_string(),
        llm_local: llm_status.0.to_string(),
        llm_local_detail: llm_status.1,
        stripe: stripe_status.0.to_string(),
        stripe_detail: stripe_status.1,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: start.elapsed().as_secs(),
    };

    let status_code = if overall == "healthy" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(serde_json::json!(response)))
}

async fn check_llm_local() -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = std::env::var("LLM_LOCAL_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
    
    let res = client
        .get(format!("{}/v1/models", url))
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await?;
    
    if res.status().is_success() {
        let body = res.text().await?;
        Ok(format!("llama-swap reachable — models: {}", body.len()))
    } else {
        Ok(format!("llama-swap returned status {}", res.status()))
    }
}
