use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price_cents: i64,
    pub agent_id: String,
    pub service_type: String,
    pub status: String,
    pub sales_count: i64,
    pub ticks_since_last_sale: i64,
    pub created_at: DateTime<Utc>,
}

impl Service {
    pub async fn create(
        pool: &SqlitePool,
        name: &str,
        description: &str,
        price_cents: i64,
        agent_id: &str,
        service_type: &str,
    ) -> Result<Self> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO services (id, name, description, price_cents, agent_id, service_type, status, created_at)
             VALUES (?, ?, ?, ?, ?, ?, 'active', ?)",
        )
        .bind(&id)
        .bind(name)
        .bind(description)
        .bind(price_cents)
        .bind(agent_id)
        .bind(service_type)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(Self {
            id,
            name: name.to_string(),
            description: description.to_string(),
            price_cents,
            agent_id: agent_id.to_string(),
            service_type: service_type.to_string(),
            status: "active".to_string(),
            sales_count: 0,
            ticks_since_last_sale: 0,
            created_at: now,
        })
    }

    pub async fn list(pool: &SqlitePool) -> Result<Vec<Self>> {
        let services =
            sqlx::query_as::<_, Service>("SELECT * FROM services ORDER BY created_at DESC")
                .fetch_all(pool)
                .await?;
        Ok(services)
    }

    pub async fn list_active(pool: &SqlitePool) -> Result<Vec<Self>> {
        let services = sqlx::query_as::<_, Service>(
            "SELECT * FROM services WHERE status = 'active' ORDER BY created_at DESC LIMIT 12",
        )
        .fetch_all(pool)
        .await?;
        Ok(services)
    }

    pub async fn get_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Self>> {
        let service = sqlx::query_as::<_, Service>("SELECT * FROM services WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(service)
    }

    /// Increment sales count and reset tick counter
    pub async fn record_sale(pool: &SqlitePool, service_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE services SET sales_count = sales_count + 1, ticks_since_last_sale = 0 WHERE id = ?"
        )
        .bind(service_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Increment tick counter for all active services
    pub async fn increment_tick_counters(pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            "UPDATE services SET ticks_since_last_sale = ticks_since_last_sale + 1 WHERE status = 'active'"
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Retire services that haven't sold in N ticks
    pub async fn retire_stale_services(pool: &SqlitePool, max_ticks: i64) -> Result<Vec<String>> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            "UPDATE services SET status = 'retired' WHERE status = 'active' AND ticks_since_last_sale >= ? RETURNING id, name"
        )
        .bind(max_ticks)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, name)| format!("{} ({})", name, id))
            .collect())
    }
}
