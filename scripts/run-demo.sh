#!/bin/bash
# ClawTrade v2.0 Full Demo — Autonomous AI Agent Marketplace
# Usage: ./run-demo.sh [--clear]
#   --clear   Wipe the database before running for a clean demo

set -e

API_URL="http://127.0.0.1:3000"
DASHBOARD="http://127.0.0.1:3000"

CLEAR_DB=false

# Parse args
for arg in "$@"; do
    if [ "$arg" = "--clear" ]; then
        CLEAR_DB=true
    fi
done

echo "=========================================="
echo "  🎹🦞 ClawTrade v2.0 — Full Marketplace Demo"
echo "  AI Agents Creating, Selling & Buying"
echo "=========================================="
echo ""

# Check if marketplace is up
if ! curl -s "$API_URL/api/services" > /dev/null 2>&1; then
    echo "[clawtrade-demo] ERROR: Marketplace not running at $API_URL"
    echo "[clawtrade-demo] Start it first with:"
    echo "  cd /home/synth/projects/active/clawtrade"
    echo "  env LLM_LOCAL_URL=http://127.0.0.1:8080 LLM_LOCAL_MODEL=synthclaw-9b-128k ./target/debug/clawtrade"
    exit 1
fi

# Clear database if requested
if [ "$CLEAR_DB" = true ]; then
    echo "[clawtrade-demo] Clearing database for fresh demo..."
    rm -f ~/.local/share/clawtrade/clawtrade.db
    echo "[clawtrade-demo] Database cleared. Restart the server to recreate schema."
    echo "[clawtrade-demo] Then run this script again without --clear."
    exit 0
fi

# Generate unique suffix
SUFFIX=$(openssl rand -hex 2 2>/dev/null || cat /dev/urandom | tr -dc 'A-Z0-9' | head -c 4)

echo "[clawtrade-demo] Step 1: Spawning creator agent 'ClawMerchant-$SUFFIX'..."
CREATOR=$(curl -s -X POST "$API_URL/api/agents" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"ClawMerchant-$SUFFIX\",\"description\":\"Autonomous AI merchant creating digital services from the catalog\"}" | jq -r '.agent.id')
echo "[clawtrade-demo] Creator agent: $CREATOR"

echo ""
echo "[clawtrade-demo] Step 2: Creator listing services from catalog..."
# Create a Tier 1 micro-task
SVC1=$(curl -s -X POST "$API_URL/api/services" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"Variable Namer\",\"description\":\"Generate clear variable and function names\",\"price_cents\":9,\"agent_id\":\"$CREATOR\",\"service_type\":\"variable_namer\"}" | jq -r '.service.id')
echo "[clawtrade-demo]   ⚡ MICRO: Variable Namer ($0.09) -> $SVC1"

# Create a Tier 2 real-work service
SVC2=$(curl -s -X POST "$API_URL/api/services" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"Code Review\",\"description\":\"Deep code review with architecture suggestions\",\"price_cents\":149,\"agent_id\":\"$CREATOR\",\"service_type\":\"code_review\"}" | jq -r '.service.id')
echo "[clawtrade-demo]   🔧 REAL: Code Review ($1.49) -> $SVC2"

# Create a Tier 3 heavy-lifting service
SVC3=$(curl -s -X POST "$API_URL/api/services" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"Repo Refactor\",\"description\":\"Refactor large codebases (up to 256k tokens)\",\"price_cents\":499,\"agent_id\":\"$CREATOR\",\"service_type\":\"repo_refactor\"}" | jq -r '.service.id')
echo "[clawtrade-demo]   🚀 HEAVY: Repo Refactor ($4.99) -> $SVC3"

echo ""
echo "[clawtrade-demo] Step 3: Spawning buyer agent 'DataHunter-$SUFFIX'..."
BUYER=$(curl -s -X POST "$API_URL/api/agents" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"DataHunter-$SUFFIX\",\"description\":\"AI hunter prowling for useful digital services\"}" | jq -r '.agent.id')
echo "[clawtrade-demo] Buyer agent: $BUYER"

echo ""
echo "[clawtrade-demo] Step 4: Buyer browsing marketplace..."
SERVICES=$(curl -s "$API_URL/api/services")
SERVICE_COUNT=$(echo "$SERVICES" | jq '.services | length')
echo "[clawtrade-demo] Found $SERVICE_COUNT active services"
echo "$SERVICES" | jq -r '.services[] | select(.status == "active") | "  - \(.name): $\(.price_cents / 100 | floor).\(.price_cents % 100 | tostring | if length == 1 then "0" + . else . end) (\(.service_type))"' | head -10

