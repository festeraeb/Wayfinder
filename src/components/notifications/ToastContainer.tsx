import React from "react";
import Toast from "./Toast";
import "../../styles/notifications.css";

export default function ToastContainer({ notifications, onDismiss }: { notifications: any[]; onDismiss: (id: string) => void }) {
    return (
        <div className="toast-container bottom-right">
            {notifications.map((n) => (
                <Toast key={n.id} notification={n} onDismiss={onDismiss} />
            ))}
        </div>
    );
}
