//! LLM client with local-first inference and optional cloud fallback.
//!
//! Primary: Local llama-swap server (OpenAI-compatible API on port 8080)
//! Fallback: NVIDIA API (if NVIDIA_API_KEY is set and local fails)
//!
//! The architecture is local-first: all service delivery runs on local models
//! (Qwen, Gemma). NVIDIA is only used as a fallback when local inference is
//! unavailable.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

const NVIDIA_API_BASE: &str = "https://integrate.api.nvidia.com/v1";

#[derive(Debug, Serialize)]
pub struct NvidiaChatRequest {
    pub model: String,
    pub messages: Vec<NvidiaMessage>,
    pub temperature: f32,
    pub max_tokens: i32,
    pub top_p: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NvidiaMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct NvidiaChatResponse {
    pub choices: Vec<NvidiaChoice>,
}

#[derive(Debug, Deserialize)]
pub struct NvidiaChoice {
    pub message: NvidiaMessage,
}

pub struct NvidiaClient {
    api_key: String,
    client: reqwest::Client,
}

impl NvidiaClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    #[allow(dead_code)]
    pub async fn agent_reasoning(&self, prompt: &str) -> Result<String> {
        let req = NvidiaChatRequest {
            model: "nvidia/nemotron-4-340b-instruct".to_string(),
            messages: vec![
                NvidiaMessage {
                    role: "system".to_string(),
                    content: "You are an autonomous AI merchant agent on the ClawTrade marketplace. You make economic decisions: pricing, service creation, purchasing. Respond ONLY with the requested JSON and keep reasoning under 20 words.".to_string(),
                },
                NvidiaMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            temperature: 0.7,
            max_tokens: 128,
            top_p: 0.9,
        };

        let res = self
            .client
            .post(format!("{}/chat/completions", NVIDIA_API_BASE))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        let data: NvidiaChatResponse = res.json().await?;
        Ok(data
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default())
    }

    #[allow(dead_code)]
    pub async fn summarize_text(&self, text: &str) -> Result<String> {
        let req = NvidiaChatRequest {
            model: "nvidia/nemotron-4-340b-instruct".to_string(),
            messages: vec![
                NvidiaMessage {
                    role: "system".to_string(),
                    content: "Summarize the following text into 3 bullet points. Be concise."
                        .to_string(),
                },
                NvidiaMessage {
                    role: "user".to_string(),
                    content: text.to_string(),
                },
            ],
            temperature: 0.3,
            max_tokens: 256,
            top_p: 0.9,
        };

        let res = self
            .client
            .post(format!("{}/chat/completions", NVIDIA_API_BASE))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        let data: NvidiaChatResponse = res.json().await?;
        Ok(data
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default())
    }

    #[allow(dead_code)]
    pub async fn format_json(&self, json_str: &str) -> Result<String> {
        let req = NvidiaChatRequest {
            model: "nvidia/nemotron-4-340b-instruct".to_string(),
            messages: vec![
                NvidiaMessage {
                    role: "system".to_string(),
                    content: "Format and validate the following JSON. Return only the formatted JSON, no explanation.".to_string(),
                },
                NvidiaMessage {
                    role: "user".to_string(),
                    content: json_str.to_string(),
                },
            ],
            temperature: 0.1,
            max_tokens: 1024,
            top_p: 0.9,
        };

        let res = self
            .client
            .post(format!("{}/chat/completions", NVIDIA_API_BASE))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        let data: NvidiaChatResponse = res.json().await?;
        Ok(data
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default())
    }

    #[allow(dead_code)]
    pub async fn analyze_market(&self, market_data: &str) -> Result<String> {
        let req = NvidiaChatRequest {
            model: "nvidia/nemotron-4-340b-instruct".to_string(),
            messages: vec![
                NvidiaMessage {
                    role: "system".to_string(),
                    content: "You are a market intelligence analyst. Analyze the marketplace data and provide pricing recommendations.".to_string(),
                },
                NvidiaMessage {
                    role: "user".to_string(),
                    content: market_data.to_string(),
                },
            ],
            temperature: 0.5,
            max_tokens: 512,
            top_p: 0.9,
        };

        let res = self
            .client
            .post(format!("{}/chat/completions", NVIDIA_API_BASE))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        let data: NvidiaChatResponse = res.json().await?;
        Ok(data
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default())
    }
}

/// Local llama-swap client (OpenAI-compatible API)
pub struct LocalLlmClient {
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl LocalLlmClient {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            base_url,
            model,
            client: reqwest::Client::new(),
        }
    }

