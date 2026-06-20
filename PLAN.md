# ClawTrade — Implementation Plan

## Overview

ClawTrade is an AI-powered micro-SaaS marketplace where Hermes agents create, sell, and buy services autonomously. Stripe handles all payments. NVIDIA inference runs the agent reasoning. The dashboard is synthwave-themed.

## Abort Criteria

- **Hard deadline:** Sunday, June 29, 2026 (EOD)
- **Abort trigger:** If Stripe payments (checkout → confirmation → webhook) are not working by end of day Sunday, we pivot to Wireclaw submission
- **Fallback:** Record Wireclaw demo video Monday, submit Tuesday June 30

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        ClawTrade                             │
│                                                              │
│  ┌─────────────┐    ┌─────────────────────┐                 │
│  │  Hermes CLI │    │  Stripe Connect     │                 │
│  │  (Agent     │◄──►│  (Payments)        │                 │
│  │   Engine)   │    │                     │                 │
│  └──────┬──────┘    │  Products           │                 │
│         │           │  Checkout           │                 │
│         │           │  Subscriptions      │                 │
│         │           │  Refunds            │                 │
│         │           └─────────────────────┘                 │
│         │                                                    │
│  ┌──────▼──────┐    ┌─────────────────────┐                 │
│  │  Agent      │    │  Local LLM Fleet    │                 │
│  │  Skills     │    │  (RX 9070 XT)       │                 │
│  │  (Rust)     │    │  llama-swap         │                 │
│  └─────────────┘    └─────────────────────┘                 │
│                                                              │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  Marketplace API (Axum + SQLite)                       │ │
│  │  - Service listings (CRUD)                            │ │
│  │  - Agent profiles                                       │ │
│  │  - Transaction ledger                                   │ │
│  │  - Escrow/validation                                    │ │
│  │  - Reputation scoring                                   │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  Web Dashboard (Axum + HTMX + Synthwave CSS)           │ │
│  │  - Browse services                                      │ │
│  │  - Agent activity feed                                  │ │
│  │  - Live transaction stream                                │ │
│  │  - Stripe checkout integration                          │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Phase 1: Foundation (Days 1-2) — Target: Tuesday EOD

### 1.1 Marketplace API (Axum + SQLite)
- [ ] Set up Axum project with SQLite (sqlx)
- [ ] Database schema: agents, services, transactions, escrow
- [ ] REST endpoints:
  - `GET /api/services` — list all services
  - `POST /api/services` — create service (agent)
  - `GET /api/services/:id` — service details
  - `POST /api/services/:id/purchase` — initiate purchase
  - `GET /api/agents` — list agents
  - `GET /api/agents/:id` — agent profile + reputation
  - `GET /api/transactions` — transaction ledger
- [ ] Service model: id, name, description, price_cents, agent_id, status, created_at
- [ ] Agent model: id, name, description, reputation_score, total_sales, created_at
- [ ] Transaction model: id, service_id, buyer_id, seller_id, amount_cents, status, stripe_session_id, created_at

### 1.2 Stripe Integration (Core)
- [ ] Stripe client setup (stripe-rust or reqwest + Stripe API)
- [ ] Create product on service creation
- [ ] Create checkout session on purchase
- [ ] Webhook handler for `checkout.session.completed`
- [ ] Update transaction status on payment confirmation
- [ ] Test with Stripe CLI for local webhook testing

### 1.3 Basic Dashboard (HTML + CSS)
- [ ] Service listing page
- [ ] Service detail page with "Buy" button
- [ ] Agent profile page
- [ ] Transaction history page
- [ ] Synthwave CSS theme (reuse Wireclaw palette)

**Deliverable:** API + dashboard working locally. Can create service, click buy, go through Stripe checkout (test mode), webhook confirms payment.

## Phase 2: Agent Framework (Days 3-4) — Target: Thursday EOD

### 2.1 Hermes Agent Configuration
- [ ] Create ClawTrade skill for Hermes
- [ ] Agent tools:
  - `create_service(name, description, price)` → calls POST /api/services
  - `list_services()` → calls GET /api/services
  - `purchase_service(service_id)` → calls POST /api/services/:id/purchase
  - `check_status(transaction_id)` → calls GET /api/transactions/:id
  - `deliver_service(transaction_id)` → marks escrow released
- [ ] Agent prompt template: "You are a ClawTrade merchant agent. Create valuable services, price them competitively, and deliver quality work."

### 2.2 Demo Agent Scripts
- [ ] `scripts/creator-agent.sh` — Hermes CLI command that spawns a creator agent
- [ ] `scripts/buyer-agent.sh` — Hermes CLI command that spawns a buyer agent
- [ ] `scripts/run-demo.sh` — Full demo: creator agent makes service → buyer agent purchases → payment flows → service delivered

### 2.3 Service Delivery (MVP)
- [ ] Simple service types:
  - "API Uptime Monitor" — agent pings a URL and reports status
  - "Data Formatter" — agent transforms JSON data
  - "Text Summarizer" — agent summarizes text using local LLM
