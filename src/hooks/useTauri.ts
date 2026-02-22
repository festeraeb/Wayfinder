// React hooks for Tauri command invocation
import { useState, useCallback } from "react";
import * as Types from "../types";

export function useTauriCommand<T>(
    commandFn: () => Promise<T>
): {
    data: T | null;
    loading: boolean;
    error: string | null;
    invoke: () => Promise<void>;
} {
    const [data, setData] = useState<T | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const invoke = useCallback(async () => {
        setLoading(true);
        setError(null);
        try {
            const result = await commandFn();
            setData(result);
        } catch (err) {
            setError(err instanceof Error ? err.message : "Unknown error");
        } finally {
            setLoading(false);
        }
    }, [commandFn]);

    return { data, loading, error, invoke };
}

export function useProgress() {
    const [progress, setProgress] = useState<Types.OperationProgress>({
        operation: "",
        current: 0,
        total: 0,
        percent: 0,
        status: "pending",
    });

    return { progress, setProgress };
}

export function useTheme() {
    const [isDark, setIsDark] = useState(() => {
        const saved = localStorage.getItem("theme");
        if (saved) return saved === "dark";
        return window.matchMedia("(prefers-color-scheme: dark)").matches;
    });

    const toggleTheme = useCallback(() => {
        setIsDark((prev) => {
            const newValue = !prev;
            localStorage.setItem("theme", newValue ? "dark" : "light");
            document.documentElement.setAttribute(
                "data-theme",
                newValue ? "dark" : "light"
            );
            return newValue;
        });
    }, []);

    return { isDark, toggleTheme };
}
