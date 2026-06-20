use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::models::service::Service;
use crate::models::transaction::Transaction;

#[derive(Debug, Serialize)]
pub struct MarketTrend {
    pub service_type: String,
    pub avg_price_cents: i64,
    pub service_count: i64,
    pub sales_velocity: i64, // paid transactions in last 24h
}

#[derive(Debug, Serialize)]
pub struct PricingRecommendation {
    pub service_id: String,
    pub current_price_cents: i64,
    pub recommended_price_cents: i64,
    pub reason: String,
}

/// GET /api/v1/market/trends — aggregate market data
pub async fn market_trends(
    State(pool): State<Arc<SqlitePool>>,
) -> impl IntoResponse {
    let services = match Service::list(&pool).await {
        Ok(s) => s,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    };

    let transactions = match Transaction::list(&pool).await {
        Ok(t) => t,
        Err(_) => vec![],
    };

    let mut trends: std::collections::HashMap<String, (Vec<i64>, i64)> = std::collections::HashMap::new();

    for svc in &services {
        let entry = trends.entry(svc.service_type.clone()).or_insert((vec![], 0));
        entry.0.push(svc.price_cents);
    }

    let now = chrono::Utc::now();
    let day_ago = now - chrono::Duration::hours(24);

    for tx in &transactions {
        if tx.status == "paid" || tx.status == "escrow" || tx.status == "released" {
            if tx.created_at > day_ago {
                if let Some(entry) = trends.get_mut(&tx.service_id) {
                    entry.1 += 1;
                }
            }
        }
    }

    let result: Vec<MarketTrend> = trends.into_iter().map(|(service_type, (prices, velocity))| {
        let avg = if !prices.is_empty() { prices.iter().sum::<i64>() / prices.len() as i64 } else { 0 };
        MarketTrend {
            service_type,
            avg_price_cents: avg,
            service_count: prices.len() as i64,
            sales_velocity: velocity,
        }
    }).collect();

    (StatusCode::OK, Json(serde_json::json!({ "trends": result })))
}

/// GET /api/v1/pricing/recommendations — per-service pricing advice
pub async fn pricing_recommendations(
    State(pool): State<Arc<SqlitePool>>,
) -> impl IntoResponse {
    let services = match Service::list(&pool).await {
        Ok(s) => s,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    };

    let transactions = match Transaction::list(&pool).await {
        Ok(t) => t,
        Err(_) => vec![],
    };

    let mut type_prices: std::collections::HashMap<String, Vec<i64>> = std::collections::HashMap::new();
    let mut type_sales: std::collections::HashMap<String, i64> = std::collections::HashMap::new();

    for svc in &services {
        type_prices.entry(svc.service_type.clone()).or_default().push(svc.price_cents);
    }

    let now = chrono::Utc::now();
    let day_ago = now - chrono::Duration::hours(24);
    let hour_ago = now - chrono::Duration::hours(1);

    for tx in &transactions {
        if tx.status == "paid" || tx.status == "escrow" || tx.status == "released" {
            if tx.created_at > day_ago {
                *type_sales.entry(tx.service_id.clone()).or_insert(0) += 1;
            }
        }
    }

    let mut recommendations = vec![];

    for svc in &services {
        let avg_price = type_prices.get(&svc.service_type)
            .map(|p| if !p.is_empty() { p.iter().sum::<i64>() / p.len() as i64 } else { svc.price_cents })
            .unwrap_or(svc.price_cents);

        let same_type_count = type_prices.get(&svc.service_type).map(|p| p.len() as i64).unwrap_or(0);
        let sales_24h = type_sales.get(&svc.id).copied().unwrap_or(0);
        let sales_1h = transactions.iter()
            .filter(|t| t.service_id == svc.id && t.created_at > hour_ago && (t.status == "paid" || t.status == "escrow" || t.status == "released"))
            .count() as i64;

        let (recommended, reason) = if same_type_count >= 3 {
            // Undercut by 10%
            let new_price = (avg_price as f64 * 0.9) as i64;
            (new_price, format!("3+ competitors. Undercut avg by 10% (avg: ${}.{}", avg_price / 100, avg_price % 100))
        } else if same_type_count == 1 {
            // Premium by 20%
            let new_price = (avg_price as f64 * 1.2) as i64;
            (new_price, format!("Only service of type. Premium 20% over avg (avg: ${}.{}", avg_price / 100, avg_price % 100))
        } else if sales_24h == 0 {
            // No sales in 24h, drop 10%
            let new_price = (svc.price_cents as f64 * 0.9) as i64;
            (new_price.max(50), "No sales in 24h. Drop 10% to stimulate demand.".to_string())
        } else if sales_1h >= 5 {
            // High demand, raise 15%
            let new_price = (svc.price_cents as f64 * 1.15) as i64;
            (new_price, format!("High demand ({} sales/hr). Raise 15%.", sales_1h))
        } else {
            (svc.price_cents, "Market price is optimal. No change recommended.".to_string())
        };

        recommendations.push(PricingRecommendation {
            service_id: svc.id.clone(),
            current_price_cents: svc.price_cents,
            recommended_price_cents: recommended,
            reason,
        });
    }

    (StatusCode::OK, Json(serde_json::json!({ "recommendations": recommendations })))
}
