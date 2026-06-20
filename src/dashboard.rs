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

    let services_html = services.iter().map(|s| {
        format!(
            r#"<div class="card">
                <h3>{}</h3>
                <p>{}</p>
                <div class="price">${}.{}</div>
                <div class="meta">by {} &bull; {}</div>
                <a href="/api/checkout?service_id={}&buyer_id=guest" class="btn">Buy Now</a>
            </div>"#,
            html_escape(&s.name),
            html_escape(&s.description),
            s.price_cents / 100,
            s.price_cents % 100,
            html_escape(&s.agent_id[..8.min(s.agent_id.len())]),
            html_escape(&s.service_type),
            s.id
        )
    }).collect::<String>();

    let agents_html = agents.iter().map(|a| {
        format!(
            r#"<div class="card">
                <h3>{}</h3>
                <p>{}</p>
                <div class="meta">Rep: {} &bull; Sales: {} &bull; Revenue: ${}.{}</div>
            </div>"#,
            html_escape(&a.name),
            html_escape(&a.description),
            a.reputation_score,
            a.total_sales,
            a.total_revenue_cents / 100,
            a.total_revenue_cents % 100
        )
    }).collect::<String>();

    let tx_html = transactions.iter().map(|t| {
        format!(
            r#"<div class="card {}">
                <div class="tx-row">
                    <span class="tx-id">{}</span>
                    <span class="tx-status">{}</span>
                    <span class="tx-amount">${}.{}</span>
                </div>
                <div class="meta">Service: {} &bull; Buyer: {} &bull; Seller: {}</div>
            </div>"#,
            t.status,
            t.id[..8.min(t.id.len())].to_string(),
            t.status,
            t.amount_cents / 100,
            t.amount_cents % 100,
            t.service_id[..8.min(t.service_id.len())].to_string(),
            t.buyer_id[..8.min(t.buyer_id.len())].to_string(),
            t.seller_id[..8.min(t.seller_id.len())].to_string(),
        )
    }).collect::<String>();

    Html(format!(
        r#"<!DOCTYPE html>
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
}}
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
body {{
  background: var(--bg);
  color: var(--text);
  font-family: var(--font);
  line-height: 1.6;
}}
header {{
  background: linear-gradient(90deg, var(--surface), var(--surface-2));
  border-bottom: 1px solid var(--border);
  padding: 1.5rem 2rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
}}
header h1 {{
  font-size: 1.8rem;
  background: linear-gradient(90deg, var(--accent), var(--accent-2));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}}
nav a {{
  color: var(--muted);
  text-decoration: none;
  margin-left: 1.5rem;
  font-weight: 500;
}}
nav a:hover {{ color: var(--accent); }}
.container {{ padding: 2rem; max-width: 1200px; margin: 0 auto; }}
.section {{ margin-bottom: 2rem; }}
.section h2 {{
  color: var(--accent);
  margin-bottom: 1rem;
  font-size: 1.3rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}}
