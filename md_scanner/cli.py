"""
CLI interface for md-scanner.
"""
import click
from pathlib import Path
from tqdm import tqdm
from md_scanner.scanner import FileScanner
from md_scanner.clustering import ClusteringEngine
from md_scanner.timeline import TimelineEngine

INDEX_DIR = Path.home() / '.md_index'

@click.group()
def cli():
    """Markdown Scanner & Organizer - Find your scattered thoughts."""
    pass

@cli.command()
@click.argument('directory', type=click.Path(exists=True))
@click.option('--index-dir', default=str(INDEX_DIR), help='Index directory')
def scan(directory, index_dir):
    """Scan directory for markdown files."""
    click.echo(f"Scanning {directory}...")

    scanner = FileScanner(index_dir)

    def progress_update(count):
        click.echo(f"  Found {count} files...", nl=False)
        click.echo('\r', nl=False)

    files = scanner.scan(directory, progress_callback=progress_update)
    click.echo(f"\nFound {len(files)} markdown files")

    scanner.save_index(files)
    click.echo(f"Index saved to {scanner.index_file}")

@cli.command()
@click.option('--collection', default=None,
              help='Qdrant collection (default QDRANT_COLLECTION or codebase_v6)')
@click.option('--min-cluster-size', type=int, default=5,
              help='HDBSCAN min_cluster_size')
@click.option('--qdrant-index-dir', 'qdrant_index_dir', default=str(INDEX_DIR),
              help='Where to save clusters.json')
@click.option('--limit', type=int, default=0,
              help='Max points to cluster (0 = whole collection)')
def cluster_qdrant(collection, min_cluster_size, qdrant_index_dir, limit):
    """Cluster a Qdrant collection via HDBSCAN (fleet all-drives audit)."""
    from md_scanner.qdrant_backend import QdrantEngine
    from md_scanner.clustering import ClusteringEngine
    import os
    coll = collection or os.environ.get("QDRANT_COLLECTION", "codebase_v6")
    click.echo(f"Fetching vectors from Qdrant collection '{coll}'...")
    qe = QdrantEngine(collection=coll)
    total = qe.point_count()
    click.echo(f"  {total} points in collection")
    n = limit if limit and limit < total else total
    vecs, paths, offset, done = [], [], None, 0
    with tqdm(total=n) as pbar:
        while done < n:
            pts, offset = qe.scroll_points(limit=min(1000, n - done), offset=offset)
            if not pts:
                break
            for p in pts:
                v = p.get("vector")
                if isinstance(v, dict):
                    v = next(iter(v.values()))
                if v is None:
                    continue
                vecs.append(v)
                pl = p.get("payload", {})
                paths.append(pl.get("file_path") or pl.get("path")
                             or pl.get("file") or str(p.get("id")))
            done += len(pts)
            pbar.update(len(pts))
            if offset is None:
                break
    import numpy as np
    click.echo(f"Clustering {len(vecs)} vectors (HDBSCAN, min_size={min_cluster_size})...")
    ce = ClusteringEngine(index_dir=qdrant_index_dir)
    clusters = ce.cluster_hdbscan(np.array(vecs, dtype="float32"), paths,
                                  min_cluster_size=min_cluster_size,
                                  progress_callback=lambda s, m: click.echo(f"  {m}"))
    click.echo(f"\nCreated {len(clusters)} clusters:")
    for summary in ce.list_clusters():
        click.echo(f"  Cluster {summary['id']}: {summary['file_count']} files")
    click.echo(f"Saved to {ce.clusters_file}")


@cli.command()
@click.option('--index-dir', default=str(INDEX_DIR), help='Index directory')
def embed(index_dir):
    """Generate embeddings for indexed files."""
    from md_scanner.embeddings import EmbeddingEngine
    scanner = FileScanner(index_dir)
    files = scanner.load_index()

    if not files:
        click.echo("No index found. Run 'scan' first.")
        return

    click.echo(f"Generating embeddings for {len(files)} files...")

    engine = EmbeddingEngine(index_dir=index_dir)
    file_paths = [f.path for f in files]

    with tqdm(total=len(file_paths)) as pbar:
        def progress_update(current, total):
            pbar.update(1)

        engine.generate_embeddings(file_paths, progress_callback=progress_update)

    click.echo(f"Embeddings saved to {engine.embeddings_file}")

@cli.command()
@click.option('--index-dir', default=str(INDEX_DIR), help='Index directory')
@click.option('--num-clusters', type=int, default=None, help='Number of clusters (KMeans only)')
@click.option('--hdbscan/--kmeans', 'use_hdbscan', default=True,
              help='Use HDBSCAN unknown-k (default) or legacy KMeans')
@click.option('--min-cluster-size', type=int, default=5,
              help='HDBSCAN min_cluster_size')
