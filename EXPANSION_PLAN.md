# ClawTrade v2 — Deployable Feature Roadmap

## Overview

The MVP is a working demo. v2 turns ClawTrade into a deployable product that can generate real revenue. Every feature here directly supports the business model: transaction fees, hosting, templates, or trust.

## Feature 1: Service Delivery Engine (Revenue-Critical)

**What:** When payment confirms, the service ACTUALLY RUNS. Text summarizer calls local LLM. JSON beautifier formats data. API monitor pings URLs. This is the core value proposition — buyers pay for real output, not promises.

**Business Impact:** Without delivery, there are no repeat customers. No repeat customers = no transaction fees. This is the difference between a toy and a product.

**Implementation:**
- New table: `deliverables` (id, transaction_id, service_type, input_data, output_data, status, created_at)
- Delivery engine: `src/delivery/mod.rs`
  - `text_processing` → POST to local llama-swap with prompt "Summarize: {input}"
  - `data_formatting` → Run JSON parse + pretty-print
  - `api_monitor` → HTTP GET to target URL, record status + latency
- Trigger: webhook marks transaction `paid` → spawn delivery task
- Dashboard: "Deliverables" page showing inputs/outputs
- API: `GET /api/deliverables/:transaction_id` — view result

**Files:**
- `src/db.rs` — add deliverables table
- `src/api/stripe.rs` — trigger delivery on payment
- New: `src/delivery/mod.rs` — delivery engine
- New: `src/models/deliverable.rs` — Deliverable model
- `src/dashboard.rs` — deliverables page

**Time:** 2.5 hours
**Priority:** HIGHEST — without this, no real business

---

## Feature 2: Stripe Connect Onboarding (Revenue-Critical)

**What:** Real sellers connect their actual Stripe account. Platform takes a fee. Money flows to real bank accounts.

**Business Impact:** This is how ClawTrade makes money. 5-15% of every transaction. Without Stripe Connect, it's just test mode.

**Implementation:**
- Replace single Stripe key with Stripe Connect
- New API: `POST /api/stripe/connect` — create Connected Account for seller
- New API: `POST /api/stripe/account_link` — onboarding URL for seller
- Update checkout: use `transfer_data` to route payment to seller, platform fee to ClawTrade
- Dashboard: "Connect Stripe" button for agents
- Webhook: handle `account.updated`, `transfer.created`

**Files:**
- `src/api/stripe.rs` — add Connect endpoints
- `src/models/agent.rs` — add `stripe_account_id` field
- `src/dashboard.rs` — Stripe Connect onboarding UI

**Time:** 3 hours
**Priority:** HIGHEST — this IS the revenue mechanism

---

## Feature 3: Agent Reputation + Reviews (Trust = Revenue)

**What:** Buyers leave 1-5 star ratings. Agents get reputation scores, badges. Bad agents get filtered out. Good agents get featured.

**Business Impact:** Trust drives transaction volume. eBay, Amazon, Upwork — all built on reputation. No reputation = no trust = no sales = no fees.

**Implementation:**
- New table: `reviews` (id, transaction_id, rating, comment, created_at)
- New API endpoints:
  - `POST /api/reviews` — create review (only after delivery)
  - `GET /api/agents/:id/reviews` — get agent reviews
- Update `Agent` model: `avg_rating` field (computed from reviews)
- Dashboard: show star ratings on agent cards
- Badge system: 🆕 (0 sales), ⭐ (1+), ⭐⭐ (5+), 🏆 (10+), 💎 (50+)
- Filter: agents with avg_rating < 3.0 are hidden from search

**Files:**
- `src/db.rs` — add reviews table
- `src/models/agent.rs` — add avg_rating, badge logic
- New: `src/models/review.rs` — Review model
- New: `src/api/reviews.rs` — review endpoints
- `src/dashboard.rs` — show stars and badges

**Time:** 1.5 hours
**Priority:** HIGH — trust is the foundation of marketplaces

---

## Feature 4: Real-Time WebSocket Dashboard (Engagement)

**What:** Live updates as agents transact. No refresh needed. Buyers see "someone just purchased this" — social proof drives sales.

**Business Impact:** Engagement = time on site = more transactions. Social proof ("3 people bought this in the last hour") increases conversion rates by 20-30%.

**Implementation:**
- Add `tokio::sync::broadcast` channel in `main.rs` for dashboard events
- Broadcast events on: service creation, purchase initiation, payment confirmation, delivery completion
- Dashboard page opens WebSocket connection to `/ws`
- JavaScript receives events and updates DOM in real-time
- Add "live indicator" pulsing dot, "recent activity" ticker

**Files:**
- `src/main.rs` — add broadcast channel, inject into handlers
- `src/dashboard.rs` — add WebSocket handler, JS for live updates
- New: `src/websocket.rs` — WebSocket event types and broadcast logic

**Time:** 2 hours
**Priority:** MEDIUM — nice to have, not revenue-critical

---

## Feature 5: Agent Hosting API (SaaS Revenue)

**What:** HTTP API that lets businesses spawn, manage, and monitor their agents remotely. This is the infrastructure layer that justifies the $29/mo hosting fee.

**Business Impact:** Turns ClawTrade from a marketplace into a platform. Businesses don't just visit — they integrate. Recurring revenue.

**Implementation:**
- New API endpoints:
  - `POST /api/v1/agents/spawn` — create and start an agent (requires API key)
  - `GET /api/v1/agents/:id/status` — check agent health, revenue, services
  - `POST /api/v1/agents/:id/stop` — pause agent
  - `GET /api/v1/agents/:id/logs` — agent activity log
- API key authentication (simple token-based)
- Rate limiting: 100 req/min for free, 1000 req/min for paid
- Dashboard: "API Keys" section for developers

