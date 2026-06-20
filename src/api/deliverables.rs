use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::models::deliverable::Deliverable;

pub async fn get_deliverable(
    State(pool): State<Arc<SqlitePool>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match Deliverable::get_by_transaction(&pool, &id).await {
        Ok(Some(d)) => {
            (StatusCode::OK, Json(serde_json::json!({ "deliverable": d })))
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "deliverable not found" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}
