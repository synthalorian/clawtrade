use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;

use crate::AppState;

pub mod activity;
pub mod agents;
pub mod deliverables;
pub mod hosting;
pub mod llm;
pub mod monitor;
pub mod pricing;
pub mod reviews;
pub mod services;
pub mod stripe;
pub mod templates;
pub mod transactions;

/// Build the API router with all routes wired up.
/// This is the single source of truth for API route configuration.
pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        // Services
        .route("/api/services", get(services::list_services).post(services::create_service))
        .route("/api/services/{id}", get(services::get_service))
        // Agents
        .route("/api/agents", get(agents::list_agents).post(agents::create_agent))
        .route("/api/agents/{id}", get(agents::get_agent))
        // Transactions
        .route("/api/transactions", get(transactions::list_transactions).post(transactions::create_transaction))
        .route("/api/transactions/{id}", get(transactions::get_transaction))
        .route("/api/transactions/{id}/release", post(transactions::release_escrow))
        .route("/api/transactions/{id}/dispute", post(transactions::dispute_transaction))
        // Stripe
        .route("/api/checkout", get(stripe::create_checkout))
        .route("/api/demo/purchase", post(stripe::demo_purchase))
        .route("/api/webhooks/stripe", post(stripe::stripe_webhook))
        .route("/api/stripe/connect", post(stripe::create_connect_account))
        .route("/api/stripe/account_link", post(stripe::create_account_link))
        // Deliverables
        .route("/api/deliverables/{id}", get(deliverables::get_deliverable))
        .route("/api/services/{id}/execute", post(deliverables::execute_service))
        // Reviews
        .route("/api/reviews", post(reviews::create_review))
        .route("/api/agents/{id}/reviews", get(reviews::list_reviews))
        // LLM
        .route("/api/llm/summarize", post(llm::summarize))
        .route("/api/llm/analyze", post(llm::analyze))
        // Hosting API
        .route("/api/v1/agents/spawn", post(hosting::spawn_agent))
        .route("/api/v1/agents/{id}/status", get(hosting::agent_status))
        .route("/api/v1/agents/{id}/stop", post(hosting::stop_agent))
        .route("/api/v1/agents/{id}/logs", get(hosting::agent_logs))
        // Pricing / Market
        .route("/api/v1/market/trends", get(pricing::market_trends))
        .route("/api/v1/pricing/recommendations", get(pricing::pricing_recommendations))
        // Templates
        .route("/api/v1/templates", get(templates::list_templates))
        .route("/api/v1/templates/{id}", get(templates::get_template))
        .route("/api/v1/templates/{id}/deploy", post(templates::deploy_template))
        // Monitor / Agent Loop
        .route("/api/monitor/catalog", get(monitor::get_catalog))
        .route("/api/monitor/demonstrate/{service_id}", get(monitor::demonstrate))
        .route("/api/monitor/stats", get(monitor::marketplace_stats))
        .route("/api/agents/tick", post(monitor::agent_tick))
        .route("/api/agents/states", get(monitor::agent_states))
        .route("/api/agents/{id}/create-service", post(monitor::agent_create_service))
        .route("/api/agents/{id}/review", post(monitor::agent_leave_review))
        // Activity
        .route("/api/activity", get(activity::global_activity))
        .route("/api/activity/stats", get(activity::activity_stats))
        .route("/api/activity/agent/{id}", get(activity::agent_activity))
        .route("/api/activity/tx/{id}", get(activity::tx_activity))
        // WebSocket
        .route("/ws", get(crate::websocket::ws_handler))
        .with_state(state)
}
