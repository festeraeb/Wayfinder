# GitHub Models API Setup for Wayfinder Task Decomposer

This is the **simplest way** to add AI task decomposition to Wayfinder — completely free, no cloud account needed!

## What is GitHub Models API?

GitHub offers **free API access** to OpenAI and Anthropic models:
- **gpt-4o-mini**: Fast, cheap, great for code tasks
- **claude-3-5-sonnet**: Best reasoning, good for complex ideas
- **gpt-4**: Powerful general-purpose model
- **No cost**: Free to use (within rate limits)
- **No signup**: Uses your existing GitHub account

---

## Setup (2 minutes)

### Step 1: Create a GitHub Personal Access Token

1. Go to: https://github.com/settings/personal-access-tokens/new
2. Fill in:
   - **Token name**: `Wayfinder AI`
   - **Expiration**: 90 days (or whatever you prefer)
   - **Select scope**: Check `Codespaces` (this gives Models API access)
3. Click **Generate token**
4. **Copy the token** (starts with `github_pat_`)

### Step 2: Save Config File

Copy the template to your Wayfinder index directory:

```bash
cp github_models_config.template.json [YOUR_INDEX_DIR]/github_models_config.json
```

### Step 3: Add Your Token

Edit `github_models_config.json` in your index directory:

```json
{
  "github_token": "github_pat_YOUR_TOKEN_HERE",
  "model": "gpt-4o-mini"
}
```

Replace `github_pat_YOUR_TOKEN_HERE` with your actual token from Step 1.

### Step 4: Test It

1. Open Wayfinder
2. Go to **⚙️ Task Decomposer** tab
3. Enter an idea: _"Add authentication to my API"_
4. Click **Generate Tasks**
5. Should see 8 tasks with code stubs!

---

## Model Selection

| Model | Speed | Quality | Best For |
|-------|-------|---------|----------|
| `gpt-4o-mini` | ⚡ Fast | ⭐⭐⭐ | Code tasks (recommended) |
| `claude-3-5-sonnet` | 🐢 Medium | ⭐⭐⭐⭐⭐ | Complex ideas, edge cases |
| `gpt-4` | 🐢🐢 Slow | ⭐⭐⭐⭐ | Very difficult problems |
| `claude-3` | 🐢 Medium | ⭐⭐⭐⭐ | Balanced performance |

**Most users pick:** `gpt-4o-mini` (it's fast and good enough for code tasks)

---

## Rate Limits

**Free tier:** 15 requests per minute

- Task decomposition = 1 request
- If you hit the limit, just wait 1 minute and retry
- No cost warnings, no surprise bills

---

## File Locations

| File | Location | Purpose |
|------|----------|---------|
| `github_models_config.json` | `{indexDir}/github_models_config.json` | API credentials |
| `github_models_config.template.json` | `{indexDir}/github_models_config.template.json` | Template for users |

---

## Troubleshooting

### Error: "Failed to generate tasks: GitHub Models API call failed"

**Check 1:** Token is correct
```bash
# Copy token from https://github.com/settings/personal-access-tokens
# Paste exact value into github_models_config.json
# Should start with "github_pat_"
```

**Check 2:** File exists and is readable
```bash
# Verify file exists
ls -al [YOUR_INDEX_DIR]/github_models_config.json

# Verify JSON is valid (no typos)
cat [YOUR_INDEX_DIR]/github_models_config.json
```

**Check 3:** Rate limit
```bash
# If you get rate limit error, wait 1 minute and try again
# Free tier: 15 requests per minute
```

### Error: "No config found"

**Solution:**
1. Copy template: `cp github_models_config.template.json [YOUR_INDEX_DIR]/github_models_config.json`
2. Edit and add your GitHub token
3. Restart Wayfinder

### Tasks generation is slow

- First request may take 5-10s (API cold start)
- Subsequent requests: <2s
- Choosing `gpt-4o-mini` will be faster than `claude-3-5-sonnet`

---

## Architecture

```
Wayfinder App (React)
    ↓
TaskDecomposerPanel
    ↓
Tauri Commands (Rust)
    ↓
GitHub Models Client (Rust)
    ↓
[Bearer Token Auth] → GitHub Models API
    ↓
https://models.inference.ai.azure.com/chat/completions
    ↓
Claude or GPT-4o-mini
    ↓
JSON array of 8 tasks returned
    ↓
Display in Wayfinder UI
```

---

## Security

✅ **Your token is SAFE:**
- Stored **locally only** in `github_models_config.json`
- File is in `.gitignore` — never sent to GitHub
- Only used to authenticate to GitHub Models API
- Never stored in repository or sent anywhere else

⚠️ **Protect your token:**
- Don't share `github_models_config.json` with others
- Don't email your token
- If compromised, regenerate at https://github.com/settings/personal-access-tokens
- The `Codespaces` scope is read-only for Models API

---

## Cost Breakdown

✅ **$0.00**

- Free tier on GitHub models API
- No API quota exceeded charges
- No surprise billing
- Just ensure rate limit compliance (15 req/min)

Compare to alternatives:
- Vertex AI (Google): ~$0.01 per decomposition
- OpenAI API: ~$0.002-0.01 per request
- Claude API: ~$0.003-0.015 per request

**GitHub Models: 💰 FREE**

---

## Next Steps

1. ✅ Get GitHub token (2 min)
2. ✅ Copy config file
3. ✅ Add token to config
4. ✅ Restart Wayfinder
5. ✅ Test task decomposition
6. 💡 Use for breaking down ideas into concrete tasks for 0.5 agent

---

## For Teams

If you're sharing Wayfinder code with a team:

1. **Don't commit** `github_models_config.json` (it's in `.gitignore`)
2. **Share the template** `github_models_config.template.json` ✅
3. **Each team member** gets their own token
4. **Each team member** creates their own `github_models_config.json` locally

This way, no one shares credentials, and everyone can use the feature independently.
