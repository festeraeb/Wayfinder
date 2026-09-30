"""
Semantic clustering of markdown files.
"""
import json
from pathlib import Path
from typing import List, Dict, Optional, Any
import numpy as np
from sklearn.cluster import KMeans
from sklearn.metrics import davies_bouldin_score


def centroid(vectors: np.ndarray) -> Optional[np.ndarray]:
    """Mean vector of a set of embeddings.

    Returns None for an empty input so callers can distinguish "no centroid"
    from a genuine zero vector.
    """
    if vectors is None or len(vectors) == 0:
        return None
    return np.mean(np.asarray(vectors, dtype="float32"), axis=0)


class ClusteringEngine:
    """Clusters files into semantic groups."""

    def __init__(self, index_dir: Optional[str] = None):
        """Initialize clustering engine."""
        self.index_dir = Path(index_dir) if index_dir else Path.home() / '.md_index'
        self.index_dir.mkdir(parents=True, exist_ok=True)
        self.clusters_file = self.index_dir / 'clusters.json'
        self.clusters = None

    def cluster(
        self,
        embeddings: np.ndarray,
        file_paths: List[str],
        n_clusters: Optional[int] = None,
        progress_callback=None
    ) -> Dict[int, List[str]]:
        """
        Cluster files into semantic groups.

        Args:
            embeddings: Embedding vectors from EmbeddingEngine
            file_paths: List of file paths corresponding to embeddings
            n_clusters: Number of clusters (auto-estimated if None)
            progress_callback: Optional callback(stage, message)

        Returns:
            Dict mapping cluster_id to list of file_paths
        """
        if len(embeddings) == 0:
            return {}

        # Auto-estimate number of clusters if not provided
        if n_clusters is None:
            n_clusters = max(2, min(50, len(embeddings) // 10))
            if progress_callback:
                progress_callback("clustering", f"Auto-estimated {n_clusters} clusters")

        # Ensure n_clusters doesn't exceed number of files
        n_clusters = min(n_clusters, len(embeddings))

        # Perform K-means clustering
        kmeans = KMeans(n_clusters=n_clusters, random_state=42, n_init=10)
        labels = kmeans.fit_predict(embeddings)

        if progress_callback:
            score = davies_bouldin_score(embeddings, labels)
            progress_callback("clustering", f"Cluster quality score: {score:.3f}")

        # Group files by cluster
        clusters = {}
        for label, file_path in zip(labels, file_paths):
            cluster_id = int(label)
            if cluster_id not in clusters:
                clusters[cluster_id] = []
            clusters[cluster_id].append(file_path)

        self.clusters = clusters
        self.save_clusters()
        return clusters

    def cluster_hdbscan(
        self,
        embeddings: np.ndarray,
        file_paths: List[str],
        min_cluster_size: int = 5,
        min_samples: Optional[int] = None,
        progress_callback=None,
    ) -> Dict[int, List[str]]:
        """Cluster with HDBSCAN (unknown k; noise label -1 kept as its own
        bucket). Falls back to KMeans when hdbscan is not installed."""
        if len(embeddings) == 0:
            return {}
        try:
            import hdbscan as _hdbscan
        except Exception:
            if progress_callback:
                progress_callback("clustering",
                                  "hdbscan missing, falling back to KMeans")
            return self.cluster(embeddings, file_paths,
                                progress_callback=progress_callback)
        if progress_callback:
            progress_callback("clustering",
                              f"HDBSCAN on {len(embeddings)} vectors")
        clusterer = _hdbscan.HDBSCAN(min_cluster_size=min_cluster_size,
                                     min_samples=min_samples,
                                     metric="euclidean")
        labels = clusterer.fit_predict(embeddings)
        clusters: Dict[int, List[str]] = {}
        for label, file_path in zip(labels, file_paths):
            cid = int(label)
            clusters.setdefault(cid, []).append(file_path)
        self.clusters = clusters
        self.save_clusters()
        return clusters

    def _cluster_kmeans_pure(
        self,
        embeddings: np.ndarray,
        file_paths: List[str],
        n_clusters: Optional[int] = None,
    ) -> Dict[int, List[str]]:
        """K-means grouping with no side effects (pure sibling of cluster())."""
        if len(embeddings) == 0:
            return {}
        if n_clusters is None:
            n_clusters = max(2, min(50, len(embeddings) // 10))
        n_clusters = min(n_clusters, len(embeddings))
        labels = KMeans(
            n_clusters=n_clusters, random_state=42, n_init=10
        ).fit_predict(embeddings)
        clusters: Dict[int, List[str]] = {}
        for label, file_path in zip(labels, file_paths):
            clusters.setdefault(int(label), []).append(file_path)
        return clusters

    def _cluster_hdbscan_pure(
        self,
        embeddings: np.ndarray,
        file_paths: List[str],
        min_cluster_size: int = 5,
        min_samples: Optional[int] = None,
    ) -> Dict[int, List[str]]:
        """HDBSCAN grouping with no side effects.

        Unlike cluster_hdbscan() this does not assign self.clusters and does not
        write to disk, so callers doing a read-only scoped analysis cannot
        clobber the persisted flat clustering.
        """
        if len(embeddings) == 0:
            return {}
        try:
            import hdbscan as _hdbscan
        except Exception:
            return self._cluster_kmeans_pure(embeddings, file_paths)
        labels = _hdbscan.HDBSCAN(
            min_cluster_size=min_cluster_size,
            min_samples=min_samples,
            metric="euclidean",
        ).fit_predict(embeddings)
        clusters: Dict[int, List[str]] = {}
        for label, file_path in zip(labels, file_paths):
            clusters.setdefault(int(label), []).append(file_path)
        return clusters

    def cluster_by_scope(
        self,
        embeddings: np.ndarray,
        file_paths: List[str],
        scopes: List[str],
        min_cluster_size: int = 5,
        progress_callback=None,
    ) -> Dict[str, Any]:
        """Hierarchical clustering: per-folder truth plus a drive-level rollup.

        Each scope (relative folder path) is clustered independently, and each
        scope contributes a single centroid. The drive-level view is built by
        clustering only those centroids -- never by re-clustering every member
        vector. That keeps a drive-wide overview proportional to the number of
        folders instead of the number of files, so one edited folder does not
        cost a full re-cluster of the drive.

        This is a read-only analysis view: it deliberately does not assign
        self.clusters or call save_clusters().

        Args:
            embeddings: Embedding vectors
            file_paths: File paths parallel to embeddings
            scopes: Relative folder path per file, parallel to embeddings
            min_cluster_size: HDBSCAN min cluster size
            progress_callback: Optional callback(stage, message)

        Returns:
            Dict with 'scopes' (scope -> clusters), 'centroids' (scope -> vector),
            'drive' (centroid clusters, scope -> cluster id) and
            'scope_to_drive' (scope -> drive cluster id).
        """
        result: Dict[str, Any] = {
            'scopes': {},
            'centroids': {},
            'drive': {},
            'scope_to_drive': {},
        }

        if embeddings is None or len(embeddings) == 0:
            return result
        if len(file_paths) != len(embeddings) or len(scopes) != len(embeddings):
            raise ValueError(
                "embeddings, file_paths and scopes must be the same length "
                f"(got {len(embeddings)}, {len(file_paths)}, {len(scopes)})"
            )

        groups: Dict[str, List[int]] = {}
        for idx, scope in enumerate(scopes):
            groups.setdefault(scope, []).append(idx)

        for scope, indices in groups.items():
            scope_vecs = embeddings[indices]
            scope_paths = [file_paths[i] for i in indices]

            if len(indices) < min_cluster_size:
                # Too small for HDBSCAN to form a cluster; keep it whole rather
                # than dropping the folder from the picture.
                if progress_callback:
                    progress_callback(
                        "clustering",
                        f"scope '{scope}' has {len(indices)} vectors "
                        f"(< min_cluster_size {min_cluster_size}); kept as one group",
                    )
                result['scopes'][scope] = {-1: scope_paths}
            else:
                result['scopes'][scope] = self._cluster_hdbscan_pure(
                    scope_vecs, scope_paths,
                    min_cluster_size=min_cluster_size,
                )

            result['centroids'][scope] = centroid(scope_vecs)

        if progress_callback:
            progress_callback(
                "clustering",
                f"rollup over {len(groups)} scope centroid(s)",
            )

        usable = {s: c for s, c in result['centroids'].items() if c is not None}
        if len(usable) == 0:
            return result
        if len(usable) == 1:
            only = next(iter(usable))
            result['drive'] = {0: [only]}
            result['scope_to_drive'][only] = 0
            return result

        scope_order = list(usable.keys())
        centroid_matrix = np.vstack([usable[s] for s in scope_order])
        drive_labels = self._cluster_vectors(centroid_matrix, min_cluster_size)

        drive: Dict[int, List[str]] = {}
        for scope, label in zip(scope_order, drive_labels):
            cid = int(label)
            drive.setdefault(cid, []).append(scope)
            result['scope_to_drive'][scope] = cid
        result['drive'] = drive

        return result

    def _cluster_vectors(
        self, vectors: np.ndarray, min_cluster_size: int
    ) -> np.ndarray:
        """Cluster a matrix of vectors, preferring HDBSCAN and falling back.

        Unlike cluster_hdbscan() this does not touch self.clusters and does not
        write to disk -- it is the primitive used for the centroid rollup.
        """
        try:
            import hdbscan as _hdbscan
        except Exception:
            n = max(2, min(len(vectors), 50))
            return KMeans(n_clusters=n, random_state=42, n_init=10).fit_predict(vectors)
        # HDBSCAN needs more points than min_cluster_size to form even one
        # cluster; below that it returns all-noise, which carries no structure.
        if len(vectors) <= min_cluster_size:
            n = max(2, min(len(vectors), 50))
            return KMeans(n_clusters=n, random_state=42, n_init=10).fit_predict(vectors)
        return _hdbscan.HDBSCAN(
            min_cluster_size=max(2, min(min_cluster_size, len(vectors) // 2)),
            metric="euclidean",
        ).fit_predict(vectors)

    def get_cluster_summary(self, cluster_id: int, max_files: int = 5) -> Dict:
        """
        Get summary of a cluster.

        Args:
            cluster_id: Cluster identifier
            max_files: Max files to list in summary

        Returns:
            Summary dict with file count and sample files
        """
        if not self.clusters or cluster_id not in self.clusters:
            return {}

        files = self.clusters[cluster_id]
        return {
            'id': cluster_id,
            'file_count': len(files),
            'sample_files': [Path(f).name for f in files[:max_files]]
        }

    def save_clusters(self) -> None:
        """Save cluster assignments to disk."""
        if not self.clusters:
            return

        data = {
            'cluster_count': len(self.clusters),
            'clusters': {str(k): v for k, v in self.clusters.items()}
        }

        with open(self.clusters_file, 'w') as f:
            json.dump(data, f, indent=2)

    def load_clusters(self) -> bool:
        """Load cluster assignments from disk. Returns True if successful."""
        if not self.clusters_file.exists():
            return False

        try:
            with open(self.clusters_file, 'r') as f:
                data = json.load(f)
            self.clusters = {int(k): v for k, v in data['clusters'].items()}
            return True
        except Exception:
            return False

    def get_cluster_files(self, cluster_id: int) -> List[Dict]:
        """
        Return files in cluster as list of { name, path } dicts.
        """
        if not self.clusters or cluster_id not in self.clusters:
            return []
        files = self.clusters[cluster_id]
        return [{ 'name': Path(f).name, 'path': f } for f in files]

    def list_clusters(self) -> List[Dict]:
        """List all clusters with summaries and file objects."""
        if not self.clusters:
            return []
        return [
            {
                'id': cid,
                'file_count': len(self.clusters[cid]),
                'files': self.get_cluster_files(cid),
                'sample_files': [Path(f).name for f in self.clusters[cid][:5]]
            }
            for cid in sorted(self.clusters.keys())
        ]
