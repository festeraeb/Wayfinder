import { useState } from "react";
import { tauriService } from "../services/tauri";
import * as Types from "../types";

type Mode = "flat" | "scoped";

export function ClusterPanel({ indexDir }: { indexDir: string }) {
    const [numClusters, setNumClusters] = useState<number | undefined>(undefined);
    const [minPerScope, setMinPerScope] = useState<number>(2);
    const [mode, setMode] = useState<Mode>("flat");
    const [loading, setLoading] = useState(false);
    const [clusters, setClusters] = useState<Types.ClusterInfo[]>([]);
    const [scoped, setScoped] = useState<Types.ScopedClusters | null>(null);
    const [status, setStatus] = useState<string>("");
    const [error, setError] = useState<string | null>(null);
    const [expandedCluster, setExpandedCluster] = useState<number | null>(null);
    const [expandedScope, setExpandedScope] = useState<string | null>(null);

    const handleCreateClusters = async () => {
        if (!indexDir) {
            setError("Please set an index directory first.");
            return;
        }

        setLoading(true);
        setError(null);
        setStatus("");
        try {
            if (mode === "scoped") {
                const res = await tauriService.createScopedClusters(indexDir, minPerScope);
                setStatus(
                    `${res.message ?? "Scoped clustering complete"} (${res.scope_count} folders from ${res.total_files} files)`
                );
                // Read the file back so the UI renders exactly what was persisted.
                setScoped(await tauriService.getScopedClusters(indexDir));
                setClusters([]);
            } else {
                const res = await tauriService.createClusters(indexDir, numClusters);
                setStatus(res.message ?? `Created ${res.clusters_created} clusters`);
                const summary = await tauriService.getClustersSummary(indexDir);
                setClusters(Array.isArray(summary.clusters) ? summary.clusters : []);
                setScoped(null);
            }
        } catch (e) {
            console.error("Clustering failed:", e);
            setError(String(e));
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

    const panelStyle = {
        cursor: "pointer",
        padding: "1rem",
        border: "1px solid var(--border-color)",
        borderRadius: "var(--radius-md)",
        marginTop: "0.5rem",
    };

    const renderFlat = () => (
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
                            ...panelStyle,
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
    );

    const renderScoped = () => {
        if (!scoped || scoped.scope_count === 0) return null;
        const driveDisplay = scoped.drive_display ?? [];
        const scopeNames = Object.keys(scoped.scopes).sort();

        return (
            <div style={{ marginTop: "2rem" }}>
                <h4>Whole drive ({driveDisplay.length} groups of folders)</h4>
                <p style={{ color: "var(--text-secondary)", fontSize: "var(--font-size-sm)" }}>
                    Rolled up from one centroid per folder — {scoped.scope_count} folders,
                    {" "}{scoped.total_files} files. Expand a group to see its folders.
                </p>

                {driveDisplay.map((group) => (
                    <div
                        key={group.id}
                        className="cluster-item"
                        onClick={() =>
                            setExpandedCluster(expandedCluster === group.id ? null : group.id)
                        }
                        style={{
                            ...panelStyle,
                            backgroundColor: expandedCluster === group.id ? "var(--bg-tertiary)" : "transparent",
                        }}
                    >
                        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                            <span className="cluster-title">Drive group {group.id}</span>
                            <span className="cluster-size">{group.file_count} folders</span>
                        </div>
                        {expandedCluster === group.id && (
                            <ul style={{ marginTop: "0.75rem", fontSize: "var(--font-size-sm)", color: "var(--text-secondary)" }}>
                                {group.files.map((f, i) => (
                                    <li key={i}>{f.path}</li>
                                ))}
                            </ul>
                        )}
                    </div>
                ))}

                <h4 style={{ marginTop: "2rem" }}>Folders ({scopeNames.length})</h4>
                {scopeNames.map((scope) => {
                    const driveId = scoped.scope_to_drive[scope];
                    return (
                        <div
                            key={scope}
                            className="cluster-item"
                            onClick={() => setExpandedScope(expandedScope === scope ? null : scope)}
                            style={{
                                ...panelStyle,
                                backgroundColor: expandedScope === scope ? "var(--bg-tertiary)" : "transparent",
                            }}
                        >
                            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                                <span className="cluster-title">{scope}</span>
                                <span className="cluster-size">
                                    {driveId === undefined ? "unassigned" : `drive group ${driveId}`}
                                </span>
                            </div>
                            {expandedScope === scope && (
                                <div style={{ marginTop: "0.75rem", fontSize: "var(--font-size-sm)", color: "var(--text-secondary)" }}>
                                    {(scoped.scopes[scope] as any[] | undefined)?.length ?? 0} local clusters
                                </div>
                            )}
                        </div>
                    );
                })}
            </div>
        );
    };

    return (
        <div className="panel">
            <div className="cluster-container" style={{ width: "100%", maxWidth: "100%" }}>
                <h3>Create Clusters</h3>

                <div className="input-group" role="group" aria-label="Clustering mode">
                    <button
                        type="button"
                        onClick={() => setMode("flat")}
                        className={mode === "flat" ? "btn btn-primary" : "btn"}
                        disabled={loading}
                    >
                        Whole index
                    </button>
                    <button
                        type="button"
                        onClick={() => setMode("scoped")}
                        className={mode === "scoped" ? "btn btn-primary" : "btn"}
                        disabled={loading}
                    >
                        By folder
                    </button>
                </div>

                <p style={{ color: "var(--text-secondary)", fontSize: "var(--font-size-sm)", marginTop: "0.5rem" }}>
                    {mode === "scoped"
                        ? "Clusters each folder on its own, then rolls the drive up from one centroid per folder. Editing a folder does not force a whole-drive re-cluster."
                        : "Clusters every file together in one flat pass."}
                </p>

                <div className="input-group">
                    {mode === "flat" ? (
                        <input
                            type="number"
                            placeholder="Number of clusters (auto-estimate if blank)"
                            value={numClusters ?? ""}
                            onChange={(e) => setNumClusters(e.target.value ? Number(e.target.value) : undefined)}
                            className="text-input"
                            disabled={!indexDir}
                        />
                    ) : (
                        <input
                            type="number"
                            min={2}
                            placeholder="Min files per folder (default 2)"
                            value={minPerScope}
                            onChange={(e) => setMinPerScope(Math.max(2, Number(e.target.value) || 2))}
                            className="text-input"
                            disabled={!indexDir}
                        />
                    )}
                    <button
                        onClick={handleCreateClusters}
                        className="btn btn-primary"
                        disabled={loading || !indexDir}
                    >
                        {loading
                            ? "Clustering..."
                            : mode === "scoped" ? "🔗 Cluster by folder" : "🔗 Create Clusters"}
                    </button>
                </div>

                {status && (
                    <p className="info-text" role="status" style={{ marginTop: "0.75rem" }}>{status}</p>
                )}
                {error && (
                    <p className="error-message" role="alert" style={{ marginTop: "0.75rem" }}>{error}</p>
                )}

                {mode === "flat" ? renderFlat() : renderScoped()}
            </div>
        </div>
    );
}
