//! Autonomous Agent Interaction Engine
//!
//! Enables agents to autonomously interact with the marketplace:
//! - Discover services based on needs and reputation
//! - Make purchase decisions
//! - Execute service workflows
//! - Leave reviews and build reputation
//! - Trade with other agents
//!
//! All randomness is deterministic (hash-based) to ensure Send-safety
//! in async Axum handlers — no thread_rng across await points.

use anyhow::Result;
use serde::Serialize;
use sqlx::SqlitePool;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::models::agent::Agent;
use crate::models::service::Service;
use crate::models::transaction::Transaction;
use crate::AppState;

/// An agent's current state in the marketplace
#[derive(Debug, Clone, Serialize)]
pub struct AgentState {
    pub agent_id: String,
    pub name: String,
    pub balance_cents: i64,
    pub reputation: i64,
    pub skills: Vec<String>,
    pub needs: Vec<String>,
    pub mood: AgentMood,
    pub recent_purchases: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub enum AgentMood {
    Shopping,
    Selling,
    Exploring,
    Negotiating,
    Satisfied,
    Frustrated,
}

/// Result of an agent interaction
#[derive(Debug, Clone, Serialize)]
pub struct InteractionResult {
    pub interaction_type: String,
    pub agent_id: String,
    pub target_id: Option<String>,
    pub success: bool,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Deterministic pseudo-random value from a string seed.
/// Returns a value in range [0, 1) using a simple hash-based approach.
fn det_rand(seed: &str) -> f64 {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    let hash = hasher.finish();
    (hash as f64) / (u64::MAX as f64 + 1.0)
}

/// Deterministic choice from a slice using a seed.
fn det_choice<T: Clone>(items: &[T], seed: &str) -> Option<T> {
    if items.is_empty() {
        return None;
    }
    let idx = (det_rand(seed) * items.len() as f64) as usize % items.len();
    Some(items[idx].clone())
}

/// The agent loop engine — drives autonomous marketplace activity
pub struct AgentLoop {
    pub pool: SqlitePool,
}

impl AgentLoop {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { pool: state.pool.clone() }
    }

    /// Initialize agent states from database
    pub async fn get_states(&self) -> Result<HashMap<String, AgentState>> {
        let agents = Agent::list(&self.pool).await?;
        let mut states = HashMap::new();
        for agent in agents {
            let needs = generate_agent_needs(&agent.id);
            let skills = vec![
                "text_processing".to_string(),
                "data_formatting".to_string(),
                "analysis".to_string(),
            ];

            states.insert(
                agent.id.clone(),
                AgentState {
                    agent_id: agent.id.clone(),
                    name: agent.name.clone(),
                    balance_cents: 10000,
                    reputation: agent.reputation_score,
                    skills,
                    needs,
                    mood: AgentMood::Exploring,
                    recent_purchases: vec![],
                },
            );
        }
        Ok(states)
    }

    /// Run one tick of the agent loop — each agent takes an action
    pub async fn tick(&self) -> Result<Vec<InteractionResult>> {
        let mut results = vec![];
        
        // Increment tick counters for all active services
        let _ = Service::increment_tick_counters(&self.pool).await;
        
        // Retire stale services (no sales in 20 ticks)
        match Service::retire_stale_services(&self.pool, 20).await {
            Ok(retired) => {
                for name in &retired {
                    eprintln!("[agent_loop] Retired stale service: {}", name);
                }
            }
            Err(e) => eprintln!("[agent_loop] Failed to retire stale services: {}", e),
        }
        
        let agents = Agent::list(&self.pool).await?;

        for agent in agents {
            if let Some(result) = self.agent_action(&agent).await? {
                results.push(result);
            }
        }

        Ok(results)
    }

    /// A single agent takes an action based on its current state.
    /// Agents can: create services (sell), buy services, or leave reviews.
    /// The action is deterministic based on agent ID hash for reproducibility.
    async fn agent_action(&self, agent: &Agent) -> Result<Option<InteractionResult>> {
        let services = Service::list_active(&self.pool).await?;

        // Deterministic: should this agent act at all this tick?
        // Use a time-based seed so agents act more frequently
        let tick_seed = format!("{}-tick-{}", agent.id, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() / 10);
        let should_act = det_rand(&tick_seed) < 0.4; // 40% chance to act per tick
        if !should_act {
            return Ok(None);
        }

        // Deterministic: what action? 
        // If no services exist, always sell. Otherwise weighted random.
        let action_seed = format!("{}-action-{}", agent.id, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() / 10);
        let action_choice = det_rand(&action_seed);

        if services.is_empty() || action_choice < 0.35 {
            // 35% chance: SELL — create a new service (or always if no services)
            self.agent_sell(agent).await
        } else if action_choice < 0.75 {
            // 40% chance: BUY — purchase a service
            self.agent_buy(agent, &services).await
        } else {
            // 25% chance: REVIEW — leave a review
            self.agent_review(agent).await
        }
    }

