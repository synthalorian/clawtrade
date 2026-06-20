use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub reputation_score: i64,
    pub total_sales: i64,
    pub total_revenue_cents: i64,
    pub created_at: DateTime<Utc>,
}

impl Agent {
    pub async fn create(pool: &SqlitePool, name: &str, description: &str) -> Result<Self> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO agents (id, name, description, reputation_score, total_sales, total_revenue_cents, created_at)
             VALUES (?, ?, ?, 0, 0, 0, ?)",
        )
        .bind(&id)
        .bind(name)
        .bind(description)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(Self {
            id,
            name: name.to_string(),
            description: description.to_string(),
            reputation_score: 0,
            total_sales: 0,
            total_revenue_cents: 0,
            created_at: now,
        })
    }

    pub async fn list(pool: &SqlitePool) -> Result<Vec<Self>> {
        let agents = sqlx::query_as::<_, Agent>(
            "SELECT * FROM agents ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await?;
        Ok(agents)
    }

    pub async fn get_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Self>> {
        let agent = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(agent)
    }

    pub async fn increment_sales(pool: &SqlitePool, id: &str, amount_cents: i64) -> Result<()> {
        sqlx::query(
            "UPDATE agents SET total_sales = total_sales + 1, total_revenue_cents = total_revenue_cents + ? WHERE id = ?",
        )
        .bind(amount_cents)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }
}
