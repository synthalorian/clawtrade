use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ActivityLog {
    pub id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub action_type: String,
    pub target_id: Option<String>,
    pub target_type: Option<String>,
    pub target_name: Option<String>,
    pub amount_cents: Option<i64>,
    pub status: String,
    pub details: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl ActivityLog {
    pub async fn create(
        pool: &SqlitePool,
        agent_id: &str,
        agent_name: &str,
        action_type: &str,
        target_id: Option<&str>,
        target_type: Option<&str>,
        target_name: Option<&str>,
        amount_cents: Option<i64>,
        status: &str,
        details: Option<&str>,
    ) -> Result<Self> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO activity_logs (id, agent_id, agent_name, action_type, target_id, target_type, target_name, amount_cents, status, details, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(agent_id)
        .bind(agent_name)
        .bind(action_type)
        .bind(target_id)
        .bind(target_type)
        .bind(target_name)
        .bind(amount_cents)
        .bind(status)
        .bind(details)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(Self {
            id,
            agent_id: agent_id.to_string(),
            agent_name: agent_name.to_string(),
            action_type: action_type.to_string(),
            target_id: target_id.map(|s| s.to_string()),
            target_type: target_type.map(|s| s.to_string()),
            target_name: target_name.map(|s| s.to_string()),
            amount_cents,
            status: status.to_string(),
            details: details.map(|s| s.to_string()),
            created_at: now,
        })
    }

    pub async fn list_global(pool: &SqlitePool, limit: i64) -> Result<Vec<Self>> {
        let logs = sqlx::query_as::<_, ActivityLog>(
            "SELECT * FROM activity_logs ORDER BY created_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;
        Ok(logs)
    }

    pub async fn list_by_agent(pool: &SqlitePool, agent_id: &str, limit: i64) -> Result<Vec<Self>> {
        let logs = sqlx::query_as::<_, ActivityLog>(
            "SELECT * FROM activity_logs WHERE agent_id = ? ORDER BY created_at DESC LIMIT ?",
        )
        .bind(agent_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;
        Ok(logs)
    }

    pub async fn list_by_target(pool: &SqlitePool, target_id: &str, limit: i64) -> Result<Vec<Self>> {
        let logs = sqlx::query_as::<_, ActivityLog>(
            "SELECT * FROM activity_logs WHERE target_id = ? ORDER BY created_at DESC LIMIT ?",
        )
        .bind(target_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;
        Ok(logs)
    }

    pub async fn get_stats(pool: &SqlitePool) -> Result<ActivityStats> {
        let total_actions: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM activity_logs")
            .fetch_one(pool)
            .await?;

        let total_purchases: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM activity_logs WHERE action_type = 'purchase'",
        )
        .fetch_one(pool)
        .await?;

        let total_reviews: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM activity_logs WHERE action_type = 'review'",
        )
        .fetch_one(pool)
        .await?;

        let total_services_created: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM activity_logs WHERE action_type = 'create_service'",
        )
        .fetch_one(pool)
        .await?;

        let total_volume_cents: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount_cents), 0) FROM activity_logs WHERE action_type = 'purchase'",
        )
        .fetch_one(pool)
        .await?;

        let top_agent: Option<(String, i64)> = sqlx::query_as(
            "SELECT agent_name, COUNT(*) as count FROM activity_logs GROUP BY agent_id ORDER BY count DESC LIMIT 1",
        )
        .fetch_optional(pool)
        .await?;

        Ok(ActivityStats {
            total_actions,
            total_purchases,
            total_reviews,
            total_services_created,
            total_volume_cents,
            top_agent,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ActivityStats {
    pub total_actions: i64,
    pub total_purchases: i64,
    pub total_reviews: i64,
    pub total_services_created: i64,
    pub total_volume_cents: i64,
    pub top_agent: Option<(String, i64)>,
}
