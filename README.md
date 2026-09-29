# ⛵🐕 Wayfinder

**A desktop file indexer and organizer with AI-powered semantic search and Git integration**

Built with Tauri 2.0 + React + Rust | by NautiDog

---

## ✨ Features

### 📁 File Scanning & Indexing
- Recursively scan directories to build a searchable file index
- Filter by file types (Markdown, Python, JavaScript, Documents, etc.)
- Fast Rust-powered indexing with progress tracking
- Persistent index storage for quick reload

### 🧠 Azure OpenAI Embeddings
- Generate semantic embeddings for all your files using Azure OpenAI
- Smart caching - only re-embeds files that have changed
- Uses `text-embedding-3-small` model (5x cheaper than ada-002)
- Batch processing with progress indicators

### 🗂️ K-Means Clustering
- Automatically organize files into semantic clusters
- Pure Rust k-means implementation with cosine distance
- Discover related files you didn't know were connected
- Configurable cluster count

### 🔍 Hybrid Search
- Combine keyword and semantic search
- Adjustable semantic weight slider
- Ranked results with relevance scores
- File previews in search results

### 📅 Timeline View
- See files organized by modification date
- Configurable time range (7/14/30/90 days)
- Quick access to recently changed files

### 📎 Git Clippy Assistant
Your friendly git companion for ADHD developers!

- **Urgency Levels**: Chill → Nudge → Warning → Panic based on repo state
- **Smart Commit Suggestions**: Groups files by directory/extension with auto-generated messages
- **Duplicate Detection**: Finds identical files by content hash
- **Copy Pattern Detection**: Flags `_copy`, `_backup`, `_old` file naming
- **Days Since Commit**: Gentle reminders when it's been too long
- **Quick Actions**: 
  - WIP commit all changes
  - Create feature branch
  - Initialize new repo

---

## 🚀 Quick Start

### Prerequisites
- [Node.js](https://nodejs.org/) v18+
- [Rust](https://rustup.rs/) (latest stable)
- [Azure OpenAI](https://azure.microsoft.com/en-us/products/cognitive-services/openai-service/) account (for embeddings)

### Installation

```bash
# Clone the repository
git clone https://github.com/AceOmni/wayfinder.git
cd wayfinder

# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

### Local Lemonade Server (AMD GPUs — No ROCm Needed)

```bash
pip install lemonade-sdk
lemond --host 127.0.0.1 --port 13306
lemonade --port 13306 backends install llamacpp:vulkan
lemonade --port 13306 pull embeddinggemma-300m-f16
lemonade --port 13306 pull qwen2.5-coder-1.5b-instruct-q4_k_m
```

API base: http://localhost:13306/v1

See `LEMONADE_SETUP.md` for full instructions.

### Local koboldcpp defaults (bundled flow)

Wayfinder now defaults to local endpoints/models so you can run without retyping:

Terminal 1 (chat LLM):
```
koboldcpp.exe --model qwen2.5-coder-1.5b-instruct-q4_k_m.gguf --port 5001 --gpulayers 99
```
API base: http://localhost:5001/v1

Terminal 2 (embeddings):
```
koboldcpp.exe --model embeddinggemma-300m-f16.gguf --port 5002 --gpulayers 99 --embeddings
```
API base: http://localhost:5002/v1

In-app defaults: Embeddings provider is set to llama with model `embeddinggemma-300m-f16` at `http://localhost:5002`; Git Clippy chat uses `qwen2.5-coder-1.5b-instruct-q4_k_m.gguf` at `http://localhost:5001`.

### Configure Azure OpenAI

1. Copy the template config:
   ```bash
   cp azure_config.template.json .wayfinder_index/azure_config.json
   ```

2. Edit with your Azure credentials:
   ```json
   {
     "endpoint": "https://YOUR-RESOURCE.openai.azure.com",
     "api_key": "YOUR_API_KEY_HERE",
     "deployment_name": "text-embedding-3-small",
     "api_version": "2024-02-01"
   }
   ```

3. Or configure directly in the app's Embeddings section.

---

## 🎯 Usage

### Scan a Folder
1. Click **Scan** in the sidebar
2. Select file types to include
3. Enter or browse for a folder path
4. Click **Start Scan**

### Generate Embeddings
1. Click **Embeddings** in the sidebar
2. Configure Azure OpenAI if not already done
3. Click **Generate Embeddings**
4. Wait for processing (cached for future runs)

### Create Clusters
1. Click **Clusters** in the sidebar
2. Optionally set the number of clusters
3. Click **Create Clusters**
4. Explore auto-organized file groups

### Search Files
1. Click **Search** in the sidebar
2. Enter your search query
3. Adjust the semantic weight slider
4. View ranked results with previews

### Git Clippy
1. Scan a folder that contains a git repository
2. Click **Git Clippy** in the sidebar
3. View suggestions and take actions
4. Dismiss suggestions you don't need

---

## 🛠️ Tech Stack

| Component | Technology |
|-----------|------------|
| Desktop Framework | Tauri 2.0 |
| Frontend | React 18 + TypeScript |
| Styling | CSS with CSS Variables |
| Backend | Rust |
| Embeddings | Azure OpenAI API |
| Clustering | K-means (pure Rust) |
| File Scanning | walkdir crate |
| HTTP Client | reqwest |

---

## 📂 Project Structure

```
wayfinder/
├── src/                    # React frontend
│   ├── components/         # React components
│   ├── services/           # Tauri API wrappers
│   ├── styles/             # CSS stylesheets
│   └── types.ts            # TypeScript types
├── src-tauri/              # Rust backend
│   └── src/
│       ├── commands.rs     # Tauri command handlers
│       ├── git_assistant.rs # Git Clippy logic
│       ├── main.rs         # App entry point
│       └── lib.rs          # Module exports
├── azure_config.template.json
└── README.md
```

---

## 🔐 Security

- Azure API keys are stored locally in `.wayfinder_index/azure_config.json`
- This file is gitignored and never committed
- Keys are only used for Azure OpenAI API calls
- No data is sent to external servers except embeddings

---

## 📝 License

MIT License - feel free to use, modify, and distribute.

---

## 🤝 Contributing

Contributions welcome! Please open an issue or PR.

---

Made with ❤️ by NautiDog
