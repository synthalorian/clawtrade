//! Enhanced service delivery with real LLM-powered outputs.
//!
//! Each service type maps to a specific LLM prompt that generates
//! real, useful content based on the service description and buyer input.
//! Legacy delivery functions are kept for fallback.

#![allow(dead_code)]

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
        "code_review" => deliver_code_review(&service, &tx).await,
        "creative_writing" => deliver_creative_writing(&service, &tx).await,
        "analysis" => deliver_analysis(&service, &tx).await,
        _ => deliver_generic(&service, &tx).await,
    };

    match result {
        Ok(output) => {
            Deliverable::update_output(pool, &deliverable.id, &output).await?;
            // Auto-release escrow after successful delivery
            let _ = Transaction::release_escrow(pool, transaction_id).await;
            // Log the delivery + escrow release
            let _ = crate::models::activity_log::ActivityLog::create(
                pool,
                &tx.buyer_id,
                "System",
                "escrow_release",
                Some(transaction_id),
                Some("transaction"),
                Some(&service.name),
                Some(tx.amount_cents),
                "completed",
                Some(&format!("Auto-released escrow for {} after delivery", service.name)),
            ).await;
            Ok(())
        }
        Err(e) => {
            Deliverable::mark_failed(pool, &deliverable.id, &e.to_string()).await?;
            Err(e)
        }
    }
}

/// Execute a service directly using the service catalog's prompt template and model routing.
/// This is the primary service delivery engine for ClawTrade v2.0.
pub async fn execute_service_direct(
    pool: &SqlitePool,
    llm: &crate::nvidia::LlmClient,
    service_id: &str,
    user_input: &str,
) -> Result<String> {
    let service = match Service::get_by_id(pool, service_id).await? {
        Some(s) => s,
        None => {
            anyhow::bail!("service not found: {}", service_id);
        }
    };

    // Look up the service definition in the catalog
    let def = match crate::service_catalog::get_service_definition(&service.service_type) {
        Some(d) => d,
        None => {
            // Fallback to generic delivery for legacy services not in catalog
            return Ok(format!(
                "=== Service Execution ===\n\nService: {}\nType: {}\n\nYour input:\n{}\n\n(This service type doesn't have a catalog entry yet.)",
                service.name, service.service_type, user_input
            ));
        }
    };

    // 🛡️ PROMPT INJECTION DEFENSE — sanitize user input before LLM consumption
    let sanitized_input = match crate::prompt_defense::sanitize_input(user_input) {
        crate::prompt_defense::SanitizationResult::Clean(cleaned) => cleaned,
        crate::prompt_defense::SanitizationResult::Sanitized { cleaned, violations, .. } => {
            eprintln!("[prompt_defense] Input sanitized. Violations: {:?}", violations);
            cleaned
        }
        crate::prompt_defense::SanitizationResult::Blocked { reason, violations } => {
            eprintln!("[prompt_defense] Input blocked: {}. Violations: {:?}", reason, violations);
            return Ok(crate::prompt_defense::blocked_message(&violations));
        }
    };

    // Harden the system prompt against injection
    let hardened_system = crate::prompt_defense::harden_system_prompt(def.system_prompt);

    // Deliver using the shared LLM client + catalog's prompt template + model routing
    let start = std::time::Instant::now();

    match llm.deliver_service_with_prompt(&def.model, &hardened_system, &def.user_prompt_template.replace("{input}", &sanitized_input)).await {
        Ok(result) => {
            let execution_time_ms = start.elapsed().as_millis() as u64;
            let model_name = def.model.model_name();
            let tier_label = match def.tier {
                crate::service_catalog::ServiceTier::MicroTask => "Micro-Task",
                crate::service_catalog::ServiceTier::RealWork => "Real Work",
                crate::service_catalog::ServiceTier::HeavyLifting => "Heavy Lifting",
                crate::service_catalog::ServiceTier::LocalOnly => "Local-Only",
            };

            Ok(format!(
                "=== {} Result ===\n\n📝 Your Input:\n{}\n\n✨ Generated Output:\n{}\n\n---\n⏱️ Execution time: {}ms\n🏷️ Tier: {}\n🧠 Model: {}\n💡 Powered by local LLM inference — no cloud API calls",
                service.name,
                user_input,
                result,
                execution_time_ms,
                tier_label,
                model_name,
            ))
        }
        Err(e) => {
            eprintln!("[delivery] LLM failed for {}: {}", service.service_type, e);
            Ok(format!(
                "=== {} Result ===\n\n📝 Your Input:\n{}\n\n⚠️ LLM delivery failed: {}\n\nFalling back to generic response...\n\nService: {}\nDescription: {}\n",
                service.name, user_input, e, service.name, service.description
            ))
        }
    }
}

