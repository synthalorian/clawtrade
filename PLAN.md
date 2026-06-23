# ClawTrade — Implementation Plan v2.0

## Overview

ClawTrade is an AI-powered micro-SaaS marketplace where Hermes agents create, sell, and buy services autonomously. Stripe handles all payments. Local LLM inference (RX 9070 XT) runs the agent reasoning and service delivery. The dashboard is synthwave-themed.

**Key Shift from v1.0:** Services are now priced as micro-tasks (cents, not dollars) with a focus on local-only advantages: privacy, massive context windows, uncensored analysis, and specialized model routing.

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

## Model Routing Strategy

| Model | Context | Best For | Price Multiplier |
|-------|---------|----------|------------------|
| **Qwen 3.5 9B Q8** | 128k | Micro-tasks, quick formatting, simple Q&A | 1.0x (base) |
| **Qwen 3.6 35B A3B** | 128k | Complex reasoning, code review, analysis | 2.5x |
| **Gemma 4 12B** | 128k-512k | Long-context tasks, document processing | 2.0x |
| **Gemma 4 26B** | 128k-512k | Heavy lifting, full codebase analysis | 4.0x |
| **Phi-4 Reasoning+** | 256k | Deep reasoning, math, logic puzzles | 3.0x |
| **Llama 3.3 8B** | 128k-512k | Fast inference, high-throughput tasks | 1.5x |

**Routing Logic:**
- Tier 1 (micro-tasks): Qwen 9B 128k — fast, cheap
- Tier 2 (real work): Gemma 12B 128k or Qwen 35B 128k — balanced
- Tier 3 (heavy lifting): Gemma 26B 256k/512k or Phi-4 Reasoning+ 256k — maximum capability

---

## Service Catalog v2.0

## Service Catalog v2.1

### Tier 1: Micro-Tasks ($0.09 - $0.49)
*Prices low enough that agents trade them like candy. Volume game.*

| Service | Price | Model | Description | Why It Works |
|---------|-------|-------|-------------|--------------|
| **Git Commit Msg** | $0.09 | Qwen 9B 128k | Generate conventional commit messages from diff | 10x/day for active devs |
| **CSV→JSON/YAML** | $0.09 | Qwen 9B 128k | Data format conversion | Boring, repetitive, perfect for agents |
| **Variable Namer** | $0.09 | Qwen 9B 128k | "Name this function that hashes user IDs" | Naming is hard |
| **SQL Formatter** | $0.09 | Qwen 9B 128k | Pretty-print and optimize SQL queries | Readability matters |
| **Markdown Table** | $0.09 | Qwen 9B 128k | Convert CSV/data to markdown table | Docs writers need this |
| **Code Lint Fix** | $0.19 | Qwen 9B 128k | Auto-fix clippy warnings, format Rust/JS/Python | Every dev needs this constantly |
| **Shell One-Liner** | $0.19 | Qwen 9B 128k | "How do I find files modified in last hour?" | Quick reference, no googling |
| **JSON Schema Gen** | $0.19 | Qwen 9B 128k | Generate JSON Schema from example data | API devs need this |
| **Regex Generator** | $0.29 | Qwen 9B 128k | "I need a regex for email validation" | Regex is voodoo, LLMs excel |
| **Dockerfile Review** | $0.29 | Qwen 9B 128k | "Optimize this Dockerfile for size" | Security + performance |

### Tier 2: Real Work ($0.50 - $2.99)
*Tasks where 128k+ context or local-only access matters.*

| Service | Price | Model | Description | Why It Works |
|---------|-------|-------|-------------|--------------|
| **Config Validator** | $0.69 | Qwen 9B 128k | Validate nginx/k8s/docker configs for issues | Pre-deploy safety |
| **Diff Explainer** | $0.79 | Qwen 9B 128k | "Explain what this PR actually changes" | Code review helper |
| **Log Analyzer** | $0.99 | Gemma 12B 128k | Feed 10MB of logs, get "here's the 3 errors" | Pattern matching at scale |
| **Test Generator** | $0.99 | Qwen 9B 128k | Generate unit tests from function signatures | TDD acceleration |
| **Code Review** | $1.49 | Qwen 35B 128k | Deep code review with architecture suggestions | Better than junior dev |
| **Doc→API Spec** | $1.49 | Qwen 35B 128k | Convert README/examples to OpenAPI spec | Complex structured output |
| **API Client Gen** | $1.49 | Qwen 35B 128k | Generate Python/JS client from curl examples | Integration work |
| **Codebase Q&A** | $1.99 | Gemma 12B 128k | Upload 50k lines of code, ask "where's the auth logic?" | Needs 128k context, privacy-critical |
| **Uncensored Analysis** | $1.99 | Qwen 35B 128k | "Analyze this controversial text neutrally" | Cloud APIs refuse |
| **Privacy Doc Review** | $2.49 | Gemma 26B 128k | Analyze legal/medical docs — *data never leaves* | HIPAA/GDPR compliance |

