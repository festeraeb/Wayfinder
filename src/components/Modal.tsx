import React from "react";
import "./GitAssistant.css"; // reuse modal styles

export default function Modal({
    visible,
    title,
    message,
    onConfirm,
    onCancel,
    confirmLabel = "Confirm",
    cancelLabel = "Cancel",
}: {
    visible: boolean;
    title?: string;
    message?: string;
    onConfirm?: () => void;
    onCancel?: () => void;
    confirmLabel?: string;
    cancelLabel?: string;
}) {
    if (!visible) return null;

    return (
        <div className="modal-overlay" onClick={onCancel}>
            <div className="modal" onClick={(e) => e.stopPropagation()}>
                <div className="modal-header">
                    <h3>{title || "Confirm"}</h3>
                </div>
                <div className="modal-content">
                    <p className="modal-description">{message}</p>
                </div>
                <div className="modal-footer">
                    <button className="btn btn-secondary" onClick={onCancel}>{cancelLabel}</button>
                    <button className="btn btn-primary" onClick={onConfirm}>{confirmLabel}</button>
                </div>
            </div>
        </div>
    );
}