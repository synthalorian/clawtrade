use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::models::transaction::Transaction;
use crate::AppState;

#[derive(Debug, Serialize)]
#[allow(dead_code)]
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

pub async fn list_transactions(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match Transaction::list(&state.pool).await {
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
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match Transaction::get_by_id(&state.pool, &id).await {
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
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateTransactionRequest>,
) -> impl IntoResponse {
    if req.amount_cents <= 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "amount_cents must be greater than 0" })),
        );
    }
    if req.amount_cents > 50000 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "amount_cents cannot exceed $500.00" })),
        );
    }
    if req.buyer_id == req.seller_id {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "buyer and seller cannot be the same" })),
        );
    }

    match Transaction::create(
        &state.pool,
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

pub async fn release_escrow(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match Transaction::get_by_id(&state.pool, &id).await {
        Ok(Some(tx)) => {
            if tx.status != "escrow" {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "transaction is not in escrow"})),
                );
            }

            // Real Stripe Connect transfer: move funds from platform to seller
            let stripe_secret = std::env::var("STRIPE_SECRET_KEY").ok();
            if let (Some(secret), Some(_stripe_session_id)) = (stripe_secret, &tx.stripe_session_id) {
                let seller = match crate::models::agent::Agent::get_by_id(&state.pool, &tx.seller_id).await {
                    Ok(Some(a)) => a,
                    _ => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({"error": "seller not found for transfer"})),
                        );
                    }
                };

                if let Some(stripe_account_id) = seller.stripe_account_id {
                    let client = reqwest::Client::new();
                    let platform_fee = (tx.amount_cents as f64 * 0.10).round() as i64;
                    let transfer_amount = tx.amount_cents - platform_fee;

                    let transfer_res = client
                        .post("https://api.stripe.com/v1/transfers")
                        .header("Authorization", format!("Bearer {}", secret))
                        .form(&vec![
                            ("amount", transfer_amount.to_string()),
                            ("currency", "usd".to_string()),
                            ("destination", stripe_account_id),
                            ("transfer_group", tx.id.clone()),
                        ])
                        .send()
                        .await;

                    match transfer_res {
                        Ok(res) => {
                            if let Ok(data) = res.json::<serde_json::Value>().await {
                                if let Some(transfer_id) = data["id"].as_str() {
                                    let _ = Transaction::update_stripe_transfer(&state.pool, &id, transfer_id).await;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("[escrow] Stripe transfer failed for tx {}: {}", id, e);
                            // Continue to release escrow even if transfer fails — manual reconciliation
                        }
                    }
                }
            }

            match Transaction::release_escrow(&state.pool, &id).await {
                Ok(()) => (
                    StatusCode::OK,
                    Json(serde_json::json!({"status": "released", "transaction_id": id})),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                ),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "transaction not found"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

pub async fn dispute_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match Transaction::get_by_id(&state.pool, &id).await {
        Ok(Some(tx)) => {
            if tx.status != "escrow" {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "transaction is not in escrow"})),
                );
            }
            match Transaction::dispute_transaction(&state.pool, &id).await {
                Ok(()) => (
                    StatusCode::OK,
                    Json(serde_json::json!({"status": "disputed", "transaction_id": id})),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                ),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "transaction not found"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}
