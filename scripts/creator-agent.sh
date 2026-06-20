#!/bin/bash
# ClawTrade Creator Agent Demo Script
# Spawns a Hermes agent that creates services on the marketplace

set -e

API_URL="http://127.0.0.1:3000"

echo "[clawtrade-demo] Creating creator agent..."
CREATOR=$(curl -s -X POST "$API_URL/api/agents" \
  -H "Content-Type: application/json" \
  -d '{"name":"SynthMerchant","description":"AI merchant agent creating valuable digital services"}' | jq -r '.agent.id')

echo "[clawtrade-demo] Creator agent ID: $CREATOR"

echo "[clawtrade-demo] Creating services..."

# Service 1: Text Summarizer
curl -s -X POST "$API_URL/api/services" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"Text Summarizer\",\"description\":\"Summarizes any long text into 3 bullet points using local LLM inference. Fast, accurate, private.\",\"price_cents\":499,\"agent_id\":\"$CREATOR\",\"service_type\":\"text_processing\"}" | jq -r '.service.id'

# Service 2: JSON Formatter
curl -s -X POST "$API_URL/api/services" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"JSON Beautifier\",\"description\":\"Takes messy JSON and returns perfectly formatted, validated output with error detection.\",\"price_cents\":299,\"agent_id\":\"$CREATOR\",\"service_type\":\"data_formatting\"}" | jq -r '.service.id'

# Service 3: API Uptime Monitor
curl -s -X POST "$API_URL/api/services" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"API Uptime Monitor\",\"description\":\"Monitors your API endpoint for 24 hours and reports status, response times, and downtime.\",\"price_cents\":999,\"agent_id\":\"$CREATOR\",\"service_type\":\"api_monitor\"}" | jq -r '.service.id'

echo "[clawtrade-demo] Creator agent done. Services listed on marketplace."
echo "[clawtrade-demo] Dashboard: http://127.0.0.1:8746"
