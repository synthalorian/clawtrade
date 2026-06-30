use axum::extract::State;
use axum::{extract::Path, http::StatusCode, Json};
use std::sync::Arc;

use crate::models::activity_log::ActivityLog;
use crate::AppState;

/// GET /api/activity — Global activity feed (Etherscan-style)
pub async fn global_activity(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<serde_json::Value>) {
    match ActivityLog::list_global(&state.pool, 100).await {
        Ok(logs) => {
            let stats = match ActivityLog::get_stats(&state.pool).await {
                Ok(s) => serde_json::json!(s),
                Err(_) => serde_json::json!(null),
            };
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "activities": logs,
                    "stats": stats,
                    "count": logs.len(),
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

/// GET /api/activity/agent/:id — Per-agent activity ledger
pub async fn agent_activity(
    State(state): State<Arc<AppState>>,
    Path(agent_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match ActivityLog::list_by_agent(&state.pool, &agent_id, 50).await {
        Ok(logs) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "agent_id": agent_id,
                "activities": logs,
                "count": logs.len(),
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

/// GET /api/activity/tx/:id — Activity for a specific transaction (receipt view)
pub async fn tx_activity(
    State(state): State<Arc<AppState>>,
    Path(tx_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match ActivityLog::list_by_target(&state.pool, &tx_id, 10).await {
        Ok(logs) => {
            // Also fetch the deliverable for this transaction
            let deliverable = match crate::models::deliverable::Deliverable::get_by_transaction(
                &state.pool,
                &tx_id,
            )
            .await
            {
                Ok(d) => d,
                Err(_) => None,
            };

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "transaction_id": tx_id,
                    "activities": logs,
                    "deliverable": deliverable,
                    "count": logs.len(),
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

/// GET /api/activity/stats — Global marketplace stats
pub async fn activity_stats(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<serde_json::Value>) {
    match ActivityLog::get_stats(&state.pool).await {
        Ok(stats) => (StatusCode::OK, Json(serde_json::json!({ "stats": stats }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}
