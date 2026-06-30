use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
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
        Ok(services) => (
            StatusCode::OK,
            Json(serde_json::json!({ "services": services })),
        ),
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
        Ok(Some(service)) => (
            StatusCode::OK,
            Json(serde_json::json!({ "service": service })),
        ),
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
            crate::websocket::broadcast_event(crate::websocket::DashboardEvent::ServiceCreated {
                service_id: service.id.clone(),
                name: service.name.clone(),
                agent_name: req.agent_id.clone(),
            });
            (
                StatusCode::CREATED,
                Json(serde_json::json!({ "service": service })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct TryServiceRequest {
    pub input: String,
}

#[derive(Debug, Serialize)]
pub struct TryServiceResponse {
    pub service_name: String,
    pub model_used: String,
    pub tier: String,
    pub execution_time_ms: u64,
    pub output: String,
    pub watermark: bool,
}

pub async fn try_service(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<TryServiceRequest>,
) -> impl IntoResponse {
    let service = match Service::get_by_id(&state.pool, &id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "service not found" })),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
    };

    let def = match crate::service_catalog::get_service_definition(&service.service_type) {
        Some(d) => d,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "service type not in catalog" })),
            );
        }
    };

    let start = std::time::Instant::now();

    match state.llm.deliver_service(def, &req.input).await {
        Ok(result) => {
            let execution_time_ms = start.elapsed().as_millis() as u64;
            let tier_label = match def.tier {
                crate::service_catalog::ServiceTier::MicroTask => "Micro-Task",
                crate::service_catalog::ServiceTier::RealWork => "Real Work",
                crate::service_catalog::ServiceTier::HeavyLifting => "Heavy Lifting",
                crate::service_catalog::ServiceTier::LocalOnly => "Local-Only",
            };

            let output = format!(
                "{result}\n\n---\n💧 PREVIEW — Purchase to remove watermark\n🏷️ Tier: {tier_label} | 🧠 Model: {model} | ⏱️ {execution_time_ms}ms",
                result = result,
                tier_label = tier_label,
                model = def.model.model_name(),
                execution_time_ms = execution_time_ms
            );

            (
                StatusCode::OK,
                Json(serde_json::json!(TryServiceResponse {
                    service_name: service.name,
                    model_used: def.model.model_name(),
                    tier: tier_label.to_string(),
                    execution_time_ms,
                    output,
                    watermark: true,
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("LLM execution failed: {}", e),
                "service_name": service.name,
            })),
        ),
    }
}