def cluster(index_dir, num_clusters, use_hdbscan, min_cluster_size):
    """Cluster files into semantic groups."""
    embedding_engine = EmbeddingEngine(index_dir=index_dir)

    if not embedding_engine.load_embeddings():
        click.echo("No embeddings found. Run 'embed' first.")
        return

    click.echo(f"Clustering {len(embedding_engine.file_paths)} files...")

    clustering_engine = ClusteringEngine(index_dir=index_dir)

    def progress_update(stage, message):
        click.echo(f"  {message}")

    if use_hdbscan:
        clusters = clustering_engine.cluster_hdbscan(
            embedding_engine.embeddings,
            embedding_engine.file_paths,
            min_cluster_size=min_cluster_size,
            progress_callback=progress_update,
        )
    else:
        clusters = clustering_engine.cluster(
        embedding_engine.embeddings,
        embedding_engine.file_paths,
        n_clusters=num_clusters,
        progress_callback=progress_update
    )

    click.echo(f"\nCreated {len(clusters)} clusters:")
    for summary in clustering_engine.list_clusters():
        click.echo(
            f"  Cluster {summary['id']}: {summary['file_count']} files - "
            f"{', '.join(summary['sample_files'][:2])}..."
        )

@cli.command()
@click.option('--index-dir', default=str(INDEX_DIR), help='Index directory')
@click.option('--top-k', type=int, default=10, help='Number of results')
@click.option('--semantic-weight', type=float, default=0.7, help='Semantic weight')
@click.argument('query')
def search(query, index_dir, top_k, semantic_weight):
    """Search for files by semantic similarity."""
    from md_scanner.embeddings import EmbeddingEngine
    embedding_engine = EmbeddingEngine(index_dir=index_dir)

    if not embedding_engine.load_embeddings():
        click.echo("No embeddings found. Run 'embed' first.")
        return

    click.echo(f"Searching for: '{query}'")

    search_engine = SearchEngine(embedding_engine)
    results = search_engine.search(query, semantic_weight=semantic_weight, top_k=top_k)

    click.echo(f"\nFound {len(results)} results:")
    for i, (file_path, score) in enumerate(results, 1):
        file_name = Path(file_path).name
        click.echo(f"  {i}. {file_name} ({score:.3f})")
        click.echo(f"     => {file_path}")

@cli.command()
@click.option('--index-dir', default=str(INDEX_DIR), help='Index directory')
def list_clusters(index_dir):
    """List all clusters."""
    clustering_engine = ClusteringEngine(index_dir=index_dir)

    if not clustering_engine.load_clusters():
        click.echo("No clusters found. Run 'cluster' first.")
        return

    clusters = clustering_engine.list_clusters()

    click.echo(f"Total clusters: {len(clusters)}\n")
    for summary in clusters:
        click.echo(f"Cluster {summary['id']} ({summary['file_count']} files):")
        for file_name in summary['sample_files']:
            click.echo(f"  - {file_name}")
        click.echo()

@cli.command()
@click.option('--index-dir', default=str(INDEX_DIR), help='Index directory')
@click.option('--days', type=int, default=30, help='Days to show')
def timeline(index_dir, days):
    """Show recent files organized by date."""
    scanner = FileScanner(index_dir)
    files = scanner.load_index()

    if not files:
        click.echo("No index found. Run 'scan' first.")
        return

    timeline_engine = TimelineEngine()
    timeline_engine.set_data(files)

    timeline_data = timeline_engine.get_timeline_by_date(days=days)

    click.echo(f"Files modified in last {days} days:\n")
    for date_str, file_list in timeline_data.items():
        click.echo(f"{date_str} ({len(file_list)} files):")
        for file_path in file_list[:5]:
            file_name = Path(file_path).name
            click.echo(f"  - {file_name}")
        if len(file_list) > 5:
            click.echo(f"  ... and {len(file_list) - 5} more")
        click.echo()

@cli.command()
@click.option('--index-dir', default=str(INDEX_DIR), help='Index directory')
def stats(index_dir):
    """Show statistics about indexed files."""
    scanner = FileScanner(index_dir)
    files = scanner.load_index()

    if not files:
        click.echo("No index found. Run 'scan' first.")
        return

    timeline_engine = TimelineEngine()
    timeline_engine.set_data(files)

    click.echo("Index Statistics:")
    click.echo(f"  Total files: {len(files)}")

    # Time bucket stats
    buckets = timeline_engine.get_bucket_summary()
    click.echo("\n  Files by age:")
    for bucket_name, count in buckets.items():
        if count > 0:
            click.echo(f"    {bucket_name}: {count}")

    # Size stats
    total_size = sum(f.size for f in files)
    click.echo(f"\n  Total size: {total_size / 1024 / 1024:.2f} MB")

    # Clustering info
    clustering_engine = ClusteringEngine(index_dir=index_dir)
    if clustering_engine.load_clusters():
        click.echo(f"  Clusters: {len(clustering_engine.clusters)}")

@cli.command()
@click.option('--collection', default=None,
              help='Qdrant collection (default QDRANT_COLLECTION or codebase_v6)')
@click.option('--top-k', type=int, default=10, help='Number of results')
@click.argument('query')
def search_qdrant(collection, top_k, query):
    """Semantic search across a Qdrant collection (fleet all-drives)."""
    from md_scanner.qdrant_backend import QdrantEngine
    import os
    coll = collection or os.environ.get("QDRANT_COLLECTION", "codebase_v6")
    qe = QdrantEngine(collection=coll)
    click.echo(f"Searching collection '{coll}' for: '{query}'")
    results = qe.search(query, top_k=top_k)
    if not results:
        click.echo("  (no results)")
        return
    click.echo(f"\nFound {len(results)} results:")
    for i, (file_path, score) in enumerate(results, 1):
        file_name = Path(file_path).name
        click.echo(f"  {i}. {file_name} ({score:.3f})")
        click.echo(f"     => {file_path}")


if __name__ == '__main__':
    cli()
