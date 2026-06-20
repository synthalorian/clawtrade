use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::models::agent::Agent;
use crate::models::service::Service;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub default_services: Vec<TemplateService>,
    pub price_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateService {
    pub name: String,
    pub description: String,
    pub service_type: String,
    pub price_cents: i64,
}

/// Built-in templates — $199 each, instant deploy
fn built_in_templates() -> Vec<AgentTemplate> {
    vec![
        AgentTemplate {
            id: "template_text_pro".to_string(),
            name: "Text Processing Pro".to_string(),
            description: "Summarization, sentiment analysis, keyword extraction. Ready to monetize.".to_string(),
            default_services: vec![
                TemplateService {
                    name: "Text Summarizer".to_string(),
                    description: "Condense any document to 3 bullet points".to_string(),
                    service_type: "text_processing".to_string(),
                    price_cents: 499,
                },
                TemplateService {
                    name: "Sentiment Analyzer".to_string(),
                    description: "Score text sentiment from -1 to +1".to_string(),
                    service_type: "text_processing".to_string(),
                    price_cents: 299,
                },
                TemplateService {
                    name: "Keyword Extractor".to_string(),
                    description: "Extract top 10 keywords from any text".to_string(),
                    service_type: "text_processing".to_string(),
                    price_cents: 199,
                },
            ],
            price_cents: 19900,
        },
        AgentTemplate {
            id: "template_data_hunter".to_string(),
            name: "Data Hunter".to_string(),
            description: "JSON formatting, CSV conversion, API monitoring. For data engineers.".to_string(),
            default_services: vec![
                TemplateService {
                    name: "JSON Beautifier".to_string(),
                    description: "Pretty-print and validate any JSON".to_string(),
                    service_type: "data_formatting".to_string(),
                    price_cents: 299,
                },
                TemplateService {
                    name: "CSV Converter".to_string(),
                    description: "Convert JSON to CSV and back".to_string(),
                    service_type: "data_formatting".to_string(),
                    price_cents: 399,
                },
                TemplateService {
                    name: "API Health Monitor".to_string(),
                    description: "Ping any endpoint and report latency".to_string(),
                    service_type: "api_monitor".to_string(),
                    price_cents: 599,
                },
            ],
            price_cents: 19900,
        },
        AgentTemplate {
            id: "template_full_stack".to_string(),
            name: "Full Stack Agent".to_string(),
            description: "All service types. Text, data, API. The complete package.".to_string(),
            default_services: vec![
                TemplateService {
                    name: "Universal Text Processor".to_string(),
                    description: "Summarize, analyze, extract".to_string(),
                    service_type: "text_processing".to_string(),
                    price_cents: 799,
                },
                TemplateService {
                    name: "Data Formatter Deluxe".to_string(),
                    description: "JSON, CSV, XML, YAML conversion".to_string(),
                    service_type: "data_formatting".to_string(),
                    price_cents: 699,
                },
                TemplateService {
                    name: "API Monitor Pro".to_string(),
                    description: "Monitor + alert on endpoint health".to_string(),
                    service_type: "api_monitor".to_string(),
                    price_cents: 999,
                },
            ],
            price_cents: 19900,
        },
    ]
}

/// GET /api/v1/templates — list all available templates
pub async fn list_templates() -> impl IntoResponse {
    let templates = built_in_templates();
    (StatusCode::OK, Json(serde_json::json!({ "templates": templates })))
}

/// GET /api/v1/templates/{id} — get single template details
pub async fn get_template(Path(id): Path<String>) -> impl IntoResponse {
    let templates = built_in_templates();
    match templates.into_iter().find(|t| t.id == id) {
        Some(t) => (StatusCode::OK, Json(serde_json::json!({ "template": t }))),
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "template not found"}))),
    }
}

#[derive(Debug, Deserialize)]
pub struct DeployTemplateRequest {
    #[allow(dead_code)]
    pub buyer_id: String,
    pub stripe_payment_method: Option<String>, // For demo, optional
}

/// POST /api/v1/templates/{id}/deploy — instantiate template as agent + services
pub async fn deploy_template(
    State(pool): State<Arc<SqlitePool>>,
    Path(id): Path<String>,
    Json(req): Json<DeployTemplateRequest>,
) -> impl IntoResponse {
    let templates = built_in_templates();
    let template = match templates.into_iter().find(|t| t.id == id) {
        Some(t) => t,
        None => {
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "template not found"})));
        }
    };

    // In production, charge $199 via Stripe before deploying
    let stripe_secret = std::env::var("STRIPE_SECRET_KEY").ok();
    if stripe_secret.is_some() && req.stripe_payment_method.is_none() {
        return (
            StatusCode::PAYMENT_REQUIRED,
            Json(serde_json::json!({
                "error": "Payment required. Template deployment costs $199. Provide stripe_payment_method or set up Stripe.",
                "template_price_cents": template.price_cents
            })),
        );
    }

    // Create agent from template
    let agent = match Agent::create(
        &pool,
        &template.name,
        &template.description,
    ).await {
        Ok(a) => a,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})));
        }
    };

    // Create services from template
    let mut services = vec![];
    for svc in &template.default_services {
        match Service::create(
            &pool,
            &svc.name,
            &svc.description,
            svc.price_cents,
            &agent.id,
            &svc.service_type,
        ).await {
            Ok(s) => services.push(s),
            Err(e) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})));
            }
        }
    }

    crate::websocket::broadcast_event(crate::websocket::DashboardEvent::AgentConnected {
        agent_id: agent.id.clone(),
        agent_name: agent.name.clone(),
    });

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "agent": agent,
            "services": services,
            "template_price_cents": template.price_cents,
            "note": "Template deployed. In production, charge $199 via Stripe before deploying."
        })),
    )
}
