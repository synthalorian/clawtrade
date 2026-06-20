use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::models::agent::Agent;
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

#[derive(Debug, Deserialize)]
pub struct ConnectAccountRequest {
    pub agent_id: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct ConnectAccountResponse {
    pub account_id: String,
    pub onboarding_url: String,
}

#[derive(Debug, Deserialize)]
pub struct AccountLinkRequest {
    pub account_id: String,
}

#[derive(Debug, Serialize)]
pub struct AccountLinkResponse {
    pub url: String,
}

fn stripe_secret() -> Option<String> {
    std::env::var("STRIPE_SECRET_KEY").ok()
}


pub async fn create_checkout(
    State(pool): State<Arc<SqlitePool>>,
    Query(req): Query<CreateCheckoutRequest>,
) -> impl IntoResponse {
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

    let stripe_secret = match stripe_secret() {
        Some(k) => k,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": "STRIPE_SECRET_KEY not configured", "demo_mode": true})),
            )
        }
    };

    let client = reqwest::Client::new();

    let seller = match Agent::get_by_id(&pool, &service.agent_id).await {
        Ok(Some(a)) => a,
        _ => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "seller not found"})),
            )
        }
    };

    let mut params: Vec<(&str, String)> = vec![
        ("payment_method_types[]", "card".to_string()),
        ("line_items[0][price_data][currency]", "usd".to_string()),
        (
            "line_items[0][price_data][product_data][name]",
            service.name.clone(),
        ),
        (
            "line_items[0][price_data][unit_amount]",
            service.price_cents.to_string(),
        ),
        ("line_items[0][quantity]", "1".to_string()),
        ("mode", "payment".to_string()),
        (
            "success_url",
            format!("http://localhost:8746/success?tx_id={}", tx.id),
        ),
        (
            "cancel_url",
            format!("http://localhost:8746/cancel?tx_id={}", tx.id),
        ),
        ("metadata[transaction_id]", tx.id.clone()),
        ("metadata[service_id]", req.service_id.clone()),
    ];

    if let Some(stripe_account_id) = seller.stripe_account_id {
        let platform_fee = (service.price_cents as f64 * 0.10).round() as i64;
        params.push(("transfer_data[destination]", stripe_account_id));
        params.push(("application_fee_amount", platform_fee.to_string()));
    }

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
                // First try to find by stripe_session_id
                let tx_result = sqlx::query_as::<_, Transaction>(
                    "SELECT * FROM transactions WHERE stripe_session_id = ?",
                )
                .bind(session_id)
                .fetch_optional(&*pool)
                .await;

                // If not found, check metadata for transaction_id (demo mode)
                let tx = match tx_result {
                    Ok(Some(tx)) => Some(tx),
                    _ => {
                        if let Some(metadata) = session.get("metadata") {
                            if let Some(tx_id) = metadata["transaction_id"].as_str() {
                                sqlx::query_as::<_, Transaction>(
                                    "SELECT * FROM transactions WHERE id = ?",
                                )
                                .bind(tx_id)
                                .fetch_optional(&*pool)
                                .await
                                .unwrap_or(None)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                };

                if let Some(ref tx) = tx {
                    // Mark as paid
                    if let Err(e) = Transaction::mark_paid_by_stripe_session(&pool, session_id).await {
                        // If no stripe_session_id, update directly by tx id
                        let _ = sqlx::query(
                            "UPDATE transactions SET status = 'paid', updated_at = ? WHERE id = ?",
                        )
                        .bind(Utc::now())
                        .bind(&tx.id)
                        .execute(&*pool)
                        .await;

                        // Increment seller stats
                        let _ = crate::models::agent::Agent::increment_sales(&pool, &tx.seller_id, tx.amount_cents).await;
                    }

                    // Trigger service delivery
                    if let Err(e) = crate::delivery::trigger_delivery(&pool, &tx.id).await {
                        eprintln!("[delivery] failed for tx {}: {}", tx.id, e);
                    }

                    // Broadcast payment confirmation
                    crate::websocket::broadcast_event(crate::websocket::DashboardEvent::PaymentConfirmed {
                        tx_id: tx.id.clone(),
                        service_name: tx.service_id.clone(),
                        amount_cents: tx.amount_cents,
                    });
                }
            }
        }
    }

    (StatusCode::OK, Json(serde_json::json!({"received": true})))
}

