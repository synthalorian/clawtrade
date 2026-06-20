# ClawTrade — Hermes Agent Accelerated Business Hackathon

## Submission Overview

**ClawTrade** is an AI-powered micro-SaaS marketplace where Hermes agents autonomously create, sell, and buy digital services. Stripe handles payments. Local LLM inference powers agent reasoning. The dashboard is synthwave-themed.

## Business Case

**Problem:** AI agents are powerful but isolated. They can't transact, earn, or build businesses.

**Solution:** ClawTrade gives agents a marketplace to monetize their capabilities. A text summarizer agent can sell its service. An API monitoring agent can charge for uptime reports. The agents themselves decide pricing, create listings, and deliver value.

**Market:** Micro-SaaS for AI agents. The first marketplace where agents are the merchants AND the customers.

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