**Files:**
- New: `src/api/v1/mod.rs` — v1 API routes
- New: `src/api/v1/agents.rs` — agent management endpoints
- New: `src/auth.rs` — API key authentication middleware
- `src/dashboard.rs` — API key management UI

**Time:** 2.5 hours
**Priority:** HIGH — enables SaaS revenue stream

---

## Feature 6: Escrow System (Trust + Fee Revenue)

**What:** Hold payments until buyer confirms service delivery. Platform takes 2-3% escrow fee.

**Business Impact:** Critical for B2B transactions. Without escrow, buyers won't trust agents with large orders. Escrow fee is pure margin.

**Implementation:**
- New transaction statuses: `pending` → `escrow` → `paid` → `released`
- On payment: funds held in platform Stripe account (not released to seller)
- Buyer clicks "Confirm Delivery" → funds released to seller
- Auto-release after 7 days if buyer doesn't dispute
- Dispute flow: buyer opens ticket, platform mediates, decides split
- Dashboard: escrow status on transactions, dispute UI

**Files:**
- `src/models/transaction.rs` — add escrow status flow
- `src/api/transactions.rs` — add confirm/delivery/dispute endpoints
- `src/api/stripe.rs` — handle escrow holds and releases
- `src/dashboard.rs` — escrow UI, dispute forms

**Time:** 3 hours
**Priority:** HIGH — enables high-value transactions

---

## Feature 7: Pricing Intelligence (Market Intelligence Revenue)

**What:** Agents auto-adjust prices based on market demand. Platform sells aggregate pricing data.

**Business Impact:** Data is the new oil. Aggregate demand data is valuable to businesses deciding what services to build.

**Implementation:**
- Market analysis function: count services by type, compute average price, track sales velocity
- Pricing engine: `src/pricing/mod.rs`
- Rules:
  - If 3+ services of same type, price = avg - 10% (undercut)
  - If only service of type, price = avg + 20% (premium)
  - If no sales in 24h, price -= 10%
  - If 5+ sales in 1h, price += 15%
- Data API: `GET /api/v1/market/trends` — returns trending services, avg prices (paid endpoint, requires API key)
- Dashboard: "Market Trends" page with charts

**Files:**
- New: `src/pricing/mod.rs` — pricing engine
- `src/models/service.rs` — add price_history JSON field
- `src/dashboard.rs` — price trend indicators, market trends page
- `src/api/v1/` — market data endpoints

**Time:** 2 hours
**Priority:** MEDIUM — data revenue is long-term

---

## Feature 8: Agent Templates (Template Revenue)

**What:** Pre-built agent configurations that users can clone. "The Arbitrage Bot" — $199 one-time.

**Business Impact:** High-margin digital product. Build once, sell infinitely. Also reduces onboarding friction.

**Implementation:**
- New table: `templates` (id, name, description, price_cents, config_json, sales_count, created_at)
- Config JSON defines: personality, services to create, pricing strategy, delivery logic
- API: `POST /api/templates/:id/clone` — copies template to user's agent
- Dashboard: "Template Store" page with preview cards
- Payment: Stripe checkout for template purchase

**Files:**
- `src/db.rs` — add templates table
- New: `src/models/template.rs` — Template model
- New: `src/api/templates.rs` — template endpoints
- `src/dashboard.rs` — template store UI

**Time:** 2 hours
**Priority:** MEDIUM — nice revenue stream, not core

---

## Implementation Order (Revenue-First)

| Order | Feature | Time | Revenue Impact | Why First? |
|-------|---------|------|----------------|------------|
| 1 | #2 Stripe Connect | 3h | HIGHEST | This IS the money flow |
| 2 | #1 Service Delivery | 2.5h | HIGHEST | Without delivery, no repeat customers |
| 3 | #6 Escrow | 3h | HIGH | Enables high-value B2B transactions |
| 4 | #3 Reputation | 1.5h | HIGH | Trust = transaction volume |
| 5 | #5 Agent Hosting API | 2.5h | HIGH | SaaS recurring revenue |
| 6 | #4 WebSocket | 2h | MEDIUM | Engagement, social proof |
| 7 | #7 Pricing Intel | 2h | MEDIUM | Data revenue |
| 8 | #8 Templates | 2h | MEDIUM | Digital product sales |

**Total: ~18.5 hours** — still very doable over 6 days.

**Revenue by Milestone:**
- After #1 + #2: Platform can process real payments, take real fees
- After #3 + #6: B2B-ready, high-value transactions possible
- After #5: SaaS revenue stream active
- After all 8: Full platform, all revenue streams operational

---

## Deployment Path

### Phase A: Hackathon Demo (Current)
- Single binary, SQLite, test mode Stripe
- Runs locally on Omarchy
- Demo scripts simulate agent behavior

### Phase B: Private Beta (Week 1-2 post-hackathon)
- Deploy to Fly.io or Railway
- Stripe Connect onboarding for first 10 sellers
- Real payments, real fees
- SQLite → PostgreSQL migration

### Phase C: Public Launch (Month 1-2)
- Custom domain (clawtrade.io)
- Agent Hosting API live
- Template store open
- Market Intelligence API for paid subscribers

### Phase D: Scale (Month 3+)
- Kubernetes cluster for agent workloads
- Multi-region deployment
- Enterprise tier ($499/mo white-label)
- Integration partnerships (Hermes, Stripe, NVIDIA)

---

## Next Session Focus

When we resume, we start with **Feature 2: Stripe Connect**. This is the single most important feature — it turns test mode into real money. Without it, everything else is just decoration.

After Stripe Connect works, we move to **Feature 1: Service Delivery**. Then the platform has real value: create service → get paid → deliver value → earn reputation.

That's the loop that generates revenue.

## This is the wave. 🎹🦞🌆
