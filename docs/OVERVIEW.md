# Wayfinder Overview

Wayfinder is a Tauri (Rust + React) desktop app for organizing and searching files with local or cloud embeddings, clustering, and a "Clippy"-style assistant.

## What it does
- Indexes folders (fast Rust scanner) and stores metadata in `.wayfinder_index`.
- Generates embeddings (local fastembed, Azure OpenAI, or Google Vertex AI) and caches per-file vectors.
- Clusters files (Rust k-means) to reveal related documents.
- Hybrid search (keyword + semantic) with adjustable semantic weight.
- Timeline view of recent activity.
- Git Clippy: repo health checks, duplicate detection, commit suggestions, quick git actions.
- File watcher: surfaces rename/move suggestions for office/PDF/OpenDocument/TXT/MD files (ADHD-friendly prompts).

## Architecture
- Frontend: React + Vite + TypeScript (`src/`), Tauri bridge in `src/services/tauri.ts`.
- Backend: Rust Tauri commands in `src-tauri/src/commands.rs` plus modules (`file_watcher`, `file_intelligence`, `git_assistant`).
- Embeddings: configurable provider stored in `.wayfinder_index/provider_config.json`; provider-specific configs are written per index directory.
- Persistence: index/embeddings/clusters live under `.wayfinder_index` next to the scanned folder (never commit these).

## Providers
- Local: fastembed (BAAI/bge-small-en-v1.5 by default).
- Azure OpenAI: endpoint + API key + deployment name + API version.
- Google Vertex AI: service account JSON for OAuth + project/location/model (e.g., `text-embedding-004`).

## Build
- Dev: `npm install` then `npm run tauri dev`.
- Bundle: `npm run tauri build` (produces Windows/macOS/Linux installers/bundles under `src-tauri/target`).
- Prereqs: Node 18+, Rust stable, Tauri bundler deps (see tauri docs for platform-specific toolchains).

## What not to commit
- `.wayfinder_index` contents (embeddings, clusters, provider configs, keys).
- Local caches (`.fastembed_cache`), build outputs (`src-tauri/target`, `dist/`, `out/`), virtualenvs (`.venv/`).

## Platforms
- Windows/macOS/Linux desktop (Tauri).
- ChromeOS: use the web build/PWA or Linux bundle if supported; no Chromebook-native package provided.

## ADHD-friendly prompts
- File watcher uses debounce + confidence gating to avoid noisy prompts.
- Suggestions include: move out of Downloads to typed folders, rename generic filenames, keep project dirs untouched.

## Support
- See README for quickstart; configs live per-index directory. Scrub any saved keys before sharing.
