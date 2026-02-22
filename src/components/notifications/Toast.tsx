import React from "react";

export default function Toast({ notification, onDismiss }: { notification: any; onDismiss: (id: string) => void }) {
    const { id, title, message, level } = notification;

    return (
        <div className={`toast ${level || "info"}`} role="status">
            <div className="toast-body">
                <strong>{title}</strong>
                <div className="toast-message">{message}</div>
            </div>
            <div className="toast-actions">
                <button className="btn btn-link" onClick={() => onDismiss(id)}>Dismiss</button>
            </div>
        </div>
    );
}
