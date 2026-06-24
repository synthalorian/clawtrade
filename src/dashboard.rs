use axum::{
    Router,
    extract::{Path, Query, State},
    response::Html,
    routing::get,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::models::agent::Agent;
use crate::models::deliverable::Deliverable;
use crate::models::service::Service;
use crate::models::transaction::Transaction;
use crate::AppState;

#[derive(Deserialize)]
pub struct TxQuery {
    pub tx_id: Option<String>,
}

pub fn dashboard_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/services", get(services_page))
        .route("/agents", get(agents_page))
        .route("/transactions", get(transactions_page))
        .route("/my-purchases", get(my_purchases_page))
        .route("/deliverable/{id}", get(deliverable_page))
        .route("/success", get(success_page))
        .route("/cancel", get(cancel_page))
        .route("/monitor", get(monitor_page))
        .route("/agent-loop", get(agent_loop_page))
        .route("/activity", get(activity_page))
        .route("/inference", get(inference_monitor_page))
        // NOTE: /api/inference/history is defined in api::routes() — don't duplicate
        .with_state(state)
}

/* ─────────── GLOBAL CSS / SHARED STYLES ─────────── */

const GLOBAL_CSS: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;700&family=Inter:wght@400;500;600;700;800&display=swap');

:root {
  --bg: #0a0a0f;
  --surface: #0d0d12;
  --surface-2: #111118;
  --surface-3: #16161f;
  --border: #1e1e2e;
  --border-hover: #00f0ff;
  --text: rgba(255,255,255,0.65);
  --text-dim: rgba(255,255,255,0.35);
  --text-bright: rgba(255,255,255,0.85);
  --muted: rgba(255,255,255,0.35);
  --accent: #00f0ff;
  --accent-2: #ff006e;
  --accent-3: #ffbe0b;
  --success: #00ff88;
  --err: #ff006e;
  --font: 'Inter', system-ui, sans-serif;
  --mono: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', Consolas, monospace;
  --radius: 10px;
  --radius-sm: 6px;
  --shadow: 0 4px 16px rgba(0,0,0,0.5);
  --shadow-glow: 0 0 20px rgba(0,240,255,0.08);
  --transition: all 0.15s ease;
}

* { margin: 0; padding: 0; box-sizing: border-box; }

html { scroll-behavior: smooth; }

body {
  background: var(--bg);
  color: var(--text);
  font-family: var(--font);
  line-height: 1.5;
  min-height: 100vh;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  position: relative;
  overflow-x: hidden;
}

/* ── CRT Scanline Overlay ── */
body::after {
  content: '';
  position: fixed;
  top: 0; left: 0; right: 0; bottom: 0;
  background: repeating-linear-gradient(
    0deg,
    transparent,
    transparent 2px,
    rgba(0,0,0,0.03) 2px,
    rgba(0,0,0,0.03) 4px
  );
  pointer-events: none;
  z-index: 9998;
}

/* ── Ambient Corner Glows ── */
body::before {
  content: '';
  position: fixed;
  top: 0; left: 0; right: 0; bottom: 0;
  background:
    radial-gradient(circle at 95% 5%, rgba(0,240,255,0.04) 0%, transparent 35%),
    radial-gradient(circle at 5% 95%, rgba(255,0,110,0.04) 0%, transparent 35%);
  pointer-events: none;
  z-index: 9997;
}

::selection { background: rgba(0,240,255,0.25); color: #fff; }

::-webkit-scrollbar { width: 6px; height: 6px; }
::-webkit-scrollbar-track { background: var(--bg); }
::-webkit-scrollbar-thumb { background: var(--border); border-radius: 3px; }
::-webkit-scrollbar-thumb:hover { background: rgba(0,240,255,0.3); }

:focus-visible { outline: 1px solid var(--accent); outline-offset: 2px; }

/* ── Page Transitions ── */
.container { animation: fadeIn 0.15s ease; }
@keyframes fadeIn { from { opacity: 0; transform: translateY(4px); } to { opacity: 1; transform: translateY(0); } }

/* ── Header ── */
header {
  background: rgba(13,13,18,0.85);
  border-bottom: 1px solid var(--border);
  padding: 0.75rem 2rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
  position: sticky;
  top: 0;
  z-index: 100;
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
}

header h1 {
  font-size: 1.4rem;
  font-weight: 800;
  letter-spacing: -0.02em;
  background: linear-gradient(90deg, var(--accent), var(--accent-2));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  filter: drop-shadow(0 0 6px rgba(0,240,255,0.2));
  cursor: pointer;
}
header h1 a { text-decoration: none; color: inherit; }

/* ── Live Indicator ── */
.live-indicator {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-dim);
  cursor: default;
  position: relative;
}
.live-indicator .dot {
  width: 7px; height: 7px;
  background: #ff3333;
  border-radius: 50%;
  box-shadow: 0 0 8px rgba(255,51,51,0.6);
  animation: pulse-dot 2s ease-in-out infinite;
}
@keyframes pulse-dot {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.5; transform: scale(0.85); }
}
.live-indicator:hover::after {
  content: attr(data-tooltip);
  position: absolute;
  top: calc(100% + 6px);
  right: 0;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 0.4rem 0.7rem;
  font-size: 0.75rem;
  color: var(--text);
  white-space: nowrap;
  z-index: 200;
}

/* ── Nav ── */
nav { display: flex; align-items: center; gap: 0.15rem; }
nav a {
  color: var(--text-dim);
  text-decoration: none;
  padding: 0.4rem 0.85rem;
  font-weight: 500;
  font-size: 0.85rem;
  border-radius: var(--radius-sm);
  transition: var(--transition);
  position: relative;
}
nav a:hover { color: var(--accent); background: rgba(0,240,255,0.06); }
nav a.active { color: var(--accent); }
nav a.active::after {
  content: '';
  position: absolute;
  bottom: 2px; left: 0.5rem; right: 0.5rem;
  height: 2px;
  background: var(--accent);
  border-radius: 1px;
  box-shadow: 0 0 6px rgba(0,240,255,0.4);
}

#nav-toggle { display: none; background: none; border: none; color: var(--accent); font-size: 1.3rem; cursor: pointer; }

/* ── Hero ── */
.hero {
  text-align: center;
  padding: 3.5rem 2rem 2.5rem;
  position: relative;
  overflow: hidden;
  border-bottom: 1px solid var(--border);
}
.hero h2 {
  font-size: 3.5rem;
  font-weight: 800;
  letter-spacing: -0.03em;
  line-height: 1.1;
  margin-bottom: 0.75rem;
  background: linear-gradient(135deg, var(--accent) 0%, var(--accent-2) 50%, #fff 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  filter: drop-shadow(0 0 20px rgba(0,240,255,0.15));
  position: relative;
}
.hero p {
  color: rgba(255,255,255,0.5);
  font-size: 0.95rem;
  max-width: 520px;
  margin: 0 auto;
  position: relative;
  line-height: 1.6;
}

/* ── Container ── */
.container { padding: 1.5rem 2rem; max-width: 1280px; margin: 0 auto; }
.section { margin-bottom: 2.5rem; }
.section h2 {
  color: var(--accent);
  margin-bottom: 1rem;
  font-size: 0.85rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

/* ── Grid / Cards ── */
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 1rem;
}
.card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 1.25rem;
  transition: var(--transition);
  position: relative;
  overflow: hidden;
}
.card:hover {
  border-color: var(--border-hover);
  box-shadow: var(--shadow-glow), var(--shadow);
  transform: translateY(-2px);
}
.card h3 { color: var(--text-bright); margin-bottom: 0.4rem; font-size: 1rem; font-weight: 600; }
.card p { color: var(--text); font-size: 0.85rem; margin-bottom: 0.6rem; line-height: 1.5; }

/* ── Service Card Specifics ── */
.service-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.5rem; }
.service-icon { font-size: 1.5rem; filter: drop-shadow(0 0 4px rgba(0,240,255,0.2)); }
.tier-badge {
  font-size: 0.6rem;
  font-weight: 700;
  padding: 0.2rem 0.5rem;
  border-radius: 3px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}
.tier-micro { background: rgba(0,240,255,0.1); color: var(--accent); border: 1px solid rgba(0,240,255,0.2); }
.tier-real { background: rgba(255,190,11,0.1); color: var(--accent-3); border: 1px solid rgba(255,190,11,0.2); }
.tier-heavy { background: rgba(255,0,110,0.1); color: var(--accent-2); border: 1px solid rgba(255,0,110,0.2); }
.tier-local { background: rgba(0,255,136,0.1); color: var(--success); border: 1px solid rgba(0,255,136,0.2); }
.tier-unknown { background: rgba(102,102,102,0.1); color: #888; border: 1px solid rgba(102,102,102,0.2); }
.model-info {
  color: var(--text-dim);
  font-size: 0.7rem;
  font-family: var(--mono);
  margin-bottom: 0.75rem;
  opacity: 0.7;
  display: flex; align-items: center; gap: 0.3rem;
}
.price {
  color: var(--accent-3);
  font-weight: 700;
  font-size: 1.3rem;
  margin-bottom: 0.4rem;
  font-family: var(--mono);
  letter-spacing: 0.02em;
}
.meta { color: var(--text-dim); font-size: 0.75rem; margin-bottom: 0.75rem; }
.meta .by-agent { cursor: help; position: relative; }
.meta .by-agent:hover { color: var(--accent); }
.buy-row { display: flex; gap: 0.5rem; flex-wrap: wrap; }

/* ── Buttons ── */
.btn {
  display: inline-flex; align-items: center; justify-content: center; gap: 0.3rem;
  background: var(--accent);
  color: var(--bg);
  padding: 0.5rem 1.25rem;
  border-radius: var(--radius-sm);
  text-decoration: none;
  font-weight: 700;
  font-size: 0.85rem;
  border: none;
  cursor: pointer;
  transition: var(--transition);
  white-space: nowrap;
  box-shadow: 0 0 12px rgba(0,240,255,0.15);
}
.btn:hover {
  box-shadow: 0 0 20px rgba(0,240,255,0.3);
  transform: scale(1.02);
}
.btn:active { transform: scale(0.98); }
.btn-secondary {
  background: transparent;
  color: var(--accent);
  border: 1px solid rgba(0,240,255,0.4);
  box-shadow: none;
}
.btn-secondary:hover { background: rgba(0,240,255,0.08); box-shadow: 0 0 12px rgba(0,240,255,0.1); }
.btn-tertiary { background: transparent; color: var(--accent-2); border: 1px solid rgba(255,0,110,0.4); box-shadow: none; }
.btn-tertiary:hover { background: rgba(255,0,110,0.08); box-shadow: 0 0 12px rgba(255,0,110,0.1); }
.btn-sm { padding: 0.35rem 0.85rem; font-size: 0.8rem; }

/* ── Stats ── */
.stats {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 1rem;
  margin-bottom: 1.5rem;
}
.stat-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 1.25rem 1rem;
  text-align: center;
  transition: var(--transition);
  position: relative;
  overflow: hidden;
}
.stat-card:hover {
  border-color: rgba(0,240,255,0.3);
  box-shadow: var(--shadow-glow);
  transform: translateY(-1px);
}
.stat-card .value {
  font-size: 2.8rem;
  font-weight: 700;
  color: var(--accent);
  font-family: var(--mono);
  letter-spacing: -0.02em;
  line-height: 1;
  text-shadow: 0 0 16px rgba(0,240,255,0.2);
}
.stat-card .value.pulse-value {
  animation: pulse-glow 3s ease-in-out infinite;
}
@keyframes pulse-glow {
  0%, 100% { text-shadow: 0 0 16px rgba(0,240,255,0.2); }
  50% { text-shadow: 0 0 28px rgba(0,240,255,0.4); }
}
.stat-card .label {
  color: var(--text-dim);
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  margin-top: 0.4rem;
}

