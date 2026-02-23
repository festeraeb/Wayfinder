import React, { useEffect, useState } from "react";
import { tauriService } from "../services/tauri";
import type { NautiReport, DuplicateMeta } from "../types";
import "./GitAssistant.css";

interface Props {
    defaultPaths: string[];
}

export const NautiClippy: React.FC<Props> = ({ defaultPaths }) => {
    const [paths, setPaths] = useState<string[]>(defaultPaths);
    const [report, setReport] = useState<NautiReport | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [chatInput, setChatInput] = useState("");
    const [chatReply, setChatReply] = useState<string | null>(null);
    const [chatLoading, setChatLoading] = useState(false);
    const [chatError, setChatError] = useState<string | null>(null);

    const runScan = async () => {
        if (!paths || paths.length === 0) return;
        setLoading(true);
        setError(null);
        try {
            const res = await tauriService.getNautiClippyReport(paths);
            setReport(res);
        } catch (e) {
            setError(String(e));
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        runScan();
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []);

    const handlePathsChange = (value: string) => {
        const list = value.split(/[,\n]/).map(p => p.trim()).filter(Boolean);
        setPaths(list);
    };

    const renderMeta = (meta?: DuplicateMeta) => {
        if (!meta) return null;
        const sizeKb = Math.round((meta.size_bytes / 1024) * 10) / 10;
        return <span className="meta">{sizeKb} KB · {meta.modified || ""}</span>;
    };

    const askLlama = async () => {
        if (!chatInput.trim()) return;
        setChatLoading(true);
        setChatError(null);
        try {
            const res = await tauriService.chatLlama(chatInput, "qwen2.5-coder-1.5b-instruct", undefined);
            setChatReply(res.text);
        } catch (e) {
            setChatError(String(e));
        } finally {
            setChatLoading(false);
        }
    };

    return (
        <div className="git-assistant">
            <div className="git-header">
                <span className="clippy-icon">🧭</span>
                <div>
                    <h3>Nauti-Clippy</h3>
                    <p className="section-desc">Organize non-git folders (Documents, Downloads, Pictures).</p>
                </div>
            </div>

            <div className="form-group">
                <label>Folders to watch (comma or newline separated)</label>
                <textarea
                    className="folder-input"
                    rows={3}
                    value={paths.join("\n")}
                    onChange={(e) => handlePathsChange(e.target.value)}
                />
                <small>.git folders are skipped automatically.</small>
                <div className="actions-row">
                    <button className="action-btn primary" onClick={runScan} disabled={loading}>
                        {loading ? "Scanning..." : "Scan now"}
                    </button>
                </div>
            </div>

            {error && <div className="git-error">{error}</div>}

            {report && (
                <div className="nauti-report">
                    <div className="stat-cards">
                        <div className="stat-card">
                            <span className="stat-value">{report.scanned_files}</span>
                            <span className="stat-label">files scanned</span>
                        </div>
                        <div className="stat-card">
                            <span className="stat-value">{report.duplicates.reduce((acc, d) => acc + (d.duplicate_meta?.length || d.duplicates.length), 0)}</span>
                            <span className="stat-label">duplicates</span>
                        </div>
                        <div className="stat-card">
                            <span className="stat-value">{report.copy_pattern_files.length}</span>
                            <span className="stat-label">copy/backup names</span>
                        </div>
                    </div>

                    {report.duplicates.length > 0 && (
                        <div className="duplicate-list">
                            <h4>🗑️ Potential duplicates</h4>
                            {report.duplicates.map((dup, i) => (
                                <div key={i} className="duplicate-group">
                                    <div className="original">
                                        <span className="label">Keep:</span>
                                        <span className="path">{dup.original}</span>
                                        {renderMeta(dup.original_meta)}
                                    </div>
                                    {(dup.duplicate_meta && dup.duplicate_meta.length > 0 ? dup.duplicate_meta.map((m, j) => (
                                        <div key={j} className="duplicate">
                                            <span className="label">Review:</span>
                                            <span className="path">{m.path}</span>
                                            {renderMeta(m)}
                                        </div>
                                    )) : dup.duplicates.map((d, j) => (
                                        <div key={j} className="duplicate">
                                            <span className="label">Review:</span>
                                            <span className="path">{d}</span>
                                        </div>
                                    )))}
                                </div>
                            ))}
                        </div>
                    )}

                    {report.copy_pattern_files.length > 0 && (
                        <div className="copy-list">
                            <h4>📄 Copy/backup candidates</h4>
                            {report.copy_pattern_files.map((f, i) => (
                                <div key={i} className="duplicate">
                                    <span className="path">{f.file_path}</span>
                                    <small className="meta">{f.reason}</small>
                                </div>
                            ))}
                        </div>
                    )}

                    {report.duplicates.length === 0 && report.copy_pattern_files.length === 0 && (
                        <div className="no-suggestions">
                            <span className="big-emoji">🎉</span>
                            <p>{report.message}</p>
                        </div>
                    )}
                </div>
            )}

            <div className="git-chat" style={{ marginTop: "16px" }}>
                <h3>💬 Ask Qwen (local)</h3>
                <textarea
                    className="folder-input"
                    rows={3}
                    placeholder="Ask for an organizing tip..."
                    value={chatInput}
                    onChange={(e) => setChatInput(e.target.value)}
                />
                <div className="actions-row">
                    <button className="action-btn primary" onClick={askLlama} disabled={chatLoading}>
                        {chatLoading ? "Thinking..." : "Ask"}
                    </button>
                </div>
                {chatError && <div className="git-error">{chatError}</div>}
                {chatReply && (
                    <div className="chat-reply">
                        <strong>Reply:</strong> {chatReply}
                    </div>
                )}
                <small>Uses your local llama.cpp server at its default endpoint.</small>
            </div>
        </div>
    );
};

export default NautiClippy;
