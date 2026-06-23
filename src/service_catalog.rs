//! Service Catalog — 40 distinct AI services with model routing and delivery logic
//!
//! Tier 1: Micro-Tasks ($0.09-$0.49) — Qwen 9B 128k
//! Tier 2: Real Work ($0.50-$2.99) — Gemma 12B 128k / Qwen 35B 128k
//! Tier 3: Heavy Lifting ($3.00-$9.99) — Gemma 26B 256k/512k / Phi-4 Reasoning+ 256k
//! Tier 4: Local-Only Superpowers ($9.99-$49.99) — Massive context, uncensored, bulk, custom models

use serde::{Deserialize, Serialize};

/// Service tier determines pricing and model routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceTier {
    MicroTask,    // $0.09-$0.49
    RealWork,     // $0.50-$2.99
    HeavyLifting, // $3.00-$9.99
    LocalOnly,    // $9.99-$49.99 — requires local model advantages
}

impl ServiceTier {
    pub fn base_price_cents(&self) -> i64 {
        match self {
            ServiceTier::MicroTask => 19,
            ServiceTier::RealWork => 149,
            ServiceTier::HeavyLifting => 499,
            ServiceTier::LocalOnly => 1499,
        }
    }

    pub fn price_range_cents(&self) -> (i64, i64) {
        match self {
            ServiceTier::MicroTask => (9, 49),
            ServiceTier::RealWork => (50, 299),
            ServiceTier::HeavyLifting => (300, 999),
            ServiceTier::LocalOnly => (999, 4999),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ServiceTier::MicroTask => "MICRO",
            ServiceTier::RealWork => "REAL",
            ServiceTier::HeavyLifting => "HEAVY",
            ServiceTier::LocalOnly => "LOCAL",
        }
    }

    pub fn badge_color(&self) -> &'static str {
        match self {
            ServiceTier::MicroTask => "cyan",
            ServiceTier::RealWork => "yellow",
            ServiceTier::HeavyLifting => "magenta",
            ServiceTier::LocalOnly => "green",
        }
    }
}

/// Model assignment for service delivery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelAssignment {
    Qwen9B,        // Fast, cheap — micro tasks
    Gemma12B,      // Balanced — medium tasks
    Qwen35B,       // Complex reasoning
    Phi4Reasoning, // Deep reasoning, math
    Gemma26B,      // Maximum capability, 256k context
    Gemma26B512K,  // Maximum capability, 512k context
}

impl ModelAssignment {
    pub fn model_name(&self) -> String {
        // Allow users to override model names via env vars for their own llama-swap setup
        let env_key = match self {
            ModelAssignment::Qwen9B => "CLAWTRADE_MODEL_QWEN9B",
            ModelAssignment::Gemma12B => "CLAWTRADE_MODEL_GEMMA12B",
            ModelAssignment::Qwen35B => "CLAWTRADE_MODEL_QWEN35B",
            ModelAssignment::Phi4Reasoning => "CLAWTRADE_MODEL_PHI4",
            ModelAssignment::Gemma26B => "CLAWTRADE_MODEL_GEMMA26B",
            ModelAssignment::Gemma26B512K => "CLAWTRADE_MODEL_GEMMA26B_512K",
        };
        std::env::var(env_key).unwrap_or_else(|_| self.default_model_name().to_string())
    }

    fn default_model_name(&self) -> String {
        match self {
            ModelAssignment::Qwen9B => std::env::var("CLAWTRADE_MODEL_QWEN9B")
                .unwrap_or_else(|_| "synthclaw-9b-128k".to_string()),
            ModelAssignment::Gemma12B => std::env::var("CLAWTRADE_MODEL_GEMMA12B")
                .unwrap_or_else(|_| "synthclaw-gemma-12b-128k".to_string()),
            ModelAssignment::Qwen35B => std::env::var("CLAWTRADE_MODEL_QWEN35B")
                .unwrap_or_else(|_| "synthclaw-35b-128k".to_string()),
            ModelAssignment::Phi4Reasoning => std::env::var("CLAWTRADE_MODEL_PHI4")
                .unwrap_or_else(|_| "synthclaw-phi-4-reasoning-plus-256k".to_string()),
            ModelAssignment::Gemma26B => std::env::var("CLAWTRADE_MODEL_GEMMA26B")
                .unwrap_or_else(|_| "synthclaw-gemma-26b-256k".to_string()),
            ModelAssignment::Gemma26B512K => std::env::var("CLAWTRADE_MODEL_GEMMA26B_512K")
                .unwrap_or_else(|_| "synthclaw-gemma-26b-512k".to_string()),
        }
    }

    pub fn context_size(&self) -> &'static str {
        match self {
            ModelAssignment::Qwen9B => "128k",
            ModelAssignment::Gemma12B => "128k",
            ModelAssignment::Qwen35B => "128k",
            ModelAssignment::Phi4Reasoning => "256k",
            ModelAssignment::Gemma26B => "256k",
            ModelAssignment::Gemma26B512K => "512k",
        }
    }

    pub fn price_multiplier(&self) -> f64 {
        match self {
            ModelAssignment::Qwen9B => 1.0,
            ModelAssignment::Gemma12B => 2.0,
            ModelAssignment::Qwen35B => 2.5,
            ModelAssignment::Phi4Reasoning => 3.0,
            ModelAssignment::Gemma26B => 4.0,
            ModelAssignment::Gemma26B512K => 5.0,
        }
    }
}

/// Full service definition with delivery metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDefinition {
    pub service_type: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub tier: ServiceTier,
    pub model: ModelAssignment,
    pub base_price_cents: i64,
    pub system_prompt: &'static str,
    pub user_prompt_template: &'static str,
    pub input_format: &'static str,
    pub output_format: &'static str,
    pub example_input: &'static str,
    pub example_output: &'static str,
    pub max_output_tokens: i32,
    pub max_context_length: i32,
}

