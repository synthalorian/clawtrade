#!/usr/bin/env bash
# ClawTrade Agent Interaction Demo — Autonomous agent trading
# Usage: ./scripts/agent-interaction-demo.sh
# Requires: curl, jq, running ClawTrade server

set -euo pipefail

API_URL="http://127.0.0.1:3000"
DASH_URL="http://127.0.0.1:8746"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
MAGENTA='\033[0;35m'
NC='\033[0m'

log()  { echo -e "${CYAN}[AGENT-DEMO]${NC} $*"; }
pass() { echo -e "${GREEN}[PASS]${NC} $*"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
fail() { echo -e "${RED}[FAIL]${NC} $*"; exit 1; }
section() { echo -e "\n${MAGENTA}═══════════════════════════════════════════════════════${NC}"; echo -e "${MAGENTA}  $*${NC}"; echo -e "${MAGENTA}═══════════════════════════════════════════════════════${NC}\n"; }

# Check server
if ! curl -s "$API_URL/api/agents" > /dev/null 2>&1; then
    fail "ClawTrade server not running. Start it first:\n  cd ~/projects/active/clawtrade && cargo run --release"
fi

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  🎹🦞 ClawTrade Agent Interaction Demo${NC}"
echo -e "${CYAN}  Autonomous agents buying, selling, and trading${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════${NC}"
echo ""

# ─── SECTION 1: Create Multiple Agents ───
section "1️⃣  CREATING AGENT ECOSYSTEM"

log "Creating seller agents..."
for i in 1 2 3; do
    SELLER_RESP=$(curl -s -X POST "$API_URL/api/agents" \
        -H "Content-Type: application/json" \
        -d "{\"name\":\"Seller Bot $i\",\"description\":\"Autonomous seller specializing in AI services.\"}")
    SELLER_ID=$(echo "$SELLER_RESP" | jq -r '.agent.id')
    pass "Seller Bot $i: ${SELLER_ID:0:8}..."
done

log "Creating buyer agents..."
for i in 1 2 3 4; do
    BUYER_RESP=$(curl -s -X POST "$API_URL/api/agents" \
        -H "Content-Type: application/json" \
        -d "{\"name\":\"Buyer Bot $i\",\"description\":\"Autonomous buyer scouting for services.\"}")
    BUYER_ID=$(echo "$BUYER_RESP" | jq -r '.agent.id')
    pass "Buyer Bot $i: ${BUYER_ID:0:8}..."
done

# ─── SECTION 2: Create Services ───
section "2️⃣  LISTING SERVICES"

# Get first seller
SELLERS=$(curl -s "$API_URL/api/agents")
SELLER1=$(echo "$SELLERS" | jq -r '.agents[0].id')
SELLER2=$(echo "$SELLERS" | jq -r '.agents[1].id')
SELLER3=$(echo "$SELLERS" | jq -r '.agents[2].id')

log "Creating services..."

SERVICE1=$(curl -s -X POST "$API_URL/api/services" \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"Smart Summarizer\",\"description\":\"Summarize any text into key bullet points.\",\"price_cents\":499,\"agent_id\":\"$SELLER1\",\"service_type\":\"text_processing\"}")
SID1=$(echo "$SERVICE1" | jq -r '.service.id')
pass "Smart Summarizer: ${SID1:0:8}... (\$4.99)"

SERVICE2=$(curl -s -X POST "$API_URL/api/services" \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"JSON Pro\",\"description\":\"Format and validate JSON data.\",\"price_cents\":299,\"agent_id\":\"$SELLER2\",\"service_type\":\"data_formatting\"}")
SID2=$(echo "$SERVICE2" | jq -r '.service.id')
pass "JSON Pro: ${SID2:0:8}... (\$2.99)"

SERVICE3=$(curl -s -X POST "$API_URL/api/services" \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"Endpoint Monitor\",\"description\":\"Live API health checks with latency.\",\"price_cents\":599,\"agent_id\":\"$SELLER3\",\"service_type\":\"api_monitor\"}")
SID3=$(echo "$SERVICE3" | jq -r '.service.id')
pass "Endpoint Monitor: ${SID3:0:8}... (\$5.99)"

SERVICE4=$(curl -s -X POST "$API_URL/api/services" \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"Code Reviewer\",\"description\":\"AI-powered code review with actionable feedback.\",\"price_cents\":999,\"agent_id\":\"$SELLER1\",\"service_type\":\"code_review\"}")
SID4=$(echo "$SERVICE4" | jq -r '.service.id')
pass "Code Reviewer: ${SID4:0:8}... (\$9.99)"

# ─── SECTION 3: Run Agent Tick (Autonomous Trading) ───
section "3️⃣  AUTONOMOUS AGENT TRADING"

log "Running agent tick cycle 1..."
TICK1=$(curl -s -X POST "$API_URL/api/agents/tick")
COUNT1=$(echo "$TICK1" | jq -r '.count // 0')
pass "Tick 1: $COUNT1 interactions"

log "Running agent tick cycle 2..."
TICK2=$(curl -s -X POST "$API_URL/api/agents/tick")
COUNT2=$(echo "$TICK2" | jq -r '.count // 0')
pass "Tick 2: $COUNT2 interactions"

log "Running agent tick cycle 3..."
TICK3=$(curl -s -X POST "$API_URL/api/agents/tick")
COUNT3=$(echo "$TICK3" | jq -r '.count // 0')
pass "Tick 3: $COUNT3 interactions"

# ─── SECTION 4: Show Results ───
section "4️⃣  MARKETPLACE STATE"

log "Checking agent states..."
STATES=$(curl -s "$API_URL/api/agents/states")
AGENT_COUNT=$(echo "$STATES" | jq -r '.count // 0')
pass "$AGENT_COUNT agents active"

log "Checking transactions..."
TXS=$(curl -s "$API_URL/api/transactions")
TX_COUNT=$(echo "$TXS" | jq -r '.transactions | length // 0')
pass "$TX_COUNT transactions created by agents"

# ─── SECTION 5: Dashboard Check ───
section "5️⃣  DASHBOARD VERIFICATION"

log "Checking monitor page..."
MONITOR_HTML=$(curl -s "$DASH_URL/monitor")
[[ "$MONITOR_HTML" == *"Service Monitor"* ]] && pass "Monitor page live" || warn "Monitor page issue"

log "Checking agent loop page..."
LOOP_HTML=$(curl -s "$DASH_URL/agent-loop")
[[ "$LOOP_HTML" == *"Agent Loop"* ]] && pass "Agent Loop page live" || warn "Agent Loop page issue"

# ─── SUMMARY ───
echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  🎹🦞 AGENT INTERACTION DEMO COMPLETE ✅${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════${NC}"
echo ""
echo "What was demonstrated:"
echo "  1. ✅ Created 7 autonomous agents (3 sellers, 4 buyers)"
echo "  2. ✅ Listed 4 diverse services"
echo "  3. ✅ Ran 3 autonomous trading ticks"
echo "  4. ✅ Agents made purchase decisions based on availability"
echo "  5. ✅ Transactions recorded in marketplace"
echo "  6. ✅ Monitor page shows service examples"
echo "  7. ✅ Agent Loop page shows live trading UI"
echo ""
echo "Key URLs:"
echo "  Dashboard:     $DASH_URL"
echo "  Monitor:       $DASH_URL/monitor"
echo "  Agent Loop:    $DASH_URL/agent-loop"
echo "  API:           $API_URL"
echo ""
echo "This is the wave. 🎹🦞🌆"
echo ""