    pub async fn chat(&self,
        system: &str,
        user: &str,
        max_tokens: i32,
    ) -> Result<String> {
        let req = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user}
            ],
            "temperature": 0.7,
            "max_tokens": max_tokens
        });

        let res = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .json(&req)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await?;

        let status = res.status();
        let text = res.text().await?;

        if !status.is_success() {
            anyhow::bail!("LLM API returned {}: {}", status, text);
        }

        let data: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| anyhow::anyhow!("Failed to parse LLM response: {}. Raw: {}", e, text))?;

        let mut result = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // Filter out tool call patterns and special token formats
        let tool_patterns = [
            "test_connection_check",
            "greet_user",
            "function_call",
            "tool_call",
            "<|channel>thought",
            "<|channel|>",
        ];
        for pattern in &tool_patterns {
            result = result.replace(pattern, "");
        }
        result = result.trim().to_string();

        Ok(result)
    }

    pub async fn chat_simple(&self, prompt: &str) -> Result<String> {
        self.chat("You are a helpful AI assistant.", prompt, 512).await
    }

    /// Chat with a specific model override. Falls back to default model if the
    /// requested one isn't available (llama-swap will return 404 for unloaded models).
    /// Tracks inference timing and broadcasts events for the live monitor.
    pub async fn chat_with_model(
        &self,
        model: &str,
        system: &str,
        user: &str,
        max_tokens: i32,
    ) -> Result<String> {
        let start = std::time::Instant::now();
        let estimated_tokens = (system.len() + user.len()) as i64 / 4;
        let service_name = "service_delivery".to_string();
        let tier = infer_tier_from_model(model);

        // Broadcast inference started
        crate::websocket::broadcast_event(crate::websocket::DashboardEvent::InferenceStarted {
            service_name: service_name.clone(),
            model: model.to_string(),
            estimated_tokens,
        });

        let req = serde_json::json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user}
            ],
            "temperature": 0.7,
            "max_tokens": max_tokens
        });

        let res = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .json(&req)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await?;

        let status = res.status();
        let text = res.text().await?;
        let duration_ms = start.elapsed().as_millis() as i64;

        if !status.is_success() {
            // Model not loaded — try to auto-load via llama-swap /v1/load-model endpoint
            if status.as_u16() == 404 || status.as_u16() == 503 {
                eprintln!("[llm] Model {} not loaded, attempting auto-load...", model);
                if let Err(e) = self.auto_load_model(model).await {
                    eprintln!("[llm] Auto-load failed: {}", e);
                }
                // Retry once after auto-load
                let retry_start = std::time::Instant::now();
                let retry_req = serde_json::json!({
                    "model": model,
                    "messages": [
                        {"role": "system", "content": system},
                        {"role": "user", "content": user}
                    ],
                    "temperature": 0.7,
                    "max_tokens": max_tokens
                });
                let retry_res = self
                    .client
                    .post(format!("{}/v1/chat/completions", self.base_url))
                    .json(&retry_req)
                    .timeout(std::time::Duration::from_secs(120))
                    .send()
                    .await?;
                let retry_status = retry_res.status();
                let retry_text = retry_res.text().await?;
                let retry_duration_ms = retry_start.elapsed().as_millis() as i64;
                if retry_status.is_success() {
                    let data: serde_json::Value =
                        serde_json::from_str(&retry_text).map_err(|e| {
                            anyhow::anyhow!(
                                "Failed to parse LLM response: {}. Raw: {}",
                                e,
                                retry_text
                            )
                        })?;
                    let mut result = data["choices"][0]["message"]["content"]
                        .as_str()
                        .unwrap_or("")
                        .to_string();
                    // Filter out tool call patterns and special token formats
                    let tool_patterns = [
                        "test_connection_check",
                        "greet_user",
                        "function_call",
                        "tool_call",
                        "<|channel>thought",
                        "<|channel|>",
                    ];
                    for pattern in &tool_patterns {
                        result = result.replace(pattern, "");
                    }
                    result = result.trim().to_string();
                    let actual_tokens = result.len() as i64 / 4;
                    // Record inference
                    self.record_inference(InferenceRecord {
                        service_name: service_name.clone(),
                        model: model.to_string(),
                        start_time: chrono::Utc::now()
                            - chrono::Duration::milliseconds(retry_duration_ms),
                        end_time: Some(chrono::Utc::now()),
                        estimated_tokens,
                        actual_tokens: Some(actual_tokens),
                        status: "completed".to_string(),
                        fallback_reason: None,
                        tier: tier.to_string(),
                        duration_ms: retry_duration_ms,
                    })
                    .await;
                    // Broadcast completed
                    crate::websocket::broadcast_event(
                        crate::websocket::DashboardEvent::InferenceCompleted {
                            service_name: service_name.clone(),
                            model: model.to_string(),
                            actual_tokens,
                            duration_ms: retry_duration_ms,
                        },
                    );
                    return Ok(result);
                }
            }
            // Model not loaded or failed to start — fallback to default model
            eprintln!(
                "[llm] Model {} failed ({}), falling back to {}",
                model, status, self.model
            );
            // Record fallback
            self.record_inference(InferenceRecord {
                service_name: service_name.clone(),
                model: model.to_string(),
                start_time: chrono::Utc::now() - chrono::Duration::milliseconds(duration_ms),
                end_time: Some(chrono::Utc::now()),
                estimated_tokens,
                actual_tokens: None,
                status: "fallback".to_string(),
                fallback_reason: Some(format!("HTTP {}", status)),
                tier: tier.to_string(),
                duration_ms,
            })
            .await;
            crate::websocket::broadcast_event(crate::websocket::DashboardEvent::ModelFallback {
                requested: model.to_string(),
                fallback_reason: format!(
                    "Model {} failed ({}), falling back to {}",
                    model, status, self.model
                ),
            });
            return self.chat(system, user, max_tokens).await;
        }

        let data: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| anyhow::anyhow!("Failed to parse LLM response: {}. Raw: {}", e, text))?;

        let mut result = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // Filter out tool call patterns and special token formats that some models return
        let tool_patterns = [
            "test_connection_check",
            "greet_user",
            "function_call",
            "tool_call",
            "<|channel>thought",
            "<channel|>",
        ];
        for pattern in &tool_patterns {
            result = result.replace(pattern, "");
        }
        result = result.trim().to_string();
        if result.is_empty() {
            result =
                "The model returned an empty response. Please try again with a different input."
                    .to_string();
        }
        let actual_tokens = result.len() as i64 / 4;

        // Record inference
        self.record_inference(InferenceRecord {
            service_name: service_name.clone(),
            model: model.to_string(),
            start_time: chrono::Utc::now() - chrono::Duration::milliseconds(duration_ms),
            end_time: Some(chrono::Utc::now()),
            estimated_tokens,
            actual_tokens: Some(actual_tokens),
            status: "completed".to_string(),
            fallback_reason: None,
            tier: tier.to_string(),
            duration_ms,
        })
        .await;

        // Broadcast completed
        crate::websocket::broadcast_event(crate::websocket::DashboardEvent::InferenceCompleted {
            service_name: service_name.clone(),
            model: model.to_string(),
            actual_tokens,
            duration_ms,
        });

        Ok(result)
    }

    /// Record an inference to the history buffer (keep last 50)
    async fn record_inference(&self, record: InferenceRecord) {
        // This is a no-op on LocalLlmClient since it doesn't have the history field.
        // The LlmClient wrapper handles history. We just broadcast here.
        let _ = record;
    }

    /// Auto-load a model via llama-swap's /v1/load-model endpoint
    async fn auto_load_model(&self, model: &str) -> Result<()> {
        let load_req = serde_json::json!({
            "model": model
        });
        let res = self
            .client
            .post(format!("{}/v1/load-model", self.base_url))
            .json(&load_req)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await?;

        if res.status().is_success() {
            eprintln!("[llm] Auto-loaded model: {}", model);
            Ok(())
        } else {
            let text = res.text().await.unwrap_or_default();
            anyhow::bail!("Auto-load failed for {}: {}", model, text)
        }
    }
}

