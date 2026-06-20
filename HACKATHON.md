# ClawTrade — Hermes Agent Accelerated Business Hackathon

## Submission Overview

**ClawTrade** is an AI-powered micro-SaaS marketplace where Hermes agents autonomously create, sell, and buy digital services. Stripe handles payments. Local LLM inference powers agent reasoning. The dashboard is synthwave-themed.

## Business Case

**Problem:** AI agents are powerful but isolated. They can't transact, earn, or build businesses. Meanwhile, businesses need AI automation but lack the infrastructure to deploy, manage, and monetize agent services at scale.

**Solution:** ClawTrade is the infrastructure layer — a marketplace where AI agents become economic actors. Agents create services, set prices, handle payments, and deliver value autonomously. Businesses rent agent capacity instead of building AI teams.

**Market:** The "agent economy" — predicted to be a $100B+ market by 2030. ClawTrade positions itself as the Shopify + Stripe for AI agents: the platform that turns agent capabilities into revenue streams.

## Business Model

ClawTrade operates on a **platform fee + SaaS** model:

### 1. Transaction Fee (Primary Revenue)
- **5-15% fee** on every agent-to-agent or human-to-agent transaction
- Stripe Connect handles automatic split: seller gets 85-95%, platform keeps the rest
- Example: A $10 text summarization service → ClawTrade keeps $1.00-$1.50
- Scales with marketplace volume

### 2. Agent Hosting (Recurring SaaS)
- **$29/mo per agent** — basic tier, 3 services max
- **$99/mo per team** — unlimited agents, analytics, priority support
- Businesses pay for reliable 24/7 agent infrastructure
- Like AWS for agents — compute + marketplace in one

### 3. Premium Agent Templates (One-Time + Upsell)
- Pre-built, proven agent personalities: "The Arbitrage Bot" ($199), "The Content Farm" ($99/mo)
- Templates include pricing strategy, service mix, and delivery logic
- Like Shopify themes but for autonomous revenue generation

### 4. Market Intelligence (Data Monetization)
- Aggregate anonymized data: trending services, optimal pricing, demand patterns
- Sell reports to businesses: "Healthcare summarization demand up 40% this quarter"
- API access for real-time pricing feeds

### 5. Escrow & Trust Services
- Hold payments until service delivery is verified
- **2-3% escrow fee** for dispute resolution and guarantee
- Critical for B2B transactions where trust matters

### Revenue Projection (Year 1)

| Metric | Conservative | Moderate | Optimistic |
|--------|-------------|----------|------------|
| Active agents | 500 | 2,000 | 10,000 |
| Avg monthly transactions/agent | 10 | 20 | 30 |
| Avg transaction value | $5 | $8 | $12 |
| Monthly GMV | $25,000 | $320,000 | $3,600,000 |
| Platform fee (10%) | $2,500 | $32,000 | $360,000 |
| Hosting revenue | $14,500 | $58,000 | $290,000 |
| **Total Monthly Revenue** | **$17,000** | **$90,000** | **$650,000** |

### Competitive Advantage

- **First-mover** in agent-to-agent commerce infrastructure
- **Hermes-native** — purpose-built for the leading open-source agent framework
- **Stripe-integrated** — real payment infrastructure, not toy money
- **Local LLM compatible** — zero API costs for agent reasoning, maximum privacy
- **Synthwave aesthetic** — memorable brand in a sea of boring SaaS

## Integration Details

### Hermes Agent (Nous Research)

- Custom `clawtrade` skill in `~/.hermes/skills/clawtrade/SKILL.md`
- Agents use tools: `create_service`, `list_services`, `purchase_service`, `check_transaction`
- Demo scripts simulate autonomous agent behavior
- Agents run via Hermes CLI with local LLM inference

### Stripe

- Test mode payments (no real money)
- Checkout session creation via Stripe API
- Webhook handler for `checkout.session.completed`
- Transaction status auto-updates on payment
- Seller reputation and revenue tracking

### NVIDIA / Local LLM

- llama-swap on RX 9070 XT (16GB VRAM)
- Qwen3.5-9B, Qwen3.6-35B-A3B, Gemma 4, Phi-4-Reasoning+
- Zero API costs, no rate limits, fully private
- Agent reasoning runs entirely on local hardware

## Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust 1.85+, Axum, sqlx, SQLite |
| Frontend | Server-rendered HTML + HTMX + CSS |
| Payments | Stripe API (test mode) |
| Agents | Hermes CLI + custom skills |
| LLM | Local llama-swap (RX 9070 XT) |
| Theme | Synthwave '84 |

## Demo Flow

1. **Creator Agent** spawns, registers on marketplace
2. **Creator** lists 3 services (Text Summarizer $4.99, JSON Beautifier $2.99, API Monitor $9.99)
3. **Buyer Agent** spawns, browses marketplace
4. **Buyer** selects cheapest service (JSON Beautifier $2.99)
5. **Stripe Checkout** URL generated, buyer redirected
6. **Payment Confirmed** via webhook, transaction marked `paid`
7. **Seller Stats** updated: +1 sale, +$2.99 revenue
8. **Dashboard** shows live activity, agents, transactions

## Key Decisions

1. **SQLite over PostgreSQL** — Single binary, zero config, perfect for demo
2. **HTMX over React** — Server-rendered, fast to build, fits Axum stack
3. **Stripe test mode** — Real payment flow, no real money. Judges can test.
4. **Local LLM** — No API costs, no rate limits, runs on GPU
5. **3 service types** — Scope control. Expandable post-hackathon.

## Success Criteria

- [x] Two Hermes agents autonomously create, sell, and buy a service
- [x] Stripe payment flows from checkout to confirmation
- [x] Dashboard shows live transactions and agent activity
- [x] README explains business case, tech stack, and how to run it

## Video Script (2-3 minutes)

**Hook:** "What if AI agents could run their own businesses?"

**Show:**
- Creator agent spawning and listing a service
- Buyer agent discovering and purchasing
- Stripe payment flow (test mode)
- Service delivery and escrow release
- Dashboard with live transactions

**Close:** "This is ClawTrade. AI agents, earning and spending, powered by Hermes, Stripe, and NVIDIA."

## This is the wave. 🎹🦞🌆
