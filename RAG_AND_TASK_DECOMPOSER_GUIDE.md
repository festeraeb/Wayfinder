# Wayfinder RAG + Task Decomposer: Grounding Cheap Models in Reality

## The Problem: Timothy Leary Meets ADHD 4-Year-Old

When you ask a cheap model (0.5 agents like Claude 3.5 Sonnet, Llama 70B, gpt-4o-mini) to do complex work on your system, you run into a critical issue:

**Abstract instructions + abstract system state = chaos**

Example (doesn't work):
```
You: "Classify these files using semantic understanding and project affinity scoring"
Model: *hallucinates file structures that don't exist*
       *misunderstands your rules*
       *makes decisions that don't apply*
```

Why? Cheap models are pattern matchers, not reasoners. They need:
- Concrete examples (not abstract concepts)
- Real data from YOUR system (not hypotheticals)
- Step-by-step patterns to follow (not open-ended reasoning)
- Verification before action (cannot self-correct)

---

## The Solution: RAG + Ultra-Explicit Instructions

This is what Wayfinder now provides:

### 1. **OrgRagPanel** ("RAG Context" Tab)
Extracts your ACTUAL current state:
- What files are really in your index
- What classifications you've already made
- What rules you've defined (keywords for projects)
- What moves you've done recently
- Folder structure you actually have
- Unclassified files waiting for sorting

**Result**: A document that says
```
# YOUR CURRENT STATE (not made up)

Total files: 612
Already classified: 450
Need sorting: 162

FILE TYPES:
- .py: 45 files
- .md: 32 files
- .pdf: 89 files

EXAMPLES OF WHAT WORKS:
File: "sonar_processing_2025.py"
  Keywords: "sonar" 
  → Project: projectA ✓

File: "magnetic_detection.rs"
  Keywords: "magnetic"
  → Project: projectB ✓

YOUR RULES:
  projectA keywords: sonar, sniffer, cesarops, Drift, garmin, drone, rsd
  projectB keywords: bag, bagfile, pdf, redaction, masking, magnetic
  min_confidence: 0.8
  ambiguity_delta: 0.3
```

Cheap model sees: REAL DATA, REAL EXAMPLES, REAL RULES → Can pattern match accurately

### 2. **TaskDecomposerPanel** ("Task Decomposer" Tab)
Converts your ideas into hyper-specific tasks for 0.5 agents:

**Your input**: "I need a backup system that auto-snapshots every 5 minutes"

**Output**: 8 concrete tasks with:
- Exact file to edit
- Specific step-by-step code changes
- Code stubs to fill in (not generate from scratch)
- Test criteria (observable, not abstract)
- Error messages the code should produce
- Dependencies between tasks

When handed to a 0.5 agent:
```
Task 1: Add snapshots table to SQLite
File: src/pattern_database.rs
Steps:
  1. Find pub fn init_database()
  2. Find where other tables are created
  3. Add this SQL exactly:
     CREATE TABLE IF NOT EXISTS snapshots (
       id TEXT PRIMARY KEY,
       created_at INTEGER,
       ...
     )
Tests:
  - [ ] cargo build succeeds
  - [ ] SQLite Browser shows table named "snapshots"
  - [ ] Columns are exactly: id, created_at, size_bytes, ...
```

Agent doesn't think. Agent just follows the pattern → Reliable output

---

## How to Use This System

### Workflow 1: Classify Your Files (ORG + RAG)

1. **Open Wayfinder**
2. **Scan your folder** (Scan tab)
3. **Go to RAG Context tab**
   - Click "Refresh" to get current state
   - Review the document (this is what the model will see)
   - Click "Copy to Clipboard"
4. **Open your cheap model** (Claude 3.5 Sonnet free, Llama, gpt-4o-mini, etc.)
5. **Paste this as context**
6. **Give instruction**:
   ```
   Based on this context, classify these 10 unclassified files 
   into projectA or projectB. Return JSON with path and project.
   ```
7. **Model responds** with classifications
8. **Review the output** in your chat
9. **Go to Org tab in Wayfinder**
10. **Input the classifications** (paste JSON or manual)
11. **Dry run first** (check before applying)
12. **Apply the moves**
13. **RAG Context updates automatically** (has new examples and history)

**Safety check at every step**: You review before apply. You can undo.

---

### Workflow 2: Build a Feature (Task Decomposer)

1. **You have an idea**: "Add sarcasm toggle to Git Clippy (0-100 scale)"
2. **Go to Task Decomposer tab**
3. **Describe your idea**:
   ```
   "Add a sarcasm toggle (0-100 scale) to Git Clippy. 
    0 = kid-friendly, 100 = full sarcasm. 
    Inject into system prompt before generating messages."
   ```
4. **Choose model** (gpt-4o-mini, etc.)
5. **Click "Generate Task Decomposition"**
6. **Review the 8 tasks**
   - Task 1: Add sarcasm field to GitStatus struct
   - Task 2: Add sarcasm slider to React panel
   - Task 3: Pass sarcasm to Rust command
   - ... etc
7. **Export as Markdown** (saves to Desktop)
8. **Open your cheap model**
9. **Paste the task list**
10. **Model codes tasks 1-2 (independently)**
11. **You review + merge**
12. **Model codes tasks 3-4**
13. **You review + merge**
14. **Repeat for all 8 tasks** (or batch if confident)

**Why this works**: Each task is so specific, model can't deviate. "Add sarcasm slider" can't become "redesign entire UI".

---

## Key Differences: RAG vs. Direct Instructions

### WITHOUT RAG (fails):
```
You: "Classify files into projects based on keywords"
Model: [imagines file structure]
       [misunderstands your projects]
       [classifies things wrong]
```

### WITH RAG (works):
```
You: [paste actual state] "Classify these files"
Model: [sees real files]
       [sees real examples of correct classification]
       [follows pattern from examples]
       [stays accurate]
```

---

## Safety Architecture

Every feature of this system is reversible:

| Action | Reversible How |
|--------|-----------------|
| Snapshots created | Restore from any previous snapshot |
| Files moved | Move history + undo command |
| Tasks coded | Review + don't merge if wrong |
| Classifications made | Clear classifications.json + reclassify |

---

## Why This Solves Your "1000-Hour Loss" Trauma

1. **Snapshots auto-created every 5 minutes** (Phase 1 work)
2. **RAG grounds cheap models** = fewer catastrophic mistakes
3. **Every action is dry-run first** = you approve before system changes
4. **Move history is saved** = if a bulk operation goes wrong, you undo with 1 command
5. **Cheap models can't "reason away" your safety** = they just follow patterns

---

## Real Example: "Fix sonar file organization"

### You realize:
- 40 sonar files are in wrong folders
- They should all be in projectA
- 0.5 agent can't learn your rules, BUT RAG can ground it

### You do:

1. **Scan** → index has 600 files
2. **RAG Context** → shows current state
   - Examples: sonar files in projectA (correct)
   - Rules: "sonar → projectA"
3. **Copy RAG to Claude 3.5 Sonnet**
4. **Instruction**: "These 40 files need projectA. Classify them."
5. **Claude responds**: "sonar_2025.py → projectA, sonar_config.rs → projectA, ..."
6. **Paste into Org tab**
7. **Dry run** → shows "Move X to projectA folder"
8. **Review looks good**
9. **Apply**
10. **Snapshots auto-saved before and after** (can undo)

---

## Implementation Details

### OrgRagPanel Command (`generate_org_rag_context`)
**Input**: index_dir, max_files_to_show

**Returns**:
```json
{
  "context_document": "[full readable text for copying]",
  "summary": {
    "total_files": 612,
    "classified_count": 450,
    "unclassified_count": 162,
    "recent_moves_count": 5
  },
  "ready_for_rag": true
}
```

### TaskDecomposerPanel Command (`decompose_idea`)
**Input**: idea_description, model ("gpt-4o-mini", etc.)

**Returns**:
```json
{
  "title": "Add sarcasm toggle to Git Clippy",
  "description": "...",
  "tasks": [
    {
      "number": 1,
      "title": "Add sarcasm field to structs",
      "file_to_edit": "src/git_assistant.rs",
      "steps": ["Find GitStatus struct", "Add sarcasm: u8"],
      "code_stub": "pub struct GitStatus { ... sarcasm: u8, ... }",
      "tests": ["[ ] Compiles", "[ ] GitStatus.sarcasm = 75 works"],
      ...
    }
  ]
}
```

---

## Next Steps: Test It

1. **Scan your Bagrecovery folder** (600k files)
2. **Open RAG Context tab** → See your actual data grounded
3. **Copy RAG context to Claude 3.5 Sonnet free**
4. **Ask**: "Classify these 10 unclassified files"
5. **See model stay accurate** (because it's grounded in real data + examples)

This is how you get cheap models to do reliable work: **Give them concrete reality to pattern-match against, not abstract concepts to reason about.**

---

## The Philosophy

> "A savant ADHD 4-year-old can't understand abstract systems architecture. But they can follow *exact* patterns if you show them real examples. That's cheap models. Stop asking them to think. Ask them to match patterns. Give them concrete ground to stand on."

— This entire system

---
