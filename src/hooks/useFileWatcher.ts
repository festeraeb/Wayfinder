import { useEffect, useRef, useState, useCallback } from "react";
import { tauriService } from "../services/tauri";
import type { FileEvent, FileEventsResponse } from "../types";

type WatcherOptions = {
    indexPath?: string;
    watchAll?: boolean;
    customPaths?: string[];
};

// Start/stop the backend watcher and poll for events while an index is active, or when watchAll is true
export function useFileWatcher(options: WatcherOptions = {}) {
    const { indexPath, watchAll, customPaths } = options;
    const [active, setActive] = useState(false);
    const [events, setEvents] = useState<FileEvent[]>([]);
    const pollHandle = useRef<NodeJS.Timer | null>(null);
    const running = useRef(false);
    const latestPaths = useRef<string[] | undefined>(undefined);

    useEffect(() => {
        const shouldWatch = watchAll || !!indexPath;
        if (!shouldWatch) {
            setActive(false);
            setEvents([]);
            return;
        }

        // Derive a sensible watch root: parent of .wayfinder_index or the given path
        const watchRoot = indexPath
            ? (indexPath.includes('.wayfinder_index')
                ? indexPath.replace(/\\?\.wayfinder_index$/, '').replace(/\/\.wayfinder_index$/, '')
                : indexPath)
            : undefined;

        const derivedPaths = customPaths && customPaths.length > 0
            ? customPaths
            : (watchAll ? undefined : (watchRoot ? [watchRoot] : undefined));

        latestPaths.current = derivedPaths;
        startWatcher(derivedPaths);

        return () => {
            stopWatcher();
        };
    }, [indexPath, watchAll, JSON.stringify(customPaths)]);

    const startWatcher = useCallback(async (pathsOverride?: string[] | undefined) => {
        if (running.current) return;
        const effectivePaths = pathsOverride ?? latestPaths.current;
        try {
            await tauriService.startFileWatcher(effectivePaths);
            running.current = true;
            setActive(true);
        } catch (e) {
            // ignore start errors for now
        }

        if (pollHandle.current) clearInterval(pollHandle.current);
        pollHandle.current = setInterval(async () => {
            try {
                const res = await tauriService.getFileEvents(true);
                const nextEvents = (res as FileEventsResponse)?.events ?? (res as any);
                if (Array.isArray(nextEvents) && nextEvents.length > 0) {
                    setEvents(nextEvents);
                }
            } catch (e) {
                // ignore transient errors
            }
        }, 5000); // lighter polling to reduce memory/CPU load
    }, []);

    const stopWatcher = useCallback(async () => {
        if (pollHandle.current) {
            clearInterval(pollHandle.current);
            pollHandle.current = null;
        }
        running.current = false;
        setActive(false);
        try {
            await tauriService.stopFileWatcher();
        } catch (e) {
            // ignore
        }
    }, []);

    return { active, events, startWatcher, stopWatcher };
}
