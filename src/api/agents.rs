use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::models::agent::Agent;
use crate::AppState;

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

pub async fn list_agents(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match Agent::list(&state.pool).await {
        Ok(agents) => (
            StatusCode::OK,
            Json(serde_json::json!({ "agents": agents })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn get_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match Agent::get_by_id(&state.pool, &id).await {
        Ok(Some(agent)) => (StatusCode::OK, Json(serde_json::json!({ "agent": agent }))),
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
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateAgentRequest>,
) -> impl IntoResponse {
    if req.name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "name cannot be empty" })),
        );
    }

    match Agent::create(&state.pool, &req.name, &req.description).await {
        Ok(agent) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "agent": agent })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}