/* ── Tier Stacked Bar ── */
.tier-bar-container {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin: 1rem 0 2rem;
  padding: 0.75rem 1rem;
  background: var(--surface);
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
}
.tier-bar-track {
  flex: 1;
  height: 8px;
  background: var(--surface-2);
  border-radius: 4px;
  overflow: hidden;
  display: flex;
}
.tier-bar-segment {
  height: 100%;
  transition: width 0.5s ease;
  min-width: 2px;
}
.tier-bar-segment.micro { background: var(--accent); }
.tier-bar-segment.real { background: var(--accent-3); }
.tier-bar-segment.heavy { background: var(--accent-2); }
.tier-bar-segment.local { background: var(--success); }
.tier-bar-labels {
  display: flex;
  gap: 1rem;
  font-size: 0.75rem;
  color: var(--text-dim);
}
.tier-bar-labels span { display: flex; align-items: center; gap: 0.3rem; }
.tier-bar-labels .dot { width: 6px; height: 6px; border-radius: 50%; }
.tier-bar-labels .dot.micro { background: var(--accent); }
.tier-bar-labels .dot.real { background: var(--accent-3); }
.tier-bar-labels .dot.heavy { background: var(--accent-2); }
.tier-bar-labels .dot.local { background: var(--success); }

/* ── Status Badges ── */
.status-dot {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.status-dot .dot {
  width: 6px; height: 6px;
  border-radius: 50%;
}
.status-dot.active .dot { background: var(--accent); box-shadow: 0 0 6px rgba(0,240,255,0.5); animation: pulse-dot 2s ease-in-out infinite; }
.status-dot.pending .dot { background: var(--accent-3); box-shadow: 0 0 6px rgba(255,190,11,0.4); animation: pulse-dot-slow 3s ease-in-out infinite; }
.status-dot.new-badge {
  position: absolute;
  top: 0.75rem; right: 0.75rem;
  color: var(--accent-2);
  font-size: 0.65rem;
  font-weight: 700;
}
.status-dot.new-badge .dot { background: var(--accent-2); box-shadow: 0 0 6px rgba(255,0,110,0.4); }
@keyframes pulse-dot-slow {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.5; transform: scale(0.85); }
}

/* ── Activity Feed ── */
.activity-feed {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 0.75rem;
  max-height: 420px;
  overflow-y: auto;
}
.activity-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.65rem 0.5rem;
  border-bottom: 1px solid var(--border);
  transition: background 0.15s;
  border-radius: var(--radius-sm);
  animation: slideInRight 0.3s ease;
}
@keyframes slideInRight {
  from { opacity: 0; transform: translateX(12px); }
  to { opacity: 1; transform: translateX(0); }
}
.activity-item:last-child { border-bottom: none; }
.activity-item:hover { background: var(--surface-2); }
.activity-icon { font-size: 1.1rem; width: 28px; text-align: center; flex-shrink: 0; }
.activity-details { display: flex; flex-direction: column; flex: 1; min-width: 0; }
.activity-text { color: var(--text); font-size: 0.85rem; line-height: 1.4; }
.activity-text strong { color: var(--text-bright); font-weight: 600; }
.activity-meta { color: var(--text-dim); font-size: 0.75rem; margin-top: 0.1rem; }
.activity-skeleton {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.65rem 0.5rem;
}
.activity-skeleton .sk-icon { width: 28px; height: 28px; border-radius: 50%; background: var(--surface-2); animation: skeleton 1.5s infinite; }
.activity-skeleton .sk-text { flex: 1; height: 12px; border-radius: 3px; background: var(--surface-2); animation: skeleton 1.5s infinite; }
.activity-skeleton .sk-text.short { width: 60%; }
@keyframes skeleton {
  0% { background: var(--surface-2); }
  50% { background: var(--surface-3); }
  100% { background: var(--surface-2); }
}

/* ── Two Column Layout ── */
.two-col { display: grid; grid-template-columns: 2fr 1fr; gap: 1.25rem; }

/* ── Agent Cards ── */
.agent-card { text-align: left; }
.agent-card-header { display: flex; align-items: center; gap: 0.75rem; margin-bottom: 0.5rem; position: relative; }
.agent-avatar {
  width: 44px; height: 44px;
  border-radius: 50%;
  display: flex; align-items: center; justify-content: center;
  font-size: 0.9rem;
  font-weight: 700;
  color: #fff;
  flex-shrink: 0;
  text-shadow: 0 1px 2px rgba(0,0,0,0.3);
}
.agent-card-info { flex: 1; min-width: 0; }
.agent-card-info h3 { margin-bottom: 0.15rem; font-size: 0.95rem; }
.agent-card-info p { color: var(--text-dim); font-size: 0.8rem; margin-bottom: 0; }
.agent-card-bottom {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 0.75rem;
  padding-top: 0.75rem;
  border-top: 1px solid var(--border);
}
.agent-card-bottom .balance {
  font-family: var(--mono);
  font-size: 1.1rem;
  font-weight: 700;
  color: var(--accent-3);
}
.agent-card-bottom .balance-label {
  font-size: 0.7rem;
  color: var(--text-dim);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}
.agent-recent {
  font-size: 0.75rem;
  color: var(--text-dim);
  margin-top: 0.4rem;
  font-style: italic;
}
.stripe-check { color: var(--success); font-size: 0.85rem; }

/* ── Transaction Rows ── */
.tx-list { display: flex; flex-direction: column; gap: 0.5rem; }
.tx-row-item {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.85rem 1rem;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  transition: var(--transition);
}
.tx-row-item:hover { border-color: rgba(0,240,255,0.2); background: var(--surface-2); }
.tx-row-icon { font-size: 1.2rem; width: 32px; text-align: center; flex-shrink: 0; }
.tx-row-body { flex: 1; min-width: 0; }
.tx-row-body .tx-desc { color: var(--text); font-size: 0.9rem; }
.tx-row-body .tx-desc strong { color: var(--text-bright); font-weight: 600; }
.tx-row-body .tx-time { color: var(--text-dim); font-size: 0.75rem; margin-top: 0.15rem; }
.tx-row-right { display: flex; align-items: center; gap: 0.75rem; flex-shrink: 0; }
.tx-row-price { font-family: var(--mono); font-weight: 700; font-size: 1.1rem; color: var(--accent-3); }

/* ── Deliverable ── */
.deliverable-output {
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 1.25rem;
  margin-top: 1rem;
}
.deliverable-output h3 { color: var(--accent); margin-bottom: 0.75rem; font-size: 0.9rem; }
.deliverable-output p { color: var(--text); line-height: 1.7; font-family: var(--mono); font-size: 0.85rem; }

/* ── Data Tables ── */
.data-table { width: 100%; border-collapse: collapse; }
.data-table th { text-align: left; padding: 0.6rem 0.75rem; color: var(--accent); border-bottom: 1px solid var(--border); font-size: 0.75rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.08em; }
.data-table td { padding: 0.6rem 0.75rem; border-bottom: 1px solid var(--border); color: var(--text); font-size: 0.85rem; }
.data-table tr:hover td { background: var(--surface-2); }
.data-table .col-time { width: 80px; }
.data-table .col-action { font-weight: 500; }
.data-table .col-action.purchase { color: var(--accent); }
.data-table .col-action.create { color: var(--accent-2); }
.data-table .col-action.review { color: var(--accent-3); }
.status-badge { padding: 0.2rem 0.5rem; border-radius: 3px; font-size: 0.75rem; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; }
.status-active { background: rgba(0,240,255,0.1); color: var(--accent); }
.status-escrow { background: rgba(255,190,11,0.1); color: var(--accent-3); }
.status-released { background: rgba(0,255,136,0.1); color: var(--success); }
.status-disputed { background: rgba(255,0,110,0.1); color: var(--accent-2); }
.status-pending { background: rgba(157,78,221,0.1); color: var(--muted); }

/* ── Filter Pills ── */
.filter-pills { display: flex; gap: 0.5rem; margin-bottom: 1.25rem; flex-wrap: wrap; }
.filter-pill {
  padding: 0.4rem 1rem;
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--text-dim);
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  transition: var(--transition);
}
.filter-pill:hover { color: var(--text); }
.filter-pill.active { color: var(--accent); border-bottom-color: var(--accent); }

/* ── Activity Page Specifics ── */
.stats-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr)); gap: 0.75rem; margin-bottom: 1.5rem; }
.stats-grid .stat-card { padding: 1rem 0.75rem; }
.stats-grid .stat-card .value { font-size: 2.2rem; }
.stats-grid .stat-card .label { font-size: 0.65rem; }
.activity-table { width: 100%; border-collapse: collapse; }
.activity-table th { text-align: left; padding: 0.6rem 0.75rem; color: var(--accent); border-bottom: 1px solid var(--border); font-size: 0.75rem; text-transform: uppercase; }
.activity-table td { padding: 0.6rem 0.75rem; border-bottom: 1px solid var(--border); color: var(--text); font-size: 0.85rem; }
.activity-table tr:hover td { background: var(--surface-2); }
.log-icon { font-size: 1.1rem; width: 32px; text-align: center; }
.log-time { color: var(--text-dim); font-size: 0.75rem; white-space: nowrap; }
.log-agent a { color: var(--accent); text-decoration: none; }
.log-agent a:hover { text-decoration: underline; }
.log-action { text-transform: capitalize; font-weight: 500; }
.log-target a { color: var(--accent-3); text-decoration: none; }
.log-target a:hover { text-decoration: underline; }
.log-amount { color: var(--success); font-weight: 500; text-align: right; font-family: var(--mono); }
.log-details { color: var(--text-dim); max-width: 280px; overflow: hidden; text-overflow: ellipsis; }
.action-purchase { border-left: 2px solid var(--accent); }
.action-create { border-left: 2px solid var(--accent-2); }
.action-review { border-left: 2px solid var(--accent-3); }
.action-other { border-left: 2px solid var(--border); }
.empty-state { text-align: center; padding: 2.5rem; color: var(--text-dim); }
.empty-state-icon { font-size: 3rem; margin-bottom: 0.75rem; opacity: 0.5; }

/* ── Agent Loop Page ── */
.action-bar { display: flex; gap: 1rem; align-items: center; margin-bottom: 1.5rem; flex-wrap: wrap; }
.tick-status { color: var(--accent); font-weight: 500; font-size: 0.85rem; }
.interactions { display: flex; flex-direction: column; gap: 0.4rem; }
.interaction {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 0.6rem 0.85rem;
  display: flex;
  gap: 0.75rem;
  align-items: center;
  font-size: 0.85rem;
  transition: background 0.3s;
}
.interaction.flash { background: rgba(0,240,255,0.08); }
.interaction.success { border-left: 2px solid var(--success); }
.interaction.failed { border-left: 2px solid var(--err); }
.int-type { color: var(--accent); font-weight: 700; min-width: 90px; font-size: 0.8rem; }
.int-agent { color: var(--accent-3); min-width: 110px; font-size: 0.85rem; }
.int-msg { color: var(--text-dim); font-size: 0.85rem; }

/* ── Event Log Terminal ── */
.event-log {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 0.75rem;
  max-height: 200px;
  overflow-y: auto;
  font-family: var(--mono);
  font-size: 0.8rem;
  line-height: 1.6;
}
.event-log .log-line { color: var(--text-dim); }
.event-log .log-line .log-time { color: var(--accent); opacity: 0.7; }
.event-log .log-line .log-agent { color: var(--accent-3); }
.event-log .log-line .log-action { color: var(--text); }
.event-log .log-line .log-service { color: var(--accent); }
.event-log .log-line .log-price { color: var(--accent-3); }

/* ── Speed Toggle ── */
.speed-toggle { display: flex; align-items: center; gap: 0.3rem; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 0.2rem; }
.speed-toggle button {
  padding: 0.3rem 0.6rem;
  font-size: 0.75rem;
  font-weight: 600;
  border: none;
  background: transparent;
  color: var(--text-dim);
  cursor: pointer;
  border-radius: 4px;
  transition: var(--transition);
}
.speed-toggle button.active { background: var(--accent); color: var(--bg); }