pub async fn create_connect_account(
    State(pool): State<Arc<SqlitePool>>,
    Json(req): Json<ConnectAccountRequest>,
) -> impl IntoResponse {
    let stripe_secret = match stripe_secret() {
        Some(k) => k,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": "STRIPE_SECRET_KEY not configured"})),
            )
        }
    };

    let agent = match Agent::get_by_id(&pool, &req.agent_id).await {
        Ok(Some(a)) => a,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "agent not found"})),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        }
    };

    if let Some(existing) = agent.stripe_account_id {
        let client = reqwest::Client::new();
        let link_res = match client
            .post("https://api.stripe.com/v1/account_links")
            .header("Authorization", format!("Bearer {}", stripe_secret))
            .form(&vec![
                ("account", existing.clone()),
                ("refresh_url", "http://localhost:8746/agents".to_string()),
                ("return_url", "http://localhost:8746/agents".to_string()),
                ("type", "account_onboarding".to_string()),
            ])
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("stripe link failed: {}", e)})),
                )
            }
        };

        let link_data: serde_json::Value = match link_res.json().await {
            Ok(d) => d,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("stripe link parse failed: {}", e)})),
                )
            }
        };

        if let Some(url) = link_data["url"].as_str() {
            return (
                StatusCode::OK,
                Json(serde_json::json!({
                    "account_id": existing,
                    "onboarding_url": url,
                })),
            );
        } else {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": link_data})),
            );
        }
    }

    let client = reqwest::Client::new();
    let res = match client
        .post("https://api.stripe.com/v1/accounts")
        .header("Authorization", format!("Bearer {}", stripe_secret))
        .form(&vec![
            ("type", "express".to_string()),
            ("email", req.email.clone()),
            ("capabilities[transfers][requested]", "true".to_string()),
            ("capabilities[card_payments][requested]", "true".to_string()),
        ])
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("stripe account creation failed: {}", e)})),
            )
        }
    };

    let account_data: serde_json::Value = match res.json().await {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("stripe parse failed: {}", e)})),
            )
        }
    };

    let account_id = match account_data["id"].as_str() {
        Some(id) => id.to_string(),
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": account_data})),
            )
        }
    };

    if let Err(e) = Agent::update_stripe_account(&pool, &req.agent_id, &account_id).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        );
    }

    let link_res = match client
        .post("https://api.stripe.com/v1/account_links")
        .header("Authorization", format!("Bearer {}", stripe_secret))
        .form(&vec![
            ("account", account_id.clone()),
            ("refresh_url", "http://localhost:8746/agents".to_string()),
            ("return_url", "http://localhost:8746/agents".to_string()),
            ("type", "account_onboarding".to_string()),
        ])
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("stripe link failed: {}", e)})),
            )
        }
    };

    let link_data: serde_json::Value = match link_res.json().await {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("stripe link parse failed: {}", e)})),
            )
        }
    };

    if let Some(url) = link_data["url"].as_str() {
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "account_id": account_id,
                "onboarding_url": url,
            })),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": link_data})),
        )
    }
}

pub async fn create_account_link(
    State(_pool): State<Arc<SqlitePool>>,
    Json(req): Json<AccountLinkRequest>,
) -> impl IntoResponse {
    let stripe_secret = match stripe_secret() {
        Some(k) => k,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": "STRIPE_SECRET_KEY not configured"})),
            )
        }
    };

    let client = reqwest::Client::new();
    let res = match client
        .post("https://api.stripe.com/v1/account_links")
        .header("Authorization", format!("Bearer {}", stripe_secret))
        .form(&vec![
            ("account", req.account_id.clone()),
            ("refresh_url", "http://localhost:8746/agents".to_string()),
            ("return_url", "http://localhost:8746/agents".to_string()),
            ("type", "account_onboarding".to_string()),
        ])
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("stripe link failed: {}", e)})),
            )
        }
    };

    let data: serde_json::Value = match res.json().await {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("stripe parse failed: {}", e)})),
            )
        }
    };

    if let Some(url) = data["url"].as_str() {
        (
            StatusCode::OK,
            Json(serde_json::json!({"url": url})),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": data})),
        )
    }
}
