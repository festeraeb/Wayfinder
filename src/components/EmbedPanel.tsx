import { useState } from "react";
import { tauriService } from "../services/tauri";

export function EmbedPanel({ indexDir }: { indexDir: string }) {
    const [loading, setLoading] = useState(false);
    const [result, setResult] = useState<any>(null);
    const [progress, setProgress] = useState({ percent: 0 });

    const handleGenerateEmbeddings = async () => {
        if (!indexDir) {
            alert("Please set an index directory");
            return;
        }

        setLoading(true);
        setProgress({ percent: 0 });
        try {
            const embedResult = await tauriService.generateEmbeddings(indexDir);
            setResult(embedResult);
        } catch (error) {
            console.error("Embedding failed:", error);
            alert(`Embedding error: ${error}`);
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="panel">
            <div className="embed-container" style={{ width: "100%", maxWidth: "100%" }}>
                <h3>Generate Embeddings</h3>
                <p style={{ color: "var(--text-secondary)", marginBottom: "1.5rem" }}>
                    Convert your indexed files into semantic embeddings for smart search and clustering.
                </p>

                <button
                    onClick={handleGenerateEmbeddings}
                    className="btn btn-primary"
                    disabled={loading || !indexDir}
                >
                    {loading ? "Generating..." : "🧠 Generate Embeddings"}
                </button>

                {loading && (
                    <div style={{ marginTop: "2rem" }}>
                        <div className="progress-bar">
                            <div
                                className="progress-bar-fill"
                                style={{ width: `${progress.percent}%` }}
                            ></div>
                        </div>
                        <p style={{ textAlign: "center", color: "var(--text-secondary)", marginTop: "1rem" }}>
                            Processing... {progress.percent.toFixed(0)}%
                        </p>
                    </div>
                )}

                {result && (
                    <div className="embed-result" style={{ marginTop: "2rem" }}>
                        <h4>Embedding Complete ✓</h4>
                        <div className="result-stats">
                            <div className="stat">
                                <span className="stat-label">Generated:</span>
                                <span className="stat-value">{result.embeddings_generated}</span>
                            </div>
                            <div className="stat">
                                <span className="stat-label">Cached:</span>
                                <span className="stat-value">{result.cached_count}</span>
                            </div>
                        </div>
                        <p style={{ color: "var(--success-color)", marginTop: "1rem" }}>
                            Ready for clustering and search!
                        </p>
                    </div>
                )}
            </div>
        </div>
    );
}
