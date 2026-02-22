import React, { createContext, useContext, useState, useCallback } from "react";
import ToastContainer from "./ToastContainer";
import { FileSuggestionCard } from "./FileSuggestionCard";

export type NotificationLevel = "info" | "success" | "warning" | "error";

export interface Notification {
    id: string;
    title?: string;
    message?: string;
    level?: NotificationLevel;
    data?: any;
    timeout?: number;
}

interface NotificationContextValue {
    notify: (n: Notification) => void;
    dismiss: (id: string) => void;
}

const NotificationContext = createContext<NotificationContextValue | null>(null);

export const useNotifications = () => {
    const ctx = useContext(NotificationContext);
    if (!ctx) throw new Error("useNotifications must be used within NotificationProvider");
    return ctx;
};

export const NotificationProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    const [notifications, setNotifications] = useState<Notification[]>([]);

    const notify = useCallback((n: Notification) => {
        setNotifications((prev) => [n, ...prev]);
        if (n.timeout && n.timeout > 0) {
            setTimeout(() => {
                setNotifications((prev) => prev.filter(x => x.id !== n.id));
            }, n.timeout);
        }
    }, []);

    const dismiss = useCallback((id: string) => {
        setNotifications((prev) => prev.filter(x => x.id !== id));
    }, []);

    return (
        <NotificationContext.Provider value={{ notify, dismiss }}>
            {children}
            <ToastContainer notifications={notifications} onDismiss={dismiss} />
        </NotificationContext.Provider>
    );
};

export default NotificationProvider;
