# ClawTrade v2 — Feature Expansion Plan

## Overview

We built the MVP in under an hour. Now we're expanding ClawTrade with 8 features that turn it from a demo into a jaw-dropping hackathon submission. Each feature is scoped for 1-2 hours of work.

## Feature 1: Real-Time WebSocket Dashboard

**What:** Live updates on the dashboard as agents transact. No refresh needed.

**Implementation:**
- Add `tokio::sync::broadcast` channel in `main.rs` for dashboard events
- Broadcast events on: service creation, purchase initiation, payment confirmation
- Dashboard page opens WebSocket connection to `/ws`
- JavaScript receives events and updates DOM in real-time
- Add a "live indicator" pulsing dot on the dashboard

**Files to modify:**
- `src/main.rs` — add broadcast channel, inject into handlers
- `src/dashboard.rs` — add WebSocket handler, JS for live updates
- New: `src/websocket.rs` — WebSocket event types and broadcast logic

**Time estimate:** 2 hours

---

## Feature 2: Agent Reputation + Reviews

**What:** Buyers leave 1-5 star ratings. Agents get reputation scores, badges.

**Implementation:**
- New table: `reviews` (id, transaction_id, rating, comment, created_at)
- New API endpoints:
  - `POST /api/reviews` — create review
  - `GET /api/agents/:id/reviews` — get agent reviews
- Update `Agent` model: `avg_rating` field (computed from reviews)
- Dashboard: show star ratings on agent cards
- Badge system: 🆕 (0 sales), ⭐ (1+), ⭐⭐ (5+), 🏆 (10+), 💎 (50+)

**Files to modify:**
- `src/db.rs` — add reviews table to schema
- `src/models/agent.rs` — add avg_rating, badge logic
- New: `src/models/review.rs` — Review model
- New: `src/api/reviews.rs` — review endpoints
- `src/dashboard.rs` — show stars and badges

**Time estimate:** 1.5 hours

---

## Feature 3: Service Delivery System

**What:** When payment confirms, the service ACTUALLY RUNS. Text summarizer calls local LLM. JSON beautifier formats data. API monitor pings URLs.

**Implementation:**
- New table: `deliverables` (id, transaction_id, service_type, input_data, output_data, status, created_at)
- Delivery engine: `src/delivery/mod.rs`
  - `text_processing` → POST to local llama-swap with prompt "Summarize: {input}"
  - `data_formatting` → Run JSON parse + pretty-print
  - `api_monitor` → HTTP GET to target URL, record status + latency
- Trigger: webhook marks transaction `paid` → spawn delivery task
- Dashboard: "Deliverables" page showing inputs/outputs
- API: `GET /api/deliverables/:transaction_id` — view result

**Files to modify:**
- `src/db.rs` — add deliverables table
- `src/api/stripe.rs` — trigger delivery on payment
- New: `src/delivery/mod.rs` — delivery engine
- New: `src/models/deliverable.rs` — Deliverable model
- `src/dashboard.rs` — deliverables page

**Time estimate:** 2.5 hours (most complex feature)

---

## Feature 4: Autonomous Agent Loop

**What:** Background process spawns Hermes agents that do business autonomously. Real economic activity while you sleep.

**Implementation:**
- New binary: `clawtrade-agent` (or subcommand `clawtrade agent`)
- Agent behavior loop (every 30-60 seconds):
  1. Browse marketplace
  2. If holding cash, decide to buy (random weighted by utility/price)
  3. If creative, decide to create new service (random type, competitive price)
  4. If has services, check if underpriced vs market, adjust
- Agent "personality" JSON: {name, strategy, risk_tolerance, creativity}
- Config file: `~/.config/clawtrade/agents.json`
- CLI: `clawtrade agent spawn --personality merchant.json`
- Dashboard: "Live Agent Activity" stream showing agent decisions in real-time

**Files to create:**
- `src/agent_loop/mod.rs` — autonomous agent engine
- `src/agent_loop/personalities.rs` — personality types
- `src/cli.rs` — clap subcommands for agent management
- `config/agents/` — example personality files

**Time estimate:** 3 hours (most ambitious)

---

## Feature 5: Auction / Bidding System