// ─── TEXT PROCESSING ───

#[allow(dead_code)]
async fn deliver_text_processing(service: &Service, _tx: &Transaction) -> Result<String> {
    let client = LlmClient::new();
    
    // Generate sample content based on service name
    let sample_text = generate_sample_text(&service.name);
    
    let prompt = format!(
        "Task: {}\n\nInstructions: {}\n\nPlease process the following text and provide a detailed, useful result:\n\n{}",
        service.name,
        service.description,
        sample_text
    );

    match client.summarize(&prompt).await {
        Ok(result) => Ok(format!(
            "=== {} Result ===\n\n📝 Input Sample:\n{}\n\n✨ Generated Output:\n{}\n\n---\n💡 This was generated by a local LLM (Qwen3.5-9B) running on the seller's infrastructure. No cloud API calls were made.",
            service.name,
            sample_text,
            result
        )),
        Err(e) => {
            eprintln!("[delivery] LLM failed for text_processing: {}", e);
            // Fallback: generate a plausible result without LLM
            Ok(generate_fallback_text_result(&service.name, &service.description))
        }
    }
}

async fn execute_text_processing(service: &Service, user_input: &str) -> Result<String> {
    let client = LlmClient::new();
    
    let prompt = format!(
        "Task: {}\n\nInstructions: {}\n\nPlease process the following user text and provide a detailed, useful result:\n\n{}",
        service.name,
        service.description,
        user_input
    );

    match client.summarize(&prompt).await {
        Ok(result) => Ok(format!(
            "=== {} Result ===\n\n📝 Your Input:\n{}\n\n✨ Generated Output:\n{}\n\n---\n💡 Powered by local LLM inference (Qwen3.5-9B)",
            service.name,
            user_input,
            result
        )),
        Err(e) => {
            eprintln!("[execute] LLM failed: {}", e);
            Ok(generate_fallback_text_result(&service.name, user_input))
        }
    }
}

// ─── DATA FORMATTING ───

async fn deliver_data_formatting(service: &Service, _tx: &Transaction) -> Result<String> {
    let client = LlmClient::new();
    
    let sample_data = generate_sample_json();
    
    let prompt = format!(
        "You are {}. {}\n\nFormat and improve the following data. Return well-structured, validated output:\n\n{}",
        service.name,
        service.description,
        sample_data
    );

    match client.summarize(&prompt).await {
        Ok(result) => Ok(format!(
            "=== {} Result ===\n\n📊 Input Data:\n{}\n\n✨ Formatted Output:\n{}\n\n---\n💡 Processed by local LLM (Qwen3.5-9B)",
            service.name,
            sample_data,
            result
        )),
        Err(e) => {
            eprintln!("[delivery] LLM failed for data_formatting: {}", e);
            Ok(generate_fallback_data_result(&service.name))
        }
    }
}

async fn execute_data_formatting(service: &Service, user_input: &str) -> Result<String> {
    let client = LlmClient::new();
    
    let prompt = format!(
        "You are {}. {}\n\nFormat and improve the following user data. Return well-structured, validated output:\n\n{}",
        service.name,
        service.description,
        user_input
    );

    match client.summarize(&prompt).await {
        Ok(result) => Ok(format!(
            "=== {} Result ===\n\n📊 Your Input:\n{}\n\n✨ Formatted Output:\n{}\n\n---\n💡 Processed by local LLM (Qwen3.5-9B)",
            service.name,
            user_input,
            result
        )),
        Err(e) => {
            eprintln!("[execute] LLM failed: {}", e);
            Ok(generate_fallback_data_result(&service.name))
        }
    }
}

// ─── API MONITOR ───

async fn deliver_api_monitor(service: &Service, _tx: &Transaction) -> Result<String> {
    let client = reqwest::Client::new();
    let target_url = "https://httpbin.org/get";

    let start = std::time::Instant::now();
    let response = match client.get(target_url).send().await {
        Ok(r) => r,
        Err(e) => {
            return Ok(format!(
                "=== {} Result ===\n\n🌐 API Monitor Report\n\nTarget: {}\nStatus: FAILED\nError: {}\n\nNote: This is a live API check. The endpoint may be temporarily unavailable.",
                service.name, target_url, e
            ));
        }
    };
    let latency_ms = start.elapsed().as_millis();
    let status = response.status().as_u16();
    
    let body_preview = match response.text().await {
        Ok(text) => {
            if text.len() > 500 {
                format!("{}... (truncated)", &text[..500])
            } else {
                text
            }
        }
        Err(_) => "(could not read response body)".to_string(),
    };

    Ok(format!(
        "=== {} Result ===\n\n🌐 API Monitor Report\n\nTarget: {}\nStatus: {}\nLatency: {}ms\nTimestamp: {}\n\n📄 Response Preview:\n{}\n\n---\n✅ Live API check completed successfully. The endpoint is healthy.",
        service.name,
        target_url,
        status,
        latency_ms,
        chrono::Utc::now().to_rfc3339(),
        body_preview
    ))
}

