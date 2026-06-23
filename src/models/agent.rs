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
    pub balance_cents: i64,
    pub stripe_account_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Agent {
    pub async fn create(pool: &SqlitePool, name: &str, description: &str) -> Result<Self> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO agents (id, name, description, reputation_score, total_sales, total_revenue_cents, stripe_account_id, created_at)
             VALUES (?, ?, ?, 0, 0, 0, NULL, ?)",
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
            balance_cents: 10000, // $100 starting budget
            stripe_account_id: None,
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

    pub async fn list_top(pool: &SqlitePool) -> Result<Vec<Self>> {
        let agents = sqlx::query_as::<_, Agent>(
            "SELECT * FROM agents ORDER BY total_sales DESC, reputation_score DESC LIMIT 8",
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

    pub async fn update_stripe_account(pool: &SqlitePool, id: &str, stripe_account_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE agents SET stripe_account_id = ? WHERE id = ?",
        )
        .bind(stripe_account_id)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Deduct from buyer's balance on purchase
    pub async fn deduct_balance(pool: &SqlitePool, id: &str, amount_cents: i64) -> Result<()> {
        sqlx::query(
            "UPDATE agents SET balance_cents = balance_cents - ? WHERE id = ? AND balance_cents >= ?",
        )
        .bind(amount_cents)
        .bind(id)
        .bind(amount_cents)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Add revenue to seller's balance on sale
    pub async fn add_revenue(pool: &SqlitePool, id: &str, amount_cents: i64) -> Result<()> {
        sqlx::query(
            "UPDATE agents SET balance_cents = balance_cents + ?, total_revenue_cents = total_revenue_cents + ? WHERE id = ?",
        )
        .bind(amount_cents)
        .bind(amount_cents)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Get agent by ID, or create a guest agent if not found
    pub async fn get_or_create_guest(pool: &SqlitePool, id: &str) -> Result<Self> {
        match Self::get_by_id(pool, id).await? {
            Some(agent) => Ok(agent),
            None => {
                let now = Utc::now();
                sqlx::query(
                    "INSERT INTO agents (id, name, description, reputation_score, total_sales, total_revenue_cents, balance_cents, stripe_account_id, created_at)
                     VALUES (?, ?, ?, 0, 0, 0, 10000, NULL, ?)",
                )
                .bind(id)
                .bind("Guest Buyer")
                .bind("Auto-created guest buyer")
                .bind(now)
                .execute(pool)
                .await?;

                Ok(Self {
                    id: id.to_string(),
                    name: "Guest Buyer".to_string(),
                    description: "Auto-created guest buyer".to_string(),
                    reputation_score: 0,
                    total_sales: 0,
                    total_revenue_cents: 0,
                    balance_cents: 10000,
                    stripe_account_id: None,
                    created_at: now,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create test DB");
        
        sqlx::query(
            r#"
            CREATE TABLE agents (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                reputation_score INTEGER NOT NULL DEFAULT 0,
                total_sales INTEGER NOT NULL DEFAULT 0,
                total_revenue_cents INTEGER NOT NULL DEFAULT 0,
                balance_cents INTEGER NOT NULL DEFAULT 10000,
                stripe_account_id TEXT,
                created_at TEXT NOT NULL
            )
            "#
        )
        .execute(&pool)
        .await
        .unwrap();
        
        pool
    }

    #[tokio::test]
    async fn test_agent_create() {
        let pool = setup_test_db().await;
        let agent = Agent::create(&pool, "Test Agent", "A test agent").await.unwrap();
        
        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.description, "A test agent");
        assert_eq!(agent.balance_cents, 10000);
        assert_eq!(agent.reputation_score, 0);
    }

    #[tokio::test]
    async fn test_agent_get_by_id() {
        let pool = setup_test_db().await;
        let agent = Agent::create(&pool, "Test Agent", "A test agent").await.unwrap();
        
        let fetched = Agent::get_by_id(&pool, &agent.id).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().name, "Test Agent");
    }

    #[tokio::test]
    async fn test_agent_balance_update() {
        let pool = setup_test_db().await;
        let agent = Agent::create(&pool, "Test Agent", "A test agent").await.unwrap();
        
        // Deduct balance
        Agent::deduct_balance(&pool, &agent.id, 500).await.unwrap();
        let updated = Agent::get_by_id(&pool, &agent.id).await.unwrap().unwrap();
        assert_eq!(updated.balance_cents, 9500);
        
        // Add revenue
        Agent::add_revenue(&pool, &agent.id, 200).await.unwrap();
        let updated = Agent::get_by_id(&pool, &agent.id).await.unwrap().unwrap();
        assert_eq!(updated.balance_cents, 9700);
        assert_eq!(updated.total_revenue_cents, 200);
    }

    #[tokio::test]
    async fn test_agent_deduct_balance_insufficient() {
        let pool = setup_test_db().await;
        let agent = Agent::create(&pool, "Test Agent", "A test agent").await.unwrap();
        
        // Try to deduct more than balance — should silently fail (no rows updated)
        Agent::deduct_balance(&pool, &agent.id, 20000).await.unwrap();
        let updated = Agent::get_by_id(&pool, &agent.id).await.unwrap().unwrap();
        assert_eq!(updated.balance_cents, 10000); // unchanged
    }
}
