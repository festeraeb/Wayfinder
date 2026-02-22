import React from "react";

export default function WatcherStatusBadge({ active }: { active: boolean }) {
    return (
        <div className={`watcher-badge ${active ? 'active' : 'inactive'}`}>
            {active ? 'Watcher: On' : 'Watcher: Off'}
        </div>
    );
}