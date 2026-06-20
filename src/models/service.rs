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
            created_at: now,
        })
    }

    pub async fn list(pool: &SqlitePool) -> Result<Vec<Self>> {
        let services = sqlx::query_as::<_, Service>(
            "SELECT * FROM services WHERE status = 'active' ORDER BY created_at DESC",
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
}
