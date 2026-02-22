import { useState, useCallback } from "react";
import { tauriService } from "../services/tauri";
import * as Types from "../types";

export function SearchPanel({ indexDir }: { indexDir: string }) {
    const [searchQuery, setSearchQuery] = useState("");
    const [topK, setTopK] = useState(10);
    const [semanticWeight, setSemanticWeight] = useState(0.7);
    const [results, setResults] = useState<Types.SearchResult[]>([]);
    const [loading, setLoading] = useState(false);

    const handleSearch = useCallback(async () => {
        if (!searchQuery.trim() || !indexDir) {
            alert("Please enter a search query and index directory");
            return;
        }

        setLoading(true);
        try {
            const searchResults = await tauriService.search(
                searchQuery,
                indexDir,
                topK,
                semanticWeight
            );
            setResults(Array.isArray(searchResults) ? searchResults : []);
        } catch (error) {
            console.error("Search failed:", error);
            alert(`Search error: ${error}`);
        } finally {
            setLoading(false);
        }
    }, [searchQuery, indexDir, topK, semanticWeight]);

    const handleKeyPress = (e: React.KeyboardEvent) => {
        if (e.key === "Enter") {
            handleSearch();
        }
    };

    return (
        <div className="panel">
            <div className="search-container" style={{ width: "100%", maxWidth: "100%" }}>
                <div className="search-input-group">
                    <input
                        type="text"
                        placeholder="Search your files..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        onKeyPress={handleKeyPress}
                        className="search-input"
                        disabled={!indexDir}
                    />
                    <button
                        onClick={handleSearch}
                        className="btn btn-primary"
                        disabled={loading || !indexDir}
                    >
                        {loading ? "Searching..." : "🔍 Search"}
                    </button>
                </div>

                <div className="search-options">
                    <div className="option-group">
                        <label>
                            Results (top-k): {topK}
                            <input
                                type="range"
                                min="1"
                                max="50"
                                value={topK}
                                onChange={(e) => setTopK(Number(e.target.value))}
                                className="slider"
                            />
                        </label>
                    </div>
                    <div className="option-group">
                        <label>
                            Semantic Weight: {semanticWeight.toFixed(2)}
                            <input
                                type="range"
                                min="0"
                                max="1"
                                step="0.1"
                                value={semanticWeight}
                                onChange={(e) => setSemanticWeight(Number(e.target.value))}
                                className="slider"
                            />
                        </label>
                    </div>
                </div>

                {results.length > 0 && (
                    <div className="search-results">
                        <h3>Results ({results.length})</h3>
                        {results.map((result, idx) => (
                            <div key={idx} className="result-item">
                                <div className="result-name">{result.name}</div>
                                <div className="result-path">{result.path}</div>
                                <div className="result-score">
                                    Score: <strong>{(result.score * 100).toFixed(1)}%</strong>
                                </div>
                                {result.preview && <div className="result-preview">{result.preview}</div>}
                            </div>
                        ))}
                    </div>
                )}

                {!loading && results.length === 0 && searchQuery && (
                    <div style={{ textAlign: "center", color: "var(--text-secondary)", padding: "2rem" }}>
                        No results found
                    </div>
                )}
            </div>
        </div>
    );
}
