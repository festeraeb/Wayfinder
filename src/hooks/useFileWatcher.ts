import { useEffect, useState } from "react";
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

        const paths = customPaths && customPaths.length > 0
            ? customPaths
            : (watchAll ? undefined : (watchRoot ? [watchRoot] : undefined));

        let pollHandle: any = null;

        const start = async () => {
            try {
                await tauriService.startFileWatcher(paths);
                setActive(true);
            } catch (e) {
                // ignore start errors for now
            }

            pollHandle = setInterval(async () => {
                try {
                    const res = await tauriService.getFileEvents(true);
                    const nextEvents = (res as FileEventsResponse)?.events ?? (res as any);
                    if (Array.isArray(nextEvents) && nextEvents.length > 0) {
                        setEvents(nextEvents);
                    }
                } catch (e) {
                    // ignore transient errors
                }
            }, 2000);
        };

        start();

        return () => {
            if (pollHandle) clearInterval(pollHandle);
            tauriService.stopFileWatcher().catch(() => {});
            setActive(false);
        };
    }, [indexPath, watchAll, JSON.stringify(customPaths)]);

    return { active, events };
}
