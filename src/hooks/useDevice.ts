// Device detection and tablet-specific hooks
import { useEffect, useState } from "react";
import { tauriService } from "../services/tauri";
import * as Types from "../types";

export type DeviceType = "desktop" | "tablet" | "learning-pad" | "unknown";

export function useDeviceDetection() {
    const [deviceType, setDeviceType] = useState<DeviceType>("unknown");
    const [systemInfo, setSystemInfo] = useState<Types.SystemInfo | null>(null);

    useEffect(() => {
        const detectDevice = async () => {
            try {
                const info = await tauriService.getSystemInfo();
                setSystemInfo(info as Types.SystemInfo);

                // Simple device detection based on screen size and system info
                const screenWidth = window.innerWidth;
                const isMobile = screenWidth < 600;
                const isTablet = screenWidth >= 600 && screenWidth < 1024;

                if (info.device_type === "tablet" || isTablet) {
                    setDeviceType("tablet");
                } else if (info.device_type === "learning-pad") {
                    setDeviceType("learning-pad");
                } else if (isMobile) {
                    setDeviceType("tablet"); // Treat phones as tablets for UI purposes
                } else {
                    setDeviceType("desktop");
                }
            } catch (error) {
                console.error("Failed to detect device:", error);
                setDeviceType("desktop");
            }
        };

        detectDevice();

        // Re-detect on window resize
        const handleResize = () => {
            detectDevice();
        };

        window.addEventListener("resize", handleResize);
        return () => window.removeEventListener("resize", handleResize);
    }, []);

    return { deviceType, systemInfo };
}

export function useTabletMode() {
    const [isTabletMode, setIsTabletMode] = useState(false);

    useEffect(() => {
        // Check if running on a tablet/learning device
        const savedMode = localStorage.getItem("tablet-mode");
        if (savedMode) {
            setIsTabletMode(JSON.parse(savedMode));
        }
    }, []);

    const toggleTabletMode = (enabled: boolean) => {
        setIsTabletMode(enabled);
        localStorage.setItem("tablet-mode", JSON.stringify(enabled));
    };

    return { isTabletMode, toggleTabletMode };
}

export function useOfflineCapability() {
    const [isOffline, setIsOffline] = useState(false);
    const [cachedIndex, setCachedIndex] = useState<any>(null);

    useEffect(() => {
        // Detect offline status
        const handleOnline = () => setIsOffline(false);
        const handleOffline = () => setIsOffline(true);

        window.addEventListener("online", handleOnline);
        window.addEventListener("offline", handleOffline);

        // Set initial state
        setIsOffline(!navigator.onLine);

        return () => {
            window.removeEventListener("online", handleOnline);
            window.removeEventListener("offline", handleOffline);
        };
    }, []);

    return { isOffline, cachedIndex, setCachedIndex };
}
