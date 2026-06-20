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

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct AgentListResponse {
    pub agents: Vec<Agent>,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct AgentDetailResponse {
    pub agent: Agent,
}

#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub description: String,
}

pub async fn list_agents(State(pool): State<Arc<SqlitePool>>) -> impl IntoResponse {
    match Agent::list(&pool).await {
        Ok(agents) => (StatusCode::OK, Json(serde_json::json!({ "agents": agents }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn get_agent(
    State(pool): State<Arc<SqlitePool>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match Agent::get_by_id(&pool, &id).await {
        Ok(Some(agent)) => {
            (StatusCode::OK, Json(serde_json::json!({ "agent": agent })))
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "agent not found" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn create_agent(
    State(pool): State<Arc<SqlitePool>>,
    Json(req): Json<CreateAgentRequest>,
) -> impl IntoResponse {
    match Agent::create(&pool, &req.name, &req.description).await {
        Ok(agent) => {
            (StatusCode::CREATED, Json(serde_json::json!({ "agent": agent })))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}