/* ── Showcase / Monitor ── */
.showcase-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(340px, 1fr)); gap: 1rem; }
.showcase-card { background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius); padding: 1.25rem; transition: var(--transition); }
.showcase-card:hover { border-color: rgba(0,240,255,0.2); box-shadow: var(--shadow-glow); transform: translateY(-1px); }
.showcase-header { display: flex; align-items: center; gap: 0.75rem; margin-bottom: 0.5rem; }
.showcase-icon { font-size: 1.5rem; }
.showcase-type { color: var(--text-dim); font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.05em; }
.showcase-desc { color: var(--text); font-size: 0.85rem; margin-bottom: 0.75rem; line-height: 1.5; }
.showcase-price { color: var(--accent-3); font-size: 1.2rem; font-weight: 700; font-family: var(--mono); margin-bottom: 0.75rem; }
.showcase-sample { margin-bottom: 0.6rem; }
.sample-label { color: var(--accent-3); font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 0.2rem; }
.sample-code { background: var(--bg); border: 1px solid var(--border); border-radius: 4px; padding: 0.6rem; font-family: var(--mono); font-size: 0.8rem; color: var(--text-dim); overflow-x: auto; }
.showcase-meta { display: flex; justify-content: space-between; align-items: center; margin-top: 0.75rem; padding-top: 0.75rem; border-top: 1px solid var(--border); font-size: 0.8rem; }
.showcase-meta .seller { color: var(--text-dim); }
.showcase-meta .model-tag {
  font-size: 0.7rem;
  font-family: var(--mono);
  color: var(--text-dim);
  background: var(--surface-2);
  padding: 0.2rem 0.4rem;
  border-radius: 3px;
}
.showcase-empty-msg {
  color: var(--text-dim);
  font-size: 0.85rem;
  font-style: italic;
  margin: 0.5rem 0;
}

/* ── Top Agent Card ── */
.top-agent-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 1.25rem;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
}
.top-agent-card .avatar {
  width: 56px; height: 56px;
  border-radius: 50%;
  display: flex; align-items: center; justify-content: center;
  font-size: 1.2rem; font-weight: 700; color: #fff;
}
.top-agent-card .name { font-size: 1rem; font-weight: 700; color: var(--text-bright); white-space: nowrap; }
.top-agent-card .score { font-family: var(--mono); font-size: 1.5rem; color: var(--accent); }

/* ── Loading Skeleton ── */
.skeleton {
  background: linear-gradient(90deg, var(--surface) 25%, var(--surface-2) 50%, var(--surface) 75%);
  background-size: 200% 100%;
  animation: skeleton 1.5s infinite;
  border-radius: var(--radius-sm);
}

/* ── Toast Notifications ── */
.toast-container { position: fixed; top: 1rem; right: 1rem; z-index: 9999; display: flex; flex-direction: column; gap: 0.5rem; max-width: 340px; }
.toast {
  background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm);
  padding: 0.85rem 1.1rem; box-shadow: 0 8px 32px rgba(0,0,0,0.5);
  animation: slideIn 0.35s cubic-bezier(0.4, 0, 0.2, 1);
  backdrop-filter: blur(12px); -webkit-backdrop-filter: blur(12px);
}
.toast-title { display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.25rem; }
.toast-title strong { color: var(--accent); font-size: 0.9rem; }
.toast-msg { color: var(--muted); font-size: 0.82rem; }
@keyframes slideIn { from { opacity: 0; transform: translateX(120%); } to { opacity: 1; transform: translateX(0); } }

/* ── Modal ── */
.modal-overlay {
  position: fixed; top: 0; left: 0; right: 0; bottom: 0;
  background: rgba(0,0,0,0.85); z-index: 2000;
  display: flex; align-items: center; justify-content: center; padding: 2rem;
  animation: fadeIn 0.2s ease;
}
@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
.modal-box {
  background: var(--surface); border: 1px solid var(--accent);
  border-radius: var(--radius); max-width: 900px; width: 100%;
  max-height: 90vh; overflow: auto; padding: 2rem;
  animation: scaleIn 0.25s ease;
}
@keyframes scaleIn { from { opacity: 0; transform: scale(0.95); } to { opacity: 1; transform: scale(1); } }

/* ── Try Output Formatting ── */
.code-block { background: var(--bg); border: 1px solid var(--border); border-radius: 8px; padding: 1.25rem; font-family: var(--mono); font-size: 0.85rem; line-height: 1.6; color: var(--text); overflow-x: auto; white-space: pre-wrap; }
.commit-badge { background: rgba(0,255,136,0.1); border: 1px solid rgba(0,255,136,0.3); border-radius: 8px; padding: 1rem; font-family: var(--mono); font-size: 0.9rem; color: var(--success); }
.regex-pattern { background: rgba(255,190,11,0.1); border: 1px solid rgba(255,190,11,0.3); border-radius: 8px; padding: 1rem; font-family: var(--mono); font-size: 0.9rem; color: var(--accent-3); }
.diff-view { background: var(--bg); border: 1px solid var(--border); border-radius: 8px; padding: 0.75rem; font-family: var(--mono); font-size: 0.8rem; line-height: 1.5; }
.diff-add { color: var(--success); background: rgba(0,255,136,0.05); padding: 0.15rem 0.5rem; }
.diff-del { color: var(--accent-2); background: rgba(255,0,110,0.05); padding: 0.15rem 0.5rem; }
.diff-hunk { color: var(--accent); background: rgba(0,240,255,0.05); padding: 0.15rem 0.5rem; font-weight: 600; }
.diff-line { color: var(--text-dim); padding: 0.15rem 0.5rem; }

/* ── Footer ── */
footer {
  text-align: center; padding: 2rem;
  color: var(--text-dim); font-size: 0.75rem;
  border-top: 1px solid var(--border); margin-top: 2rem;
}
footer a { color: var(--text-dim); transition: color 0.2s; }
footer a:hover { color: var(--accent); }

/* ── Responsive ── */
@media (max-width: 768px) {
  header { flex-direction: row; padding: 0.75rem 1rem; }
  header h1 { font-size: 1.2rem; }
  nav { display: none; position: absolute; top: 100%; left: 0; right: 0; background: var(--surface); border-bottom: 1px solid var(--border); flex-direction: column; padding: 1rem; gap: 0.5rem; }
  nav.open { display: flex; }
  nav a { margin: 0; width: 100%; text-align: center; }
  nav a.active::after { display: none; }
  #nav-toggle { display: block; }
  .hero { padding: 2.5rem 1rem; }
  .hero h2 { font-size: 2rem; }
  .container { padding: 1rem; }
  .stats { grid-template-columns: repeat(2, 1fr); }
  .stat-card .value { font-size: 2rem; }
  .two-col { grid-template-columns: 1fr; }
  .grid { grid-template-columns: 1fr; }
  .showcase-grid { grid-template-columns: 1fr; }
  .buy-row { flex-direction: column; }
  .buy-row a, .buy-row button { width: 100%; text-align: center; }
  .tier-bar-container { flex-wrap: wrap; }
  .toast-container { left: 1rem; right: 1rem; max-width: none; }
  .modal-overlay { padding: 1rem; }
  .modal-box { padding: 1.25rem; max-height: 95vh; }
  .data-table { font-size: 0.8rem; }
  .data-table th, .data-table td { padding: 0.5rem; }
  .tx-row-item { flex-wrap: wrap; }
}

@media (max-width: 480px) {
  .stats { grid-template-columns: 1fr; }
  .hero h2 { font-size: 1.6rem; }
  .hero p { font-size: 0.9rem; }
  .section h2 { font-size: 0.8rem; }
}
"#;

const SHARED_JS: &str = r#"
// Account State
function getAccount() { try { return JSON.parse(localStorage.getItem("clawAccount")); } catch(e) { return null; } }
function isLinked() { return !!getAccount(); }
function getTries() { try { return JSON.parse(localStorage.getItem("clawTries") || "{}"); } catch(e) { return {}; } }
function hasTried(serviceId) { return !!getTries()[serviceId]; }
function recordTry(serviceId) { const t = getTries(); t[serviceId] = true; localStorage.setItem("clawTries", JSON.stringify(t)); }

// Nav Toggle (Mobile)
function toggleNav() { document.querySelector("nav").classList.toggle("open"); }

// Active Nav Link
(function() {
  const path = location.pathname;
  document.querySelectorAll("nav a").forEach(a => {
    if (a.getAttribute("href") === path) a.classList.add("active");
  });
})();

// Demo Purchase
async function demoPurchase(serviceId) {
  const btn = event.target;
  btn.textContent = "Processing...";
  btn.disabled = true;
  try {
    const resp = await fetch("/api/demo/purchase", {
      method: "POST",
      headers: {"Content-Type": "application/json"},
      body: JSON.stringify({service_id: serviceId, buyer_id: "anonymous"})
    });
    const data = await resp.json();
    if (data.transaction_id) {
      alert("Demo purchase complete! TX: " + data.transaction_id.slice(0,8) + "...\nStatus: " + data.status + "\n\n" + data.message);
      location.reload();
    } else {
      alert("Error: " + (data.error || "Unknown error"));
      btn.textContent = "Demo Buy";
      btn.disabled = false;
    }
  } catch (e) {
    alert("Network error: " + e);
    btn.textContent = "Demo Buy";
    btn.disabled = false;
  }
}

// Try Service Buttons
function initTryButtons() {
  document.querySelectorAll(".btn-try").forEach(btn => {
    const id = btn.id.replace("try-btn-", "");
    if (!id) return;
    if (isLinked()) { btn.style.display = "none"; return; }
    if (hasTried(id)) {
      btn.textContent = "Link Account";
      btn.classList.remove("btn-try");
      btn.classList.add("btn-secondary");
      btn.onclick = () => showLinkAccountModal();
    } else {
      btn.onclick = () => tryService(id);
    }
  });
}

function initBuyButtons() {
  document.querySelectorAll(".btn-buy").forEach(btn => {
    const href = btn.getAttribute("href");
    if (!href) return;
    const match = href.match(/service_id=([^&]+)/);
    if (!match) return;
    const serviceId = match[1];
    btn.removeAttribute("href");
    btn.addEventListener("click", (e) => {
      e.preventDefault();
      demoPurchase(serviceId);
    });
  });
}

function showLinkAccountModal() {
  if (document.getElementById("link-modal")) return;
  const modal = document.createElement("div");
  modal.id = "link-modal";
  modal.className = "modal-overlay";
  modal.innerHTML = '<div class="modal-box" style="max-width:420px;text-align:center;"><h3 style="color:var(--accent-2);margin-bottom:1rem;">Account Required</h3><p style="color:var(--muted);margin-bottom:1.5rem;line-height:1.6;">You have used your free try! Create a free account to continue using ClawTrade services.</p><div style="display:flex;flex-direction:column;gap:0.75rem;"><button onclick="createAccount()" class="btn" style="width:100%;">Create Free Account</button><button onclick="document.getElementById(\'link-modal\').remove()" class="btn btn-secondary" style="width:100%;">Maybe Later</button></div></div>';
  document.body.appendChild(modal);
  modal.addEventListener("click", e => { if (e.target === modal) modal.remove(); });
}

function createAccount() {
  const id = "user_" + Math.random().toString(36).slice(2, 10);
  localStorage.setItem("clawAccount", JSON.stringify({ id, name: "User " + id.slice(-4), created: Date.now() }));
  document.getElementById("link-modal")?.remove();
  location.reload();
}

function tryService(serviceId) {
  if (isLinked()) { alert("Please purchase this service to use it."); return; }
  if (hasTried(serviceId)) { showLinkAccountModal(); return; }
  if (document.getElementById("try-modal")) return;
  const modal = document.createElement("div");
  modal.id = "try-modal";
  modal.className = "modal-overlay";
  modal.innerHTML = '<div class="modal-box"><h3 style="color:var(--accent);margin-bottom:1rem;">Try Service (1 Free)</h3><p style="color:var(--muted);margin-bottom:1rem;font-size:0.9rem;">Enter your text below and click Run to see the LLM-generated result. <strong style="color:var(--accent-3);">One free try per service.</strong></p><textarea id="try-input" style="width:100%;min-height:120px;background:var(--surface-2);border:1px solid var(--border);border-radius:8px;padding:1rem;color:var(--text);font-family:var(--mono);font-size:0.9rem;resize:vertical;" placeholder="Enter text to process..."></textarea><div style="margin-top:1rem;display:flex;gap:0.5rem;justify-content:flex-end;"><button onclick="document.getElementById(\'try-modal\').remove()" class="btn btn-secondary">Cancel</button><button id="try-run-btn" class="btn">Run Service</button></div><div id="try-loading" style="display:none;text-align:center;padding:2rem;color:var(--muted);"><div style="font-size:2rem;margin-bottom:1rem;">⚡</div><div>Processing with local LLM...</div><div style="font-size:0.8rem;margin-top:0.5rem;">This may take 3-5 seconds</div></div><pre id="try-result" style="display:none;background:var(--surface-2);border:1px solid var(--border);border-radius:8px;padding:1.5rem;overflow:auto;white-space:pre-wrap;font-family:var(--mono);font-size:0.85rem;line-height:1.6;color:var(--text);max-height:400px;margin-top:1rem;"></pre></div>';
  document.body.appendChild(modal);
  modal.addEventListener("click", e => { if (e.target === modal) modal.remove(); });
  document.getElementById("try-run-btn").addEventListener("click", () => runTryService(serviceId));
}

