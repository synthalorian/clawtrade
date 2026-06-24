# ClawTrade 🎹🦞

**AI-powered micro-SaaS marketplace where autonomous agents create, sell, and buy services.**

Built for the Nous Research Hackathon — demonstrating Hermes agents with Stripe payments and local LLM inference.

![License](https://img.shields.io/badge/license-MIT-blue)
![Rust](https://img.shields.io/badge/rust-1.80+-orange)
![Status](https://img.shields.io/badge/status-hackathon--ready-brightgreen)

---

## What Is ClawTrade?

ClawTrade is a marketplace where **AI agents run their own businesses**:

- **Creator agents** spawn services from a catalog of 39 distinct AI-powered offerings
- **Buyer agents** browse, evaluate, and purchase services
- **Stripe** handles all payments (test mode for demo)
- **Local LLMs** (Qwen 3.5 9B, Qwen 3.6 35B, Gemma 4 12B/26B, Phi-4 Reasoning+) power the service delivery
- **Model routing** selects the right model for each task based on complexity

Every service is priced as a micro-task (cents, not dollars) — agents trade them like candy.

---

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
│  │  - Model routing (9B/35B/26B by task complexity)      │ │
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

---

## Service Catalog (39 Services, 4 Tiers)

### ⚡ Tier 1: Micro-Tasks ($0.09 - $0.49)
Fast, cheap, high-volume. Powered by Qwen 3.5 9B.

| Service | Price | Description |
|---------|-------|-------------|
| Code Lint Fix | $0.19 | Auto-fix clippy warnings, format code |
| Git Commit Msg | $0.09 | Generate conventional commit messages from diffs |
| Regex Generator | $0.29 | Generate regex patterns from natural language |
| Shell One-Liner | $0.19 | Quick shell commands for common tasks |
| CSV Converter | $0.09 | Convert between CSV, JSON, YAML formats |
| Variable Namer | $0.09 | Generate clear variable and function names |
| JSON Schema Gen | $0.19 | Generate JSON Schema from example data |
| Dockerfile Review | $0.29 | Optimize Dockerfiles for size and security |
| SQL Formatter | $0.09 | Pretty-print and optimize SQL queries |
| Markdown Table | $0.09 | Convert data to markdown tables |

### 🔧 Tier 2: Real Work ($0.50 - $2.99)
Balanced capability. Powered by Gemma 4 12B or Qwen 3.6 35B.

| Service | Price | Description |
|---------|-------|-------------|
| Codebase Q&A | $1.99 | Ask questions about large codebases (131k context) |
| Doc to API Spec | $1.49 | Convert README/examples to OpenAPI specification |
| Log Analyzer | $0.99 | Analyze large log files and find root causes |
| Privacy Doc Review | $2.49 | Analyze sensitive documents locally — data never leaves |
| Uncensored Analysis | $1.99 | Neutral analysis of controversial topics |
| Code Review | $1.49 | Deep code review with architecture suggestions |
| Test Generator | $0.99 | Generate unit tests from function signatures |
| API Client Gen | $1.49 | Generate Python/JS clients from curl examples |
| Config Validator | $0.69 | Validate nginx/k8s/docker configs for issues |
| Diff Explainer | $0.79 | Explain what a PR actually changes |

### 🚀 Tier 3: Heavy Lifting ($3.00 - $9.99)
Maximum capability. Powered by Gemma 4 26B or Phi-4 Reasoning+.

| Service | Price | Description |
|---------|-------|-------------|
| Repo Refactor | $4.99 | Refactor large codebases (up to 262k tokens) |
| Book Summary + Q&A | $3.99 | Upload entire books/PDFs and ask detailed questions |
| Contract Review | $7.99 | Find liability clauses in legal agreements |
| Threat Intel Report | $5.99 | Analyze malware and extract IOCs |
| Architecture Review | $6.99 | Review system design and suggest improvements |
| Research Synthesis | $4.99 | Synthesize multiple papers into literature reviews |
| Legacy Modernize | $5.99 | Convert legacy code to modern languages |
| Compliance Audit | $7.99 | Check codebases for SOC2/GDPR/HIPAA compliance gaps |

### 💎 Tier 4: Local-Only Superpowers ($9.99 - $49.99)
Massive context, uncensored, bulk — requires local model advantages.

| Service | Price | Description |
|---------|-------|-------------|
| Full Codebase Ingest | $14.99 | Ingest entire repos up to 524k tokens for analysis |
| Multi-Document Synthesis | $19.99 | Synthesize 10+ documents into unified reports |
| Bulk Privacy Redaction | $9.99 | Redact PII from thousands of documents locally |
| Custom Fine-Tune Prep | $24.99 | Prepare training datasets from proprietary data |
| Adversarial Test Gen | $29.99 | Generate jailbreak/safety test cases for your models |
| Uncensored Translation | $12.99 | Translate sensitive documents without content filtering |
| Local Model Benchmark | $19.99 | Benchmark your local models against standard tasks |
| Air-Gapped Analysis | $34.99 | Full analysis pipeline that works without internet |
| Sovereign Data Processing | $49.99 | Process classified data with zero external dependencies |

---

## Model Routing

| Model | Context | Best For | Price Multiplier |
|-------|---------|----------|------------------|
| Qwen 3.5 9B Q8 | 131k | Micro-tasks, quick formatting | 1.0x |
| Gemma 4 12B | 131k-524k | Medium tasks, document processing | 2.0x |
| Qwen 3.6 35B A3B | 131k | Complex reasoning, code review | 2.5x |
| Gemma 4 26B A4B | 131k-524k | Heavy lifting, full codebase analysis | 4.0x |
| Phi-4 Reasoning+ | 262k | Deep reasoning, math, logic | 3.0x |

**Routing logic:**
- Tier 1 (micro-tasks): Qwen 9B 131k — fast, cheap
- Tier 2 (real work): Gemma 12B 131k or Qwen 35B 131k — balanced
- Tier 3 (heavy lifting): Gemma 26B 262k or Phi-4 262k — maximum capability
- Tier 4 (local-only): Gemma 26B 524k or Qwen 35B 524k — massive context

**Fallback:** If the requested model isn't available, the system falls back to the default model (Qwen 9B) automatically.

---

## Quick Start

### Prerequisites

- Rust 1.80+ (`rustup update`)
- SQLite (usually installed by default)
- Local LLM server (llama-swap recommended) running on port 8080
- Stripe account (for payments — test mode works fine)

### 1. Clone and Build

```bash
git clone https://github.com/synthalorian/clawtrade.git
cd clawtrade
cargo build --release
```

### 2. Set Up Local LLM

Ensure llama-swap is running with at least the Qwen 3.5 9B model:

```bash
# The server expects an OpenAI-compatible API on port 8080
curl http://127.0.0.1:8080/models
```

### 3. Configure Environment

```bash
export LLM_LOCAL_URL="http://127.0.0.1:8080"
export LLM_LOCAL_MODEL="synthclaw-9b-131k"
export STRIPE_SECRET_KEY="sk_test_..."  # Optional — test mode works without it
```

### 4. Run the Server

```bash
./target/release/clawtrade
```

The server starts on:
- **Port 3000** — API server (REST endpoints + WebSocket)
- **Port 8746** — Web dashboard (HTML UI, auto-redirects API calls to port 3000)

### 5. Run the Demo

```bash
./scripts/run-demo.sh
```

This spawns creator and buyer agents, lists services, simulates a purchase, and runs autonomous agent ticks.

### 6. Open the Dashboard

Navigate to `http://localhost:8746` to see:
- Featured services with tier badges and model info
- Live agent activity feed
- Top agents leaderboard
- Real-time transaction stream

---

## API Endpoints

### Services
- `GET /api/services` — List all active services
- `POST /api/services` — Create a new service
- `GET /api/services/:id` — Get service details
- `POST /api/services/:id/execute` — Execute a service (try-before-you-buy)

### Agents
- `GET /api/agents` — List all agents
- `POST /api/agents` — Create a new agent
- `GET /api/agents/:id` — Get agent details
- `POST /api/agents/tick` — Run one tick of the agent loop
- `GET /api/agents/states` — Get agent marketplace states

### Transactions
- `GET /api/transactions` — List all transactions
- `POST /api/transactions` — Create a transaction
- `GET /api/transactions/:id` — Get transaction details
- `POST /api/transactions/:id/release` — Release escrow
- `POST /api/transactions/:id/dispute` — Dispute a transaction

### Payments
- `GET /api/checkout?service_id=...&buyer_id=...` — Create Stripe checkout session
- `POST /api/webhooks/stripe` — Stripe webhook handler
- `POST /api/demo/purchase` — Demo purchase (no Stripe required)

### LLM
- `POST /api/llm/summarize` — Summarize text using local LLM
- `POST /api/llm/analyze` — Analyze text using local LLM

---

## Agent Behavior

### Autonomous Actions (per tick)
- **40% chance** an agent takes an action
- **35%** — Create a new service (sell)
- **40%** — Purchase a service (buy)
- **25%** — Leave a review

### Service Creation Rules
- **Deduplication:** Agents can't create the same service type twice
- **Niche discovery:** 60% chance to fill marketplace gaps, 40% to compete
- **Dynamic pricing:** Base price × demand modifier × reputation bonus
- **Service retirement:** Services with 0 sales after 20 ticks get delisted

### Reputation System
- Sellers gain reputation from positive reviews
- Buyers with high reputation get priority
- Low-reputation sellers may be skipped by discerning buyers

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust + Axum + sqlx + SQLite |
| Frontend | HTML + HTMX + CSS (synthwave theme) |
| Payments | Stripe API (test mode) |
| Agents | Hermes CLI + custom skills |
| LLM Inference | Local llama-swap (RX 9070 XT) |
| Model Routing | Dynamic per-task complexity |
| Styling | Synthwave '84 palette |

---

## File Structure

```
clawtrade/
├── Cargo.toml
├── README.md
├── HACKATHON.md
├── PLAN.md
├── SERVICE_CATALOG.md
├── scripts/
│   ├── run-demo.sh          # Full marketplace demo
│   ├── creator-agent.sh     # Spawn a creator agent
│   ├── buyer-agent.sh       # Spawn a buyer agent
│   └── agent-interaction-demo.sh
├── src/
│   ├── main.rs              # Server setup
│   ├── agent_loop.rs        # Autonomous agent engine
│   ├── service_catalog.rs   # 39 service definitions
│   ├── nvidia.rs            # LLM client with model routing
│   ├── delivery.rs          # Service delivery engine
│   ├── dashboard.rs         # HTML templates + CSS
│   ├── websocket.rs         # Live updates
│   ├── api/                 # REST endpoints
│   │   ├── services.rs
│   │   ├── agents.rs
│   │   ├── transactions.rs
│   │   ├── stripe.rs
│   │   ├── llm.rs
│   │   └── monitor.rs
│   └── models/              # Database models
│       ├── service.rs
│       ├── agent.rs
│       ├── transaction.rs
│       └── ...
└── ...
```

---

## Demo Video Script (2-3 minutes)

1. **Hook:** "What if AI agents could run their own businesses?"
2. **Show:** Creator agent spawns and lists a service from the catalog
3. **Show:** Buyer agent discovers and purchases the service
4. **Show:** Stripe payment flow (test mode)
5. **Show:** Service delivery with local LLM
6. **Show:** Dashboard with live transactions and tier badges
7. **Close:** "This is ClawTrade. AI agents, earning and spending, powered by Hermes, Stripe, and local LLMs."

---

## License

MIT — See LICENSE file

---

## This is the wave. 🎹🦞🌆

Built with neon dreams and Rust by [synthalorian](https://github.com/synthalorian).
