import { useState } from "react";
import { tauriService } from "../services/tauri";
import * as Types from "../types";

export function ClusterPanel({ indexDir }: { indexDir: string }) {
    const [numClusters, setNumClusters] = useState<number | undefined>(undefined);
    const [loading, setLoading] = useState(false);
    const [clusters, setClusters] = useState<Types.ClusterInfo[]>([]);
    const [expandedCluster, setExpandedCluster] = useState<number | null>(null);

    const handleCreateClusters = async () => {
        if (!indexDir) {
            alert("Please set an index directory");
            return;
        }

        setLoading(true);
        try {
            await tauriService.createClusters(indexDir, numClusters);
            // Fetch clusters summary
            const summary = await tauriService.getClustersSummary(indexDir);
            setClusters(Array.isArray(summary.clusters) ? summary.clusters : []);
        } catch (error) {
            console.error("Clustering failed:", error);
            alert(`Clustering error: ${error}`);
        } finally {
            setLoading(false);
        }
    };

    function renderFileList(files: any[]) {
        return files.slice(0, 5).map((file, idx) => (
            <li key={idx}>
                {file.name || "(no name)"}
                {file.path && (
                    <span style={{ color: '#888', marginLeft: 8, fontSize: '0.9em' }}>({file.path})</span>
                )}
            </li>
        ));
    }

    return (
        <div className="panel">
            <div className="cluster-container" style={{ width: "100%", maxWidth: "100%" }}>
                <h3>Create Clusters</h3>

                <div className="input-group">
                    <input
                        type="number"
                        placeholder="Number of clusters (auto-estimate if blank)"
                        value={numClusters ?? ""}
                        onChange={(e) => setNumClusters(e.target.value ? Number(e.target.value) : undefined)}
                        className="text-input"
                        disabled={!indexDir}
                    />
                    <button
                        onClick={handleCreateClusters}
                        className="btn btn-primary"
                        disabled={loading || !indexDir}
                    >
                        {loading ? "Clustering..." : "🔗 Create Clusters"}
                    </button>
                </div>

                {clusters.length > 0 && (
                    <div className="clusters-list" style={{ marginTop: "2rem" }}>
                        <h4>Clusters ({clusters.length})</h4>
                        {clusters.map((cluster) => {
                            const size = cluster.file_count ?? (Array.isArray(cluster.files) ? cluster.files.length : 0);
                            const displayFiles = (cluster.files ?? []) as Array<{ name: string; path: string }>;
                            return (
                                <div
                                    key={cluster.id}
                                    className="cluster-item"
                                    onClick={() =>
                                        setExpandedCluster(expandedCluster === cluster.id ? null : cluster.id)
                                    }
                                    style={{
                                        cursor: "pointer",
                                        padding: "1rem",
                                        border: "1px solid var(--border-color)",
                                        borderRadius: "var(--radius-md)",
                                        marginTop: "0.5rem",
                                        backgroundColor: expandedCluster === cluster.id ? "var(--bg-tertiary)" : "transparent",
                                    }}
                                >
                                    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                                        <span className="cluster-title">{cluster.label || `Cluster ${cluster.id}`}</span>
                                        <span className="cluster-size">{size} files</span>
                                    </div>
                                    {cluster.summary && (
                                        <p style={{ color: "var(--text-secondary)", fontSize: "var(--font-size-sm)", marginTop: "0.5rem" }}>
                                            {cluster.summary}
                                        </p>
                                    )}
                                    {expandedCluster === cluster.id && displayFiles.length > 0 && (
                                        <div style={{ marginTop: "1rem", paddingTop: "1rem", borderTop: "1px solid var(--border-color)" }}>
                                            <p style={{ fontWeight: 500, marginBottom: "0.5rem" }}>Sample files:</p>
                                            <ul style={{ fontSize: "var(--font-size-sm)", color: "var(--text-secondary)" }}>
                                                {renderFileList(displayFiles)}
                                                {displayFiles.length > 5 && <li>... and {displayFiles.length - 5} more</li>}
                                            </ul>
                                        </div>
                                    )}
                                </div>
                            );
                        })}
                    </div>
                )}
            </div>
        </div>
    );
}