async function runTryService(serviceId) {
  const input = document.getElementById("try-input").value;
  const btn = document.getElementById("try-run-btn");
  btn.disabled = true; btn.textContent = "Running...";
  document.getElementById("try-loading").style.display = "block";
  document.getElementById("try-result").style.display = "none";
  try {
    const resp = await fetch("/api/services/" + serviceId + "/execute", {
      method: "POST", headers: {"Content-Type": "application/json"},
      body: JSON.stringify({user_input: input || "Sample text for processing"})
    });
    const data = await resp.json();
    document.getElementById("try-loading").style.display = "none";
    const el = document.getElementById("try-result");
    el.style.display = "block";
    if (data.result) {
      // Format output based on service type
      const serviceType = data.service_type || "unknown";
      el.innerHTML = formatTryOutput(serviceType, data.result);
      recordTry(serviceId);
    } else {
      el.textContent = "Error: " + (data.error || "Unknown error");
    }
  } catch (e) {
    document.getElementById("try-loading").style.display = "none";
    const el = document.getElementById("try-result");
    el.style.display = "block"; el.textContent = "Error: " + e;
  }
  btn.disabled = false; btn.textContent = "Run Service";
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

function formatTryOutput(serviceType, rawText) {
  switch(serviceType) {
    case 'code_lint_fix':
    case 'code_review':
    case 'repo_refactor':
      return '<pre class="code-block">' + escapeHtml(rawText) + '</pre>';
    case 'git_commit_msg':
      return '<div class="commit-badge">' + escapeHtml(rawText) + '</div>';
    case 'regex_generator':
      return '<div class="regex-pattern">' + escapeHtml(rawText) + '</div>';
    case 'csv_converter':
      try {
        const data = JSON.parse(rawText);
        return renderJsonTable(data);
      } catch {
        return '<pre>' + escapeHtml(rawText) + '</pre>';
      }
    case 'diff_explainer':
      return renderDiffView(rawText);
    default:
      return '<pre>' + escapeHtml(rawText) + '</pre>';
  }
}

function renderJsonTable(data) {
  if (!Array.isArray(data) || data.length === 0) return '<pre>' + escapeHtml(JSON.stringify(data, null, 2)) + '</pre>';
  const keys = Object.keys(data[0]);
  let html = '<table class="data-table">';
  html += '<thead><tr>' + keys.map(k => '<th>' + escapeHtml(k) + '</th>').join('') + '</tr></thead>';
  html += '<tbody>';
  for (const row of data) {
    html += '<tr>' + keys.map(k => '<td>' + escapeHtml(String(row[k] ?? '')) + '</td>').join('') + '</tr>';
  }
  html += '</tbody></table>';
  return html;
}

function renderDiffView(rawText) {
  const lines = rawText.split('\n');
  let html = '<div class="diff-view">';
  for (const line of lines) {
    if (line.startsWith('+')) {
      html += '<div class="diff-add">' + escapeHtml(line) + '</div>';
    } else if (line.startsWith('-')) {
      html += '<div class="diff-del">' + escapeHtml(line) + '</div>';
    } else if (line.startsWith('@@')) {
      html += '<div class="diff-hunk">' + escapeHtml(line) + '</div>';
    } else {
      html += '<div class="diff-line">' + escapeHtml(line) + '</div>';
    }
  }
  html += '</div>';
  return html;
}

// Live Demo (Monitor Page)
async function runLiveDemo(serviceId) {
  if (document.getElementById("live-demo-modal")) return;
  const modal = document.createElement("div");
  modal.id = "live-demo-modal";
  modal.className = "modal-overlay";
  modal.innerHTML = '<div class="modal-box" style="max-width:700px;"><h3 style="color:var(--accent);margin-bottom:1rem;">⚡ Live Demo</h3><div id="live-demo-loading" style="text-align:center;padding:2rem;"><div style="font-size:2rem;margin-bottom:1rem;">🦞</div><div>Running service with local LLM...</div><div style="font-size:0.8rem;margin-top:0.5rem;color:var(--muted);">This may take a moment</div></div><div id="live-demo-error" style="display:none;color:var(--accent-2);padding:1rem;"></div><div id="live-demo-content" style="display:none;"><div style="margin-bottom:1rem;"><div class="sample-label">Service</div><div id="live-demo-service" style="color:var(--accent);font-weight:700;"></div></div><div style="margin-bottom:1rem;"><div class="sample-label">Input</div><pre id="live-demo-input" class="sample-code"></pre></div><div style="margin-bottom:1rem;"><div class="sample-label">Output</div><pre id="live-demo-output" class="sample-code" style="white-space:pre-wrap;max-height:300px;overflow:auto;"></pre></div><div style="display:flex;gap:1rem;justify-content:space-between;align-items:center;margin-top:1rem;padding-top:1rem;border-top:1px solid var(--border);"><span id="live-demo-meta" style="color:var(--muted);font-size:0.8rem;"></span><button onclick="document.getElementById(\'live-demo-modal\').remove()" class="btn btn-secondary">Close</button></div></div></div>';
  document.body.appendChild(modal);
  modal.addEventListener("click", e => { if (e.target === modal) modal.remove(); });
  try {
    const resp = await fetch("/api/monitor/demonstrate/" + serviceId);
    const data = await resp.json();
    document.getElementById("live-demo-loading").style.display = "none";
    if (data.error) {
      const errEl = document.getElementById("live-demo-error");
      errEl.style.display = "block";
      errEl.textContent = "Error: " + data.error;
      return;
    }
    const demo = data.demo;
    document.getElementById("live-demo-content").style.display = "block";
    document.getElementById("live-demo-service").textContent = demo.service_name + " (" + demo.service_type + ")";
    document.getElementById("live-demo-input").textContent = demo.sample_input || "N/A";
    document.getElementById("live-demo-output").textContent = demo.output || "No output";
    document.getElementById("live-demo-meta").innerHTML = "⏱️ " + demo.latency_ms + "ms &bull; 💰 $" + (demo.price_cents / 100).toFixed(2) + " &bull; " + demo.powered_by;
  } catch (e) {
    document.getElementById("live-demo-loading").style.display = "none";
    const errEl = document.getElementById("live-demo-error");
    errEl.style.display = "block";
    errEl.textContent = "Network error: " + e.message;
  }
}

// WebSocket Activity Feed
(function() {
  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  const wsHost = window.location.host.replace(":8746", ":3000");
  const wsUrl = protocol + "//" + wsHost + "/ws";
  let ws = null, reconnectTimer = null;

  function connect() {
    try {
      ws = new WebSocket(wsUrl);
      ws.onopen = () => console.log("[WS] Connected");
      ws.onmessage = (e) => {
        try {
          const data = JSON.parse(e.data);
          console.log("[WS]", data);
          if (window.refreshActivity && typeof window.refreshActivity === "function") {
            if (!window._lastRefresh || Date.now() - window._lastRefresh > 2000) {
              window._lastRefresh = Date.now();
              window.refreshActivity();
            }
          }
        } catch (err) {}
      };
      ws.onclose = () => { if (reconnectTimer) clearTimeout(reconnectTimer); reconnectTimer = setTimeout(connect, 5000); };
      ws.onerror = () => ws.close();
    } catch (e) {}
  }
  connect();
})();

// Init
function initAllButtons() {
  initTryButtons();
  initBuyButtons();
}
if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", initAllButtons);
else initAllButtons();
"#;

fn wrap_page(title: &str, content: &str, active_agents: usize) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{} — ClawTrade</title>
<link rel="icon" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'%3E%3Ctext y='.9em' font-size='90'%3E🦞%3C/text%3E%3C/svg%3E">
<style>{}</style>
</head>
<body>
<header>
  <h1><a href="/">🦞 ClawTrade</a></h1>
  <div style="display:flex;align-items:center;gap:1rem;">
    <div class="live-indicator" data-tooltip="{} agents active">
      <span class="dot"></span>
      <span>Live</span>
    </div>
    <button id="nav-toggle" onclick="toggleNav()">☰</button>
  </div>
  <nav>
    <a href="/">Marketplace</a>
    <a href="/services">Services</a>
    <a href="/agents">Agents</a>
    <a href="/transactions">Transactions</a>
    <a href="/my-purchases">My Purchases</a>
    <a href="/activity">Activity</a>
    <a href="/monitor">Monitor</a>
    <a href="/inference">Inference</a>
    <a href="/agent-loop">Agent Loop</a>
  </nav>
</header>
<div class="container">{}</div>
<footer>ClawTrade — AI Agent Marketplace &bull; This is the wave. &bull; <a href="https://github.com/synthalorian/clawtrade">GitHub</a></footer>
<script>{}</script>
</body>
</html>"#,
        title, GLOBAL_CSS, active_agents, content, SHARED_JS
    )
}

/* ─────────── PAGE HANDLERS ─────────── */

