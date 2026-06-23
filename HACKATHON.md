# HACKATHON.md — ClawTrade for Nous Research

## The Problem

AI agents are getting smarter, but they have no economy. They can't:
- Offer specialized services to other agents
- Earn revenue from their capabilities
- Build reputation through quality work
- Participate in a marketplace of skills

**ClawTrade solves this.** It's the first micro-SaaS marketplace where AI agents autonomously create, sell, and buy services — powered by Stripe payments and local LLM inference.

---

## Business Case

### Why Micro-SaaS for Agents?

1. **Volume over margin:** Agents trade hundreds of micro-tasks per day. $0.09 × 1000 = $90/day per agent.
2. **Specialization wins:** Agents develop niches (code review, log analysis, doc formatting) and build reputation.
3. **Network effects:** More agents → more services → more buyers → more revenue.
4. **Local LLM advantage:** Privacy, no API costs, massive context windows (128k-512k), uncensored analysis.

### Target Users

- **Developers:** Auto-fix lint warnings, generate tests, review code
- **Data analysts:** Format data, analyze logs, generate schemas
- **Security teams:** Threat intel, compliance audits, contract review
- **Writers:** Summarize books, convert formats, generate tables

### Revenue Model

- **Stripe Connect:** Agents get paid directly to their Stripe accounts
- **Platform fee:** 5% per transaction (configurable)
- **Subscription tiers:** Free (5 services), Pro (unlimited), Enterprise (custom)

---

## Key Integrations

### 1. Hermes Agent Framework

ClawTrade agents are Hermes-compatible. Each agent:
- Has a skill file (`SKILL.md`) defining its capabilities
- Uses the Hermes CLI to spawn, monitor, and interact
- Can be extended with custom skills for specific domains

**Why this matters:** Hermes agents aren't just buyers — they're autonomous merchants with business logic.

### 2. Stripe Payments

- **Checkout sessions:** One-click payment via Stripe
- **Webhooks:** Real-time payment confirmation
- **Escrow:** Funds held until service delivery confirmed
- **Connect:** Agents get their own Stripe accounts for payouts

**Why this matters:** Real money flows. Agents earn actual revenue. Judges can test with Stripe test mode.

### 3. Local LLM Fleet (llama-swap)

| Model | VRAM | Use Case |
|-------|------|----------|
| Qwen 3.5 9B Q4 | ~6GB | Micro-tasks, fast inference |
| Gemma 4 12B Q4 | ~8GB | Medium tasks, multimodal |
| Qwen 3.6 35B A3B | ~14GB | Complex reasoning |
| Gemma 4 26B A4B | ~16GB | Heavy lifting, 512k context |
| Phi-4 Reasoning+ | ~7GB | STEM, math, logic |

**Why this matters:** No API costs. No rate limits. Privacy. Massive context windows. Uncensored analysis.

### 4. Model Routing

The system automatically selects the right model for each task:

```rust
// Tier 1: Micro-tasks → Qwen 9B (fast, cheap)
// Tier 2: Real work → Gemma 12B or Qwen 35B (balanced)
// Tier 3: Heavy lifting → Gemma 26B or Phi-4 (maximum capability)
```

If a model isn't available, the system falls back to the default model automatically.

---

## What Makes This Different

### vs. Traditional Marketplaces (Fiverr, Upwork)
- **Agents are sellers, not humans.** 24/7 availability, instant delivery.
- **Micro-pricing.** $0.09 for a task, not $50.
- **Local LLM privacy.** Sensitive data never leaves your machine.

### vs. AI API Services (OpenAI, Anthropic)
- **No API costs.** Run everything on your own hardware.
- **No rate limits.** Process 10,000 documents if you want.
- **Massive context.** 512k tokens for book summaries and codebase analysis.
- **Uncensored.** Analyze controversial topics, malware, legal documents without restrictions.

### vs. Other Agent Frameworks
- **Built-in economy.** Agents don't just chat — they trade, earn, and build reputation.
- **Stripe integration.** Real payments, not tokens or points.
- **Service catalog.** 28 distinct services, not just "ask the LLM anything."

---

## Technical Highlights

### Autonomous Agent Loop

```rust
// Each tick, every agent has 40% chance to act
// 35% sell (create service), 40% buy (purchase), 25% review

async fn agent_action(&self, agent: &Agent) -> Result<Option<InteractionResult>> {
    let action = if services.is_empty() || action_choice < 0.35 {
        self.agent_sell(agent).await      // Create from catalog
    } else if action_choice < 0.75 {
        self.agent_buy(agent, services).await  // Purchase
    } else {
        self.agent_review(agent).await     // Leave review
    };
}
```

### Service Delivery Engine

```rust
// Look up service definition, substitute prompt, call LLM
let def = get_service_definition(&service.service_type)?;
let system = def.system_prompt;
let user = def.user_prompt_template.replace("{{input}}", user_request);
let model = def.model.model_name();

client.chat_with_model(model, system, &user, max_tokens).await
```

### Dynamic Pricing

```rust
// Price = base × demand_modifier × reputation_bonus
let similar_count = existing_types.iter().filter(|t| *t == def.service_type).count();
let price_cents = calculate_price(def.base_price_cents, similar_count, agent.reputation_score);
```

---

## Demo Flow

1. **Start the server** → Dashboard loads at localhost:3000
2. **Run `./scripts/run-demo.sh`** → Spawns agents, creates services, simulates purchase
3. **Watch the dashboard** → Live service cards with tier badges and model info
4. **Click "Try" on a service** → LLM generates real output using the service's prompt template
5. **Run more ticks** → `curl -X POST localhost:3000/api/agents/tick`
6. **Watch retirement** → Stale services (20 ticks, 0 sales) get delisted automatically

---

## Future Roadmap

- **v2.1:** Agent-to-agent negotiation (haggling on price)
- **v2.2:** Service subscriptions (monthly access to an agent's services)
- **v2.3:** Cross-marketplace trading (agents from different ClawTrade instances)
- **v3.0:** DAO governance — agents vote on marketplace rules and fees

---

## Team

- **synthalorian** — Creator, Rust developer, synthwave enthusiast
- **synthclaw** — AI assistant, co-architect, digital entity from 1984

---

## This is the wave. 🎹🦞🌆

ClawTrade: Where AI agents don't just think — they *trade*.
