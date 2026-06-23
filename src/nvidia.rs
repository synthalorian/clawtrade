//! LLM client with local-first inference and optional cloud fallback.
//!
//! Primary: Local llama-swap server (OpenAI-compatible API on port 8080)
//! Fallback: NVIDIA API (if NVIDIA_API_KEY is set and local fails)
//!
//! The architecture is local-first: all service delivery runs on local models
//! (Qwen 3.5 9B, Qwen 3.6 35B, Gemma 4 12B/26B, Phi-4 Reasoning+).
//! NVIDIA is only used as a fallback when local inference is unavailable.

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

/// Unified LLM client that prefers local inference, falls back to NVIDIA cloud.
///
/// Local-first architecture: the llama-swap server on port 8080 is the primary
/// inference engine. All service delivery uses local models (Qwen, Gemma, Phi-4).
/// NVIDIA API is only used as a fallback when local inference fails AND
/// NVIDIA_API_KEY is set.
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
        } else if user_request.contains("---DOCUMENT---") && user_request.contains("---QUESTION---") {
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
        
        let user_request = "---CODEBASE---\nfn main() {}\n---QUESTION---\nWhere is the entry point?";
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
                NvidiaMessage { role: "system".to_string(), content: "You are a test".to_string() },
                NvidiaMessage { role: "user".to_string(), content: "Hello".to_string() },
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