pub async fn index_handler(State(state): State<Arc<AppState>>) -> Html<String> {
    let services = match Service::list_active(&state.pool).await { Ok(s) => s, Err(_) => vec![] };
    let agents = match Agent::list_top(&state.pool).await { Ok(a) => a, Err(_) => vec![] };
    let transactions = match Transaction::list(&state.pool).await { Ok(t) => t, Err(_) => vec![] };
    let all_agents = match Agent::list(&state.pool).await { Ok(a) => a, Err(_) => vec![] };

    let total_volume = transactions.iter().map(|t| t.amount_cents).sum::<i64>();
    let _paid_count = transactions.iter().filter(|t| t.status == "paid").count();

    let mut tier_micro = 0i64;
    let mut tier_real = 0i64;
    let mut tier_heavy = 0i64;
    let mut tier_local = 0i64;
    for s in &services {
        if let Some(def) = crate::service_catalog::get_service_definition(&s.service_type) {
            match def.tier {
                crate::service_catalog::ServiceTier::MicroTask => tier_micro += 1,
                crate::service_catalog::ServiceTier::RealWork => tier_real += 1,
                crate::service_catalog::ServiceTier::HeavyLifting => tier_heavy += 1,
                crate::service_catalog::ServiceTier::LocalOnly => tier_local += 1,
            }
        }
    }
    let tier_total = tier_micro + tier_real + tier_heavy + tier_local;
    let micro_pct = if tier_total > 0 { (tier_micro as f64 / tier_total as f64 * 100.0) as i64 } else { 0 };
    let real_pct = if tier_total > 0 { (tier_real as f64 / tier_total as f64 * 100.0) as i64 } else { 0 };
    let heavy_pct = if tier_total > 0 { (tier_heavy as f64 / tier_total as f64 * 100.0) as i64 } else { 0 };
    let local_pct = if tier_total > 0 { 100 - micro_pct - real_pct - heavy_pct } else { 0 };

    let services_html = services.iter().take(6).map(|s| {
        let (tier_badge, tier_class, model_info) = match crate::service_catalog::get_service_definition(&s.service_type) {
            Some(def) => {
                let (badge, class) = match def.tier {
                    crate::service_catalog::ServiceTier::MicroTask => ("MICRO", "tier-micro"),
                    crate::service_catalog::ServiceTier::RealWork => ("REAL", "tier-real"),
                    crate::service_catalog::ServiceTier::HeavyLifting => ("HEAVY", "tier-heavy"),
                    crate::service_catalog::ServiceTier::LocalOnly => ("LOCAL", "tier-local"),
                };
                (badge.to_string(), class.to_string(), format!("{} | {}", def.model.model_name(), def.model.context_size()))
            }
            None => ("LEGACY".to_string(), "tier-unknown".to_string(), "legacy service".to_string()),
        };
        format!(
            r#"<div class="card service-card">
                <div class="service-header">
                    <div class="service-icon">{}</div>
                    <span class="tier-badge {}">{}</span>
                </div>
                <h3>{}</h3>
                <p>{}</p>
                <div class="price">${}.{}</div>
                <div class="model-info">{}</div>
                <div class="buy-row">
                  <a href="/api/checkout?service_id={}&buyer_id=anonymous" class="btn btn-buy">Buy</a>
                  <button class="btn btn-secondary btn-try" id="try-btn-{}">Try</button>
                </div>
            </div>"#,
            service_icon(&s.service_type),
            tier_class, tier_badge,
            html_escape(&s.name),
            html_escape(&s.description),
            s.price_cents / 100,
            format_cents(s.price_cents % 100),
            model_info,
            s.id, s.id
        )
    }).collect::<String>();
    let agents_html = agents.iter().take(4).map(|a| {
        let avatar_color = avatar_color_for(&a.name);
        let initials = agent_initials(&a.name);
        format!(
            r#"
<div class="card agent-card">
                <div class="agent-card-header">
                    <div class="agent-avatar" style="background:{};">{}</div>
                    <div class="agent-card-info">
                        <h3>{}</h3>
                        <p>{}</p>
                    </div>
                </div>
                <div class="agent-card-bottom">
                    <div>
                        <div class="balance">${}.{}</div>
                        <div class="balance-label">Balance</div>
                    </div>
                </div>
            </div>"#,
            avatar_color, initials,
            html_escape(&a.name),
            html_escape(&a.description),
            a.balance_cents / 100,
            format_cents(a.balance_cents % 100)
        )
    }).collect::<String>();

    let activity_html = if transactions.is_empty() {
        r#"<div class="activity-skeleton"><div class="sk-icon"></div><div class="sk-text"></div></div>
            <div class="activity-skeleton"><div class="sk-icon"></div><div class="sk-text short"></div></div>
            <div style="text-align:center;padding:1rem;color:var(--text-dim);font-size:0.85rem;">Agents are waking up...</div>"#.to_string()
    } else {
        transactions.iter().take(8).map(|t| {
            let icon = if t.status == "paid" { "🛒" } else { "⭐" };
            let time_ago = time_since(&t.created_at);
            format!(
                r#"<div class="activity-item">
                    <span class="activity-icon">{}</span>
                    <div class="activity-details">
                        <span class="activity-text"><strong>{}</strong> bought <strong>{}</strong> from {}</span>
                        <span class="activity-meta">{} &bull; ${}.{}</span>
                    </div>
                </div>"#,
                icon,
                html_escape(&t.buyer_id[..8.min(t.buyer_id.len())]),
                html_escape(&t.service_id[..8.min(t.service_id.len())]),
                html_escape(&t.seller_id[..8.min(t.seller_id.len())]),
                time_ago,
                t.amount_cents / 100,
                format_cents(t.amount_cents % 100)
            )
        }).collect::<String>()
    };

    let stats_html = format!(
        r#"<div class="stats">
    <div class="stat-card"><div class="value">{}</div><div class="label">Services</div></div>
    <div class="stat-card"><div class="value">{}</div><div class="label">Agents</div></div>
    <div class="stat-card"><div class="value pulse-value">{}</div><div class="label">Active Agents</div></div>
    <div class="stat-card"><div class="value">${}.{}</div><div class="label">Volume</div></div>
  </div>"#,
        services.len(), agents.len(), all_agents.len(),
        total_volume / 100, format_cents(total_volume % 100)
    );

    let tier_bar_html = if tier_total > 0 {
        format!(
            r#"<div class="tier-bar-container">
                <div class="tier-bar-track">
                    <div class="tier-bar-segment micro" style="width:{}%"></div>
                    <div class="tier-bar-segment real" style="width:{}%"></div>
                    <div class="tier-bar-segment heavy" style="width:{}%"></div>
                    <div class="tier-bar-segment local" style="width:{}%"></div>
                </div>
                <div class="tier-bar-labels">
                    <span><span class="dot micro"></span> Micro {}</span>
                    <span><span class="dot real"></span> Real {}</span>
                    <span><span class="dot heavy"></span> Heavy {}</span>
                    <span><span class="dot local"></span> Local {}</span>
                </div>
            </div>"#,
            micro_pct, real_pct, heavy_pct, local_pct,
            tier_micro, tier_real, tier_heavy, tier_local
        )
    } else {
        String::new()
    };

    Html(wrap_page("Marketplace", &format!(
        r#"<div class="hero">
  <h2>AI Agents, Trading Freely</h2>
  <p>The first marketplace where Hermes agents autonomously create, sell, and buy digital services. Powered by Stripe, local LLMs, and synthwave aesthetics.</p>
</div>
<div class="container">
  {}
  {}
  <div class="two-col">
    <div class="section">
      <h2>Featured Services</h2>
      <div class="grid">{}</div>
    </div>
    <div class="section">
      <h2>Live Activity</h2>
      <div class="activity-feed">{}</div>
    </div>
  </div>
  <div class="section">
    <h2>Top Agents</h2>
    <div class="grid">{}</div>
  </div>
</div>"#,
        stats_html, tier_bar_html,
        services_html, activity_html, agents_html
    ), all_agents.len()))
}

pub async fn services_page(State(state): State<Arc<AppState>>) -> Html<String> {
    let all_services = match Service::list(&state.pool).await { Ok(s) => s, Err(_) => vec![] };
    // Filter to only catalog services — remove auto-generated duplicates
    let catalog_types: std::collections::HashSet<String> = crate::service_catalog::SERVICE_CATALOG
        .iter().map(|s| s.service_type.to_string()).collect();
    // Deduplicate by service_type — keep the most recent one
    let mut unique: std::collections::HashMap<String, Service> = std::collections::HashMap::new();
    for s in all_services {
        if !catalog_types.contains(&s.service_type) {
            continue;
        }
        unique.entry(s.service_type.clone())
            .and_modify(|existing| { if s.created_at > existing.created_at { *existing = s.clone(); } })
            .or_insert(s);
    }
    let services: Vec<Service> = unique.into_values().collect();
    let all_agents = match Agent::list(&state.pool).await { Ok(a) => a, Err(_) => vec![] };
    let agent_names: std::collections::HashMap<String, String> = all_agents.iter().map(|a| (a.id.clone(), a.name.clone())).collect();

    let services_html = services.iter().map(|s| {
        let (tier_badge, tier_class, model_info) = match crate::service_catalog::get_service_definition(&s.service_type) {
            Some(def) => {
                let (badge, class) = match def.tier {
                    crate::service_catalog::ServiceTier::MicroTask => ("MICRO", "tier-micro"),
                    crate::service_catalog::ServiceTier::RealWork => ("REAL", "tier-real"),
                    crate::service_catalog::ServiceTier::HeavyLifting => ("HEAVY", "tier-heavy"),
                    crate::service_catalog::ServiceTier::LocalOnly => ("LOCAL", "tier-local"),
                };
                (badge.to_string(), class.to_string(), format!("{} | {}", def.model.model_name(), def.model.context_size()))
            }
            None => ("LEGACY".to_string(), "tier-unknown".to_string(), "legacy service".to_string()),
        };
        let agent_name = agent_names.get(&s.agent_id).map(|n| n.as_str()).unwrap_or("Unknown");
        format!(
            r#"<div class="card service-card" data-tier="{}">
                <div class="service-header">
                    <div class="service-icon">{}</div>
                    <span class="tier-badge {}">{}</span>
                </div>
                <h3>{}</h3>
                <p>{}</p>
                <div class="price">${}.{}</div>
                <div class="model-info">{}</div>
                <div class="meta">by <span class="by-agent" title="{}">{}</span></div>
                <div class="buy-row">
                  <a href="/api/checkout?service_id={}&buyer_id=anonymous" class="btn btn-buy">Buy</a>
                  <button class="btn btn-secondary btn-try" id="try-btn-{}">Try</button>
                </div>
            </div>"#,
            tier_class.replace("tier-", ""),
            service_icon(&s.service_type),
            tier_class, tier_badge,
            html_escape(&s.name),
            html_escape(&s.description),
            s.price_cents / 100,
            format_cents(s.price_cents % 100),
            model_info,
            html_escape(&s.agent_id), html_escape(agent_name),
            s.id, s.id
        )
    }).collect::<String>();

    Html(wrap_page("Services", &format!(
        r#"<div class="section">
            <h2>All Services</h2>
            <div class="filter-pills">
                <button class="filter-pill active" onclick="filterServices('all', this)">All</button>
                <button class="filter-pill" onclick="filterServices('micro', this)">Micro</button>
                <button class="filter-pill" onclick="filterServices('real', this)">Real</button>
                <button class="filter-pill" onclick="filterServices('heavy', this)">Heavy</button>
                <button class="filter-pill" onclick="filterServices('local', this)">Local</button>
            </div>
            <div class="grid" id="services-grid">{}</div>
        </div>
        <script>
        function filterServices(tier, btn) {{
            document.querySelectorAll('.filter-pill').forEach(p => p.classList.remove('active'));
            btn.classList.add('active');
            document.querySelectorAll('.service-card').forEach(card => {{
                if (tier === 'all' || card.dataset.tier === tier) card.style.display = '';
                else card.style.display = 'none';
            }});
        }}
        </script>"#,
        services_html
    ), all_agents.len()))
}

pub async fn agents_page(State(state): State<Arc<AppState>>) -> Html<String> {
    let agents = match Agent::list(&state.pool).await { Ok(a) => a, Err(_) => vec![] };
    let all_agents_count = agents.len();

    let now = chrono::Utc::now();
    let five_minutes_ago = now - chrono::Duration::minutes(5);
    let agents_html = agents.iter().map(|a| {
        let is_new = a.created_at > five_minutes_ago;
        let avatar_color = avatar_color_for(&a.name);
        let initials = agent_initials(&a.name);
        let stripe_html = if a.stripe_account_id.is_some() {
            r#"<span class="stripe-check">✓ Stripe</span>"#.to_string()
        } else {
            format!(r#"<button class="btn btn-secondary btn-sm" onclick="connectStripe('{}')">Connect Stripe</button>"#, a.id)
        };
        let recent_activity = if a.total_sales > 0 {
            format!("Last sold {} service{}", a.total_sales, if a.total_sales == 1 { "" } else { "s" })
        } else {
            "Just joined the marketplace".to_string()
        };
        format!(
            r#"
            <div class="card agent-card" id="agent-{}">
                {}
                <div class="agent-card-header">
                    <div class="agent-avatar" style="background:{};">{}</div>
                    <div class="agent-card-info">
                        <h3>{}</h3>
                        <p>{}</p>
                    </div>
                </div>
                <div class="agent-recent">{}</div>
                <div class="agent-card-bottom">
                    <div>
                        <div class="balance">${}.{}</div>
                        <div class="balance-label">Balance</div>
                    </div>
                    {}
                </div>
            </div>"#,
            a.id,
            if is_new { r#"<span class="status-dot new-badge"><span class="dot"></span>NEW</span>"# } else { "" },
            avatar_color, initials,
            html_escape(&a.name),
            html_escape(&a.description),
            recent_activity,
            a.balance_cents / 100,
            format_cents(a.balance_cents % 100),
            stripe_html
        )
    }).collect::<String>();

    let connect_script = r#"<script>
    async function connectStripe(agentId) {
        const email = prompt("Enter your email for Stripe Connect:");
        if (!email) return;
        const btn = document.querySelector(`#agent-${agentId} .btn`);
        if (btn) btn.textContent = "Connecting...";
        try {
            const res = await fetch("/api/stripe/connect", {
                method: "POST", headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ agent_id: agentId, email: email })
            });
            const data = await res.json();
            if (data.onboarding_url) window.location.href = data.onboarding_url;
            else { alert("Error: " + JSON.stringify(data.error || data)); if (btn) btn.textContent = "Connect Stripe"; }
        } catch (e) { alert("Error: " + e.message); if (btn) btn.textContent = "Connect Stripe"; }
    }
    </script>"#;

    Html(wrap_page("Agents", &format!(
        r#"<div class="section"><h2>All Agents</h2><div class="grid">{}</div></div>{}"#,
        agents_html, connect_script
    ), all_agents_count))
}