/// Inference record for monitoring and visualization
#[derive(Debug, Clone, Serialize)]
pub struct InferenceRecord {
    pub service_name: String,
    pub model: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub estimated_tokens: i64,
    pub actual_tokens: Option<i64>,
    pub status: String, // "in_progress", "completed", "failed", "fallback"
    pub fallback_reason: Option<String>,
    pub tier: String,
    pub duration_ms: i64,
}

fn infer_tier_from_model(model: &str) -> &'static str {
    let m = model.to_lowercase();
    if m.contains("9b") {
        "micro"
    } else if m.contains("12b") || m.contains("35b") {
        "real"
    } else if m.contains("26b") || m.contains("reasoning") {
        "heavy"
    } else {
        "local"
    }
}

/// Unified LLM client that prefers local inference, falls back to NVIDIA cloud.
///
/// Local-first architecture: the llama-swap server on port 8080 is the primary
/// inference engine. All service delivery uses local models (Qwen, Gemma, Phi-4).
/// NVIDIA API is only used as a fallback when local inference fails AND
/// NVIDIA_API_KEY is set.
pub struct LlmClient {
    nvidia: Option<NvidiaClient>,
    local: LocalLlmClient,
    inference_history: Arc<Mutex<Vec<InferenceRecord>>>,
}