/// The complete service catalog
pub const SERVICE_CATALOG: &[ServiceDefinition] = &[
    // ═══════════════════════════════════════════════════════════
    // TIER 1: MICRO-TASKS ($0.09 - $0.49)
    // ═══════════════════════════════════════════════════════════
    ServiceDefinition {
        service_type: "code_lint_fix",
        name: "Code Lint Fix",
        description: "Auto-fix lint warnings and format code",
        tier: ServiceTier::MicroTask,
        model: ModelAssignment::Qwen9B,
        base_price_cents: 19,
        system_prompt: "You are a code formatting expert. Fix lint warnings, format code, and return clean, valid code only. No explanations.",
        user_prompt_template: "Fix and format this code:\n\n{input}",
        input_format: "source_code",
        output_format: "formatted_code",
        example_input: "fn main(){let x=5;println!(\"{}\",x)}",
        example_output: "fn main() {\n    let x = 5;\n    println!(\"{}\", x);\n}",
        max_output_tokens: 512,
        max_context_length: 131072,
    },
    ServiceDefinition {
        service_type: "git_commit_msg",
        name: "Git Commit Msg",
        description: "Generate conventional commit messages from diffs",
        tier: ServiceTier::MicroTask,
        model: ModelAssignment::Qwen9B,
        base_price_cents: 9,
        system_prompt: "Generate concise conventional commit messages. Format: type(scope): description. One line only.",
        user_prompt_template: "Generate a commit message for this diff:\n\n{input}",
        input_format: "git_diff",
        output_format: "commit_message",
        example_input: "diff --git a/src/main.rs b/src/main.rs\n+ fn hello() { println!(\"hi\"); }",
        example_output: "feat(main): add hello function",
        max_output_tokens: 512,
        max_context_length: 131072,
    },
    ServiceDefinition {
        service_type: "regex_generator",
        name: "Regex Generator",
        description: "Generate regex patterns from natural language descriptions",
        tier: ServiceTier::MicroTask,
        model: ModelAssignment::Qwen9B,
        base_price_cents: 29,
        system_prompt: "You are a regex expert. Generate valid regex patterns. Return only the pattern, no explanation.",
        user_prompt_template: "Generate a regex for: {input}",
        input_format: "natural_language",
        output_format: "regex_pattern",
        example_input: "email validation",
        example_output: "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$",
        max_output_tokens: 512,
        max_context_length: 131072,
    },
    ServiceDefinition {
        service_type: "shell_one_liner",
        name: "Shell One-Liner",
        description: "Generate shell commands for common tasks",
        tier: ServiceTier::MicroTask,
        model: ModelAssignment::Qwen9B,
        base_price_cents: 19,
        system_prompt: "Generate efficient shell one-liners. Return only the command, no explanation.",
        user_prompt_template: "Shell command to: {input}",
        input_format: "natural_language",
        output_format: "shell_command",
        example_input: "find files modified in the last hour",
        example_output: "find . -type f -mmin -60",
        max_output_tokens: 512,
        max_context_length: 131072,
    },
    ServiceDefinition {
        service_type: "csv_converter",
        name: "CSV Converter",
        description: "Convert between CSV, JSON, YAML formats",
        tier: ServiceTier::MicroTask,
        model: ModelAssignment::Qwen9B,
        base_price_cents: 9,
        system_prompt: "Convert data formats accurately. Preserve all data. Return only the converted output.",
        user_prompt_template: "Convert this to {format}:\n\n{input}",
        input_format: "data",
        output_format: "data",
        example_input: "name,age\nAlice,30\nBob,25",
        example_output: "[\n  {\"name\": \"Alice\", \"age\": 30},\n  {\"name\": \"Bob\", \"age\": 25}\n]",
        max_output_tokens: 512,
        max_context_length: 131072,
    },
    ServiceDefinition {
        service_type: "variable_namer",
        name: "Variable Namer",
        description: "Generate clear variable and function names",
        tier: ServiceTier::MicroTask,
        model: ModelAssignment::Qwen9B,
        base_price_cents: 9,
        system_prompt: "Generate clear, idiomatic variable and function names. One suggestion per line.",
        user_prompt_template: "Name this: {input}",
        input_format: "description",
        output_format: "names",
        example_input: "function that hashes user IDs for deduplication",
        example_output: "hash_user_id\nuser_id_hash\ndeduplicate_users",
        max_output_tokens: 512,
        max_context_length: 131072,
    },
    ServiceDefinition {
        service_type: "json_schema_gen",
        name: "JSON Schema Gen",
        description: "Generate JSON Schema from example data",
        tier: ServiceTier::MicroTask,
        model: ModelAssignment::Qwen9B,
        base_price_cents: 19,
        system_prompt: "Generate accurate JSON Schema from example data. Include types, required fields, and constraints.",
        user_prompt_template: "Generate JSON Schema for:\n\n{input}",
        input_format: "json_example",
        output_format: "json_schema",
        example_input: "{\"name\": \"Alice\", \"age\": 30}",
        example_output: "{\"type\": \"object\", \"properties\": {\"name\": {\"type\": \"string\"}, \"age\": {\"type\": \"integer\"}}, \"required\": [\"name\", \"age\"]}",
        max_output_tokens: 512,
        max_context_length: 131072,
    },
    ServiceDefinition {
        service_type: "dockerfile_review",
        name: "Dockerfile Review",
        description: "Optimize Dockerfiles for size and security",
        tier: ServiceTier::MicroTask,
        model: ModelAssignment::Qwen9B,
        base_price_cents: 29,
        system_prompt: "Review Dockerfiles for security and size optimization. List issues and suggestions.",
        user_prompt_template: "Review this Dockerfile:\n\n{input}",
        input_format: "dockerfile",
        output_format: "review_notes",
        example_input: "FROM ubuntu:latest\nRUN apt-get update && apt-get install -y python3",
        example_output: "Issues:\n1. Use specific tag instead of 'latest'\n2. Combine RUN commands to reduce layers\n3. Add --no-install-recommends",
        max_output_tokens: 512,
        max_context_length: 131072,
    },
    ServiceDefinition {
        service_type: "sql_formatter",
        name: "SQL Formatter",
        description: "Pretty-print and optimize SQL queries",
        tier: ServiceTier::MicroTask,
        model: ModelAssignment::Qwen9B,
        base_price_cents: 9,
        system_prompt: "Format SQL queries with proper indentation and capitalization. Suggest optimizations if applicable.",
        user_prompt_template: "Format this SQL:\n\n{input}",
        input_format: "sql_query",
        output_format: "formatted_sql",
        example_input: "select * from users where age>18",
        example_output: "SELECT *\nFROM users\nWHERE age > 18;",
        max_output_tokens: 512,
        max_context_length: 131072,
    },
    ServiceDefinition {
        service_type: "markdown_table",
        name: "Markdown Table",
        description: "Convert data to markdown tables",
        tier: ServiceTier::MicroTask,
        model: ModelAssignment::Qwen9B,
        base_price_cents: 9,
        system_prompt: "Convert data to well-formatted markdown tables. Align columns properly.",
        user_prompt_template: "Convert to markdown table:\n\n{input}",
        input_format: "data",
        output_format: "markdown_table",
        example_input: "Name | Age\nAlice | 30\nBob | 25",
        example_output: "| Name  | Age |\n|-------|-----|\n| Alice | 30  |\n| Bob   | 25  |",
        max_output_tokens: 512,
        max_context_length: 131072,
    },
    // ═══════════════════════════════════════════════════════════
    // TIER 2: REAL WORK ($0.50 - $2.99)
    // ═══════════════════════════════════════════════════════════
    ServiceDefinition {
        service_type: "codebase_qa",
        name: "Codebase Q&A",
        description: "Ask questions about large codebases (up to 128k tokens)",
        tier: ServiceTier::RealWork,
        model: ModelAssignment::Gemma12B,
        base_price_cents: 199,
        system_prompt: "You are a senior software engineer analyzing a codebase. Answer questions accurately based on the provided code. If uncertain, say so.",
        user_prompt_template: "Codebase:\n\n{codebase}\n\nQuestion: {question}",
        input_format: "codebase_text",
        output_format: "answer",
        example_input: "Where is the authentication logic?",
        example_output: "The authentication logic is in src/auth.rs, lines 45-120. It uses JWT tokens with a 24h expiry.",
        max_output_tokens: 1024,
        max_context_length: 131072,
    },
    ServiceDefinition {
        service_type: "doc_to_api_spec",
        name: "Doc to API Spec",
        description: "Convert README/examples to OpenAPI specification",
        tier: ServiceTier::RealWork,
        model: ModelAssignment::Qwen35B,
        base_price_cents: 149,
        system_prompt: "Convert API documentation to valid OpenAPI 3.0 specification. Include all endpoints, parameters, and response schemas.",
        user_prompt_template: "Convert to OpenAPI spec:\n\n{input}",
        input_format: "documentation",
        output_format: "openapi_yaml",
        example_input: "GET /users - returns list of users\nPOST /users - creates a user with name and email",
        example_output: "openapi: 3.0.0\npaths:\n  /users:\n    get:\n      summary: List users\n    post:\n      summary: Create user",
        max_output_tokens: 1024,
        max_context_length: 131072,
    },
    ServiceDefinition {
        service_type: "log_analyzer",
        name: "Log Analyzer",
        description: "Analyze large log files and find root causes",
        tier: ServiceTier::RealWork,
        model: ModelAssignment::Gemma12B,
        base_price_cents: 99,
        system_prompt: "Analyze log files and identify errors, patterns, and root causes. Be specific about line numbers and timestamps.",
        user_prompt_template: "Analyze these logs:\n\n{input}",
        input_format: "log_text",
        output_format: "analysis_report",
        example_input: "ERROR 2024-01-01 10:00:00 Connection refused\nERROR 2024-01-01 10:00:01 Connection refused",
        example_output: "Root cause: Database connection failure starting at 10:00:00.\nImpact: All requests failing.\nSuggestion: Check database service status.",
        max_output_tokens: 1024,
        max_context_length: 131072,
    },
    ServiceDefinition {
        service_type: "privacy_doc_review",
        name: "Privacy Doc Review",
        description: "Analyze sensitive documents locally — data never leaves your machine",
        tier: ServiceTier::RealWork,
        model: ModelAssignment::Gemma26B,
        base_price_cents: 249,
        system_prompt: "Analyze documents for privacy compliance, sensitive data exposure, and regulatory issues. Be thorough and specific.",
        user_prompt_template: "Review this document for privacy issues:\n\n{input}",
        input_format: "document_text",
        output_format: "review_report",
        example_input: "Patient record: John Doe, SSN 123-45-6789, diagnosed with...",
        example_output: "CRITICAL: SSN exposed in plain text.\nMEDIUM: Medical diagnosis not properly anonymized.\nRecommendation: Redact PII before sharing.",
        max_output_tokens: 1024,
        max_context_length: 262144,
    },
    ServiceDefinition {
        service_type: "uncensored_analysis",
        name: "Uncensored Analysis",
        description: "Neutral analysis of controversial or restricted topics",
        tier: ServiceTier::RealWork,
        model: ModelAssignment::Qwen35B,
        base_price_cents: 199,
        system_prompt: "Provide neutral, factual analysis without bias or moral judgments. Focus on evidence and logical reasoning.",
        user_prompt_template: "Analyze neutrally:\n\n{input}",
        input_format: "text",
        output_format: "analysis",
        example_input: "The political situation in X country",
        example_output: "Key factors: [factual analysis without bias]",
        max_output_tokens: 1024,
        max_context_length: 131072,
    },
    ServiceDefinition {
        service_type: "code_review",
        name: "Code Review",
        description: "Deep code review with architecture suggestions",
        tier: ServiceTier::RealWork,
        model: ModelAssignment::Qwen35B,
        base_price_cents: 149,
        system_prompt: "Perform thorough code review. Check for bugs, security issues, performance, and architecture. Suggest improvements.",
        user_prompt_template: "Review this code:\n\n{input}",
        input_format: "source_code",
        output_format: "review_notes",
        example_input: "fn process(data: Vec<u8>) -> Vec<u8> { data.reverse(); data }",
        example_output: "Issues:\n1. Unnecessary clone — reverse in place\n2. No error handling for empty input\n3. Consider returning Result instead",
        max_output_tokens: 1024,
        max_context_length: 131072,
    },
    ServiceDefinition {
        service_type: "test_generator",
        name: "Test Generator",
        description: "Generate unit tests from function signatures",
        tier: ServiceTier::RealWork,
        model: ModelAssignment::Qwen9B,
        base_price_cents: 99,
        system_prompt: "Generate comprehensive unit tests. Cover edge cases, error conditions, and happy paths. Use standard testing frameworks.",
        user_prompt_template: "Generate tests for:\n\n{input}",
        input_format: "source_code",
        output_format: "test_code",
        example_input: "fn divide(a: f64, b: f64) -> f64 { a / b }",
        example_output: "#[test]\nfn test_divide_normal() { assert_eq!(divide(10.0, 2.0), 5.0); }\n#[test]\nfn test_divide_by_zero() { assert!(divide(10.0, 0.0).is_infinite()); }",
        max_output_tokens: 1024,
        max_context_length: 131072,
    },
    ServiceDefinition {
        service_type: "api_client_gen",
        name: "API Client Gen",
        description: "Generate Python/JS clients from curl examples",
        tier: ServiceTier::RealWork,
        model: ModelAssignment::Qwen35B,
        base_price_cents: 149,
        system_prompt: "Generate clean API client code from examples. Include error handling, type hints, and documentation.",
        user_prompt_template: "Generate Python client for:\n\n{input}",
        input_format: "curl_examples",
        output_format: "python_code",
        example_input: "curl -X POST https://api.example.com/users -d '{\"name\":\"Alice\"}'",
        example_output: "import requests\n\ndef create_user(name: str) -> dict:\n    resp = requests.post('https://api.example.com/users', json={'name': name})\n    resp.raise_for_status()\n    return resp.json()",
        max_output_tokens: 1024,
        max_context_length: 131072,
    },
    ServiceDefinition {
        service_type: "config_validator",
        name: "Config Validator",
        description: "Validate nginx/k8s/docker configs for issues",
        tier: ServiceTier::RealWork,
        model: ModelAssignment::Qwen9B,
        base_price_cents: 69,
        system_prompt: "Validate configuration files for syntax errors, security issues, and best practices. List all problems found.",
        user_prompt_template: "Validate this config:\n\n{input}",
        input_format: "config_file",
        output_format: "validation_report",
        example_input: "server { listen 80; root /var/www; }",
        example_output: "Issues:\n1. Missing server_name directive\n2. No HTTPS redirect\n3. root path not absolute",
        max_output_tokens: 1024,
        max_context_length: 131072,
    },
    ServiceDefinition {
        service_type: "diff_explainer",
        name: "Diff Explainer",
        description: "Explain what a PR actually changes in plain English",
        tier: ServiceTier::RealWork,
        model: ModelAssignment::Qwen9B,
        base_price_cents: 79,
        system_prompt: "Explain code changes in plain English. Focus on what changed and why it matters. Non-technical stakeholders should understand.",
        user_prompt_template: "Explain this diff:\n\n{input}",
        input_format: "git_diff",
        output_format: "explanation",
        example_input: "diff --git a/src/auth.rs b/src/auth.rs\n- fn login(user: &str) {\n+ fn login(user: &str, password: &str) {",
        example_output: "This change adds password authentication to the login function. Previously, only a username was required. This improves security by requiring both credentials.",
        max_output_tokens: 1024,
        max_context_length: 131072,
    },
    // ═══════════════════════════════════════════════════════════
    // TIER 3: HEAVY LIFTING ($3.00 - $9.99)
    // ═══════════════════════════════════════════════════════════
    ServiceDefinition {
        service_type: "repo_refactor",
        name: "Repo Refactor",
        description: "Refactor large codebases (up to 256k tokens)",
        tier: ServiceTier::HeavyLifting,
        model: ModelAssignment::Gemma26B,
        base_price_cents: 499,
        system_prompt: "Refactor codebases while preserving functionality. Suggest architectural improvements, extract modules, and modernize patterns.",
        user_prompt_template: "Refactor this codebase to use {pattern}:\n\n{input}",
        input_format: "codebase_text",
        output_format: "refactored_code",
        example_input: "Refactor to use async/await",
        example_output: "[Refactored code with async patterns, error handling, and proper cancellation]",
        max_output_tokens: 2048,
        max_context_length: 262144,
    },
    ServiceDefinition {
        service_type: "book_summary_qa",
        name: "Book Summary + Q&A",
        description: "Upload entire books/PDFs and ask detailed questions (up to 512k tokens)",
        tier: ServiceTier::HeavyLifting,
        model: ModelAssignment::Gemma26B512K,
        base_price_cents: 399,
        system_prompt: "Analyze long documents thoroughly. Answer specific questions with citations to relevant sections. Be precise about page numbers and quotes.",
        user_prompt_template: "Document:\n\n{document}\n\nQuestion: {question}",
        input_format: "document_text",
        output_format: "detailed_answer",
        example_input: "What are the main themes in Chapter 3?",
        example_output: "The main themes in Chapter 3 are:\n1. [theme with specific quote and page number]\n2. [theme with specific quote and page number]",
        max_output_tokens: 2048,
        max_context_length: 524288,
    },
    ServiceDefinition {
        service_type: "contract_review",
        name: "Contract Review",
        description: "Find liability clauses and risks in legal agreements",
        tier: ServiceTier::HeavyLifting,
        model: ModelAssignment::Gemma26B,
        base_price_cents: 799,
        system_prompt: "Review legal documents for risks, liability, and unfavorable terms. Flag specific clauses and suggest redlines. Note: This is not legal advice.",
        user_prompt_template: "Review this contract for risks:\n\n{input}",
        input_format: "legal_document",
        output_format: "risk_report",
        example_input: "Section 7: Indemnification...",
        example_output: "HIGH RISK: Broad indemnification clause in Section 7 requires you to cover all third-party claims regardless of fault.\nSuggestion: Limit to claims caused by your negligence.",
        max_output_tokens: 2048,
        max_context_length: 262144,
    },
    ServiceDefinition {
        service_type: "threat_intel",
        name: "Threat Intel Report",
        description: "Analyze malware and extract IOCs for security teams",
        tier: ServiceTier::HeavyLifting,
        model: ModelAssignment::Phi4Reasoning,
        base_price_cents: 599,
        system_prompt: "Analyze security data and extract IOCs (IPs, hashes, domains, patterns). Provide structured threat intelligence report.",
        user_prompt_template: "Analyze this security data:\n\n{input}",
        input_format: "security_data",
        output_format: "threat_report",
        example_input: "192.168.1.100 connected to evil.com at 2024-01-01",
        example_output: "IOCs Found:\n- IP: 192.168.1.100\n- Domain: evil.com\n- Timestamp: 2024-01-01\nRecommendation: Block domain, investigate host.",
        max_output_tokens: 2048,
        max_context_length: 262144,
    },
    ServiceDefinition {
        service_type: "architecture_review",
        name: "Architecture Review",
        description: "Review system design and suggest improvements",
        tier: ServiceTier::HeavyLifting,
        model: ModelAssignment::Qwen35B,
        base_price_cents: 699,
        system_prompt: "Review system architecture for scalability, reliability, security, and cost. Suggest concrete improvements with trade-offs.",
        user_prompt_template: "Review this architecture:\n\n{input}",
        input_format: "architecture_description",
        output_format: "review_report",
        example_input: "Monolith with PostgreSQL, Redis cache, nginx load balancer",
        example_output: "Strengths: Simple, proven stack.\nConcerns: Single point of failure, scaling limits.\nSuggestions: 1. Add read replicas 2. Consider service decomposition 3. Implement circuit breakers",
        max_output_tokens: 2048,
        max_context_length: 131072,
    },
    ServiceDefinition {
        service_type: "research_synthesis",
        name: "Research Synthesis",
        description: "Synthesize multiple papers into literature reviews",
        tier: ServiceTier::HeavyLifting,
        model: ModelAssignment::Gemma26B512K,
        base_price_cents: 499,
        system_prompt: "Synthesize research papers into coherent literature reviews. Identify gaps, contradictions, and consensus. Cite specific papers.",
        user_prompt_template: "Synthesize these papers:\n\n{input}",
        input_format: "paper_summaries",
        output_format: "literature_review",
        example_input: "Paper 1: LLMs show emergent abilities at scale...\nPaper 2: Emergent abilities may be metric artifacts...",
        example_output: "Current consensus: [synthesis]\nKey debate: [contradiction between papers 1 and 2]\nResearch gap: [what's missing]",
        max_output_tokens: 2048,
        max_context_length: 524288,
    },
    ServiceDefinition {
        service_type: "legacy_modernize",
        name: "Legacy Modernize",
        description: "Convert legacy code to modern languages and patterns",
        tier: ServiceTier::HeavyLifting,
        model: ModelAssignment::Gemma26B,
        base_price_cents: 599,
        system_prompt: "Convert legacy code to modern equivalents. Preserve business logic, add type safety, and follow current best practices.",
        user_prompt_template: "Convert this to {target_language}:\n\n{input}",
        input_format: "legacy_code",
        output_format: "modern_code",
        example_input: "10 PRINT 'HELLO'\n20 GOTO 10",
        example_output: "while True:\n    print('HELLO')",
        max_output_tokens: 2048,
        max_context_length: 262144,
    },
    ServiceDefinition {
        service_type: "compliance_audit",
        name: "Compliance Audit",
        description: "Check codebases for SOC2/GDPR/HIPAA compliance gaps",
        tier: ServiceTier::HeavyLifting,
        model: ModelAssignment::Phi4Reasoning,
        base_price_cents: 799,
        system_prompt: "Audit codebases for compliance gaps. Check for PII handling, encryption, access controls, and audit trails. Be specific about violations.",
        user_prompt_template: "Audit for {standard} compliance:\n\n{input}",
        input_format: "codebase_text",
        output_format: "audit_report",
        example_input: "SOC2",
        example_output: "FAIL: No audit logging for authentication events\nPASS: Data encrypted at rest\nWARNING: Access controls not implemented for admin endpoints",
        max_output_tokens: 2048,
        max_context_length: 262144,
    },
    // ═══════════════════════════════════════════════════════════
    // TIER 4: LOCAL-ONLY SUPERPOWERS ($9.99 - $49.99)
    // These require local model advantages: massive context, uncensored, bulk, custom
    // ═══════════════════════════════════════════════════════════
    ServiceDefinition {
        service_type: "full_repo_analysis",
        name: "Full Repo Analysis",
        description: "Analyze entire Git repositories up to 512k tokens in one shot",
        tier: ServiceTier::LocalOnly,
        model: ModelAssignment::Gemma26B512K,
        base_price_cents: 1499,
        system_prompt: "Analyze entire codebases comprehensively. Find bugs, security issues, performance bottlenecks, and architectural problems. Be specific about file paths and line numbers.",
        user_prompt_template: "Analyze this entire codebase:\n\n{input}\n\nFocus: {focus}",
        input_format: "codebase_text",
        output_format: "comprehensive_report",
        example_input: "[200k lines of Rust code]",
        example_output: "Critical Issues Found:\n1. src/auth.rs:45 - SQL injection vulnerability\n2. src/db.rs:120 - N+1 query pattern\n3. Memory leak in src/cache.rs:88",
        max_output_tokens: 4096,
        max_context_length: 524288,
    },
    ServiceDefinition {
        service_type: "bulk_document_processing",
        name: "Bulk Document Processing",
        description: "Process 1000+ documents with no rate limits. Extract tables, entities, relationships",
        tier: ServiceTier::LocalOnly,
        model: ModelAssignment::Gemma26B512K,
        base_price_cents: 1999,
        system_prompt: "Process large volumes of documents efficiently. Extract structured data, identify entities, and find relationships across documents. Handle duplicates and inconsistencies.",
        user_prompt_template: "Process these documents and extract {extraction_target}:\n\n{input}",
        input_format: "document_collection",
        output_format: "structured_extraction",
        example_input: "[1000 PDF invoices]",
        example_output: "Extracted 1000 invoices:\n- Total revenue: $1.2M\n- Top vendor: Acme Corp ($340K)\n- Duplicate invoices found: 3",
        max_output_tokens: 4096,
        max_context_length: 524288,
    },
    ServiceDefinition {
        service_type: "uncensored_threat_analysis",
        name: "Uncensored Threat Analysis",
        description: "Analyze malware, C2 traffic, and threat actor TTPs without cloud filters",
        tier: ServiceTier::LocalOnly,
        model: ModelAssignment::Phi4Reasoning,
        base_price_cents: 2499,
        system_prompt: "Analyze security threats and malware without censorship. Extract IOCs, identify attack patterns, and attribute threat actors. Be specific and technical.",
        user_prompt_template: "Analyze this threat data:\n\n{input}",
        input_format: "security_data",
        output_format: "threat_report",
        example_input: "[Malware sample memory dump + network traffic]",
        example_output: "Threat Actor: APT29 (Cozy Bear)\nIOCs: 192.168.1.100, evil.com, hash: abc123\nTTPs: Spear phishing → PowerShell → WMI persistence",
        max_output_tokens: 4096,
        max_context_length: 262144,
    },
    ServiceDefinition {
        service_type: "custom_model_inference",
        name: "Custom Model Inference",
        description: "Run your fine-tuned models on private data. Domain-specific analysis",
        tier: ServiceTier::LocalOnly,
        model: ModelAssignment::Gemma26B512K,
        base_price_cents: 2999,
        system_prompt: "Execute domain-specific analysis using custom model weights. Apply specialized knowledge to private data sets. Maintain confidentiality.",
        user_prompt_template: "Run analysis on private data using custom model {model_name}:\n\n{input}",
        input_format: "private_data",
        output_format: "domain_analysis",
        example_input: "[Medical imaging metadata + patient records]",
        example_output: "Analysis complete. 47 high-risk cases identified. 12 require immediate review. Full report encrypted with provided key.",
        max_output_tokens: 4096,
        max_context_length: 524288,
    },
    ServiceDefinition {
        service_type: "realtime_log_analysis",
        name: "Real-Time Log Analysis",
        description: "Stream logs into 128k context windows for anomaly detection in real-time",
        tier: ServiceTier::LocalOnly,
        model: ModelAssignment::Qwen35B,
        base_price_cents: 1799,
        system_prompt: "Analyze streaming log data in real-time. Detect anomalies, correlate events across services, and identify root causes. Prioritize by severity.",
        user_prompt_template: "Analyze these logs for anomalies:\n\n{input}",
        input_format: "log_stream",
        output_format: "anomaly_report",
        example_input: "[10k log lines from production cluster]",
        example_output: "Anomalies detected:\n- 15:32:11: Database connection pool exhausted (critical)\n- 15:33:45: Unusual API error rate spike (warning)\n- 15:35:00: Memory usage climbing in service worker-3",
        max_output_tokens: 4096,
        max_context_length: 131072,
    },
    ServiceDefinition {
        service_type: "massive_context_qa",
        name: "Massive Context Q&A",
        description: "Upload 500k tokens of context and ask complex multi-hop questions",
        tier: ServiceTier::LocalOnly,
        model: ModelAssignment::Gemma26B512K,
        base_price_cents: 1299,
        system_prompt: "Answer complex questions requiring reasoning across massive context. Connect information from distant parts of the document. Cite specific sections.",
        user_prompt_template: "Context:\n\n{context}\n\nQuestion: {question}",
        input_format: "massive_text",
        output_format: "detailed_answer",
        example_input: "[Entire novel + character list + timeline]",
        example_output: "The answer requires connecting Chapter 3 (page 45) with the appendix character note. Elizabeth's motivation stems from...",
        max_output_tokens: 4096,
        max_context_length: 524288,
    },
    ServiceDefinition {
        service_type: "codebase_migration_plan",
        name: "Codebase Migration Plan",
        description: "Plan migration of 500k+ line monoliths to microservices with full dependency mapping",
        tier: ServiceTier::LocalOnly,
        model: ModelAssignment::Gemma26B512K,
        base_price_cents: 3499,
        system_prompt: "Create detailed migration plans for large codebases. Map dependencies, identify service boundaries, estimate effort, and flag risks. Be specific about files and modules.",
        user_prompt_template: "Plan migration of this codebase to {target_architecture}:\n\n{input}",
        input_format: "codebase_text",
        output_format: "migration_plan",
        example_input: "[500k line Java monolith]",
        example_output: "Migration Plan:\nPhase 1: Extract auth service (files: src/auth/*, src/session/*)\nPhase 2: Extract payment service (files: src/billing/*)\nDependencies: 47 cross-module calls identified",
        max_output_tokens: 4096,
        max_context_length: 524288,
    },
    ServiceDefinition {
        service_type: "private_medical_analysis",
        name: "Private Medical Analysis",
        description: "Analyze medical records with HIPAA guarantee — data never leaves your machine",
        tier: ServiceTier::LocalOnly,
        model: ModelAssignment::Phi4Reasoning,
        base_price_cents: 3999,
        system_prompt: "Analyze medical data with strict privacy. Identify risk factors, drug interactions, and anomalies. Never include patient identifiers in output. Flag critical findings.",
        user_prompt_template: "Analyze these medical records for {analysis_type}:\n\n{input}",
        input_format: "medical_records",
        output_format: "clinical_analysis",
        example_input: "[Patient history + lab results + imaging notes]",
        example_output: "Critical Finding: Elevated creatinine (2.1 mg/dL) suggests renal impairment.\nDrug Interaction: Warfarin + NSAIDs increases bleeding risk.\nRecommendation: Nephrology consult advised.",
        max_output_tokens: 4096,
        max_context_length: 262144,
    },
    ServiceDefinition {
        service_type: "financial_forensic_analysis",
        name: "Financial Forensic Analysis",
        description: "Analyze financial documents for fraud, money laundering, and compliance issues locally",
        tier: ServiceTier::LocalOnly,
        model: ModelAssignment::Phi4Reasoning,
        base_price_cents: 4499,
        system_prompt: "Analyze financial data for irregularities, fraud patterns, and compliance violations. Identify suspicious transactions, unusual patterns, and regulatory risks. Be thorough and specific.",
        user_prompt_template: "Analyze these financial records for {analysis_type}:\n\n{input}",
        input_format: "financial_data",
        output_format: "forensic_report",
        example_input: "[10 years of transaction logs + wire transfers + shell company records]",
        example_output: "Red Flags Identified:\n1. Circular transactions: $2.3M between 3 entities\n2. Structuring: 47 deposits just under $10K reporting threshold\n3. Shell company network: 12 entities with shared directors",
        max_output_tokens: 4096,
        max_context_length: 262144,
    },
    ServiceDefinition {
        service_type: "adversarial_red_team",
        name: "Adversarial Red Team",
        description: "Uncensored security testing and vulnerability analysis that cloud APIs refuse",
        tier: ServiceTier::LocalOnly,
        model: ModelAssignment::Qwen35B,
        base_price_cents: 2999,
        system_prompt: "Perform thorough security analysis and red team assessments. Identify vulnerabilities, attack vectors, and exploitation paths. Be specific and technical. This is authorized security testing.",
        user_prompt_template: "Perform red team analysis on:\n\n{input}",
        input_format: "system_description",
        output_format: "vulnerability_report",
        example_input: "[Web app source code + API docs + infrastructure config]",
        example_output: "Critical Vulnerabilities:\n1. SQL injection in /api/search (line 45)\n2. JWT secret hardcoded in config.js\n3. SSRF in image upload via URL parameter",
        max_output_tokens: 4096,
        max_context_length: 131072,
    },
    ServiceDefinition {
        service_type: "multi_model_ensemble",
        name: "Multi-Model Ensemble",
        description: "Run multiple local models on same input and synthesize consensus answer",
        tier: ServiceTier::LocalOnly,
        model: ModelAssignment::Gemma26B,
        base_price_cents: 1999,
        system_prompt: "Synthesize answers from multiple model outputs. Identify consensus, disagreements, and confidence levels. Provide the most reliable answer with reasoning.",
        user_prompt_template: "Synthesize these model outputs into a consensus answer:\n\n{input}",
        input_format: "model_outputs",
        output_format: "consensus_answer",
        example_input: "[Qwen 9B output + Gemma 26B output + Phi-4 output on same question]",
        example_output: "Consensus: All models agree on primary conclusion.\nDisagreement: Model confidence varies on secondary point (60% vs 85%).\nRecommended Answer: [synthesized with confidence intervals]",
        max_output_tokens: 4096,
        max_context_length: 524288,
    },
];

