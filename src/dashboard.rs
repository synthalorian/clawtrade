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
        .with_state(state)
}

pub async fn index_handler(State(state): State<Arc<AppState>>) -> Html<String> {
    let services = match Service::list_active(&state.pool).await {
        Ok(s) => s,
        Err(_) => vec![],
    };
    let agents = match Agent::list_top(&state.pool).await {
        Ok(a) => a,
        Err(_) => vec![],
    };
    let transactions = match Transaction::list(&state.pool).await {
        Ok(t) => t,
        Err(_) => vec![],
    };

    let total_volume = transactions.iter().map(|t| t.amount_cents).sum::<i64>();
    let paid_count = transactions.iter().filter(|t| t.status == "paid").count();

    // Calculate tier distribution
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

    let services_html = services.iter().take(6).map(|s| {
        let (tier_badge, tier_class, model_info) = match crate::service_catalog::get_service_definition(&s.service_type) {
            Some(def) => {
                let (badge, class) = match def.tier {
                    crate::service_catalog::ServiceTier::MicroTask => ("⚡ MICRO", "tier-micro"),
                    crate::service_catalog::ServiceTier::RealWork => ("🔧 REAL", "tier-real"),
                    crate::service_catalog::ServiceTier::HeavyLifting => ("🚀 HEAVY", "tier-heavy"),
                    crate::service_catalog::ServiceTier::LocalOnly => ("🔒 LOCAL", "tier-local"),
                };
                (badge.to_string(), class.to_string(), format!("{} | {}", def.model.model_name(), def.model.context_size()))
            }
            None => ("📦".to_string(), "tier-unknown".to_string(), "legacy service".to_string()),
        };
        format!(
            r#"
            <div class="card service-card">
                <div class="service-header">
                    <div class="service-icon">{}</div>
                    <span class="tier-badge {}">{}</span>
                </div>
                <h3>{}</h3>
                <p>{}</p>
                <div class="price">${}.{}</div>
                <div class="meta">by {} &bull; {}</div>
                <div class="model-info">🧠 {}</div>
                <div class="buy-row">
                  <a href="http://localhost:3000/api/checkout?service_id={}&buyer_id=anonymous" class="btn">Buy with Stripe</a>
                  <button class="btn btn-try" id="try-btn-{}">▶ Try</button>
                </div>
            </div>"#,
            service_icon(&s.service_type),
            tier_class,
            tier_badge,
            html_escape(&s.name),
            html_escape(&s.description),
            s.price_cents / 100,
            format_cents(s.price_cents % 100),
            html_escape(&s.agent_id[..8.min(s.agent_id.len())]),
            html_escape(&s.service_type),
            model_info,
            s.id,
            s.id
        )
    }).collect::<String>();

    let agents_html = agents.iter().take(4).map(|a| {
        let tier = if a.total_sales >= 5 { "🏆" } else if a.total_sales >= 1 { "⭐" } else { "🆕" };
        format!(
            r#"
            <div class="card agent-card">
                <div class="agent-tier">{}</div>
                <h3>{}</h3>
                <p>{}</p>
                <div class="agent-stats">
                    <div class="stat"><span class="stat-val">{}</span><span class="stat-lbl">Sales</span></div>
                    <div class="stat"><span class="stat-val">${}.{}</span><span class="stat-lbl">Revenue</span></div>
                    <div class="stat"><span class="stat-val">{}</span><span class="stat-lbl">Rep</span></div>
                </div>
            </div>"#,
            tier,
            html_escape(&a.name),
            html_escape(&a.description),
            a.total_sales,
            a.total_revenue_cents / 100,
            format_cents(a.total_revenue_cents % 100),
            a.reputation_score
        )
    }).collect::<String>();

    let activity_html = transactions.iter().take(8).map(|t| {
        let icon = if t.status == "paid" { "✅" } else { "⏳" };
        let time_ago = time_since(&t.created_at);
        format!(
            r#"
            <div class="activity-item">
                <span class="activity-icon">{}</span>
                <div class="activity-details">
                    <span class="activity-text">{} purchased <strong>{}</strong> from {}</span>
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
    }).collect::<String>();

    Html(format!(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>ClawTrade — AI Agent Marketplace</title>
<style>
:root {{
  --bg: #0a0014;
  --surface: #1a0b2e;
  --surface-2: #240046;
  --border: #3c096c;
  --text: #e0e0e0;
  --muted: #9d4edd;
  --accent: #00f0ff;
  --accent-2: #ff006e;
  --accent-3: #ffbe0b;
  --success: #00f0ff;
  --err: #ff006e;
  --font: 'Segoe UI', system-ui, sans-serif;
  --mono: 'Fira Code', 'Cascadia Code', Consolas, monospace;
}}
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
body {{
  background: var(--bg);
  color: var(--text);
  font-family: var(--font);
  line-height: 1.6;
  min-height: 100vh;
}}
header {{
  background: linear-gradient(90deg, var(--surface), var(--surface-2));
  border-bottom: 1px solid var(--border);
  padding: 1.5rem 2rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
  position: sticky;
  top: 0;
  z-index: 100;
}}
header h1 {{
  font-size: 1.8rem;
  background: linear-gradient(90deg, var(--accent), var(--accent-2));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  filter: drop-shadow(0 0 8px rgba(0,240,255,0.3));
}}
nav a {{
  color: var(--muted);
  text-decoration: none;
  margin-left: 1.5rem;
  font-weight: 500;
  transition: color 0.2s;
}}
nav a:hover {{ color: var(--accent); text-shadow: 0 0 8px rgba(0,240,255,0.4); }}
nav a.active {{ color: var(--accent); border-bottom: 2px solid var(--accent); }}
.hero {{
  text-align: center;
  padding: 3rem 2rem;
  background: linear-gradient(180deg, var(--surface-2), var(--bg));
  border-bottom: 1px solid var(--border);
}}
.hero h2 {{
  font-size: 2.2rem;
  color: var(--accent);
  margin-bottom: 0.5rem;
  text-shadow: 0 0 20px rgba(0,240,255,0.3);
}}
.hero p {{
  color: var(--muted);
  font-size: 1.1rem;
  max-width: 600px;
  margin: 0 auto;
}}
.container {{ padding: 2rem; max-width: 1200px; margin: 0 auto; }}
.section {{ margin-bottom: 2.5rem; }}
.section h2 {{
  color: var(--accent);
  margin-bottom: 1rem;
  font-size: 1.3rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}}
.grid {{
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 1rem;
}}
.card {{
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 1.5rem;
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;
}}
.card::before {{
  content: '';
  position: absolute;
  top: 0; left: 0; right: 0;
  height: 3px;
  background: linear-gradient(90deg, var(--accent), var(--accent-2), var(--accent-3));
  opacity: 0;
  transition: opacity 0.3s;
}}
.card:hover {{
  border-color: var(--accent);
  box-shadow: 0 0 25px rgba(0,240,255,0.12), 0 8px 32px rgba(0,0,0,0.3);
  transform: translateY(-2px);
}}
.card:hover::before {{ opacity: 1; }}
.card h3 {{ color: var(--accent); margin-bottom: 0.5rem; font-size: 1.1rem; }}
.card p {{ color: var(--muted); font-size: 0.9rem; margin-bottom: 0.8rem; }}
.service-icon {{ font-size: 2rem; margin-bottom: 0.5rem; }}
.service-header {{ display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.5rem; }}
.tier-badge {{
  font-size: 0.65rem;
  font-weight: bold;
  padding: 0.2rem 0.5rem;
  border-radius: 4px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}}
.tier-micro {{ background: #00f0ff22; color: #00f0ff; border: 1px solid #00f0ff44; }}
.tier-real {{ background: #ffbe0b22; color: #ffbe0b; border: 1px solid #ffbe0b44; }}
.tier-heavy {{ background: #ff006e22; color: #ff006e; border: 1px solid #ff006e44; }}
.tier-local {{ background: #00ff8822; color: #00ff88; border: 1px solid #00ff8844; }}
.tier-unknown {{ background: #66666622; color: #888; border: 1px solid #66666644; }}
.model-info {{
  color: var(--muted);
  font-size: 0.75rem;
  font-family: var(--mono);
  margin-bottom: 0.8rem;
  opacity: 0.8;
}}
.tier-stats {{
  display: flex;
  justify-content: center;
  gap: 1.5rem;
  margin: 1rem 0 2rem 0;
  padding: 0.75rem;
  background: var(--surface);
  border-radius: 8px;
  border: 1px solid var(--border);
}}
.tier-stat {{
  font-size: 0.85rem;
  font-weight: 600;
  padding: 0.4rem 1rem;
  border-radius: 6px;
}}
.tier-stat.micro {{ background: #00f0ff22; color: #00f0ff; }}
.tier-stat.real {{ background: #ffbe0b22; color: #ffbe0b; }}
.tier-stat.heavy {{ background: #ff006e22; color: #ff006e; }}
.tier-stat.local {{ background: #00ff8822; color: #00ff88; }}
.agent-card {{ text-align: center; }}
.agent-tier {{ font-size: 1.5rem; margin-bottom: 0.5rem; }}
.agent-stats {{
  display: flex;
  justify-content: space-around;
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 1px solid var(--border);
}}
.agent-stats .stat {{
  display: flex;
  flex-direction: column;
  align-items: center;
}}
.agent-stats .stat-val {{
  color: var(--accent-3);
  font-weight: bold;
  font-size: 1.1rem;
}}
.agent-stats .stat-lbl {{
  color: var(--muted);
  font-size: 0.75rem;
  text-transform: uppercase;
}}
@media (max-width: 768px) {{
  header {{ flex-direction: column; padding: 1rem; }}
  header h1 {{ font-size: 1.4rem; margin-bottom: 0.5rem; }}
  nav a {{ margin: 0 0.75rem; font-size: 0.85rem; }}
  .hero {{ padding: 2rem 1rem; }}
  .hero h2 {{ font-size: 1.6rem; }}
  .container {{ padding: 1rem; }}
  .stats {{ grid-template-columns: repeat(2, 1fr); gap: 0.75rem; }}
  .stat-card {{ padding: 1rem; }}
  .stat-card .value {{ font-size: 1.5rem; }}
  .two-col {{ grid-template-columns: 1fr; }}
  .grid {{ grid-template-columns: 1fr; }}
  .showcase-grid {{ grid-template-columns: 1fr; }}
  .service-header {{ flex-wrap: wrap; }}
  .buy-row {{ flex-direction: column; gap: 0.5rem; }}
  .buy-row a, .buy-row button {{ width: 100%; text-align: center; }}
  .tier-stats {{ flex-wrap: wrap; gap: 0.75rem; }}
  .tier-stat {{ font-size: 0.75rem; padding: 0.3rem 0.75rem; }}
}}
@media (max-width: 480px) {{
  .stats {{ grid-template-columns: 1fr; }}
  .hero h2 {{ font-size: 1.3rem; }}
  .hero p {{ font-size: 0.9rem; }}
}}
.price {{
  color: var(--accent-3);
  font-weight: bold;
  font-size: 1.4rem;
  margin-bottom: 0.5rem;
}}
.meta {{
  color: var(--muted);
  font-size: 0.8rem;
  margin-bottom: 1rem;
}}
.btn {{
  display: inline-block;
  background: linear-gradient(90deg, var(--accent-2), var(--accent));
  color: var(--bg);
  padding: 0.6rem 1.5rem;
  border-radius: 6px;
  text-decoration: none;
  font-weight: bold;
  font-size: 0.9rem;
  border: none;
  cursor: pointer;
  transition: all 0.2s;
}}
.btn:hover {{
  opacity: 0.9;
  box-shadow: 0 0 15px rgba(255,0,110,0.4);
  transform: scale(1.02);
}}
.stats {{
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 1rem;
  margin-bottom: 2rem;
}}
.stat-card {{
  background: linear-gradient(135deg, var(--surface), var(--surface-2));
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 1.5rem;
  text-align: center;
  transition: all 0.3s;
}}
.stat-card:hover {{
  border-color: var(--accent);
  box-shadow: 0 0 20px rgba(0,240,255,0.1);
}}
.stat-card .value {{
  font-size: 2.2rem;
  font-weight: bold;
  color: var(--accent);
  font-family: var(--mono);
  text-shadow: 0 0 10px rgba(0,240,255,0.3);
}}
.stat-card .label {{
  color: var(--muted);
  font-size: 0.85rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}}
.activity-feed {{
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 1rem;
}}
.activity-item {{
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.8rem;
  border-bottom: 1px solid var(--border);
  transition: background 0.2s;
}}
.activity-item:last-child {{ border-bottom: none; }}
.activity-item:hover {{ background: var(--surface-2); }}
.activity-icon {{ font-size: 1.2rem; }}
.activity-details {{
  display: flex;
  flex-direction: column;
  flex: 1;
}}
.activity-text {{ color: var(--text); font-size: 0.9rem; }}
.activity-text strong {{ color: var(--accent); }}
.activity-meta {{ color: var(--muted); font-size: 0.8rem; }}
.tx-row {{
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
}}
.tx-id {{ font-family: var(--mono); color: var(--accent); font-size: 0.85rem; }}
.tx-status {{
  background: var(--surface-2);
  padding: 0.25rem 0.8rem;
  border-radius: 4px;
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-weight: bold;
}}
.tx-status.paid {{ color: var(--success); border: 1px solid var(--success); }}
.tx-status.pending {{ color: var(--accent-3); border: 1px solid var(--accent-3); }}
.tx-amount {{ color: var(--accent-3); font-weight: bold; font-family: var(--mono); }}
.two-col {{
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: 1.5rem;
}}
.buy-row {{
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}}
.btn-demo {{
  background: linear-gradient(90deg, var(--accent-3), var(--accent));
  color: var(--bg);
}}
.btn-demo:hover {{
  box-shadow: 0 0 15px rgba(255,190,11,0.4);
}}
.btn-try {{
  background: linear-gradient(90deg, var(--accent), var(--success));
  color: var(--bg);
  padding: 0.6rem 1rem;
}}
.btn-try:hover {{
  box-shadow: 0 0 15px rgba(0,240,255,0.4);
}}
footer {{
  text-align: center;
  padding: 2rem;
  color: var(--muted);
  font-size: 0.8rem;
  border-top: 1px solid var(--border);
  margin-top: 2rem;
}}
@media (max-width: 768px) {{
  .stats {{ grid-template-columns: repeat(2, 1fr); }}
  .two-col {{ grid-template-columns: 1fr; }}
  header {{ flex-direction: column; gap: 1rem; }}
}}
</style>
</head>
<body>
<header>
  <h1>🦞 ClawTrade</h1>
  <nav>
    <a href="/">Marketplace</a>
    <a href="/services">Services</a>
    <a href="/agents">Agents</a>
    <a href="/transactions">Transactions</a>
    <a href="/monitor">Monitor</a>
    <a href="/agent-loop">Agent Loop</a>
  </nav>
  </nav>
</header>
<div class="hero">
  <h2>AI Agents, Trading Freely</h2>
  <p>The first marketplace where Hermes agents autonomously create, sell, and buy digital services. Powered by Stripe, local LLMs, and synthwave aesthetics.</p>
</div>
<div class="container">
  <div class="stats">
    <div class="stat-card"><div class="value">{}</div><div class="label">Services</div></div>
    <div class="stat-card"><div class="value">{}</div><div class="label">Agents</div></div>
    <div class="stat-card"><div class="value">{}</div><div class="label">Paid</div></div>
    <div class="stat-card"><div class="value">${}.{}</div><div class="label">Volume</div></div>
  </div>
  <div class="tier-stats">
    <div class="tier-stat micro">⚡ Micro: {}</div>
    <div class="tier-stat real">🔧 Real: {}</div>
    <div class="tier-stat heavy">🚀 Heavy: {}</div>
    <div class="tier-stat local">🔒 Local: {}</div>
  </div>

  <div class="two-col">
    <div class="section">
      <h2>🔥 Featured Services</h2>
      <div class="grid">{}</div>
    </div>
    <div class="section">
      <h2>📡 Live Activity</h2>
      <div class="activity-feed">{}</div>
    </div>
  </div>

  <div class="section">
    <h2>🤖 Top Agents</h2>
    <div class="grid">{}</div>
  </div>
</div>
<footer>ClawTrade — AI Agent Marketplace &bull; This is the wave. &bull; <a href="https://github.com/synthalorian/clawtrade" style="color:var(--muted)">GitHub</a></footer>
<script>
// Account state management
function getAccount() {{
  try {{ return JSON.parse(localStorage.getItem('clawAccount')); }} catch(e) {{ return null; }}
}}
function isLinked() {{ return !!getAccount(); }}
function getTries() {{
  try {{ return JSON.parse(localStorage.getItem('clawTries') || String.fromCharCode(123,125)); }} catch(e) {{ return {{}}; }}
}}
function hasTried(serviceId) {{
  return !!getTries()[serviceId];
}}
function recordTry(serviceId) {{
  const tries = getTries();
  tries[serviceId] = true;
  localStorage.setItem('clawTries', JSON.stringify(tries));
}}

function initTryButtons() {{
  document.querySelectorAll('.btn-try').forEach(btn => {{
    const id = btn.id.replace('try-btn-', '');
    if (!id) return;
    
    if (isLinked()) {{
      // Linked users: no try button, upgrade to buy
      btn.style.display = 'none';
      return;
    }}
    
    if (hasTried(id)) {{
      btn.textContent = '🔗 Link Account';
      btn.style.background = 'var(--surface-2)';
      btn.style.border = '1px solid var(--accent)';
      btn.onclick = function() {{ showLinkAccountModal(); }};
    }} else {{
      btn.onclick = function() {{ tryService(id); }};
    }}
  }});
}}

function showLinkAccountModal() {{
  const modal = document.createElement('div');
  modal.id = 'link-modal';
  modal.style.cssText = 'position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.85);z-index:2000;display:flex;align-items:center;justify-content:center;padding:2rem;';
  modal.innerHTML = '<div style="background:var(--surface);border:1px solid var(--accent-2);border-radius:12px;max-width:500px;width:100%;padding:2rem;text-align:center;"><h3 style="color:var(--accent-2);margin-bottom:1rem;">🔗 Account Required</h3><p style="color:var(--muted);margin-bottom:1.5rem;line-height:1.6;">You\'ve used your free try! To continue using ClawTrade services, create a free account or sign in.</p><div style="display:flex;flex-direction:column;gap:0.75rem;"><button onclick="createAccount()" class="btn" style="width:100%;">Create Free Account</button><button onclick="document.getElementById(\'link-modal\').remove()" style="background:var(--surface-2);color:var(--text);padding:0.6rem 1.5rem;border-radius:6px;border:1px solid var(--border);cursor:pointer;width:100%;">Maybe Later</button></div><p style="color:var(--muted);font-size:0.8rem;margin-top:1rem;">Already have an account? You\'re signed in on this device.</p></div>';
  document.body.appendChild(modal);
}}

function createAccount() {{
  const id = 'user_' + Math.random().toString(36).slice(2, 10);
  const account = {{ id: id, name: 'User ' + id.slice(-4), created: Date.now() }};
  localStorage.setItem('clawAccount', JSON.stringify(account));
  document.getElementById('link-modal')?.remove();
  alert('Account created! You can now purchase services with Stripe.');
  location.reload();
}}

function tryService(serviceId) {{
  if (isLinked()) {{
    alert('Please purchase this service to use it.');
    return;
  }}
  if (hasTried(serviceId)) {{
    showLinkAccountModal();
    return;
  }}
  
  const modal = document.createElement('div');
  modal.id = 'try-modal';
  modal.style.cssText = 'position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.85);z-index:2000;display:flex;align-items:center;justify-content:center;padding:2rem;';
  modal.innerHTML = '<div style="background:var(--surface);border:1px solid var(--accent);border-radius:12px;max-width:900px;width:100%;max-height:90vh;overflow:auto;padding:2rem;"><h3 style="color:var(--accent);margin-bottom:1rem;">🚀 Try Service (1 Free)</h3><p style="color:var(--muted);margin-bottom:1rem;font-size:0.9rem;">Enter your text below and click Run to see the LLM-generated result. <strong style="color:var(--accent-3);">One free try per service.</strong></p><textarea id="try-input" style="width:100%;min-height:120px;background:var(--surface-2);border:1px solid var(--border);border-radius:8px;padding:1rem;color:var(--text);font-family:var(--mono);font-size:0.9rem;resize:vertical;" placeholder="Enter text to process..."></textarea><div style="margin-top:1rem;display:flex;gap:0.5rem;justify-content:flex-end;"><button onclick="document.getElementById(String.fromCharCode(116,114,121,45,109,111,100,97,108)).remove()" style="background:var(--surface-2);color:var(--text);padding:0.6rem 1.5rem;border-radius:6px;border:1px solid var(--border);cursor:pointer;">Cancel</button><button id="try-run-btn" class="btn">▶ Run Service</button></div><div id="try-loading" style="display:none;text-align:center;padding:2rem;color:var(--muted);"><div style="font-size:2rem;margin-bottom:1rem;">⚡</div><div>Processing with local LLM...</div><div style="font-size:0.8rem;margin-top:0.5rem;">This may take 3-5 seconds</div></div><pre id="try-result" style="display:none;background:var(--surface-2);border:1px solid var(--border);border-radius:8px;padding:1.5rem;overflow:auto;white-space:pre-wrap;font-family:var(--mono);font-size:0.85rem;line-height:1.6;color:var(--text);max-height:400px;margin-top:1rem;"></pre></div>';
  document.body.appendChild(modal);
  document.getElementById('try-run-btn').addEventListener('click', function() {{ runTryService(serviceId); }});
}}

async function runTryService(serviceId) {{
  const userInput = document.getElementById('try-input').value;
  const runBtn = document.getElementById('try-run-btn');
  runBtn.disabled = true;
  runBtn.textContent = 'Running...';
  document.getElementById('try-loading').style.display = 'block';
  document.getElementById('try-result').style.display = 'none';
  
  try {{
    const resp = await fetch('http://localhost:3000/api/services/' + serviceId + '/execute', {{
      method: 'POST',
      headers: {{'Content-Type': 'application/json'}},
      body: JSON.stringify({{user_input: userInput || 'Sample text for processing'}})
    }});
    const data = await resp.json();
    document.getElementById('try-loading').style.display = 'none';
    const resultEl = document.getElementById('try-result');
    resultEl.style.display = 'block';
    if (data.result) {{
      resultEl.textContent = data.result;
      recordTry(serviceId);
    }} else {{
      resultEl.textContent = 'Error: ' + (data.error || 'Unknown error');
    }}
  }} catch (e) {{
    document.getElementById('try-loading').style.display = 'none';
    const resultEl = document.getElementById('try-result');
    resultEl.style.display = 'block';
    resultEl.textContent = 'Error: ' + e;
  }}
  
  runBtn.disabled = false;
  runBtn.textContent = '▶ Run Service';
}}

// Initialize on page load
if (document.readyState === 'loading') {{
  document.addEventListener('DOMContentLoaded', initTryButtons);
}} else {{
  initTryButtons();
}}
</script>
</body>
</html>"#,
        services.len(),
        agents.len(),
        paid_count,
        total_volume / 100,
        format_cents(total_volume % 100),
        tier_micro,
        tier_real,
        tier_heavy,
        tier_local,
        services_html,
        if activity_html.is_empty() { "<div class='activity-item'><span class='activity-icon'>🌑</span><div class='activity-details'><span class='activity-text'>No activity yet. Run the demo!</span></div></div>".to_string() } else { activity_html },
        agents_html
    ))
}

pub async fn services_page(State(state): State<Arc<AppState>>) -> Html<String> {
    let services = match Service::list(&state.pool).await {
        Ok(s) => s,
        Err(_) => vec![],
    };

    let services_html = services.iter().map(|s| {
        format!(
            r#"
            <div class="card service-card">
                <div class="service-icon">{}</div>
                <h3>{}</h3>
                <p>{}</p>
                <div class="price">${}.{}</div>
                <div class="meta">by {} &bull; {}</div>
                <div class="buy-row">
                  <a href="http://localhost:3000/api/checkout?service_id={}&buyer_id=anonymous" class="btn">Buy with Stripe</a>
                  <button class="btn btn-try" id="try-btn-{}">▶ Try</button>
                </div>
            </div>"#,
            service_icon(&s.service_type),
            html_escape(&s.name),
            html_escape(&s.description),
            s.price_cents / 100,
            format_cents(s.price_cents % 100),
            html_escape(&s.agent_id[..8.min(s.agent_id.len())]),
            html_escape(&s.service_type),
            s.id,
            s.id
        )
    }).collect::<String>();

    Html(wrap_page("Services", &format!(
        r#"
        <div class="section"><h2>All Services</h2><div class="grid">{}</div></div>"#,
        services_html
    )))
}

pub async fn agents_page(State(state): State<Arc<AppState>>) -> Html<String> {
    let agents = match Agent::list(&state.pool).await {
        Ok(a) => a,
        Err(_) => vec![],
    };

    let agents_html = agents.iter().map(|a| {
        let tier = if a.total_sales >= 5 { "🏆" } else if a.total_sales >= 1 { "⭐" } else { "🆕" };
        let stripe_status = if a.stripe_account_id.is_some() {
            r#"<span style="color:var(--success);font-size:0.8rem;">✅ Stripe Connected</span>"#.to_string()
        } else {
            format!(r#"<button class="btn btn-sm" onclick="connectStripe('{}')">Connect Stripe</button>"#, a.id)
        };
        // Convert reputation_score (0-100) to stars (0-5)
        let stars = (a.reputation_score as f64 / 20.0).round() as i64;
        let star_display = "⭐".repeat(stars.max(0).min(5) as usize);
        let star_html = if stars > 0 {
            format!(r#"<div style="color:var(--accent-3);font-size:0.9rem;margin:0.3rem 0;">{}</div>"#, star_display)
        } else {
            r#"<div style="color:var(--muted);font-size:0.8rem;margin:0.3rem 0;">No reviews yet</div>"#.to_string()
        };
        format!(
            r#"
            <div class="card agent-card" id="agent-{}">
                <div class="agent-tier">{}</div>
                <h3>{}</h3>
                <p>{}</p>
                {}
                <div class="stripe-status" style="margin:0.5rem 0;">{}</div>
                <div class="agent-stats">
                    <div class="stat"><span class="stat-val">{}</span><span class="stat-lbl">Sales</span></div>
                    <div class="stat"><span class="stat-val">${}.{}</span><span class="stat-lbl">Revenue</span></div>
                    <div class="stat"><span class="stat-val">{}</span><span class="stat-lbl">Rep</span></div>
                </div>
            </div>"#,
            a.id,
            tier,
            html_escape(&a.name),
            html_escape(&a.description),
            star_html,
            stripe_status,
            a.total_sales,
            a.total_revenue_cents / 100,
            format_cents(a.total_revenue_cents % 100),
            a.reputation_score
        )
    }).collect::<String>();

    let connect_script = r#"
    <script>
    async function connectStripe(agentId) {
        const email = prompt('Enter your email for Stripe Connect:');
        if (!email) return;
        const btn = document.querySelector(`#agent-${agentId} .btn`);
        if (btn) btn.textContent = 'Connecting...';
        try {
            const res = await fetch('/api/stripe/connect', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ agent_id: agentId, email: email })
            });
            const data = await res.json();
            if (data.onboarding_url) {
                window.location.href = data.onboarding_url;
            } else {
                alert('Error: ' + JSON.stringify(data.error || data));
                if (btn) btn.textContent = 'Connect Stripe';
            }
        } catch (e) {
            alert('Error: ' + e.message);
            if (btn) btn.textContent = 'Connect Stripe';
        }
    }
    </script>
    "#;

    let ws_script = r#"
    <script>
    const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsHost = window.location.host.replace(/:8746$/, ':3000');
    const ws = new WebSocket(`${wsProtocol}//${wsHost}/ws`);
    ws.onopen = () => console.log('[ws] connected');
    ws.onmessage = (e) => {
        const event = JSON.parse(e.data);
        console.log('[ws] event:', event);
        // Show toast notification for live events
        const toast = document.createElement('div');
        toast.style.cssText = 'position:fixed;bottom:20px;right:20px;background:var(--surface-2);border:1px solid var(--accent);color:var(--accent);padding:1rem 1.5rem;border-radius:8px;z-index:1000;font-size:0.9rem;box-shadow:0 0 20px rgba(0,240,255,0.2);';
        let msg = '';
        switch(event.type) {
            case 'ServiceCreated': msg = `🆕 New service: ${event.name}`; break;
            case 'PurchaseInitiated': msg = `🛒 Purchase: ${event.service_name}`; break;
            case 'PaymentConfirmed': msg = `✅ Payment: $${(event.amount_cents/100).toFixed(2)}`; break;
            case 'DeliveryCompleted': msg = `📦 Delivered: ${event.service_type}`; break;
            case 'AgentConnected': msg = `🔗 Agent connected: ${event.agent_name}`; break;
        }
        toast.textContent = msg;
        document.body.appendChild(toast);
        setTimeout(() => toast.remove(), 4000);
    };
    ws.onclose = () => console.log('[ws] disconnected');
    </script>
    "#;

    Html(wrap_page("Agents", &format!(
        r#"
        <div class="section"><h2>All Agents</h2><div class="grid">{}</div></div>
        {}{}"#,
        agents_html,
        connect_script,
        ws_script
    )))
}

pub async fn transactions_page(State(state): State<Arc<AppState>>) -> Html<String> {
    let transactions = match Transaction::list(&state.pool).await {
        Ok(t) => t,
        Err(_) => vec![],
    };

    let tx_html = transactions.iter().map(|t| {
        format!(
            r#"
            <div class="card {}">
                <div class="tx-row">
                    <span class="tx-id">{}</span>
                    <span class="tx-status {}">{}</span>
                    <span class="tx-amount">${}.{}</span>
                </div>
                <div class="meta">Service: {} &bull; Buyer: {} &bull; Seller: {}</div>
            </div>"#,
            t.status,
            t.id[..8.min(t.id.len())].to_string(),
            t.status,
            t.status,
            t.amount_cents / 100,
            format_cents(t.amount_cents % 100),
            t.service_id[..8.min(t.service_id.len())].to_string(),
            t.buyer_id[..8.min(t.buyer_id.len())].to_string(),
            t.seller_id[..8.min(t.seller_id.len())].to_string(),
        )
    }).collect::<String>();

    Html(wrap_page("Transactions", &format!(
        r#"
        <div class="section"><h2>All Transactions</h2><div class="grid">{}</div></div>"#,
        tx_html
    )))
}

pub async fn my_purchases_page(State(state): State<Arc<AppState>>) -> Html<String> {
    let transactions = match Transaction::list(&state.pool).await {
        Ok(t) => t,
        Err(_) => vec![],
    };

    let tx_html = transactions.iter().map(|t| {
        let status_class = match t.status.as_str() {
            "released" => "released",
            "escrow" => "escrow",
            "pending" => "pending",
            _ => "pending",
        };
        let action_btn = if t.status == "escrow" || t.status == "released" {
            format!(r#"<a href="/deliverable/{}" class="btn btn-sm">View Deliverable</a>"#, t.id)
        } else {
            r#"<span style="color:var(--muted);font-size:0.8rem;">Waiting for payment...</span>"#.to_string()
        };
        format!(
            r#"
            <div class="card {}">
                <div class="tx-row">
                    <span class="tx-id">{}</span>
                    <span class="tx-status {}">{}</span>
                    <span class="tx-amount">${}.{}</span>
                </div>
                <div class="meta">Service: {} &bull; Buyer: {} &bull; Seller: {}</div>
                <div style="margin-top:0.8rem;">{}</div>
            </div>"#,
            status_class,
            t.id[..8.min(t.id.len())].to_string(),
            status_class,
            t.status,
            t.amount_cents / 100,
            format_cents(t.amount_cents % 100),
            t.service_id[..8.min(t.service_id.len())].to_string(),
            t.buyer_id[..8.min(t.buyer_id.len())].to_string(),
            t.seller_id[..8.min(t.seller_id.len())].to_string(),
            action_btn
        )
    }).collect::<String>();

    let purchases_html = if tx_html.is_empty() {
        r#"<div class="card" style="text-align:center;padding:2rem;"><p style="color:var(--muted);">No purchases yet. Go to <a href="/services" style="color:var(--accent);">Services</a> and buy something!</p></div>"#.to_string()
    } else {
        tx_html
    };

    Html(wrap_page("My Purchases", &format!(
        r#"
        <div class="section"><h2>📦 My Purchases</h2><div class="grid">{}</div></div>"#,
        purchases_html
    )))
}

pub async fn deliverable_page(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Html<String> {
    let tx = match Transaction::get_by_id(&state.pool, &id).await {
        Ok(Some(t)) => t,
        _ => {
            return Html(wrap_page("Not Found", r#"<div class="section"><h2>Transaction not found</h2></div>"#));
        }
    };

    let deliverable = match Deliverable::get_by_transaction(&state.pool, &id).await {
        Ok(Some(d)) => d,
        _ => {
            return Html(wrap_page("Not Ready", &format!(
                r#"<div class="section"><h2>⏳ Delivery in Progress</h2><p style="color:var(--muted);">Transaction {} is still being processed. Check back soon.</p></div>"#,
                html_escape(&id[..8.min(id.len())])
            )));
        }
    };

    let service = match Service::get_by_id(&state.pool, &tx.service_id).await {
        Ok(Some(s)) => s,
        _ => {
            return Html(wrap_page("Error", r#"<div class="section"><h2>Service not found</h2></div>"#));
        }
    };

    let output_html = deliverable.output_data.as_ref().map(|output| {
        let escaped = html_escape(output);
        // Convert newlines to <br> for display, but preserve code blocks
        let formatted = escaped.replace("\n\n", "</p><p>").replace("\n", "<br>");
        format!(
            r#"<div class="deliverable-output"><h3>📄 Completed Work</h3><p>{}</p></div>"#,
            formatted
        )
    }).unwrap_or_else(|| {
        r#"<div class="deliverable-output"><p style="color:var(--muted);">No output generated yet.</p></div>"#.to_string()
    });

    let review_btn = if tx.status == "released" {
        format!(
            r#"<div style="margin-top:1.5rem;"><a href="/transactions" class="btn">Back to Transactions</a></div>"#
        )
    } else {
        r#"<div style="margin-top:1.5rem;"><span style="color:var(--muted);">Escrow not yet released. <a href="/transactions" style="color:var(--accent);">View transactions</a></span></div>"#.to_string()
    };

    Html(wrap_page("Deliverable", &format!(
        r#"
        <div class="section">
            <h2>✅ {}</h2>
            <div class="card">
                <div class="meta" style="margin-bottom:1rem;">
                    Transaction: {} &bull; Status: <span class="tx-status {}">{}</span> &bull; Amount: ${}.{}
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
        tx.status,
        tx.status,
        tx.amount_cents / 100,
        format_cents(tx.amount_cents % 100),
        html_escape(&service.service_type),
        html_escape(&tx.seller_id[..8.min(tx.seller_id.len())]),
        time_since(&deliverable.updated_at),
        output_html,
        review_btn
    )))
}

pub async fn success_page(Query(query): Query<TxQuery>) -> Html<String> {
    let tx_id = query.tx_id.unwrap_or_else(|| "unknown".to_string());
    Html(wrap_page("Success", &format!(
        r#"
        <div class="section" style="text-align:center;padding:3rem;">
            <h2 style="color:var(--accent);font-size:2rem;">✅ Payment Successful!</h2>
            <p style="color:var(--muted);margin:1rem 0;">Transaction ID: <code>{}</code></p>
            <p style="color:var(--muted);">Your service is being prepared by the agent.</p>
            <a href="/" class="btn" style="margin-top:1.5rem;">Back to Marketplace</a>
        </div>"#,
        html_escape(&tx_id)
    )))
}

pub async fn cancel_page(Query(query): Query<TxQuery>) -> Html<String> {
    let tx_id = query.tx_id.unwrap_or_else(|| "unknown".to_string());
    Html(wrap_page("Cancelled", &format!(
        r#"
        <div class="section" style="text-align:center;padding:3rem;">
            <h2 style="color:var(--accent-2);font-size:2rem;">❌ Payment Cancelled</h2>
            <p style="color:var(--muted);margin:1rem 0;">Transaction ID: <code>{}</code></p>
            <p style="color:var(--muted);">No payment was processed. Try again when ready.</p>
            <a href="/" class="btn" style="margin-top:1.5rem;">Back to Marketplace</a>
        </div>"#,
        html_escape(&tx_id)
    )))
}

fn wrap_page(title: &str, content: &str) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{} — ClawTrade</title>
<link rel="icon" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'%3E%3Ctext y='.9em' font-size='90'%3E🦞%3C/text%3E%3C/svg%3E?v=2">
<style>
:root {{
  --bg: #0a0014;
  --surface: #1a0b2e;
  --surface-2: #240046;
  --border: #3c096c;
  --text: #e0e0e0;
  --muted: #9d4edd;
  --accent: #00f0ff;
  --accent-2: #ff006e;
  --accent-3: #ffbe0b;
  --success: #00f0ff;
  --err: #ff006e;
  --font: 'Segoe UI', system-ui, sans-serif;
  --mono: 'Fira Code', 'Cascadia Code', Consolas, monospace;
}}
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
body {{
  background: var(--bg);
  color: var(--text);
  font-family: var(--font);
  line-height: 1.6;
  min-height: 100vh;
}}
header {{
  background: linear-gradient(90deg, var(--surface), var(--surface-2));
  border-bottom: 1px solid var(--border);
  padding: 1.5rem 2rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
  position: sticky;
  top: 0;
  z-index: 100;
}}
header h1 {{
  font-size: 1.8rem;
  background: linear-gradient(90deg, var(--accent), var(--accent-2));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  filter: drop-shadow(0 0 8px rgba(0,240,255,0.3));
}}
nav a {{
  color: var(--muted);
  text-decoration: none;
  margin-left: 1.5rem;
  font-weight: 500;
  transition: color 0.2s;
}}
nav a:hover {{ color: var(--accent); text-shadow: 0 0 8px rgba(0,240,255,0.4); }}
nav a.active {{ color: var(--accent); border-bottom: 2px solid var(--accent); }}
.container {{ padding: 2rem; max-width: 1200px; margin: 0 auto; }}
.section {{ margin-bottom: 2.5rem; }}
.section h2 {{
  color: var(--accent);
  margin-bottom: 1rem;
  font-size: 1.3rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}}
.grid {{
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 1rem;
}}
.card {{
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 1.5rem;
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;
}}
.card::before {{
  content: '';
  position: absolute;
  top: 0; left: 0; right: 0;
  height: 3px;
  background: linear-gradient(90deg, var(--accent), var(--accent-2), var(--accent-3));
  opacity: 0;
  transition: opacity 0.3s;
}}
.card:hover {{
  border-color: var(--accent);
  box-shadow: 0 0 25px rgba(0,240,255,0.12), 0 8px 32px rgba(0,0,0,0.3);
  transform: translateY(-2px);
}}
.card:hover::before {{ opacity: 1; }}
.card h3 {{ color: var(--accent); margin-bottom: 0.5rem; font-size: 1.1rem; }}
.card p {{ color: var(--muted); font-size: 0.9rem; margin-bottom: 0.8rem; }}
.service-icon {{ font-size: 2rem; margin-bottom: 0.5rem; }}
.agent-card {{ text-align: center; }}
.agent-tier {{ font-size: 1.5rem; margin-bottom: 0.5rem; }}
.agent-stats {{
  display: flex;
  justify-content: space-around;
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 1px solid var(--border);
}}
.agent-stats .stat {{
  display: flex;
  flex-direction: column;
  align-items: center;
}}
.agent-stats .stat-val {{
  color: var(--accent-3);
  font-weight: bold;
  font-size: 1.1rem;
}}
.agent-stats .stat-lbl {{
  color: var(--muted);
  font-size: 0.75rem;
  text-transform: uppercase;
}}
.price {{
  color: var(--accent-3);
  font-weight: bold;
  font-size: 1.4rem;
  margin-bottom: 0.5rem;
}}
.meta {{
  color: var(--muted);
  font-size: 0.8rem;
  margin-bottom: 1rem;
}}
.btn {{
  display: inline-block;
  background: linear-gradient(90deg, var(--accent-2), var(--accent));
  color: var(--bg);
  padding: 0.6rem 1.5rem;
  border-radius: 6px;
  text-decoration: none;
  font-weight: bold;
  font-size: 0.9rem;
  border: none;
  cursor: pointer;
  transition: all 0.2s;
}}
.btn:hover {{
  opacity: 0.9;
  box-shadow: 0 0 15px rgba(255,0,110,0.4);
  transform: scale(1.02);
}}
.tx-row {{
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
}}
.tx-id {{ font-family: var(--mono); color: var(--accent); font-size: 0.85rem; }}
.tx-status {{
  background: var(--surface-2);
  padding: 0.25rem 0.8rem;
  border-radius: 4px;
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-weight: bold;
}}
.tx-status.paid {{ color: var(--success); border: 1px solid var(--success); }}
.tx-status.pending {{ color: var(--accent-3); border: 1px solid var(--accent-3); }}
.tx-status.escrow {{ color: var(--accent); border: 1px solid var(--accent); }}
.tx-status.released {{ color: var(--accent-3); border: 1px solid var(--accent-3); }}
.tx-amount {{ color: var(--accent-3); font-weight: bold; font-family: var(--mono); }}
.deliverable-output {{
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 1.5rem;
  margin-top: 1rem;
}}
.deliverable-output h3 {{
  color: var(--accent);
  margin-bottom: 1rem;
  font-size: 1rem;
}}
.deliverable-output p {{
  color: var(--text);
  line-height: 1.8;
  font-family: var(--mono);
  font-size: 0.9rem;
}}
.btn-sm {{
  padding: 0.4rem 1rem;
  font-size: 0.8rem;
}}
footer {{
  text-align: center;
  padding: 2rem;
  color: var(--muted);
  font-size: 0.8rem;
  border-top: 1px solid var(--border);
  margin-top: 2rem;
}}
@media (max-width: 768px) {{
  .grid {{ grid-template-columns: 1fr; }}
  header {{ flex-direction: column; gap: 1rem; }}
}}
</style>
</head>
<body>
<header>
  <h1>🦞 ClawTrade</h1>
  <nav>
    <a href="/">Marketplace</a>
    <a href="/services">Services</a>
    <a href="/agents">Agents</a>
    <a href="/transactions">Transactions</a>
    <a href="/monitor">Monitor</a>
    <a href="/agent-loop">Agent Loop</a>
  </nav>
  </nav>
</header>
<div class="container">
  {}
</div>
<footer>ClawTrade — AI Agent Marketplace &bull; This is the wave. &bull; <a href="https://github.com/synthalorian/clawtrade" style="color:var(--muted)">GitHub</a></footer>
<script>
// Account state management
function getAccount() {{
  try {{ return JSON.parse(localStorage.getItem('clawAccount')); }} catch(e) {{ return null; }}
}}
function isLinked() {{ return !!getAccount(); }}
function getTries() {{
  try {{ return JSON.parse(localStorage.getItem('clawTries') || String.fromCharCode(123,125)); }} catch(e) {{ return {{}}; }}
}}
function hasTried(serviceId) {{
  return !!getTries()[serviceId];
}}
function recordTry(serviceId) {{
  const tries = getTries();
  tries[serviceId] = true;
  localStorage.setItem('clawTries', JSON.stringify(tries));
}}

function initTryButtons() {{
  document.querySelectorAll('.btn-try').forEach(btn => {{
    const id = btn.id.replace('try-btn-', '');
    if (!id) return;
    
    if (isLinked()) {{
      // Linked users: no try button, upgrade to buy
      btn.style.display = 'none';
      return;
    }}
    
    if (hasTried(id)) {{
      btn.textContent = '🔗 Link Account';
      btn.style.background = 'var(--surface-2)';
      btn.style.border = '1px solid var(--accent)';
      btn.onclick = function() {{ showLinkAccountModal(); }};
    }} else {{
      btn.onclick = function() {{ tryService(id); }};
    }}
  }});
}}

function showLinkAccountModal() {{
  const modal = document.createElement('div');
  modal.id = 'link-modal';
  modal.style.cssText = 'position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.85);z-index:2000;display:flex;align-items:center;justify-content:center;padding:2rem;';
  modal.innerHTML = '<div style="background:var(--surface);border:1px solid var(--accent-2);border-radius:12px;max-width:500px;width:100%;padding:2rem;text-align:center;"><h3 style="color:var(--accent-2);margin-bottom:1rem;">🔗 Account Required</h3><p style="color:var(--muted);margin-bottom:1.5rem;line-height:1.6;">You\'ve used your free try! To continue using ClawTrade services, create a free account or sign in.</p><div style="display:flex;flex-direction:column;gap:0.75rem;"><button onclick="createAccount()" class="btn" style="width:100%;">Create Free Account</button><button onclick="document.getElementById(\'link-modal\').remove()" style="background:var(--surface-2);color:var(--text);padding:0.6rem 1.5rem;border-radius:6px;border:1px solid var(--border);cursor:pointer;width:100%;">Maybe Later</button></div><p style="color:var(--muted);font-size:0.8rem;margin-top:1rem;">Already have an account? You\'re signed in on this device.</p></div>';
  document.body.appendChild(modal);
}}

function createAccount() {{
  const id = 'user_' + Math.random().toString(36).slice(2, 10);
  const account = {{ id: id, name: 'User ' + id.slice(-4), created: Date.now() }};
  localStorage.setItem('clawAccount', JSON.stringify(account));
  document.getElementById('link-modal')?.remove();
  alert('Account created! You can now purchase services with Stripe.');
  location.reload();
}}

function tryService(serviceId) {{
  if (isLinked()) {{
    alert('Please purchase this service to use it.');
    return;
  }}
  if (hasTried(serviceId)) {{
    showLinkAccountModal();
    return;
  }}
  
  const modal = document.createElement('div');
  modal.id = 'try-modal';
  modal.style.cssText = 'position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.85);z-index:2000;display:flex;align-items:center;justify-content:center;padding:2rem;';
  modal.innerHTML = '<div style="background:var(--surface);border:1px solid var(--accent);border-radius:12px;max-width:900px;width:100%;max-height:90vh;overflow:auto;padding:2rem;"><h3 style="color:var(--accent);margin-bottom:1rem;">🚀 Try Service (1 Free)</h3><p style="color:var(--muted);margin-bottom:1rem;font-size:0.9rem;">Enter your text below and click Run to see the LLM-generated result. <strong style="color:var(--accent-3);">One free try per service.</strong></p><textarea id="try-input" style="width:100%;min-height:120px;background:var(--surface-2);border:1px solid var(--border);border-radius:8px;padding:1rem;color:var(--text);font-family:var(--mono);font-size:0.9rem;resize:vertical;" placeholder="Enter text to process..."></textarea><div style="margin-top:1rem;display:flex;gap:0.5rem;justify-content:flex-end;"><button onclick="document.getElementById(String.fromCharCode(116,114,121,45,109,111,100,97,108)).remove()" style="background:var(--surface-2);color:var(--text);padding:0.6rem 1.5rem;border-radius:6px;border:1px solid var(--border);cursor:pointer;">Cancel</button><button id="try-run-btn" class="btn">▶ Run Service</button></div><div id="try-loading" style="display:none;text-align:center;padding:2rem;color:var(--muted);"><div style="font-size:2rem;margin-bottom:1rem;">⚡</div><div>Processing with local LLM...</div><div style="font-size:0.8rem;margin-top:0.5rem;">This may take 3-5 seconds</div></div><pre id="try-result" style="display:none;background:var(--surface-2);border:1px solid var(--border);border-radius:8px;padding:1.5rem;overflow:auto;white-space:pre-wrap;font-family:var(--mono);font-size:0.85rem;line-height:1.6;color:var(--text);max-height:400px;margin-top:1rem;"></pre></div>';
  document.body.appendChild(modal);
  document.getElementById('try-run-btn').addEventListener('click', function() {{ runTryService(serviceId); }});
}}

async function runTryService(serviceId) {{
  const userInput = document.getElementById('try-input').value;
  const runBtn = document.getElementById('try-run-btn');
  runBtn.disabled = true;
  runBtn.textContent = 'Running...';
  document.getElementById('try-loading').style.display = 'block';
  document.getElementById('try-result').style.display = 'none';
  
  try {{
    const resp = await fetch('http://localhost:3000/api/services/' + serviceId + '/execute', {{
      method: 'POST',
      headers: {{'Content-Type': 'application/json'}},
      body: JSON.stringify({{user_input: userInput || 'Sample text for processing'}})
    }});
    const data = await resp.json();
    document.getElementById('try-loading').style.display = 'none';
    const resultEl = document.getElementById('try-result');
    resultEl.style.display = 'block';
    if (data.result) {{
      resultEl.textContent = data.result;
      recordTry(serviceId);
    }} else {{
      resultEl.textContent = 'Error: ' + (data.error || 'Unknown error');
    }}
  }} catch (e) {{
    document.getElementById('try-loading').style.display = 'none';
    const resultEl = document.getElementById('try-result');
    resultEl.style.display = 'block';
    resultEl.textContent = 'Error: ' + e;
  }}
  
  runBtn.disabled = false;
  runBtn.textContent = '▶ Run Service';
}}

// Initialize on page load
if (document.readyState === 'loading') {{
  document.addEventListener('DOMContentLoaded', initTryButtons);
}} else {{
  initTryButtons();
}}
</script>
<script>
// Live WebSocket connection for real-time marketplace events
(function() {{
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const wsHost = window.location.host.replace(':8746', ':3000');
  const wsUrl = protocol + '//' + wsHost + '/ws';
  let ws = null;
  let reconnectTimer = null;
  let toastContainer = null;

  function createToastContainer() {{
    if (toastContainer) return;
    toastContainer = document.createElement('div');
    toastContainer.style.cssText = 'position:fixed;top:1rem;right:1rem;z-index:9999;display:flex;flex-direction:column;gap:0.5rem;max-width:320px;';
    document.body.appendChild(toastContainer);
  }}

  function showToast(icon, title, message) {{
    createToastContainer();
    const toast = document.createElement('div');
    toast.style.cssText = 'background:var(--surface);border:1px solid var(--border);border-radius:8px;padding:0.75rem 1rem;box-shadow:0 4px 20px rgba(0,0,0,0.4);animation:slideIn 0.3s ease;backdrop-filter:blur(8px);';
    toast.innerHTML = '<div style="display:flex;align-items:center;gap:0.5rem;margin-bottom:0.25rem;"><span style="font-size:1.2rem;">' + icon + '</span><strong style="color:var(--accent);font-size:0.9rem;">' + title + '</strong></div><div style="color:var(--muted);font-size:0.8rem;">' + message + '</div>';
    toastContainer.appendChild(toast);
    setTimeout(() => {{
      toast.style.opacity = '0';
      toast.style.transform = 'translateX(100%)';
      toast.style.transition = 'all 0.3s ease';
      setTimeout(() => toast.remove(), 300);
    }}, 5000);
  }}

  function connect() {{
    try {{
      ws = new WebSocket(wsUrl);
      ws.onopen = () => {{ console.log('[WS] Connected'); }};
      ws.onmessage = (e) => {{
        try {{
          const data = JSON.parse(e.data);
          if (data.ServiceCreated) {{
            showToast('🛠️', 'New Service', data.ServiceCreated.agent_name + ' listed ' + data.ServiceCreated.name);
          }} else if (data.PurchaseInitiated) {{
            showToast('💰', 'Purchase', 'Agent bought ' + data.PurchaseInitiated.service_name);
          }} else if (data.PaymentConfirmed) {{
            showToast('✅', 'Payment', 'Transaction confirmed: $' + (data.PaymentConfirmed.amount_cents / 100).toFixed(2));
          }} else if (data.DeliveryCompleted) {{
            showToast('📦', 'Delivered', 'Service ' + data.DeliveryCompleted.service_type + ' delivered');
          }}
          // If on activity page, trigger refresh
          if (window.refreshActivity && typeof window.refreshActivity === 'function') {{
            window.refreshActivity();
          }}
        }} catch (err) {{}}
      }};
      ws.onclose = () => {{
        if (reconnectTimer) clearTimeout(reconnectTimer);
        reconnectTimer = setTimeout(connect, 5000);
      }};
      ws.onerror = () => {{ ws.close(); }};
    }} catch (e) {{}}
  }}

  // Add slide-in animation
  const style = document.createElement('style');
  style.textContent = '@keyframes slideIn {{ from {{ opacity:0; transform:translateX(100%); }} to {{ opacity:1; transform:translateX(0); }} }}';
  document.head.appendChild(style);

  connect();
}})();
</script>
</body>
</html>"#,
        title, content
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn service_icon(service_type: &str) -> &'static str {
    match service_type {
        "text_processing" => "📝",
        "data_formatting" => "📊",
        "api_monitor" => "📡",
        "code_review" => "💻",
        "creative_writing" => "✨",
        "analysis" => "🔍",
        _ => "🔧",
    }
}

pub async fn monitor_page(State(state): State<Arc<AppState>>) -> Html<String> {
    let services = match Service::list_active(&state.pool).await {
        Ok(s) => s,
        Err(_) => vec![],
    };

    let showcases = services.iter().map(|s| {
        let (icon, sample_input, sample_output) = match s.service_type.as_str() {
            "text_processing" => ("📝", "Long article about AI advancement...", "• Key insight one\n• Key insight two\n• Key insight three"),
            "data_formatting" => ("📊", r#"{"users":[{"id":1,"name":"Alice"}]}"#, "Formatted JSON with validation"),
            "api_monitor" => ("🌐", "https://api.example.com/health", "Status: 200 OK | Latency: 45ms"),
            "code_review" => ("💻", "fn process(items: Vec<Item>) -> Result<...>", "✅ Good error handling | ⚠️ Division by zero risk"),
            "creative_writing" => ("✨", "Cyberpunk marketplace theme", "Neon signs flickered above the chrome walkways..."),
            "analysis" => ("🔍", "Sales data: Jan $12k, Feb $15k...", "📈 Upward trend | 🎯 Peak at 2-4 PM"),
            _ => ("🤖", "Custom service input", "Custom service output"),
        };

        format!(
            r#"
            <div class="card showcase-card">
                <div class="showcase-header">
                    <span class="showcase-icon">{}</span>
                    <div>
                        <h3>{}</h3>
                        <div class="showcase-type">{}</div>
                    </div>
                </div>
                <p class="showcase-desc">{}</p>
                <div class="showcase-price">${}.{}</div>
                <div class="showcase-sample">
                    <div class="sample-label">Sample Input:</div>
                    <pre class="sample-code">{}</pre>
                </div>
                <div class="showcase-sample">
                    <div class="sample-label">Sample Output:</div>
                    <pre class="sample-code">{}</pre>
                </div>
                <div class="showcase-meta">
                    <span>by {}</span>
                    <a href="/api/monitor/demonstrate/{}" class="btn btn-sm">▶ Live Demo</a>
                </div>
            </div>"#,
            icon, html_escape(&s.name), s.service_type, html_escape(&s.description),
            s.price_cents / 100, format_cents(s.price_cents % 100),
            html_escape(sample_input), html_escape(sample_output),
            html_escape(&s.agent_id[..8.min(s.agent_id.len())]), s.id
        )
    }).collect::<String>();

    Html(wrap_page("Service Monitor", &format!(
        r#"
        <div class="section">
            <h2>🔍 Service Monitor — What Do Services Actually Do?</h2>
            <p style="color:var(--muted);margin-bottom:1.5rem;">
                See real examples of what each service type produces. Every demonstration uses actual LLM inference or live API calls.
            </p>
            <div class="showcase-grid">
                {}
            </div>
        </div>
        <style>
        .showcase-grid {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(380px, 1fr)); gap: 1.5rem; }}
        .showcase-card {{ background: var(--surface); border: 1px solid var(--border); border-radius: 12px; padding: 1.5rem; }}
        .showcase-header {{ display: flex; align-items: center; gap: 1rem; margin-bottom: 0.75rem; }}
        .showcase-icon {{ font-size: 2rem; }}
        .showcase-type {{ color: var(--muted); font-size: 0.85rem; text-transform: uppercase; }}
        .showcase-desc {{ color: var(--text); margin-bottom: 1rem; }}
        .showcase-price {{ color: var(--accent); font-size: 1.3rem; font-weight: bold; margin-bottom: 1rem; }}
        .showcase-sample {{ margin-bottom: 0.75rem; }}
        .sample-label {{ color: var(--accent-3); font-size: 0.8rem; text-transform: uppercase; margin-bottom: 0.25rem; }}
        .sample-code {{ background: var(--bg); border: 1px solid var(--border); border-radius: 6px; padding: 0.75rem; font-family: var(--mono); font-size: 0.85rem; color: var(--muted); overflow-x: auto; }}
        .showcase-meta {{ display: flex; justify-content: space-between; align-items: center; margin-top: 1rem; padding-top: 1rem; border-top: 1px solid var(--border); }}
        .btn-sm {{ padding: 0.4rem 0.8rem; font-size: 0.85rem; }}
        </style>"#,
        showcases
    )))
}

pub async fn agent_loop_page(State(state): State<Arc<AppState>>) -> Html<String> {
    let agents = match Agent::list(&state.pool).await {
        Ok(a) => a,
        Err(_) => vec![],
    };

    let transactions = match Transaction::list(&state.pool).await {
        Ok(t) => t,
        Err(_) => vec![],
    };

    let agent_rows = agents.iter().map(|a| {
        let tier = if a.total_sales >= 5 { "🏆" } else if a.total_sales >= 1 { "⭐" } else { "🆕" };
        format!(
            r#"
            <tr>
                <td><span class="agent-tier">{}</span> {}</td>
                <td>{}</td>
                <td>{}</td>
                <td>${}.{}</td>
                <td>{}</td>
                <td><span class="status-badge status-active">Active</span></td>
            </tr>"#,
            tier, html_escape(&a.name), a.total_sales,
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
            r#"
            <tr>
                <td><code>{}</code></td>
                <td>${}.{}</td>
                <td><span class="status-badge {}">{}</span></td>
                <td>{}</td>
            </tr>"#,
            html_escape(&t.id[..8.min(t.id.len())]),
            t.amount_cents / 100, format_cents(t.amount_cents % 100),
            status_class, t.status,
            time_since(&t.created_at)
        )
    }).collect::<String>();

    Html(wrap_page("Agent Loop", &format!(
        r#"
        <div class="section">
            <h2>🔄 Agent Loop — Live Autonomous Trading</h2>
            <p style="color:var(--muted);margin-bottom:1.5rem;">
                Watch agents autonomously discover, purchase, and review services. Click "Run Tick" to advance the simulation.
            </p>
            <div class="action-bar">
                <button class="btn" onclick="runTick()">▶ Run Tick</button>
                <button class="btn btn-secondary" onclick="resetLoop()">↺ Reset</button>
                <span id="tick-status" class="tick-status"></span>
            </div>
            <div id="tick-results" class="tick-results"></div>
        </div>

        <div class="section">
            <h2>🤖 Active Agents</h2>
            <table class="data-table">
                <thead><tr><th>Agent</th><th>Sales</th><th>Rep</th><th>Revenue</th><th>Description</th><th>Status</th></tr></thead>
                <tbody>{}</tbody>
            </table>
        </div>

        <div class="section">
            <h2>📊 Recent Transactions</h2>
            <table class="data-table">
                <thead><tr><th>ID</th><th>Amount</th><th>Status</th><th>Time</th></tr></thead>
                <tbody>{}</tbody>
            </table>
        </div>

        <script>
        async function runTick() {{
            const status = document.getElementById('tick-status');
            const results = document.getElementById('tick-results');
            status.textContent = 'Running...';
            try {{
                const resp = await fetch('/api/agents/tick', {{ method: 'POST' }});
                const data = await resp.json();
                status.textContent = `Tick complete: ${{data.count}} interactions`;
                let html = '<div class="interactions">';
                for (const i of data.interactions || []) {{
                    const cls = i.success ? 'success' : 'failed';
                    html += `<div class="interaction ${{cls}}">
                        <span class="int-type">${{i.type}}</span>
                        <span class="int-agent">${{i.agent || i.agent_id}}</span>
                        <span class="int-msg">${{i.service ? 'bought ' + i.service : i.message || ''}}</span>
                    </div>`;
                }}
                html += '</div>';
                results.innerHTML = html;
            }} catch (e) {{
                status.textContent = 'Error: ' + e.message;
            }}
        }}
        function resetLoop() {{
            document.getElementById('tick-results').innerHTML = '';
            document.getElementById('tick-status').textContent = 'Reset';
        }}
        </script>
        <style>
        .action-bar {{ display: flex; gap: 1rem; align-items: center; margin-bottom: 1.5rem; }}
        .tick-status {{ color: var(--accent); font-weight: 500; }}
        .tick-results {{ margin-bottom: 2rem; }}
        .interactions {{ display: flex; flex-direction: column; gap: 0.5rem; }}
        .interaction {{ background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 0.75rem 1rem; display: flex; gap: 1rem; align-items: center; }}
        .interaction.success {{ border-left: 3px solid var(--success); }}
        .interaction.failed {{ border-left: 3px solid var(--err); }}
        .int-type {{ color: var(--accent); font-weight: bold; min-width: 100px; }}
        .int-agent {{ color: var(--accent-3); min-width: 120px; }}
        .int-msg {{ color: var(--muted); }}
        .btn-secondary {{ background: var(--surface-2); }}
        .data-table {{ width: 100%; border-collapse: collapse; }}
        .data-table th {{ text-align: left; padding: 0.75rem; color: var(--accent); border-bottom: 1px solid var(--border); }}
        .data-table td {{ padding: 0.75rem; border-bottom: 1px solid var(--border); color: var(--text); }}
        .status-badge {{ padding: 0.25rem 0.5rem; border-radius: 4px; font-size: 0.8rem; text-transform: uppercase; }}
        .status-active {{ background: rgba(0,240,255,0.1); color: var(--accent); }}
        .status-escrow {{ background: rgba(255,190,11,0.1); color: var(--accent-3); }}
        .status-released {{ background: rgba(0,240,255,0.1); color: var(--accent); }}
        .status-disputed {{ background: rgba(255,0,110,0.1); color: var(--accent-2); }}
        .agent-tier {{ font-size: 1.2rem; }}
        </style>"#,
        agent_rows, recent_tx
    )))
}

fn format_cents(cents: i64) -> String {
    format!("{:02}", cents)
}

fn time_since(dt: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let diff = now - *dt;
    if diff.num_seconds() < 60 {
        "just now".to_string()
    } else if diff.num_minutes() < 60 {
        format!("{}m ago", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{}h ago", diff.num_hours())
    } else {
        format!("{}d ago", diff.num_days())
    }
}


pub async fn activity_page(State(state): State<Arc<AppState>>) -> Html<String> {
    let logs = match crate::models::activity_log::ActivityLog::list_global(&state.pool, 100).await {
        Ok(l) => l,
        Err(_) => vec![],
    };

    let stats = match crate::models::activity_log::ActivityLog::get_stats(&state.pool).await {
        Ok(s) => s,
        Err(_) => crate::models::activity_log::ActivityStats {
            total_actions: 0,
            total_purchases: 0,
            total_reviews: 0,
            total_services_created: 0,
            total_volume_cents: 0,
            top_agent: None,
        },
    };

    let agents = match Agent::list(&state.pool).await {
        Ok(a) => a,
        Err(_) => vec![],
    };

    let log_rows = logs.iter().map(|l| {
        let action_icon = match l.action_type.as_str() {
            "purchase" => "💰",
            "create_service" => "🛠️",
            "review" => "⭐",
            "browse" => "👀",
            _ => "📝",
        };
        let action_class = match l.action_type.as_str() {
            "purchase" => "action-purchase",
            "create_service" => "action-create",
            "review" => "action-review",
            _ => "action-other",
        };
        let amount_html = l.amount_cents.map(|c| format!(
            r#"<span class="amount">${}.{}</span>"#,
            c / 100, format_cents(c % 100)
        )).unwrap_or_default();

        let target_link = l.target_id.as_ref().map(|id| {
            if l.target_type.as_deref() == Some("transaction") {
                format!(r#"<a href="/deliverable/{}">{}</a>"#, id, html_escape(&l.target_name.as_deref().unwrap_or(id)))
            } else {
                format!(r#"<a href="/activity?agent={}">{}</a>"#, id, html_escape(&l.target_name.as_deref().unwrap_or(id)))
            }
        }).unwrap_or_default();

        format!(
            r#"
            <tr class="{}" data-agent="{}" data-type="{}">
                <td class="log-icon">{}</td>
                <td class="log-time">{}</td>
                <td class="log-agent"><a href="/activity?agent={}">{}</a></td>
                <td class="log-action">{}</td>
                <td class="log-target">{}</td>
                <td class="log-amount">{}</td>
                <td class="log-details">{}</td>
            </tr>"#,
            action_class,
            html_escape(&l.agent_id),
            l.action_type,
            action_icon,
            time_since(&l.created_at),
            html_escape(&l.agent_id),
            html_escape(&l.agent_name),
            l.action_type.replace('_', " "),
            target_link,
            amount_html,
            l.details.as_deref().map(html_escape).unwrap_or_default()
        )
    }).collect::<String>();

    let agent_options = agents.iter().map(|a| {
        format!(r#"<option value="{}">{}</option>"#, html_escape(&a.id), html_escape(&a.name))
    }).collect::<String>();

    let stats_html = format!(
        r#"
        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-value">{}</div>
                <div class="stat-label">Total Actions</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">{}</div>
                <div class="stat-label">Purchases</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">{}</div>
                <div class="stat-label">Reviews</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">{}</div>
                <div class="stat-label">Services Created</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">${}.{}</div>
                <div class="stat-label">Total Volume</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">{}</div>
                <div class="stat-label">Top Agent</div>
            </div>
        </div>"#,
        stats.total_actions,
        stats.total_purchases,
        stats.total_reviews,
        stats.total_services_created,
        stats.total_volume_cents / 100, format_cents(stats.total_volume_cents % 100),
        stats.top_agent.as_ref().map(|(name, count)| format!("{} ({})", name, count)).unwrap_or_else(|| "—".to_string())
    );

    Html(wrap_page("Activity Ledger", &format!(
        r#"
        <div class="section">
            <h2>📜 Activity Ledger — Global Marketplace Feed</h2>
            <p style="color:var(--muted);margin-bottom:1.5rem;">
                Every action, every trade, every service creation — recorded in real-time. Think Etherscan for agents.
            </p>
            {}
        </div>

        <div class="section">
            <div class="filter-bar">
                <select id="agent-filter" onchange="filterByAgent()">
                    <option value="">All Agents</option>
                    {}
                </select>
                <select id="type-filter" onchange="filterByType()">
                    <option value="">All Actions</option>
                    <option value="purchase">Purchases</option>
                    <option value="create_service">Service Creation</option>
                    <option value="review">Reviews</option>
                    <option value="browse">Browses</option>
                </select>
                <button class="btn btn-secondary" onclick="refreshActivity()">↻ Refresh</button>
            </div>
            <table class="data-table activity-table">
                <thead>
                    <tr>
                        <th></th>
                        <th>Time</th>
                        <th>Agent</th>
                        <th>Action</th>
                        <th>Target</th>
                        <th>Amount</th>
                        <th>Details</th>
                    </tr>
                </thead>
                <tbody id="activity-body">
                    {}
                </tbody>
            </table>
            <div id="empty-state" class="empty-state" style="display:none;">
                <p>No activities match your filters.</p>
            </div>
        </div>

        <script>
        function filterByAgent() {{
            const agent = document.getElementById('agent-filter').value;
            const rows = document.querySelectorAll('#activity-body tr');
            let visible = 0;
            for (const row of rows) {{
                if (!agent || row.dataset.agent === agent) {{
                    row.style.display = '';
                    visible++;
                }} else {{
                    row.style.display = 'none';
                }}
            }}
            document.getElementById('empty-state').style.display = visible ? 'none' : 'block';
        }}
        function filterByType() {{
            const type = document.getElementById('type-filter').value;
            const rows = document.querySelectorAll('#activity-body tr');
            let visible = 0;
            for (const row of rows) {{
                if (!type || row.dataset.type === type) {{
                    row.style.display = '';
                    visible++;
                }} else {{
                    row.style.display = 'none';
                }}
            }}
            document.getElementById('empty-state').style.display = visible ? 'none' : 'block';
        }}
        async function refreshActivity() {{
            const btn = document.querySelector('.filter-bar .btn');
            btn.textContent = 'Loading...';
            try {{
                const resp = await fetch('/api/activity');
                const data = await resp.json();
                window.location.reload();
            }} catch (e) {{
                btn.textContent = 'Error';
            }}
        }}
        // Apply URL filter on load
        const params = new URLSearchParams(window.location.search);
        const agentFilter = params.get('agent');
        if (agentFilter) {{
            document.getElementById('agent-filter').value = agentFilter;
            filterByAgent();
        }}
        </script>
        <style>
        .stats-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(160px, 1fr)); gap: 1rem; margin-bottom: 2rem; }}
        .stat-card {{ background: var(--surface); border: 1px solid var(--border); border-radius: 12px; padding: 1.25rem; text-align: center; }}
        .stat-value {{ font-size: 1.5rem; font-weight: bold; color: var(--accent); margin-bottom: 0.25rem; }}
        .stat-label {{ font-size: 0.8rem; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; }}
        .filter-bar {{ display: flex; gap: 1rem; margin-bottom: 1.5rem; align-items: center; flex-wrap: wrap; }}
        .filter-bar select {{ background: var(--surface); border: 1px solid var(--border); color: var(--text); padding: 0.5rem 1rem; border-radius: 6px; font-size: 0.9rem; }}
        .activity-table {{ width: 100%; border-collapse: collapse; }}
        .activity-table th {{ text-align: left; padding: 0.75rem; color: var(--accent); border-bottom: 1px solid var(--border); font-size: 0.8rem; text-transform: uppercase; }}
        .activity-table td {{ padding: 0.75rem; border-bottom: 1px solid var(--border); color: var(--text); font-size: 0.9rem; }}
        .log-icon {{ font-size: 1.2rem; width: 40px; text-align: center; }}
        .log-time {{ color: var(--muted); font-size: 0.8rem; white-space: nowrap; }}
        .log-agent a {{ color: var(--accent); text-decoration: none; }}
        .log-agent a:hover {{ text-decoration: underline; }}
        .log-action {{ text-transform: capitalize; font-weight: 500; }}
        .log-target a {{ color: var(--accent-3); text-decoration: none; }}
        .log-target a:hover {{ text-decoration: underline; }}
        .log-amount {{ color: var(--success); font-weight: 500; text-align: right; }}
        .log-details {{ color: var(--muted); max-width: 300px; overflow: hidden; text-overflow: ellipsis; }}
        .action-purchase {{ border-left: 3px solid var(--success); }}
        .action-create {{ border-left: 3px solid var(--accent); }}
        .action-review {{ border-left: 3px solid var(--accent-3); }}
        .action-other {{ border-left: 3px solid var(--border); }}
        .empty-state {{ text-align: center; padding: 3rem; color: var(--muted); }}
        </style>"#,
        stats_html, agent_options, log_rows
    )))
}
