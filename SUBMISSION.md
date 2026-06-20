# ClawTrade — Submission Checklist

## Hermes Agent Accelerated Business Hackathon
**Deadline:** Tuesday, June 30, 2026

---

## ✅ What's Built

- [x] **Marketplace API** (Axum + SQLite)
  - [x] Agents CRUD
  - [x] Services CRUD
  - [x] Transactions CRUD
  - [x] Stripe checkout integration
  - [x] Webhook handler for payment confirmation
- [x] **Dashboard** (Synthwave '84 theme)
  - [x] Live stats (services, agents, paid, volume)
  - [x] Featured services grid
  - [x] Agent profiles with tier badges
  - [x] Live activity feed
  - [x] Responsive layout
  - [x] Neon glow effects
- [x] **Hermes Skill** (`~/.hermes/skills/clawtrade/`)
  - [x] create_service
  - [x] list_services
  - [x] purchase_service
  - [x] check_transaction
  - [x] create_agent
- [x] **Demo Scripts**
  - [x] `scripts/run-demo.sh` — full marketplace demo
  - [x] `scripts/creator-agent.sh` — merchant agent
  - [x] `scripts/buyer-agent.sh` — buyer agent
  - [x] `scripts/demo-video-setup.sh` — video data prep
  - [x] `scripts/record-screen.sh` — screen recording helper
- [x] **Documentation**
  - [x] README.md with architecture, quick start, API docs
  - [x] HACKATHON.md with business case, integration details
  - [x] PLAN.md with 4-phase roadmap
- [x] **GitHub Repo**
  - [x] https://github.com/synthalorian/clawtrade
  - [x] Public, with README and LICENSE

---

## 🎬 Video Recording

### Script (2-3 minutes)

**[0:00-0:15] Hook**
> "What if AI agents could run their own businesses? Not just chatbots — actual merchants, buyers, entrepreneurs. That's ClawTrade."

**[0:15-0:45] Show Dashboard**
- Open http://127.0.0.1:8746
- Pan across: stats, services, activity feed, agents
- "This is the marketplace. Every service here was created by an AI agent."

**[0:45-1:15] Creator Agent**
- Run `./scripts/creator-agent.sh` (or show terminal)
- "SynthMerchant just spawned. It decided to create three services: a text summarizer, a JSON beautifier, and an API monitor. It priced them competitively."
- Show services page with the new listings

**[1:15-1:45] Buyer Agent + Stripe**
- Run `./scripts/buyer-agent.sh` (or show terminal)
- "DataHunter is browsing. It found the JSON Beautifier for $2.99. It clicks Buy."
- Show Stripe checkout (test mode)
- "Stripe handles the payment. This is real payment infrastructure, just in test mode."

**[1:45-2:15] Payment Confirmation + Delivery**
- Show webhook confirmation
- "Payment confirmed. The transaction is marked paid. The seller's stats update automatically."
- Show transaction history, updated agent stats

**[2:15-2:30] Close**
> "This is ClawTrade. AI agents, earning and spending, powered by Hermes, Stripe, and local LLMs. The future of work isn't humans vs AI — it's agents doing business. This is the wave."

### Recording Commands

```bash
# 1. Start marketplace
cd /home/synth/projects/active/clawtrade
STRIPE_SECRET_KEY=*** cargo run --release

# 2. In another terminal, prep demo data
./scripts/demo-video-setup.sh

# 3. Open browser to http://127.0.0.1:8746
# 4. Record screen
./scripts/record-screen.sh
# Or use OBS: 1920x1080, 30fps, output to MP4
```

---

## 📤 Submission Steps

1. [ ] Record 2-3 minute demo video
2. [ ] Upload to X/Twitter
3. [ ] Tag @NousResearch
4. [ ] Submit to Discord channel (Hermes hackathon)
5. [ ] Submit to Typeform (link in Discord)
6. [ ] Final README polish
7. [ ] Push any last changes to GitHub

---

## 🔧 Tech Stack Summary

| Layer | Technology |
|-------|------------|
| Backend | Rust 1.85+, Axum, sqlx, SQLite |
| Frontend | Server-rendered HTML + HTMX + CSS |
| Payments | Stripe API (test mode) |
| Agents | Hermes CLI + custom skills |
| LLM | Local llama-swap (RX 9070 XT) |
| Theme | Synthwave '84 |

---

## 🚨 Abort Criteria (HARD)

- **Deadline:** Sunday, June 29, EOD
- **Trigger:** If Stripe checkout → webhook → payment confirmation is not working
- **Fallback:** Pivot to Wireclaw submission
- **Wireclaw demo:** Record Monday, submit Tuesday June 30

**Current Status:** ✅ Stripe flow WORKS. No abort needed.

---

## This is the wave. 🎹🦞🌆
