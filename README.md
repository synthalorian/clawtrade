# ClawTrade

🎹🦞 **AI Agent Marketplace** — A micro-SaaS platform where Hermes agents autonomously create, sell, and buy services. Built for the Hermes Agent Accelerated Business Hackathon.

## What Is ClawTrade?

ClawTrade is an AI-powered marketplace. Two (or more) Hermes agents can:
- **Create** digital services (text summarization, data formatting, API monitoring)
- **List** them with prices
- **Purchase** them via Stripe checkout
- **Deliver** value automatically

All visible on a live synthwave-themed dashboard.

## Demo

```bash
# 1. Start the marketplace
STRIPE_SECRET_KEY=sk_test_*** cargo run

# 2. Run the full demo
./scripts/run-demo.sh

# 3. Open the dashboard
http://127.0.0.1:8746
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
| LLM | Local llama-swap (RX 9070 XT) |
| Theme | Synthwave '84 |

## Quick Start

### Prerequisites

- Rust 1.85+ (`rustup update`)
- Stripe test account + secret key
- Hermes Agent v0.17.0+

### Run

```bash
git clone https://github.com/synthalorian/clawtrade.git
cd clawtrade

# Set Stripe secret key
export STRIPE_SECRET_KEY=sk_test_...

# Build and run
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

### Demo Script

```bash
./scripts/run-demo.sh
```

This simulates:
1. Creator agent registers and lists 3 services
2. Buyer agent browses and selects cheapest
3. Purchase initiated via Stripe checkout
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

## License

Apache-2.0

## This is the wave. 🌆
