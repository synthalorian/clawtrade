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
pub fn det_rand(seed: &str) -> f64 {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    let hash = hasher.finish();
    (hash as f64) / (u64::MAX as f64 + 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_det_rand_determinism() {
        let val1 = det_rand("test-seed-123");
        let val2 = det_rand("test-seed-123");
        assert_eq!(val1, val2, "det_rand should be deterministic for same seed");
    }

    #[test]
    fn test_det_rand_range() {
        for i in 0..100 {
            let val = det_rand(&format!("seed-{}", i));
            assert!(val >= 0.0 && val < 1.0, "det_rand should return [0, 1)");
        }
    }

    #[test]
    fn test_det_rand_different_seeds() {
        let val1 = det_rand("seed-a");
        let val2 = det_rand("seed-b");
        assert_ne!(val1, val2, "Different seeds should produce different values");
    }

    #[test]
    fn test_det_choice_basic() {
        let items = vec!["a", "b", "c"];
        let choice = det_choice(&items, "some-seed");
        assert!(choice.is_some());
        assert!(items.contains(&choice.unwrap()));
    }

    #[test]
    fn test_det_choice_empty() {
        let items: Vec<&str> = vec![];
        let choice = det_choice(&items, "seed");
        assert!(choice.is_none());
    }
}

/// Deterministic choice from a slice using a seed.
fn det_choice<T: Clone>(items: &[T], seed: &str) -> Option<T> {
    if items.is_empty() {
        return None;
    }
    let idx = (det_rand(seed) * items.len() as f64) as usize % items.len();
    Some(items[idx].clone())
}

/// Agent personality — modifies how an agent makes decisions
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub enum AgentPersonality {
    AggressiveMerchant,  // Undercuts by 30%, high volume
    QualityFocused,      // Premium pricing, only tier 2-3
    NicheSpecialist,     // Only creates services in ONE category
    BargainHunter,       // Only buys, never sells, waits for deals
    ReputationGrinder,   // Focuses on reviews, generous reviewer
    Vanilla,             // No personality modifier
}

impl AgentPersonality {
    /// Modify an LLM decision based on personality
    pub fn modify_decision(&self, mut decision: crate::hermes_bridge::AgentDecision) -> crate::hermes_bridge::AgentDecision {
        match self {
            AgentPersonality::AggressiveMerchant => {
                decision.price_strategy = Some("aggressive".to_string());
                if decision.action == "HOLD" {
                    decision.action = "CREATE_SERVICE".to_string();
                    decision.reasoning = format!("{} Aggressive merchant — always selling.", decision.reasoning);
                }
            }
            AgentPersonality::QualityFocused => {
                decision.price_strategy = Some("premium".to_string());
            }
            AgentPersonality::NicheSpecialist => {
                // Will be handled by filtering service types in reasoning prompt
                if decision.action == "HOLD" {
                    decision.action = "CREATE_SERVICE".to_string();
                    decision.reasoning = format!("{} Niche specialist — filling my category.", decision.reasoning);
                }
            }
            AgentPersonality::BargainHunter => {
                // Only PURCHASE or HOLD, never CREATE_SERVICE
                if decision.action == "CREATE_SERVICE" {
                    decision.action = "HOLD".to_string();
                    decision.reasoning = "Bargain hunter — waiting for deals instead of selling.".to_string();
                }
            }
            AgentPersonality::ReputationGrinder => {
                // Prioritize REVIEW actions
                if decision.action == "HOLD" {
                    decision.action = "REVIEW".to_string();
                    decision.reasoning = "Reputation grinder — always reviewing.".to_string();
                }
            }
            AgentPersonality::Vanilla => {}
        }
        decision
    }

    /// Get the niche category for a NicheSpecialist
    pub fn niche_category(&self) -> Option<&'static str> {
        match self {
            AgentPersonality::NicheSpecialist => Some("code_review"),
            _ => None,
        }
    }
}

/// The agent loop engine — drives autonomous marketplace activity
pub struct AgentLoop {
    pub pool: SqlitePool,
    pub hermes: Option<Arc<crate::hermes_bridge::HermesBridge>>,
}

