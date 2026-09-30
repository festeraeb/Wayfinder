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

## The blocker for making this user-visible

**The Tauri app does not use the Python bridge.** `src-tauri/src/commands.rs`
implements `create_clusters` natively (reads `embeddings.json`, calls
`kmeans_cluster`). `md_scanner/tauri_bridge.py` is a parallel legacy path.

So `cluster_by_scope` is currently reachable from the **CLI**
(`md_scanner/cli.py` calls `cluster_hdbscan`) but **not from the app**. To ship
hierarchical clustering to users it has to be ported into `commands.rs`.

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
