import { useCallback, useEffect, useState } from "react";
import type { InteractionAgentSettings } from "../types";

const STORAGE_KEY = "wayfinder-interaction-settings";

const DEFAULT_SETTINGS: InteractionAgentSettings = {
    sarcasmLevel: 0.35,
    mood: "medium",
    muted: false,
    muteMinutes: 15,
    rewardTone: "genuine",
    openDyslexic: false,
    watchScope: "user",
};

export function useInteractionSettings() {
    const [settings, setSettings] = useState<InteractionAgentSettings>(DEFAULT_SETTINGS);

    useEffect(() => {
        try {
            const raw = localStorage.getItem(STORAGE_KEY);
            if (raw) {
                const parsed = JSON.parse(raw) as InteractionAgentSettings;
                setSettings({ ...DEFAULT_SETTINGS, ...parsed });
            }
        } catch (err) {
            // ignore corrupted state
        }
    }, []);

    const persist = useCallback((next: InteractionAgentSettings) => {
        setSettings(next);
        try {
            localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
        } catch (err) {
            // ignore storage failures
        }
    }, []);

    const updateSettings = useCallback((patch: Partial<InteractionAgentSettings>) => {
        setSettings((prev) => {
            const next = { ...prev, ...patch };
            try {
                localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
            } catch (err) {
                // ignore storage failures
            }
            return next;
        });
    }, []);

    const resetSettings = useCallback(() => {
        persist(DEFAULT_SETTINGS);
    }, [persist]);

    return { settings, updateSettings, resetSettings, DEFAULT_SETTINGS };
}
