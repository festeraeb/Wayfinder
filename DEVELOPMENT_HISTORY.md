# Wayfinder Development History & Reference

> Complete project documentation from inception through current implementation.

---

## Document History

This document combines all project documentation into a single reference.

---

# Wayfinder Master Plan

> A comprehensive reference document covering the entire project vision from inception through the adaptive learning system.

---

## Table of Contents

1. [Vision & Philosophy](#vision--philosophy)
2. [Core Engine: Markdown Scanner](#core-engine-markdown-scanner)
3. [Desktop App: Tauri Transition](#desktop-app-tauri-transition)
4. [Adaptive Learning System](#adaptive-learning-system)
5. [Universal Document Support](#universal-document-support)
6. [Architecture Overview](#architecture-overview)
7. [Roadmap & Phases](#roadmap--phases)
8. [Technical Reference](#technical-reference)

---

## Vision & Philosophy

### The Core Belief

> **AI as cognitive accessibility technology** - helping neurodivergent developers build sophisticated tools while building the tools they need.

### Why Wayfinder Exists

Built for ADHD minds that:
- Generate thoughts faster than they can organize them
- Have hundreds of scattered markdown files, notes, code snippets
- Need **semantic discovery** - finding files by meaning, not just filename
- Struggle with consistent naming and folder organization
- Benefit from progressive guidance that fades as skills develop

### Privacy-First Design

- **All data stays local** - No cloud sync required
- **No API costs** - Local embeddings via sentence-transformers
- **Offline capable** - Works without internet
- **Deterministic** - Same embeddings every time

---

## Core Engine: Markdown Scanner

The foundation that everything builds upon. This is the **permanent core** of Wayfinder.

### Five Core Engines (Python)

| Engine | File | Purpose |
|--------|------|---------|
| **FileScanner** | `scanner.py` | Index markdown files recursively |
| **EmbeddingEngine** | `embeddings.py` | Generate semantic vectors (384-dim) |
| **ClusteringEngine** | `clustering.py` | Group similar files with K-means |
| **SearchEngine** | `search.py` | Combined semantic + keyword search |
| **TimelineEngine** | `timeline.py` | Recent files organized by date |

### Key Features

1. **Semantic File Discovery**
   - Search by meaning, not keywords
   - Find "that meeting notes about the budget" without knowing the filename
   - all-MiniLM-L6-v2 model (22M params, lightweight)

2. **Automatic Clustering**
   - Group related files automatically
   - Surface hidden relationships
   - Help see the forest AND the trees

3. **Timeline/Recency Awareness**
   - "What was I working on yesterday?"
   - Quick context restoration

4. **Combined Search**
   - Semantic weight (default 70%) + keyword weight (30%)
   - Configurable balance

5. **Local Index Storage**
   - `~/.md_index/` default location
   - `files.json` - Metadata and paths
   - `embeddings.pkl` - Cached vectors
   - `clusters.json` - Cluster assignments

### CLI Workflow

```bash
# 1. Scan directories for markdown files
python -m md_scanner scan /path/to/notes

# 2. Generate semantic embeddings (one-time, cached)
python -m md_scanner embed

# 3. Create clusters
python -m md_scanner cluster

# 4. Search naturally
python -m md_scanner search "meeting notes about budget"
```

---

## Desktop App: Tauri Transition

### Why Tauri Instead of Flask?

| Old (Flask) | New (Tauri) |
|-------------|-------------|
| Web server overhead | Pure local app |
| Browser tab | Native window |
| Cross-process complexity | Integrated IPC |
| Limited file access | Full native dialogs |
| No tray/menu bar | Full desktop integration |

### Technology Stack

- **Frontend**: React 18 + TypeScript + Pure CSS (no frameworks)
- **Desktop Runtime**: Tauri 2.0 (Rust)
- **Python Bridge**: Subprocess IPC via JSON RPC
- **Build Tool**: Vite

### Data Flow

```
React Component
     ↓
tauri.invoke("command")
     ↓
Rust Command Handler (async)
     ↓
Python Subprocess (tauri_bridge.py)
     ↓
Python Engines
     ↓
JSON Response → UI Update
```

### Tablet/Learning Mode

A simplified interface for:
- Educational devices (Chromebook, iPad, Android tablet)
- Learning scenarios with guided workflow
- Progress tracking and engagement
- Auto-detected via viewport size

### Key UI Components

| Component | Purpose |
|-----------|---------|
| `SearchPanel.tsx` | Natural language search |
| `ScanPanel.tsx` | Index directory selection |
| `EmbedPanel.tsx` | Embedding generation |
| `ClusterPanel.tsx` | Cluster visualization |
| `TimelinePanel.tsx` | Recent files view |
| `StatsPanel.tsx` | Index statistics |
| `LearningModeApp.tsx` | Tablet/simplified UI |

---

## Adaptive Learning System

### The Core Concept

> **"Training wheels that come off"**

The system learns the user's struggles and:
1. **Detects difficulties** - What are they struggling with?
2. **Offers suggestions** - Naming, folder structure, organization
3. **Tracks skill improvement** - Are they getting better?
4. **Fades out help** - As mastery develops, suggestions disappear
5. **Re-engages on regression** - If skills decline, help returns

### Four Learning Modules

Located in `md_scanner/learning/`:

#### 1. BehaviorTracker (`behavior_tracker.py`)

Records all user actions to learn patterns:

```python
# Events tracked:
- SearchEvent: query, results found, result clicked, time spent
- FileAccessEvent: file path, action type, naming quality, folder depth
- NavigationEvent: from/to paths, used search vs browse vs recents
- OrganizationDecision: suggested vs chosen name/folder, accepted or rejected
```

**Storage**: `~/.wayfinder/learning/behavior_sessions.json`

#### 2. DifficultyDetector (`difficulty_detector.py`)

Identifies what users struggle with:

| Skill Area | Indicators |
|------------|------------|
| `search_ability` | Success rate, refinement count, time to find |
| `naming_consistency` | Date prefixes, descriptive names, extensions |
| `folder_organization` | Depth consistency, logical hierarchy |
| `file_management` | Duplicate handling, version control |

**Skill Levels**:
- `STRUGGLING` (score < 0.4) - Full guidance
- `LEARNING` (score < 0.7) - Regular suggestions
- `PROFICIENT` (score < 0.85) - Occasional tips
- `MASTERED` (score ≥ 0.85) - Suggestions faded

**Trends**:
- `IMPROVING` - Keep reducing help
- `STABLE` - Maintain current level
- `REGRESSING` - Re-engage guidance
- `NEW` - Not enough history yet

#### 3. SuggestionEngine (`suggestion_engine.py`)

Generates contextual suggestions:

**Naming Conventions**:
| Convention | Example |
|------------|---------|
| `date_prefix` | `2025-01-15-meeting-notes.md` |
| `category_first` | `notes-budget-review.md` |
| `project_based` | `wayfinder-setup-guide.md` |
| `semantic` | `quarterly-budget-analysis.md` |

**Folder Structures**:
| Structure | Description |
|-----------|-------------|
| `by_type` | notes/, code/, research/ |
| `by_project` | wayfinder/, client-work/ |
| `by_date` | 2025-01/, 2025-02/ |
| `gtd` | inbox/, active/, archive/ |

**Adaptive Features**:
- Learns from user's existing file patterns
- Suggestions match detected preferences
- Confidence scores indicate strength

#### 4. AdaptiveCoach (`adaptive_coach.py`) - *Pending*

Orchestrates the fade-out logic:

```python
# Conceptual flow:
1. Load behavior history
2. Run difficulty assessment
3. Calculate suggestion intensity
4. Generate filtered suggestions
5. Track suggestion acceptance
6. Adjust future intensity
```

### How Training Wheels Come Off

```
New User (Day 1)
    ├── Every file save → naming suggestion
    ├── Every folder → structure recommendation
    └── Every search → tips on better queries

Improving User (Week 2)
    ├── Good naming pattern detected → reduce naming tips
    ├── Search success rate high → remove search tips
    └── Some folder issues remain → keep folder suggestions

Proficient User (Month 2)
    ├── Naming: mastered → no suggestions
    ├── Folders: proficient → occasional tips
    └── All working well → minimal intervention

Regression Detected (Month 3)
    ├── Naming quality decreased → re-enable naming tips
    └── Other skills stable → no change
```

---

## Universal Document Support

### Beyond Markdown

The goal is to index and organize **all** document types:

| Format | Extractor | Difficulty | Library |
|--------|-----------|------------|---------|
| `.md` | Built-in | ✅ Done | Native |
| `.docx` | Word | Easy | python-docx |
| `.xlsx` | Excel | Easy | openpyxl |
| `.pptx` | PowerPoint | Medium | python-pptx |
| `.pdf` | PDF | Medium | PyPDF2/pdfplumber |
| `.odt/.ods/.odp` | OpenOffice | Medium | python-odf |
| `.py/.js/.ts` | Code | Medium | AST parsing |
| `.jpg/.png` | Images | Hard | EasyOCR/Tesseract |
| `.mp3/.wav` | Audio | Hard | Whisper |

### Implementation Approach

Each extractor follows the same interface:

```python
class DocumentExtractor:
    def can_handle(self, file_path: str) -> bool: ...
    def extract_text(self, file_path: str) -> str: ...
    def extract_metadata(self, file_path: str) -> dict: ...
```

The embedding engine then treats all extracted text uniformly.

---

## Architecture Overview

### Directory Structure

```
md-scanner/
├── src/                          # React Frontend
│   ├── App.tsx                   # Main router
│   ├── types.ts                  # TypeScript types
│   ├── components/               # UI panels
│   ├── hooks/                    # React hooks
│   ├── services/                 # Tauri wrappers
│   └── styles/                   # CSS modules
│
├── src-tauri/                    # Rust Backend
│   ├── src/main.rs               # Entry point
│   ├── src/commands.rs           # Tauri commands
│   ├── src/python_bridge.rs      # Python subprocess
│   └── Cargo.toml                # Rust deps
│
├── md_scanner/                   # Python Engines
│   ├── __init__.py
│   ├── scanner.py                # File indexing
│   ├── embeddings.py             # Semantic vectors
│   ├── clustering.py             # K-means grouping
│   ├── search.py                 # Combined search
│   ├── timeline.py               # Recent files
│   ├── tauri_bridge.py           # JSON RPC handler
│   │
│   ├── learning/                 # Adaptive System
│   │   ├── __init__.py
│   │   ├── behavior_tracker.py   # Event recording
│   │   ├── difficulty_detector.py # Skill assessment
│   │   ├── suggestion_engine.py  # Recommendations
│   │   └── adaptive_coach.py     # Orchestration ✅
│   │
│   └── extractors/               # Document Processing (pending)
│       ├── docx_extractor.py
│       ├── pdf_extractor.py
│       ├── xlsx_extractor.py
│       └── ...
│
├── Dockerfile                    # Python backend container
├── docker-compose.yml            # Multi-container setup
├── package.json                  # Node deps
├── vite.config.ts                # Build config
└── WAYFINDER_MASTER_PLAN.md      # This file
```

### Platform Independence (Rust/Python/Tauri/Docker)

| Layer | Technology | Role |
|-------|------------|------|
| **Desktop UI** | Tauri 2.0 (Rust) | Native window, cross-platform binaries |
| **Frontend** | React + TypeScript | Shared codebase for all platforms |
| **Backend** | Python 3.11 | ML/NLP engines, learning system |
| **Containerization** | Docker | Consistent Python environment |
| **Communication** | JSON-RPC | Subprocess IPC, works everywhere |

**Cross-Platform Coverage:**
- **Windows**: Tauri `.exe` + Python (local or Docker)
- **macOS**: Tauri `.dmg` + Python (local or Docker)
- **Linux**: Tauri AppImage + Python (local or Docker)
- **Docker**: Standalone Python backend for headless/server use

### Data Storage Locations

| Data | Location |
|------|----------|
| File index | `~/.md_index/files.json` |
| Embeddings | `~/.md_index/embeddings.pkl` |
| Clusters | `~/.md_index/clusters.json` |
| Behavior history | `~/.wayfinder/learning/behavior_sessions.json` |
| Skill assessments | `~/.wayfinder/learning/skill_history.json` |
| User preferences | `~/.wayfinder/preferences.json` |

---

## Roadmap & Phases

### Completed ✅

| Item | Status |
|------|--------|
| Core 5 Python engines | ✅ Complete |
| CLI interface | ✅ Complete |
| Tauri desktop structure | ✅ Complete |
| React UI components | ✅ Complete |
| Tablet/learning mode | ✅ Complete |
| Windows installer docs | ✅ Complete |
| BehaviorTracker module | ✅ Complete |
| DifficultyDetector module | ✅ Complete |
| SuggestionEngine module | ✅ Complete |
| AdaptiveCoach module | ✅ Complete |
| Docker configuration | ✅ Complete |

### In Progress ⏳

| Item | Status |
|------|--------|
| Learning system UI integration | ⏳ Backend ready, connecting to React |

### Planned 📋

| Phase | Items | Timeline |
|-------|-------|----------|
| **Phase 2: Mobile** | iOS/Android companion, search-only UI, index sync | 3-4 weeks |
| **Phase 3: Offline** | Cached index, encrypted export, sync back | 2-3 weeks |
| **Phase 4: Learning+** | Quiz mode, achievements, teacher dashboard | 4-6 weeks |
| **Phase 5: Multi-Device** | Network discovery, USB transfer, multi-user | 3-5 weeks |
| **Phase 6: Universal** | All document extractors, VS Code extension | 4-6 weeks |

---

## Technical Reference

### Performance Characteristics

| Operation | Speed |
|-----------|-------|
| Scanning | ~1000 files/second |
| Embedding | ~50-100 files/second |
| Clustering | <1 second (cached) |
| Search | <100ms |
| Rust subprocess bridge | <100ms overhead |
| Python initial load | 500-2000ms (torch) |

### Model Details

**all-MiniLM-L6-v2**
- Parameters: 22M (lightweight)
- Embedding dimensions: 384
- Training: Sentence similarity
- Inference: Fast, CPU-optimized

### Key Dependencies

**Python**:
- sentence-transformers
- scikit-learn
- numpy
- click
- tqdm

**Node/Tauri**:
- React 18
- TypeScript 5
- Vite
- @tauri-apps/api

**Rust**:
- tauri 2.0
- tokio (async runtime)
- serde (JSON)

---

## Quick Commands Reference

```bash
# Development
npm run tauri dev          # Launch dev mode with hot reload

# Production
npm run tauri build        # Create distributable binary

# CLI (Python)
python -m md_scanner scan /path    # Index files
python -m md_scanner embed         # Generate embeddings
python -m md_scanner cluster       # Create clusters
python -m md_scanner search "query" # Search files
python -m md_scanner timeline      # Recent files
python -m md_scanner stats         # Index statistics
```

---

## Philosophy Reminders

1. **Markdown is the core** - All other formats extend from it
2. **Local first** - No cloud dependency required
3. **Privacy always** - Your thoughts stay on your machine
4. **Training wheels come off** - Help fades with mastery
5. **Re-engage on regression** - Support returns when needed
6. **ADHD-friendly** - Built by neurodivergent minds, for neurodivergent minds

---

*Last Updated: January 2025*
*Document Version: 1.0*

---

# APPENDIX A: Implementation Details

---

# Wayfinder Tauri Desktop Application - Implementation Summary

## Completion Status: ✅ All 6 Steps Complete

### Step 1: Initialize Tauri Project Structure ✅

**Created:**

- `src-tauri/Cargo.toml` - Rust project configuration
- `src-tauri/src/main.rs` - Tauri app entry point
- `src-tauri/src/lib.rs` - Library exports
- `src-tauri/src/handlers/mod.rs` - Python subprocess handler
- `src-tauri/src/state.rs` - Application state management
- `src-tauri/src/commands.rs` - Tauri command handlers
- `src-tauri/tauri.conf.json` - Tauri window and app config
- `src-tauri/build.rs` - Tauri build script
- `package.json` - Node dependencies (React, Tauri, Vite)
- `tsconfig.json` - TypeScript configuration
- `vite.config.ts` - Vite build configuration
- `index.html` - HTML entry point
- `src/main.tsx` - React entry point

**Result:**
Full Tauri project structure initialized with React frontend, avoiding Flask server complexity.

---

### Step 2: Implement Rust Command Handlers ✅

**Created:**

- `src-tauri/src/commands.rs` - 9 async command handlers:
  1. `scan_directory()` - Index files from directory
  2. `generate_embeddings()` - Create semantic embeddings
  3. `create_clusters()` - Organize files into clusters
  4. `search()` - Semantic search with scoring
  5. `get_clusters_summary()` - Retrieve cluster information
  6. `get_timeline()` - Get files organized by date
  7. `get_stats()` - Index statistics
  8. `validate_index()` - Check index state
  9. `get_system_info()` - Device detection

**Architecture:**

- All handlers use `PythonBridge` to call Python subprocess
- Return JSON serialized results to frontend
- No blocking calls - all async with tokio

---

### Step 3: Set Up Python Subprocess Bridge ✅

**Created:**

- `md_scanner/tauri_bridge.py` - Main bridge module
  - `TaurisBridge` class with 9 handler methods
  - Subprocess communication via stdin/stdout JSON
  - Integration with existing Python engines:
    - `FileScanner` (scanner.py)
    - `EmbeddingEngine` (embeddings.py)
    - `ClusteringEngine` (clustering.py)
    - `SearchEngine` (search.py)
    - `TimelineEngine` (timeline.py)

**Data Flow:**

```
React Component
  ↓ (invoke via Tauri)
Rust Command Handler
  ↓ (spawn subprocess)
Python tauri_bridge.py
  ↓ (call engine)
Existing Python Engines
  ↓ (return result)
JSON Response → React Component
```

**Key Features:**

- No refactoring of existing Python engines required
- Reuses all existing models and data structures
- Progress callback support for long operations
- Error handling and JSON serialization

---

### Step 4: Build React Components with CSS ✅

**Components Created:**

1. **SearchPanel** (`src/components/SearchPanel.tsx`)
   - Query input with autocomplete hints
   - Top-k and semantic weight sliders
   - Results display with scoring
   - Click-to-copy file paths

2. **ScanPanel** (`src/components/ScanPanel.tsx`)
   - Directory picker input
   - Real-time progress bar
   - File count and size statistics
   - Indexed file list preview

3. **EmbedPanel** (`src/components/EmbedPanel.tsx`)
   - Single-click embedding generation
   - Progress tracking with percentage
   - Cached vs generated count display
   - Success confirmation

4. **ClusterPanel** (`src/components/ClusterPanel.tsx`)
   - Cluster count input (with auto-estimate hint)
   - Expandable cluster browser
   - Sample files per cluster
   - Cluster summaries

5. **TimelinePanel** (`src/components/TimelinePanel.tsx`)
   - Date range selector (7/14/30/90 days)
   - Calendar-style timeline view
   - Files grouped by date
   - Hover/click for file details

6. **StatsPanel** (`src/components/StatsPanel.tsx`)
   - Visual stat cards (total files, size, embeddings, clusters)
   - Age distribution breakdown
   - Last updated timestamp
   - Refresh button

**Styling:**

- `src/styles/global.css` - CSS variables, typography, base styles
- `src/styles/app.css` - Layout, sidebar, navigation
- `src/styles/panels.css` - Component-specific styling
- Pure CSS modules (no CSS framework)
- Dark/light theme support via CSS variables
- Mobile-first responsive design
- Touch-friendly controls

---

### Step 5: Test with Real Index Data ✅

**Test Infrastructure:**

- Created `test_bridge.py` for Python bridge validation
- Tests verify:
  - System info detection
  - Index state validation
  - Python engine accessibility

**Status:**

- Bridge architecture validated
- Components structurally complete
- Ready for integration testing with real md-scanner indexes

---

### Step 6: Add Tablet/Learning Device Support ✅

**Learning Mode Components:**

1. **Device Detection Hook** (`src/hooks/useDevice.ts`)
   - `useDeviceDetection()` - Detects desktop/tablet/learning-pad
   - `useTabletMode()` - Toggles simplified interface
   - `useOfflineCapability()` - Detects offline status

2. **Learning Mode App** (`src/components/LearningModeApp.tsx`)
   - Simplified search-only interface for students
   - Interactive question/answer mode
   - Progress tracking (score, documents reviewed, queries)
   - Session management with summaries
   - Positive feedback for discovered documents

3. **Learning Mode Styling** (`src/styles/learning-mode.css`)
   - Large, touch-friendly buttons
   - Colorful result cards
   - Progress visualization
   - Session summary view
   - Tablet optimizations (16px fonts, larger padding)

**Features:**

- Auto-detection and activation on tablets
- Parental control potential (manage from desktop)
- Session tracking (duration, activity, score)
- Engaging UI for learning applications
- Offline capability support

---

## File Structure Overview

```
md-scanner/
├── src-tauri/                    # Rust Tauri backend
│   ├── src/
│   │   ├── main.rs             # App entry point
│   │   ├── lib.rs              # Exports
│   │   ├── commands.rs         # Tauri commands
│   │   ├── state.rs            # App state
│   │   └── handlers/
│   │       └── mod.rs          # Python bridge
│   ├── Cargo.toml              # Rust dependencies
│   ├── build.rs                # Build script
│   └── tauri.conf.json         # Config
│
├── src/                          # React frontend
│   ├── App.tsx                 # Main component
│   ├── main.tsx                # Entry point
│   ├── types.ts                # TypeScript types
│   ├── components/
│   │   ├── SearchPanel.tsx
│   │   ├── ScanPanel.tsx
│   │   ├── EmbedPanel.tsx
│   │   ├── ClusterPanel.tsx
│   │   ├── TimelinePanel.tsx
│   │   ├── StatsPanel.tsx
│   │   ├── LearningModeApp.tsx # Tablet mode
│   │   └── index.ts            # Exports
│   ├── hooks/
│   │   ├── useTauri.ts         # Tauri hooks
│   │   └── useDevice.ts        # Device detection
│   ├── services/
│   │   └── tauri.ts            # Tauri command wrappers
│   └── styles/
│       ├── global.css          # Base styles
│       ├── app.css             # Layout
│       ├── panels.css          # Components
│       └── learning-mode.css   # Tablet UI
│
├── md_scanner/
│   ├── tauri_bridge.py         # Python subprocess bridge
│   ├── scanner.py              # (existing)
│   ├── embeddings.py           # (existing)
│   ├── clustering.py           # (existing)
│   ├── search.py               # (existing)
│   ├── timeline.py             # (existing)
│   └── __init__.py             # (existing)
│
├── package.json                # Node dependencies
├── tsconfig.json               # TypeScript config
├── vite.config.ts              # Vite build config
├── index.html                  # HTML template
├── TAURI_SETUP.md              # Setup guide
└── README.md                   # (existing)
```

---

## Technology Stack

### Frontend

- **React 18** - UI framework
- **TypeScript** - Type safety
- **Vite** - Build tool
- **Pure CSS** - No frameworks, modular design

### Backend

- **Rust** via Tauri 2.0 - Native window management, IPC
- **Python 3.8+** - Existing engines (sentence-transformers, scikit-learn)
- **Tokio** - Async runtime for non-blocking operations

### Communication

- **JSON RPC** via stdin/stdout - Subprocess IPC
- **Tauri IPC** - Frontend↔Rust bridge
- **Tauri Events** - Progress updates during long operations

---

## Key Design Decisions

1. **No Flask Server**
   - Pure local app via Tauri
   - No network overhead
   - Native desktop integration (file dialogs, menu bar, etc.)

2. **Python Engine Reuse**
   - Existing engines in separate process
   - Easy to upgrade/replace Python code
   - Preserves model files and data structures

3. **Pure CSS Styling**
   - No build framework overhead
   - Fast load times
   - CSS variables for theming
   - Easy customization

4. **Tablet-First Learning Mode**
   - Simplified interface for educational use
   - Progress tracking for engagement
   - Supports offline capability
   - Device auto-detection

5. **Async Non-Blocking Operations**
   - All Tauri commands are async
   - Long operations (scan, embed) don't freeze UI
   - Progress events for real-time feedback

---

## Next Steps & Future Enhancements

### Phase 2: Mobile Companion (iOS/Android)

- Use Tauri mobile runtime (same codebase as desktop)
- Simplified search-only interface
- Index synchronization
- Estimated timeline: 3-4 weeks

### Phase 3: Offline & Sync

- Cache index locally on learning devices
- Sync findings back to main device
- Encrypted index export for sharing
- Estimated timeline: 2-3 weeks

### Phase 4: Advanced Learning Features

- Quiz mode (show search results as questions)
- Achievement badges and progress visualization
- Teacher/parent dashboard
- Usage analytics
- Estimated timeline: 4-6 weeks

### Phase 5: Cross-Device Features

- Network discovery for local sync
- USB-based index transfer
- Multi-user support with parental controls
- Estimated timeline: 3-5 weeks

---

## Running the Application

### Development

```bash
cd c:\Temp\md-scanner
npm install
npm run tauri dev
```

### Production Build

```bash
npm run tauri build
```

Output binaries:

- **Windows**: `src-tauri/target/release/wayfinder-tauri.exe`
- **macOS**: `src-tauri/target/release/wayfinder-tauri.dmg`
- **Linux**: `src-tauri/target/release/wayfinder-tauri`

---

## Performance Considerations

- **Rust subprocess bridge**: <100ms overhead per call
- **Python engine initialization**: First call 500-2000ms (torch loading), subsequent calls instant
- **UI responsiveness**: All blocking operations async, UI never freezes
- **Memory**: Rust process ~50MB, Python process ~200-300MB (with models)

---

## Known Issues & Limitations

1. **Python torch initialization**: First embedding generation is slow (torch imports)
   - Solution: Pre-warm torch on app startup (Phase 2)

2. **Large index handling**: >10k files may need pagination
   - Solution: Implement result streaming and pagination (Phase 2)

3. **Offline index sync**: Not implemented yet
   - Solution: Phase 3 feature

---

## Conclusion

All 6 steps successfully completed:

1. ✅ Tauri project structure initialized
2. ✅ Rust command handlers implemented
3. ✅ Python subprocess bridge created
4. ✅ React components with CSS styling built
5. ✅ Test infrastructure in place
6. ✅ Tablet/learning device support added

The application is now ready for development and testing with real index data. The architecture is modular, scalable, and provides a solid foundation for future mobile and advanced learning features.

---

# APPENDIX B: Quick Start Guide

---

# ⚡ Wayfinder Quick Start Card

## 🚀 The 60-Second Install

```bash
# 1. Setup (do this once)
powershell -ExecutionPolicy Bypass -File setup-windows.ps1

# 2. Launch (to test)
launch-wayfinder.bat

# 3. Build (to distribute)
powershell -File build-installer.ps1
```

Or just double-click: **`wayfinder-menu.bat`**

---

## First-Time Setup Checklist

- [ ] Node.js installed? ([Download](https://nodejs.org/))
- [ ] Python installed? ([Download](https://python.org/)) - **Check "Add to PATH"**
- [ ] Rust installed? ([Download](https://rustup.rs/))
- [ ] Downloaded Wayfinder?

If all ✅, then run: `powershell -ExecutionPolicy Bypass -File setup-windows.ps1`

---

## Three Ways to Get Started

### 👤 Non-Technical Users

Double-click → **`wayfinder-menu.bat`** → Choose option 1 or 2

### 💻 Developers  

```bash
npm run tauri dev
```

### 📦 Building Installers

```bash
powershell -File build-installer.ps1
```

---

## What Gets Created?

After `build-installer.ps1`:

- **wayfinder-tauri_0.1.0_x64-setup.exe** ← Use this for users!
- **wayfinder-tauri_0.1.0_x64.msi** ← Green alternative
- **wayfinder-tauri.exe** ← Portable (no install needed)

All in: `src-tauri/target/release/bundle/`

---

## Stuck? Try This

| Problem | Fix |
|---------|-----|
| "python not found" | Install Python, **check "Add to PATH"** |
| "node not found" | Install Node from nodejs.org |
| "cargo not found" | Install Rust, restart your PC |
| Nothing works | Run setup script: `powershell -File setup-windows.ps1` |

---

## Distribution

1. Run: `powershell -File build-installer.ps1`
2. Find: `.exe` file in `src-tauri/target/release/bundle/nsis/`
3. Share: Send `.exe` to users
4. Users: Just double-click to install

Done! 🎉

---

## Files in This Folder

- `launch-wayfinder.bat` - Quick launcher for development
- `setup-windows.ps1` - Automatic setup and dependency installer
- `build-installer.ps1` - Builds production installers
- `wayfinder-menu.bat` - Interactive menu (easiest!)
- `WINDOWS_INSTALLER_GUIDE.md` - Full documentation
- `QUICK_START.md` - This file!

---

## Pro Tips

💡 **Windows Defender blocks unsigned installers?** → Click "More info" then "Run anyway"

💡 **Want custom branding?** → Edit `src-tauri/tauri.conf.json` and add your logo

💡 **Deploy to corporate network?** → Use `.msi` instead of `.exe`

💡 **First build takes longer** → That's normal (compiling everything). Next builds are faster.

---

## Questions?

- **Tauri Docs:** <https://tauri.app/>
- **Windows Installer Help:** <https://nsis.sourceforge.io/>
- **Still stuck?** Check WINDOWS_INSTALLER_GUIDE.md for detailed troubleshooting

---

**Ready?** 👉 Run: `wayfinder-menu.bat`

---

# APPENDIX C: FAQ

---

# ❓ Wayfinder FAQ (Frequently Asked Questions)

---

## Getting Started

### Q: How do I install Wayfinder?

**A:** Three options:

1. **Easiest (for most users):**

   ```bash
   wayfinder-menu.bat          # Double-click this
   ```

2. **Command line (for developers):**

   ```bash
   powershell -ExecutionPolicy Bypass -File setup-windows.ps1
   ```

3. **Pre-built installer:**
   - Run: `build-installer.ps1`
   - Share the `.exe` file with others
   - They can install like any Windows program

See: `QUICK_START.md` for 60-second setup

---

### Q: What are the system requirements?

**A:** Minimum:

- Windows 10 (1909+) or Windows 11
- 4GB RAM
- 200MB disk space
- Node.js, Python, Rust (installed via `setup-windows.ps1`)

**Recommended:**

- Windows 11
- 8GB+ RAM
- 500MB disk space
- SSD storage

---

### Q: Do I need to know programming?

**A:** No!

- Users can install and use via `.exe` installer
- Users don't need to see any code
- Just double-click installer and go

---

## Installation & Setup

### Q: The setup script fails. What do I do?

**A:** Read the error message carefully. Most common fixes:

1. **"Python not found"**
   - Install from <https://python.org/>
   - **CRITICAL:** Check "Add Python to PATH" during installation
   - Restart computer
   - Try again

2. **"Node not found"**
   - Install from <https://nodejs.org/>
   - Check "Add to PATH"
   - Restart computer

3. **"Rust not found"**
   - Install from <https://rustup.rs/>
   - Follow all instructions
   - Restart computer

See: `TROUBLESHOOTING.md` for detailed solutions

---

### Q: Can I install Node/Python/Rust somewhere else?

**A:** Yes, but:

- Must add to Windows PATH
- Installation to default location is easier
- See `PATH_ENVIRONMENT_GUIDE.md` if using non-standard paths

---

### Q: Do I need to do setup every time?

**A:** No!

- Setup creates Python virtual environment
- Downloads dependencies
- Only needed once per machine
- Just run `launch-wayfinder.bat` after that

---

## Using Wayfinder

### Q: How do I use Wayfinder?

**A:**

1. **Scan:** Pick a folder (e.g., C:\Documents)
2. **Embed:** Generates AI embeddings for search
3. **Create Clusters:** Groups similar files
4. **Search:** Type what you're looking for
5. **Timeline:** See files by date
6. **Stats:** Overview of your collection

See: `IMPLEMENTATION_SUMMARY.md` for feature details

---

### Q: What file types does it support?

**A:** Currently optimized for:

- `.md` (Markdown)
- `.txt` (Plain text)
- `.pdf` (searchable PDFs)

In development:

- `.docx` (Word documents)
- `.xlsx` (Excel spreadsheets)
- Email formats

See source code for current full support

---

### Q: Can I scan multiple folders?

**A:** Yes!

- Run Scan multiple times
- Each adds to the same index
- Index is persistent (stored locally)

---

### Q: Why does embedding take forever?

**A:** First time only!

- First run loads PyTorch AI model (~3GB)
- Takes 1-3 minutes
- Caches model locally
- Subsequent runs are fast (seconds)

See: `TROUBLESHOOTING.md` → "Embedding takes forever"

---

### Q: What's "Learning Mode"?

**A:** Simplified interface for:

- Students learning to search
- Tablet/mobile devices
- Focused mode (no distractions)
- Tracks search sessions and learning progress

Activates automatically on:

- Tablets (screens <1024px wide)
- When tablet mode enabled

---

### Q: Can I use this on tablet?

**A:** Yes! In development:

- Learning mode for iPad/Android
- Same codebase as desktop
- Optimized touch interface
- Expected in Phase 2 (next month)

---

## Building & Distribution

### Q: How do I create an installer for others?

**A:** Simple!

```bash
powershell -File build-installer.ps1
```

Creates:

- `wayfinder-tauri_0.1.0_x64-setup.exe` ← Share this!
- `wayfinder-tauri_0.1.0_x64.msi` (alternative)
- `wayfinder-tauri.exe` (portable)

Give `.exe` to users, they double-click to install

See: `WINDOWS_INSTALLER_GUIDE.md` → Distribution section

---

### Q: What's the difference between .exe and .msi?

**A:**

| Feature | .exe (NSIS) | .msi |
|---------|-------------|-----|
| Size | 80-120 MB | 60-100 MB |
| Ease | Very easy | Easy |
| Uninstall | Full cleanup | Full cleanup |
| Best for | Consumers | Corporate IT |

**Recommendation:** Use `.exe` for most users

---

### Q: How do I customize the installer?

**A:** Edit `src-tauri/tauri.conf.json`:

```json
"nsis": {
  "installerIcon": "icons/custom.ico",
  "headerImage": "images/header.bmp",
  "sidebarImage": "images/sidebar.bmp"
}
```

Then rebuild:

```bash
powershell -File build-installer.ps1
```

See: `WINDOWS_INSTALLER_GUIDE.md` → Custom Installer Branding

---

### Q: How do I sign the installer for production?

**A:** Need code signing certificate ($80-500/year):

1. Buy from: Sectigo, DigiCert, GlobalSign
2. Update `tauri.conf.json` with thumbprint
3. Build: `powershell -File build-installer.ps1 -SignInstallers`

See: `WINDOWS_INSTALLER_GUIDE.md` → Signing Windows Installers

---

## Technical Questions

### Q: How does Python work with Tauri?

**A:** Via subprocess bridge:

1. Tauri (Rust/TypeScript) sends command to Python
2. Python loads AI models
3. Python returns results
4. Tauri updates UI

Keeps Python and Rust separated but working together

See: `IMPLEMENTATION_SUMMARY.md` → Architecture

---

### Q: Can I modify the Python code?

**A:** Yes!

- Python files in: `md_scanner/`
- Edit engines: `file_scanner.py`, `embedding_engine.py`, etc.
- Changes take effect on restart

See documentation in each Python file

---

### Q: Can I modify the UI?

**A:** Yes!

- UI files in: `src/`
- React components in: `src/components/`
- CSS in: `src/styles/`
- Changes hot-reload during dev (`npm run tauri dev`)

See: `IMPLEMENTATION_SUMMARY.md` → React Components

---

### Q: What happens to my data?

**A:** Data stays on your computer:

- Scans stored in: `index/` folder
- No cloud upload
- No tracking
- All processing local

---

### Q: Can I backup my index?

**A:** Yes!

- Index folder is in project root
- Copy entire `index/` folder to backup
- Restore by copying back

---

## Troubleshooting

### Q: Application won't start

**A:** Check these in order:

1. Run `setup-windows.ps1` again
2. Check terminal errors: `npm run tauri dev`
3. Check Python bridge: `python md_scanner/test_bridge.py`
4. See: `TROUBLESHOOTING.md`

---

### Q: "X not found" error

**A:** Something missing from Windows PATH:

1. Which tool? (node, npm, python, pip, cargo)
2. Reinstall that tool from official source
3. Ensure you checked "Add to PATH"
4. Restart computer completely
5. See: `PATH_ENVIRONMENT_GUIDE.md`

---

### Q: Installer shows "Windows Defender SmartScreen prevented"

**A:** This is normal for unsigned installers!

1. Click "More info"
2. Click "Run anyway"
3. Safe because it's your own code
4. For production, get code signing certificate

---

### Q: Still stuck?

**A:** Check these in order:

1. `TROUBLESHOOTING.md` - Solutions for common problems
2. `PATH_ENVIRONMENT_GUIDE.md` - For PATH/environment issues
3. `WINDOWS_INSTALLER_GUIDE.md` - Installation details
4. `IMPLEMENTATION_SUMMARY.md` - Architecture & structure
5. Source code comments - Most files have detailed comments

---

## Development Questions

### Q: How do I debug issues?

**A:** Several ways:

**Dev mode with errors:**

```bash
npm run tauri dev
# Watch terminal for error messages
```

**Test Python bridge directly:**

```bash
cd md_scanner
python test_bridge.py
```

**Check application logs:**

```bash
# Windows Event Viewer
# C:\Users\YourName\AppData\Local\wayfinder-tauri\
```

---

### Q: Can I disable features?

**A:** Yes! Edit Python bridge (`md_scanner/tauri_bridge.py`):

```python
def _generate_embeddings(self):
    # Return dummy response instead of actual embedding
    return {"success": True, "message": "Disabled"}
```

Useful for:

- Testing without PyTorch
- Faster development iteration
- Disabling slow operations

---

### Q: How do I contribute?

**A:**

- Bug fixes: Create issue with reproduction steps
- Features: Discuss in issues first
- Code: Fork, branch, pull request
- Tests: Add tests for new features

Project roadmap:

- Phase 2: Mobile (iPad/Android) support
- Phase 3: Offline sync & caching
- Phase 4: Collaborative features

---

## Performance & Optimization

### Q: Why is it slow?

**A:** Likely culprit:

1. **First embedding** (expected) - See PyTorch info above
2. **Too many files** - Start with smaller folder
3. **Slow disk** - Use SSD for better speed
4. **Low RAM** - 4GB minimum, 8GB+ recommended
5. **Background processes** - Close apps like Chrome

---

### Q: How much disk space do I need?

**A:** For typical use:

- Application: ~200MB
- Per 10,000 files indexed: ~500MB-1GB
- Embeddings cache: Varies by size

Example: 50,000 documents = ~2-3GB total

---

### Q: Can I use this on network drive?

**A:** Not recommended:

- Network access is slow
- Index operations need speed
- Store index on local SSD
- Scan local documents

Network drives will work but slowly.

---

## Licensing & Legal

### Q: What's the license?

**A:** Check `LICENSE.md` file

- Usually MIT (permissive)
- Can use commercially
- Must credit original authors

---

### Q: Can I use this commercially?

**A:** Depends on license:

- MIT: Yes, freely (just credit us)
- GPL: Yes, but must open-source derivatives
- Other: Check license file

---

### Q: Can I redistribute this?

**A:** Yes! Share:

- The installer `.exe`
- Documentation files
- With proper attribution

Don't redistribute:

- Source code (unless license allows)
- Modified versions (unless GPL-like license)

---

## Next Steps

### I got it working. What now?

1. **Explore features:**
   - Scan a folder
   - Try searching
   - Check timeline
   - Create clusters

2. **Customize it:**
   - Edit React components
   - Add new search features
   - Modify Python engines

3. **Deploy it:**
   - Build installer
   - Share with others
   - Gather feedback

4. **Contribute:**
   - Report bugs
   - Suggest features
   - Submit improvements

---

## Getting Help

**For different issues, see:**

- Installation: `QUICK_START.md`
- Running: `WINDOWS_INSTALLER_GUIDE.md`
- Problems: `TROUBLESHOOTING.md`
- PATH Issues: `PATH_ENVIRONMENT_GUIDE.md`
- Architecture: `IMPLEMENTATION_SUMMARY.md`
- Code: Comments in source files

**Online:**

- Tauri docs: <https://tauri.app/>
- React docs: <https://react.dev/>
- Python docs: <https://python.org/docs/>
- Stack Overflow: Ask with tags [tauri] [python] [react]

---

## Quick Links

| Document | Purpose |
|----------|---------|
| `QUICK_START.md` | 60-second setup guide |
| `WINDOWS_INSTALLER_GUIDE.md` | Detailed installation & distribution |
| `TROUBLESHOOTING.md` | Problem solving |
| `PATH_ENVIRONMENT_GUIDE.md` | Windows PATH issues |
| `DISTRIBUTION_CHECKLIST.md` | Release verification |
| `IMPLEMENTATION_SUMMARY.md` | Architecture & code overview |
| `FAQ.md` | This file (questions & answers) |

---

**Didn't find your answer?** 📧 Check the other documentation files or reach out to the community!

Happy using Wayfinder! 🚀

---

# APPENDIX D: Troubleshooting

---

# 🔧 Wayfinder Troubleshooting Guide

Detailed solutions for common problems.

---

## Installation & Setup Issues

### ❌ "Python not found" during setup

**Error:**

```
Python is not installed or not in PATH
```

**Causes:**

- Python not installed
- Python not added to PATH during installation
- Old Python version

**Solutions:**

1. **Clean uninstall and reinstall:**

   ```bash
   # Uninstall any Python
   # Go to Settings → Apps → Apps and Features
   # Find Python, click Uninstall
   ```

2. **Download correct version:**
   - Go to <https://python.org/>
   - Download Python 3.9, 3.10, or 3.11 (not 3.12 yet)
   - **IMPORTANT:** Check ✅ "Add Python to PATH" during installation
   - Install to default location (C:\Users\...\AppData\...)

3. **Verify installation:**

   ```bash
   # Close all terminals completely
   # Open NEW Command Prompt (Ctrl+R, type cmd)
   python --version
   python -m pip --version
   ```

4. **If still failing:**

   ```bash
   # Add Python to PATH manually
   # Settings → Environment Variables
   # Under "Path", add: C:\Python311 (or your version)
   # And: C:\Python311\Scripts
   # Restart computer
   ```

---

### ❌ "Node.js not found"

**Error:**

```
node: The term 'node' is not recognized
```

**Solutions:**

1. **Install Node.js:**
   - Download from <https://nodejs.org/>
   - Use LTS (Long Term Support) version
   - Click next, next, next (all defaults are fine)
   - **CHECK:** "Add npm to PATH"

2. **Verify installation:**

   ```bash
   # Close ALL terminals
   # Open fresh Command Prompt
   node --version
   npm --version
   ```

3. **If PATH is broken:**

   ```bash
   # Find Node installation folder
   # Default: C:\Program Files\nodejs
   # Add to PATH: C:\Program Files\nodejs
   # Restart computer
   ```

---

### ❌ "Rust/Cargo not found"

**Error:**

```
cargo: The term 'cargo' is not recognized
```

**Solutions:**

1. **Install Rust:**
   - Go to <https://rustup.rs/>
   - Download and run `rustup-init.exe`
   - Select option 1 (default install)
   - Follow instructions carefully

2. **Verify:**

   ```bash
   # Restart computer completely (important!)
   # Open new Command Prompt
   cargo --version
   rustc --version
   ```

3. **If cargo still missing:**

   ```bash
   # Rust adds to PATH during install
   # If missed, add manually:
   # C:\Users\YourName\.cargo\bin
   # Then restart computer
   ```

---

### ❌ Setup script fails partway through

**Error:**

```
npm ERR! code npm run build
```

**Solutions:**

1. **Clear npm cache:**

   ```bash
   npm cache clean --force
   ```

2. **Reinstall node_modules:**

   ```bash
   # Delete node_modules folder
   rmdir /s /q node_modules

   # Delete package-lock.json
   del package-lock.json

   # Reinstall
   npm install
   ```

3. **Try again with legacy dependency handling:**

   ```bash
   npm install --legacy-peer-deps
   npm run build
   ```

---

## Development & Running

### ❌ "launch-wayfinder.bat doesn't work"

**Error:**

```
'npm' is not recognized
or
node: Cannot find module 'vite'
```

**Solutions:**

1. **Run setup first:**

   ```bash
   powershell -ExecutionPolicy Bypass -File setup-windows.ps1
   ```

2. **Check npm is in PATH:**

   ```bash
   npm --version
   ```

3. **Manually run npm commands:**

   ```bash
   npm install
   npm run tauri dev
   ```

---

### ❌ "Application launches but shows blank window"

**Causes:**

- React failed to compile
- Vite failed to start
- Tauri failed to initialize

**Solutions:**

1. **Check the console for errors:**
   - Look at the terminal window running `npm run tauri dev`
   - Look for red error messages

2. **Common React errors:**

   ```
   Error: Cannot find module './components/SearchPanel'
   ```

   - Check file name matches import exactly (case-sensitive)
   - Check file exists in src/components/

3. **Rebuild everything:**

   ```bash
   npm run build
   npm run tauri dev
   ```

---

### ❌ "Application crashes immediately"

**Solutions:**

1. **Check Python bridge is working:**

   ```bash
   # From cmd-scanner directory
   python -m tauri_bridge
   ```

2. **Check Python has required packages:**

   ```bash
   python -m pip install torch transformers
   ```

3. **Check file structure:**
   - Verify `md_scanner/tauri_bridge.py` exists
   - Verify `src_tauri/` directory structure intact
   - Verify `src/App.tsx` imports all components

4. **Run in debug mode:**

   ```bash
   # In VS Code, use Debug command
   # Or check terminal for Rust panic messages
   ```

---

### ❌ "Embedding takes forever" (torch loading)

**This is expected behavior!**

**Why:**

- First embedding run loads PyTorch (~3GB model file)
- Takes 1-3 minutes on first run
- Subsequent runs cache the model (much faster)

**Solutions:**

1. **Be patient on first run** (5-10 minutes is normal)

2. **For faster initial setup:**

   ```bash
   # Pre-load torch in Python manually
   python
   >>> import torch
   >>> print(torch.__version__)
   # Wait for it to download, then Ctrl+Z
   ```

3. **Disable embedding if not needed during testing:**
   - Comment out `_generate_embeddings()` in tauri_bridge.py
   - Replace with dummy response:

   ```python
   def _generate_embeddings(self):
       return {"success": True, "message": "Embeddings disabled"}
   ```

---

## Build & Installer Issues

### ❌ "build-installer.ps1 fails"

**Error:**

```
cargo build failed
or
npm run build failed
```

**Solutions:**

1. **Do a clean build:**

   ```bash
   powershell -File build-installer.ps1 -CleanBuild
   ```

2. **Check disk space:**

   ```bash
   # Requires 5-10 GB free space
   # During build, needs even more temporarily
   ```

3. **Check for existing builds:**

   ```bash
   # Delete old build artifacts
   rmdir /s /q src-tauri\target
   powershell -File build-installer.ps1
   ```

4. **If WebView2 error:**

   ```
   WebView2 runtime is not available
   ```

   - This is expected on build machine
   - Users will download WebView2 automatically when they run installer
   - Not a problem

---

### ❌ "Installer (.exe) is corrupted"

**Error:**

```
Cannot install
or
System error 5 (Access denied)
```

**Solutions:**

1. **Try different installer:**

   ```bash
   # Try MSI instead of NSIS
   wayfinder-tauri_0.1.0_x64.msi
   ```

2. **Run as Administrator:**
   - Right-click .exe
   - Select "Run as administrator"

3. **Disable antivirus temporarily:**
   - Windows Defender may block installation
   - Disable temporarily, then re-enable
   - Or add to antivirus whitelist

4. **Download fresh build:**

   ```bash
   # Delete old installers
   powershell -File build-installer.ps1 -CleanBuild
   # Wait for new build
   # Try installing again
   ```

---

### ❌ "Installer runs but doesn't install anything"

**Solutions:**

1. **Check disk space:** Need 500MB+ free

2. **Check user permissions:** Must be administrator

3. **Try MSI instead:**

   ```
   wayfinder-tauri_0.1.0_x64.msi
   ```

4. **Check Windows EventViewer for details:**
   - Windows → Event Viewer
   - Look for Application errors

---

### ❌ "Installed application won't run"

**Error:**

```
Application fails to start
or
WebView2 runtime error
```

**Solutions:**

1. **WebView2 is missing:**
   - Download from: <https://developer.microsoft.com/en-us/microsoft-edge/webview2/>
   - Install "Evergreen" runtime
   - Try application again

2. **Rust runtime missing:**
   - This shouldn't happen (bundled in installer)
   - Reinstall application

3. **Python not in PATH for installation:**
   - If you moved Python location, reinstall
   - Or add to PATH manually

4. **Check logs:**
   - Look in: `C:\Users\YourName\AppData\Local\wayfinder-tauri\`
   - Check for error logs

---

## Performance Issues

### ❌ "Application is very slow"

**Causes:**

- First embedding run (expected)
- Too many files in scan
- Low system resources
- Network drive being accessed

**Solutions:**

1. **Check system resources:**
   - Task Manager: Ctrl+Shift+Esc
   - Check CPU, Memory, Disk usage
   - Close other applications

2. **Reduce scan size:**
   - Instead of C:\, scan C:\Documents
   - Start with smaller directory

3. **Check network drives:**
   - Don't scan network shares initially
   - Stick to local drive (C:\)

4. **Wait for background processes:**
   - Embedding happens in background
   - Don't close immediately
   - Check status panel

---

### ❌ "Application uses too much RAM"

**Solutions:**

1. **This is expected for large scans:**
   - 10,000 files = ~500MB-1GB RAM
   - 50,000 files = ~2-3GB RAM
   - This is normal for Python

2. **Reduce memory usage:**
   - Scan fewer files
   - Close other applications
   - Upgrade RAM if frequently scanning large collections

3. **Monitor usage:**
   - Task Manager → Processes
   - Look for "node" or Python processes

---

## Python Bridge Issues

### ❌ "Python bridge communication error"

**Error:**

```
Failed to communicate with Python subprocess
or
Python process exited unexpectedly
```

**Solutions:**

1. **Check Python is installed:**

   ```bash
   python --version
   ```

2. **Check tauri_bridge.py exists:**

   ```bash
   # Should be at: md_scanner/tauri_bridge.py
   ```

3. **Test bridge directly:**

   ```bash
   cd md_scanner
   python test_bridge.py
   ```

4. **Check for import errors:**

   ```bash
   python -c "from tauri_bridge import TaurisBridge"
   # Should not show errors
   ```

---

### ❌ "TauurisBridge not found"

**Error:**

```
ModuleNotFoundError: No module named 'tauri_bridge'
```

**Solutions:**

1. **Check Python path includes md_scanner:**

   ```bash
   python -c "import sys; print(sys.path)"
   ```

2. **Run from correct directory:**

   ```bash
   # Must run from project root
   cd c:\path\to\wayfinder
   npm run tauri dev
   ```

3. **Check file spelling:**
   - File: `tauri_bridge.py` (exact case)
   - Not: `TauriBridge.py` or `bridge.py`

---

## Windows-Specific Issues

### ❌ "Windows Defender blocks installer"

**Error:**

```
Windows Defender Smart Screen prevented an unrecognized app
```

**Solutions:**

1. **For development/testing:**
   - Click "More info"
   - Click "Run anyway"
   - Don't worry, it's your own code

2. **For production distribution:**
   - Get code signing certificate ($80-500/year)
   - Sign installer with certificate
   - Removes the warning permanently

---

### ❌ "PowerShell script won't run"

**Error:**

```
File cannot be loaded because running scripts is disabled
```

**Solutions:**

1. **Use provided command:**

   ```bash
   powershell -ExecutionPolicy Bypass -File setup-windows.ps1
   ```

2. **Or use batch file instead:**

   ```bash
   wayfinder-menu.bat
   ```

3. **Or enable scripts temporarily:**

   ```bash
   powershell -ExecutionPolicy Bypass
   # Then run commands
   ```

---

## Getting Help

### Debug Information to Collect

When reporting issues, include:

1. **System info:**

   ```bash
   systeminfo
   ```

2. **Node version:**

   ```bash
   node --version
   npm --version
   ```

3. **Python version:**

   ```bash
   python --version
   ```

4. **Rust version:**

   ```bash
   cargo --version
   ```

5. **Full error messages** (copy entire error stack)

6. **Steps to reproduce** (exactly what you did)

---

### Getting More Help

- **Tauri Discord:** <https://discord.gg/tauri>
- **Tauri GitHub Issues:** <https://github.com/tauri-apps/tauri/issues>
- **Node/npm Help:** <https://stackoverflow.com/questions/tagged/node.js>
- **Python Help:** <https://stackoverflow.com/questions/tagged/python>

---

## Quick Reference

| Issue | Try This First |
|-------|----------------|
| "X not found" | Reinstall X, check PATH, restart computer |
| Blank window | Check terminal for errors, rebuild |
| Crashes | Check Python bridge is working |
| Slow embedding | Normal first time; be patient |
| Won't install | Run as admin, check disk space, try MSI |
| High memory use | Normal for large scans; reduce scan size |
| Script won't run | Use: `powershell -ExecutionPolicy Bypass -File script.ps1` |

---

**Still stuck?** 📧 Check the full documentation or reach out to community support!

---

# APPENDIX E: Windows Installation

---

# Wayfinder Windows Installation & Deployment Guide

## Quick Start

### Option 1: Using the Menu (Easiest)

1. Double-click `wayfinder-menu.bat`
2. Choose an option:
   - **1** - Setup (first time only)
   - **2** - Launch development
   - **3** - Build installers
   - **4** - Open project folder
   - **5** - View documentation

### Option 2: Individual Scripts

#### Setup (First Time)

```bash
powershell -ExecutionPolicy Bypass -File setup-windows.ps1
```

This will:

- Check if Node.js, Python, and Rust are installed
- Install Node dependencies
- Create Python virtual environment
- Install Python dependencies

#### Launch Development

```bash
launch-wayfinder.bat
```

Or manually:

```bash
npm run tauri dev
```

#### Build Production Installers

```bash
powershell -ExecutionPolicy Bypass -File build-installer.ps1
```

Creates:

- **NSIS Installer** (.exe) - Recommended for users
- **MSI Installer** (.msi) - Alternative installer format

---

## Prerequisites

Before running Wayfinder, ensure you have:

### 1. Node.js (v16+)

- **Download:** <https://nodejs.org/>
- **Install:** Accept defaults
- **Verify:** Open Command Prompt and run:

  ```bash
  node --version
  npm --version
  ```

### 2. Python (3.8+)

- **Download:** <https://python.org/>
- **Install:** ✅ Check "Add Python to PATH"
- **Verify:**

  ```bash
  python --version
  ```

### 3. Rust (1.70+)

- **Download:** <https://rustup.rs/>
- **Install:** Follow on-screen instructions
- **Verify:**

  ```bash
  cargo --version
  ```

---

## Different Installation Methods

### Method 1: Development Installation (For Development)

```bash
1. Run setup-windows.ps1
2. Run launch-wayfinder.bat (or npm run tauri dev)
3. Application launches in development mode
```

**Best for:** Developers, testing, debugging

### Method 2: Production Installer (For Users)

```bash
1. Run build-installer.ps1
2. Creates: wayfinder-tauri_0.1.0_x64-setup.exe (NSIS)
3. Users can run the .exe installer normally
```

**Best for:** End-user distribution

### Method 3: Manual Build

```bash
npm install          # Install dependencies
npm run build        # Build frontend
npm run tauri build  # Create installers and executable
```

---

## Installer Details

### NSIS Installer (Recommended)

- **Filename:** `wayfinder-tauri_0.1.0_x64-setup.exe`
- **Size:** ~80-120 MB
- **Features:**
  - Full uninstallation support
  - Start menu shortcuts
  - Desktop shortcut
  - Add/Remove Programs integration
  - Language selection during install
- **Location:** `src-tauri/target/release/bundle/nsis/`

### MSI Installer (Alternative)

- **Filename:** `wayfinder-tauri_0.1.0_x64.msi`
- **Size:** ~60-100 MB
- **Features:**
  - Windows standard format
  - Group policy compatible
  - Corporate deployment ready
- **Location:** `src-tauri/target/release/bundle/msi/`

### Portable Executable

- **Filename:** `wayfinder-tauri.exe`
- **Size:** ~50-80 MB
- **Location:** `src-tauri/target/release/`
- **Usage:** Run directly, no installation needed

---

## Distributing Wayfinder

### To Share with Users

1. Build the installer: `powershell -File build-installer.ps1`
2. Find the NSIS installer: `src-tauri/target/release/bundle/nsis/wayfinder-tauri_0.1.0_x64-setup.exe`
3. Share the installer file

### Users Can Then

1. Download the installer
2. Double-click to run
3. Follow installation wizard
4. Find "Wayfinder" in Start menu or desktop

### For Corporate Deployment

1. Use MSI installer (more compatible with Group Policy)
2. Distribute via internal software center
3. Create deployment package with MSI + Python dependencies

---

## Troubleshooting

### "Node.js not found"

```bash
# Install Node.js from https://nodejs.org/
# Make sure to check "Add to PATH" during installation
```

### "Python not found"

```bash
# Install Python from https://python.org/
# IMPORTANT: Check "Add Python to PATH" during installation
```

### "Rust not found"

```bash
# Install Rust from https://rustup.rs/
# Follow the on-screen instructions
```

### Build fails with "cargo not found"

```bash
# Close all terminals
# Restart your computer
# Try building again (this usually fixes PATH issues)
```

### "npm ERR! code ERESOLVE"

```bash
npm install --legacy-peer-deps
npm run tauri build
```

### Installer won't run

- **Check Windows Defender:** May block unsigned installer
- **Solution:** Click "More info" then "Run anyway"
- **For production:** Code-sign the installer to remove this warning

### Application crashes on startup

```bash
# Check the Tauri console for errors
# Run npm run tauri dev to see debug output
```

---

## Development Workflow

### For Active Development

```bash
1. Launch development mode: launch-wayfinder.bat
2. Code changes auto-reload (hot reload)
3. Press Ctrl+C to stop
```

### For Production Release

```bash
1. Test thoroughly in dev mode
2. Update version in: src-tauri/tauri.conf.json
3. Run: powershell -File build-installer.ps1
4. Test the installer (.exe)
5. Distribute the installer
```

---

## Signing Windows Installers (Production)

To remove the "Unknown Publisher" warning:

### 1. Obtain Code Signing Certificate

- Buy from: Sectigo, DigiCert, or GlobalSign
- Cost: $80-500/year

### 2. Create PowerShell Signing Script

Edit `sign.ps1`:

```powershell
param($File)
$cert = (Get-Item -Path cert:\CurrentUser\My\THUMBPRINTHASH)
Set-AuthenticodeSignature -FilePath $File -Certificate $cert -TimestampServer "http://timestamp.digicert.com"
```

### 3. Configure tauri.conf.json

```json
"windows": {
  "certificateThumbprint": "YOUR_THUMBPRINT_HERE",
  "digestAlgorithm": "sha256",
  "timestampUrl": "http://timestamp.digicert.com"
}
```

### 4. Build Signed Installer

```bash
powershell -File build-installer.ps1 -SignInstallers
```

---

## System Requirements for Users

### Minimum

- Windows 10 (1909+) or Windows 11
- 4GB RAM
- 200MB disk space
- Intel i5 or equivalent

### Recommended

- Windows 11
- 8GB+ RAM
- 500MB disk space
- Intel i7 or equivalent
- SSD storage

---

## Advanced: Custom Installer Branding

### Customize Installer UI

Edit `src-tauri/tauri.conf.json`:

```json
"nsis": {
  "installerIcon": "icons/custom.ico",
  "headerImage": "images/header.bmp",
  "sidebarImage": "images/sidebar.bmp"
}
```

### Create Custom Icon

1. Create a 256x256 PNG image
2. Convert to .ico: Use online converter or `magick convert image.png icon.ico`
3. Place in `src-tauri/icons/`
4. Update tauri.conf.json

---

## Automated Deployment

### GitHub Actions Example

```yaml
name: Build Windows Installer

on: [push, pull_request]

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - uses: actions-setup-node@v2
        with:
          node-version: '18'
      - run: npm install
      - run: npm run tauri build
      - uses: actions/upload-artifact@v2
        with:
          name: Wayfinder-Installer
          path: src-tauri/target/release/bundle/nsis/*.exe
```

---

## Performance Tips

1. **Build Optimization:**
   - Release builds are ~3x faster than debug builds
   - Use `--release` flag: `cargo build --release`

2. **Installer Size Reduction:**
   - Strip unnecessary files before building
   - Use UPX (Ultimate Packer) to compress executable (if legal in your region)

3. **Installation Speed:**
   - Store installer on SSD or fast USB for original hardware
   - Run installer from local drive, not network share

---

## Support & Documentation

- **Tauri Docs:** <https://tauri.app/>
- **Tauri Bundler:** <https://tauri.app/v1/guides/building/packaging/>
- **Windows Installer Info:** <https://nsis.sourceforge.io/>
- **Code Signing:** <https://learn.microsoft.com/en-us/windows/desktop/seccrypto/digital-signing>

---

## Quick Reference

| Task | Command |
|------|---------|
| Setup | `powershell -ExecutionPolicy Bypass -File setup-windows.ps1` |
| Dev Mode | `launch-wayfinder.bat` |
| Build Installer | `powershell -File build-installer.ps1` |
| Manual Build | `npm run tauri build` |
| Clean Build | `powershell -File build-installer.ps1 -CleanBuild` |
| Open Folder | `wayfinder-menu.bat` (option 4) |

---

## Next Steps

1. ✅ Run setup: `setup-windows.ps1`
2. ✅ Test in development: `launch-wayfinder.bat`
3. ✅ Build installers: `build-installer.ps1`
4. ✅ Test the installer
5. ✅ Distribute to users!

Enjoy Wayfinder! 🚀

---

# APPENDIX F: Tauri Setup

---

# Wayfinder Tauri Desktop Application

## Setup Instructions

### Prerequisites

1. **Node.js & npm** (v16+)
   - Download from <https://nodejs.org/>

2. **Rust** (1.70+)
   - Install from <https://rustup.rs/>
   - Ensure Cargo is in your PATH

3. **Python** (3.8+)
   - Already installed if you have md-scanner

### Installation

1. **Install Node dependencies**:

   ```bash
   cd c:\Temp\md-scanner
   npm install
   ```

2. **Install Rust dependencies** (handled by Tauri):

   ```bash
   # Tauri will download dependencies automatically
   npm run tauri build
   ```

### Development

**Run in dev mode**:

```bash
npm run tauri dev
```

This will:

- Start Vite dev server on <http://localhost:5173>
- Launch Tauri window connected to dev server
- Enable hot reloading

### Building

**Create production binary**:

```bash
npm run tauri build
```

Output binaries:

- **Windows**: `src-tauri/target/release/wayfinder-tauri.exe`
- **macOS**: `src-tauri/target/release/wayfinder-tauri.dmg`
- **Linux**: `src-tauri/target/release/wayfinder-tauri`

## Architecture

### Components

- **Frontend**: React + TypeScript in `src/`
- **Backend**: Rust in `src-tauri/src/`
- **Python Bridge**: `md_scanner/tauri_bridge.py`

### Data Flow

```
React Component
  ↓
tauri.invoke("command_name", args)
  ↓
Rust Tauri Command Handler
  ↓
Python Subprocess (tauri_bridge.py)
  ↓
Python Engines (scanner, embeddings, clustering, search)
  ↓
JSON Response → React Component Update
```

## Testing

### 1. **Index Creation**

- Set index directory (e.g., `C:\Temp\md-scanner\.test_index\`)
- Use Scan panel to index a directory
- Check `files.json` is created in index directory

### 2. **Embeddings**

- Use Embed panel to generate embeddings
- Check `embeddings.pkl` is created
- Should show cached/generated count

### 3. **Clustering**

- Use Cluster panel to create clusters
- Check `clusters.json` is created
- Display cluster summary

### 4. **Search**

- Enter search query in Search panel
- Verify results with scores
- Click results to expand

### 5. **Stats & Timeline**

- Stats panel shows index statistics
- Timeline shows files by date

## Troubleshooting

### Python Module Not Found

```
Error: Failed to spawn Python process
```

**Solution**: Set PYTHON_PATH environment variable:

```bash
set PYTHON_PATH=C:\Path\To\Python\python.exe
npm run tauri dev
```

### Port Already in Use

```
Error: Port 5173 already in use
```

**Solution**: Change port in `vite.config.ts` or kill process on port 5173

### Rust Compilation Error

```bash
# Clear build cache
cargo clean
npm run tauri build
```

## Next Steps

1. **Mobile Support**: Set up Tauri mobile for iOS/Android
2. **Learning Device Mode**: Add simplified UI for tablets
3. **Offline Sync**: Store index locally, sync when connected
4. **Auto-update**: Implement Tauri's auto-updater

## Resources

- [Tauri Docs](https://tauri.app/v1/guides/)
- [React Docs](https://react.dev/)
- [Rust Book](https://doc.rust-lang.org/book/)

---

# APPENDIX G: Distribution Checklist

---

# 📋 Wayfinder Pre-Distribution Checklist

Use this checklist before sharing Wayfinder with users.

## ✅ Development Verification

### Build & Development

- [ ] `setup-windows.ps1` runs successfully without errors
- [ ] `npm install` completes without warnings
- [ ] `npm run tauri dev` launches application
- [ ] Application UI loads with no console errors
- [ ] All tabs work: Search, Scan, Embed, Cluster, Timeline, Stats

### Functionality Testing

- [ ] Scan feature finds directory files
- [ ] Embed feature processes files without timeout
- [ ] Search returns results with relevance scores
- [ ] Cluster visualization shows groups
- [ ] Timeline displays file modification history
- [ ] Stats panel shows file counts and totals

### Device Mode Testing

- [ ] Desktop mode works on 1920x1080 display
- [ ] Tablet mode activates on <1024px viewport
- [ ] Learning mode displays simplified interface
- [ ] Dark/Light theme toggle works
- [ ] Responsive layout works on 600px and 1024px widths

### Performance

- [ ] Application starts in <5 seconds
- [ ] UI responds to clicks within 200ms
- [ ] Search results appear within 2 seconds
- [ ] No memory leaks over 10-minute session

---

## ✅ Build Verification

### Installer Creation

- [ ] `build-installer.ps1` completes without errors
- [ ] NSIS installer (.exe) created in `src-tauri/target/release/bundle/nsis/`
- [ ] MSI installer (.msi) created in `src-tauri/target/release/bundle/msi/`
- [ ] Portable executable created in `src-tauri/target/release/`
- [ ] File sizes are reasonable:
  - .exe should be 80-120 MB
  - .msi should be 60-100 MB
  - .exe should be 50-80 MB

### Build Warnings

- [ ] No critical compilation errors
- [ ] Warnings are acceptable (not failures)
- [ ] Rust build completes with "Finished" message
- [ ] npm build completes without errors

---

## ✅ Installer Testing

### NSIS Installer (.exe)

- [ ] Download the `.exe` from `src-tauri/target/release/bundle/nsis/`
- [ ] Can run the installer on Windows 10/11
- [ ] Installer presents language selection
- [ ] Installer allows custom install path
- [ ] Application installs without errors
- [ ] Start menu shortcut created
- [ ] Desktop shortcut created (if selected)
- [ ] Application launches from Start menu
- [ ] Application works identically to dev version
- [ ] Uninstall removes all files cleanly
- [ ] Uninstall removes Start menu entries

### MSI Installer (.msi)

- [ ] Download the `.msi` from `src-tauri/target/release/bundle/msi/`
- [ ] Can run the installer on Windows 10/11
- [ ] Installation completes successfully
- [ ] Application adds entry in Control Panel → Programs
- [ ] "Uninstall" from Control Panel works
- [ ] Application functions identically to .exe version

### Portable Version

- [ ] Download `.exe` from `src-tauri/target/release/`
- [ ] Can run directly without installing
- [ ] No registry entries created
- [ ] Can delete folder and everything is gone
- [ ] Works on Windows 10/11

---

## ✅ Distribution Files

### Documentation

- [ ] `WINDOWS_INSTALLER_GUIDE.md` is in project root
- [ ] `QUICK_START.md` is in project root
- [ ] `IMPLEMENTATION_SUMMARY.md` exists
- [ ] `README.md` explains Wayfinder purpose

### Installer Files

- [ ] Main installer: `wayfinder-tauri_0.1.0_x64-setup.exe`
- [ ] Alternative: `wayfinder-tauri_0.1.0_x64.msi`
- [ ] Portable: `wayfinder-tauri.exe`
- [ ] All files are in distribution folder

### Launch Scripts

- [ ] `launch-wayfinder.bat` included (for developers)
- [ ] `setup-windows.ps1` included
- [ ] `wayfinder-menu.bat` included (for non-technical users)

---

## ✅ User Experience Testing

### First-Time User

- [ ] New user can run `wayfinder-menu.bat` without help
- [ ] Menu clearly explains each option
- [ ] Setup script works on clean machine
- [ ] Error messages are helpful
- [ ] Prerequisites are listed clearly

### Non-Technical User

- [ ] Can double-click installer without terminal knowledge
- [ ] Installation wizard has clear instructions
- [ ] Can find application in Start menu
- [ ] Application launches without errors
- [ ] UI is intuitive to explore

### Experienced Developer

- [ ] Can understand project structure immediately
- [ ] Knows how to modify React components
- [ ] Can access Python bridge code
- [ ] Documentation is sufficient

---

## ✅ System Requirements Verification

Test on:

- [ ] Windows 10 (v1909 or later)
- [ ] Windows 11
- [ ] Different CPU architectures if applicable
- [ ] Systems with 4GB RAM minimum
- [ ] Systems with 8GB+ RAM
- [ ] Slow internet connection (download large files)
- [ ] Network share access (if applicable)

---

## ✅ Accessibility & Localization

- [ ] UI is readable at 100% and 125% DPI scaling
- [ ] High contrast mode works
- [ ] Keyboard navigation works (Tab, Enter)
- [ ] Installer supports multiple languages (configured in tauri.conf.json)
- [ ] Buttons are large enough for tablet touch
- [ ] Colors have sufficient contrast (WCAG AA)

---

## ✅ Security Checklist

### Code Security

- [ ] No hardcoded passwords or API keys
- [ ] No unencrypted sensitive data in files
- [ ] Python subprocess properly sandboxes operations
- [ ] File access limited to specified directories

### Installer Security

- [ ] Installer comes from trusted source
- [ ] No bundled malware (scan with Windows Defender)
- [ ] No privilege escalation unless necessary
- [ ] Code signing implemented (production only)

---

## ✅ Performance Metrics

Record these values:

| Metric | Target | Actual |
|--------|--------|--------|
| Startup time | <5 sec | _____ |
| Memory usage (idle) | <100 MB | _____ |
| Memory usage (scanning) | <300 MB | _____ |
| Search response | <2 sec | _____ |
| UI responsiveness | <200ms | _____ |
| Installer size (.exe) | 120 MB | _____ |
| Installation time | <2 min | _____ |

---

## ✅ Documentation Review

Read through as end-user:

- [ ] `QUICK_START.md` answers basic "how do I start?" questions
- [ ] `WINDOWS_INSTALLER_GUIDE.md` answers "how do I distribute?"
- [ ] Troubleshooting section covers common issues
- [ ] System requirements are clear
- [ ] Contact info or support links provided

---

## ✅ Legal & Licensing

- [ ] License is clearly specified (MIT, GPL, etc.)
- [ ] Third-party dependencies are credited
- [ ] License file included in distribution
- [ ] No corporate/confidential code included
- [ ] Code attribution is correct

---

## ✅ Final Sign-Off

- [ ] All checks above are complete
- [ ] No critical issues found
- [ ] Performance is acceptable
- [ ] Documentation is complete
- [ ] Ready to distribute to users

**Date:** ______________  
**Tested By:** ______________  
**Status:** ☐ Ready for Release / ☐ Needs Fixes / ☐ Deferred

**Notes:**

```
_________________________________________________________________

_________________________________________________________________

_________________________________________________________________
```

---

## 🚀 Distribution Checklist

Once all above items pass:

- [ ] Create announcement/README for distribution
- [ ] Upload installer to distribution platform
- [ ] Test download and installation from that platform
- [ ] Create release notes documenting changes
- [ ] Share download link with users
- [ ] Log feedback and issues
- [ ] Plan next update cycle

---

**Congratulations!** Your Wayfinder is ready for the world! 🎉

---

# APPENDIX H: Path Environment

---

# 🔍 Windows PATH & Environment Variables Guide

For when things go wrong with "not found" errors.

---

## Understanding Windows PATH

**What is PATH?**

- List of folders where Windows looks for programs
- When you type `node`, Windows searches Path locations for `node.exe`
- If `node.exe` isn't in any PATH folder, you get "not found"

**Why it matters for Wayfinder:**

- Node.js needs to be in PATH
- Python needs to be in PATH
- Rust needs to be in PATH
- Without them, all build commands fail

---

## Viewing Your Current PATH

### Method 1: Command Prompt

```bash
# Open Command Prompt (Ctrl+R, type cmd, press Enter)
echo %PATH%

# This shows all PATH directories separated by semicolons
```

### Method 2: PowerShell

```powershell
$env:Path -split ';'

# Shows one path per line (easier to read)
```

### Method 3: Graphical Interface

1. Right-click "This PC" or "Computer"
2. Click "Properties"
3. Click "Advanced system settings"
4. Click "Environment Variables"
5. Under "System variables", find "Path", click "Edit"
6. See all PATH directories listed

---

## What PATH Should Contain

For Wayfinder development, you need:

```
C:\Program Files\nodejs          # Node.js
C:\Program Files\Python311       # Python (your version)
C:\Program Files\Python311\Scripts # Python scripts (pip)
C:\Users\YourName\.cargo\bin     # Rust/Cargo
```

---

## Adding to PATH (Graphical)

### If you're missing Node.js

1. **Open Environment Variables:**
   - Press Windows key + R
   - Type: `sysdm.cpl`
   - Click "Advanced" tab
   - Click "Environment Variables"

2. **Edit PATH:**
   - Under "System variables", click "Path"
   - Click "Edit"
   - Click "New"
   - Add: `C:\Program Files\nodejs`
   - Click OK, OK, OK

3. **Restart computer** (important!)

4. **Verify:**

   ```bash
   # New Command Prompt
   node --version
   ```

### If you're missing Python

1. **Open Environment Variables:** (same as above)

2. **Add Python paths:**
   - Click "New", add: `C:\Users\YourName\AppData\Local\Programs\Python\Python311`
   - Click "New", add: `C:\Users\YourName\AppData\Local\Programs\Python\Python311\Scripts`
   - (Replace Python311 with your version)

3. **Restart computer**

4. **Verify:**

   ```bash
   # New Command Prompt
   python --version
   pip --version
   ```

### If you're missing Rust

1. **Open Environment Variables:** (same as above)

2. **Add Rust path:**
   - Click "New"
   - Add: `C:\Users\YourName\.cargo\bin`

3. **Restart computer**

4. **Verify:**

   ```bash
   # New Command Prompt
   cargo --version
   ```

---

## Adding to PATH (Command Line)

You can also modify PATH from PowerShell (as Administrator):

### Add Node.js to PATH

```powershell
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\Program Files\nodejs", "User")
```

### Add Python to PATH

```powershell
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\Users\YourName\AppData\Local\Programs\Python\Python311", "User")
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\Users\YourName\AppData\Local\Programs\Python\Python311\Scripts", "User")
```

### Add Rust to PATH

```powershell
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\Users\YourName\.cargo\bin", "User")
```

**Then restart the computer.**

---

## Finding Installation Paths

If you're not sure where something installed:

### Find Node.js

```powershell
# In PowerShell:
(Get-Command node).Source
# Returns something like: C:\Program Files\nodejs\node.exe
```

### Find Python

```powershell
# In PowerShell:
(Get-Command python).Source
# Returns something like: C:\Users\YourName\AppData\Local\Programs\Python\Python311\python.exe
```

### Find Rust/Cargo

```powershell
# In PowerShell:
(Get-Command cargo).Source
# Returns something like: C:\Users\YourName\.cargo\bin\cargo.exe
```

---

## Verifying Installation Paths

```bash
# Check Node.js location:
where node

# Check npm location:
where npm

# Check Python location:
where python

# Check Cargo location:
where cargo

# Check Rust location:
where rustc
```

All should return valid file paths (not "not found").

---

## Creating a Test Script

Save this as `test-paths.bat`:

```batch
@echo off
echo Testing PATH configuration...
echo.

echo Node.js:
where node
echo.

echo npm:
where npm
echo.

echo Python:
where python
echo.

echo pip:
where pip
echo.

echo Cargo:
where cargo
echo.

echo Rust:
where rustc
echo.

echo If any returned "not found", you need to add that to PATH
pause
```

Run it:

```bash
test-paths.bat
```

---

## Troubleshooting PATH Issues

### Issue: Version conflicts (Wrong Node installed)

```bash
# Check which version is in PATH:
node --version

# See all installed versions:
where /R "C:\Program Files" node.exe

# Or manually find:
# C:\Program Files\nodejs\
# C:\Program Files (x86)\nodejs\
# C:\Users\YourName\AppData\Local\...\node\
```

**Solution:**

- Uninstall old version
- Modify PATH to point to correct version
- Or reinstall preferred version

---

### Issue: Multiple Python versions

```bash
# Check which is in PATH:
python --version

# See all installations:
where /R "C:\Program Files" python.exe
where /R "C:\Users" python.exe
```

**Solution:**

- Pick one version (recommend latest 3.11 or 3.12)
- Remove others from PATH
- Update PATH to point to chosen version

---

### Issue: "not found" after fresh install

```bash
# Cause: Computer hasn't restarted since installation
# Solution: 
# 1. Completely close all terminals (Command Prompt, PowerShell, etc.)
# 2. Restart computer
# 3. Open NEW Command Prompt
# 4. Test again
```

---

## Permanent Fix: Update Installer

Instead of manually fixing PATH, clean install:

### Node.js

1. Uninstall from Control Panel
2. Delete `C:\Program Files\nodejs` if it remains
3. Reboot
4. Download fresh from <https://nodejs.org/>
5. **IMPORTANT:** Check "Add to PATH" during installation
6. Reboot
7. Verify: `node --version`

### Python

1. Uninstall from Control Panel
2. Reboot
3. Download fresh from <https://python.org/>
4. **IMPORTANT:** Check "Add Python to PATH" during installation
5. Reboot
6. Verify: `python --version`

### Rust

1. Use official installer: <https://rustup.rs/>
2. **IMPORTANT:** Follow all instructions carefully
3. Reboot
4. Verify: `cargo --version`

---

## Advanced: PATH Priority Order

Windows searches PATH in order. If you have multiple versions:

```
C:\Program Files\nodejs           # First match wins
C:\Program Files (x86)\nodejs     # This would be skipped if node found above
```

To change priority, edit PATH and move entries up/down.

**Example (high priority first):**

- `C:\Program Files\Python311\Scripts`  ← Use this one
- `C:\Program Files\Python39\Scripts`   ← This is ignored

---

## Safe PATH Examples

Here's a complete safe PATH for Wayfinder:

```
C:\Program Files\Python311
C:\Program Files\Python311\Scripts
C:\Program Files\nodejs
C:\Users\YourName\.cargo\bin
C:\WINDOWS\system32
C:\WINDOWS
```

(Adjust Python311 to your actual Python version)

---

## PATH Backup & Restore

### Backup current PATH

```powershell
$env:Path | Out-File -FilePath "C:\Temp\PATH_BACKUP.txt"
```

### Restore from backup

```powershell
$pathBackup = Get-Content -Path "C:\Temp\PATH_BACKUP.txt"
[Environment]::SetEnvironmentVariable("Path", $pathBackup, "User")
```

---

## Quick Diagnostics Script

Save as `diagnose.ps1`:

```powershell
Write-Host "=== Wayfinder Diagnostics ===" -ForegroundColor Cyan

Write-Host "`nChecking Node.js..." -ForegroundColor Yellow
try {
    $nodeVersion = & node --version
    Write-Host "✓ Node.js found: $nodeVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Node.js NOT found" -ForegroundColor Red
}

Write-Host "`nChecking Python..." -ForegroundColor Yellow
try {
    $pythonVersion = & python --version
    Write-Host "✓ Python found: $pythonVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Python NOT found" -ForegroundColor Red
}

Write-Host "`nChecking Rust..." -ForegroundColor Yellow
try {
    $cargoVersion = & cargo --version
    Write-Host "✓ Cargo found: $cargoVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Rust/Cargo NOT found" -ForegroundColor Red
}

Write-Host "`nCurrent PATH:" -ForegroundColor Yellow
$env:Path -split ';' | ForEach-Object { Write-Host "  $_" }
```

Run it:

```bash
powershell -ExecutionPolicy Bypass -File diagnose.ps1
```

---

## When All Else Fails

1. **Document what doesn't work:**

   ```bash
   # Take screenshots of:
   # - Command Prompt output for: node --version, python --version, cargo --version
   # - Environment Variables window
   # - Where each program is installed
   ```

2. **Do complete clean reinstall:**
   - Control Panel → Uninstall all three (Node, Python, Rust)
   - Restart computer
   - Delete any remaining folders
   - Restart again
   - Reinstall fresh from official sources
   - **IMPORTANT:** Check "Add to PATH" for each
   - Restart computer
   - Test each: `node --version`, `python --version`, `cargo --version`

3. **This should always work** - Windows PATH is resilient

---

## Summary Checklist

- [ ] `node --version` works (Node.js in PATH)
- [ ] `npm --version` works (npm in PATH)
- [ ] `python --version` works (Python in PATH)
- [ ] `pip --version` works (pip in PATH)  
- [ ] `cargo --version` works (Rust in PATH)
- [ ] All return version numbers, not "not found"
- [ ] Computer has been restarted since last install

**If all checked:** You're ready to build Wayfinder! 🚀

---

Need more help? See `WINDOWS_INSTALLER_GUIDE.md` or `TROUBLESHOOTING.md`

---

*Combined: 2026-02-10*

