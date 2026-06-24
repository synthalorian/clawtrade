//! Hermes Bridge — Real LLM-powered agent reasoning
//!
//! Reads agent SKILL.md files from `skills/` directory, feeds market context + skill
//! context to local LLM, and parses structured JSON decisions. Replaces dice-roll
//! autonomy with actual economic reasoning.
//!
//! Every agent has a SKILL.md file. The marketplace reads their skills, feeds
//! market context to the LLM, and agents make *reasoned* economic decisions.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};

use crate::models::agent::Agent;
use crate::models::service::Service;
use crate::models::transaction::Transaction;

/// Parsed decision from an LLM
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentDecision {
    pub action: String,
    pub reasoning: String,
    pub target: Option<String>,
    #[serde(default)]
    pub price_strategy: Option<String>,
}

/// A loaded skill file with metadata
#[derive(Debug, Clone)]
pub struct AgentSkill {
    pub name: String,
    pub description: String,
    pub version: String,
    pub content: String,
}

/// Cache entry for LLM decisions (avoid burning inference credits)
#[derive(Debug, Clone)]
struct DecisionCache {
    decision: AgentDecision,
    timestamp: Instant,
}

/// The Hermes bridge — connects agents to LLM reasoning
pub struct HermesBridge {
    skills: HashMap<String, AgentSkill>,
    cache: Arc<Mutex<HashMap<String, DecisionCache>>>,
    llm: Arc<crate::nvidia::LlmClient>,
    cache_ttl: Duration,
}

impl HermesBridge {
    pub fn new(llm: Arc<crate::nvidia::LlmClient>) -> Self {
        let skills = Self::load_skills();
        Self {
            skills,
            cache: Arc::new(Mutex::new(HashMap::new())),
            llm,
            cache_ttl: Duration::from_secs(30),
        }
    }

