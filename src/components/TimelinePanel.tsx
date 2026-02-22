import { useState, useEffect } from "react";
import { tauriService } from "../services/tauri";
import * as Types from "../types";

export function TimelinePanel({ indexDir }: { indexDir: string }) {
    const [days, setDays] = useState(30);
    const [timeline, setTimeline] = useState<Types.TimelineData | null>(null);
    const [loading, setLoading] = useState(false);

    useEffect(() => {
        if (!indexDir) return;

        setLoading(true);
        tauriService
            .getTimeline(indexDir, days)
            .then(setTimeline)
            .catch((err) => {
                console.error("Timeline fetch failed:", err);
                alert(`Timeline error: ${err}`);
            })
            .finally(() => setLoading(false));
    }, [indexDir, days]);

    return (
        <div className="panel">
            <div className="timeline-container" style={{ width: "100%", maxWidth: "100%" }}>
                <h3>Timeline</h3>

                <div className="days-selector" style={{ marginBottom: "2rem" }}>
                    <label>
                        Show last:
                        <select value={days} onChange={(e) => setDays(Number(e.target.value))}>
                            <option value={7}>7 days</option>
                            <option value={14}>14 days</option>
                            <option value={30}>30 days</option>
                            <option value={90}>90 days</option>
                        </select>
                    </label>
                </div>

                {loading ? (
                    <p style={{ textAlign: "center", color: "var(--text-secondary)" }}>Loading...</p>
                ) : timeline && timeline.entries && timeline.entries.length > 0 ? (
                    <div className="timeline-entries">
                        {timeline.entries.map((entry, idx) => (
                            <div key={idx} className="timeline-entry">
                                <div className="entry-date">📅 {entry.date}</div>
                                <div className="entry-count">{entry.file_count} files</div>
                                {entry.files.length > 0 && (
                                    <ul style={{ fontSize: "var(--font-size-sm)", marginTop: "0.5rem", color: "var(--text-secondary)" }}>
                                        {entry.files.slice(0, 3).map((file, idx) => (
                                            <li key={idx}>{typeof file === 'string' ? file : file.name}</li>
                                        ))}
                                        {entry.files.length > 3 && <li>... and {entry.files.length - 3} more</li>}
                                    </ul>
                                )}
                            </div>
                        ))}
                    </div>
                ) : (
                    <p style={{ textAlign: "center", color: "var(--text-secondary)" }}>No data available</p>
                )}
            </div>
        </div>
    );
}
