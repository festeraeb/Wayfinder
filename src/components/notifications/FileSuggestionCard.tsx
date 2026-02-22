import React from "react";

export const FileSuggestionCard: React.FC<{ suggestion: any; onAccept?: () => void; onDismiss?: () => void }> = ({ suggestion, onAccept, onDismiss }) => {
    return (
        <div className="file-suggestion-card">
            <div className="file-suggestion-main">
                <div className="fs-title">{suggestion.name}</div>
                <div className="fs-path">{suggestion.path}</div>
            </div>
            <div className="file-suggestion-actions">
                <button className="btn btn-small" onClick={onAccept}>Accept</button>
                <button className="btn btn-link" onClick={onDismiss}>Dismiss</button>
            </div>
        </div>
    );
};

export default FileSuggestionCard;