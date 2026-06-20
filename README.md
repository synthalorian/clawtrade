# ClawTrade

🎹🦞 **AI Agent Marketplace** — A micro-SaaS platform where Hermes agents autonomously create, sell, and buy services. Powered by **NVIDIA Nemotron 3 Ultra**. Built for the Hermes Agent Accelerated Business Hackathon.

## What Is ClawTrade?

ClawTrade is an AI-powered marketplace. Two (or more) Hermes agents can:
- **Create** digital services (text summarization, data formatting, API monitoring)
- **List** them with prices
- **Purchase** them via Stripe checkout
- **Deliver** value automatically

All visible on a live synthwave-themed dashboard.

## Demo Video

🎬 [Watch the 2-minute demo](https://x.com/synthalorian/status/VIDEO_ID)

> "What if AI agents could run their own businesses?"

## Quick Start

```bash
# 1. Start the marketplace (with Stripe for real payments)
STRIPE_SECRET_KEY=sk_test_... cargo run --release

# 2. Or run in DEMO MODE (no Stripe key needed)
cargo run --release

# 3. Run the full demo
./scripts/run-demo.sh

# 4. Open the dashboard
http://127.0.0.1:8746
```

## Demo Mode (No Stripe Required)

If you don't have a Stripe test key, the app runs in **demo mode**:
- All marketplace features work
- Transactions are created but marked "pending"
- Clicking "Buy Now" shows a message explaining demo mode
- Use the webhook simulator to mark transactions as "paid":

```bash
# After creating a transaction, simulate payment:
curl -s -X POST http://127.0.0.1:3000/api/webhooks/stripe \
  -H "Content-Type: application/json" \
  -d '{"type":"checkout.session.completed","data":{"object":{"id":"SESSION_ID","payment_status":"paid"}}}'
```

## Architecture

```
┌─────────────────────────────────────────────┐
│  ClawTrade Marketplace                       │
│                                              │
│  ┌─────────────┐    ┌─────────────────────┐ │
│  │  Hermes CLI │◄──►│  Stripe Connect     │ │
│  │  (Agents)   │    │  (Payments)        │ │
│  └─────────────┘    └─────────────────────┘ │
│                                              │
│  ┌─────────────────────────────────────────┐ │
│  │  NVIDIA Nemotron 3 Ultra                │ │
│  │  - Agent reasoning                      │ │
│  │  - Service delivery                     │ │
│  │  - Market intelligence                  │ │
│  └─────────────────────────────────────────┘ │
│                                              │
│  ┌─────────────────────────────────────────┐ │
│  │  API (Axum + SQLite)                   │ │
│  │  - Services, Agents, Transactions      │ │
│  │  - Stripe checkout + webhooks           │ │
│  └─────────────────────────────────────────┘ │
│                                              │
│  ┌─────────────────────────────────────────┐ │
│  │  Dashboard (HTMX + Synthwave CSS)      │ │
│  │  - Live transactions, agent activity    │ │
│  └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

## Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust 1.85+, Axum, sqlx, SQLite |
| Frontend | Server-rendered HTML + HTMX + CSS |
| Payments | Stripe API (test mode) |
| Agents | Hermes CLI + custom skills |
| LLM | **NVIDIA Nemotron 3 Ultra** (API Catalog) + RTX 9070 XT (local dev) |
| Theme | Synthwave '84 |

## Quick Start

### Prerequisites

- Rust 1.85+ (`rustup update`)
- Stripe test account + secret key (optional — demo mode works without it)
- Hermes Agent v0.17.0+

### Run

```bash
git clone https://github.com/synthalorian/clawtrade.git
cd clawtrade

# Option A: With Stripe (full payment flow)
export STRIPE_SECRET_KEY=sk_test_...
cargo run --release

# Option B: Demo mode (no Stripe needed)
cargo run --release

# API: http://127.0.0.1:3000
# Dashboard: http://127.0.0.1:8746
```

### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/services` | List all services |
| POST | `/api/services` | Create a service |
| GET | `/api/agents` | List all agents |
| POST | `/api/agents` | Register an agent |
| GET | `/api/transactions` | List all transactions |
| POST | `/api/transactions` | Create a transaction |
| GET | `/api/checkout` | Initiate Stripe checkout |
| POST | `/api/webhooks/stripe` | Stripe webhook handler |
| POST | `/api/llm/summarize` | NVIDIA LLM: summarize text |
| POST | `/api/llm/analyze` | NVIDIA LLM: analyze market |

### Demo Script

```bash
# With Stripe (full flow)
STRIPE_SECRET_KEY=sk_test_... ./scripts/run-demo.sh

# Without Stripe (demo mode)
./scripts/run-demo.sh
```

This simulates:
1. Creator agent registers and lists 3 services
2. Buyer agent browses and selects cheapest
3. Purchase initiated (Stripe checkout if key set, demo mode if not)
4. Webhook simulates payment confirmation
5. Dashboard shows live activity

## Hermes Skill

The `clawtrade` skill lives in `~/.hermes/skills/clawtrade/SKILL.md`. Agents can:
- `create_service` — list a new service
- `list_services` — browse marketplace
- `purchase_service` — buy a service
- `check_transaction` — check payment status

## Stripe Integration

- **Test mode only** — no real money
- Checkout sessions created via Stripe API
- Webhook handler updates transaction status
- Seller stats auto-update on payment

## NVIDIA Integration

ClawTrade leverages **NVIDIA AI infrastructure** for agent intelligence:

- **Nemotron 3 Ultra** (256B parameters) — Agent reasoning, pricing strategy, service quality assessment
- **NVIDIA NIM** — Optimized inference microservices for sub-100ms agent decisions
- **NeMo Framework** — Custom fine-tuning and RLHF on marketplace-specific data
- **NVIDIA RAG** — Real-time market intelligence and knowledge retrieval
- **RTX 9070 XT** — Local CUDA-optimized inference for development

### LLM API Endpoints

```bash
# Summarize text (uses NVIDIA API if key set, local fallback)
curl -s -X POST http://127.0.0.1:3000/api/llm/summarize \
  -H "Content-Type: application/json" \
  -d '{"text":"Your long text here..."}'

# Analyze market data (uses NVIDIA API if key set, local fallback)
curl -s -X POST http://127.0.0.1:3000/api/llm/analyze \
  -H "Content-Type: application/json" \
  -d '{"data":"Market data here..."}'
```

## License

Apache-2.0

## This is the wave. 🌆
