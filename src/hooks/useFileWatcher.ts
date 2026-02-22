import { useEffect, useState } from "react";
import { tauriService } from "../services/tauri";
import type { FileEvent, FileEventsResponse } from "../types";

export function useFileWatcher(indexPath?: string) {
    const [active, setActive] = useState(false);
    const [events, setEvents] = useState<FileEvent[]>([]);

    useEffect(() => {
        let interval: any = null;
        if (indexPath) {
            setActive(true);
            interval = setInterval(async () => {
                try {
                    // Poll backend for new file events
                    const res = await tauriService.getFileEvents(true);
                    const nextEvents = (res as FileEventsResponse)?.events ?? (res as any);
                    if (Array.isArray(nextEvents) && nextEvents.length > 0) {
                        setEvents(nextEvents);
                    }
                } catch (e) {
                    // ignore
                }
            }, 2000);
        }
        return () => {
            if (interval) clearInterval(interval);
            setActive(false);
        };
    }, [indexPath]);

    return { active, events };
}
