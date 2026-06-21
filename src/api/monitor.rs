use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use rand::seq::SliceRandom;
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::agent_loop::AgentLoop;
use crate::models::agent::Agent;
use crate::models::service::Service;
use crate::models::transaction::Transaction;
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
    // Simple tick: agents browse and potentially buy
    let agents = match Agent::list(&pool).await {
        Ok(a) => a,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
    };
    
    let services = match Service::list_active(&pool).await {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
    };
    
    let mut interactions = vec![];
    
    for agent in agents {
        if services.is_empty() {
            break;
        }
        
        // Simple deterministic action: every 3rd agent buys
        let should_act = agent.id.chars().next().map(|c| c as u32 % 3 == 0).unwrap_or(false);
        if !should_act {
            continue;
        }
        
        // Pick first available service
        if let Some(service) = services.first() {
            // Create transaction (demo purchase)
            let pool_ref = pool.clone();
            let service_id = service.id.clone();
            let agent_id = agent.id.clone();
            let seller_id = service.agent_id.clone();
            let price = service.price_cents;
            let agent_name = agent.name.clone();
            let service_name = service.name.clone();
            
            match Transaction::create(&pool_ref, &service_id, &agent_id, &seller_id, price).await {
                Ok(tx) => {
                    interactions.push(serde_json::json!({
                        "type": "purchase",
                        "agent": agent_name,
                        "service": service_name,
                        "price_cents": price,
                        "transaction_id": tx.id,
                        "success": true,
                    }));
                }
                Err(e) => {
                    interactions.push(serde_json::json!({
                        "type": "purchase_failed",
                        "agent": agent_name,
                        "error": e.to_string(),
                        "success": false,
                    }));
                }
            }
        }
    }
    
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "tick": "completed",
            "interactions": interactions,
            "count": interactions.len(),
        })),
    )
}

/// GET /api/agents/states — Get current agent states
pub async fn agent_states(
    State(pool): State<Arc<SqlitePool>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut loop_engine = AgentLoop::new(pool);
    
    if let Err(e) = loop_engine.init().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("init failed: {}", e) })),
        );
    }

    let mut states = vec![];
    for (id, state) in &loop_engine.agent_states {
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