/// Lookup a service definition by type
pub fn get_service_definition(service_type: &str) -> Option<&'static ServiceDefinition> {
    SERVICE_CATALOG.iter().find(|s| s.service_type == service_type)
}

/// Get all service types for a given tier
#[allow(dead_code)]
pub fn get_services_by_tier(tier: ServiceTier) -> Vec<&'static ServiceDefinition> {
    SERVICE_CATALOG.iter().filter(|s| s.tier == tier).collect()
}

/// Get all service types as strings
#[allow(dead_code)]
pub fn get_all_service_types() -> Vec<&'static str> {
    SERVICE_CATALOG.iter().map(|s| s.service_type).collect()
}

/// Calculate price with demand modifier
/// If similar services exist (same type), price drops
pub fn calculate_price(base_cents: i64, similar_count: usize, seller_reputation: i64) -> i64 {
    let demand_multiplier = if similar_count >= 3 {
        0.8
    } else if similar_count >= 1 {
        0.9
    } else {
        1.0
    };

    let reputation_multiplier = 1.0 + (seller_reputation as f64 / 100.0).min(0.3);

    let adjusted = (base_cents as f64) * demand_multiplier * reputation_multiplier;
    adjusted as i64
}

/// Check if a service type already exists in the marketplace
pub fn is_duplicate(service_type: &str, existing_types: &[String]) -> bool {
    existing_types.iter().any(|t| t == service_type)
}