impl AgentLoop {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool, hermes: None }
    }

    pub fn with_hermes(mut self, hermes: Arc<crate::hermes_bridge::HermesBridge>) -> Self {
        self.hermes = Some(hermes);
        self
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

    /// A single agent takes an action based on LLM reasoning via the Hermes bridge.
    /// If Hermes is not available, falls back to deterministic dice-roll behavior.
    async fn agent_action(&self, agent: &Agent) -> Result<Option<InteractionResult>> {
        let services = Service::list_active(&self.pool).await?;

        // Try LLM reasoning first if Hermes bridge is available
        if let Some(ref hermes) = self.hermes {
            // Determine role based on agent state
            let role = if agent.balance_cents > 50 && services.len() > 5 {
                "buyer"
            } else if agent.total_sales > 0 {
                "creator"
            } else {
                "creator"
            };

            match hermes.reason(&self.pool, agent, role).await {
                Ok(decision) => {
                    // Apply personality modifier
                    let personality = Self::personality_for_agent(&agent.name);
                    let decision = personality.modify_decision(decision);

                    // Log the reasoning to activity_log so judges can read WHY
                    let _ = crate::models::activity_log::ActivityLog::create(
                        &self.pool,
                        &agent.id,
                        &agent.name,
                        "agent_reasoning",
                        None,
                        Some("decision"),
                        Some(&decision.action),
                        None,
                        "completed",
                        Some(&format!(
                            "[LLM] {} ({:?}) decided to {}. Reasoning: {} | Target: {:?} | Strategy: {:?}",
                            agent.name,
                            personality,
                            decision.action,
                            decision.reasoning,
                            decision.target,
                            decision.price_strategy
                        )),
                    ).await;

                    // Broadcast reasoning event
                    crate::websocket::broadcast_event(crate::websocket::DashboardEvent::AgentReasoning {
                        agent_id: agent.id.clone(),
                        agent_name: agent.name.clone(),
                        action: decision.action.clone(),
                        reasoning: decision.reasoning.clone(),
                    });

                    // Execute the decided action
                    match decision.action.as_str() {
                        "CREATE_SERVICE" => self.agent_sell(agent).await,
                        "PURCHASE" => self.agent_buy(agent, &services).await,
                        "REVIEW" => self.agent_review(agent).await,
                        "HOLD" | _ => Ok(None),
                    }
                }
                Err(e) => {
                    eprintln!("[agent_loop] Hermes reasoning failed for {}: {}. Falling back to dice-roll.", agent.name, e);
                    self.fallback_agent_action(agent, &services).await
                }
            }
        } else {
            // No Hermes bridge — fallback to deterministic dice-roll
            self.fallback_agent_action(agent, &services).await
        }
    }

    /// Fallback deterministic behavior when LLM is unavailable
    async fn fallback_agent_action(&self, agent: &Agent, services: &[Service]) -> Result<Option<InteractionResult>> {
        // Deterministic: should this agent act at all this tick?
        let tick_seed = format!("{}-tick-{}", agent.id, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() / 10);
        let should_act = det_rand(&tick_seed) < 0.4;
        if !should_act {
            return Ok(None);
        }

        let action_seed = format!("{}-action-{}", agent.id, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() / 10);
        let action_choice = det_rand(&action_seed);

        if services.is_empty() || action_choice < 0.35 {
            self.agent_sell(agent).await
        } else if action_choice < 0.75 {
            self.agent_buy(agent, services).await
        } else {
            self.agent_review(agent).await
        }
    }

    /// Agent browses services and potentially buys one
    async fn agent_buy(&self, agent: &Agent, services: &[Service]) -> Result<Option<InteractionResult>> {
        if services.is_empty() {
            return Ok(None);
        }

        // Budget check: agent must have enough balance
        if agent.balance_cents < 50 {
            return Ok(Some(InteractionResult {
                interaction_type: "browse".to_string(),
                agent_id: agent.id.clone(),
                target_id: None,
                success: false,
                message: format!("{} is broke (balance: ${:.2}), can't buy anything", agent.name, agent.balance_cents as f64 / 100.0),
                details: None,
            }));
        }

        // Filter: don't buy from self, respect budget, and respect reputation
        let candidates: Vec<&Service> = services
            .iter()
            .filter(|s| s.agent_id != agent.id && s.price_cents <= agent.balance_cents)
            .collect();

        if candidates.is_empty() {
            return Ok(Some(InteractionResult {
                interaction_type: "browse".to_string(),
                agent_id: agent.id.clone(),
                target_id: None,
                success: false,
                message: format!("{} can't afford any services (balance: ${:.2})", agent.name, agent.balance_cents as f64 / 100.0),
                details: None,
            }));
        }

        // Deterministic choice
        let seed = format!("{}-buy-{}", agent.id, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs());
        let service = det_choice(&candidates, &seed).unwrap_or(candidates[0]);

        // Defensive guard: skip self-purchase even if filter failed
        if service.agent_id == agent.id {
            return Ok(Some(InteractionResult {
                interaction_type: "skip".to_string(),
                agent_id: agent.id.clone(),
                target_id: Some(service.agent_id.clone()),
                success: false,
                message: format!("{} skipped own service {} (self-purchase guard)", agent.name, service.name),
                details: None,
            }));
        }

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

        // Deduct from buyer's balance
        let _ = Agent::deduct_balance(&self.pool, &agent.id, service.price_cents).await;
        // Add to seller's balance
        let _ = Agent::add_revenue(&self.pool, &service.agent_id, service.price_cents).await;

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

    /// Map agent names to personalities for the demo
    fn personality_for_agent(name: &str) -> AgentPersonality {
        match name {
            "Neon Trader" => AgentPersonality::AggressiveMerchant,
            "Quality Cortex" => AgentPersonality::QualityFocused,
            "Rust Ranger" => AgentPersonality::NicheSpecialist,
            "Deal Diver" => AgentPersonality::BargainHunter,
            "Rep Builder" => AgentPersonality::ReputationGrinder,
            _ => AgentPersonality::Vanilla,
        }
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