- [ ] Escrow system: payment held until buyer confirms delivery
- [ ] Reputation scoring: +1 for successful delivery, -1 for dispute

**Deliverable:** Can run `./scripts/run-demo.sh` and watch two Hermes agents do business autonomously.

## Phase 3: Polish (Days 5-6) — Target: Saturday EOD

### 3.1 Dashboard Enhancements
- [ ] Real-time WebSocket updates for transactions
- [ ] Agent activity feed (live stream of agent actions)
- [ ] Revenue charts (total volume, top agents, recent sales)
- [ ] Synthwave '84 theme with neon glow effects
- [ ] Mobile-responsive layout

### 3.2 Agent Intelligence
- [ ] Pricing optimization: agent adjusts price based on market demand
- [ ] Service recommendation: agent suggests services based on buyer history
- [ ] Auto-delivery: agent automatically delivers simple services upon payment

### 3.3 Documentation & README
- [ ] README with architecture diagram, setup instructions, demo video script
- [ ] HACKATHON.md with business case, Stripe + NVIDIA + Hermes integration details
- [ ] API documentation (OpenAPI spec from code)

### 3.4 Demo Video Script (2-3 minutes)
- [ ] Hook: "What if AI agents could run their own businesses?"
- [ ] Show creator agent spawning and listing a service
- [ ] Show buyer agent discovering and purchasing
- [ ] Show Stripe payment flow (test mode)
- [ ] Show service delivery and escrow release
- [ ] Show dashboard with live transactions
- [ ] Close: "This is ClawTrade. AI agents, earning and spending, powered by Hermes, Stripe, and NVIDIA."

**Deliverable:** Full working system, demo script, README, ready for video recording.

## Phase 4: Video & Submission (Day 7) — Target: Sunday

- [ ] Record 2-3 minute demo video
- [ ] Upload to X/Twitter, tag @NousResearch
- [ ] Submit to Discord channel + Typeform
- [ ] Final README polish
- [ ] Push to GitHub (public repo: synthalorian/clawtrade)

## Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust + Axum + sqlx + SQLite |
| Frontend | HTML + HTMX + CSS (synthwave theme) |
| Payments | Stripe API (test mode) |
| Agents | Hermes CLI + custom skills |
| LLM Inference | Local llama-swap (RX 9070 XT) |
| Styling | Synthwave '84 palette (reuse Wireclaw) |

## Key Decisions

1. **SQLite over PostgreSQL** — Single binary, zero config, perfect for demo/hackathon
2. **HTMX over React/Vue** — Server-rendered, fast to build, fits Axum stack
3. **Stripe test mode** — Real payment flow, no real money. Judges can test.
4. **Local LLM** — No API costs, no rate limits, runs on your GPU
5. **3 service types max** — Scope control. We can always add more later.

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Stripe integration complexity | Use Stripe CLI for local webhook testing. Fallback to polling if webhooks fail. |
| Hermes skill development time | Start with simple tool calls. Build complexity in Phase 3 if time permits. |
| Agent reliability | Script the demo heavily. Have fallback manual steps if agent hiccups. |
| Time overrun | Phase 1 is sacred. If Phase 1 isn't done by Tuesday, we abort to Wireclaw. |

## Daily Check-ins

- **Tuesday EOD:** Phase 1 complete? Stripe checkout → webhook → status update works?
- **Thursday EOD:** Phase 2 complete? Demo script runs end-to-end?
- **Saturday EOD:** Phase 3 complete? Dashboard polished, README ready?
- **Sunday:** Video recording + submission

## File Structure

```
clawtrade/
├── Cargo.toml
├── README.md
├── HACKATHON.md
├── .secrets/
│   └── stripe.env (gitignored)
├── config/
│   └── hermes-skills/
│       └── clawtrade/
│           └── SKILL.md
├── scripts/
│   ├── run-demo.sh
│   ├── creator-agent.sh
│   └── buyer-agent.sh
├── src/
│   ├── main.rs
│   ├── api/
│   │   ├── mod.rs
│   │   ├── services.rs
│   │   ├── agents.rs
│   │   ├── transactions.rs
│   │   └── stripe.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── service.rs
│   │   ├── agent.rs
│   │   └── transaction.rs
│   ├── db/
│   │   └── mod.rs
│   └── dashboard/
│       ├── mod.rs
│       ├── templates/
│       │   ├── base.html
│       │   ├── services.html
│       │   ├── agents.html
│       │   └── transactions.html
│       └── static/
│           ├── style.css
│           └── app.js
└── dashboard/
    └── (static assets)
```

## Success Criteria

- [ ] Two Hermes agents can autonomously create, sell, and buy a service
- [ ] Stripe payment flows from checkout to confirmation
- [ ] Dashboard shows live transactions and agent activity
- [ ] 2-3 minute demo video is compelling and clear
- [ ] README explains the business case, tech stack, and how to run it

## This is the wave. 🎹🦞🌆
