use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::models::service::Service;
use crate::AppState;

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct ServiceListResponse {
    pub services: Vec<Service>,
}

#[derive(Debug, Deserialize)]
pub struct CreateServiceRequest {
    pub name: String,
    pub description: String,
    pub price_cents: i64,
    pub agent_id: String,
    pub service_type: String,
}

pub async fn list_services(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match Service::list(&state.pool).await {
        Ok(services) => {
            (StatusCode::OK, Json(serde_json::json!({ "services": services })))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn get_service(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match Service::get_by_id(&state.pool, &id).await {
        Ok(Some(service)) => {
            (StatusCode::OK, Json(serde_json::json!({ "service": service })))
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "service not found" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn create_service(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateServiceRequest>,
) -> impl IntoResponse {
    // Input validation
    if req.name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "name cannot be empty" })),
        );
    }
    if req.description.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "description cannot be empty" })),
        );
    }
    if req.price_cents <= 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "price_cents must be greater than 0" })),
        );
    }
    if req.price_cents > 50000 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "price_cents cannot exceed $500.00" })),
        );
    }
    if req.name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "name cannot be empty" })),
        );
    }
    if req.agent_id.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "agent_id cannot be empty" })),
        );
    }

    match Service::create(
        &state.pool,
        &req.name,
        &req.description,
        req.price_cents,
        &req.agent_id,
        &req.service_type,
    )
    .await
    {
        Ok(service) => {
            // Broadcast service creation event
            crate::websocket::broadcast_event(crate::websocket::DashboardEvent::ServiceCreated {
                service_id: service.id.clone(),
                name: service.name.clone(),
                agent_name: req.agent_id.clone(),
            });
            (StatusCode::CREATED, Json(serde_json::json!({ "service": service })))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}
