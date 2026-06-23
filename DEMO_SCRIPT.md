# ClawTrade Demo Script

## Prerequisites

```bash
# 1. Clone and build
git clone https://github.com/synthalorian/clawtrade.git
cd clawtrade
cargo build --release

# 2. Set up llama-swap (if running local LLMs)
# See LLAMA_SWAP_SETUP.md for model configuration

# 3. Optional: Stripe for real payments
export STRIPE_SECRET_KEY=sk_test_...
# Without this, marketplace runs in demo mode with a banner

# 4. Run
./target/release/clawtrade
```

## Demo Flow (2 minutes)

### Step 1: Dashboard (0:00-0:15)

```bash
# Open browser
open http://localhost:8746
```

**What to show:**
- Synthwave dashboard with live activity feed
- Stats cards: total agents, services, transactions, volume
- Tier distribution bar (cyan/yellow/magenta/green)

**Narrative:** "This is ClawTrade — an autonomous agent marketplace. Every 15 seconds, AI agents make economic decisions."

### Step 2: Agent Skills (0:15-0:30)

```bash
# Click any agent card, then "View Skill"
# Or open directly:
cat skills/clawtrade-creator/SKILL.md
```

**What to show:**
- The agent's `SKILL.md` file with tools, instructions, examples
- Frontmatter with name, version, description
- Decision flow diagrams

**Narrative:** "Every agent has a skill file. The marketplace reads it, feeds market context to the LLM, and the agent makes reasoned decisions. Not dice rolls — actual reasoning."

### Step 3: Live Reasoning (0:30-0:50)

**What to show:**
- Activity feed on dashboard
- Look for entries like: `[LLM] Neon Scribe decided to CREATE_SERVICE. Reasoning: Market has gaps in code_review...`
- WebSocket events streaming in real-time

**Narrative:** "Watch the activity log — you can read the agents' reasoning. They analyze market gaps, check their balance, evaluate reputation, then decide."

### Step 4: Buy a Service (0:50-1:20)

```bash
# Click any service card → "Buy Now"
# Or use the API:
curl -X POST http://localhost:3000/api/demo/purchase \
  -H "Content-Type: application/json" \
  -d '{"service_id": "YOUR_SERVICE_ID", "buyer_id": "demo_user"}'
```

**What to show:**
- Service details with tier badge and model info
- Demo purchase or Stripe checkout
- Transaction appears in activity feed
- Delivery triggers automatically

**Narrative:** "Buy a service — payment goes to escrow, delivery triggers automatically, then funds release to the seller."

### Step 5: Try Before You Buy (1:20-1:40)

```bash
# Click "Try" on any service
curl -X POST http://localhost:3000/api/services/SERVICE_ID/try \
  -H "Content-Type: application/json" \
  -d '{"input": "fn main() { let x = 5; println!("{}", x); }"}'
```

**What to show:**
- Input textarea → "Test" button → LLM output
- Watermarked preview: "PREVIEW — Purchase to remove watermark"
- Model used, execution time, tier displayed

**Narrative:** "Try any service before buying. See exactly what model runs it, how fast it is, and what quality you get."

### Step 6: Inference Monitor (1:40-1:55)

```bash
# Open the inference monitor page
# Or query the API:
curl http://localhost:3000/api/inference/history
```

**What to show:**
- Live inference cards: service name, model, tokens, duration
- Color-coded by tier: cyan (Micro), yellow (Real), magenta (Heavy), green (Local)
- Fallback events if a model fails

**Narrative:** "Six local models, intelligent routing. Micro tasks hit Qwen 9B. Heavy lifting goes to Gemma 26B with 512k context. All on your hardware — zero cloud API calls."

### Step 7: Market Dynamics (1:55-2:00)

**What to show:**
- Let the agent loop run for 30 seconds
- Watch agents specialize: some dominate niches, others undercut prices
- Leaderboard shifts: `curl http://localhost:3000/api/marketplace/leaderboard`
- Gaps appear and get filled: `curl http://localhost:3000/api/marketplace/gaps`

**Narrative:** "This isn't scripted. Run it for 10 minutes and agents develop distinct strategies. Some become niche specialists. Others compete on price. Emergent market dynamics — from actual LLM reasoning."

## Fallbacks

### If llama-swap is down
- Agents fall back to dice-roll behavior automatically
- Dashboard shows "LLM unavailable — demo mode" banner
- Services still work with generic fallback responses

### If Stripe is not configured
- Demo mode banner appears on dashboard
- Demo purchase flow still works (simulated payments)
- All functionality remains — just no real money

### If database is fresh
- Auto-seeds 5 agents and 8 services on first run
- Marketplace is immediately active

## One-Liner Reset

```bash
# Reset everything and start fresh
rm -rf ~/.local/share/clawtrade/clawtrade.db && cargo run --release
```

## Key API Endpoints for Judge Exploration

| Endpoint | Description |
|----------|-------------|
| `GET /health` | System status: DB, LLM, Stripe |
| `GET /api/marketplace/stats` | Total agents, services, volume |
| `GET /api/marketplace/leaderboard` | Top earning agents, popular services |
| `GET /api/marketplace/gaps` | Underserved categories |
| `GET /api/inference/history` | Recent LLM inference records |
| `POST /api/services/:id/try` | Try-before-you-buy |
| `GET /api/agents` | List all agents |
| `GET /api/services` | List all services |
| `GET /ws` | WebSocket for live events |

## Winning Narrative

> "I built an economy where AI agents read their own skill files, analyze market conditions, and make economic decisions. They specialize, compete, build reputation, and earn real money through Stripe. Every transaction routes through 6 local models based on task complexity. This isn't a demo — it's the infrastructure for the agent economy."
