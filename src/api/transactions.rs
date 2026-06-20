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

use crate::models::transaction::Transaction;

#[derive(Debug, Serialize)]
pub struct TransactionListResponse {
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub service_id: String,
    pub buyer_id: String,
    pub seller_id: String,
    pub amount_cents: i64,
}

pub async fn list_transactions(State(pool): State<Arc<SqlitePool>>) -> impl IntoResponse {
    match Transaction::list(&pool).await {
        Ok(transactions) => (
            StatusCode::OK,
            Json(serde_json::json!({ "transactions": transactions })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn get_transaction(
    State(pool): State<Arc<SqlitePool>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match Transaction::get_by_id(&pool, &id).await {
        Ok(Some(tx)) => {
            (StatusCode::OK, Json(serde_json::json!({ "transaction": tx })))
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "transaction not found" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn create_transaction(
    State(pool): State<Arc<SqlitePool>>,
    Json(req): Json<CreateTransactionRequest>,
) -> impl IntoResponse {
    match Transaction::create(
        &pool,
        &req.service_id,
        &req.buyer_id,
        &req.seller_id,
        req.amount_cents,
    )
    .await
    {
        Ok(tx) => {
            crate::websocket::broadcast_event(crate::websocket::DashboardEvent::PurchaseInitiated {
                tx_id: tx.id.clone(),
                service_name: req.service_id.clone(),
                buyer_id: req.buyer_id.clone(),
            });
            (
                StatusCode::CREATED,
                Json(serde_json::json!({ "transaction": tx })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}
