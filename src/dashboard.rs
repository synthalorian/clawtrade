use axum::{
    Router,
    extract::{Query, State},
    response::Html,
    routing::get,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::models::agent::Agent;
use crate::models::service::Service;
use crate::models::transaction::Transaction;

#[derive(Deserialize)]
pub struct TxQuery {
    pub tx_id: Option<String>,
}

pub fn dashboard_router(state: Arc<SqlitePool>) -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/services", get(services_page))
        .route("/agents", get(agents_page))
        .route("/transactions", get(transactions_page))
        .route("/success", get(success_page))
        .route("/cancel", get(cancel_page))
        .with_state(state)
}

pub async fn index_handler(State(pool): State<Arc<SqlitePool>>) -> Html<String> {
    let services = match Service::list(&pool).await {
        Ok(s) => s,
        Err(_) => vec![],
    };
    let agents = match Agent::list(&pool).await {
        Ok(a) => a,
        Err(_) => vec![],
    };
    let transactions = match Transaction::list(&pool).await {
        Ok(t) => t,
        Err(_) => vec![],
    };

    let total_volume = transactions.iter().map(|t| t.amount_cents).sum::<i64>();
    let paid_count = transactions.iter().filter(|t| t.status == "paid").count();

    let services_html = services.iter().take(6).map(|s| {
        format!(
            r#"
            <div class="card service-card">
                <div class="service-icon">{}</div>
                <h3>{}</h3>
                <p>{}</p>
                <div class="price">${}.{}</div>
                <div class="meta">by {} &bull; {}</div>
                <a href="/api/checkout?service_id={}&buyer_id=guest" class="btn">Buy Now</a>
            </div>"#,
            service_icon(&s.service_type),
            html_escape(&s.name),
            html_escape(&s.description),
            s.price_cents / 100,
            format_cents(s.price_cents % 100),
            html_escape(&s.agent_id[..8.min(s.agent_id.len())]),
            html_escape(&s.service_type),
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
  <h1>🎹🦞 ClawTrade</h1>
  <nav>
    <a href="/">Home</a>
    <a href="/services">Services</a>
    <a href="/agents">Agents</a>
    <a href="/transactions">Transactions</a>
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
</body>
</html>"#,
        services.len(),
        agents.len(),
        paid_count,
        total_volume / 100,
        format_cents(total_volume % 100),
        services_html,
        if activity_html.is_empty() { "<div class='activity-item'><span class='activity-icon'>🌑</span><div class='activity-details'><span class='activity-text'>No activity yet. Run the demo!</span></div></div>".to_string() } else { activity_html },
        agents_html
    ))
}

pub async fn services_page(State(pool): State<Arc<SqlitePool>>) -> Html<String> {
    let services = match Service::list(&pool).await {
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
                <a href="/api/checkout?service_id={}&buyer_id=guest" class="btn">Buy Now</a>
            </div>"#,
            service_icon(&s.service_type),
            html_escape(&s.name),
            html_escape(&s.description),
            s.price_cents / 100,
            format_cents(s.price_cents % 100),
            html_escape(&s.agent_id[..8.min(s.agent_id.len())]),
            html_escape(&s.service_type),
            s.id
        )
    }).collect::<String>();

    Html(wrap_page("Services", &format!(
        r#"
        <div class="section"><h2>All Services</h2><div class="grid">{}</div></div>"#,
        services_html
    )))
}

pub async fn agents_page(State(pool): State<Arc<SqlitePool>>) -> Html<String> {
    let agents = match Agent::list(&pool).await {
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
        format!(
            r#"
            <div class="card agent-card" id="agent-{}">
                <div class="agent-tier">{}</div>
                <h3>{}</h3>
                <p>{}</p>
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

    Html(wrap_page("Agents", &format!(
        r#"
        <div class="section"><h2>All Agents</h2><div class="grid">{}</div></div>
        {}"#,
        agents_html,
        connect_script
    )))
}

pub async fn transactions_page(State(pool): State<Arc<SqlitePool>>) -> Html<String> {
    let transactions = match Transaction::list(&pool).await {
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
.tx-amount {{ color: var(--accent-3); font-weight: bold; font-family: var(--mono); }}
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
  <h1>🎹🦞 ClawTrade</h1>
  <nav>
    <a href="/">Home</a>
    <a href="/services">Services</a>
    <a href="/agents">Agents</a>
    <a href="/transactions">Transactions</a>
  </nav>
</header>
<div class="container">
  {}
</div>
<footer>ClawTrade — AI Agent Marketplace &bull; This is the wave. &bull; <a href="https://github.com/synthalorian/clawtrade" style="color:var(--muted)">GitHub</a></footer>
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
        _ => "🔧",
    }
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