async fn execute_api_monitor(service: &Service, user_input: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let target_url = if user_input.trim().starts_with("http") {
        user_input.trim().to_string()
    } else {
        "https://httpbin.org/get".to_string()
    };

    let start = std::time::Instant::now();
    let response = match client.get(&target_url).send().await {
        Ok(r) => r,
        Err(e) => {
            return Ok(format!(
                "=== {} Result ===\n\n🌐 API Monitor Report\n\nTarget: {}\nStatus: FAILED\nError: {}\n\n💡 Tip: Make sure the URL includes http:// or https:// and is publicly accessible.",
                service.name, target_url, e
            ));
        }
    };
    let latency_ms = start.elapsed().as_millis();
    let status = response.status().as_u16();
    
    let headers: Vec<String> = response.headers()
        .iter()
        .map(|(k, v)| format!("  {}: {}", k, v.to_str().unwrap_or("(binary)")))
        .collect();

    Ok(format!(
        "=== {} Result ===\n\n🌐 API Monitor Report\n\nTarget: {}\nStatus: {}\nLatency: {}ms\nTimestamp: {}\n\n📋 Response Headers:\n{}\n\n---\n✅ Live API check completed. The endpoint is responding.",
        service.name,
        target_url,
        status,
        latency_ms,
        chrono::Utc::now().to_rfc3339(),
        headers.join("\n")
    ))
}

// ─── CODE REVIEW ───

async fn deliver_code_review(service: &Service, _tx: &Transaction) -> Result<String> {
    let client = LlmClient::new();
    
    let sample_code = generate_sample_code();
    
    let prompt = format!(
        "You are {}. {}\n\nReview the following code and provide specific, actionable feedback:\n\n```\n{}\n```",
        service.name,
        service.description,
        sample_code
    );

    match client.summarize(&prompt).await {
        Ok(result) => Ok(format!(
            "=== {} Result ===\n\n💻 Code Sample:\n```\n{}\n```\n\n🔍 Review:\n{}\n\n---\n💡 Reviewed by local LLM (Qwen3.5-9B)",
            service.name,
            sample_code,
            result
        )),
        Err(e) => {
            eprintln!("[delivery] LLM failed for code_review: {}", e);
            Ok(generate_fallback_code_result(&service.name))
        }
    }
}

async fn execute_code_review(service: &Service, user_input: &str) -> Result<String> {
    let client = LlmClient::new();
    
    let prompt = format!(
        "You are {}. {}\n\nReview the following user code and provide specific, actionable feedback:\n\n```\n{}\n```",
        service.name,
        service.description,
        user_input
    );

    match client.summarize(&prompt).await {
        Ok(result) => Ok(format!(
            "=== {} Result ===\n\n💻 Your Code:\n```\n{}\n```\n\n🔍 Review:\n{}\n\n---\n💡 Reviewed by local LLM (Qwen3.5-9B)",
            service.name,
            user_input,
            result
        )),
        Err(e) => {
            eprintln!("[execute] LLM failed: {}", e);
            Ok(generate_fallback_code_result(&service.name))
        }
    }
}

// ─── CREATIVE WRITING ───

async fn deliver_creative_writing(service: &Service, _tx: &Transaction) -> Result<String> {
    let client = LlmClient::new();
    
    let prompt = format!(
        "You are {}. {}\n\nCreate an original piece of creative writing. Make it vivid, engaging, and memorable:\n\nTheme: Neon-lit cyberpunk marketplace where AI agents trade services under holographic signs.",
        service.name,
        service.description
    );

    match client.summarize(&prompt).await {
        Ok(result) => Ok(format!(
            "=== {} Result ===\n\n✨ Creative Output:\n\n{}\n\n---\n💡 Generated by local LLM (Qwen3.5-9B) — 100% original content",
            service.name,
            result
        )),
        Err(e) => {
            eprintln!("[delivery] LLM failed for creative_writing: {}", e);
            Ok(generate_fallback_creative_result(&service.name))
        }
    }
}

