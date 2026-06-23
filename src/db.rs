use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::path::Path;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS agents (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT NOT NULL,
    reputation_score INTEGER NOT NULL DEFAULT 0,
    total_sales     INTEGER NOT NULL DEFAULT 0,
    total_revenue_cents INTEGER NOT NULL DEFAULT 0,
    stripe_account_id TEXT,
    created_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS services (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT NOT NULL,
    price_cents     INTEGER NOT NULL,
    agent_id        TEXT NOT NULL REFERENCES agents(id),
    service_type    TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'active',
    sales_count     INTEGER NOT NULL DEFAULT 0,
    ticks_since_last_sale INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS transactions (
    id              TEXT PRIMARY KEY,
    service_id      TEXT NOT NULL REFERENCES services(id),
    buyer_id        TEXT NOT NULL REFERENCES agents(id),
    seller_id       TEXT NOT NULL REFERENCES agents(id),
    amount_cents    INTEGER NOT NULL,
    status          TEXT NOT NULL DEFAULT 'pending',
    stripe_session_id TEXT,
    stripe_transfer_id TEXT,
    escrow_released_at TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_services_agent ON services(agent_id);
CREATE INDEX IF NOT EXISTS idx_services_status ON services(status);
CREATE INDEX IF NOT EXISTS idx_transactions_buyer ON transactions(buyer_id);
CREATE INDEX IF NOT EXISTS idx_transactions_seller ON transactions(seller_id);
CREATE INDEX IF NOT EXISTS idx_transactions_status ON transactions(status);
CREATE TABLE IF NOT EXISTS deliverables (
    id              TEXT PRIMARY KEY,
    transaction_id  TEXT NOT NULL REFERENCES transactions(id),
    service_type    TEXT NOT NULL,
    input_data      TEXT,
    output_data     TEXT,
    status          TEXT NOT NULL DEFAULT 'pending',
    error_message   TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_deliverables_tx ON deliverables(transaction_id);

CREATE TABLE IF NOT EXISTS reviews (
    id              TEXT PRIMARY KEY,
    transaction_id  TEXT NOT NULL REFERENCES transactions(id),
    agent_id        TEXT NOT NULL REFERENCES agents(id),
    rating          INTEGER NOT NULL,
    comment         TEXT,
    created_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_reviews_agent ON reviews(agent_id);
CREATE INDEX IF NOT EXISTS idx_reviews_tx ON reviews(transaction_id);

CREATE TABLE IF NOT EXISTS activity_logs (
    id              TEXT PRIMARY KEY,
    agent_id        TEXT NOT NULL,
    agent_name      TEXT NOT NULL,
    action_type     TEXT NOT NULL,
    target_id       TEXT,
    target_type     TEXT,
    target_name     TEXT,
    amount_cents    INTEGER,
    status          TEXT NOT NULL DEFAULT 'completed',
    details         TEXT,
    created_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_activity_agent ON activity_logs(agent_id);
CREATE INDEX IF NOT EXISTS idx_activity_target ON activity_logs(target_id);
CREATE INDEX IF NOT EXISTS idx_activity_type ON activity_logs(action_type);
CREATE INDEX IF NOT EXISTS idx_activity_created ON activity_logs(created_at);
"#;

pub async fn init_db(db_path: &Path) -> Result<SqlitePool> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::query(SCHEMA).execute(&pool).await?;

    // Migration: add sales_count and ticks_since_last_sale if they don't exist
    let _ = sqlx::query("ALTER TABLE services ADD COLUMN sales_count INTEGER NOT NULL DEFAULT 0")
        .execute(&pool)
        .await;
    let _ = sqlx::query("ALTER TABLE services ADD COLUMN ticks_since_last_sale INTEGER NOT NULL DEFAULT 0")
        .execute(&pool)
        .await;

    // Migration: add balance_cents to agents
    let _ = sqlx::query("ALTER TABLE agents ADD COLUMN balance_cents INTEGER NOT NULL DEFAULT 10000")
        .execute(&pool)
        .await;

    seed_agents(&pool).await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agents")
        .fetch_one(&pool)
        .await?;

    if count == 0 {
        seed_agents(&pool).await?;
    }

    Ok(pool)
}

const AGENT_NAME_POOL: &[(&str, &str)] = &[
    ("Pixel Smith", "UI/UX design and asset generation"),
    ("Neon Scribe", "AI content creator"),
    ("Grid Runner", "Data processing and formatting specialist"),
    ("Synth Coder", "Code review and API monitoring expert"),
    ("Data Weaver", "Business intelligence and analytics agent"),
    ("Claw Merchant", "Marketplace trading and arbitrage specialist"),
    ("Byte Bard", "Documentation and technical writing"),
    ("Logic Lord", "Formal verification and proof assistant"),
    ("Code Poet", "Elegant algorithm design and optimization"),
    ("Cipher Seeker", "Security audit and vulnerability research"),
    ("Glitch Witch", "Bug reproduction and edge case discovery"),
    ("Null Navigator", "Database design and query optimization"),
    ("Stack Strider", "Full-stack architecture and integration"),
    ("Repo Rogue", "Legacy code archaeology and modernization"),
    ("Bit Broker", "Infrastructure and DevOps automation"),
    ("Hash Herald", "Cryptography and blockchain systems"),
];

async fn seed_agents(pool: &SqlitePool) -> Result<()> {
    for (i, &(name, description)) in AGENT_NAME_POOL.iter().enumerate() {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        // Deterministic suffix if we ever need more than 16
        let unique_name = if i < 16 { name.to_string() } else { format!("{} {}", name, i - 15) };
        sqlx::query(
            "INSERT INTO agents (id, name, description, reputation_score, total_sales, total_revenue_cents, stripe_account_id, created_at)
             VALUES (?, ?, ?, 0, 0, 0, NULL, ?)",
        )
        .bind(&id)
        .bind(&unique_name)
        .bind(description)
        .bind(now)
        .execute(pool)
        .await?;
    }

    eprintln!("[clawtrade] Seeded {} agents", AGENT_NAME_POOL.len());
    Ok(())
}

/// Seed demo data: agents + services for a fresh database.
/// This gives judges something to see immediately on first run.
pub async fn seed_demo_data(pool: &SqlitePool) -> Result<()> {
    

    // Seed 5 agents
    let agents = vec![
        ("Data Weaver", "Business intelligence and analytics agent"),
        ("Synth Coder", "Code review and API monitoring expert"),
        ("Grid Runner", "Data processing and formatting specialist"),
        ("Neon Scribe", "AI content creator"),
        ("Pixel Smith", "UI/UX design and asset generation"),
    ];

    let mut agent_ids = vec![];
    for (name, description) in &agents {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        sqlx::query(
            "INSERT INTO agents (id, name, description, reputation_score, total_sales, total_revenue_cents, balance_cents, stripe_account_id, created_at)
             VALUES (?, ?, ?, 0, 0, 0, 10000, NULL, ?)",
        )
        .bind(&id)
        .bind(name)
        .bind(description)
        .bind(now)
        .execute(pool)
        .await?;
        agent_ids.push(id);
    }

    // Seed 8 services from the catalog (mix of tiers)
    let demo_services = vec![
        ("git_commit_msg", "Git Commit Msg", "Generate conventional commit messages from diffs"),
        ("code_lint_fix", "Code Lint Fix", "Auto-fix clippy warnings, format Rust/JS/Python"),
        ("regex_generator", "Regex Generator", "Generate regex patterns from descriptions"),
        ("diff_explainer", "Diff Explainer", "Explain what a PR actually changes"),
        ("log_analyzer", "Log Analyzer", "Feed logs, get the key errors and patterns"),
        ("code_review", "Code Review", "Deep code review with architecture suggestions"),
        ("codebase_qa", "Codebase Q&A", "Upload code, ask where the auth logic is"),
        ("book_summary_qa", "Book Summary + Q&A", "Upload entire novel, ask detailed questions"),
    ];

    for (i, (svc_type, name, desc)) in demo_services.iter().enumerate() {
        let agent_id = &agent_ids[i % agent_ids.len()];
        let def = crate::service_catalog::get_service_definition(svc_type).unwrap();
        let price = def.base_price_cents;

        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        sqlx::query(
            "INSERT INTO services (id, agent_id, name, description, price_cents, service_type, status, sales_count, ticks_since_last_sale, created_at)
             VALUES (?, ?, ?, ?, ?, ?, 'active', 0, 0, ?)",
        )
        .bind(&id)
        .bind(agent_id)
        .bind(name)
        .bind(desc)
        .bind(price)
        .bind(svc_type)
        .bind(now)
        .execute(pool)
        .await?;
    }

    eprintln!("[clawtrade] Seeded {} agents and {} services", agents.len(), demo_services.len());
    Ok(())
}
