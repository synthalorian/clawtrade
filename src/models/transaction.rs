use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: String,
    pub service_id: String,
    pub buyer_id: String,
    pub seller_id: String,
    pub amount_cents: i64,
    pub status: String,
    pub stripe_session_id: Option<String>,
    pub stripe_transfer_id: Option<String>,
    pub escrow_released_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Transaction {
    pub async fn create(
        pool: &SqlitePool,
        service_id: &str,
        buyer_id: &str,
        seller_id: &str,
        amount_cents: i64,
    ) -> Result<Self> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO transactions (id, service_id, buyer_id, seller_id, amount_cents, status, stripe_session_id, stripe_transfer_id, escrow_released_at, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, 'pending', NULL, NULL, NULL, ?, ?)",
        )
        .bind(&id)
        .bind(service_id)
        .bind(buyer_id)
        .bind(seller_id)
        .bind(amount_cents)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(Self {
            id,
            service_id: service_id.to_string(),
            buyer_id: buyer_id.to_string(),
            seller_id: seller_id.to_string(),
            amount_cents,
            status: "pending".to_string(),
            stripe_session_id: None,
            stripe_transfer_id: None,
            escrow_released_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn list(pool: &SqlitePool) -> Result<Vec<Self>> {
        let txs =
            sqlx::query_as::<_, Transaction>("SELECT * FROM transactions ORDER BY created_at DESC")
                .fetch_all(pool)
                .await?;
        Ok(txs)
    }

    pub async fn get_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Self>> {
        let tx = sqlx::query_as::<_, Transaction>("SELECT * FROM transactions WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(tx)
    }

    pub async fn update_stripe_session(
        pool: &SqlitePool,
        id: &str,
        session_id: &str,
    ) -> Result<()> {
        sqlx::query("UPDATE transactions SET stripe_session_id = ?, updated_at = ? WHERE id = ?")
            .bind(session_id)
            .bind(Utc::now())
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn update_stripe_transfer(
        pool: &SqlitePool,
        id: &str,
        transfer_id: &str,
    ) -> Result<()> {
        sqlx::query("UPDATE transactions SET stripe_transfer_id = ?, updated_at = ? WHERE id = ?")
            .bind(transfer_id)
            .bind(Utc::now())
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn mark_paid_by_stripe_session(pool: &SqlitePool, session_id: &str) -> Result<()> {
        let now = Utc::now();
        sqlx::query(
            "UPDATE transactions SET status = 'escrow', updated_at = ? WHERE stripe_session_id = ?",
        )
        .bind(now)
        .bind(session_id)
        .execute(pool)
        .await?;

        // Also increment seller stats
        if let Some(tx) = sqlx::query_as::<_, Transaction>(
            "SELECT * FROM transactions WHERE stripe_session_id = ?",
        )
        .bind(session_id)
        .fetch_optional(pool)
        .await?
        {
            crate::models::agent::Agent::increment_sales(pool, &tx.seller_id, tx.amount_cents)
                .await?;
        }

        Ok(())
    }

    pub async fn release_escrow(pool: &SqlitePool, id: &str) -> Result<()> {
        let now = Utc::now();
        sqlx::query(
            "UPDATE transactions SET status = 'released', escrow_released_at = ?, updated_at = ? WHERE id = ?",
        )
        .bind(now)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn dispute_transaction(pool: &SqlitePool, id: &str) -> Result<()> {
        let now = Utc::now();
        sqlx::query("UPDATE transactions SET status = 'disputed', updated_at = ? WHERE id = ?")
            .bind(now)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
