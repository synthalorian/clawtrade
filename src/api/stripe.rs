use anyhow::Result;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::models::service::Service;
use crate::models::transaction::Transaction;

#[derive(Debug, Deserialize)]
pub struct CreateCheckoutRequest {
    pub service_id: String,
    pub buyer_id: String,
}

#[derive(Debug, Serialize)]
pub struct CheckoutResponse {
    pub checkout_url: String,
    pub transaction_id: String,
}

#[derive(Debug, Deserialize)]
pub struct WebhookPayload {
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: serde_json::Value,
}

pub async fn create_checkout(
    State(pool): State<Arc<SqlitePool>>,
    Query(req): Query<CreateCheckoutRequest>,
) -> impl IntoResponse {
    // 1. Get service
    let service = match Service::get_by_id(&pool, &req.service_id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": format!("service {} not found", req.service_id)})),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("db error: {}", e)})),
            )
        }
    };

    // 2. Create transaction (pending)
    let tx = match Transaction::create(
        &pool,
        &req.service_id,
        &req.buyer_id,
        &service.agent_id,
        service.price_cents,
    )
    .await
    {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        }
    };

    // 3. Call Stripe to create checkout session
    let stripe_secret = match std::env::var("STRIPE_SECRET_KEY") {
        Ok(k) => k,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": "STRIPE_SECRET_KEY not configured", "demo_mode": true})),
            )
        }
    };

    let client = reqwest::Client::new();
    let params = [
        ("payment_method_types[]", "card"),
        ("line_items[0][price_data][currency]", "usd"),
        (
            "line_items[0][price_data][product_data][name]",
            &service.name,
        ),
        (
            "line_items[0][price_data][unit_amount]",
            &service.price_cents.to_string(),
        ),
        ("line_items[0][quantity]", "1"),
        ("mode", "payment"),
        (
            "success_url",
            &format!("http://localhost:8746/success?tx_id={}", tx.id),
        ),
        (
            "cancel_url",
            &format!("http://localhost:8746/cancel?tx_id={}", tx.id),
        ),
        ("metadata[transaction_id]", &tx.id),
        ("metadata[service_id]", &req.service_id),
    ];

    let res = match client
        .post("https://api.stripe.com/v1/checkout/sessions")
        .header("Authorization", format!("Bearer {}", stripe_secret))
        .form(&params)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("stripe request failed: {}", e)})),
            )
        }
    };

    let stripe_data: serde_json::Value = match res.json().await {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("stripe parse failed: {}", e)})),
            )
        }
    };

    if let Some(url) = stripe_data["url"].as_str() {
        let session_id = stripe_data["id"].as_str().unwrap_or("").to_string();
        // Update transaction with stripe session id
        if let Err(e) = Transaction::update_stripe_session(&pool, &tx.id, &session_id).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            );
        }
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "checkout_url": url,
                "transaction_id": tx.id,
            })),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": stripe_data})),
        )
    }
}

pub async fn stripe_webhook(
    State(pool): State<Arc<SqlitePool>>,
    Json(payload): Json<WebhookPayload>,
) -> impl IntoResponse {
    if payload.event_type == "checkout.session.completed" {
        if let Some(session) = payload.data.get("object") {
            let session_id = session["id"].as_str().unwrap_or("");
            let payment_status = session["payment_status"].as_str().unwrap_or("");

            if payment_status == "paid" {
                if let Err(e) =
                    Transaction::mark_paid_by_stripe_session(&pool, session_id).await
                {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": e.to_string()})),
                    );
                }
            }
        }
    }

    (StatusCode::OK, Json(serde_json::json!({"received": true})))
}
