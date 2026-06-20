#!/bin/bash
# ClawTrade Buyer Agent Demo Script
# Spawns a Hermes agent that browses and purchases services

set -e

API_URL="http://127.0.0.1:3000"

echo "[clawtrade-demo] Creating buyer agent..."
BUYER=$(curl -s -X POST "$API_URL/api/agents" \
  -H "Content-Type: application/json" \
  -d '{"name":"DataHunter","description":"AI buyer agent seeking useful digital services"}' | jq -r '.agent.id')

echo "[clawtrade-demo] Buyer agent ID: $BUYER"

echo "[clawtrade-demo] Browsing services..."
SERVICES=$(curl -s "$API_URL/api/services")
echo "$SERVICES" | jq '.services[] | {name, price_cents, id}'

# Pick the cheapest service
SERVICE_ID=$(echo "$SERVICES" | jq -r '.services | sort_by(.price_cents) | .[0].id')
SERVICE_NAME=$(echo "$SERVICES" | jq -r '.services | sort_by(.price_cents) | .[0].name')
PRICE=$(echo "$SERVICES" | jq -r '.services | sort_by(.price_cents) | .[0].price_cents')

echo "[clawtrade-demo] Selected: $SERVICE_NAME (\$$PRICE)"
echo "[clawtrade-demo] Initiating purchase..."

CHECKOUT=$(curl -s "$API_URL/api/checkout?service_id=$SERVICE_ID\&buyer_id=$BUYER")
echo "$CHECKOUT" | jq .

if echo "$CHECKOUT" | jq -e '.checkout_url' > /dev/null 2>&1; then
    URL=$(echo "$CHECKOUT" | jq -r '.checkout_url')
    TX_ID=$(echo "$CHECKOUT" | jq -r '.transaction_id')
    echo "[clawtrade-demo] Purchase initiated!"
    echo "[clawtrade-demo] Checkout URL: $URL"
    echo "[clawtrade-demo] Transaction ID: $TX_ID"
    echo "[clawtrade-demo] Open the checkout URL to complete payment (Stripe test mode)"
else
    echo "[clawtrade-demo] Purchase failed:"
    echo "$CHECKOUT" | jq .
fi
