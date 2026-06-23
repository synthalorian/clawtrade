# ClawTrade — Local LLM Setup Guide

ClawTrade uses local LLM inference via an OpenAI-compatible API server (llama-swap, llama.cpp, vLLM, etc.). By default it connects to `http://127.0.0.1:8080`.

## Quick Start

### 1. Install llama-swap (recommended)

```bash
git clone https://github.com/mostlygeeks/llama-swap
cd llama-swap
cargo build --release
```

### 2. Configure Model Aliases

ClawTrade expects these generic model aliases. Add them to your `llama-swap` config:

```yaml
# ~/.config/llama-swap/config.yaml
models:
  qwen-9b-128k:
    cmd: llama-server --model /path/to/Qwen3.5-9B-Q4_K_M.gguf --port 8099 --alias qwen-9b-128k

  gemma-12b-128k:
    cmd: llama-server --model /path/to/gemma-4-12b-it.gguf --port 8100 --alias gemma-12b-128k

  qwen-35b-128k:
    cmd: llama-server --model /path/to/Qwen3.6-35B-A3B.gguf --port 8101 --alias qwen-35b-128k

  phi-4-reasoning-256k:
    cmd: llama-server --model /path/to/Phi-4-Reasoning-Plus.gguf --port 8102 --alias phi-4-reasoning-256k

  gemma-26b-256k:
    cmd: llama-server --model /path/to/gemma-4-26b-it.gguf --port 8103 --alias gemma-26b-256k
```

### 3. Override with Your Own Aliases

If you already have model aliases set up, override ClawTrade's defaults with env vars:

```bash
export CLAWTRADE_MODEL_QWEN9B="my-9b-alias"
export CLAWTRADE_MODEL_GEMMA12B="my-gemma-alias"
export CLAWTRADE_MODEL_QWEN35B="my-35b-alias"
export CLAWTRADE_MODEL_PHI4="my-phi-alias"
export CLAWTRADE_MODEL_GEMMA26B="my-26b-alias"
```

### 4. Override the Base URL

If your llama-swap runs on a different port or host:

```bash
export LLM_LOCAL_URL="http://127.0.0.1:8080"
export LLM_LOCAL_MODEL="qwen-9b-128k"  # fallback model
```

## Model Requirements

| Alias | Min VRAM | Context | Best For |
|-------|----------|---------|----------|
| qwen-9b-128k | 6 GB | 128k | Micro-tasks, fast inference |
| gemma-12b-128k | 10 GB | 128k | Real work, balanced |
| qwen-35b-128k | 20 GB | 128k | Complex reasoning |
| phi-4-reasoning-256k | 14 GB | 256k | Deep reasoning, math |
| gemma-26b-256k | 22 GB | 256k | Heavy lifting, huge context |

## Fallback Behavior

If a requested model isn't loaded, ClawTrade falls back to the default model (`LLM_LOCAL_MODEL`, default `qwen-9b-128k`). This means the system works even with only one model available — just slower or lower quality for high-tier services.

## NVIDIA Cloud Fallback (Optional)

Set `NVIDIA_API_KEY` to use NVIDIA's API as a fallback when local inference fails:

```bash
export NVIDIA_API_KEY="nvapi-..."
```

This is never required — ClawTrade is designed to work 100% locally.

## Testing

```bash
curl http://127.0.0.1:8080/v1/models
```

Should return your loaded models. If this works, ClawTrade will work.
