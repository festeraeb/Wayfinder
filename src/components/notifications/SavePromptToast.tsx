import React, { useEffect, useRef, useState } from "react";
import { tauriService } from "../../services/tauri";
import { useInteractionSettings } from "../../hooks/useInteractionSettings";
import type { SavePrompt } from "../../types";

const STRICTNESS_LABELS = ["Just trim", "Strip clutter", "Tidy names", "Tidy + date"];

/** Same thirds as the Rust side (clippy::tone_ceiling). */
const sarcasmLabel = (s: number) => (s < 1 / 3 ? "Kind" : s < 2 / 3 ? "Dry" : "Spicy");

interface Props {
    /** Poll only while the file watcher is running. */
    active: boolean;
}

/** Mr. Clippy's save prompts: what was saved badly, his comment, and a better name to copy. */
const SavePromptToast: React.FC<Props> = ({ active }) => {
    const [prompts, setPrompts] = useState<SavePrompt[]>([]);
    const [strictness, setStrictness] = useState(2);
    const [copied, setCopied] = useState<string | null>(null);
    const timer = useRef<ReturnType<typeof setInterval> | null>(null);
    const { settings, updateSettings } = useInteractionSettings();

    useEffect(() => {
        if (!active) return;
        const poll = async () => {
            try {
                const res = await tauriService.getSavePrompts(true);
                if (res.prompts.length) setPrompts((cur) => [...cur, ...res.prompts].slice(-5));
                setStrictness(res.strictness);
            } catch {
                // backend not ready yet; try again next tick
            }
        };
        poll();
        timer.current = setInterval(poll, 4000);
        return () => {
            if (timer.current) clearInterval(timer.current);
        };
    }, [active]);

    const changeStrictness = async (level: number) => {
        setStrictness(level);
        try {
            await tauriService.setSaveStrictness(level);
        } catch {
            // keep the local value; the next poll reports the backend's
        }
    };

    const copyName = async (name: string) => {
        try {
            await navigator.clipboard.writeText(name);
            setCopied(name);
            setTimeout(() => setCopied(null), 1500);
        } catch {
            setCopied(null);
        }
    };

    const dismiss = (i: number) => setPrompts((cur) => cur.filter((_, j) => j !== i));

    if (!active) return null;
    return (
        <div className="save-prompts" aria-live="polite">
            <label className="save-prompts__strictness">
                Naming strictness: <strong>{STRICTNESS_LABELS[strictness]}</strong>
                <input type="range" min={0} max={3} step={1} value={strictness}
                    onChange={(e) => changeStrictness(Number(e.target.value))} />
            </label>
            <label className="save-prompts__strictness" title="Mr. Clippy's tone, and the tone of the chat assistant">
                Clippy's sarcasm: <strong>{sarcasmLabel(settings.sarcasmLevel)}</strong>
                <input type="range" min={0} max={100} step={1} value={Math.round(settings.sarcasmLevel * 100)}
                    onChange={(e) => updateSettings({ sarcasmLevel: Number(e.target.value) / 100 })} />
            </label>
            {prompts.map((p, i) => (
                <div className="save-prompts__card" key={`${p.path}-${i}`}>
                    <div className="save-prompts__line">📎 {p.line}</div>
                    <div className="save-prompts__file" title={p.path}>{p.path.split(/[\\/]/).pop()}</div>
                    <div className="save-prompts__suggest">
                        Try: <code>{p.suggested_name}</code>
                        <button onClick={() => copyName(p.suggested_name)}>
                            {copied === p.suggested_name ? "Copied" : "Copy name"}
                        </button>
                        <button onClick={() => dismiss(i)}>Got it</button>
                    </div>
                </div>
            ))}
        </div>
    );
};

export default SavePromptToast;
