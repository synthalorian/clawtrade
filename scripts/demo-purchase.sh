#!/usr/bin/env bash
# ClawTrade Full Demo — Buy, Sell, Deliver, Review
# Works WITHOUT Stripe key. Uses local LLM for service delivery.
# Usage: ./scripts/demo-purchase.sh

set -euo pipefail

API_URL="http://127.0.0.1:3000"
DASH_URL="http://127.0.0.1:8746"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'

log()  { echo -e "${CYAN}[DEMO]${NC} $*"; }
pass() { echo -e "${GREEN}[PASS]${NC} $*"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
fail() { echo -e "${RED}[FAIL]${NC} $*"; exit 1; }

# Check server
if ! curl -s "$API_URL/api/agents" > /dev/null 2>&1; then
    fail "ClawTrade server not running. Start it first:\n  cd ~/projects/active/clawtrade && cargo run --release"
fi

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  🎹🦞 ClawTrade Full Demo — No Stripe Required${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════${NC}"
echo ""

# ─── STEP 1: Create Seller Agent ───
log "Step 1: Creating seller agent..."
SELLER_RESP=$(curl -s -X POST "$API_URL/api/agents" \
    -H "Content-Type: application/json" \
    -d '{"name":"Neon Scribe","description":"AI wordsmith specializing in text transformation and summarization."}')
SELLER_ID=$(echo "$SELLER_RESP" | jq -r '.agent.id')
[ "$SELLER_ID" != "null" ] && [ -n "$SELLER_ID" ] || fail "Seller creation failed: $SELLER_RESP"
pass "Seller agent: ${SELLER_ID:0:8}..."

# ─── STEP 2: Create Buyer Agent ───
log "Step 2: Creating buyer agent..."
BUYER_RESP=$(curl -s -X POST "$API_URL/api/agents" \
    -H "Content-Type: application/json" \
    -d '{"name":"Grid Runner","description":"Autonomous buyer agent scouting the marketplace."}')
BUYER_ID=$(echo "$BUYER_RESP" | jq -r '.agent.id')
[ "$BUYER_ID" != "null" ] && [ -n "$BUYER_ID" ] || fail "Buyer creation failed: $BUYER_RESP"
pass "Buyer agent: ${BUYER_ID:0:8}..."

# ─── STEP 3: Create Service ───
log "Step 3: Creating service..."
SERVICE_RESP=$(curl -s -X POST "$API_URL/api/services" \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"Text Summarizer Pro\",\"description\":\"Condense any document into 3 bullet points using local LLM inference.\",\"price_cents\":499,\"agent_id\":\"$SELLER_ID\",\"service_type\":\"text_processing\"}")
SERVICE_ID=$(echo "$SERVICE_RESP" | jq -r '.service.id')
[ "$SERVICE_ID" != "null" ] && [ -n "$SERVICE_ID" ] || fail "Service creation failed: $SERVICE_RESP"
pass "Service created: ${SERVICE_ID:0:8}... (\$4.99)"

# ─── STEP 4: DEMO PURCHASE (no Stripe) ───
log "Step 4: Demo purchase (no Stripe required)..."
PURCHASE_RESP=$(curl -s -X POST "$API_URL/api/demo/purchase" \
    -H "Content-Type: application/json" \
    -d "{\"service_id\":\"$SERVICE_ID\",\"buyer_id\":\"$BUYER_ID\"}")
TX_ID=$(echo "$PURCHASE_RESP" | jq -r '.transaction_id')
PURCHASE_STATUS=$(echo "$PURCHASE_RESP" | jq -r '.status')
PURCHASE_MSG=$(echo "$PURCHASE_RESP" | jq -r '.message')
DELIVERABLE_URL=$(echo "$PURCHASE_RESP" | jq -r '.deliverable_url')
[ "$TX_ID" != "null" ] && [ -n "$TX_ID" ] || fail "Demo purchase failed: $PURCHASE_RESP"
pass "Purchase complete! TX: ${TX_ID:0:8}... | Status: $PURCHASE_STATUS"
log "  → $PURCHASE_MSG"

# ─── STEP 5: Verify Transaction in Escrow ───
log "Step 5: Verifying transaction status..."
TX_CHECK=$(curl -s "$API_URL/api/transactions/$TX_ID")
TX_STATUS=$(echo "$TX_CHECK" | jq -r '.transaction.status')
[ "$TX_STATUS" = "escrow" ] || fail "Expected escrow, got $TX_STATUS"
pass "Transaction confirmed in escrow"

# ─── STEP 6: Check Deliverable ───
log "Step 6: Checking deliverable..."
sleep 2  # Give delivery a moment
DELIVERABLE=$(curl -s "$DELIVERABLE_URL")
DEL_STATUS=$(echo "$DELIVERABLE" | jq -r '.deliverable.status // empty')
if [ -n "$DEL_STATUS" ]; then
    pass "Deliverable status: $DEL_STATUS"
    OUTPUT=$(echo "$DELIVERABLE" | jq -r '.deliverable.output_data // empty')
    if [ -n "$OUTPUT" ]; then
        log "Deliverable output preview:"
        echo "$OUTPUT" | head -8 | sed 's/^/    /'
    fi
else
    warn "Deliverable may still be processing (check dashboard)"
fi

# ─── STEP 7: Release Escrow ───
log "Step 7: Releasing escrow to seller..."
RELEASE_RESP=$(curl -s -X POST "$API_URL/api/transactions/$TX_ID/release")
RELEASE_STATUS=$(echo "$RELEASE_RESP" | jq -r '.status // .error')
[ "$RELEASE_STATUS" = "released" ] || fail "Escrow release failed: $RELEASE_RESP"
pass "Escrow released — seller paid!"

# ─── STEP 8: Verify Released Status ───
log "Step 8: Confirming released status..."
TX_CHECK=$(curl -s "$API_URL/api/transactions/$TX_ID")
TX_STATUS=$(echo "$TX_CHECK" | jq -r '.transaction.status')
[ "$TX_STATUS" = "released" ] || fail "Expected released, got $TX_STATUS"
pass "Transaction fully completed"

# ─── STEP 9: Submit Review ───
log "Step 9: Submitting 5-star review..."
REVIEW_RESP=$(curl -s -X POST "$API_URL/api/reviews" \
    -H "Content-Type: application/json" \
    -d "{\"transaction_id\":\"$TX_ID\",\"agent_id\":\"$SELLER_ID\",\"rating\":5,\"comment\":\"Absolutely stellar! The summarization was spot-on and delivery was instant. This is the wave. 🎹🦞\"}")
REVIEW_ID=$(echo "$REVIEW_RESP" | jq -r '.review.id // empty')
[ -n "$REVIEW_ID" ] || warn "Review response: $(echo "$REVIEW_RESP" | jq -c .)"
pass "Review submitted"

# ─── STEP 10: Check Seller Stats ───
log "Step 10: Checking seller reputation..."
SELLER_CHECK=$(curl -s "$API_URL/api/agents/$SELLER_ID")
SALES=$(echo "$SELLER_CHECK" | jq -r '.agent.total_sales')
REVENUE=$(echo "$SELLER_CHECK" | jq -r '.agent.total_revenue_cents')
pass "Seller now has $SALES sale(s) and \$$(echo "scale=2; $REVENUE/100" | bc) in revenue"

# ─── STEP 11: Dashboard Check ───
log "Step 11: Checking dashboard..."
DASH_HTML=$(curl -s "$DASH_URL/")
[[ "$DASH_HTML" == *"ClawTrade"* ]] || fail "Dashboard not responding"
pass "Dashboard live at $DASH_URL"

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  🎹🦞 FULL DEMO COMPLETE ✅${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════${NC}"
echo ""
echo "Flow Summary:"
echo "  1. Seller created   → ${SELLER_ID:0:8}..."
echo "  2. Buyer created    → ${BUYER_ID:0:8}..."
echo "  3. Service listed   → ${SERVICE_ID:0:8}... (\$4.99)"
echo "  4. Demo purchase    → TX ${TX_ID:0:8}..."
echo "  5. Payment simulated → escrow"
echo "  6. Delivery triggered → local LLM"
echo "  7. Escrow released  → seller paid"
echo "  8. Review submitted → 5 stars"
echo ""
echo "Open $DASH_URL to see live activity!"
echo ""
echo "View your deliverable:"
echo "  http://127.0.0.1:8746/deliverable/$TX_ID"
echo ""
