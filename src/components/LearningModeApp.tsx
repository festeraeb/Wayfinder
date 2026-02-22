import { useState, useEffect } from "react";
import { tauriService } from "../services/tauri";
import * as Types from "../types";
import "../styles/learning-mode.css";

interface LearningSession {
    sessionId: string;
    startTime: Date;
    queriesPerformed: string[];
    documentsReviewed: string[];
    score: number;
}

export function LearningModeApp({ indexDir }: { indexDir: string }) {
    const [session, setSession] = useState<LearningSession | null>(null);
    const [searchQuery, setSearchQuery] = useState("");
    const [results, setResults] = useState<Types.SearchResult[]>([]);
    const [feedback, setFeedback] = useState("");
    const [score, setScore] = useState(0);
    const [loading, setLoading] = useState(false);

    useEffect(() => {
        if (session) {
            // Log session start to backend
            tauriService.logSessionStart(session.sessionId, session.startTime.toISOString());
        }
    }, [session]);

    const startSession = () => {
        const newSession: LearningSession = {
            sessionId: Date.now().toString(),
            startTime: new Date(),
            queriesPerformed: [],
            documentsReviewed: [],
            score: 0,
        };
        setSession(newSession);
        setScore(0);
    };

    const handleSearch = async () => {
        if (!searchQuery.trim()) return;

        setLoading(true);
        try {
            const results = await tauriService.search(searchQuery, indexDir, 5, 0.7);
            setResults(Array.isArray(results) ? results : []);
            // Removed: setCurrentQuestion(0);

            // Update session
            if (session) {
                session.queriesPerformed.push(searchQuery);
                setSession({ ...session });
            }
        } catch (error) {
            console.error("Search failed:", error);
        } finally {
            setLoading(false);
        }
    };

    const handleAnswer = (resultPath: string) => {
        if (session) {
            session.documentsReviewed.push(resultPath);
            setSession({ ...session });
            setScore(score + 10);

            // Show positive feedback
            setFeedback("✓ Great! You found a relevant document!");
            setTimeout(() => setFeedback(""), 2000);
        }
    };

    const endSession = () => {
        if (session) {
            console.log("Session ended:", {
                duration: new Date().getTime() - session.startTime.getTime(),
                queriesPerformed: session.queriesPerformed.length,
                documentsReviewed: session.documentsReviewed.length,
                score,
            });
            setSession(null);
            setScore(0);
            setResults([]);
        }
    };

    if (!session) {
        return (
            <div className="learning-mode-start">
                <div className="learning-logo">📚</div>
                <h1>Welcome to Learning Mode</h1>
                <p className="learning-subtitle">
                    Explore and discover documents by searching and answering questions
                </p>
                <button
                    onClick={startSession}
                    className="btn btn-primary learning-start-btn"
                    disabled={!indexDir}
                >
                    Start Learning Session
                </button>
                {!indexDir && <p className="learning-hint">Please set an index directory first</p>}
            </div>
        );
    }

    return (
        <div className="learning-mode">
            <div className="learning-header">
                <div className="learning-score">📊 Score: {score}</div>
                <button onClick={endSession} className="btn btn-secondary">
                    Exit Learning Mode
                </button>
            </div>

            <div className="learning-container">
                <h2>What would you like to learn about?</h2>

                <div className="learning-search">
                    <input
                        type="text"
                        placeholder="Ask a question or topic..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        onKeyDown={(e) => e.key === "Enter" && handleSearch()}
                        className="learning-search-input"
                        disabled={loading}
                    />
                    <button
                        onClick={handleSearch}
                        className="btn btn-primary"
                        disabled={loading || !searchQuery.trim()}
                    >
                        {loading ? "Looking..." : "🔍 Explore"}
                    </button>
                </div>

                {feedback && <div className="learning-feedback">{feedback}</div>}

                {results.length > 0 && (
                    <div className="learning-results">
                        <h3>Documentation Found</h3>
                        <p className="learning-hint">Click on a document to mark it as reviewed</p>

                        <div className="learning-items">
                            {results.map((result, idx) => (
                                <div
                                    key={idx}
                                    className="learning-item"
                                    onClick={() => handleAnswer(result.path)}
                                >
                                    <div className="learning-item-title">{result.name}</div>
                                    <div className="learning-item-preview">{result.preview || result.path}</div>
                                    <div className="learning-item-relevance">
                                        Relevance: {Math.round(result.score * 100)}%
                                    </div>
                                </div>
                            ))}
                        </div>

                        <p className="learning-progress">
                            Documents reviewed: {session.documentsReviewed.length} | Searches: {session.queriesPerformed.length}
                        </p>
                    </div>
                )}

                {!results.length && searchQuery && !loading && (
                    <div className="learning-empty">No documents found. Try another search!</div>
                )}
            </div>
        </div>
    );
}

export function SessionSummary({ session, finalScore }: { session: LearningSession; finalScore: number }) {
    const sessionDuration = new Date().getTime() - session.startTime.getTime();
    const durationMinutes = Math.floor(sessionDuration / 60000);

    return (
        <div className="learning-summary">
            <div className="summary-icon">🎉</div>
            <h2>Great Work!</h2>

            <div className="summary-stats">
                <div className="summary-stat">
                    <span className="stat-label">Session Duration</span>
                    <span className="stat-value">{durationMinutes} min</span>
                </div>
                <div className="summary-stat">
                    <span className="stat-label">Search Queries</span>
                    <span className="stat-value">{session.queriesPerformed.length}</span>
                </div>
                <div className="summary-stat">
                    <span className="stat-label">Documents Reviewed</span>
                    <span className="stat-value">{session.documentsReviewed.length}</span>
                </div>
                <div className="summary-stat">
                    <span className="stat-label">Your Score</span>
                    <span className="stat-value" style={{ color: "var(--success-color)" }}>
                        {finalScore}
                    </span>
                </div>
            </div>

            <p className="summary-message">
                Keep exploring and learning! 🌟
            </p>
        </div>
    );
}
