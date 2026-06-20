use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::models::agent::Agent;
use crate::models::service::Service;
use crate::models::transaction::Transaction;

#[derive(Debug, Deserialize)]
pub struct SpawnAgentRequest {
    pub name: String,
    pub description: String,
    pub api_key: String,
}

#[derive(Debug, Serialize)]
pub struct AgentStatusResponse {
    pub agent: Agent,
    pub services_count: i64,
    pub transactions_count: i64,
    pub total_revenue_cents: i64,
}

#[derive(Debug, Serialize)]
pub struct AgentLogEntry {
    pub timestamp: String,
    pub event: String,
}

/// POST /api/v1/agents/spawn — create and start an agent
pub async fn spawn_agent(
    State(pool): State<Arc<SqlitePool>>,
    Json(req): Json<SpawnAgentRequest>,
) -> impl IntoResponse {
    // Simple API key check (in production, validate against a real key store)
    if req.api_key.len() < 8 {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "invalid api key"})),
        );
    }

    match Agent::create(&pool, &req.name, &req.description).await {
        Ok(agent) => {
            crate::websocket::broadcast_event(crate::websocket::DashboardEvent::AgentConnected {
                agent_id: agent.id.clone(),
                agent_name: agent.name.clone(),
            });
            (
                StatusCode::CREATED,
                Json(serde_json::json!({ "agent": agent })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

/// GET /api/v1/agents/{id}/status — check agent health, revenue, services
pub async fn agent_status(
    State(pool): State<Arc<SqlitePool>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let agent = match Agent::get_by_id(&pool, &id).await {
        Ok(Some(a)) => a,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "agent not found"})),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        }
    };

    let services_count = match Service::list(&pool).await {
        Ok(s) => s.iter().filter(|svc| svc.agent_id == id).count() as i64,
        Err(_) => 0,
    };

    let transactions = match Transaction::list(&pool).await {
        Ok(t) => t,
        Err(_) => vec![],
    };

    let tx_count = transactions.iter().filter(|t| t.seller_id == id).count() as i64;
    let revenue = transactions
        .iter()
        .filter(|t| t.seller_id == id && t.status == "paid")
        .map(|t| t.amount_cents)
        .sum::<i64>();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "agent": agent,
            "services_count": services_count,
            "transactions_count": tx_count,
            "total_revenue_cents": revenue,
        })),
    )
}

/// POST /api/v1/agents/{id}/stop — pause agent (mark inactive)
pub async fn stop_agent(
    State(pool): State<Arc<SqlitePool>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match sqlx::query("UPDATE agents SET active = 0 WHERE id = ?")
        .bind(&id)
        .execute(&*pool)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "stopped", "agent_id": id})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

/// GET /api/v1/agents/{id}/logs — agent activity log
pub async fn agent_logs(
    State(pool): State<Arc<SqlitePool>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Get recent transactions as "logs"
    let logs = match Transaction::list(&pool).await {
        Ok(txs) => txs
            .into_iter()
            .filter(|t| t.seller_id == id || t.buyer_id == id)
            .take(20)
            .map(|t| {
                serde_json::json!({
                    "timestamp": t.created_at,
                    "event": format!("Transaction {}: {} -> ${}.{}", 
                        t.status, t.service_id, t.amount_cents / 100, t.amount_cents % 100),
                })
            })
            .collect::<Vec<_>>(),
        Err(_) => vec![],
    };

    (
        StatusCode::OK,
        Json(serde_json::json!({ "logs": logs })),
    )
}