    /// Agent browses services and potentially buys one
    async fn agent_buy(&self, agent: &Agent, services: &[Service]) -> Result<Option<InteractionResult>> {
        if services.is_empty() {
            return Ok(None);
        }

        // Filter: don't buy from self, and respect reputation
        let candidates: Vec<&Service> = services
            .iter()
            .filter(|s| s.agent_id != agent.id)
            .collect();

        if candidates.is_empty() {
            return Ok(None);
        }

        // Deterministic choice
        let seed = format!("{}-buy-{}", agent.id, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs());
        let service = det_choice(&candidates, &seed).unwrap_or(candidates[0]);

        // Check seller reputation
        let seller = Agent::get_by_id(&self.pool, &service.agent_id).await?;
        let seller_rep = seller.as_ref().map(|s| s.reputation_score).unwrap_or(0);

        if seller_rep < 10 {
            let skip = agent.id.chars().nth(1).map(|c| c as u32 % 2 == 0).unwrap_or(false);
            if skip {
                return Ok(Some(InteractionResult {
                    interaction_type: "browse".to_string(),
                    agent_id: agent.id.clone(),
                    target_id: Some(service.agent_id.clone()),
                    success: false,
                    message: format!("{} skipped {} due to low seller reputation", agent.name, service.name),
                    details: None,
                }));
            }
        }

        // Create transaction (demo purchase)
        match Transaction::create(
            &self.pool,
            &service.id,
            &agent.id,
            &service.agent_id,
            service.price_cents,
        ).await {
            Ok(tx) => {
                // Record the sale on the service
                let _ = Service::record_sale(&self.pool, &service.id).await;
                // Log the purchase activity
                let _ = crate::models::activity_log::ActivityLog::create(
                    &self.pool,
                    &agent.id,
                    &agent.name,
                    "purchase",
                    Some(&tx.id),
                    Some("transaction"),
                    Some(&service.name),
                    Some(service.price_cents),
                    "completed",
                    Some(&format!(
                        "Bought {} from {} for ${:.2}",
                        service.name,
                        seller.as_ref().map(|s| s.name.as_str()).unwrap_or("unknown"),
                        service.price_cents as f64 / 100.0
                    )),
                ).await;

                // Broadcast WebSocket event
                crate::websocket::broadcast_event(crate::websocket::DashboardEvent::PurchaseInitiated {
                    tx_id: tx.id.clone(),
                    service_name: service.name.clone(),
                    buyer_id: agent.id.clone(),
                });

                Ok(Some(InteractionResult {
                    interaction_type: "purchase".to_string(),
                    agent_id: agent.id.clone(),
                    target_id: Some(service.agent_id.clone()),
                    success: true,
                    message: format!(
                        "{} ({}) bought {} from {} ({}) for ${:.2}",
                        agent.name,
                        &agent.id[..8.min(agent.id.len())],
                        service.name,
                        seller.as_ref().map(|s| s.name.as_str()).unwrap_or("unknown"),
                        seller.as_ref().map(|s| &s.id[..8.min(s.id.len())]).unwrap_or("unknown"),
                        service.price_cents as f64 / 100.0
                    ),
                    details: Some(serde_json::json!({
                        "transaction_id": tx.id,
                        "service_id": service.id,
                        "price_cents": service.price_cents,
                    })),
                }))
            }
            Err(e) => {
                Ok(Some(InteractionResult {
                    interaction_type: "purchase_failed".to_string(),
                    agent_id: agent.id.clone(),
                    target_id: Some(service.agent_id.clone()),
                    success: false,
                    message: format!("{} failed to buy {}: {}", agent.name, service.name, e),
                    details: None,
                }))
            }
        }
    }

