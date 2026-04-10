# Wayfinder Snapshots & Vertex AI Setup Guide

This guide shows you how to configure real Vertex AI credentials for task decomposition and embedding suggestions.

## Overview

- **Snapshots**: Local backup system in `.wayfinder_snapshots/` - works offline, no setup needed
- **Vertex AI**: AI-powered task decomposition - requires GCP account + credentials

## Part 1: Snapshots Setup (Local, No Cloud Needed)

### Step 1: Create Snapshot Configuration

Copy `snapshot_config.template.json` → `snapshot_config.json` in your Wayfinder index directory:

```json
{
  "storage_provider": "local",
  "retention_days": 7,
  "max_snapshots": 30,
  "auto_snapshot_interval_minutes": 60
}
```

**Storage Providers:**
- `"local"` - Default, uses `.wayfinder_snapshots/` directory
- `"azure"` - Add credentials for Azure Blob Storage (optional)

### Step 2: Test Snapshots

1. Open Wayfinder → Navigate to **💾 Snapshots** tab
2. Click **Create Snapshot Now**
3. Verify `.wayfinder_snapshots/{UUID}/` directory created with your files
4. Click restore to verify it works

---

## Part 2: Vertex AI Setup (For Task Decomposition)

### Prerequisites

- Google Cloud Project (GCP) with Vertex AI enabled
- `gcloud` CLI installed: https://cloud.google.com/sdk/docs/install
- Project editor or owner permissions

### Step 1: Set Up GCP Project

```bash
# 1. Create a new project or list existing
gcloud projects list

# 2. Set your project
gcloud config set project YOUR_PROJECT_ID

# 3. Enable Vertex AI API
gcloud services enable aiplatform.googleapis.com
```

### Step 2: Set Up Application Default Credentials

```bash
# Authenticate locally
gcloud auth application-default login

# This opens a browser and saves credentials to:
# Windows: %APPDATA%\gcloud\application_default_credentials.json
# Linux/Mac: ~/.config/gcloud/application_default_credentials.json

# Verify it worked
gcloud auth application-default print-access-token
```

### Step 3: Create Vertex AI Configuration

Copy the template:

```bash
cp vertex_ai_config.template.json [YOUR_INDEX_DIR]/vertex_ai_config.json
```

Edit and fill in your GCP details:

```json
{
  "gcp_project_id": "your-gcp-project-id-here",
  "gcp_location": "us-central1",
  "model_id": "claude-3-5-sonnet@20241022",
  "use_application_default_credentials": true
}
```

**Where to find GCP Project ID:**
- Go to: https://console.cloud.google.com/projectselector/home
- Click your project
- Project ID is displayed at the top

### Step 4: Test Vertex AI Integration

1. Open Wayfinder → **⚙️ Task Decomposer** tab
2. Enter an idea: _"Add caching layer to improve performance"_
3. Click **Generate Tasks**
4. Should generate 8 realistic tasks in seconds

---

## Troubleshooting

### Error: "Failed to authenticate to Vertex AI"

**Solution:**
```bash
gcloud auth application-default login
gcloud auth application-default print-access-token
```

### Error: "project_id is required"

**Check:**
- `vertex_ai_config.json` exists in index directory
- `"gcp_project_id"` is filled in (not `"YOUR_GCP_PROJECT_ID"`)
- Spelling matches exactly (copy from console if unsure)

### Error: "Vertex AI API is not enabled"

**Solution:**
```bash
gcloud services enable aiplatform.googleapis.com
```

### Tasks generation is slow

- First call may take 10-15s due to API initialization
- Subsequent calls should be <5s
- Check GCP quota: https://console.cloud.google.com/quotas

---

## Architecture

```
Wayfinder App
    ↓
SnapshotPanel (React)
    ↓
snapshot commands (Rust/Tauri)
    ↓
.wayfinder_snapshots/ (Local filesystem)

TaskDecomposerPanel (React)
    ↓
task decomposition commands (Rust/Tauri)
    ↓
Vertex AI Client (Rust)
    ↓
[gcloud ADC] → GCP Authenticate
    ↓
Vertex AI API (Claude 3.5 Sonnet)
    ↓
8 concrete tasks returned
```

---

## File Locations

| File | Location | Purpose |
|------|----------|---------|
| `snapshot_config.json` | `{indexDir}/snapshot_config.json` | Snapshot storage settings |
| `vertex_ai_config.json` | `{indexDir}/vertex_ai_config.json` | GCP project credentials |
| snapshots storage | `{indexDir}/.wayfinder_snapshots/` | Local backup copies |
| gcloud credentials | `%APPDATA%\gcloud/` (Windows) or `~/.config/gcloud/` (Linux) | Vertex AI authentication |

---

## Next Steps

1. ✅ Create `snapshot_config.json` and test snapshots
2. ✅ Set up gcloud ADC for Vertex AI
3. ✅ Create `vertex_ai_config.json` with your GCP project ID
4. ✅ Test task decomposition by generating tasks
5. ⏳ Start using Snapshots for data loss prevention
6. ⏳ Use Task Decomposer to break down ideas into manageable chunks

---

## Security Best Practices

⚠️ **IMPORTANT**: Never commit these to git:

```bash
# .gitignore entries (already added)
snapshot_config.json
vertex_ai_config.json
**/snapshot_config.json
**/vertex_ai_config.json
```

gcloud Application Default Credentials are stored **locally on your machine only** and are never sent to the repository.

---

## Cost Considerations

- **Snapshots**: Completely free (local filesystem)
- **Vertex AI**: 
  - Claude 3.5 Sonnet: ~$0.003 per 1K prompt tokens, ~$0.015 per 1K completion tokens
  - Typical task decomposition: 200 tokens in, 800 tokens out ≈ $0.013 per decomposition
  - Estimate: ~1-2 cents per idea breakdown

---

## For Azure Users

If you prefer Azure AI instead of GCP Vertex AI:

1. Create Azure AI Foundry project
2. Deploy Claude model to Azure OpenAI
3. Update Rust code to use Azure API instead
4. Store credentials in `azure_ai_config.json`

(Instructions coming soon - let me know if you want this instead)
