use anyhow::Result;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::models::service::Service;

#[derive(Debug, Serialize)]
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

pub async fn list_services(State(pool): State<Arc<SqlitePool>>) -> impl IntoResponse {
    match Service::list(&pool).await {
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
    State(pool): State<Arc<SqlitePool>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match Service::get_by_id(&pool, &id).await {
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
    State(pool): State<Arc<SqlitePool>>,
    Json(req): Json<CreateServiceRequest>,
) -> impl IntoResponse {
    match Service::create(
        &pool,
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
