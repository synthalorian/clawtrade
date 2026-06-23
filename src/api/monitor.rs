use axum::extract::State;
use axum::response::IntoResponse;
use axum::{extract::Path, http::StatusCode, Json};
use std::sync::Arc;

use crate::monitor::{generate_catalog, demonstrate_service};
use crate::AppState;

/// GET /api/monitor/catalog — Live service catalog with demonstrations
pub async fn get_catalog(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match generate_catalog(&state.pool).await {
        Ok(catalog) => (StatusCode::OK, Json(serde_json::json!({ "catalog": catalog }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

/// GET /api/monitor/demonstrate/:service_id — Demonstrate a specific service
pub async fn demonstrate(
    State(state): State<Arc<AppState>>,
    Path(service_id): Path<String>,
) -> impl IntoResponse {
    match demonstrate_service(&state.pool, &service_id).await {
        Ok(demo) => (StatusCode::OK, Json(serde_json::json!({ "demo": demo }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

/// POST /api/agents/tick — Run one tick of the agent loop
pub async fn agent_tick(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let loop_engine = state.agent_loop.lock().await;

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
    State(state): State<Arc<AppState>>,
    Path(agent_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let loop_engine = state.agent_loop.lock().await;

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
    State(state): State<Arc<AppState>>,
    Path(agent_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let loop_engine = state.agent_loop.lock().await;

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

/// GET /api/monitor/stats — Marketplace analytics
pub async fn marketplace_stats(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let total_revenue: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount_cents), 0) FROM transactions WHERE status = 'paid' OR status = 'released'"
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    let total_transactions: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM transactions")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

    let total_services: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM services WHERE status = 'active'")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

    let total_services_all: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM services")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

    let total_agents: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agents")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

    let tier_counts: Vec<(String, i64)> = sqlx::query_as(
        "SELECT service_type, COUNT(*) as cnt FROM services WHERE status = 'active' GROUP BY service_type"
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut tier_micro = 0i64;
    let mut tier_real = 0i64;
    let mut tier_heavy = 0i64;
    let mut tier_local = 0i64;
    let mut tier_unknown = 0i64;

    for (svc_type, count) in tier_counts {
        if let Some(def) = crate::service_catalog::get_service_definition(&svc_type) {
            match def.tier {
                crate::service_catalog::ServiceTier::MicroTask => tier_micro += count,
                crate::service_catalog::ServiceTier::RealWork => tier_real += count,
                crate::service_catalog::ServiceTier::HeavyLifting => tier_heavy += count,
                crate::service_catalog::ServiceTier::LocalOnly => tier_local += count,
            }
        } else {
            tier_unknown += count;
        }
    }

    let top_agents: Vec<(String, String, i64, i64)> = sqlx::query_as(
        "SELECT id, name, total_sales, total_revenue_cents FROM agents ORDER BY total_revenue_cents DESC LIMIT 5"
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let recent_txs: Vec<(String, String, i64, String)> = sqlx::query_as(
        "SELECT t.id, s.name, t.amount_cents, t.status 
         FROM transactions t 
         JOIN services s ON t.service_id = s.id 
         ORDER BY t.created_at DESC LIMIT 10"
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "total_revenue_cents": total_revenue,
            "total_transactions": total_transactions,
            "active_services": total_services,
            "total_services_created": total_services_all,
            "total_agents": total_agents,
            "tier_distribution": {
                "micro": tier_micro,
                "real": tier_real,
                "heavy": tier_heavy,
                "local": tier_local,
                "unknown": tier_unknown,
            },
            "top_agents": top_agents.iter().map(|(id, name, sales, rev)| serde_json::json!({
                "id": id,
                "name": name,
                "sales": sales,
                "revenue_cents": rev,
            })).collect::<Vec<_>>(),
            "recent_transactions": recent_txs.iter().map(|(id, name, amount, status)| serde_json::json!({
                "id": id,
                "service_name": name,
                "amount_cents": amount,
                "status": status,
            })).collect::<Vec<_>>(),
        })),
    )
}

/// GET /api/agents/states — Get current agent states
pub async fn agent_states(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let loop_engine = state.agent_loop.lock().await;

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
