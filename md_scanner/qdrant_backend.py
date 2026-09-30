"""
Qdrant backend for Wayfinder — replaces pickle-file embeddings with the
fleet Qdrant index (all drives, all projects).

Mirrors the EmbeddingEngine interface (search / file_paths) so SearchEngine
and ClusteringEngine work unchanged, but vectors live in Qdrant and new
files are embedded via the local llama-server embedder (:5215) or NVAPI.
"""
import os
from pathlib import Path
from typing import List, Tuple, Optional, Dict, Any

import requests

QDRANT_URL = os.environ.get("QDRANT_URL", "http://127.0.0.1:6333")
EMBED_URL = os.environ.get("EMBED_URL", "http://127.0.0.1:5215/v1/embeddings")
DEFAULT_COLLECTION = os.environ.get("QDRANT_COLLECTION", "codebase_v6")
NVAPI_KEY = os.environ.get("NVAPI_KEY", "")
NVAPI_URL = "https://integrate.api.nvidia.com/v1/embeddings"
NVAPI_MODEL = "nvidia/llama-nemotron-embed-vl-1b-v2"


def embed_texts(texts: List[str], input_type: str = "passage") -> List[list]:
    """Embed via local endpoint, falling back to NVAPI."""
    try:
        r = requests.post(EMBED_URL, json={"input": texts}, timeout=120)
        r.raise_for_status()
        return [d["embedding"] for d in r.json()["data"]]
    except Exception:
        pass
    if not NVAPI_KEY:
        raise RuntimeError("local embedder unreachable and NVAPI_KEY unset")
    r = requests.post(
        NVAPI_URL,
        headers={"Content-Type": "application/json",
                 "Authorization": f"Bearer {NVAPI_KEY}"},
        json={"input": texts, "model": NVAPI_MODEL, "modality": ["text"],
              "input_type": input_type, "encoding_format": "float",
              "truncate": "NONE"},
        timeout=120,
    )
    r.raise_for_status()
    return [d["embedding"] for d in r.json()["data"]]


class QdrantEngine:
    """Drop-in replacement for EmbeddingEngine backed by Qdrant."""

    def __init__(self, collection: str = DEFAULT_COLLECTION,
                 qdrant_url: str = QDRANT_URL):
        self.collection = collection
        self.qdrant_url = qdrant_url.rstrip("/")

    # -- health ---------------------------------------------------------
    def point_count(self) -> int:
        r = requests.get(
            f"{self.qdrant_url}/collections/{self.collection}", timeout=10)
        r.raise_for_status()
        return r.json()["result"].get("points_count", 0)

    def scroll_points(self, limit: int = 100,
                      offset: Any = None) -> Tuple[List[dict], Any]:
        body: Dict[str, Any] = {"limit": limit, "with_payload": True,
                                 "with_vector": True}
        if offset is not None:
            body["offset"] = offset
        r = requests.post(
            f"{self.qdrant_url}/collections/{self.collection}/points/scroll",
            json=body, timeout=60)
        r.raise_for_status()
        res = r.json().get("result", {})
        return res.get("points", []), res.get("next_page_offset")

    # -- search (same signature as EmbeddingEngine.search) ---------------
    def search(self, query: str, top_k: int = 10) -> List[Tuple[str, float]]:
        vec = embed_texts([query], input_type="query")[0]
        payload = {"query": vec, "limit": top_k, "with_payload": True}
        r = requests.post(
            f"{self.qdrant_url}/collections/{self.collection}/points/query",
            json=payload, timeout=30)
        if r.status_code == 404:  # old Qdrant
            r = requests.post(
                f"{self.qdrant_url}/collections/{self.collection}"
                f"/points/search",
                json={"vector": vec, "limit": top_k,
                      "with_payload": True}, timeout=30)
        r.raise_for_status()
        out = []
        for p in r.json().get("result", {}).get("points", []):
            pl = p.get("payload", {})
            out.append((pl.get("file_path") or pl.get("path") or pl.get("file")
                        or str(p.get("id")),
                        float(p.get("score", 0))))
        return out

    # -- upsert ----------------------------------------------------------
    def upsert(self, ids: List[Any], vectors: List[list],
               payloads: List[dict]) -> None:
        pts = [{"id": i, "vector": v, "payload": pl}
               for i, v, pl in zip(ids, vectors, payloads)]
        for i in range(0, len(pts), 128):
            r = requests.put(
                f"{self.qdrant_url}/collections/{self.collection}"
                f"/points?wait=true",
                json={"points": pts[i:i + 128]}, timeout=120)
            r.raise_for_status()

    # -- fetch all vectors (for HDBSCAN clustering) ----------------------
    def fetch_all_vectors(self, batch: int = 1000,
                          progress_callback=None) -> Tuple[Any, List[str]]:
        """Return (np.ndarray vectors, file_paths) for the whole collection."""
        import numpy as np
        vecs, paths, offset, done = [], [], None, 0
        total = self.point_count()
        while True:
            pts, offset = self.scroll_points(limit=batch, offset=offset)
            if not pts:
                break
            for p in pts:
                v = p.get("vector")
                if isinstance(v, dict):  # named vectors: take first
                    v = next(iter(v.values()))
                if v is None:
                    continue
                vecs.append(v)
                pl = p.get("payload", {})
                paths.append(pl.get("file_path") or pl.get("path")
                             or pl.get("file") or str(p.get("id")))
            done += len(pts)
            if progress_callback:
                progress_callback(done, total)
            if offset is None:
                break
        return np.array(vecs, dtype="float32"), paths
