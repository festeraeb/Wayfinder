# Lemonade Setup for Wayfinder

Free local AI for Wayfinder via AMD Lemonade. A local Lemonade needs no API key.

Wayfinder's default address is `http://127.0.0.1:13305/api`, Lemonade's own default port. Its Lemonade page has a **Check Lemonade** button that tests the address, both models, one embedding and one chat reply, and says which step failed.

## Quick Setup

### Option 1: Local Lemonade Server (Recommended)

If you have an AMD GPU (WX 5100, RX 580, etc.):

```bash
# Install Lemonade
pip install lemonade-sdk

# Start the server (default port 13305)
lemond --host 127.0.0.1 --port 13305

# Install Vulkan backend
lemonade backends install llamacpp:vulkan

# Pull an embedding model (Wayfinder's default)
lemonade pull nomic-embed-text-v1-GGUF

# Pull a chat model
lemonade pull qwen2.5-coder-1.5b-instruct-q4_k_m
```

### Option 2: CESARops Community Endpoint (Free, No Setup)

Point Wayfinder at the shared CESARops server — no local GPU needed:

```
Chat LLM:    https://cesarops.com/lemonade/v1
Model:       nauti-recovery (Qwen3-0.6B, CPU)
Embeddings:  https://cesarops.com/lemonade/v1
Model:       embeddinggemma-300m (768 dimensions)
API Key:     needed (the endpoint has been key-gated since 2026-10-22)
```

How people get a key is not decided yet. [[operator: decide and fill in]]

Just copy `lemonade_config.template.json` to `.wayfinder_index/lemonade_config.json` and it works.

### Option 3: A Lemonade on another computer, through a Lemonade Nexus mesh

If the Lemonade with the GPU is on another machine (a school server, a teacher's PC), a Lemonade Nexus
client device on your machine gives you a local mesh port that looks like one Lemonade. Point Wayfinder at
it:

```
Lemonade address:  http://127.0.0.1:13306/api
Chat model:        <device>/<model>      e.g. c4-test-lemond/qwen2.5-coder-1.5b-instruct-q6_k
Embedding model:   <device>/<model>      e.g. c4-test-lemond/nomic-embed-text-v1-GGUF
```

Tested 2026-10-30 and again 2026-10-31: the Wayfinder end-to-end suite passed 12/12 this way (setup check,
chat, embeddings, semantic search, save prompter, commit nudge), with every Lemonade call crossing the mesh to
a GTX 1070 on another machine. Setting up the mesh is in the Lemonade Nexus fork's
`docs/Operator-Trust-Bootstrap.md`. This uses a self-hosted test network; it is not a hosted service.

### Option 4: Any OpenAI-Compatible Server

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
- **Endpoint**: `http://127.0.0.1:13305/api` (local), `http://127.0.0.1:13306/api` (Nexus mesh port) or `https://cesarops.com/lemonade/v1` (remote, key needed)
- **Model**: `nomic-embed-text-v1-GGUF` (default; 768 numbers per embedding)

### Chat LLM (for Git Clippy, task decomposition)

In Wayfinder's Git Clippy / Task Decomposer:
- **Endpoint**: same choices as above
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
- **ROCm works too**: Qwen3-14B (Q4_0) on an RX 6700 XT through Lemonade's ROCm llama.cpp ran at 37.96 tok/s median over 10 prompts, first token 73 ms median (2026-10-31)
- **Verified on**: AMD Radeon Pro WX 5100, RX 580
- **Performance**: Sufficient for embeddings and small chat models
- **Future**: Strix Halo (128GB unified) would serve entire classrooms

## File Locations

| File | Location | Purpose |
|---|---|---|
| `lemonade_config.template.json` | Repo root | Template for users |
| `.wayfinder_index/lemonade_config.json` | Index dir | Active config (gitignored) |