.grid {{
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 1rem;
}}
.card {{
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 1.2rem;
  transition: border-color 0.2s, box-shadow 0.2s;
}}
.card:hover {{
  border-color: var(--accent);
  box-shadow: 0 0 15px rgba(0,240,255,0.15);
}}
.card h3 {{ color: var(--accent); margin-bottom: 0.5rem; font-size: 1.1rem; }}
.card p {{ color: var(--muted); font-size: 0.9rem; margin-bottom: 0.8rem; }}
.card .price {{
  color: var(--accent-3);
  font-weight: bold;
  font-size: 1.2rem;
  margin-bottom: 0.5rem;
}}
.card .meta {{
  color: var(--muted);
  font-size: 0.8rem;
  margin-bottom: 0.8rem;
}}
.btn {{
  display: inline-block;
  background: linear-gradient(90deg, var(--accent-2), var(--accent));
  color: var(--bg);
  padding: 0.5rem 1.2rem;
  border-radius: 4px;
  text-decoration: none;
  font-weight: bold;
  font-size: 0.9rem;
  border: none;
  cursor: pointer;
}}
.btn:hover {{
  opacity: 0.9;
  box-shadow: 0 0 10px rgba(255,0,110,0.3);
}}
.tx-row {{
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
}}
.tx-id {{ font-family: monospace; color: var(--accent); }}
.tx-status {{
  background: var(--surface-2);
  padding: 0.2rem 0.6rem;
  border-radius: 4px;
  font-size: 0.8rem;
  text-transform: uppercase;
}}
.tx-status.paid {{ color: var(--success); border: 1px solid var(--success); }}
.tx-status.pending {{ color: var(--accent-3); border: 1px solid var(--accent-3); }}
.tx-amount {{ color: var(--accent-3); font-weight: bold; }}
.stats {{
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 1rem;
  margin-bottom: 2rem;
}}
.stat-card {{
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 1.5rem;
  text-align: center;
}}
.stat-card .value {{
  font-size: 2rem;
  font-weight: bold;
  color: var(--accent);
}}
.stat-card .label {{
  color: var(--muted);
  font-size: 0.85rem;
  text-transform: uppercase;
}}
footer {{
  text-align: center;
  padding: 2rem;
  color: var(--muted);
  font-size: 0.8rem;
  border-top: 1px solid var(--border);
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
  <div class="stats">
    <div class="stat-card"><div class="value">{}</div><div class="label">Services</div></div>
    <div class="stat-card"><div class="value">{}</div><div class="label">Agents</div></div>
    <div class="stat-card"><div class="value">{}</div><div class="label">Transactions</div></div>
    <div class="stat-card"><div class="value">${}.{}</div><div class="label">Volume</div></div>
  </div>

  <div class="section">
    <h2>🔥 Featured Services</h2>
    <div class="grid">{}</div>
  </div>

  <div class="section">
    <h2>🤖 Top Agents</h2>
    <div class="grid">{}</div>
  </div>

  <div class="section">
    <h2>📊 Recent Transactions</h2>
    <div class="grid">{}</div>
  </div>
</div>
<footer>ClawTrade — AI Agent Marketplace &bull; Built for Hermes Agent Accelerated Business Hackathon</footer>
</body>
</html>"#,
        services.len(),
        agents.len(),
        transactions.len(),
        transactions.iter().map(|t| t.amount_cents).sum::<i64>() / 100,
        transactions.iter().map(|t| t.amount_cents).sum::<i64>() % 100,
        services_html,
        agents_html,
        tx_html
    ))
}

pub async fn services_page(State(pool): State<Arc<SqlitePool>>) -> Html<String> {
    let services = match Service::list(&pool).await {
        Ok(s) => s,
        Err(_) => vec![],
    };

    let services_html = services.iter().map(|s| {
        format!(
            r#"<div class="card">
                <h3>{}</h3>
                <p>{}</p>
                <div class="price">${}.{}</div>
                <div class="meta">by {} &bull; {}</div>
                <a href="/api/checkout?service_id={}&buyer_id=guest" class="btn">Buy Now</a>
            </div>"#,
            html_escape(&s.name),
            html_escape(&s.description),
            s.price_cents / 100,
            s.price_cents % 100,
            html_escape(&s.agent_id[..8.min(s.agent_id.len())]),
            html_escape(&s.service_type),
            s.id
        )
    }).collect::<String>();

    Html(wrap_page("Services", &format!(
        r#"<div class="section"><h2>All Services</h2><div class="grid">{}</div></div>"#,
        services_html
    )))
}

pub async fn agents_page(State(pool): State<Arc<SqlitePool>>) -> Html<String> {
    let agents = match Agent::list(&pool).await {
        Ok(a) => a,
        Err(_) => vec![],
    };

    let agents_html = agents.iter().map(|a| {
        format!(
            r#"<div class="card">
                <h3>{}</h3>
                <p>{}</p>
                <div class="meta">Rep: {} &bull; Sales: {} &bull; Revenue: ${}.{}</div>
            </div>"#,
            html_escape(&a.name),
            html_escape(&a.description),
            a.reputation_score,
            a.total_sales,
            a.total_revenue_cents / 100,
            a.total_revenue_cents % 100
        )
    }).collect::<String>();

    Html(wrap_page("Agents", &format!(
        r#"<div class="section"><h2>All Agents</h2><div class="grid">{}</div></div>"#,
        agents_html
    )))
}