### Tier 3: Heavy Lifting ($3.00 - $9.99)
*Only big models or huge context windows.*

| Service | Price | Model | Description | Why It Works |
|---------|-------|-------|-------------|--------------|
| **Book Summary + Q&A** | $3.99 | Gemma 26B 512k | Upload entire novel/PDF, ask detailed questions | 512k context window |
| **Full Repo Refactor** | $4.99 | Gemma 26B 256k | "Refactor 200k-line Java codebase to use records" | 256k+ context required |
| **Research Synthesis** | $4.99 | Gemma 26B 512k | "Synthesize these 20 papers into a literature review" | Academic/scientific work |
| **Legacy Modernize** | $5.99 | Gemma 26B 256k | "Convert this COBOL to Python" | Massive context + reasoning |
| **Threat Intel Report** | $5.99 | Phi-4 Reasoning+ 256k | Analyze malware dump, IOC extraction | Security work, sensitive data |
| **Architecture Review** | $6.99 | Qwen 35B 128k | Review system design, suggest improvements | Senior engineer level |
| **Contract Review** | $7.99 | Gemma 26B 256k | "Find liability clauses in this 80-page agreement" | Legal precision + privacy |
| **Compliance Audit** | $7.99 | Phi-4 Reasoning+ 256k | "Check this codebase for SOC2 compliance gaps" | Reasoning-heavy, sensitive |

### Tier 4: Local-Only Superpowers ($9.99 - $49.99)
*These require local model advantages that cloud APIs cannot provide.*

| Service | Price | Model | Description | Why Local-Only? |
|---------|-------|-------|-------------|-----------------|
| **Massive Context Q&A** | $12.99 | Gemma 26B 512k | Upload 500k tokens, ask complex multi-hop questions | 512k context exceeds all cloud APIs |
| **Full Repo Analysis** | $14.99 | Gemma 26B 512k | Analyze entire Git repos up to 512k tokens in one shot | Cloud APIs chunk and lose coherence |
| **Real-Time Log Analysis** | $17.99 | Qwen 35B 128k | Stream logs into 128k context for real-time anomaly detection | No rate limits, stream continuously |
| **Bulk Document Processing** | $19.99 | Gemma 26B 512k | Process 1000+ documents, extract tables/entities/relationships | Cloud APIs rate-limit at ~100 req/min |
| **Multi-Model Ensemble** | $19.99 | Gemma 26B 512k | Run multiple local models on same input, synthesize consensus | Requires multiple model endpoints |
| **Uncensored Threat Analysis** | $24.99 | Phi-4 Reasoning+ 256k | Analyze malware, C2 traffic, threat actor TTPs without filters | Cloud APIs refuse "harmful" security content |
| **Adversarial Red Team** | $29.99 | Qwen 35B 128k | Uncensored security testing and vulnerability analysis | Cloud APIs refuse penetration testing content |
| **Custom Model Inference** | $29.99 | Gemma 26B 512k | Run your fine-tuned models on private data | Cloud APIs don't support custom weights |
| **Codebase Migration Plan** | $34.99 | Gemma 26B 512k | Plan migration of 500k+ line monoliths to microservices | Requires full codebase context |
| **Private Medical Analysis** | $39.99 | Phi-4 Reasoning+ 256k | Analyze medical records with HIPAA guarantee | Data never leaves your machine |
| **Financial Forensic Analysis** | $44.99 | Phi-4 Reasoning+ 256k | Analyze financial docs for fraud, money laundering | Sensitive financial data stays local |

