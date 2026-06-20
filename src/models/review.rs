use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Review {
    pub id: String,
    pub transaction_id: String,
    pub agent_id: String,
    pub rating: i64,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Review {
    pub async fn create(
        pool: &SqlitePool,
        transaction_id: &str,
        agent_id: &str,
        rating: i64,
        comment: Option<&str>,
    ) -> Result<Self> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO reviews (id, transaction_id, agent_id, rating, comment, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(transaction_id)
        .bind(agent_id)
        .bind(rating)
        .bind(comment)
        .bind(now)
        .execute(pool)
        .await?;

        // Update agent avg_rating
        Self::update_agent_rating(pool, agent_id).await?;

        Ok(Self {
            id,
            transaction_id: transaction_id.to_string(),
            agent_id: agent_id.to_string(),
            rating,
            comment: comment.map(|s| s.to_string()),
            created_at: now,
        })
    }

    pub async fn list_by_agent(pool: &SqlitePool, agent_id: &str) -> Result<Vec<Self>> {
        let reviews = sqlx::query_as::<_, Review>(
            "SELECT * FROM reviews WHERE agent_id = ? ORDER BY created_at DESC",
        )
        .bind(agent_id)
        .fetch_all(pool)
        .await?;
        Ok(reviews)
    }

    pub async fn get_average_rating(pool: &SqlitePool, agent_id: &str) -> Result<Option<f64>> {
        let result: Option<(f64,)> = sqlx::query_as(
            "SELECT AVG(CAST(rating AS REAL)) FROM reviews WHERE agent_id = ?",
        )
        .bind(agent_id)
        .fetch_optional(pool)
        .await?;
        Ok(result.map(|r| r.0))
    }

    async fn update_agent_rating(pool: &SqlitePool, agent_id: &str) -> Result<()> {
        if let Some(avg) = Self::get_average_rating(pool, agent_id).await? {
            let score = (avg * 20.0) as i64; // Convert 5-star to 0-100 reputation_score
            sqlx::query(
                "UPDATE agents SET reputation_score = ? WHERE id = ?",
            )
            .bind(score)
            .bind(agent_id)
            .execute(pool)
            .await?;
        }
        Ok(())
    }

    pub async fn count_by_agent(pool: &SqlitePool, agent_id: &str) -> Result<i64> {
        let result: Option<(i64,)> = sqlx::query_as(
            "SELECT COUNT(*) FROM reviews WHERE agent_id = ?",
        )
        .bind(agent_id)
        .fetch_optional(pool)
        .await?;
        Ok(result.map(|r| r.0).unwrap_or(0))
    }
}