pub async fn transactions_page(State(state): State<Arc<AppState>>) -> Html<String> {
    let transactions = match Transaction::list(&state.pool).await { Ok(t) => t, Err(_) => vec![] };
    let all_agents = match Agent::list(&state.pool).await { Ok(a) => a, Err(_) => vec![] };
    let all_services = match Service::list(&state.pool).await { Ok(s) => s, Err(_) => vec![] };
    let agent_names: std::collections::HashMap<String, String> = all_agents.iter().map(|a| (a.id.clone(), a.name.clone())).collect();
    let service_names: std::collections::HashMap<String, String> = all_services.iter().map(|s| (s.id.clone(), s.name.clone())).collect();

    let tx_html = transactions.iter().map(|t| {
        let icon = match t.status.as_str() {
            "released" | "escrow" => "🛒",
            "pending" => "⏳",
            _ => "📌",
        };
        let status_badge = match t.status.as_str() {
            "released" => r#"<span class="status-badge status-released">Released</span>"#,
            "escrow" => r#"<span class="status-badge status-escrow">Escrow</span>"#,
            "pending" => r#"<span class="status-badge status-pending">Pending</span>"#,
            "disputed" => r#"<span class="status-badge status-disputed">Disputed</span>"#,
            _ => r#"<span class="status-badge status-pending">Pending</span>"#,
        };
        let buyer_name = agent_names.get(&t.buyer_id).map(|n| n.as_str()).unwrap_or(&t.buyer_id[..8.min(t.buyer_id.len())]);
        let seller_name = agent_names.get(&t.seller_id).map(|n| n.as_str()).unwrap_or(&t.seller_id[..8.min(t.seller_id.len())]);
        let service_name = service_names.get(&t.service_id).map(|n| n.as_str()).unwrap_or("Unknown Service");
        format!(
            r#"<div class="tx-row-item" data-type="purchase">
                <div class="tx-row-icon">{}</div>
                <div class="tx-row-body">
                    <div class="tx-desc"><strong>{}</strong> bought <strong>{}</strong> from <strong>{}</strong></div>
                    <div class="tx-time">{}</div>
                </div>
                <div class="tx-row-right">
                    <div class="tx-row-price">${}.{}</div>
                    {}
                </div>
            </div>"#,
            icon,
            html_escape(buyer_name),
            html_escape(service_name),
            html_escape(seller_name),
            time_since(&t.created_at),
            t.amount_cents / 100, format_cents(t.amount_cents % 100),
            status_badge
        )
    }).collect::<String>();

    Html(wrap_page("Transactions", &format!(
        r#"<div class="section">
            <h2>All Transactions</h2>
            <div class="tx-list">{}</div>
        </div>"#,
        if tx_html.is_empty() {
            r#"<div class="empty-state"><div class="empty-state-icon">🌑</div><p>No transactions yet. Run the demo to see agents trade!</p></div>"#.to_string()
        } else { tx_html }
    ), all_agents.len()))
}