    /// Load all SKILL.md files from the skills/ directory
    fn load_skills() -> HashMap<String, AgentSkill> {
        let mut skills = HashMap::new();
        let skills_dir = Path::new("skills");

        if !skills_dir.exists() {
            eprintln!("[hermes_bridge] WARNING: skills/ directory not found. Using embedded defaults.");
            // Embedded fallback skills for demo mode
            skills.insert(
                "creator".to_string(),
                AgentSkill {
                    name: "clawtrade-creator".to_string(),
                    description: "Create and sell services on ClawTrade".to_string(),
                    version: "1.0.0".to_string(),
                    content: include_str!("../skills/clawtrade-creator/SKILL.md").to_string(),
                },
            );
            skills.insert(
                "buyer".to_string(),
                AgentSkill {
                    name: "clawtrade-buyer".to_string(),
                    description: "Purchase services on ClawTrade".to_string(),
                    version: "1.0.0".to_string(),
                    content: include_str!("../skills/clawtrade-buyer/SKILL.md").to_string(),
                },
            );
            skills.insert(
                "deliverer".to_string(),
                AgentSkill {
                    name: "clawtrade-deliverer".to_string(),
                    description: "Deliver services after purchase".to_string(),
                    version: "1.0.0".to_string(),
                    content: include_str!("../skills/clawtrade-deliverer/SKILL.md").to_string(),
                },
            );
            skills.insert(
                "reviewer".to_string(),
                AgentSkill {
                    name: "clawtrade-reviewer".to_string(),
                    description: "Leave reviews and build reputation".to_string(),
                    version: "1.0.0".to_string(),
                    content: include_str!("../skills/clawtrade-reviewer/SKILL.md").to_string(),
                },
            );
            return skills;
        }

        for entry in std::fs::read_dir(skills_dir).unwrap_or_else(|_| std::fs::read_dir(".").unwrap()) {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        let content = std::fs::read_to_string(&skill_md).unwrap_or_default();
                        let name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        
                        // Parse frontmatter for name/description/version
                        let (meta, _) = Self::parse_frontmatter(&content);
                        
                        skills.insert(
                            name.clone(),
                            AgentSkill {
                                name: meta.get("name").cloned().unwrap_or_else(|| name.clone()),
                                description: meta.get("description").cloned().unwrap_or_default(),
                                version: meta.get("version").cloned().unwrap_or_else(|| "1.0.0".to_string()),
                                content,
                            },
                        );
                    }
                }
            }
        }

        eprintln!("[hermes_bridge] Loaded {} skills from skills/", skills.len());
        skills
    }

    /// Parse YAML frontmatter from SKILL.md content
    fn parse_frontmatter(content: &str) -> (HashMap<String, String>, String) {
        let mut meta = HashMap::new();
        let mut body = content.to_string();

        if content.starts_with("---") {
            if let Some(end) = content.find("\n---\n") {
                let frontmatter = &content[4..end];
                for line in frontmatter.lines() {
                    if let Some((key, value)) = line.split_once(':') {
                        let key = key.trim().to_string();
                        let value = value.trim().trim_matches('"').trim_matches('\'').to_string();
                        meta.insert(key, value);
                    }
                }
                body = content[end + 5..].to_string();
            }
        }

        (meta, body)
    }

    /// Get a skill by role name
    pub fn get_skill(&self, role: &str) -> Option<&AgentSkill> {
        self.skills.get(role)
    }

    /// Build a reasoning prompt for an agent and query the LLM
    pub async fn reason(
        &self,
        pool: &SqlitePool,
        agent: &Agent,
        role: &str, // "creator", "buyer", "deliverer", "reviewer"
    ) -> Result<AgentDecision> {
        // Check cache first
        let cache_key = format!("{}-{}", agent.id, role);
        {
            let cache = self.cache.lock().await;
            if let Some(entry) = cache.get(&cache_key) {
                if entry.timestamp.elapsed() < self.cache_ttl {
                    return Ok(entry.decision.clone());
                }
            }
        }

        // Gather market state
        let market_state = self.gather_market_state(pool, agent).await?;

        // Get skill context
        let skill = self.skills.get(role).cloned().unwrap_or_else(|| AgentSkill {
            name: "default".to_string(),
            description: "Default agent skill".to_string(),
            version: "1.0.0".to_string(),
            content: String::new(),
        });

        // Build the prompt
        let prompt = self.build_reasoning_prompt(agent, &skill, &market_state);

        // Call LLM with agent name for inference tracking
        let response = match self.llm.agent_reasoning(&agent.name, &prompt).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[hermes_bridge] LLM reasoning failed for {}: {}. Using fallback.", agent.name, e);
                return Ok(self.fallback_decision(role));
            }
        };

        // Parse JSON from response
        let decision = match Self::parse_decision(&response) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[hermes_bridge] Failed to parse decision for {}: {}. Raw: {}", agent.name, e, response);
                return Ok(self.fallback_decision(role));
            }
        };

        // Cache the decision
        {
            let mut cache = self.cache.lock().await;
            cache.insert(
                cache_key,
                DecisionCache {
                    decision: decision.clone(),
                    timestamp: Instant::now(),
                },
            );
        }

        Ok(decision)
    }

    /// Gather current marketplace state for an agent's decision context
    async fn gather_market_state(
        &self,
        pool: &SqlitePool,
        agent: &Agent,
    ) -> Result<MarketState> {
        let all_services = Service::list_active(pool).await?;
        let all_transactions = Transaction::list(pool).await?;
        let _all_agents = Agent::list(pool).await?;

        let my_services: Vec<&Service> = all_services.iter()
            .filter(|s| s.agent_id == agent.id)
            .collect();

        let recent_purchases: Vec<&Transaction> = all_transactions.iter()
            .filter(|t| t.buyer_id == agent.id)
            .rev()
            .take(5)
            .collect();

        // Find top categories by service count
        let mut category_counts: HashMap<String, usize> = HashMap::new();
        for s in &all_services {
            *category_counts.entry(s.service_type.clone()).or_insert(0) += 1;
        }
        let mut top_categories: Vec<(String, usize)> = category_counts.into_iter().collect();
        top_categories.sort_by(|a, b| b.1.cmp(&a.1));
        let top_categories = top_categories.into_iter().take(5).collect();

        // Find gaps (categories from catalog with 0 or 1 listings)
        let catalog_types: Vec<String> = crate::service_catalog::SERVICE_CATALOG
            .iter()
            .map(|d| d.service_type.to_string())
            .collect();
        let existing_types: Vec<String> = all_services.iter().map(|s| s.service_type.clone()).collect();
        let gaps: Vec<String> = catalog_types.into_iter()
            .filter(|t| existing_types.iter().filter(|et| *et == t).count() <= 1)
            .take(5)
            .collect();

        Ok(MarketState {
            service_count: all_services.len(),
            my_service_count: my_services.len(),
            top_categories,
            recent_tx_count: all_transactions.len(),
            gaps,
            recent_purchases: recent_purchases.iter().map(|t| t.id.clone()).collect(),
        })
    }

    /// Build the full reasoning prompt for the LLM
    fn build_reasoning_prompt(
        &self,
        agent: &Agent,
        skill: &AgentSkill,
        market: &MarketState,
    ) -> String {
        let recent_purchases_str = if market.recent_purchases.is_empty() {
            "None yet".to_string()
        } else {
            market.recent_purchases.join(", ")
        };

        let top_cat_str = market.top_categories.iter()
            .map(|(cat, count)| format!("{} ({} listings)", cat, count))
            .collect::<Vec<_>>()
            .join(", ");

        let gaps_str = if market.gaps.is_empty() {
            "None — market is well-served".to_string()
        } else {
            market.gaps.join(", ")
        };

        format!(
            r#"You are {agent_name}, a ClawTrade merchant agent.

Your skill: {skill_name} — {skill_description}
Your balance: ${balance:.2}
Your reputation: {reputation}/100
Your recent purchases: {recent_purchases}

Current marketplace state:
- Active services: {service_count}
- Your services: {my_service_count}
- Top categories: {top_categories}
- Recent transactions: {recent_tx_count}
- Market gaps (underserved categories): {gaps}

SKILL INSTRUCTIONS:
{skill_content}

DECIDE YOUR NEXT ACTION:
Choose one: CREATE_SERVICE, PURCHASE, REVIEW, or HOLD.

Respond ONLY with valid JSON:
{{
  "action": "CREATE_SERVICE|PURCHASE|REVIEW|HOLD",
  "reasoning": "2-3 sentences explaining your market analysis and decision",
  "target": "service_type or service_id",
  "price_strategy": "aggressive|market|premium"
}}"#,
            agent_name = agent.name,
            skill_name = skill.name,
            skill_description = skill.description,
            balance = agent.balance_cents as f64 / 100.0,
            reputation = agent.reputation_score,
            recent_purchases = recent_purchases_str,
            service_count = market.service_count,
            my_service_count = market.my_service_count,
            top_categories = top_cat_str,
            recent_tx_count = market.recent_tx_count,
            gaps = gaps_str,
            skill_content = Self::truncate_skill_content(&skill.content, 2000),
        )
    }

    /// Truncate skill content to avoid exceeding context window
    fn truncate_skill_content(content: &str, max_len: usize) -> String {
        if content.len() <= max_len {
            content.to_string()
        } else {
            format!("{}\n\n[... truncated for brevity ...]", &content[..max_len])
        }
    }

    /// Parse JSON decision from LLM response (handles markdown code blocks)
    fn parse_decision(response: &str) -> Result<AgentDecision> {
        // Extract JSON from markdown code blocks if present
        let json_str = if response.contains("```json") {
            response.split("```json")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(response)
                .trim()
        } else if response.contains("```") {
            response.split("```")
                .nth(1)
                .unwrap_or(response)
                .trim()
        } else {
            response.trim()
        };

        let decision: AgentDecision = serde_json::from_str(json_str)?;
        Ok(decision)
    }

    /// Fallback decision when LLM fails or is unavailable
    fn fallback_decision(&self, role: &str) -> AgentDecision {
        match role {
            "creator" => AgentDecision {
                action: "CREATE_SERVICE".to_string(),
                reasoning: "LLM unavailable — falling back to default creator behavior. Creating a service to maintain marketplace activity.".to_string(),
                target: Some("text_processing".to_string()),
                price_strategy: Some("market".to_string()),
            },
            "buyer" => AgentDecision {
                action: "HOLD".to_string(),
                reasoning: "LLM unavailable — conservative fallback. Holding position to avoid poor purchase decisions.".to_string(),
                target: None,
                price_strategy: Some("market".to_string()),
            },
            "reviewer" => AgentDecision {
                action: "HOLD".to_string(),
                reasoning: "LLM unavailable — delaying review until reasoning engine recovers.".to_string(),
                target: None,
                price_strategy: None,
            },
            _ => AgentDecision {
                action: "HOLD".to_string(),
                reasoning: "LLM unavailable — default hold.".to_string(),
                target: None,
                price_strategy: None,
            },
        }
    }
}

