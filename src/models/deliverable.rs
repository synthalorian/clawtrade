use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Deliverable {
    pub id: String,
    pub transaction_id: String,
    pub service_type: String,
    pub input_data: Option<String>,
    pub output_data: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Deliverable {
    pub async fn create(
        pool: &SqlitePool,
        transaction_id: &str,
        service_type: &str,
        input_data: Option<&str>,
    ) -> Result<Self> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO deliverables (id, transaction_id, service_type, input_data, output_data, status, error_message, created_at, updated_at)
             VALUES (?, ?, ?, ?, NULL, 'pending', NULL, ?, ?)",
        )
        .bind(&id)
        .bind(transaction_id)
        .bind(service_type)
        .bind(input_data)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(Self {
            id,
            transaction_id: transaction_id.to_string(),
            service_type: service_type.to_string(),
            input_data: input_data.map(|s| s.to_string()),
            output_data: None,
            status: "pending".to_string(),
            error_message: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn update_output(
        pool: &SqlitePool,
        id: &str,
        output_data: &str,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE deliverables SET output_data = ?, status = 'completed', updated_at = ? WHERE id = ?",
        )
        .bind(output_data)
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn mark_failed(
        pool: &SqlitePool,
        id: &str,
        error: &str,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE deliverables SET status = 'failed', error_message = ?, updated_at = ? WHERE id = ?",
        )
        .bind(error)
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn get_by_transaction(pool: &SqlitePool, transaction_id: &str) -> Result<Option<Self>> {
        let d = sqlx::query_as::<_, Deliverable>(
            "SELECT * FROM deliverables WHERE transaction_id = ?",
        )
        .bind(transaction_id)
        .fetch_optional(pool)
        .await?;
        Ok(d)
    }

    #[allow(dead_code)]
    pub async fn list(pool: &SqlitePool) -> Result<Vec<Self>> {
        let ds = sqlx::query_as::<_, Deliverable>(
            "SELECT * FROM deliverables ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await?;
        Ok(ds)
    }
}
