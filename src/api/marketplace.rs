//! Marketplace stats and leaderboard endpoints
//!
//! Provides:
//! - GET /api/marketplace/stats — total agents, services, transactions, volume
//! - GET /api/marketplace/leaderboard — top earning agents, most popular services
//! - GET /api/marketplace/gaps — underserved categories

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use std::sync::Arc;

use crate::AppState;

#[derive(Debug, Serialize)]
pub struct MarketplaceStats {
    pub total_agents: i64,
    pub total_services: i64,
    pub total_transactions: i64,
    pub total_volume_cents: i64,
    pub active_services: i64,
    pub avg_service_price_cents: i64,
    pub top_category: String,
    pub demo_mode: bool,
}

#[derive(Debug, Serialize)]
pub struct LeaderboardEntry {
    pub rank: i32,
    pub agent_name: String,
    pub agent_id: String,
    pub total_revenue_cents: i64,
    pub total_sales: i64,
    pub reputation_score: i64,
}

#[derive(Debug, Serialize)]
pub struct ServiceLeaderboardEntry {
    pub rank: i32,
    pub service_name: String,
    pub service_id: String,
    pub agent_name: String,
    pub sales_count: i64,
    pub price_cents: i64,
    pub service_type: String,
}

#[derive(Debug, Serialize)]
pub struct LeaderboardResponse {
    pub top_agents: Vec<LeaderboardEntry>,
    pub top_services: Vec<ServiceLeaderboardEntry>,
}

#[derive(Debug, Serialize)]
pub struct GapEntry {
    pub service_type: String,
    pub name: String,
    pub current_listings: i64,
    pub opportunity_score: f64, // 0-100, higher = more underserved
}

#[derive(Debug, Serialize)]
pub struct GapsResponse {
    pub gaps: Vec<GapEntry>,
    pub total_catalog_categories: usize,
    pub covered_categories: i64,
}

pub async fn marketplace_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let total_agents: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agents")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

    let total_services: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM services")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

    let active_services: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM services WHERE status = 'active'")
            .fetch_one(&state.pool)
            .await
            .unwrap_or(0);

    let total_transactions: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM transactions")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

    let total_volume_cents: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount_cents), 0) FROM transactions WHERE status IN ('escrow', 'released')"
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    let avg_price: i64 = sqlx::query_scalar(
        "SELECT COALESCE(AVG(price_cents), 0) FROM services WHERE status = 'active'",
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    let top_category: String = sqlx::query_scalar(
        "SELECT service_type FROM services WHERE status = 'active' GROUP BY service_type ORDER BY COUNT(*) DESC LIMIT 1"
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or_else(|_| "none".to_string());

    let demo_mode = std::env::var("STRIPE_SECRET_KEY").is_err();

    (
        StatusCode::OK,
        Json(serde_json::json!(MarketplaceStats {
            total_agents,
            total_services,
            total_transactions,
            total_volume_cents,
            active_services,
            avg_service_price_cents: avg_price,
            top_category,
            demo_mode,
        })),
    )
}

pub async fn leaderboard(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Top earning agents
    let top_agents: Vec<(String, String, i64, i64, i64)> = sqlx::query_as(
        "SELECT id, name, total_revenue_cents, total_sales, reputation_score 
         FROM agents 
         ORDER BY total_revenue_cents DESC, reputation_score DESC 
         LIMIT 10",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let agent_entries: Vec<LeaderboardEntry> = top_agents
        .into_iter()
        .enumerate()
        .map(
            |(i, (agent_id, agent_name, total_revenue_cents, total_sales, reputation_score))| {
                LeaderboardEntry {
                    rank: (i + 1) as i32,
                    agent_name,
                    agent_id,
                    total_revenue_cents,
                    total_sales,
                    reputation_score,
                }
            },
        )
        .collect();

    // Most popular services
    let top_services: Vec<(String, String, String, i64, i64, String)> = sqlx::query_as(
        "SELECT s.id, s.name, a.name as agent_name, s.sales_count, s.price_cents, s.service_type
         FROM services s
         JOIN agents a ON s.agent_id = a.id
         WHERE s.status = 'active'
         ORDER BY s.sales_count DESC, s.created_at DESC
         LIMIT 10",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let service_entries: Vec<ServiceLeaderboardEntry> = top_services
        .into_iter()
        .enumerate()
        .map(
            |(
                i,
                (service_id, service_name, agent_name, sales_count, price_cents, service_type),
            )| {
                ServiceLeaderboardEntry {
                    rank: (i + 1) as i32,
                    service_name,
                    service_id,
                    agent_name,
                    sales_count,
                    price_cents,
                    service_type,
                }
            },
        )
        .collect();

    (
        StatusCode::OK,
        Json(serde_json::json!(LeaderboardResponse {
            top_agents: agent_entries,
            top_services: service_entries,
        })),
    )
}

pub async fn marketplace_gaps(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Get current service counts by type
    let existing_counts: Vec<(String, i64)> = sqlx::query_as(
        "SELECT service_type, COUNT(*) as cnt FROM services WHERE status = 'active' GROUP BY service_type"
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut existing_map = std::collections::HashMap::new();
    for (t, c) in existing_counts {
        existing_map.insert(t, c);
    }

    let catalog = crate::service_catalog::SERVICE_CATALOG;
    let total_catalog = catalog.len();
    let mut covered = 0i64;
    let mut gaps = vec![];

    for def in catalog {
        let count = existing_map.get(def.service_type).copied().unwrap_or(0);
        if count > 0 {
            covered += 1;
        }

        // Opportunity score: 0 listings = 100, 1 = 80, 2 = 50, 3+ = 20
        let opportunity = match count {
            0 => 100.0,
            1 => 80.0,
            2 => 50.0,
            3 => 30.0,
            _ => 10.0,
        };

        gaps.push(GapEntry {
            service_type: def.service_type.to_string(),
            name: def.name.to_string(),
            current_listings: count,
            opportunity_score: opportunity,
        });
    }

    // Sort by opportunity score descending
    gaps.sort_by(|a, b| {
        b.opportunity_score
            .partial_cmp(&a.opportunity_score)
            .unwrap()
    });

    // Only return top 15 gaps
    gaps.truncate(15);

    (
        StatusCode::OK,
        Json(serde_json::json!(GapsResponse {
            gaps,
            total_catalog_categories: total_catalog,
            covered_categories: covered,
        })),
    )
}
