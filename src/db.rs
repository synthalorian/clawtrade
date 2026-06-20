use anyhow::Result;
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
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_services_agent ON services(agent_id);
CREATE INDEX IF NOT EXISTS idx_services_status ON services(status);
CREATE INDEX IF NOT EXISTS idx_transactions_buyer ON transactions(buyer_id);
CREATE INDEX IF NOT EXISTS idx_transactions_seller ON transactions(seller_id);
CREATE INDEX IF NOT EXISTS idx_transactions_status ON transactions(status);
CREATE INDEX IF NOT EXISTS idx_transactions_stripe ON transactions(stripe_session_id);
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
    Ok(pool)
}