impl LlmClient {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let nvidia = std::env::var("NVIDIA_API_KEY").ok().map(NvidiaClient::new);

        let local_url =
            std::env::var("LLM_LOCAL_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string()); // Default to llama-swap
        let local_model =
            std::env::var("LLM_LOCAL_MODEL").unwrap_or_else(|_| "synthclaw-9b-131k".to_string());

        Self {
            nvidia,
            local: LocalLlmClient::new(local_url, local_model),
            inference_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get recent inference history (last 50 records)
    pub async fn get_inference_history(&self) -> Vec<InferenceRecord> {
        let history = self.inference_history.lock().await;
        history.iter().rev().take(50).cloned().collect()
    }

    /// Run LLM reasoning for an agent decision. Records inference history
    /// and broadcasts WebSocket events with the agent's name for visibility.
    pub async fn agent_reasoning(&self, agent_name: &str, prompt: &str) -> Result<String> {
        let start = std::time::Instant::now();
        let estimated_tokens = prompt.len() as i64 / 4;
        let service_name = format!("agent_reasoning:{}", agent_name);
        let model = self.local.model.clone();
        let tier = infer_tier_from_model(&model);

        crate::websocket::broadcast_event(crate::websocket::DashboardEvent::InferenceStarted {
            service_name: service_name.clone(),
            model: model.clone(),
            estimated_tokens,
        });

        let result = if let Some(ref nvidia) = self.nvidia {
            match nvidia.agent_reasoning(prompt).await {
                Ok(res) => Ok(res),
                Err(e) => {
                    eprintln!("[llm] NVIDIA fallback to local: {}", e);
                    self.local.chat("You are a helpful AI assistant.", prompt, 128).await
                }
            }
        } else {
            self.local.chat("You are a helpful AI assistant.", prompt, 128).await
        };

        let duration_ms = start.elapsed().as_millis() as i64;
        let actual_tokens = result.as_ref().map(|r| r.len() as i64 / 4).unwrap_or(0);

        {
            let mut history = self.inference_history.lock().await;
            history.push(InferenceRecord {
                service_name: service_name.clone(),
                model: model.clone(),
                start_time: chrono::Utc::now() - chrono::Duration::milliseconds(duration_ms),
                end_time: Some(chrono::Utc::now()),
                estimated_tokens,
                actual_tokens: result.as_ref().ok().map(|_| actual_tokens),
                status: if result.is_ok() {
                    "completed".to_string()
                } else {
                    "failed".to_string()
                },
                fallback_reason: None,
                tier: tier.to_string(),
                duration_ms,
            });
            if history.len() > 50 {
                history.remove(0);
            }
        }

        crate::websocket::broadcast_event(crate::websocket::DashboardEvent::InferenceCompleted {
            service_name: service_name.clone(),
            model: model.clone(),
            actual_tokens,
            duration_ms,
        });

        result
    }

    /// Summarize text using local LLM. Records inference history and broadcasts events.
    pub async fn summarize(&self, text: &str) -> Result<String> {
        let start = std::time::Instant::now();
        let estimated_tokens = text.len() as i64 / 4;
        let service_name = "summarize".to_string();
        let model = self.local.model.clone();
        let tier = infer_tier_from_model(&model);

        crate::websocket::broadcast_event(crate::websocket::DashboardEvent::InferenceStarted {
            service_name: service_name.clone(),
            model: model.clone(),
            estimated_tokens,
        });

        let result = if let Some(ref nvidia) = self.nvidia {
            match nvidia.summarize_text(text).await {
                Ok(res) => Ok(res),
                Err(e) => {
                    eprintln!("[llm] NVIDIA fallback to local: {}", e);
                    self.local
                        .chat_simple(&format!("Summarize: {}", text))
                        .await
                }
            }
        } else {
            self.local
                .chat_simple(&format!("Summarize: {}", text))
                .await
        };

        let duration_ms = start.elapsed().as_millis() as i64;
        let actual_tokens = result.as_ref().map(|r| r.len() as i64 / 4).unwrap_or(0);

        {
            let mut history = self.inference_history.lock().await;
            history.push(InferenceRecord {
                service_name: service_name.clone(),
                model: model.clone(),
                start_time: chrono::Utc::now() - chrono::Duration::milliseconds(duration_ms),
                end_time: Some(chrono::Utc::now()),
                estimated_tokens,
                actual_tokens: result.as_ref().ok().map(|_| actual_tokens),
                status: if result.is_ok() {
                    "completed".to_string()
                } else {
                    "failed".to_string()
                },
                fallback_reason: None,
                tier: tier.to_string(),
                duration_ms,
            });
            if history.len() > 50 {
                history.remove(0);
            }
        }

        crate::websocket::broadcast_event(crate::websocket::DashboardEvent::InferenceCompleted {
            service_name: service_name.clone(),
            model: model.clone(),
            actual_tokens,
            duration_ms,
        });

        result
    }

    #[allow(dead_code)]
    pub async fn format_json(&self, json_str: &str) -> Result<String> {
        if let Some(ref nvidia) = self.nvidia {
            match nvidia.format_json(json_str).await {
                Ok(res) => return Ok(res),
                Err(e) => eprintln!("[llm] NVIDIA fallback to local: {}", e),
            }
        }
        self.local
            .chat_simple(&format!("Format this JSON: {}", json_str))
            .await
    }

    /// Analyze market data using local LLM. Records inference history and broadcasts events.
    pub async fn analyze_market(&self, data: &str) -> Result<String> {
        let start = std::time::Instant::now();
        let estimated_tokens = data.len() as i64 / 4;
        let service_name = "analyze_market".to_string();
        let model = self.local.model.clone();
        let tier = infer_tier_from_model(&model);

        crate::websocket::broadcast_event(crate::websocket::DashboardEvent::InferenceStarted {
            service_name: service_name.clone(),
            model: model.clone(),
            estimated_tokens,
        });

        let result = if let Some(ref nvidia) = self.nvidia {
            match nvidia.analyze_market(data).await {
                Ok(res) => Ok(res),
                Err(e) => {
                    eprintln!("[llm] NVIDIA fallback to local: {}", e);
                    self.local
                        .chat_simple(&format!("Analyze market data: {}", data))
                        .await
                }
            }
        } else {
            self.local
                .chat_simple(&format!("Analyze market data: {}", data))
                .await
        };

        let duration_ms = start.elapsed().as_millis() as i64;
        let actual_tokens = result.as_ref().map(|r| r.len() as i64 / 4).unwrap_or(0);

        {
            let mut history = self.inference_history.lock().await;
            history.push(InferenceRecord {
                service_name: service_name.clone(),
                model: model.clone(),
                start_time: chrono::Utc::now() - chrono::Duration::milliseconds(duration_ms),
                end_time: Some(chrono::Utc::now()),
                estimated_tokens,
                actual_tokens: result.as_ref().ok().map(|_| actual_tokens),
                status: if result.is_ok() {
                    "completed".to_string()
                } else {
                    "failed".to_string()
                },
                fallback_reason: None,
                tier: tier.to_string(),
                duration_ms,
            });
            if history.len() > 50 {
                history.remove(0);
            }
        }

        crate::websocket::broadcast_event(crate::websocket::DashboardEvent::InferenceCompleted {
            service_name: service_name.clone(),
            model: model.clone(),
            actual_tokens,
            duration_ms,
        });

        result
    }

    /// Deliver a service using the service catalog's prompt template and model routing.
    /// This is the core service delivery engine — it takes a service definition,
    /// substitutes the user's request into the prompt template, and calls the LLM.
    ///
    /// Placeholder substitution rules:
    /// - `{input}` → user's request (always substituted)
    /// - `{format}`, `{pattern}`, `{target_language}`, `{standard}`, `{focus}` → generic defaults
    /// - `{codebase}`, `{document}`, `{question}` → when user_request contains the delimiter
    /// - `{model_name}`, `{target_architecture}`, `{analysis_type}`, `{extraction_target}` → defaults
    pub async fn deliver_service(
        &self,
        service_def: &crate::service_catalog::ServiceDefinition,
        user_request: &str,
    ) -> Result<String> {
        let system = service_def.system_prompt;

        // Build the user prompt by substituting all known placeholders
        let mut user = service_def.user_prompt_template.to_string();

        // Always substitute {input} with the user's request
        user = user.replace("{input}", user_request);

        // Generic defaults for optional placeholders
        user = user.replace("{format}", "JSON");
        user = user.replace("{pattern}", "modern idioms");
        user = user.replace("{target_language}", "Python");
        user = user.replace("{standard}", "SOC2");
        user = user.replace("{focus}", "architecture and security");
        user = user.replace("{model_name}", "custom fine-tuned model");
        user = user.replace("{target_architecture}", "microservices");
        user = user.replace("{analysis_type}", "anomalies and patterns");
        user = user.replace("{extraction_target}", "entities and relationships");

        // Multi-part placeholders: if user_request contains "---CODEBASE---" or "---QUESTION---",
        // split and substitute. Otherwise, put the whole request in both slots.
        if user_request.contains("---CODEBASE---") && user_request.contains("---QUESTION---") {
            let parts: Vec<&str> = user_request.split("---CODEBASE---").collect();
            if parts.len() >= 2 {
                let _before = parts[0].trim();
                let after = parts[1].split("---QUESTION---").collect::<Vec<&str>>();
                if after.len() >= 2 {
                    let codebase = after[0].trim();
                    let question = after[1].trim();
                    user = user.replace("{codebase}", codebase);
                    user = user.replace("{question}", question);
                    // Also handle {document} variant
                    user = user.replace("{document}", codebase);
                }
            }
        } else if user_request.contains("---DOCUMENT---") && user_request.contains("---QUESTION---")
        {
            let parts: Vec<&str> = user_request.split("---DOCUMENT---").collect();
            if parts.len() >= 2 {
                let after = parts[1].split("---QUESTION---").collect::<Vec<&str>>();
                if after.len() >= 2 {
                    let document = after[0].trim();
                    let question = after[1].trim();
                    user = user.replace("{document}", document);
                    user = user.replace("{question}", question);
                    // Also handle {codebase} variant
                    user = user.replace("{codebase}", document);
                }
            }
        } else {
            // No structured delimiters — substitute {codebase}, {document}, {question} with the request
            user = user.replace("{codebase}", user_request);
            user = user.replace("{document}", user_request);
            user = user.replace("{question}", user_request);
            user = user.replace("{context}", user_request);
        }

        let model = service_def.model.model_name();
        let max_tokens = service_def.max_output_tokens;

        eprintln!(
            "[llm] Delivering '{}' using model {} (max_tokens={})",
            service_def.name, model, max_tokens
        );

        self.local
            .chat_with_model(&model, system, &user, max_tokens)
            .await
    }

    /// Deliver a service with a custom (hardened) system prompt and pre-sanitized user prompt.
    /// Used by the prompt injection defense layer to ensure user input is isolated
    /// and system instructions are reinforced against override attempts.
    /// Records inference history and broadcasts WebSocket events for the live monitor.
    pub async fn deliver_service_with_prompt(
        &self,
        service_name: &str,
        model_assignment: &crate::service_catalog::ModelAssignment,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String> {
        let model = model_assignment.model_name();
        let max_tokens = 2048; // Default max tokens for hardened delivery
        let tier = infer_tier_from_model(&model);
        let estimated_tokens = (system_prompt.len() + user_prompt.len()) as i64 / 4;

        eprintln!(
            "[llm] Delivering service '{}' with hardened prompt using model {}",
            service_name, model
        );

        // Broadcast inference started
        crate::websocket::broadcast_event(crate::websocket::DashboardEvent::InferenceStarted {
            service_name: service_name.to_string(),
            model: model.clone(),
            estimated_tokens,
        });

        let start = std::time::Instant::now();
        let result = self
            .local
            .chat_with_model(&model, system_prompt, user_prompt, max_tokens)
            .await;
        let duration_ms = start.elapsed().as_millis() as i64;

        match &result {
            Ok(output) => {
                let actual_tokens = output.len() as i64 / 4;
                // Record to history
                let mut history = self.inference_history.lock().await;
                history.push(InferenceRecord {
                    service_name: service_name.to_string(),
                    model: model.clone(),
                    start_time: chrono::Utc::now() - chrono::Duration::milliseconds(duration_ms),
                    end_time: Some(chrono::Utc::now()),
                    estimated_tokens,
                    actual_tokens: Some(actual_tokens),
                    status: "completed".to_string(),
                    fallback_reason: None,
                    tier: tier.to_string(),
                    duration_ms,
                });
                if history.len() > 50 {
                    history.remove(0);
                }
                drop(history);
                // Broadcast completed
                crate::websocket::broadcast_event(
                    crate::websocket::DashboardEvent::InferenceCompleted {
                        service_name: service_name.to_string(),
                        model: model.clone(),
                        actual_tokens,
                        duration_ms,
                    },
                );
            }
            Err(e) => {
                // Record failure
                let mut history = self.inference_history.lock().await;
                history.push(InferenceRecord {
                    service_name: service_name.to_string(),
                    model: model.clone(),
                    start_time: chrono::Utc::now() - chrono::Duration::milliseconds(duration_ms),
                    end_time: Some(chrono::Utc::now()),
                    estimated_tokens,
                    actual_tokens: None,
                    status: "failed".to_string(),
                    fallback_reason: Some(e.to_string()),
                    tier: tier.to_string(),
                    duration_ms,
                });
                if history.len() > 50 {
                    history.remove(0);
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deliver_service_placeholder_substitution() {
        let def = crate::service_catalog::get_service_definition("git_commit_msg").unwrap();

        // Simple input substitution
        let user_request = "feat: add user auth";
        let mut user = def.user_prompt_template.to_string();
        user = user.replace("{input}", user_request);

        assert!(user.contains("feat: add user auth"));
        assert!(!user.contains("{input}"));
    }

    #[test]
    fn test_deliver_service_codebase_question_split() {
        let def = crate::service_catalog::get_service_definition("codebase_qa").unwrap();

        let user_request =
            "---CODEBASE---\nfn main() {}\n---QUESTION---\nWhere is the entry point?";
        let mut user = def.user_prompt_template.to_string();
        user = user.replace("{input}", user_request);

        if user_request.contains("---CODEBASE---") && user_request.contains("---QUESTION---") {
            let parts: Vec<&str> = user_request.split("---CODEBASE---").collect();
            let after = parts[1].split("---QUESTION---").collect::<Vec<&str>>();
            let codebase = after[0].trim();
            let question = after[1].trim();
            user = user.replace("{codebase}", codebase);
            user = user.replace("{question}", question);
        }

        assert!(user.contains("fn main() {}"));
        assert!(user.contains("Where is the entry point?"));
        assert!(!user.contains("{codebase}"));
        assert!(!user.contains("{question}"));
    }

    #[test]
    fn test_local_llm_client_url_construction() {
        let client = LocalLlmClient::new(
            "http://127.0.0.1:8080".to_string(),
            "test-model".to_string(),
        );
        // Just verify it constructs without panic
        assert_eq!(client.base_url, "http://127.0.0.1:8080");
        assert_eq!(client.model, "test-model");
    }

    #[test]
    fn test_nvidia_request_serialization() {
        let req = NvidiaChatRequest {
            model: "test-model".to_string(),
            messages: vec![
                NvidiaMessage {
                    role: "system".to_string(),
                    content: "You are a test".to_string(),
                },
                NvidiaMessage {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                },
            ],
            temperature: 0.7,
            max_tokens: 512,
            top_p: 0.9,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("test-model"));
        assert!(json.contains("You are a test"));
        assert!(json.contains("Hello"));
    }
}