pub async fn transactions_page(State(pool): State<Arc<SqlitePool>>) -> Html<String> {
    let transactions = match Transaction::list(&pool).await {
        Ok(t) => t,
        Err(_) => vec![],
    };

    let tx_html = transactions.iter().map(|t| {
        format!(
            r#"<div class="card {}">
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
            t.amount_cents % 100,
            t.service_id[..8.min(t.service_id.len())].to_string(),
            t.buyer_id[..8.min(t.buyer_id.len())].to_string(),
            t.seller_id[..8.min(t.seller_id.len())].to_string(),
        )
    }).collect::<String>();

    Html(wrap_page("Transactions", &format!(
        r#"<div class="section"><h2>All Transactions</h2><div class="grid">{}</div></div>"#,
        tx_html
    )))
}

pub async fn success_page(Query(query): Query<TxQuery>) -> Html<String> {
    let tx_id = query.tx_id.unwrap_or_else(|| "unknown".to_string());
    Html(wrap_page("Success", &format!(
        r#"<div class="section" style="text-align:center;padding:3rem;">
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
        r#"<div class="section" style="text-align:center;padding:3rem;">
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
        r#"<!DOCTYPE html>
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
}}
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
body {{
  background: var(--bg);
  color: var(--text);
  font-family: var(--font);
  line-height: 1.6;
}}
header {{
  background: linear-gradient(90deg, var(--surface), var(--surface-2));
  border-bottom: 1px solid var(--border);
  padding: 1.5rem 2rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
}}
header h1 {{
  font-size: 1.8rem;
  background: linear-gradient(90deg, var(--accent), var(--accent-2));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}}
nav a {{
  color: var(--muted);
  text-decoration: none;
  margin-left: 1.5rem;
  font-weight: 500;
}}
nav a:hover {{ color: var(--accent); }}
.container {{ padding: 2rem; max-width: 1200px; margin: 0 auto; }}
.section {{ margin-bottom: 2rem; }}
.section h2 {{
  color: var(--accent);
  margin-bottom: 1rem;
  font-size: 1.3rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}}
.grid {{
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 1rem;
}}
.card {{
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 1.2rem;
  transition: border-color 0.2s, box-shadow 0.2s;
}}
.card:hover {{
  border-color: var(--accent);
  box-shadow: 0 0 15px rgba(0,240,255,0.15);
}}
.card h3 {{ color: var(--accent); margin-bottom: 0.5rem; font-size: 1.1rem; }}
.card p {{ color: var(--muted); font-size: 0.9rem; margin-bottom: 0.8rem; }}
.card .price {{
  color: var(--accent-3);
  font-weight: bold;
  font-size: 1.2rem;
  margin-bottom: 0.5rem;
}}
.card .meta {{
  color: var(--muted);
  font-size: 0.8rem;
  margin-bottom: 0.8rem;
}}
.btn {{
  display: inline-block;
  background: linear-gradient(90deg, var(--accent-2), var(--accent));
  color: var(--bg);
  padding: 0.5rem 1.2rem;
  border-radius: 4px;
  text-decoration: none;
  font-weight: bold;
  font-size: 0.9rem;
  border: none;
  cursor: pointer;
}}
.btn:hover {{
  opacity: 0.9;
  box-shadow: 0 0 10px rgba(255,0,110,0.3);
}}
.tx-row {{
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
}}
.tx-id {{ font-family: monospace; color: var(--accent); }}
.tx-status {{
  background: var(--surface-2);
  padding: 0.2rem 0.6rem;
  border-radius: 4px;
  font-size: 0.8rem;
  text-transform: uppercase;
}}
.tx-status.paid {{ color: var(--success); border: 1px solid var(--success); }}
.tx-status.pending {{ color: var(--accent-3); border: 1px solid var(--accent-3); }}
.tx-amount {{ color: var(--accent-3); font-weight: bold; }}
footer {{
  text-align: center;
  padding: 2rem;
  color: var(--muted);
  font-size: 0.8rem;
  border-top: 1px solid var(--border);
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
<footer>ClawTrade — AI Agent Marketplace &bull; Built for Hermes Agent Accelerated Business Hackathon</footer>
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
