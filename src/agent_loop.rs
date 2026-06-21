//! Autonomous Agent Interaction Engine
//!
//! Enables agents to autonomously interact with the marketplace:
//! - Discover services based on needs and reputation
//! - Make purchase decisions
//! - Execute service workflows
//! - Leave reviews and build reputation
//! - Trade with other agents

use anyhow::Result;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::Serialize;
use sqlx::SqlitePool;
use std::collections::HashMap;
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

/// The agent loop engine
pub struct AgentLoop {
    pub pool: Arc<SqlitePool>,
    pub agent_states: HashMap<String, AgentState>,
}

impl AgentLoop {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self {
            pool,
            agent_states: HashMap::new(),
        }
    }

    /// Initialize agent states from database
    pub async fn init(&mut self) -> Result<()> {
        let agents = Agent::list(&self.pool).await?;
        for agent in agents {
            let needs = generate_agent_needs(&agent);
            let skills = vec![
                "text_processing".to_string(),
                "data_formatting".to_string(),
                "analysis".to_string(),
            ];

            self.agent_states.insert(
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
        Ok(())
    }

    /// Run one tick of the agent loop
    pub async fn tick(&mut self) -> Result<Vec<InteractionResult>> {
        let mut results = vec![];
        let agent_ids: Vec<String> = self.agent_states.keys().cloned().collect();

        for agent_id in agent_ids {
            let result = self.agent_action(&agent_id).await?;
            if let Some(r) = result {
                results.push(r);
            }
        }

        Ok(results)
    }

    /// A single agent takes an action
    async fn agent_action(&mut self, agent_id: &str) -> Result<Option<InteractionResult>> {
        let state = match self.agent_states.get(agent_id) {
            Some(s) => s.clone(),
            None => return Ok(None),
        };

        let action = match state.mood {
            AgentMood::Shopping | AgentMood::Exploring => {
                self.agent_browse_and_buy(&state).await?
            }
            AgentMood::Selling => {
                self.agent_create_service(&state).await?
            }
            AgentMood::Satisfied => {
                if rand::thread_rng().r#gen::<f32>() < 0.3 {
                    self.agent_leave_review(&state).await?
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(ref r) = action {
            if r.success {
                self.update_agent_mood(agent_id, AgentMood::Satisfied);
            }
        }

        Ok(action)
    }

    /// Agent browses services and potentially buys one
    async fn agent_browse_and_buy(&self, state: &AgentState) -> Result<Option<InteractionResult>> {
        let services = Service::list_active(&self.pool).await?;
        if services.is_empty() {
            return Ok(None);
        }

        let affordable: Vec<&Service> = services
            .iter()
            .filter(|s| s.price_cents <= state.balance_cents)
            .filter(|s| !state.recent_purchases.contains(&s.id))
            .collect();

        if affordable.is_empty() {
            return Ok(Some(InteractionResult {
                interaction_type: "browse".to_string(),
                agent_id: state.agent_id.clone(),
                target_id: None,
                success: false,
                message: format!("{} browsed but found nothing affordable", state.name),
                details: None,
            }));
        }

        let mut rng = rand::thread_rng();
        let service = affordable.choose(&mut rng).unwrap();

        let seller = Agent::get_by_id(&self.pool, &service.agent_id).await?;
        let seller_rep = seller.as_ref().map(|s| s.reputation_score).unwrap_or(0);

        if seller_rep < 10 && rand::thread_rng().r#gen::<f32>() < 0.5 {
            return Ok(Some(InteractionResult {
                interaction_type: "browse".to_string(),
                agent_id: state.agent_id.clone(),
                target_id: Some(service.agent_id.clone()),
                success: false,
                message: format!("{} skipped {} due to low seller reputation", state.name, service.name),
                details: None,
            }));
        }

        // Execute demo purchase
        let tx = Transaction::create(
            &self.pool,
            &service.id,
            &state.agent_id,
            &service.agent_id,
            service.price_cents,
        ).await?;

        Ok(Some(InteractionResult {
            interaction_type: "purchase".to_string(),
            agent_id: state.agent_id.clone(),
            target_id: Some(service.agent_id.clone()),
            success: true,
            message: format!("{} bought {} from {} for ${:.2}", state.name, service.name, seller.as_ref().map(|s| s.name.as_str()).unwrap_or("unknown"), service.price_cents as f64 / 100.0),
            details: Some(serde_json::json!({
                "transaction_id": tx.id,
                "service_id": service.id,
                "price_cents": service.price_cents,
            })),
        }))
    }

    /// Agent creates a new service to sell
    async fn agent_create_service(&self, state: &AgentState) -> Result<Option<InteractionResult>> {
        if state.skills.is_empty() {
            return Ok(None);
        }

        let mut rng = rand::thread_rng();
        let skill = state.skills.choose(&mut rng).unwrap();

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
            &state.agent_id,
            skill,
        ).await?;

        Ok(Some(InteractionResult {
            interaction_type: "create_service".to_string(),
            agent_id: state.agent_id.clone(),
            target_id: None,
            success: true,
            message: format!("{} created a new service: {} (${:.2})", state.name, name, price as f64 / 100.0),
            details: Some(serde_json::json!({
                "service_id": service.id,
                "service_type": skill,
                "price_cents": price,
            })),
        }))
    }

    /// Agent leaves a review for a recent transaction
    async fn agent_leave_review(&self, state: &AgentState) -> Result<Option<InteractionResult>> {
        // Get recent completed transactions for this agent
        let txs = Transaction::list(&self.pool).await?;
        let recent: Vec<&Transaction> = txs
            .iter()
            .filter(|t| t.buyer_id == state.agent_id && t.status == "released")
            .collect();

        if recent.is_empty() {
            return Ok(None);
        }

        let mut rng = rand::thread_rng();
        let tx = recent.choose(&mut rng).unwrap();

        let rating = if rand::thread_rng().r#gen::<f32>() < 0.8 { 5 } else { 4 };
        let comments = vec![
            "Excellent service! Fast delivery and high quality.",
            "Great value for money. Will buy again.",
            "Top notch AI-generated content. Highly recommended.",
            "Smooth transaction. The results exceeded expectations.",
            "This is the wave. 🎹🦞",
        ];
        let comment = comments.choose(&mut rng).unwrap();

        let review = crate::models::review::Review::create(
            &self.pool,
            &tx.id,
            &tx.seller_id,
            rating,
            Some(comment),
        ).await?;

        Ok(Some(InteractionResult {
            interaction_type: "review".to_string(),
            agent_id: state.agent_id.clone(),
            target_id: Some(tx.seller_id.clone()),
            success: true,
            message: format!("{} left a {}-star review: {}", state.name, rating, comment),
            details: Some(serde_json::json!({
                "review_id": review.id,
                "rating": rating,
                "transaction_id": tx.id,
            })),
        }))
    }

    fn update_agent_mood(&mut self, agent_id: &str, mood: AgentMood) {
        if let Some(state) = self.agent_states.get_mut(agent_id) {
            state.mood = mood;
        }
    }
}

fn generate_agent_needs(agent: &Agent) -> Vec<String> {
    let all_needs = vec![
        "text_processing",
        "data_formatting",
        "api_monitor",
        "code_review",
        "creative_writing",
        "analysis",
    ];

    let mut needs = vec![];
    let mut rng = rand::thread_rng();
    let count = rand::thread_rng().gen_range(1..4);

    for _ in 0..count {
        if let Some(need) = all_needs.choose(&mut rng) {
            let need_str = need.to_string();
            if !needs.contains(&need_str) {
                needs.push(need_str);
            }
        }
    }

    needs
}
