import { useState } from "react";
import { tauriService } from "../services/tauri";

export function ScanPanel({ indexDir }: { indexDir: string }) {
    const [scanPath, setScanPath] = useState("");
    const [loading, setLoading] = useState(false);
    const [result, setResult] = useState<any>(null);
    const [progress, setProgress] = useState({ current: 0, total: 0 });

    const handleScan = async () => {
        if (!scanPath.trim() || !indexDir) {
            alert("Please enter a directory path and index directory");
            return;
        }

        setLoading(true);
        setProgress({ current: 0, total: 0 });
        try {
            const scanResult = await tauriService.scanDirectory(scanPath, indexDir);
            setResult(scanResult);
        } catch (error) {
            console.error("Scan failed:", error);
            alert(`Scan error: ${error}`);
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="panel">
            <div className="scan-container" style={{ width: "100%", maxWidth: "100%" }}>
                <h3>Scan Directory</h3>

                <div className="input-group">
                    <input
                        type="text"
                        placeholder="Directory to scan..."
                        value={scanPath}
                        onChange={(e) => setScanPath(e.target.value)}
                        className="text-input"
                        disabled={!indexDir}
                    />
                    <button
                        onClick={handleScan}
                        className="btn btn-primary"
                        disabled={loading || !indexDir}
                    >
                        {loading ? "Scanning..." : "📁 Scan"}
                    </button>
                </div>

                {loading && (
                    <div>
                        <div className="progress-bar">
                            <div
                                className="progress-bar-fill"
                                style={{ width: `${progress.total > 0 ? (progress.current / progress.total) * 100 : 0}%` }}
                            ></div>
                        </div>
                        <p style={{ textAlign: "center", color: "var(--text-secondary)" }}>
                            Scanning... {progress.current}/{progress.total}
                        </p>
                    </div>
                )}

                {result && (
                    <div className="scan-result">
                        <h4>Scan Complete</h4>
                        <div className="result-stats">
                            <div className="stat">
                                <span className="stat-label">Files Scanned:</span>
                                <span className="stat-value">{result.files_scanned}</span>
                            </div>
                            <div className="stat">
                                <span className="stat-label">Total Size:</span>
                                <span className="stat-value">{(result.total_size / 1024 / 1024).toFixed(2)} MB</span>
                            </div>
                            <div className="stat">
                                <span className="stat-label">Index Path:</span>
                                <span className="stat-value">{result.index_path}</span>
                            </div>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}
