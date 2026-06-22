use axum::extract::State;
use axum::response::IntoResponse;
use axum::{extract::Path, http::StatusCode, Json};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::agent_loop::AgentLoop;
use crate::monitor::{generate_catalog, demonstrate_service};

/// GET /api/monitor/catalog — Live service catalog with demonstrations
pub async fn get_catalog(
    State(pool): State<Arc<SqlitePool>>,
) -> impl IntoResponse {
    match generate_catalog(&pool).await {
        Ok(catalog) => (StatusCode::OK, Json(serde_json::json!({ "catalog": catalog }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

/// GET /api/monitor/demonstrate/:service_id — Demonstrate a specific service
pub async fn demonstrate(
    State(pool): State<Arc<SqlitePool>>,
    Path(service_id): Path<String>,
) -> impl IntoResponse {
    match demonstrate_service(&pool, &service_id).await {
        Ok(demo) => (StatusCode::OK, Json(serde_json::json!({ "demo": demo }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

/// POST /api/agents/tick — Run one tick of the agent loop
pub async fn agent_tick(
    State(pool): State<Arc<SqlitePool>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let loop_engine = AgentLoop::new(pool);
    
    match loop_engine.tick().await {
        Ok(results) => {
            let mut simplified = vec![];
            for r in results {
                simplified.push(serde_json::json!({
                    "type": r.interaction_type,
                    "agent": r.agent_id,
                    "target": r.target_id,
                    "success": r.success,
                    "message": r.message,
                }));
            }
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "tick": "completed",
                    "interactions": simplified,
                    "count": simplified.len(),
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

/// POST /api/agents/:id/create-service — Agent creates a service
pub async fn agent_create_service(
    State(pool): State<Arc<SqlitePool>>,
    Path(agent_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let loop_engine = AgentLoop::new(pool);
    
    match loop_engine.agent_create_service(&agent_id).await {
        Ok(Some(result)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "action": "create_service",
                "success": result.success,
                "message": result.message,
            })),
        ),
        Ok(None) => (
            StatusCode::OK,
            Json(serde_json::json!({ "message": "Agent could not create service" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

/// POST /api/agents/:id/review — Agent leaves a review
pub async fn agent_leave_review(
    State(pool): State<Arc<SqlitePool>>,
    Path(agent_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let loop_engine = AgentLoop::new(pool);
    
    match loop_engine.agent_leave_review(&agent_id).await {
        Ok(Some(result)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "action": "review",
                "success": result.success,
                "message": result.message,
            })),
        ),
        Ok(None) => (
            StatusCode::OK,
            Json(serde_json::json!({ "message": "No completed transactions to review" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

/// GET /api/agents/states — Get current agent states
pub async fn agent_states(
    State(pool): State<Arc<SqlitePool>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let loop_engine = AgentLoop::new(pool);
    
    let agent_states = match loop_engine.get_states().await {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
    };

    let mut states = vec![];
    for (id, state) in &agent_states {
        states.push(serde_json::json!({
            "agent_id": id,
            "name": state.name,
            "balance_cents": state.balance_cents,
            "reputation": state.reputation,
            "mood": format!("{:?}", state.mood),
            "skills": state.skills,
            "needs": state.needs,
        }));
    }
    
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "agents": states,
            "count": states.len(),
        })),
    )
}
