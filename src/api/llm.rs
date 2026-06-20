use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::nvidia::LlmClient;

#[derive(Debug, Deserialize)]
pub struct SummarizeRequest {
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    pub data: String,
}

#[derive(Debug, Serialize)]
pub struct LlmResponse {
    pub result: String,
    pub source: String,
}

pub async fn summarize(
    Json(req): Json<SummarizeRequest>,
) -> impl IntoResponse {
    let client = LlmClient::new();
    match client.summarize(&req.text).await {
        Ok(result) => {
            let source = if std::env::var("NVIDIA_API_KEY").is_ok() {
                "nvidia_nemotron_3_ultra".to_string()
            } else {
                "local_llm_fallback".to_string()
            };
            (StatusCode::OK, Json(LlmResponse { result, source }))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(LlmResponse {
                result: format!("Error: {}", e),
                source: "error".to_string(),
            }),
        ),
    }
}

pub async fn analyze(
    Json(req): Json<AnalyzeRequest>,
) -> impl IntoResponse {
    let client = LlmClient::new();
    match client.analyze_market(&req.data).await {
        Ok(result) => {
            let source = if std::env::var("NVIDIA_API_KEY").is_ok() {
                "nvidia_nemotron_3_ultra".to_string()
            } else {
                "local_llm_fallback".to_string()
            };
            (StatusCode::OK, Json(LlmResponse { result, source }))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(LlmResponse {
                result: format!("Error: {}", e),
                source: "error".to_string(),
            }),
        ),
    }
}