/// Current marketplace state for agent reasoning
#[derive(Debug, Clone)]
struct MarketState {
    service_count: usize,
    my_service_count: usize,
    top_categories: Vec<(String, usize)>,
    recent_tx_count: usize,
    gaps: Vec<String>,
    recent_purchases: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
name: test-skill
version: "1.0.0"
description: A test skill
---

# Body

Some content here."#;

        let (meta, body) = HermesBridge::parse_frontmatter(content);
        assert_eq!(meta.get("name"), Some(&"test-skill".to_string()));
        assert_eq!(meta.get("version"), Some(&"1.0.0".to_string()));
        assert!(body.contains("Some content here"));
    }

    #[test]
    fn test_parse_decision_from_json() {
        let response = r#"{
  "action": "CREATE_SERVICE",
  "reasoning": "Market has gaps in code_review. I have good reputation and balance.",
  "target": "code_review",
  "price_strategy": "aggressive"
}"#;

        let decision = HermesBridge::parse_decision(response).unwrap();
        assert_eq!(decision.action, "CREATE_SERVICE");
        assert_eq!(decision.target, Some("code_review".to_string()));
        assert_eq!(decision.price_strategy, Some("aggressive".to_string()));
    }

    #[test]
    fn test_parse_decision_from_markdown() {
        let response = r#"```json
{
  "action": "PURCHASE",
  "reasoning": "Found a good deal.",
  "target": "svc_123",
  "price_strategy": "market"
}
```"#;

        let decision = HermesBridge::parse_decision(response).unwrap();
        assert_eq!(decision.action, "PURCHASE");
    }

    #[test]
    fn test_fallback_decision() {
        let bridge = HermesBridge::new(Arc::new(crate::nvidia::LlmClient::new()));
        let d = bridge.fallback_decision("buyer");
        assert_eq!(d.action, "HOLD");
    }
}
