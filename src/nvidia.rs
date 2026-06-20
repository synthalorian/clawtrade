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

/// Fallback to local llama-swap when NVIDIA API is unavailable
pub struct LocalLlmClient {
    base_url: String,
    client: reqwest::Client,
}

impl LocalLlmClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn chat(&self, prompt: &str) -> Result<String> {
        let req = serde_json::json!({
            "model": "local",
            "messages": [
                {"role": "user", "content": prompt}
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

        let data: serde_json::Value = res.json().await?;
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

        Self {
            nvidia,
            local: LocalLlmClient::new(local_url),
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
        self.local.chat(prompt).await
    }

    pub async fn summarize(&self, text: &str) -> Result<String> {
        if let Some(ref nvidia) = self.nvidia {
            match nvidia.summarize_text(text).await {
                Ok(res) => return Ok(res),
                Err(e) => eprintln!("[llm] NVIDIA fallback to local: {}", e),
            }
        }
        self.local.chat(&format!("Summarize: {}", text)).await
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
            .chat(&format!("Format this JSON: {}", json_str))
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
            .chat(&format!("Analyze market data: {}", data))
            .await
    }
}