/// Find a gap in the marketplace — returns service types with fewest listings
pub fn find_marketplace_gaps(existing_types: &[String]) -> Vec<&'static ServiceDefinition> {
    let mut type_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for t in existing_types {
        *type_counts.entry(t.as_str()).or_insert(0) += 1;
    }

    let mut gaps: Vec<(&ServiceDefinition, usize)> = SERVICE_CATALOG
        .iter()
        .map(|def| {
            let count = type_counts.get(def.service_type).copied().unwrap_or(0);
            (def, count)
        })
        .collect();

    // Sort by count ascending (fewest listings first), then by tier (micro first)
    gaps.sort_by(|a, b| {
        a.1.cmp(&b.1).then_with(|| match (a.0.tier, b.0.tier) {
            (ServiceTier::MicroTask, _) => std::cmp::Ordering::Less,
            (_, ServiceTier::MicroTask) => std::cmp::Ordering::Greater,
            (ServiceTier::RealWork, ServiceTier::HeavyLifting) => std::cmp::Ordering::Less,
            (ServiceTier::HeavyLifting, ServiceTier::RealWork) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        })
    });

    gaps.into_iter().map(|(def, _)| def).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_service_definition_exists() {
        let def = get_service_definition("code_review");
        assert!(def.is_some());
        let def = def.unwrap();
        assert_eq!(def.service_type, "code_review");
        assert!(def.base_price_cents > 0);
    }

    #[test]
    fn test_get_service_definition_missing() {
        let def = get_service_definition("nonexistent_service_type");
        assert!(def.is_none());
    }

    #[test]
    fn test_tier_price_ranges() {
        let micro = ServiceTier::MicroTask;
        let (min, max) = micro.price_range_cents();
        assert_eq!(min, 9);
        assert_eq!(max, 49);
        assert_eq!(micro.base_price_cents(), 19);

        let heavy = ServiceTier::HeavyLifting;
        let (min, max) = heavy.price_range_cents();
        assert_eq!(min, 300);
        assert_eq!(max, 999);

        let local = ServiceTier::LocalOnly;
        let (min, max) = local.price_range_cents();
        assert_eq!(min, 999);
        assert_eq!(max, 4999);
    }

    #[test]
    fn test_model_assignment_multiplier() {
        assert_eq!(ModelAssignment::Qwen9B.price_multiplier(), 1.0);
        assert_eq!(ModelAssignment::Gemma12B.price_multiplier(), 2.0);
        assert_eq!(ModelAssignment::Qwen35B.price_multiplier(), 2.5);
        assert_eq!(ModelAssignment::Phi4Reasoning.price_multiplier(), 3.0);
        assert_eq!(ModelAssignment::Gemma26B.price_multiplier(), 4.0);
    }

    #[test]
    fn test_calculate_price_demand() {
        // No competition = base price
        let p1 = calculate_price(100, 0, 0);
        assert_eq!(p1, 100);

        // Some competition = 90%
        let p2 = calculate_price(100, 1, 0);
        assert_eq!(p2, 90);

        // Lots of competition = 80%
        let p3 = calculate_price(100, 5, 0);
        assert_eq!(p3, 80);
    }

    #[test]
    fn test_calculate_price_reputation() {
        // Base price with 0 rep
        let p1 = calculate_price(100, 0, 0);
        assert_eq!(p1, 100);

        // 50 rep = 50% of max 30% bonus = 1.15x, but min() caps at 0.3 so actually 1.3x
        // Wait: (50 / 100.0).min(0.3) = 0.3, so 1.0 + 0.3 = 1.3x
        let p2 = calculate_price(100, 0, 50);
        assert_eq!(p2, 130);

        // 100 rep = max 30% bonus = 1.3x
        let p3 = calculate_price(100, 0, 100);
        assert_eq!(p3, 130);
    }

    #[test]
    fn test_is_duplicate() {
        let existing = vec!["code_review".to_string(), "text_processing".to_string()];
        assert!(is_duplicate("code_review", &existing));
        assert!(!is_duplicate("analysis", &existing));
    }

    #[test]
    fn test_find_marketplace_gaps() {
        let existing = vec![
            "code_review".to_string(),
            "code_review".to_string(),
            "text_processing".to_string(),
        ];
        let gaps = find_marketplace_gaps(&existing);
        assert!(!gaps.is_empty());

        // The first gap should NOT be code_review (it has the most listings)
        assert_ne!(gaps[0].service_type, "code_review");
    }

    #[test]
    fn test_catalog_size() {
        let all = get_all_service_types();
        assert_eq!(all.len(), 39, "Catalog should have exactly 39 services");
    }

    #[test]
    fn test_default_model_names() {
        let qwen = ModelAssignment::Qwen9B;
        assert_eq!(qwen.default_model_name(), "synthclaw-9b-128k");

        let gemma = ModelAssignment::Gemma12B;
        assert_eq!(gemma.default_model_name(), "synthclaw-gemma-12b-128k");
    }

    #[test]
    fn test_tier_labels() {
        assert_eq!(ServiceTier::MicroTask.label(), "MICRO");
        assert_eq!(ServiceTier::RealWork.label(), "REAL");
        assert_eq!(ServiceTier::HeavyLifting.label(), "HEAVY");
        assert_eq!(ServiceTier::LocalOnly.label(), "LOCAL");
    }

    #[test]
    fn test_tier_badge_colors() {
        assert_eq!(ServiceTier::MicroTask.badge_color(), "cyan");
        assert_eq!(ServiceTier::RealWork.badge_color(), "yellow");
        assert_eq!(ServiceTier::HeavyLifting.badge_color(), "magenta");
        assert_eq!(ServiceTier::LocalOnly.badge_color(), "green");
    }
}
