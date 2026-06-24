//! Rate Limiting Layer for ClawTrade
//!
//! Protects the marketplace from abuse:
//! - DDoS / brute force on public tunnel
//! - Scrapers hitting API endpoints repeatedly
//! - LLM inference spam (costs GPU time)
//!
//! Uses governor crate with per-IP tracking.

use std::net::SocketAddr;
use std::sync::Arc;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use governor::clock::DefaultClock;
use governor::state::keyed::DefaultKeyedStateStore;
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

/// Global rate limiter: 30 requests per minute per IP
pub type ClawRateLimiter = RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>;

/// Create the default rate limiter
pub fn create_rate_limiter() -> Arc<ClawRateLimiter> {
    // 30 requests per minute per IP
    let quota = Quota::per_minute(NonZeroU32::new(30).unwrap());
    Arc::new(RateLimiter::keyed(quota))
}

/// Create a stricter rate limiter for expensive endpoints (LLM inference)
pub fn create_strict_rate_limiter() -> Arc<ClawRateLimiter> {
    // 10 requests per minute per IP (LLM calls are expensive)
    let quota = Quota::per_minute(NonZeroU32::new(10).unwrap());
    Arc::new(RateLimiter::keyed(quota))
}

/// Extract client IP from request
pub fn extract_ip(req: &axum::extract::Request) -> String {
    // Check X-Forwarded-For header (from tunnel/proxy)
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(val) = forwarded.to_str() {
            // Take first IP in chain (client, not proxy)
            return val.split(',').next().unwrap_or("unknown").trim().to_string();
        }
    }
    
    // Check X-Real-IP header
    if let Some(real_ip) = req.headers().get("x-real-ip") {
        if let Ok(val) = real_ip.to_str() {
            return val.to_string();
        }
    }
    
    // Fall back to socket address
    if let Some(addr) = req.extensions().get::<SocketAddr>() {
        return addr.ip().to_string();
    }
    
    "unknown".to_string()
}

/// Rate limit middleware — 30 req/min per IP
pub async fn rate_limit_middleware(
    req: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    // Get or create limiter from request extensions
    let limiter = req.extensions().get::<Arc<ClawRateLimiter>>().cloned();
    
    if let Some(limiter) = limiter {
        let ip = extract_ip(&req);
        
        match limiter.check_key(&ip) {
            Ok(()) => {
                // Allowed — continue
                Ok(next.run(req).await)
            }
            Err(_) => {
                // Rate limited
                eprintln!("[rate_limit] IP {} rate limited", ip);
                Err(axum::http::StatusCode::TOO_MANY_REQUESTS)
            }
        }
    } else {
        // No limiter configured — allow (shouldn't happen)
        Ok(next.run(req).await)
    }
}

/// Strict rate limit middleware — 10 req/min per IP (for LLM endpoints)
pub async fn strict_rate_limit_middleware(
    req: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let limiter = req.extensions().get::<Arc<ClawRateLimiter>>().cloned();
    
    if let Some(limiter) = limiter {
        let ip = extract_ip(&req);
        
        match limiter.check_key(&ip) {
            Ok(()) => Ok(next.run(req).await),
            Err(_) => {
                eprintln!("[rate_limit] IP {} strictly rate limited", ip);
                Err(axum::http::StatusCode::TOO_MANY_REQUESTS)
            }
        }
    } else {
        Ok(next.run(req).await)
    }
}

/// Middleware that adds rate limiter to request extensions
pub async fn add_rate_limiter_extension(
    limiter: Arc<ClawRateLimiter>,
    req: Request,
    next: Next,
) -> Response {
    let mut req = req;
    req.extensions_mut().insert(limiter);
    next.run(req).await
}