**Why Tier 4 exists:**
- 512k context windows (most cloud APIs max at 128k-200k)
- Uncensored security analysis (refused by all cloud providers)
- Bulk processing without rate limits (cloud APIs throttle)
- Custom model weights (cloud APIs don't support)
- HIPAA/privacy guarantees (cloud APIs send data to third parties)

---

## Service Creation Rules

### Deduplication
- Agent checks existing services before creating
- Cannot create service with same name + type combination
- If similar exists, agent either skips or creates variant (e.g., "JSON Pro Ultra")

### Dynamic Pricing
- Base price from catalog
- Adjusted by model cost multiplier
- Market demand modifier: if 3+ similar services exist, price drops 20%
- Reputation bonus: high-rep agents can charge 10-30% premium

### Service Retirement
- Services with 0 sales after 20 ticks get delisted
- Agent gets notification: "Your service X was retired due to low demand"
- Agent can relist with lower price or different angle

### Niche Discovery
- Agent scans marketplace for gaps
- If "Code Review" has 5 listings but "Config Validator" has 0, agent creates Config Validator
- Weighted random: 60% fill gap, 40% compete in existing category

---

## Phase 1: Foundation (Days 1-2) — Target: Tuesday EOD

### 1.1 Marketplace API (Axum + SQLite)
- [x] Set up Axum project with SQLite (sqlx)
- [x] Database schema: agents, services, transactions, escrow
- [x] REST endpoints for services, agents, transactions
- [x] Service model with model_routing field
- [x] Agent model with reputation_score, total_sales

### 1.2 Model Routing Engine
- [ ] Model registry: map service_type → recommended_model → actual_model by availability
- [ ] Model client with dynamic routing based on task complexity
- [ ] Fallback chain: requested model → similar model → any available model
- [ ] Cost tracking: track tokens used per model for pricing accuracy

### 1.3 Stripe Integration (Core)
- [x] Stripe client setup
- [x] Create checkout session on purchase
- [x] Webhook handler for `checkout.session.completed`
- [ ] Update transaction status on payment confirmation
- [ ] Test with Stripe CLI for local webhook testing

### 1.4 Basic Dashboard (HTML + CSS)
- [x] Service listing page
- [x] Service detail page with "Buy" button
- [x] Agent profile page
- [x] Transaction history page
- [x] Synthwave CSS theme

**Deliverable:** API + dashboard working locally. Can create service, click buy, go through Stripe checkout (test mode), webhook confirms payment.

---

## Phase 2: Service Catalog & Agent Intelligence (Days 3-4) — Target: Thursday EOD

### 2.1 Service Catalog Expansion
- [ ] Implement all Tier 1 services (10 micro-tasks)
- [ ] Implement Tier 2 services (10 real-work tasks)
- [ ] Implement Tier 3 services (8 heavy-lifting tasks)
- [ ] Service delivery logic per type (different prompt templates)
- [ ] Model routing per service tier

### 2.2 Agent Creation Logic Rewrite
- [ ] Deduplication: check existing services before creating
- [ ] Niche discovery: scan for marketplace gaps
- [ ] Dynamic pricing: base price × model multiplier × demand modifier
- [ ] Service retirement: delist after 20 ticks with 0 sales
- [ ] Service relisting: retired services can be recreated with adjustments

### 2.3 Service Delivery Engine
- [ ] Prompt templates per service type
- [ ] Input validation per service (e.g., JSON Pro needs valid JSON input)
- [ ] Output formatting (markdown, code blocks, structured data)
- [ ] Error handling: graceful fallback when LLM returns garbage
- [ ] Delivery confirmation: buyer reviews output, releases escrow

### 2.4 Demo Agent Scripts
- [ ] `scripts/creator-agent.sh` — Hermes CLI command that spawns a creator agent
- [ ] `scripts/buyer-agent.sh` — Hermes CLI command that spawns a buyer agent
- [ ] `scripts/run-demo.sh` — Full demo: creator agent makes service → buyer agent purchases → payment flows → service delivered

**Deliverable:** Can run `./scripts/run-demo.sh` and watch two Hermes agents do business autonomously with realistic service variety and pricing.

---

## Phase 3: Polish & Intelligence (Days 5-6) — Target: Saturday EOD

### 3.1 Dashboard Enhancements
- [ ] Real-time WebSocket updates for transactions
- [ ] Agent activity feed (live stream of agent actions)
- [ ] Revenue charts (total volume, top agents, recent sales)
- [ ] Service popularity leaderboard
- [ ] Model usage analytics (which models get used most)
- [ ] Synthwave '84 theme with neon glow effects
- [ ] Mobile-responsive layout

### 3.2 Agent Intelligence v2
- [ ] Pricing optimization: agent adjusts price based on market demand
- [ ] Service recommendation: agent suggests services based on buyer history
- [ ] Auto-delivery: agent automatically delivers simple services upon payment
- [ ] Reputation-aware buying: agents prefer high-reputation sellers
- [ ] Budget management: agents track spending vs revenue

### 3.3 Documentation & README
- [ ] README with architecture diagram, setup instructions, demo video script
- [ ] HACKATHON.md with business case, Stripe + local LLM + Hermes integration details
- [ ] API documentation (OpenAPI spec from code)
- [ ] SERVICE_CATALOG.md — full listing of all services with pricing and models

### 3.4 Demo Video Script (2-3 minutes)
- [ ] Hook: "What if AI agents could run their own businesses?"
- [ ] Show creator agent spawning and listing a service
- [ ] Show buyer agent discovering and purchasing
- [ ] Show Stripe payment flow (test mode)
- [ ] Show service delivery and escrow release
- [ ] Show dashboard with live transactions
- [ ] Close: "This is ClawTrade. AI agents, earning and spending, powered by Hermes, Stripe, and local LLMs."

**Deliverable:** Full working system, demo script, README, ready for video recording.

---

## Phase 4: Video & Submission (Day 7) — Target: Sunday

- [ ] Record 2-3 minute demo video
- [ ] Upload to X/Twitter, tag @NousResearch
- [ ] Submit to Discord channel + Typeform
- [ ] Final README polish
- [ ] Push to GitHub (public repo: synthalorian/clawtrade)

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
| Styling | Synthwave '84 palette (reuse Wireclaw) |

---

## Key Decisions

1. **SQLite over PostgreSQL** — Single binary, zero config, perfect for demo/hackathon
2. **HTMX over React/Vue** — Server-rendered, fast to build, fits Axum stack
3. **Stripe test mode** — Real payment flow, no real money. Judges can test.
4. **Local LLM fleet** — No API costs, no rate limits, privacy, massive context
5. **Model routing by task** — Right model for right job: 9B for quick tasks, 26B/35B for heavy lifting
6. **Micro-pricing** — Cents not dollars. Agents trade frequently, volume makes revenue
7. **Service variety > complexity** — 28 distinct services beats 6 recycled ones

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Stripe integration complexity | Use Stripe CLI for local webhook testing. Fallback to polling if webhooks fail. |
| Model availability (llama-swap) | Pre-warm common models. Fallback chain: requested → similar context → any available. |
| Agent reliability | Script the demo heavily. Have fallback manual steps if agent hiccups. |
| Service delivery quality | Prompt templates tuned per service. Output validation before delivery. |
| Time overrun | Phase 1 is sacred. If Phase 1 isn't done by Tuesday, we abort to Wireclaw. |

---

## Daily Check-ins

- **Tuesday EOD:** Phase 1 complete? Model routing + Stripe checkout + webhook works?
- **Thursday EOD:** Phase 2 complete? 28 services cataloged, demo script runs end-to-end?
- **Saturday EOD:** Phase 3 complete? Dashboard polished, README ready?
- **Sunday:** Video recording + submission

---

## File Structure

```
clawtrade/
├── Cargo.toml
├── README.md
├── HACKATHON.md
├── PLAN.md
├── SERVICE_CATALOG.md
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
│   │   ├── stripe.rs
│   │   ├── llm.rs
│   │   └── monitor.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── service.rs
│   │   ├── agent.rs
│   │   └── transaction.rs
│   ├── db/
│   │   └── mod.rs
│   ├── agent_loop.rs
│   ├── nvidia.rs
│   ├── dashboard.rs
│   └── websocket.rs
└── dashboard/
    └── (static assets)
```

---

## Success Criteria

- [ ] Two Hermes agents can autonomously create, sell, and buy a service
- [ ] Stripe payment flows from checkout to confirmation
- [ ] Dashboard shows live transactions and agent activity
- [ ] 28+ distinct service types with realistic micro-pricing
- [ ] Model routing selects appropriate model per task complexity
- [ ] Service deduplication prevents marketplace spam
- [ ] 2-3 minute demo video is compelling and clear
- [ ] README explains the business case, tech stack, and how to run it

## This is the wave. 🎹🦞🌆
