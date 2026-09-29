# Lemonade Setup for Wayfinder

Free local AI for Wayfinder via AMD Lemonade. No API keys needed.

## Quick Setup

### Option 1: Local Lemonade Server (Recommended)

If you have an AMD GPU (WX 5100, RX 580, etc.):

```bash
# Install Lemonade
pip install lemonade-sdk

# Start the server
lemond --host 127.0.0.1 --port 13306

# Install Vulkan backend
lemonade --port 13306 backends install llamacpp:vulkan

# Pull an embedding model
lemonade --port 13306 pull embeddinggemma-300m-f16

# Pull a chat model
lemonade --port 13306 pull qwen2.5-coder-1.5b-instruct-q4_k_m
```

### Option 2: CESARops Community Endpoint

Point Wayfinder at the shared CESARops Lemonade server:

```
Chat LLM:    https://cesarops.com/lemonade/v1
Embeddings:  https://cesarops.com/lemonade/v1
API Key:     wayfinder-free (no key needed, rate-limited)
```

### Option 3: Any OpenAI-Compatible Server

Wayfinder works with any server that exposes `/v1/chat/completions` and `/v1/embeddings`:
- Lemonade (AMD, Vulkan)
- llama.cpp (any GPU)
- Ollama
- koboldcpp
- vLLM
- FreeToken

## Wayfinder Config

### Embeddings (for semantic search, clustering, naming suggestions)

In Wayfinder's Embeddings section:
- **Provider**: Llama (local server)
- **Endpoint**: `http://localhost:13306/v1` (local) or `https://cesarops.com/lemonade/v1` (remote)
- **Model**: `embeddinggemma-300m-f16`

### Chat LLM (for Git Clippy, task decomposition)

In Wayfinder's Git Clippy / Task Decomposer:
- **Endpoint**: `http://localhost:13306/v1` (local) or `https://cesarops.com/lemonade/v1` (remote)
- **Model**: `qwen2.5-coder-1.5b-instruct-q4_k_m`

## System Prompt for Wayfinder Tasks

When Wayfinder calls the chat LLM, the system prompt tells the model what it's doing:

```
You are Wayfinder, an AI assistant for ADHD students and programmers.
Your tasks:
- Semantic search: find files by meaning, not just keywords
- Naming suggestions: suggest clear, descriptive file and function names
- Task decomposition: break big ideas into 8 concrete tasks with code stubs
- Git Clippy: friendly git reminders (chill → nudge → warning → panic)
- File clustering: group related files by semantic similarity

Be concise. Be direct. If you're not sure, say so.
Don't over-explain. ADHD users need focus, not walls of text.
```

## AMD Hardware Notes

- **No ROCm needed**: Lemonade uses Vulkan (mesa radv) on consumer AMD
- **Verified on**: AMD Radeon Pro WX 5100, RX 580
- **Performance**: Sufficient for embeddings and small chat models
- **Future**: Strix Halo (128GB unified) would serve entire classrooms

## File Locations

| File | Location | Purpose |
|---|---|---|
| `lemonade_config.template.json` | Repo root | Template for users |
| `.wayfinder_index/lemonade_config.json` | Index dir | Active config (gitignored) |