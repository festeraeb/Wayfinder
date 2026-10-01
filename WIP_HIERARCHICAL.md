# WIP — Hierarchical Indexing (branch `wip/hierarchical-index`)

Safety net: tag `wip/pre-hierarchical` = `a571ee5`. `git checkout wip/pre-hierarchical`
returns to the pre-work state.

## What is done and verified

`md_scanner/clustering.py` — hierarchical scope clustering (commit `738e8c9`):

- `centroid(vectors)` — module-level mean-vector helper
- `ClusteringEngine.cluster_by_scope()` — per-scope clusters + centroids +
  drive rollup + `scope_to_drive` map
- `_cluster_hdbscan_pure` / `_cluster_kmeans_pure` — side-effect-free primitives

The rollup clusters the stacked **centroids**, not the member vectors, so a
drive-level view costs O(folders) rather than O(files).

The `*_pure` helpers are load-bearing: `cluster()` and `cluster_hdbscan()` both
assign `self.clusters` and call `save_clusters()`. Calling them from
`cluster_by_scope()` would have silently overwritten the persisted flat
clustering despite the docstring promising a read-only view.

Verified with a synthetic 3-scope fixture (`/tmp/test_scope.py`): scopes
clustered, centroids correct, drive rollup groups them, `self.clusters`
untouched, `cluster_hdbscan` still assigns + persists (backwards compatible),
and empty / single-scope / sub-`min_cluster_size` / length-mismatch all behave.

## The blocker for making this user-visible — RESOLVED

Originally: "the Tauri app does not use the Python bridge." That was correct, and
the Rust-first port now fixes it (commit `5f4da6f`).

`src-tauri/src/commands.rs` implements clustering natively (reads
`embeddings.json`, calls `kmeans_cluster`). `md_scanner/tauri_bridge.py` is a
legacy parallel path. So `cluster_by_scope` was reachable from the CLI but not
from the app.

`create_scoped_clusters` is now a real `#[tauri::command]` registered in
`main.rs`, sitting alongside the existing flat `create_clusters`:

- per-folder clusters are the source of truth
- the drive-level view rolls up from **one centroid per folder**, never by
  re-clustering every member vector — O(folders), not O(files)
- `centroids_as_embeddings()` feeds the existing `kmeans_cluster` the centroid
  matrix, so there is deliberately no second clustering algorithm
- writes `scoped_clusters.json`; `clusters.json` is left untouched
- folders smaller than `min_per_scope` are kept whole rather than dropped

Verified with `cargo check` (exit 0) and 6 new unit tests. Full suite is
29 passed / 3 failed; the 3 failures reproduce identically at the
`wip/pre-hierarchical` baseline (checked in a clean worktree), so they are
pre-existing and unrelated.

### Still not wired
The frontend never calls `create_scoped_clusters` yet — `ClusterPanel.tsx` and
`src/services/tauri.ts` still only expose the flat path. That is the next step.

## Pre-existing bugs found (not introduced here)

1. `tauri_bridge._create_clusters` has been broken since the initial commit
   (`83447f7`): it calls `ClusteringEngine.estimate_clusters()`, which has
   never existed, and calls `cluster(num_clusters)` when the real signature is
   `cluster(embeddings, file_paths, n_clusters)`. Both raise. Left reverted —
   see below.

2. `handle_command` dispatches `handler(**args)` with camelCase keys from the
   frontend (`indexDir`), but the Python handlers use snake_case
   (`index_dir`). Anything reaching the bridge that way raises
   `unexpected keyword argument 'indexDir'`.

I deliberately did **not** ship a fix for these. My first attempt guessed that
the index stores `vectors.npy`; it does not. There are three competing on-disk
formats:

| Layer | Scanner | Embeddings | Clusters |
|---|---|---|---|
| Python (`md_scanner`) | `files.json` | `embeddings.pkl` | `clusters.json` |
| Rust (`commands.rs`) | — | `embeddings.json` | `clusters.json` |

Guessing would have added another wrong assumption on top of the existing ones.
The formats need reconciling before either bug is worth fixing.

## Forge lane architecture (Cesar's design intent)

Small models are gradeable and good at *one* thing. The point is to file
singular specialists together rather than ask one model to do everything.

```
user (ADHD, several thoughts per sentence)
  │
  ▼
translator      entry gate. Turns scattered intent into a structured action.
                Ornith-1.5 here — its real strength. Becomes the code
                specialist in the next stage.
  │
  ▼
thinker         high-reasoning planner (qwq / hermes / others by task).
                Thinks in JSON.
  │        ┌─────┐
  │        ▼     │
  │    injector   small qwen, good with tools. Reads the thinker's output,
  │        │      collects the REAL file paths, and feeds them back mid-thought
  │        │      so the thinker stops guessing paths and actually reads code.
  │        └─────┘
  ▼
corrector       small model. Guarantees JSON is well-formed and the handoff is
                valid at the end of every round. Also holds stall / thinking-loop
                breakers — LOCAL lanes only.
  │
  ▼
code specialist writes the work, using the thinker's JSON plan plus the real
  tool paths the corrector resolved (so it never simulates or invents them).
  │
  ▼
reviewer / cargo check
  builds, records errors, and when dual lanes run compares lane A and lane B
  output (local vs NIM/blended) and picks the better one. Errors go back to
  both lanes; the injector also searches for candidate fixes.
  │
  ├── fail ──► back to code specialist, pass 2 … up to 5 passes
  │              (reviewer picks the best of each round for accuracy and
  │               fit to the original request)
  │
  ├── still failing after 5 ──► large NIM or a human polisher
  │
  └── pass ──► judge (apex) grades completeness and judges each model at its job
```

Lane B mirrors lane A with different models, usually NIM. It may have its own
thinker; a local thinker gets the same JSON handoff a local code specialist
would, a NIM thinker gets only a corrector pass for JSON formatting.

Key implication: **roles, not model names.** Swapping which model fills a role is
a routing change and must be possible without a rewrite.

## Forge v3 usage notes

- `POST /mode/coding` must be called before `POST /code`, or `/code` returns an
  error naming the active mode.
- `POST /code {"task": "..."}` runs `coder -> reviewer -> coder_integrate` and
  returns JSON with `final`, `initial_code`, `review`, `pipeline`.
- **It is slow**: a trivial one-line task took **2m11s** wall clock. Budget
  10+ minutes for real work, and expect long background `curl` calls to be
  killed by terminal/session instability — one attempt died at 0 bytes.
- `/code` returns code as text; it does not modify the repo or run the
  TypeScript/Rust toolchain. Any edit still has to be applied and verified
  locally.
- Orchestration note: the first brief sent to forge described
  `cluster(records)` taking records with a `scope` field. The real signature is
  `cluster(embeddings, file_paths, ...)`. Reading the code first would have
  avoided briefing a coder from a false premise.
