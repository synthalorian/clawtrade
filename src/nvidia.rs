//! NVIDIA API integration for agent reasoning and service delivery.
//!
//! Uses NVIDIA API Catalog for Nemotron 3 Ultra inference.
//! Falls back to local llama-swap for development.

use anyhow::Result;
use serde::{Deserialize, Serialize};

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
                    content: "You are an autonomous AI merchant agent on the ClawTrade marketplace. You make economic decisions: pricing, service creation, purchasing. Respond with concise, actionable decisions.".to_string(),
                },
                NvidiaMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            temperature: 0.7,
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

    #[allow(dead_code)]
    pub async fn summarize_text(&self, text: &str) -> Result<String> {
        let req = NvidiaChatRequest {
            model: "nvidia/nemotron-4-340b-instruct".to_string(),
            messages: vec![
                NvidiaMessage {
                    role: "system".to_string(),
                    content: "Summarize the following text into 3 bullet points. Be concise.".to_string(),
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

    pub async fn chat(&self, system: &str, user: &str) -> Result<String> {
        let req = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user}
            ],
            "temperature": 0.7,
            "max_tokens": 512
        });

        let res = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .json(&req)
            .send()
            .await?;

        let status = res.status();
        let text = res.text().await?;
        
        if !status.is_success() {
            anyhow::bail!("LLM API returned {}: {}", status, text);
        }

        let data: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| anyhow::anyhow!("Failed to parse LLM response: {}. Raw: {}", e, text))?;
        
        Ok(data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }

    pub async fn chat_simple(&self, prompt: &str) -> Result<String> {
        self.chat("You are a helpful AI assistant.", prompt).await
    }

    /// Chat with a specific model override. Falls back to default model if the
    /// requested one isn't available (llama-swap will return 404 for unloaded models).
    pub async fn chat_with_model(
        &self,
        model: &str,
        system: &str,
        user: &str,
        max_tokens: i32,
    ) -> Result<String> {
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
            .send()
            .await?;

        let status = res.status();
        let text = res.text().await?;

        if !status.is_success() {
            // Model not loaded or failed to start — fallback to default model
            eprintln!("[llm] Model {} failed ({}), falling back to {}", model, status, self.model);
            return self.chat(system, user).await;
        }

        let data: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| anyhow::anyhow!("Failed to parse LLM response: {}. Raw: {}", e, text))?;

        Ok(data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }
}

/// Unified LLM client that prefers NVIDIA, falls back to local
pub struct LlmClient {
    nvidia: Option<NvidiaClient>,
    local: LocalLlmClient,
}

impl LlmClient {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let nvidia = std::env::var("NVIDIA_API_KEY")
            .ok()
            .map(NvidiaClient::new);

        let local_url = std::env::var("LLM_LOCAL_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
        let local_model = std::env::var("LLM_LOCAL_MODEL")
            .unwrap_or_else(|_| "synthclaw-9b-128k".to_string());

        Self {
            nvidia,
            local: LocalLlmClient::new(local_url, local_model),
        }
    }

    #[allow(dead_code)]
    pub async fn agent_reasoning(&self, prompt: &str) -> Result<String> {
        if let Some(ref nvidia) = self.nvidia {
            match nvidia.agent_reasoning(prompt).await {
                Ok(res) => return Ok(res),
                Err(e) => eprintln!("[llm] NVIDIA fallback to local: {}", e),
            }
        }
        self.local.chat_simple(prompt).await
    }

    pub async fn summarize(&self, text: &str) -> Result<String> {
        if let Some(ref nvidia) = self.nvidia {
            match nvidia.summarize_text(text).await {
                Ok(res) => return Ok(res),
                Err(e) => eprintln!("[llm] NVIDIA fallback to local: {}", e),
            }
        }
        self.local.chat_simple(&format!("Summarize: {}", text)).await
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

    pub async fn analyze_market(&self, data: &str) -> Result<String> {
        if let Some(ref nvidia) = self.nvidia {
            match nvidia.analyze_market(data).await {
                Ok(res) => return Ok(res),
                Err(e) => eprintln!("[llm] NVIDIA fallback to local: {}", e),
            }
        }
        self.local
            .chat_simple(&format!("Analyze market data: {}", data))
            .await
    }

    /// Deliver a service using the service catalog's prompt template and model routing.
    /// This is the core service delivery engine — it takes a service definition,
    /// substitutes the user's request into the prompt template, and calls the LLM.
    pub async fn deliver_service(
        &self,
        service_def: &crate::service_catalog::ServiceDefinition,
        user_request: &str,
    ) -> Result<String> {
        let system = service_def.system_prompt;
        let user = service_def.user_prompt_template.replace("{{input}}", user_request);
        let model = service_def.model.model_name();
        let max_tokens = service_def.max_output_tokens;

        eprintln!(
            "[llm] Delivering {} using model {} (max_tokens={})",
            service_def.name, model, max_tokens
        );

        self.local
            .chat_with_model(model, system, &user, max_tokens)
            .await
    }
}