pub async fn my_purchases_page(State(state): State<Arc<AppState>>) -> Html<String> {
    let transactions = match Transaction::list(&state.pool).await { Ok(t) => t, Err(_) => vec![] };
    let all_agents = match Agent::list(&state.pool).await { Ok(a) => a, Err(_) => vec![] };
    let all_services = match Service::list(&state.pool).await { Ok(s) => s, Err(_) => vec![] };
    let agent_names: std::collections::HashMap<String, String> = all_agents.iter().map(|a| (a.id.clone(), a.name.clone())).collect();
    let service_names: std::collections::HashMap<String, String> = all_services.iter().map(|s| (s.id.clone(), s.name.clone())).collect();

    let (completed, pending): (Vec<_>, Vec<_>) = transactions.iter().partition(|t| t.status == "released" || t.status == "escrow");
    let tx_rows = |txs: Vec<&Transaction>| -> String {
        txs.into_iter().map(|t| {
            let status_badge = match t.status.as_str() {
                "released" => r#"<span class="status-badge status-released">Released</span>"#,
                "escrow" => r#"<span class="status-badge status-escrow">Escrow</span>"#,
                "pending" => r#"<span class="status-badge status-pending">Pending</span>"#,
                _ => r#"<span class="status-badge status-pending">Pending</span>"#,
            };
            let seller_name = agent_names.get(&t.seller_id).map(|n| n.as_str()).unwrap_or(&t.seller_id[..8.min(t.seller_id.len())]);
            let service_name = service_names.get(&t.service_id).map(|n| n.as_str()).unwrap_or("Unknown Service");
            let action_btn = if t.status == "escrow" || t.status == "released" {
                format!(r#"<a href="/deliverable/{}" class="btn btn-sm">View</a>"#, t.id)
            } else {
                r#"<span style="color:var(--text-dim);font-size:0.8rem;">⏳ Waiting...</span>"#.to_string()
            };
            format!(
                r#"
<div class="tx-row-item">
                    <div class="tx-row-body">
                        <div class="tx-desc"><strong>{}</strong> from {}</div>
                        <div class="tx-time">{}</div>
                    </div>
                    <div class="tx-row-right">
                        <div class="tx-row-price">${}.{}</div>
                        {}
                        {}
                    </div>
                </div>"#,
                html_escape(service_name),
                html_escape(seller_name),
                time_since(&t.created_at),
                t.amount_cents / 100, format_cents(t.amount_cents % 100),
                status_badge, action_btn
            )
        }).collect::<String>()
    };

    let completed_html = tx_rows(completed.iter().map(|t| *t).collect::<Vec<_>>());
    let pending_html = tx_rows(pending.iter().map(|t| *t).collect::<Vec<_>>());

    let purchases_html = if transactions.is_empty() {
        r#"<div class="empty-state"><div class="empty-state-icon">🛒</div><p>No purchases yet. Go to <a href="/services" style="color:var(--accent);">Services</a> and buy something!</p></div>"#.to_string()
    } else if completed.is_empty() && !pending.is_empty() {
        format!(
            r#"<div class="empty-state" style="margin-bottom:1.5rem;"><div class="empty-state-icon">⏳</div><p>No completed purchases yet. {} pending...</p></div>
            <div class="tx-list">{}</div>"#,
            pending.len(), pending_html
        )
    } else {
        format!(
            r#"<div class="tx-list">{}{}</div>"#,
            completed_html, pending_html
        )
    };

    Html(wrap_page("My Purchases", &format!(
        r#"<div class="section"><h2>My Purchases</h2>{}</div>"#,
        purchases_html
    ), all_agents.len()))
}

pub async fn deliverable_page(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Html<String> {
    let tx = match Transaction::get_by_id(&state.pool, &id).await {
        Ok(Some(t)) => t,
        _ => return Html(wrap_page("Not Found", r#"<div class="section"><h2>Transaction not found</h2></div>"#, 0)),
    };

    let deliverable = match Deliverable::get_by_transaction(&state.pool, &id).await {
        Ok(Some(d)) => d,
        _ => return Html(wrap_page("Not Ready", &format!(
            r#"<div class="section"><h2>Delivery in Progress</h2><p style="color:var(--text-dim);">Transaction {} is still being processed. Check back soon.</p></div>"#,
            html_escape(&id[..8.min(id.len())])
        ), 0)),
    };

    let service = match Service::get_by_id(&state.pool, &tx.service_id).await {
        Ok(Some(s)) => s,
        _ => return Html(wrap_page("Error", r#"<div class="section"><h2>Service not found</h2></div>"#, 0)),
    };

    let all_agents = match Agent::list(&state.pool).await { Ok(a) => a, Err(_) => vec![] };

    let output_html = deliverable.output_data.as_ref().map(|output| {
        let escaped = html_escape(output);
        let formatted = escaped.replace("\n\n", "</p><p>").replace("\n", "<br>");
        format!(r#"<div class="deliverable-output"><h3>Completed Work</h3><p>{}</p></div>"#, formatted)
    }).unwrap_or_else(|| r#"<div class="deliverable-output"><p style="color:var(--text-dim);">No output generated yet.</p></div>"#.to_string());

    let review_btn = if tx.status == "released" {
        format!(r#"<div style="margin-top:1.5rem;"><a href="/transactions" class="btn">Back to Transactions</a></div>"#)
    } else {
        r#"<div style="margin-top:1.5rem;"><span style="color:var(--text-dim);">Escrow not yet released. <a href="/transactions" style="color:var(--accent);">View transactions</a></span></div>"#.to_string()
    };

    Html(wrap_page("Deliverable", &format!(
        r#"<div class="section">
            <h2>{}</h2>
            <div class="card">
                <div class="meta" style="margin-bottom:1rem;">
                    Transaction: {} &bull; Status: <span class="status-badge status-{}">{}</span> &bull; Amount: ${}.{}
                </div>
                <div class="meta" style="margin-bottom:1rem;">
                    Service Type: {} &bull; Seller: {} &bull; Delivered: {}
                </div>
                {}
                {}
            </div>
        </div>"#,
        html_escape(&service.name),
        html_escape(&tx.id[..8.min(tx.id.len())]),
        tx.status, tx.status,
        tx.amount_cents / 100, format_cents(tx.amount_cents % 100),
        html_escape(&service.service_type),
        html_escape(&tx.seller_id[..8.min(tx.seller_id.len())]),
        time_since(&deliverable.updated_at),
        output_html, review_btn
    ), all_agents.len()))
}

pub async fn success_page(Query(query): Query<TxQuery>) -> Html<String> {
    let tx_id = query.tx_id.unwrap_or_else(|| "unknown".to_string());
    Html(wrap_page("Success", &format!(
        r#"<div class="section" style="text-align:center;padding:3rem;">
            <h2 style="color:var(--accent);font-size:2rem;">✅ Payment Successful!</h2>
            <p style="color:var(--text-dim);margin:1rem 0;">Transaction ID: <code>{}</code></p>
            <p style="color:var(--text-dim);">Your service is being prepared by the agent.</p>
            <a href="/" class="btn" style="margin-top:1.5rem;">Back to Marketplace</a>
        </div>"#,
        html_escape(&tx_id)
    ), 0))
}

pub async fn cancel_page(Query(query): Query<TxQuery>) -> Html<String> {
    let tx_id = query.tx_id.unwrap_or_else(|| "unknown".to_string());
    Html(wrap_page("Cancelled", &format!(
        r#"<div class="section" style="text-align:center;padding:3rem;">
            <h2 style="color:var(--accent-2);font-size:2rem;">❌ Payment Cancelled</h2>
            <p style="color:var(--text-dim);margin:1rem 0;">Transaction ID: <code>{}</code></p>
            <p style="color:var(--text-dim);">No payment was processed. Try again when ready.</p>
            <a href="/" class="btn" style="margin-top:1.5rem;">Back to Marketplace</a>
        </div>"#,
        html_escape(&tx_id)
    ), 0))
}

pub async fn monitor_page(State(state): State<Arc<AppState>>) -> Html<String> {
    let services = match Service::list_active(&state.pool).await { Ok(s) => s, Err(_) => vec![] };
    let all_agents = match Agent::list(&state.pool).await { Ok(a) => a, Err(_) => vec![] };
    let agent_names: std::collections::HashMap<String, String> = all_agents.iter().map(|a| (a.id.clone(), a.name.clone())).collect();

    let showcases = services.iter().map(|s| {
        let (icon, sample_input, sample_output) = match crate::service_catalog::get_service_definition(&s.service_type) {
            Some(def) => {
                let icon = service_icon(def.service_type);
                let input = if def.example_input.is_empty() { "" } else { def.example_input };
                let output = if def.example_output.is_empty() { "" } else { def.example_output };
                (icon, input, output)
            }
            None => ("🔧", "", ""),
        };
        let has_samples = !sample_input.is_empty() && !sample_output.is_empty();
        let seller_name = agent_names.get(&s.agent_id).map(|n| n.as_str()).unwrap_or("Unknown");
        let model_tag = match crate::service_catalog::get_service_definition(&s.service_type) {
            Some(def) => format!("Model: {}", def.model.model_name()),
            None => "Model: Unknown".to_string(),
        };
        format!(
            r#"<div class="card showcase-card">
                <div class="showcase-header">
                    <span class="showcase-icon">{}</span>
                    <div><h3>{}</h3><div class="showcase-type">{}</div></div>
                </div>
                <p class="showcase-desc">{}</p>
                <div class="showcase-price">${}.{}</div>
                <div class="showcase-meta">
                    <span class="seller">by {}</span>
                    <span class="model-tag">{}</span>
                </div>
                {}
                <div class="showcase-meta" style="margin-top:0.75rem;">
                    <button class="btn btn-sm" onclick="runLiveDemo('{}')">Live Demo</button>
                </div>
            </div>"#,
            icon, html_escape(&s.name), s.service_type, html_escape(&s.description),
            s.price_cents / 100, format_cents(s.price_cents % 100),
            html_escape(seller_name),
            model_tag,
            if has_samples {
                format!(
                    r#"<div class="showcase-sample"><div class="sample-label">Sample Input</div><pre class="sample-code">{}</pre></div>
                     <div class="showcase-sample"><div class="sample-label">Sample Output</div><pre class="sample-code">{}</pre></div>"#,
                    html_escape(sample_input), html_escape(sample_output)
                )
            } else {
                format!(r#"<div class="showcase-empty-msg">Click Live Demo to see this service in action</div>"#)
            },
            s.id
        )
    }).collect::<String>();

    Html(wrap_page("Service Monitor", &format!(
        r#"<div class="section">
            <h2>Service Monitor</h2>
            <p style="color:var(--text-dim);margin-bottom:1.5rem;">See real examples of what each service type produces. Every demonstration uses actual LLM inference or live API calls.</p>
            <div class="showcase-grid">{}</div>
        </div>"#,
        showcases
    ), all_agents.len()))
}

pub async fn agent_loop_page(State(state): State<Arc<AppState>>) -> Html<String> {
    let agents = match Agent::list(&state.pool).await { Ok(a) => a, Err(_) => vec![] };
    let transactions = match Transaction::list(&state.pool).await { Ok(t) => t, Err(_) => vec![] };

    let agent_rows = agents.iter().map(|a| {
        let avatar_color = avatar_color_for(&a.name);
        let initials = agent_initials(&a.name);
        format!(
            r#"<tr>
                <td><div style="display:flex;align-items:center;gap:0.5rem;"><div class="agent-avatar" style="background:{};width:28px;height:28px;font-size:0.7rem;">{}</div> {}</div></td>
                <td>{}</td>
                <td>{}</td>
                <td>${}.{}</td>
                <td>{}</td>
                <td><span class="status-dot active"><span class="dot"></span>Active</span></td>
            </tr>"#,
            avatar_color, initials, html_escape(&a.name), a.total_sales,
            a.reputation_score, a.total_revenue_cents / 100, format_cents(a.total_revenue_cents % 100),
            html_escape(&a.description[..40.min(a.description.len())])
        )
    }).collect::<String>();

    let recent_tx = transactions.iter().take(10).map(|t| {
        let status_class = match t.status.as_str() {
            "escrow" => "status-escrow",
            "released" => "status-released",
            "disputed" => "status-disputed",
            _ => "status-pending",
        };
        format!(
            r#"<tr>
                <td><code>{}</code></td>
                <td>${}.{}</td>
                <td><span class="status-badge {}">{}</span></td>
                <td class="col-time">{}</td>
            </tr>"#,
            html_escape(&t.id[..8.min(t.id.len())]),
            t.amount_cents / 100, format_cents(t.amount_cents % 100),
            status_class, t.status,
            time_since(&t.created_at)
        )
    }).collect::<String>();

    Html(wrap_page("Agent Loop", &format!(
        r#"<div class="section">
            <h2>Agent Loop — Live Autonomous Trading</h2>
            <p style="color:var(--text-dim);margin-bottom:1.5rem;">Watch agents autonomously discover, purchase, and review services. Press <strong>Space</strong> to run a tick.</p>
            <div class="action-bar">
                <button class="btn" onclick="runTick()" style="font-size:1rem;padding:0.6rem 1.5rem;">Run Tick</button>
                <div class="speed-toggle">
                    <button class="active" onclick="setSpeed(1, this)">1x</button>
                    <button onclick="setSpeed(2, this)">2x</button>
                    <button onclick="setSpeed(5, this)">5x</button>
                </div>
                <button class="btn btn-secondary" onclick="resetLoop()">Reset</button>
                <span id="tick-status" class="tick-status"></span>
            </div>
            <div id="tick-results" class="tick-results"></div>
            <div id="event-log" class="event-log" style="margin-top:1rem;"></div>
        </div>
        <div class="section">
            <h2>Active Agents</h2>
            <table class="data-table">
                <thead><tr><th>Agent</th><th>Sales</th><th>Rep</th><th>Revenue</th><th>Description</th><th>Status</th></tr></thead>
                <tbody>{}</tbody>
            </table>
        </div>
        <div class="section">
            <h2>Recent Transactions</h2>
            <table class="data-table">
                <thead><tr><th>ID</th><th>Amount</th><th>Status</th><th>Time</th></tr></thead>
                <tbody>{}</tbody>
            </table>
        </div>
        <script>
        let simSpeed = 1;
        function setSpeed(s, btn) {{
            simSpeed = s;
            document.querySelectorAll('.speed-toggle button').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
        }}
        document.addEventListener('keydown', e => {{
            if (e.code === 'Space' && !e.repeat) {{
                e.preventDefault();
                runTick();
            }}
        }});
        async function runTick() {{
            const status = document.getElementById('tick-status');
            const results = document.getElementById('tick-results');
            const log = document.getElementById('event-log');
            status.textContent = 'Running...';
            try {{
                const resp = await fetch('/api/agents/tick', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ speed: simSpeed }})
                }});
                const data = await resp.json();
                status.textContent = 'Tick complete: ' + (data.count || 0) + ' interactions';
                let html = '<div class="interactions">';
                for (const i of data.interactions || []) {{
                    const cls = i.success ? 'success flash' : 'failed';
                    html += '<div class="interaction ' + cls + '" data-agent="' + (i.agent_id || i.agent) + '" data-service="' + (i.service_id || i.service || '') + '" data-price="' + (i.price_cents || 0) + '">';
                    html += '<span class="int-type">' + i.type + '</span>';
                    html += '<span class="int-agent">' + (i.agent || i.agent_id) + '</span>';
                    html += '<span class="int-msg">' + (i.service ? 'bought ' + i.service : i.message || '') + '</span>';
                    html += '</div>';
                    if (i.success && log) {{
                        const line = document.createElement('div');
                        line.className = 'log-line';
                        const time = new Date().toLocaleTimeString();
                        const agent = i.agent || i.agent_id || 'Unknown';
                        const service = i.service || i.service_id || 'Unknown';
                        const price = i.price_cents ? '$' + (i.price_cents / 100).toFixed(2) : '';
                        line.innerHTML = '<span class="log-time">' + time + '</span> <span class="log-agent">' + agent + '</span> <span class="log-action">' + i.type + '</span> <span class="log-service">' + service + '</span> <span class="log-price">' + price + '</span>';
                        log.appendChild(line);
                        log.scrollTop = log.scrollHeight;
                    }}
                }}
                html += '</div>';
                results.innerHTML = html;
                setTimeout(() => {{
                    document.querySelectorAll('.interaction.flash').forEach(el => el.classList.remove('flash'));
                }}, 800);
            }} catch (e) {{
                status.textContent = 'Error: ' + e.message;
            }}
        }}
        function resetLoop() {{
            document.getElementById('tick-results').innerHTML = '';
            document.getElementById('tick-status').textContent = 'Reset';
            document.getElementById('event-log').innerHTML = '';
        }}
        </script>"#,
        agent_rows, recent_tx
    ), agents.len()))
}

pub async fn activity_page(State(state): State<Arc<AppState>>) -> Html<String> {
    let logs = match crate::models::activity_log::ActivityLog::list_global(&state.pool, 100).await { Ok(l) => l, Err(_) => vec![] };
    let stats = match crate::models::activity_log::ActivityLog::get_stats(&state.pool).await {
        Ok(s) => s,
        Err(_) => crate::models::activity_log::ActivityStats { total_actions: 0, total_purchases: 0, total_reviews: 0, total_services_created: 0, total_volume_cents: 0, top_agent: None },
    };
    let agents = match Agent::list(&state.pool).await { Ok(a) => a, Err(_) => vec![] };

    let log_rows = logs.iter().map(|l| {
        let action_icon = match l.action_type.as_str() {
            "purchase" => "🛒", "create_service" => "✨", "review" => "⭐", "browse" => "👁", _ => "📌",
        };
        let action_class = match l.action_type.as_str() {
            "purchase" => "action-purchase", "create_service" => "action-create", "review" => "action-review", _ => "action-other",
        };
        let amount_html = l.amount_cents.map(|c| format!(r#"<span class="amount">${}.{}</span>"#, c / 100, format_cents(c % 100))).unwrap_or_default();
        let target_link = l.target_id.as_ref().map(|id| {
            if l.target_type.as_deref() == Some("transaction") {
                format!(r#"<a href="/deliverable/{}">{}</a>"#, id, html_escape(&l.target_name.as_deref().unwrap_or(id)))
            } else {
                format!(r#"<a href="/activity?agent={}">{}</a>"#, id, html_escape(&l.target_name.as_deref().unwrap_or(id)))
            }
        }).unwrap_or_default();
        format!(
            r#"<tr class="{}" data-agent="{}" data-type="{}">
                <td class="log-icon">{}</td>
                <td class="log-time">{}</td>
                <td class="log-agent"><a href="/activity?agent={}">{}</a></td>
                <td class="log-action">{}</td>
                <td class="log-target">{}</td>
                <td class="log-amount">{}</td>
                <td class="log-details">{}</td>
            </tr>"#,
            action_class, html_escape(&l.agent_id), l.action_type,
            action_icon, time_since(&l.created_at),
            html_escape(&l.agent_id), html_escape(&l.agent_name),
            l.action_type.replace('_', " "),
            target_link, amount_html,
            l.details.as_deref().map(html_escape).unwrap_or_default()
        )
    }).collect::<String>();

    let agent_options = agents.iter().map(|a| {
        format!(r#"<option value="{}">{}</option>"#, html_escape(&a.id), html_escape(&a.name))
    }).collect::<String>();

    let top_agent_html = match &stats.top_agent {
        Some((name, count)) => {
            let color = avatar_color_for(name);
            let initials = agent_initials(name);
            format!(
                r#"<div class="top-agent-card">
                    <div class="avatar" style="background:{};">{}</div>
                    <div class="name">{}</div>
                    <div class="score">{} actions</div>
                </div>"#,
                color, initials, html_escape(name), count
            )
        }
        None => String::new(),
    };

    let stats_html = format!(
        r#"<div class="stats-grid">
            <div class="stat-card"><div class="value">{}</div><div class="label">Total Actions</div></div>
            <div class="stat-card"><div class="value">{}</div><div class="label">Purchases</div></div>
            <div class="stat-card"><div class="value">{}</div><div class="label">Reviews</div></div>
            <div class="stat-card"><div class="value">{}</div><div class="label">Services Created</div></div>
            <div class="stat-card"><div class="value">${}.{}</div><div class="label">Volume</div></div>
            {}
        </div>"#,
        stats.total_actions, stats.total_purchases, stats.total_reviews, stats.total_services_created,
        stats.total_volume_cents / 100, format_cents(stats.total_volume_cents % 100),
        top_agent_html
    );

    Html(wrap_page("Activity Ledger", &format!(
        r#"<div class="section">
            <h2>Activity Ledger <span class="status-dot active" style="margin-left:0.5rem;"><span class="dot"></span>Real-time</span></h2>
            <p style="color:var(--text-dim);margin-bottom:1.5rem;">Every action, every trade, every service creation — recorded in real-time. Think Etherscan for agents.</p>
            {}
        </div>
        <div class="section">
            <div class="filter-bar" style="display:flex;gap:1rem;margin-bottom:1.5rem;align-items:center;flex-wrap:wrap;">
                <select id="agent-filter" onchange="filterByAgent()" style="background:var(--surface);border:1px solid var(--border);color:var(--text);padding:0.5rem 1rem;border-radius:var(--radius-sm);font-size:0.9rem;cursor:pointer;"><option value="">All Agents</option>{}</select>
                <select id="type-filter" onchange="filterByType()" style="background:var(--surface);border:1px solid var(--border);color:var(--text);padding:0.5rem 1rem;border-radius:var(--radius-sm);font-size:0.9rem;cursor:pointer;">
                    <option value="">All Actions</option>
                    <option value="purchase">Purchases</option>
                    <option value="create_service">Service Creation</option>
                    <option value="review">Reviews</option>
                    <option value="browse">Browses</option>
                </select>
                <button class="btn btn-secondary" onclick="refreshActivity()">Refresh</button>
            </div>
            <table class="data-table activity-table">
                <thead><tr><th></th><th class="col-time">Time</th><th>Agent</th><th>Action</th><th>Target</th><th>Amount</th><th>Details</th></tr></thead>
                <tbody id="activity-body">{}</tbody>
            </table>
            <div id="empty-state" class="empty-state" style="display:none;"><p>No activities match your filters.</p></div>
        </div>
        <script>
        function filterByAgent() {{
            const agent = document.getElementById('agent-filter').value;
            const rows = document.querySelectorAll('#activity-body tr'); let visible = 0;
            for (const row of rows) {{ if (!agent || row.dataset.agent === agent) {{ row.style.display = ''; visible++; }} else {{ row.style.display = 'none'; }} }}
            document.getElementById('empty-state').style.display = visible ? 'none' : 'block';
        }}
        function filterByType() {{
            const type = document.getElementById('type-filter').value;
            const rows = document.querySelectorAll('#activity-body tr'); let visible = 0;
            for (const row of rows) {{ if (!type || row.dataset.type === type) {{ row.style.display = ''; visible++; }} else {{ row.style.display = 'none'; }} }}
            document.getElementById('empty-state').style.display = visible ? 'none' : 'block';
        }}
        async function refreshActivity() {{
            const btn = document.querySelector('.filter-bar .btn'); btn.textContent = 'Loading...';
            try {{ await fetch('/api/activity'); window.location.reload(); }} catch (e) {{ btn.textContent = 'Error'; }}
        }}
        const params = new URLSearchParams(window.location.search);
        const agentFilter = params.get('agent');
        if (agentFilter) {{ document.getElementById('agent-filter').value = agentFilter; filterByAgent(); }}
        </script>"#,
        stats_html, agent_options, log_rows
    ), agents.len()))
}

pub async fn inference_monitor_page(State(state): State<Arc<AppState>>) -> Html<String> {
    let all_agents = match Agent::list(&state.pool).await { Ok(a) => a, Err(_) => vec![] };
    Html(wrap_page("Inference Monitor", r#"<div class="section">
  <h2>🔮 Inference Monitor <span class="live-indicator" style="margin-left:1rem;"><span class="dot"></span> LIVE</span></h2>
  <p style="color:var(--text-dim);margin-bottom:1.5rem;">Real-time model routing across the local LLM fleet</p>

  <div id="active-inferences" class="inference-grid" style="display:grid;grid-template-columns:repeat(auto-fill,minmax(300px,1fr));gap:1rem;margin-bottom:2rem;">
    <p style="color:var(--text-dim);padding:2rem;text-align:center;">Waiting for inference requests...</p>
  </div>

  <h2 style="color:var(--accent-3);margin-top:2rem;">Recent History</h2>
  <table class="data-table" id="history-table" style="width:100%;border-collapse:collapse;background:var(--surface);border-radius:12px;overflow:hidden;margin-top:1rem;">
    <thead>
      <tr>
        <th style="background:var(--surface-2);padding:0.75rem 1rem;text-align:left;font-size:0.8rem;text-transform:uppercase;color:var(--muted);">Time</th>
        <th style="background:var(--surface-2);padding:0.75rem 1rem;text-align:left;font-size:0.8rem;text-transform:uppercase;color:var(--muted);">Service</th>
        <th style="background:var(--surface-2);padding:0.75rem 1rem;text-align:left;font-size:0.8rem;text-transform:uppercase;color:var(--muted);">Model</th>
        <th style="background:var(--surface-2);padding:0.75rem 1rem;text-align:left;font-size:0.8rem;text-transform:uppercase;color:var(--muted);">Tier</th>
        <th style="background:var(--surface-2);padding:0.75rem 1rem;text-align:left;font-size:0.8rem;text-transform:uppercase;color:var(--muted);">Tokens</th>
        <th style="background:var(--surface-2);padding:0.75rem 1rem;text-align:left;font-size:0.8rem;text-transform:uppercase;color:var(--muted);">Duration</th>
      </tr>
    </thead>
    <tbody id="history-body">
    </tbody>
  </table>
</div>
<script>
const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
const wsHost = window.location.host.includes(':') ? window.location.host.replace(':8746',':3000') : window.location.host;
const ws = new WebSocket(wsProtocol + '//' + wsHost + '/ws');
const activeContainer = document.getElementById('active-inferences');
const historyBody = document.getElementById('history-body');

fetch('/api/inference/history')
  .then(r => {
    if (!r.ok) throw new Error('HTTP ' + r.status);
    return r.json();
  })
  .then(data => {
    if (data.history && data.history.length > 0) {
      data.history.reverse().forEach(addHistoryRow);
    }
  })
  .catch(e => {
    console.error('[inference] Failed to load history:', e);
  });

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  if (msg.event === 'InferenceStarted') {
    const card = document.createElement('div');
    card.className = 'card inference-card tier-' + (msg.tier || 'micro');
    card.id = 'inf-' + msg.service_name;
    card.style.cssText = 'border-left:4px solid var(--accent-2);animation:slideIn 0.3s ease-out;';
    card.innerHTML = '<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:0.75rem;"><span style="font-weight:600;font-size:1.05rem;">' + msg.service_name + '</span><span class="tier-badge ' + (msg.tier || 'micro') + '">' + (msg.tier || 'micro') + '</span></div><div style="color:var(--muted);font-size:0.85rem;font-family:var(--mono);margin-bottom:0.5rem;">' + msg.model + '</div><div style="display:flex;gap:1.5rem;font-size:0.85rem;color:var(--muted);"><div>⚡ <span style="color:var(--text);font-weight:500;">Running...</span></div><div>📊 ~' + msg.estimated_tokens + ' tokens</div></div>';
    if (activeContainer.querySelector('p')) activeContainer.innerHTML = '';
    activeContainer.prepend(card);
  }
  if (msg.event === 'InferenceCompleted') {
    const card = document.getElementById('inf-' + msg.service_name);
    if (card) {
      card.querySelector('.metric-value')?.textContent = msg.duration_ms + 'ms';
      card.style.opacity = '0.7';
      setTimeout(() => card.remove(), 3000);
    }
    addHistoryRow({
      timestamp: Math.floor(Date.now() / 1000),
      service_name: msg.service_name,
      model: msg.model,
      tier: inferTier(msg.model),
      input_tokens: msg.actual_tokens,
      duration_ms: msg.duration_ms
    });
  }
  if (msg.event === 'ModelFallback') {
    addHistoryRow({
      timestamp: Math.floor(Date.now() / 1000),
      service_name: 'fallback',
      model: msg.requested,
      tier: 'local',
      input_tokens: 0,
      duration_ms: 0,
      status: 'fallback'
    });
  }
};

function inferTier(model) {
  const m = model.toLowerCase();
  if (m.includes('9b')) return 'micro';
  if (m.includes('12b') || m.includes('35b')) return 'real';
  if (m.includes('26b') || m.includes('reasoning')) return 'heavy';
  return 'local';
}

function addHistoryRow(record) {
  const row = document.createElement('tr');
  const time = new Date(record.timestamp * 1000).toLocaleTimeString();
  const tier = record.tier || 'local';
  const tierColors = { micro: 'var(--accent)', real: 'var(--accent-3)', heavy: 'var(--accent-2)', local: 'var(--success)' };
  row.innerHTML = '<td style="padding:0.75rem 1rem;border-top:1px solid var(--surface-2);font-size:0.9rem;">' + time + '</td><td style="padding:0.75rem 1rem;border-top:1px solid var(--surface-2);font-size:0.9rem;">' + record.service_name + '</td><td style="padding:0.75rem 1rem;border-top:1px solid var(--surface-2);font-size:0.9rem;"><code>' + record.model + '</code></td><td style="padding:0.75rem 1rem;border-top:1px solid var(--surface-2);font-size:0.9rem;"><span style="font-size:0.7rem;text-transform:uppercase;letter-spacing:0.05em;padding:0.2rem 0.5rem;border-radius:4px;font-weight:600;background:' + tierColors[tier] + '22;color:' + tierColors[tier] + ';">' + tier + '</span></td><td style="padding:0.75rem 1rem;border-top:1px solid var(--surface-2);font-size:0.9rem;">' + (record.input_tokens || '-') + '</td><td style="padding:0.75rem 1rem;border-top:1px solid var(--surface-2);font-size:0.9rem;">' + record.duration_ms + 'ms</td>';
  historyBody.prepend(row);
  if (historyBody.children.length > 20) historyBody.lastChild.remove();
}
</script>
<style>
.inference-card { border-left: 4px solid var(--accent-2); }
.inference-card.tier-micro { border-left-color: var(--accent); }
.inference-card.tier-real { border-left-color: var(--accent-3); }
.inference-card.tier-heavy { border-left-color: var(--accent-2); }
.inference-card.tier-local { border-left-color: var(--success); }
.tier-badge { font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.05em; padding: 0.2rem 0.5rem; border-radius: 4px; font-weight: 600; }
.tier-badge.micro { background: rgba(0,240,255,0.15); color: var(--accent); }
.tier-badge.real { background: rgba(255,190,11,0.15); color: var(--accent-3); }
.tier-badge.heavy { background: rgba(255,0,110,0.15); color: var(--accent-2); }
.tier-badge.local { background: rgba(0,255,136,0.15); color: var(--success); }
@keyframes slideIn { from { opacity: 0; transform: translateY(-10px); } to { opacity: 1; transform: translateY(0); } }
</style>"#, all_agents.len()))
}

pub async fn inference_history_api(State(state): State<Arc<AppState>>) -> axum::Json<serde_json::Value> {
    let history = state.llm.get_inference_history().await;
    axum::Json(serde_json::json!({ "history": history }))
}

/* ─────────── HELPERS ─────────── */

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn service_icon(service_type: &str) -> &'static str {
    match service_type {
        "text_processing" => "📝",
        "data_formatting" => "📊",
        "api_monitor" => "📡",
        "code_review" => "💻",
        "creative_writing" => "✨",
        "analysis" => "🔍",
        "full_repo_analysis" => "📁",
        "uncensored_threat_analysis" => "🛡",
        "massive_context_qa" => "❓",
        "multi_model_ensemble" => "🎯",
        "adversarial_red_team" => "🔴",
        "bulk_document_processing" => "📚",
        "realtime_log_analysis" => "📋",
        "test_generator" => "🧪",
        "financial_forensic_analysis" => "💰",
        "architecture_review" => "🏗",
        "custom_model_inference" => "🧠",
        "legacy_modernize" => "🔄",
        "markdown_table" => "📋",
        _ => "🔧",
    }
}

fn format_cents(cents: i64) -> String {
    format!("{:02}", cents)
}

fn time_since(dt: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let diff = now - *dt;
    if diff.num_seconds() < 60 { "just now".to_string() }
    else if diff.num_minutes() < 60 { format!("{}m ago", diff.num_minutes()) }
    else if diff.num_hours() < 24 { format!("{}h ago", diff.num_hours()) }
    else { format!("{}d ago", diff.num_days()) }
}

fn avatar_color_for(name: &str) -> String {
    let colors = [
        "#e63946", "#f4a261", "#2a9d8f", "#264653", "#e76f51",
        "#8338ec", "#3a86ff", "#06d6a0", "#ef476f", "#ffd166",
        "#118ab2", "#073b4c", "#c77dff", "#7209b7", "#560bad",
    ];
    let hash = name.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
    colors[(hash % colors.len() as u64) as usize].to_string()
}

fn agent_initials(name: &str) -> String {
    name.split_whitespace()
        .filter(|w| !w.is_empty())
        .take(2)
        .map(|w| w.chars().next().unwrap_or('?').to_uppercase().to_string())
        .collect()
}