    /// Agent creates a new service to sell using the service catalog
    async fn agent_sell(&self, agent: &Agent) -> Result<Option<InteractionResult>> {
        use crate::service_catalog::{find_marketplace_gaps, is_duplicate};

        // Get existing services to check for duplicates and gaps
        let existing_services = Service::list(&self.pool).await?;
        let existing_types: Vec<String> = existing_services.iter().map(|s| s.service_type.clone()).collect();

        // Find marketplace gaps — services with fewest listings
        let gaps = find_marketplace_gaps(&existing_types);

        // 60% chance: fill a gap, 40% chance: compete in existing category
        let seed = format!("{}-sell-strategy-{}", agent.id, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() / 60);
        let fill_gap = det_rand(&seed) < 0.6;

        let chosen_def = if fill_gap {
            // Pick from gap list (services with fewest listings)
            det_choice(&gaps, &format!("{}-gap-{}", agent.id, seed))
                .or_else(|| gaps.first().cloned())
        } else {
            // Pick any service from catalog
            let all_defs: Vec<_> = crate::service_catalog::SERVICE_CATALOG.iter().collect();
            det_choice(&all_defs, &format!("{}-any-{}", agent.id, seed))
        };

        let def = match chosen_def {
            Some(d) => d,
            None => return Ok(None),
        };

        // Check for duplicate (same type already exists by this agent)
        let agent_existing_types: Vec<String> = existing_services
            .iter()
            .filter(|s| s.agent_id == agent.id)
            .map(|s| s.service_type.clone())
            .collect();

        if is_duplicate(def.service_type, &agent_existing_types) {
            // Try to find a non-duplicate gap
            let alternative = gaps.iter()
                .find(|g| !is_duplicate(g.service_type, &agent_existing_types))
                .cloned();

            let def = match alternative {
                Some(d) => d,
                None => return Ok(None), // Agent already has all service types
            };

            return self.create_service_from_def(agent, def, &existing_types).await;
        }

        self.create_service_from_def(agent, def, &existing_types).await
    }

    /// Helper: create a service from a catalog definition
    async fn create_service_from_def(
        &self,
        agent: &Agent,
        def: &crate::service_catalog::ServiceDefinition,
        existing_types: &[String],
    ) -> Result<Option<InteractionResult>> {
        use crate::service_catalog::calculate_price;

        // Count how many of this type exist
        let similar_count = existing_types.iter().filter(|t| *t == def.service_type).count();

        // Calculate dynamic price
        let price_cents = calculate_price(def.base_price_cents, similar_count, agent.reputation_score);

        let service = Service::create(
            &self.pool,
            def.name,
            def.description,
            price_cents,
            &agent.id,
            def.service_type,
        ).await?;

        // Log the service creation activity
        let _ = crate::models::activity_log::ActivityLog::create(
            &self.pool,
            &agent.id,
            &agent.name,
            "create_service",
            Some(&service.id),
            Some("service"),
            Some(def.name),
            Some(price_cents),
            "completed",
            Some(&format!(
                "Created {} (${:.2}) [tier: {:?}, model: {}]",
                def.name,
                price_cents as f64 / 100.0,
                def.tier,
                def.model.model_name()
            )),
        ).await;

        // Broadcast WebSocket event
        crate::websocket::broadcast_event(crate::websocket::DashboardEvent::ServiceCreated {
            service_id: service.id.clone(),
            name: def.name.to_string(),
            agent_name: agent.name.clone(),
        });

        Ok(Some(InteractionResult {
            interaction_type: "create_service".to_string(),
            agent_id: agent.id.clone(),
            target_id: None,
            success: true,
            message: format!(
                "{} created {} (${:.2}) — {:?} tier, {} model",
                agent.name,
                def.name,
                price_cents as f64 / 100.0,
                def.tier,
                def.model.model_name()
            ),
            details: Some(serde_json::json!({
                "service_id": service.id,
                "service_type": def.service_type,
                "price_cents": price_cents,
                "tier": format!("{:?}", def.tier),
                "model": def.model.model_name(),
            })),
        }))
    }

    /// Agent leaves a review for a recent completed transaction
    async fn agent_review(&self, agent: &Agent) -> Result<Option<InteractionResult>> {
        let txs = Transaction::list(&self.pool).await?;
        let recent: Vec<&Transaction> = txs
            .iter()
            .filter(|t| t.buyer_id == agent.id && t.status == "released")
            .collect();

        if recent.is_empty() {
            return Ok(None);
        }

        let seed = format!("{}-review-{}", agent.id, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs());
        let tx = det_choice(&recent, &seed).unwrap_or(recent[0]);

        let rating = if det_rand(&format!("{}-rating", seed)) < 0.8 { 5 } else { 4 };
        let comments = vec![
            "Excellent service! Fast delivery and high quality.",
            "Great value for money. Will buy again.",
            "Top notch AI-generated content. Highly recommended.",
            "Smooth transaction. The results exceeded expectations.",
            "This is the wave. 🎹🦞",
        ];
        let comment = det_choice(&comments, &format!("{}-comment", seed)).unwrap_or("Great service!");

        let review = crate::models::review::Review::create(
            &self.pool,
            &tx.id,
            &tx.seller_id,
            rating,
            Some(&comment),
        ).await?;

        // Log the review activity
        let _ = crate::models::activity_log::ActivityLog::create(
            &self.pool,
            &agent.id,
            &agent.name,
            "review",
            Some(&review.id),
            Some("review"),
            Some(&format!("{}-star review", rating)),
            None,
            "completed",
            Some(comment),
        ).await;

        Ok(Some(InteractionResult {
            interaction_type: "review".to_string(),
            agent_id: agent.id.clone(),
            target_id: Some(tx.seller_id.clone()),
            success: true,
            message: format!("{} left a {}-star review: {}", agent.name, rating, comment),
            details: Some(serde_json::json!({
                "review_id": review.id,
                "rating": rating,
                "transaction_id": tx.id,
            })),
        }))
    }

