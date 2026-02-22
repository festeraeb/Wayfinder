import React from "react";
import FileSuggestionCard from "./FileSuggestionCard";

export default function SuggestionPanel({ suggestions, onAccept, onDismiss }: { suggestions: any[]; onAccept: (id: string) => void; onDismiss: (id: string) => void }) {
    return (
        <div className="suggestion-panel">
            <div className="suggestion-header">
                <h3>Suggestions</h3>
                <div className="suggestion-actions">
                    <button className="btn btn-link">Accept All</button>
                    <button className="btn btn-link">Dismiss All</button>
                </div>
            </div>
            <div className="suggestion-list">
                {suggestions.map((s) => (
                    <FileSuggestionCard key={s.path} suggestion={s} onAccept={() => onAccept(s.path)} onDismiss={() => onDismiss(s.path)} />
                ))}
            </div>
        </div>
    );
}