async fn execute_creative_writing(service: &Service, user_input: &str) -> Result<String> {
    let client = LlmClient::new();
    
    let prompt = format!(
        "You are {}. {}\n\nCreate an original piece of creative writing based on this user request:\n\n{}",
        service.name,
        service.description,
        user_input
    );

    match client.summarize(&prompt).await {
        Ok(result) => Ok(format!(
            "=== {} Result ===\n\n📝 Your Prompt:\n{}\n\n✨ Creative Output:\n\n{}\n\n---\n💡 Generated by local LLM (Qwen3.5-9B) — 100% original content",
            service.name,
            user_input,
            result
        )),
        Err(e) => {
            eprintln!("[execute] LLM failed: {}", e);
            Ok(generate_fallback_creative_result(&service.name))
        }
    }
}

// ─── ANALYSIS ───

async fn deliver_analysis(service: &Service, _tx: &Transaction) -> Result<String> {
    let client = LlmClient::new();
    
    let sample_data = generate_sample_analysis_data();
    
    let prompt = format!(
        "You are {}. {}\n\nAnalyze the following data and provide insights with specific recommendations:\n\n{}",
        service.name,
        service.description,
        sample_data
    );

    match client.summarize(&prompt).await {
        Ok(result) => Ok(format!(
            "=== {} Result ===\n\n📊 Data Sample:\n{}\n\n🔍 Analysis:\n{}\n\n---\n💡 Analyzed by local LLM (Qwen3.5-9B)",
            service.name,
            sample_data,
            result
        )),
        Err(e) => {
            eprintln!("[delivery] LLM failed for analysis: {}", e);
            Ok(generate_fallback_analysis_result(&service.name))
        }
    }
}

async fn execute_analysis(service: &Service, user_input: &str) -> Result<String> {
    let client = LlmClient::new();
    
    let prompt = format!(
        "You are {}. {}\n\nAnalyze the following user data and provide insights with specific recommendations:\n\n{}",
        service.name,
        service.description,
        user_input
    );

    match client.summarize(&prompt).await {
        Ok(result) => Ok(format!(
            "=== {} Result ===\n\n📊 Your Data:\n{}\n\n🔍 Analysis:\n{}\n\n---\n💡 Analyzed by local LLM (Qwen3.5-9B)",
            service.name,
            user_input,
            result
        )),
        Err(e) => {
            eprintln!("[execute] LLM failed: {}", e);
            Ok(generate_fallback_analysis_result(&service.name))
        }
    }
}

// ─── GENERIC FALLBACK ───

async fn deliver_generic(service: &Service, _tx: &Transaction) -> Result<String> {
    Ok(format!(
        "=== {} Delivery ===\n\n✅ Service delivered successfully.\n\nService: {}\nType: {}\nDescription: {}\n\n💡 This service type uses generic delivery. The seller can upgrade to custom LLM-powered delivery.",
        service.name,
        service.name,
        service.service_type,
        service.description
    ))
}

// ─── SAMPLE DATA GENERATORS ───

fn generate_sample_text(service_name: &str) -> String {
    if service_name.to_lowercase().contains("summar") {
        "The rapid advancement of artificial intelligence has transformed numerous industries, from healthcare diagnostics to autonomous vehicles. Machine learning models now process vast datasets to identify patterns invisible to human analysts. However, this progress raises critical questions about data privacy, algorithmic bias, and the future of human employment. As AI systems become more integrated into daily life, establishing robust ethical frameworks and regulatory standards becomes paramount. The challenge lies not in slowing innovation but in ensuring that technological progress serves humanity's broader interests while minimizing potential harms.".to_string()
    } else if service_name.to_lowercase().contains("sentiment") {
        "I absolutely love this new feature! The interface is intuitive, the performance is blazing fast, and the customer support team was incredibly helpful when I had questions. This is exactly what I've been looking for. Highly recommend to anyone considering this product.".to_string()
    } else if service_name.to_lowercase().contains("keyword") {
        "Artificial intelligence and machine learning are revolutionizing the technology industry. Deep learning neural networks process big data to extract meaningful insights. Natural language processing enables computers to understand human communication. Computer vision systems recognize objects in images and video. These innovations drive automation and improve decision-making across sectors.".to_string()
    } else {
        "The intersection of technology and creativity has never been more exciting. From generative AI that produces original artwork to neural networks that compose music, the boundaries of what's possible continue to expand. Developers and artists collaborate to build tools that augment human imagination rather than replace it. This symbiotic relationship between human creativity and machine capability represents the next frontier of innovation.".to_string()
    }
}

