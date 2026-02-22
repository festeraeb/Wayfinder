import { useState, useEffect } from "react";
import { tauriService } from "../services/tauri";
import * as Types from "../types";

export function StatsPanel({ indexDir }: { indexDir: string }) {
    const [stats, setStats] = useState<Types.IndexStats | null>(null);
    const [loading, setLoading] = useState(false);

    const refreshStats = async () => {
        if (!indexDir) return;

        setLoading(true);
        try {
            const data = await tauriService.getStats(indexDir);
            setStats(data as Types.IndexStats);
        } catch (error) {
            console.error("Stats fetch failed:", error);
            alert(`Stats error: ${error}`);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        refreshStats();
    }, [indexDir]);

    return (
        <div className="panel">
            <div className="stats-container" style={{ width: "100%", maxWidth: "100%" }}>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "2rem" }}>
                    <h3>Index Statistics</h3>
                    <button onClick={refreshStats} className="btn btn-secondary" disabled={loading || !indexDir}>
                        🔄 Refresh
                    </button>
                </div>

                {loading ? (
                    <p style={{ textAlign: "center", color: "var(--text-secondary)" }}>Loading...</p>
                ) : stats ? (
                    <div className="stats-grid">
                        <div className="stat-card">
                            <div className="stat-value">{stats.total_files}</div>
                            <div className="stat-label">Total Files</div>
                        </div>

                        <div className="stat-card">
                            <div className="stat-value">{(stats.total_size_bytes / 1024 / 1024).toFixed(1)} MB</div>
                            <div className="stat-label">Total Size</div>
                        </div>

                        <div className="stat-card">
                            <div className="stat-value">{stats.embeddings_count}</div>
                            <div className="stat-label">Embeddings</div>
                        </div>

                        <div className="stat-card">
                            <div className="stat-value">{stats.cluster_count}</div>
                            <div className="stat-label">Clusters</div>
                        </div>
                    </div>
                ) : (
                    <p style={{ textAlign: "center", color: "var(--text-secondary)" }}>No data available</p>
                )}

                {stats && stats.age_buckets && stats.age_buckets.length > 0 && (
                    <div style={{ marginTop: "2rem" }}>
                        <h4>Files by Age</h4>
                        <div className="age-buckets">
                            {stats.age_buckets.map((bucket: { label: string; count: number }, idx: number) => (
                                <div
                                    key={idx}
                                    className="age-bucket"
                                    style={{
                                        display: "flex",
                                        justifyContent: "space-between",
                                        padding: "0.5rem 0",
                                        borderBottom: "1px solid var(--border-color)",
                                    }}
                                >
                                    <span>{bucket.label}</span>
                                    <span style={{ fontWeight: 500 }}>{bucket.count}</span>
                                </div>
                            ))}
                        </div>
                    </div>
                )}

                {stats && (
                    <div style={{ marginTop: "2rem", paddingTop: "1rem", borderTop: "1px solid var(--border-color)", color: "var(--text-secondary)", fontSize: "var(--font-size-sm)" }}>
                        Last updated: {stats.last_updated || "N/A"}
                    </div>
                )}
            </div>
        </div>
    );
}
