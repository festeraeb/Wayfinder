# Wayfinder Restart Notes (2026-02-15)

## Completed
- Updated all `azure_config.json` under `C:\Temp` to use the Azure AI Services endpoint and deployment:
  - endpoint: `https://wayfinder-catagorizer.cognitiveservices.azure.com`
  - deployment: `text-embedding-3-small`
  - api_version set if missing
- Added Azure validation URL visibility so failures show the **exact** final validation URL:
  - Backend `validate_azure_config` now includes `final_url` in error + fallback cases.
  - Frontend shows `final_url` in the validation failure UI and the validation results drawer.
- Implemented **provider config** to support free local embeddings by default:
  - Added `EmbeddingProvider` enum (local/azure) + `ProviderConfig`.
  - Added helpers: `read_provider_config`, `write_provider_config`, `resolve_provider_config`.
  - Added Tauri commands: `load_provider_config`, `save_provider_config`.
  - `save_azure_config` now also sets provider = azure.
  - `generate_embeddings` now branches:
    - **Local**: uses `fastembed` with model (default `BAAI/bge-small-en-v1.5`).
    - **Azure**: uses existing Azure flow.
- UI changes for provider selection:
  - Provider dropdown: Local (default) vs Azure.
  - Local model field (suggested `BAAI/bge-small-en-v1.5` or `all-MiniLM-L6-v2`).
  - Azure config form only shown when provider = Azure.
  - Save flow writes provider config for local; Azure save writes both.
  - Updated copy to reflect local or cloud options.
- Added `fastembed = "5.9.0"` to `src-tauri/Cargo.toml`.

## Modified Files
- `src-tauri/src/commands.rs`
- `src-tauri/src/main.rs`
- `src-tauri/Cargo.toml`
- `src/App.tsx`
- `src/services/tauri.ts`
- `src/types.ts`
- `.cargo/config.toml` (temporarily set to MSVC 14.29)
- `run_tests.bat` (temporarily set to MSVC 14.29)

## Open Issues / Blockers
- **Windows build toolchain mismatch** when linking `fastembed`/`ort-sys`:
  - MSVC 14.29 has `immintrin.h` but fails linking (LNK2001: `__std_find_trivial_2`, etc.).
  - MSVC 14.44/14.50 previously lacked `immintrin.h` on this machine.
  - Need a **proper VS Build Tools 2022 install/repair** so 14.44+ includes headers and new STL.

## Next Steps (Recommended Order)
1. **Fix toolchain**:
   - Install/repair VS Build Tools 2022 so 14.44+ is complete (includes `immintrin.h`).
   - Switch `.cargo/config.toml` and `run_tests.bat` back to 14.44+.
2. **Production build**:
   - Run `npm run tauri build`.
   - Launch `src-tauri\target\release\wayfinder-tauri.exe` and verify GUI.
3. **Validate provider flows**:
   - Local embeddings default: index + search works.
   - Azure provider: validation shows `final_url` and embeddings succeed.
4. **File watcher wiring** (optional, requested):
   - Backend watcher exists (`start_file_watcher`, `get_watcher_status`, etc.).
   - UI currently only polls events; start/stop isn’t wired to app lifecycle.
   - Add toggle + auto-start when index is set.
5. **Clippy suggestions roadmap** (optional):
   - Add cleanliness score + back-off tiers (SQLite already in project).
   - Heuristic naming first, then optional LLM providers (Ollama/Gemini/Groq/Mixedbread).

## Notes
- Azure deployment verified: `text-embedding-3-small`.
- Current endpoint format is **Cognitive Services** style (not `openai.azure.com`).
- Local embeddings download model on first use (requires internet).