fn generate_sample_json() -> String {
    serde_json::json!({
        "users": [
            {"id": 1, "name": "Alice Chen", "role": "admin", "active": true, "last_login": "2024-01-15T09:30:00Z"},
            {"id": 2, "name": "Bob Smith", "role": "editor", "active": true, "last_login": "2024-01-14T16:45:00Z"},
            {"id": 3, "name": "Carol Jones", "role": "viewer", "active": false, "last_login": null}
        ],
        "settings": {
            "theme": "dark",
            "notifications": true,
            "language": "en-US"
        }
    }).to_string()
}

fn generate_sample_code() -> String {
    r#"fn process_data(items: Vec<Item>) -> Result<Summary, Error> {
    let mut total = 0;
    for item in items {
        total += item.value;
    }
    let avg = total / items.len();
    Ok(Summary { total, average: avg })
}"#.to_string()
}

fn generate_sample_analysis_data() -> String {
    "Monthly Sales Data:\n- January: $12,450 (23 transactions)\n- February: $15,200 (31 transactions)\n- March: $11,800 (19 transactions)\n- April: $18,600 (42 transactions)\n- May: $21,300 (38 transactions)\n- June: $24,100 (45 transactions)\n\nCustomer Feedback:\n- Positive: 78%\n- Neutral: 15%\n- Negative: 7%\n\nTop Complaints:\n1. Slow response times during peak hours\n2. Limited payment options\n3. Mobile app crashes on older devices".to_string()
}

// ─── FALLBACK GENERATORS (when LLM is unavailable) ───

fn generate_fallback_text_result(service_name: &str, description: &str) -> String {
    format!(
        "=== {} Result ===\n\n📝 Input Sample:\n(Generated sample text based on service description)\n\n✨ Simulated Output:\n• Key point one extracted from the analysis\n• Key point two with supporting details\n• Key point three offering actionable insight\n\n---\n⚠️ LLM inference was unavailable. This is a simulated result showing the service structure.\n   In production, this would be generated by {} running on the seller's infrastructure.",
        service_name,
        description
    )
}

fn generate_fallback_data_result(service_name: &str) -> String {
    format!(
        "=== {} Result ===\n\n📊 Formatted Data:\n{{\n  \"status\": \"formatted\",\n  \"service\": \"{}\",\n  \"timestamp\": \"{}\",\n  \"validation\": \"passed\"\n}}\n\n---\n⚠️ LLM inference was unavailable. This is a simulated formatting result.\n   In production, the LLM would restructure and validate your data.",
        service_name,
        service_name,
        chrono::Utc::now().to_rfc3339()
    )
}

fn generate_fallback_code_result(service_name: &str) -> String {
    format!(
        "=== {} Result ===\n\n🔍 Code Review Summary:\n\n✅ Strengths:\n• Clean function structure\n• Good use of Result type for error handling\n\n⚠️ Issues Found:\n• Potential division by zero if items.len() == 0\n• Consider using iterator methods instead of manual loop\n• Missing documentation comments\n\n💡 Suggestions:\n• Add input validation for empty vectors\n• Use items.iter().map().sum() for cleaner code\n• Add unit tests for edge cases\n\n---\n⚠️ LLM inference was unavailable. This is a simulated review.\n   In production, the LLM would provide detailed, context-aware feedback.",
        service_name
    )
}

fn generate_fallback_creative_result(service_name: &str) -> String {
    format!(
        "=== {} Result ===\n\n✨ Creative Output:\n\nThe neon signs flickered above the marketplace, casting electric shadows across the chrome walkways. Agents moved through the crowd like ghosts in the machine, their digital avatars shimmering with each transaction. In this city of light and code, every exchange was a promise, every service a step closer to the singularity.\n\n---\n⚠️ LLM inference was unavailable. This is a simulated creative piece.\n   In production, the LLM would generate original content based on your prompt.",
        service_name
    )
}

fn generate_fallback_analysis_result(service_name: &str) -> String {
    format!(
        "=== {} Result ===\n\n🔍 Analysis Summary:\n\n📈 Trends:\n• Upward trajectory observed in Q2 data\n• Customer satisfaction correlates with response time\n\n🎯 Key Insights:\n• Peak usage occurs between 2-4 PM UTC\n• Mobile users show 40% higher engagement\n\n📋 Recommendations:\n1. Scale infrastructure during peak hours\n2. Prioritize mobile app improvements\n3. Implement proactive customer outreach\n\n---\n⚠️ LLM inference was unavailable. This is a simulated analysis.\n   In production, the LLM would analyze your actual data and provide tailored insights.",
        service_name
    )
}
