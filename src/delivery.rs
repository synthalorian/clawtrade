use anyhow::Result;
use sqlx::SqlitePool;

use crate::models::deliverable::Deliverable;
use crate::models::service::Service;
use crate::models::transaction::Transaction;
use crate::nvidia::LlmClient;

/// Trigger delivery when a transaction is marked as paid
pub async fn trigger_delivery(pool: &SqlitePool, transaction_id: &str) -> Result<()> {
    let tx = match Transaction::get_by_id(pool, transaction_id).await? {
        Some(t) => t,
        None => {
            anyhow::bail!("transaction not found: {}", transaction_id);
        }
    };

    let service = match Service::get_by_id(pool, &tx.service_id).await? {
        Some(s) => s,
        None => {
            anyhow::bail!("service not found: {}", tx.service_id);
        }
    };

    // Create deliverable record
    let input_data = Some(format!(
        "Service: {} | Type: {} | Buyer: {} | Amount: ${}.{}",
        service.name,
        service.service_type,
        tx.buyer_id,
        tx.amount_cents / 100,
        tx.amount_cents % 100
    ));

    let deliverable = Deliverable::create(
        pool,
        transaction_id,
        &service.service_type,
        input_data.as_deref(),
    )
    .await?;

    // Execute delivery based on service type
    let result = match service.service_type.as_str() {
        "text_processing" => deliver_text_processing(&service, &tx).await,
        "data_formatting" => deliver_data_formatting(&service, &tx).await,
        "api_monitor" => deliver_api_monitor(&service, &tx).await,
        _ => deliver_generic(&service, &tx).await,
    };

    match result {
        Ok(output) => {
            Deliverable::update_output(pool, &deliverable.id, &output).await?;
            Ok(())
        }
        Err(e) => {
            Deliverable::mark_failed(pool, &deliverable.id, &e.to_string()).await?;
            Err(e)
        }
    }
}

async fn deliver_text_processing(service: &Service, tx: &Transaction) -> Result<String> {
    let client = LlmClient::new();
    let prompt = format!(
        "Summarize the following service request and provide a detailed response. \
        Service: {}. Description: {}. Buyer: {}. Amount: ${}.{}",
        service.name,
        service.description,
        tx.buyer_id,
        tx.amount_cents / 100,
        tx.amount_cents % 100
    );

    match client.summarize(&prompt).await {
        Ok(result) => Ok(format!(
            "=== Text Processing Result ===\n\n{}",
            result
        )),
        Err(e) => Ok(format!(
            "=== Text Processing Result (Local Fallback) ===\n\n\
            Service: {}\n\
            Description: {}\n\
            This is a simulated text processing result. In production, this would call \
            the NVIDIA Nemotron 3 Ultra API or your local llama-swap instance.\n\n\
            Error from LLM: {}",
            service.name, service.description, e
        )),
    }
}

async fn deliver_data_formatting(service: &Service, _tx: &Transaction) -> Result<String> {
    let sample_json = serde_json::json!({
        "service": service.name,
        "description": service.description,
        "price_cents": service.price_cents,
        "service_type": service.service_type,
        "status": "delivered",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let pretty = serde_json::to_string_pretty(&sample_json)?;
    Ok(format!(
        "=== Data Formatting Result ===\n\n{}",
        pretty
    ))
}

async fn deliver_api_monitor(service: &Service, _tx: &Transaction) -> Result<String> {
    let client = reqwest::Client::new();
    let target_url = "https://httpbin.org/get";

    let start = std::time::Instant::now();
    let response = match client.get(target_url).send().await {
        Ok(r) => r,
        Err(e) => {
            return Ok(format!(
                "=== API Monitor Result ===\n\n\
                Target: {}\n\
                Status: FAILED\n\
                Error: {}\n\
                Note: This is a demo API ping. In production, the agent would monitor \
                the buyer's specified endpoint.",
                target_url, e
            ));
        }
    };
    let latency_ms = start.elapsed().as_millis();
    let status = response.status().as_u16();

    Ok(format!(
        "=== API Monitor Result ===\n\n\
        Target: {}\n\
        Status: {}\n\
        Latency: {}ms\n\
        Timestamp: {}\n\n\
        This is a live API ping. The agent successfully reached the target endpoint.",
        target_url,
        status,
        latency_ms,
        chrono::Utc::now().to_rfc3339()
    ))
}

async fn deliver_generic(service: &Service, _tx: &Transaction) -> Result<String> {
    Ok(format!(
        "=== Generic Service Delivery ===\n\n\
        Service '{}' has been delivered.\n\
        Type: {}\n\
        Description: {}\n\
        This is a generic delivery placeholder. Implement custom logic for service type '{}'.",
        service.name,
        service.service_type,
        service.description,
        service.service_type
    ))
}
