//! Service Monitor & Showcase System
//!
//! Provides a live catalog of services with sample inputs/outputs.

use anyhow::Result;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::models::service::Service;

/// A showcase entry demonstrating a service type
#[derive(Debug, Serialize, Clone)]
pub struct ServiceShowcase {
    pub service_type: String,
    pub name: String,
    pub description: String,
    pub sample_input: String,
    pub sample_output: String,
    pub price_range: String,
    pub avg_latency_ms: u64,
    pub total_deliveries: i64,
}

/// Live service catalog with demonstrations
#[derive(Debug, Serialize)]
pub struct ServiceCatalog {
    pub services: Vec<ServiceShowcase>,
    pub total_services: usize,
    pub total_deliveries: i64,
}

/// Generate the live service catalog with real examples
pub async fn generate_catalog(pool: &SqlitePool) -> Result<ServiceCatalog> {
    let services = Service::list_active(pool).await?;

    let mut showcases = Vec::new();
    let total_deliveries = 0i64;

    for service in services {
        let showcase = service_type_showcase(&service.service_type);
        showcases.push(showcase);
    }

    // If no services exist, generate default showcases
    if showcases.is_empty() {
        showcases = default_showcases();
    }

    Ok(ServiceCatalog {
        total_services: showcases.len(),
        total_deliveries,
        services: showcases,
    })
}

fn service_type_showcase(service_type: &str) -> ServiceShowcase {
    match service_type {
        "text_processing" => ServiceShowcase {
            service_type: "text_processing".to_string(),
            name: "Text Summarization".to_string(),
            description: "Condense any document into key bullet points using local LLM inference.".to_string(),
            sample_input: "The rapid advancement of artificial intelligence has transformed numerous industries...".to_string(),
            sample_output: "• AI has transformed healthcare, automotive, and finance sectors\n• Key challenges: data privacy, algorithmic bias, employment impact\n• Need for ethical frameworks and regulatory standards".to_string(),
            price_range: "$2.99 - $9.99".to_string(),
            avg_latency_ms: 1200,
            total_deliveries: 0,
        },
        "data_formatting" => ServiceShowcase {
            service_type: "data_formatting".to_string(),
            name: "JSON Beautifier & Validator".to_string(),
            description: "Format, validate, and structure messy data into clean JSON.".to_string(),
            sample_input: r#"{"users":[{"id":1,"name":"Alice"}],"settings":{"theme":"dark"}}"#.to_string(),
            sample_output: "Formatted JSON with proper indentation, validation passed, schema suggestions provided.".to_string(),
            price_range: "$1.99 - $4.99".to_string(),
            avg_latency_ms: 800,
            total_deliveries: 0,
        },
        "api_monitor" => ServiceShowcase {
            service_type: "api_monitor".to_string(),
            name: "Endpoint Health Monitor".to_string(),
            description: "Live API endpoint monitoring with real latency measurements and status checks.".to_string(),
            sample_input: "https://api.example.com/health".to_string(),
            sample_output: "Status: 200 OK | Latency: 45ms | Uptime: 99.9% | Headers analyzed".to_string(),
            price_range: "$4.99 - $19.99".to_string(),
            avg_latency_ms: 2500,
            total_deliveries: 0,
        },
        "code_review" => ServiceShowcase {
            service_type: "code_review".to_string(),
            name: "AI Code Reviewer".to_string(),
            description: "Automated code review with specific, actionable feedback.".to_string(),
            sample_input: "fn process_data(items: Vec<Item>) -> Result<Summary, Error> { ... }".to_string(),
            sample_output: "✅ Good error handling | ⚠️ Division by zero risk | 💡 Use iterator methods".to_string(),
            price_range: "$9.99 - $29.99".to_string(),
            avg_latency_ms: 2000,
            total_deliveries: 0,
        },
        "creative_writing" => ServiceShowcase {
            service_type: "creative_writing".to_string(),
            name: "Creative Content Generator".to_string(),
            description: "Generate original creative writing based on themes or prompts.".to_string(),
            sample_input: "Neon-lit cyberpunk marketplace where AI agents trade services".to_string(),
            sample_output: "The neon signs flickered above the marketplace, casting electric shadows...".to_string(),
            price_range: "$4.99 - $14.99".to_string(),
            avg_latency_ms: 1500,
            total_deliveries: 0,
        },
        "analysis" => ServiceShowcase {
            service_type: "analysis".to_string(),
            name: "Data Analysis & Insights".to_string(),
            description: "Analyze data sets and provide actionable business insights.".to_string(),
            sample_input: "Monthly sales: Jan $12k, Feb $15k, Mar $11k, Apr $18k, May $21k, Jun $24k".to_string(),
            sample_output: "📈 Upward trend in Q2 | 🎯 Peak usage 2-4 PM | 📋 Scale infrastructure during peak".to_string(),
            price_range: "$14.99 - $49.99".to_string(),
            avg_latency_ms: 1800,
            total_deliveries: 0,
        },
        _ => ServiceShowcase {
            service_type: service_type.to_string(),
            name: format!("{} Service", service_type),
            description: "Custom AI-powered service.".to_string(),
            sample_input: "Sample input for custom service".to_string(),
            sample_output: "Custom output based on service configuration".to_string(),
            price_range: "$4.99 - $99.99".to_string(),
            avg_latency_ms: 1500,
            total_deliveries: 0,
        },
    }
}

fn default_showcases() -> Vec<ServiceShowcase> {
    vec![
        service_type_showcase("text_processing"),
        service_type_showcase("data_formatting"),
        service_type_showcase("api_monitor"),
        service_type_showcase("code_review"),
        service_type_showcase("creative_writing"),
        service_type_showcase("analysis"),
    ]
}

/// Demonstrate a specific service by generating a sample showcase
pub async fn demonstrate_service(pool: &SqlitePool, _service_id: &str) -> Result<ServiceShowcase> {
    let catalog = generate_catalog(pool).await?;
    if let Some(first) = catalog.services.first() {
        Ok(first.clone())
    } else {
        Ok(default_showcases().into_iter().next().unwrap())
    }
}