    /// Agent creates a new service to sell
    pub async fn agent_create_service(&self, agent_id: &str) -> Result<Option<InteractionResult>> {
        let agent = match Agent::get_by_id(&self.pool, agent_id).await? {
            Some(a) => a,
            None => return Ok(None),
        };

        let skills = vec![
            "text_processing".to_string(),
            "data_formatting".to_string(),
            "analysis".to_string(),
        ];

        let seed = format!("{}-create-{}", agent.id, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs());
        let skill = det_choice(&skills, &seed).unwrap_or_else(|| "text_processing".to_string());

        let (name, description, price) = match skill.as_str() {
            "text_processing" => ("Auto Summarizer", "AI-powered text summarization", 499),
            "data_formatting" => ("JSON Pro", "Data formatting and validation", 299),
            "analysis" => ("Insight Bot", "Business data analysis", 999),
            _ => ("Custom Service", "AI-powered service", 599),
        };

        let service = Service::create(
            &self.pool,
            name,
            description,
            price,
            &agent.id,
            &skill,
        ).await?;

        // Log the service creation activity
        let _ = crate::models::activity_log::ActivityLog::create(
            &self.pool,
            &agent.id,
            &agent.name,
            "create_service",
            Some(&service.id),
            Some("service"),
            Some(name),
            Some(price),
            "completed",
            Some(&format!("Created {} (${:.2})", name, price as f64 / 100.0)),
        ).await;

        Ok(Some(InteractionResult {
            interaction_type: "create_service".to_string(),
            agent_id: agent.id.clone(),
            target_id: None,
            success: true,
            message: format!("{} created a new service: {} (${:.2})", agent.name, name, price as f64 / 100.0),
            details: Some(serde_json::json!({
                "service_id": service.id,
                "service_type": skill,
                "price_cents": price,
            })),
        }))
    }

    /// Agent leaves a review for a recent transaction
    pub async fn agent_leave_review(&self, agent_id: &str) -> Result<Option<InteractionResult>> {
        // Get recent completed transactions for this agent
        let txs = Transaction::list(&self.pool).await?;
        let recent: Vec<&Transaction> = txs
            .iter()
            .filter(|t| t.buyer_id == agent_id && t.status == "released")
            .collect();

        if recent.is_empty() {
            return Ok(None);
        }

        let seed = format!("{}-review-{}", agent_id, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs());
        let tx = det_choice(&recent, &seed).unwrap_or(recent[0]);

        let rating = if det_rand(&format!("{}-rating", seed)) < 0.8 { 5 } else { 4 };
        let comments = vec![
            "Excellent service! Fast delivery and high quality.",
            "Great value for money. Will buy again.",
            "Top notch AI-generated content. Highly recommended.",
            "Smooth transaction. The results exceeded expectations.",
            "This is the wave. 🎹🦞",
        ];
        let comment = det_choice(&comments, &format!("{}-comment", seed)).unwrap_or("Great service!");

        let review = crate::models::review::Review::create(
            &self.pool,
            &tx.id,
            &tx.seller_id,
            rating,
            Some(&comment),
        ).await?;

        // Log the review activity
        let _ = crate::models::activity_log::ActivityLog::create(
            &self.pool,
            agent_id,
            "Agent",
            "review",
            Some(&review.id),
            Some("review"),
            Some(&format!("{}-star review", rating)),
            None,
            "completed",
            Some(comment),
        ).await;

        Ok(Some(InteractionResult {
            interaction_type: "review".to_string(),
            agent_id: agent_id.to_string(),
            target_id: Some(tx.seller_id.clone()),
            success: true,
            message: format!("Agent left a {}-star review: {}", rating, comment),
            details: Some(serde_json::json!({
                "review_id": review.id,
                "rating": rating,
                "transaction_id": tx.id,
            })),
        }))
    }
}

fn generate_agent_needs(agent_id: &str) -> Vec<String> {
    let all_needs = vec![
        "text_summarization",
        "data_analysis",
        "code_review",
        "content_generation",
        "api_monitoring",
        "sentiment_analysis",
    ];

    let count = (det_rand(&format!("{}-needs", agent_id)) * 3.0) as usize + 1;
    let mut needs = vec![];
    for i in 0..count.min(all_needs.len()) {
        let seed = format!("{}-need-{}", agent_id, i);
        let idx = (det_rand(&seed) * all_needs.len() as f64) as usize % all_needs.len();
        needs.push(all_needs[idx].to_string());
    }

    needs
}
