use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::models::deliverable::Deliverable;
use crate::models::service::Service;

#[derive(Debug, Deserialize)]
pub struct ExecuteRequest {
    pub user_input: String,
}

#[derive(Debug, Serialize)]
pub struct ExecuteResponse {
    pub service_id: String,
    pub service_name: String,
    pub result: String,
    pub execution_time_ms: u64,
    pub powered_by: String,
}

/// Execute a service directly without purchase (demo/try-before-you-buy)
pub async fn execute_service(
    State(pool): State<Arc<SqlitePool>>,
    Path(service_id): Path<String>,
    Json(req): Json<ExecuteRequest>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();

    // Verify service exists
    let service = match Service::get_by_id(&pool, &service_id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "service not found"})),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        }
    };

    // Execute the service with user input
    let result = match crate::delivery::execute_service_direct(&pool, &service_id, &req.user_input).await {
        Ok(output) => output,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("execution failed: {}", e)})),
            )
        }
    };

    let execution_time_ms = start.elapsed().as_millis() as u64;
    let powered_by = if let Some(def) = crate::service_catalog::get_service_definition(&service.service_type) {
        format!("{} ({}, {})", def.model.model_name(), def.model.context_size(), match def.tier {
            crate::service_catalog::ServiceTier::MicroTask => "Micro-Task",
            crate::service_catalog::ServiceTier::RealWork => "Real Work",
            crate::service_catalog::ServiceTier::HeavyLifting => "Heavy Lifting",
            crate::service_catalog::ServiceTier::LocalOnly => "Local-Only",
        })
    } else if std::env::var("NVIDIA_API_KEY").is_ok() {
        "NVIDIA Nemotron 3 Ultra".to_string()
    } else {
        "Local LLM (Qwen3.5-9B via llama-swap)".to_string()
    };

    (
        StatusCode::OK,
        Json(serde_json::json!(ExecuteResponse {
            service_id: service.id,
            service_name: service.name,
            result,
            execution_time_ms,
            powered_by,
        })),
    )
}

/// Get deliverable with full output for a transaction
pub async fn get_deliverable(
    State(pool): State<Arc<SqlitePool>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match Deliverable::get_by_transaction(&pool, &id).await {
        Ok(Some(d)) => {
            (StatusCode::OK, Json(serde_json::json!({ "deliverable": d })))
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "deliverable not found" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}
