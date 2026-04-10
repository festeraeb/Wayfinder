import React, { useState, useEffect } from "react";
import { tauriService } from "../services/tauri";
import * as Types from "../types";
import "./SnapshotPanel.css";

interface SnapshotConfig {
    storage_provider: string;
    account_name?: string;
    account_key?: string;
    container_name?: string;
    retention_days: number;
    max_snapshots: number;
    auto_snapshot_interval_minutes: number;
}

export const SnapshotPanel: React.FC<{ indexPath: string }> = ({ indexPath }) => {
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [status, setStatus] = useState<Types.SnapshotStatus | null>(null);
    const [snapshots, setSnapshots] = useState<Types.SnapshotMetadata[]>([]);
    const [showConfig, setShowConfig] = useState(false);
    const [config, setConfig] = useState<SnapshotConfig>({
        storage_provider: "azure",
        account_name: "",
        account_key: "",
        container_name: "wayfinder-snapshots",
        retention_days: 7,
        max_snapshots: 30,
        auto_snapshot_interval_minutes: 5,
    });
    const [description, setDescription] = useState("");
    const [copied, setCopied] = useState(false);

    useEffect(() => {
        loadStatus();
        loadSnapshots();
    }, [indexPath]);

    const loadStatus = async () => {
        setLoading(true);
        setError(null);
        try {
            const result = await tauriService.getSnapshotStatus(indexPath);
            setStatus(result);
        } catch (err) {
            setError(err instanceof Error ? err.message : "Failed to load snapshot status");
        } finally {
            setLoading(false);
        }
    };

    const loadSnapshots = async () => {
        try {
            const result = await tauriService.listSnapshots(indexPath);
            setSnapshots(result.snapshots);
        } catch (err) {
            console.error("Failed to load snapshots:", err);
        }
    };

    const loadConfig = async () => {
        try {
            const result = await tauriService.loadSnapshotConfig(indexPath);
            if (result.config) {
                setConfig(result.config);
            }
        } catch (err) {
            console.error("Failed to load config:", err);
        }
    };

    const handleSaveConfig = async () => {
        setLoading(true);
        setError(null);
        try {
            await tauriService.saveSnapshotConfig(
                indexPath,
                config.account_name || "",
                config.account_key || "",
                config.container_name || "wayfinder-snapshots",
                config.retention_days,
                config.max_snapshots,
                config.auto_snapshot_interval_minutes
            );
            setShowConfig(false);
            await loadStatus();
        } catch (err) {
            setError(err instanceof Error ? err.message : "Failed to save config");
        } finally {
            setLoading(false);
        }
    };

    const handleCreateSnapshot = async () => {
        setLoading(true);
        setError(null);
        try {
            const result = await tauriService.createSnapshot(indexPath, description || undefined);
            setDescription("");
            await loadStatus();
            await loadSnapshots();
        } catch (err) {
            setError(err instanceof Error ? err.message : "Failed to create snapshot");
        } finally {
            setLoading(false);
        }
    };

    const handleRestoreSnapshot = async (snapshotId: string) => {
        if (!window.confirm(`Restore snapshot ${snapshotId}? Current index will be overwritten.`)) {
            return;
        }

        setLoading(true);
        setError(null);
        try {
            await tauriService.restoreSnapshot(indexPath, snapshotId);
            await loadStatus();
            await loadSnapshots();
        } catch (err) {
            setError(err instanceof Error ? err.message : "Failed to restore snapshot");
        } finally {
            setLoading(false);
        }
    };

    const formatBytes = (bytes: number) => {
        if (bytes === 0) return "0 B";
        const k = 1024;
        const sizes = ["B", "KB", "MB", "GB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + " " + sizes[i];
    };

    const formatDate = (dateString: string) => {
        try {
            return new Date(dateString).toLocaleString();
        } catch {
            return dateString;
        }
    };

    return (
        <div className="snapshot-panel">
            <div className="snapshot-header">
                <h2>💾 Snapshot Manager</h2>
                <p>Back up and restore your file index state</p>
            </div>

            {error && <div className="error-message">{error}</div>}

            {/* Status Cards */}
            <div className="snapshot-status-cards">
                <div className="status-card">
                    <div className="card-label">Status</div>
                    <div className="card-value">
                        {status?.configured ? "✅ Configured" : "⚠️ Not Configured"}
                    </div>
                </div>
                <div className="status-card">
                    <div className="card-label">Snapshots</div>
                    <div className="card-value">{status?.snapshots_available || 0}</div>
                </div>
                {status?.last_snapshot && (
                    <div className="status-card">
                        <div className="card-label">Latest</div>
                        <div className="card-value">{formatDate(status.last_snapshot.timestamp)}</div>
                    </div>
                )}
            </div>

            {/* Configuration Section */}
            {!status?.configured && (
                <div className="config-section">
                    <button
                        onClick={() => {
                            setShowConfig(!showConfig);
                            if (!showConfig) loadConfig();
                        }}
                        className="btn-primary"
                    >
                        {showConfig ? "Hide" : "Setup"} Snapshot Storage
                    </button>

                    {showConfig && (
                        <div className="config-form">
                            <div className="form-group">
                                <label>Azure Storage Account Name</label>
                                <input
                                    type="text"
                                    value={config.account_name || ""}
                                    onChange={(e) =>
                                        setConfig({ ...config, account_name: e.target.value })
                                    }
                                    placeholder="e.g., mystorageaccount"
                                />
                            </div>

                            <div className="form-group">
                                <label>Azure Storage Account Key</label>
                                <input
                                    type="password"
                                    value={config.account_key || ""}
                                    onChange={(e) =>
                                        setConfig({ ...config, account_key: e.target.value })
                                    }
                                    placeholder="Primary access key (gitignored locally)"
                                />
                            </div>

                            <div className="form-group">
                                <label>Container Name</label>
                                <input
                                    type="text"
                                    value={config.container_name || ""}
                                    onChange={(e) =>
                                        setConfig({ ...config, container_name: e.target.value })
                                    }
                                    placeholder="e.g., wayfinder-snapshots"
                                />
                            </div>

                            <div className="config-grid">
                                <div className="form-group">
                                    <label>Max Snapshots</label>
                                    <input
                                        type="number"
                                        value={config.max_snapshots}
                                        onChange={(e) =>
                                            setConfig({
                                                ...config,
                                                max_snapshots: parseInt(e.target.value) || 30,
                                            })
                                        }
                                        min="1"
                                    />
                                    <small>Keep last N snapshots</small>
                                </div>

                                <div className="form-group">
                                    <label>Retention Days</label>
                                    <input
                                        type="number"
                                        value={config.retention_days}
                                        onChange={(e) =>
                                            setConfig({
                                                ...config,
                                                retention_days: parseInt(e.target.value) || 7,
                                            })
                                        }
                                        min="1"
                                    />
                                    <small>Delete older snapshots</small>
                                </div>

                                <div className="form-group">
                                    <label>Auto-Snapshot Interval</label>
                                    <input
                                        type="number"
                                        value={config.auto_snapshot_interval_minutes}
                                        onChange={(e) =>
                                            setConfig({
                                                ...config,
                                                auto_snapshot_interval_minutes: parseInt(e.target.value) || 5,
                                            })
                                        }
                                        min="1"
                                    />
                                    <small>Minutes between auto backups</small>
                                </div>
                            </div>

                            <button
                                onClick={handleSaveConfig}
                                disabled={loading}
                                className="btn-primary"
                            >
                                {loading ? "Saving..." : "Save Configuration"}
                            </button>
                        </div>
                    )}
                </div>
            )}

            {/* Create Snapshot Section */}
            {status?.configured && (
                <div className="create-snapshot-section">
                    <h3>Create Snapshot</h3>
                    <div className="form-group">
                        <label>Snapshot Description (optional)</label>
                        <input
                            type="text"
                            value={description}
                            onChange={(e) => setDescription(e.target.value)}
                            placeholder="e.g., Before major reorganization"
                        />
                    </div>
                    <button
                        onClick={handleCreateSnapshot}
                        disabled={loading}
                        className="btn-primary"
                    >
                        {loading ? "Creating..." : "📸 Create Snapshot Now"}
                    </button>
                </div>
            )}

            {/* Snapshots List */}
            {snapshots.length > 0 && (
                <div className="snapshots-list">
                    <h3>Available Snapshots ({snapshots.length})</h3>
                    <div className="snapshots-container">
                        {snapshots.map((snapshot) => (
                            <div key={snapshot.id} className="snapshot-card">
                                <div className="snapshot-header-info">
                                    <div className="snapshot-time">
                                        {formatDate(snapshot.timestamp)}
                                    </div>
                                    {snapshot.description && (
                                        <div className="snapshot-description">
                                            {snapshot.description}
                                        </div>
                                    )}
                                </div>

                                <div className="snapshot-details">
                                    <span>{snapshot.files_count} files</span>
                                    <span>•</span>
                                    <span>{formatBytes(snapshot.size_bytes)}</span>
                                </div>

                                <button
                                    onClick={() => handleRestoreSnapshot(snapshot.id)}
                                    disabled={loading}
                                    className="btn-restore"
                                    title="Restore this snapshot (will overwrite current index)"
                                >
                                    🔄 Restore
                                </button>
                            </div>
                        ))}
                    </div>
                </div>
            )}

            {/* Info Box */}
            <div className="info-box">
                <h4>💡 How Snapshots Work</h4>
                <ul>
                    <li>
                        <strong>Create:</strong> Save index state (files, classifications, history) to Azure Blob Storage
                    </li>
                    <li>
                        <strong>Restore:</strong> Go back to any previous snapshot if something goes wrong
                    </li>
                    <li>
                        <strong>Retention:</strong> Automatically keeps only recent snapshots based on your settings
                    </li>
                    <li>
                        <strong>Safety:</strong> Always review the files before major changes, then create a snapshot
                    </li>
                </ul>
            </div>
        </div>
    );
};
