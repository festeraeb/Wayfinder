import { useState } from "react";
import { tauriService } from "../services/tauri";
import type { LemonadeSetupReport } from "../types";

/** Lemonade setup check plus a one-line chat test, so a teacher can see the server works. */
export function LemonadeSetupPanel() {
    // Blank fields mean "use the app defaults" (the local Lemonade).
    const [base, setBase] = useState("");
    const [chatModel, setChatModel] = useState("");
    const [embedModel, setEmbedModel] = useState("");
    const [checking, setChecking] = useState(false);
    const [report, setReport] = useState<LemonadeSetupReport | null>(null);
    const [checkError, setCheckError] = useState<string | null>(null);

    const [prompt, setPrompt] = useState("");
    const [chatting, setChatting] = useState(false);
    const [reply, setReply] = useState<string | null>(null);
    const [chatError, setChatError] = useState<string | null>(null);

    const runCheck = async () => {
        setChecking(true);
        setCheckError(null);
        try {
            setReport(await tauriService.checkLemonadeSetup(base.trim() || undefined, chatModel.trim() || undefined, embedModel.trim() || undefined));
        } catch (e) {
            setReport(null);
            setCheckError(String(e));
        } finally {
            setChecking(false);
        }
    };

    const sendChat = async () => {
        if (!prompt.trim()) return;
        setChatting(true);
        setChatError(null);
        setReply(null);
        try {
            const res = await tauriService.chatLlama(prompt, chatModel.trim() || undefined, base.trim() || undefined);
            setReply(res.text);
        } catch (e) {
            setChatError(String(e));
        } finally {
            setChatting(false);
        }
    };

    const row = (label: string, ok: boolean, id: string, detail?: string) => (
        <li data-testid={`lemonade-${id}`} data-ok={ok ? "true" : "false"}>
            {ok ? "✅" : "❌"} {label}{detail ? `: ${detail}` : ""}
        </li>
    );

    return (
        <div className="panel lemonade-panel">
            <div className="form-group">
                <label>Lemonade address (blank = http://127.0.0.1:13305/api):</label>
                <input type="text" data-testid="lemonade-base" value={base} onChange={(e) => setBase(e.target.value)} />
            </div>
            <div className="form-group">
                <label>Chat model (blank = default):</label>
                <input type="text" data-testid="lemonade-chat-model-input" value={chatModel} onChange={(e) => setChatModel(e.target.value)} />
            </div>
            <div className="form-group">
                <label>Embedding model (blank = nomic-embed-text-v1-GGUF):</label>
                <input type="text" data-testid="lemonade-embed-model-input" value={embedModel} onChange={(e) => setEmbedModel(e.target.value)} />
            </div>
            <div className="action-row">
                <button className="btn btn-primary" data-testid="lemonade-check" onClick={runCheck} disabled={checking}>
                    {checking ? "Checking..." : "Check Lemonade"}
                </button>
            </div>

            {checkError && <p className="warning" data-testid="lemonade-check-error">{checkError}</p>}
            {report && (
                <div data-testid="lemonade-report" data-all-ok={report.all_ok ? "true" : "false"}>
                    <h3>{report.all_ok ? "Lemonade is ready" : "Lemonade needs attention"}</h3>
                    <p><small>{report.base} · chat {report.chat_model} · embeddings {report.embed_model}</small></p>
                    <ul className="check-list">
                        {row("Server reachable, models listed", report.models_ok, "models")}
                        {row("Chat model downloaded", report.chat_model_present && report.chat_model_downloaded, "chat-model")}
                        {row("Embedding model downloaded", report.embed_model_present && report.embed_model_downloaded, "embed-model")}
                        {row("One embedding came back", (report.embed_dim ?? 0) > 0, "embed-dim", report.embed_dim ? `${report.embed_dim} numbers` : undefined)}
                        {row("One chat reply came back", !!report.chat_reply?.trim(), "chat-reply", report.chat_reply?.trim())}
                    </ul>
                    {report.problems.length > 0 && (
                        <ul data-testid="lemonade-problems">
                            {report.problems.map((p, i) => <li key={i}>{p}</li>)}
                        </ul>
                    )}
                </div>
            )}

            <h3>Try a chat</h3>
            <div className="input-row">
                <input type="text" data-testid="chat-input" placeholder="Ask something short..." value={prompt}
                    onChange={(e) => setPrompt(e.target.value)} onKeyDown={(e) => e.key === "Enter" && sendChat()} />
                <button className="btn btn-primary" data-testid="chat-send" onClick={sendChat} disabled={chatting}>
                    {chatting ? "Thinking..." : "Send"}
                </button>
            </div>
            {reply !== null && <p data-testid="chat-reply">{reply}</p>}
            {chatError && <p className="warning" data-testid="chat-error">{chatError}</p>}
        </div>
    );
}