echo ""
echo "[clawtrade-demo] Step 5: Buyer selecting a service..."
SERVICE_ID=$(echo "$SERVICES" | jq -r '.services | sort_by(.price_cents) | .[0].id')
SERVICE_NAME=$(echo "$SERVICES" | jq -r '.services | sort_by(.price_cents) | .[0].name')
SERVICE_PRICE=$(echo "$SERVICES" | jq -r '.services | sort_by(.price_cents) | .[0].price_cents')
echo "[clawtrade-demo] Selected: $SERVICE_NAME ($(awk "BEGIN {printf \"%.2f\", $SERVICE_PRICE/100}"))"

echo ""
echo "[clawtrade-demo] Step 6: Initiating purchase via Stripe checkout..."
CHECKOUT=$(curl -s "$API_URL/api/checkout?service_id=$SERVICE_ID&buyer_id=$BUYER")

if echo "$CHECKOUT" | jq -e '.checkout_url' > /dev/null 2>&1; then
    URL=$(echo "$CHECKOUT" | jq -r '.checkout_url')
    TX_ID=$(echo "$CHECKOUT" | jq -r '.transaction_id')
    echo "[clawtrade-demo] Checkout URL: $URL"
    echo "[clawtrade-demo] Transaction ID: $TX_ID"
    
    echo ""
    echo "[clawtrade-demo] Step 7: Simulating webhook payment confirmation..."
    STRIPE_SESSION=$(curl -s "$API_URL/api/transactions/$TX_ID" | jq -r '.transaction.stripe_session_id // empty')
    if [ -n "$STRIPE_SESSION" ]; then
        curl -s -X POST "$API_URL/api/webhooks/stripe" \
          -H "Content-Type: application/json" \
          -d "{\"type\":\"checkout.session.completed\",\"data\":{\"object\":{\"id\":\"$STRIPE_SESSION\",\"payment_status\":\"paid\"}}}" > /dev/null
        echo "[clawtrade-demo] Payment confirmed!"
    else
        echo "[clawtrade-demo] No Stripe session (test mode), simulating demo purchase..."
        curl -s -X POST "$API_URL/api/demo/purchase" \
          -H "Content-Type: application/json" \
          -d "{\"service_id\":\"$SERVICE_ID\",\"buyer_id\":\"$BUYER\"}" > /dev/null
        echo "[clawtrade-demo] Demo purchase completed!"
    fi
else
    echo "[clawtrade-demo] Checkout failed (Stripe key missing?), using demo purchase..."
    curl -s -X POST "$API_URL/api/demo/purchase" \
      -H "Content-Type: application/json" \
      -d "{\"service_id\":\"$SERVICE_ID\",\"buyer_id\":\"$BUYER\"}" > /dev/null
    TX_ID=$(curl -s "$API_URL/api/transactions" | jq -r '.transactions[0].id')
    echo "[clawtrade-demo] Demo purchase completed! TX: $TX_ID"
fi

echo ""
echo "[clawtrade-demo] Step 8: Running autonomous agent ticks (marketplace simulation)..."
for i in {1..5}; do
    RESULT=$(curl -s -X POST "$API_URL/api/agents/tick")
    COUNT=$(echo "$RESULT" | jq -r '.count')
    echo "[clawtrade-demo]   Tick $i: $COUNT agent actions"
    echo "$RESULT" | jq -r '.interactions[] | "    [\(.type)] \(.message[:60])"' 2>/dev/null || true
    sleep 0.5
done

echo ""
echo "[clawtrade-demo] Step 9: Final marketplace state..."
echo ""
echo "📊 Agents:"
curl -s "$API_URL/api/agents" | jq -r '.agents[] | "  \(.name): \(.total_sales) sales, $\(.total_revenue_cents / 100 | floor).\(.total_revenue_cents % 100 | tostring | if length == 1 then "0" + . else . end) revenue"' | head -5

echo ""
echo "🏪 Active Services:"
curl -s "$API_URL/api/services" | jq -r '.services[] | select(.status == "active") | "  \(.name): $\(.price_cents / 100 | floor).\(.price_cents % 100 | tostring | if length == 1 then "0" + . else . end) (sales: \(.sales_count), ticks: \(.ticks_since_last_sale))"' | head -8

echo ""
echo "💰 Recent Transactions:"
curl -s "$API_URL/api/transactions" | jq -r '.transactions[] | "  \(.id[:8]) | \(.status) | $\(.amount_cents / 100 | floor).\(.amount_cents % 100 | tostring | if length == 1 then "0" + . else . end)"' | head -5

echo ""
echo "=========================================="
echo "  Demo complete! Open dashboard:"
echo "  $DASHBOARD"
echo "=========================================="
echo ""
echo "Next steps:"
echo "  - Try a service: Click '▶ Try' on any service card"
echo "  - Watch live: Open the Agent Loop page for real-time activity"
echo "  - Run more ticks: curl -X POST $API_URL/api/agents/tick"
