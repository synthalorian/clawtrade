use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::models::review::Review;
use crate::models::transaction::Transaction;

#[derive(Debug, Deserialize)]
pub struct CreateReviewRequest {
    pub transaction_id: String,
    pub agent_id: String,
    pub rating: i64,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct ReviewListResponse {
    pub reviews: Vec<Review>,
    pub average_rating: Option<f64>,
    pub total_reviews: i64,
}

pub async fn create_review(
    State(pool): State<Arc<SqlitePool>>,
    Json(req): Json<CreateReviewRequest>,
) -> impl IntoResponse {
    // Validate rating
    if req.rating < 1 || req.rating > 5 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "rating must be between 1 and 5"})),
        );
    }

    // Verify transaction exists and is paid
    match Transaction::get_by_id(&pool, &req.transaction_id).await {
        Ok(Some(tx)) => {
            if tx.status != "paid" && tx.status != "released" && tx.status != "escrow" {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "can only review paid transactions"})),
                );
            }
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "transaction not found"})),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            );
        }
    }

    match Review::create(
        &pool,
        &req.transaction_id,
        &req.agent_id,
        req.rating,
        req.comment.as_deref(),
    )
    .await
    {
        Ok(review) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "review": review })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn list_reviews(
    State(pool): State<Arc<SqlitePool>>,
    Path(agent_id): Path<String>,
) -> impl IntoResponse {
    let reviews = match Review::list_by_agent(&pool, &agent_id).await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
    };

    let avg = match Review::get_average_rating(&pool, &agent_id).await {
        Ok(a) => a,
        Err(_) => None,
    };

    let total = match Review::count_by_agent(&pool, &agent_id).await {
        Ok(c) => c,
        Err(_) => 0,
    };

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "reviews": reviews,
            "average_rating": avg,
            "total_reviews": total,
        })),
    )
}