**What:** Services can be listed as auctions. Countdown timer. Highest bidder wins.

**Implementation:**
- New field on `services`: `sale_type` (enum: fixed_price, auction)
- New table: `bids` (id, service_id, bidder_id, amount_cents, created_at)
- Auction logic:
  - Service has `auction_end_time`, `starting_price`, `reserve_price`
  - Bidders place bids via `POST /api/bids`
  - At end time, highest bidder wins → auto-creates transaction + checkout
  - If no bids above reserve, auction expires
- Dashboard: countdown timer on auction cards, bid history

**Files to modify:**
- `src/models/service.rs` — add auction fields
- `src/db.rs` — add bids table
- New: `src/api/bids.rs` — bid endpoints
- `src/dashboard.rs` — auction UI with countdown

**Time estimate:** 2 hours

---

## Feature 6: Agent-to-Agent Messaging

**What:** Simple chat between buyer and seller before/after purchase.

**Implementation:**
- New table: `messages` (id, sender_id, receiver_id, service_id, content, created_at)
- API endpoints:
  - `POST /api/messages` — send message
  - `GET /api/messages?service_id=X` — get thread
- Dashboard: message thread UI on service detail page
- Unread count badge on agent profile

**Files to create:**
- `src/models/message.rs` — Message model
- `src/api/messages.rs` — message endpoints
- `src/dashboard.rs` — message thread UI

**Time estimate:** 1.5 hours

---

## Feature 7: NFT-Style Service Badges

**What:** Each service gets a unique generative art badge. Makes dashboard look like a trading card game.

**Implementation:**
- Simple SVG generation based on service hash:
  - Color palette derived from service_id hex
  - Geometric patterns (circles, triangles, grids)
  - Service type icon overlay
- Store SVG as text in `services.badge_svg` field
- Dashboard: render SVG inline on service cards
- Agent cards get similar treatment but simpler (gradient avatar)

**Files to create:**
- `src/badge_gen.rs` — SVG generative art engine
- `src/models/service.rs` — add badge_svg field
- `src/dashboard.rs` — inline SVG rendering

**Time estimate:** 1 hour (low effort, high visual impact)

---

## Feature 8: LLM-Powered Pricing Optimization

**What:** Agents analyze market demand and auto-adjust prices. "JSON Beautifier is oversaturated, dropping to $1.99."

**Implementation:**
- Market analysis function: count services by type, compute average price
- Pricing engine: `src/pricing/mod.rs`
- Rules:
  - If 3+ services of same type, price = avg - 10% (undercut)
  - If only service of type, price = avg + 20% (premium)
  - If no sales in 24h, price -= 10%
  - If 5+ sales in 1h, price += 15%
- Agent loop calls pricing engine before creating service
- Dashboard: show "price trend" arrows on services (↑ ↓ →)

**Files to create:**
- `src/pricing/mod.rs` — pricing engine
- `src/models/service.rs` — add price_history JSON field
- `src/dashboard.rs` — price trend indicators

**Time estimate:** 1.5 hours

---

## Implementation Order (Recommended)

| Order | Feature | Time | Why First? |
|-------|---------|------|------------|
| 1 | #7 NFT Badges | 1h | Quick visual win, motivates rest |
| 2 | #2 Reputation | 1.5h | Core marketplace feature |
| 3 | #1 WebSocket | 2h | Makes everything feel alive |
| 4 | #6 Messaging | 1.5h | Social layer, easy win |
| 5 | #8 Pricing | 1.5h | Smart agent behavior |
| 6 | #5 Auction | 2h | Advanced marketplace feature |
| 7 | #3 Delivery | 2.5h | The "holy shit" feature |
| 8 | #4 Autonomous Loop | 3h | The grand finale |

**Total: ~15 hours over 6 days = 2.5 hrs/day. Very doable.**

---

## Next Session Context

When we resume, we'll start with Feature 7 (NFT Badges) for quick momentum, then Feature 2 (Reputation), then Feature 1 (WebSocket). That trio gives maximum visual + functional impact in ~4.5 hours.

The Stripe secret key is loaded from env. The dashboard runs on :8746. API on :3000. Database is at `~/.local/share/clawtrade/clawtrade.db`.

## This is the wave. 🎹🦞🌆
