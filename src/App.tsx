import { useState, useEffect } from "react";
import { tauriService } from "./services/tauri";
import { useTheme } from "./hooks/useTauri";
import { open } from "@tauri-apps/plugin-dialog";
import { GitAssistant } from "./components/GitAssistant";
import { NautiClippy } from "./components/NautiClippy";
import { useFileWatcher } from "./hooks/useFileWatcher";
import { useNotifications } from "./components/notifications/NotificationProvider";
import Modal from "./components/Modal";
import "./styles/global.css";
import "./styles/app.css";
import "./styles/panels.css";

// File type presets
const FILE_TYPE_PRESETS = {
    "Markdown": [".md", ".mdx", ".markdown"],
    "Text": [".txt", ".text", ".log"],
    "Python": [".py", ".pyw", ".pyi"],
    "JavaScript": [".js", ".jsx", ".ts", ".tsx"],
    "Documents": [".pdf", ".doc", ".docx", ".odt"],
    "Config": [".json", ".yaml", ".yml", ".toml", ".ini"],
    "Web": [".html", ".htm", ".css", ".scss"],
    "All Text Files": ["*"],
    "All Files": ["*"],
};

type OperationStatus = "idle" | "running" | "complete" | "error";
type ActiveSection = "status" | "scan" | "embed" | "cluster" | "search" | "timeline" | "git" | "nauti" | "org";

interface Reminder {
    id: string;
    title: string;
    due?: string;
    severity?: string;
    link_path?: string;
    repo_path?: string;
    status: string;
    created_at: string;
    updated_at: string;
}

interface ClassifiedFile {
    path: string;
    project: string;
    confidence: number;
    reasons: string[];
    alternates: [string, number][];
}

interface ScanResult {
    files_scanned: number;
    total_size: number;
    index_path: string;
}

interface IndexStats {
    total_files: number;
    total_size_bytes: number;
    extensions: Record<string, number>;
    last_updated: string;
    scan_path?: string;
    // These will be false until we implement embeddings/clustering
    has_embeddings?: boolean;
    has_clusters?: boolean;
    cluster_count?: number;
    embeddings_count?: number;
}

export default function App() {
    const { isDark, toggleTheme } = useTheme();

    // Navigation state
    const [activeSection, setActiveSection] = useState<ActiveSection>("status");

    // Index configuration
    const [indexPath, setIndexPath] = useState<string>("");
    const [scanPath, setScanPath] = useState<string>("");
    const [selectedTypes, setSelectedTypes] = useState<string[]>(["Markdown"]);
    const [gitRepoPath, setGitRepoPath] = useState<string>("");

    // Reminders
    const [reminders, setReminders] = useState<Reminder[]>([]);
    const [newReminderTitle, setNewReminderTitle] = useState("");
    const [newReminderDays, setNewReminderDays] = useState<number>(1);

        // Org/classify
        const [labelSpec, setLabelSpec] = useState<string>(`{
    "projectA": ["sonar", "sniffer"],
    "projectB": ["bag", "rosbag"]
}`);
        const [rulesSpec, setRulesSpec] = useState<string>(`{
    "min_confidence": 0.8,
    "ambiguity_delta": 0.3
}`);
        const [classified, setClassified] = useState<ClassifiedFile[]>([]);
        const [viewRoot, setViewRoot] = useState<string>("Wayfinder/View");
        const [viewPlan, setViewPlan] = useState<any[]>([]);
        const [viewErrors, setViewErrors] = useState<string[]>([]);
        const [viewDryRun, setViewDryRun] = useState<boolean>(true);
        const [useSmartClassifier, setUseSmartClassifier] = useState<boolean>(false);
        const [smartLlmLimit, setSmartLlmLimit] = useState<number>(10);
        const [smartLlmMinScore, setSmartLlmMinScore] = useState<number>(1.2);
        const [smartLlmTopAlts, setSmartLlmTopAlts] = useState<number>(2);
        const [smartLlmEndpoint, setSmartLlmEndpoint] = useState<string>("http://localhost:5001");
        const [smartLlmModel, setSmartLlmModel] = useState<string>("qwen2.5-coder-14b-instruct");

    // Status/Stats state
    const [indexStats, setIndexStats] = useState<IndexStats | null>(null);

    // Scan state
    const [scanStatus, setScanStatus] = useState<OperationStatus>("idle");
    const [scanProgress, setScanProgress] = useState({ current: 0, total: 0, percent: 0 });
    const [scanResult, setScanResult] = useState<ScanResult | null>(null);

    // Embed state
    const [embedStatus, setEmbedStatus] = useState<OperationStatus>("idle");
    const [embedProgress, setEmbedProgress] = useState(0);
    const [embedProgressDetail, setEmbedProgressDetail] = useState<{ processed: number; total: number; status: string; currentBatch?: number; totalBatches?: number; batchSize?: number } | null>(null);
    const [embedResult, setEmbedResult] = useState<any>(null);
    const [embedMaxFiles, setEmbedMaxFiles] = useState<number | undefined>(undefined);
    const [embedBatchSize, setEmbedBatchSize] = useState<number | undefined>(undefined);

    // Embedding provider config state
    const [embeddingProvider, setEmbeddingProvider] = useState<"local" | "llama" | "azure" | "gcp" | "multi">("llama");
    const [localModel, setLocalModel] = useState("embeddinggemma-300m-f16");
    const [localEndpoint, setLocalEndpoint] = useState("http://localhost:5002");

    // Global quick-config modal
    const [showGlobalConfig, setShowGlobalConfig] = useState(false);
    const [llamaChatEndpoint, setLlamaChatEndpoint] = useState("http://localhost:5001");
    const [llamaChatModel, setLlamaChatModel] = useState("qwen2.5-coder-1.5b-instruct-q4_k_m.gguf");
    
    // Azure state
    const [azureConfigured, setAzureConfigured] = useState(false);
    const [azureEndpoint, setAzureEndpoint] = useState("");
    const [azureApiKey, setAzureApiKey] = useState("");
    const [azureDeployment, setAzureDeployment] = useState("text-embedding-3-small");
    const [azureApiVersion, setAzureApiVersion] = useState("");
    const [showAzureConfig, setShowAzureConfig] = useState(false);
    const [hasExistingKey, setHasExistingKey] = useState(false);

    // GCP state
    const [gcpConfigured, setGcpConfigured] = useState(false);
    const [gcpProjectId, setGcpProjectId] = useState("");
    const [gcpLocation, setGcpLocation] = useState("us-central1");
    const [gcpModelId, setGcpModelId] = useState("text-embedding-004");
    const [gcpServiceAccountPath, setGcpServiceAccountPath] = useState("");
    const [gcpEndpoint, setGcpEndpoint] = useState("");
    const [showGcpConfig, setShowGcpConfig] = useState(false);
    const [hasExistingGcpKey, setHasExistingGcpKey] = useState(false);
    const [useGcpAdc, setUseGcpAdc] = useState(false); // Allow gcloud ADC instead of key file

    // Cluster state
    const [clusterStatus, setClusterStatus] = useState<OperationStatus>("idle");
    const [numClusters, setNumClusters] = useState<number | undefined>(undefined);
    const [clusters, setClusters] = useState<any[]>([]);

    // Search state
    const [searchQuery, setSearchQuery] = useState("");
    const [searchResults, setSearchResults] = useState<any[]>([]);
    const [topK, setTopK] = useState(10);
    const [semanticWeight, setSemanticWeight] = useState(0.7);

    // Timeline state
    const [timelineDays, setTimelineDays] = useState(30);
    const [timelineData, setTimelineData] = useState<any[]>([]);

    // File watcher integration (notifications)
    // Using hooks and notification provider to surface file events
    const [watchAll, setWatchAll] = useState(false);
    const { active: watcherActive, events: watcherEvents, startWatcher, stopWatcher } = useFileWatcher({ indexPath, watchAll });
    const { notify } = useNotifications();

    // Show toast for new watcher events
    useEffect(() => {
        if (!watcherEvents || watcherEvents.length === 0) return;
        watcherEvents.slice(0,3).forEach((e:any) => {
            try {
                notify({ id: `evt-${Date.now()}-${Math.random()}`, title: 'File Event', message: `${e.event}: ${e.path}`, level: 'info', timeout: 8000 });
            } catch (err) {
                // ignore if notification system not ready
            }
        });
    }, [watcherEvents]);
    // Error state
    const [errorMsg, setErrorMsg] = useState<string>("");

    // Theme setup
    useEffect(() => {
        document.documentElement.setAttribute("data-theme", isDark ? "dark" : "light");
    }, [isDark]);

    // Load reminders on mount
    useEffect(() => {
        loadReminders();
    }, []);

    // Load stats on mount and when switching to status
    useEffect(() => {
        if (activeSection === "status" && indexPath) {
            loadStats();
        }
    }, [activeSection, indexPath]);

    // Update azureApiVersion when loading config
    const loadAzureConfig = async (dir: string) => {
        try {
            const config = await tauriService.loadAzureConfig(dir);
            setAzureConfigured(config.configured);
            setHasExistingKey(config.has_key || false);
            if (config.endpoint) setAzureEndpoint(config.endpoint);
            if (config.deployment_name) setAzureDeployment(config.deployment_name);
            if (config.api_version) setAzureApiVersion(config.api_version);
            setAzureApiKey("");
        } catch (error) {
            console.log("No Azure config found");
            setHasExistingKey(false);
        }
    };

    const addReminder = async () => {
        if (!newReminderTitle.trim()) return;
        const due = new Date(Date.now() + newReminderDays * 24 * 60 * 60 * 1000)
            .toISOString()
            .replace("T", " ")
            .slice(0, 19);
        try {
            await tauriService.addReminder(newReminderTitle, due, undefined, undefined, gitRepoPath || undefined);
            setNewReminderTitle("");
            loadReminders();
        } catch (e: any) {
            setErrorMsg(e.toString());
        }
    };

    const markReminderDone = async (id: string) => {
        try {
            await tauriService.updateReminderStatus(id, "done");
            loadReminders();
        } catch (e: any) {
            setErrorMsg(e.toString());
        }
    };

    const snoozeReminder = async (id: string, hours: number) => {
        try {
            await tauriService.snoozeReminder(id, hours);
            loadReminders();
        } catch (e: any) {
            setErrorMsg(e.toString());
        }
    };

    const loadGcpConfig = async (dir: string) => {
        try {
            const config = await tauriService.loadGcpConfig(dir);
            setGcpConfigured(config.configured);
            setHasExistingGcpKey(config.has_key || false);
            if (config.project_id) setGcpProjectId(config.project_id);
            if (config.location) setGcpLocation(config.location);
            if (config.model_id) setGcpModelId(config.model_id);
            if (config.endpoint) setGcpEndpoint(config.endpoint);
            setGcpServiceAccountPath(""); // Clear local input; we never echo stored keys
        } catch (error) {
            console.log("No GCP config found");
            setHasExistingGcpKey(false);
        }
    };

    const handleClassify = async () => {
        if (!indexPath) {
            setErrorMsg("No index available. Scan first.");
            return;
        }
        try {
            const labels = JSON.parse(labelSpec);
            const rules = rulesSpec.trim() ? JSON.parse(rulesSpec) : {};
            const res = await tauriService.classifyFiles(
                indexPath,
                labels,
                undefined,
                rules,
                useSmartClassifier,
                smartLlmModel,
                smartLlmEndpoint,
                smartLlmLimit,
                smartLlmMinScore,
                smartLlmTopAlts
            );
            setClassified(res.classified || []);
            setErrorMsg("");
        } catch (e: any) {
            setErrorMsg("Classify failed: " + e.toString());
        }
    };

    const handleBuildView = async () => {
        if (!classified || classified.length === 0) {
            setErrorMsg("Run classify first.");
            return;
        }
        try {
            const res = await tauriService.buildViewPlan(classified, viewRoot, true, viewDryRun);
            setViewPlan(res.plan || []);
            setViewErrors(res.errors || []);
            setErrorMsg(res.success ? "" : "View plan completed with errors");
        } catch (e: any) {
            setErrorMsg("View plan failed: " + e.toString());
        }
    };

    const loadProviderConfig = async (dir: string) => {
        try {
            const provider = await tauriService.loadProviderConfig(dir);
            if (provider.provider) {
                setEmbeddingProvider(provider.provider as any);
            }
            if (provider.local_model) {
                setLocalModel(provider.local_model);
            } else {
                setLocalModel("embeddinggemma-300m-f16");
            }
            if (provider.local_endpoint) {
                setLocalEndpoint(provider.local_endpoint);
            } else {
                setLocalEndpoint("http://localhost:5002");
            }
            if (provider.provider === "azure") {
                await loadAzureConfig(dir);
            } else if (provider.provider === "gcp") {
                await loadGcpConfig(dir);
            } else {
                setAzureConfigured(true); // Local is always configured
                setGcpConfigured(true);
            }
        } catch (error) {
            setEmbeddingProvider("llama");
            setLocalModel("embeddinggemma-300m-f16");
            setLocalEndpoint("http://localhost:5002");
        }
    };
        
    useEffect(() => {
        if (!indexPath) return;

        const isOperationRunning = embedStatus === "running" || clusterStatus === "running" || scanStatus === "running";
        const pollInterval = isOperationRunning ? 2000 : (activeSection === "status" ? 5000 : 0);

        if (pollInterval === 0) return;

        const interval = setInterval(() => {
            loadStats();
        }, pollInterval);

        return () => clearInterval(interval);
    }, [indexPath, embedStatus, clusterStatus, scanStatus, activeSection]);

    // Default Git repo path to the scan folder (or index folder root) if not set
    useEffect(() => {
        if (!gitRepoPath) {
            if (scanPath) {
                setGitRepoPath(scanPath);
            } else if (indexPath && indexPath.includes('.wayfinder_index')) {
                const root = indexPath.replace(/\\?\.wayfinder_index$/, '').replace(/\/\.wayfinder_index$/, '');
                setGitRepoPath(root);
            }
        }
    }, [scanPath, indexPath, gitRepoPath]);

    // Toggle file type selection ("All Files" is exclusive)
    const toggleFileType = (type: string) => {
        setSelectedTypes(prev => {
            const isAll = type === "All Files";
            const hadAll = prev.includes("All Files");

            if (isAll) {
                return ["All Files"];
            }

            const next = prev.includes(type)
                ? prev.filter(t => t !== type)
                : [...prev.filter(t => t !== "All Files"), type];

            // If we removed the last item, fall back to the clicked one
            if (next.length === 0) return [type];
            // Ensure "All Files" is exclusive
            if (hadAll) return next.filter(t => t !== "All Files");
            return next;
        });
    };

    // Get all selected extensions
    const getSelectedExtensions = (): string[] => {
        const extensions: string[] = [];
        selectedTypes.forEach(type => {
            const exts = FILE_TYPE_PRESETS[type as keyof typeof FILE_TYPE_PRESETS];
            if (exts) extensions.push(...exts);
        });
        return [...new Set(extensions)];
    };

    // Load index stats
    const loadStats = async () => {
        if (!indexPath) return;
        try {
            const stats = await tauriService.getStats(indexPath);
            setIndexStats(stats);
        } catch (error) {
            console.error("Failed to load stats:", error);
        }
    };

    const loadReminders = async () => {
        try {
            const res = await tauriService.listReminders(false);
            setReminders(res.reminders || []);
        } catch (e) {
            console.error("Failed to load reminders", e);
        }
    };

    // Handle scan
    const handleScan = async () => {
        if (!scanPath.trim()) {
            setErrorMsg("Please enter a folder path to scan");
            return;
        }

        if (selectedTypes.length === 0) {
            setErrorMsg("Please select at least one file type");
            return;
        }

        // Use backslash for Windows paths
        const separator = scanPath.includes('\\') ? '\\' : '/';
        const effectiveIndexPath = indexPath || `${scanPath}${separator}.wayfinder_index`;

        setScanStatus("running");
        setErrorMsg("");
        setScanProgress({ current: 0, total: 0, percent: 0 });

        try {
            console.log("Starting scan:", scanPath, effectiveIndexPath);
            const selectedExts = getSelectedExtensions();
            const allFiles = selectedExts.includes("*");
            const result = await tauriService.scanDirectory(scanPath, effectiveIndexPath, selectedExts, allFiles);
            console.log("Scan result:", result);
            setScanResult(result);
            setIndexPath(effectiveIndexPath);
            setScanStatus("complete");
            loadStats();
            loadProviderConfig(effectiveIndexPath);
        } catch (error: any) {
            console.error("Scan error:", error);
            setScanStatus("error");
            setErrorMsg(error.toString());
        }
    };



    // Save Azure config
    const [showValidationModal, setShowValidationModal] = useState(false);
    const [validationMessage, setValidationMessage] = useState("");
    const [validationSuggested, setValidationSuggested] = useState<string | null>(null);

    const formatValidationMessage = (validation: any) => {
        const base = validation?.message || "Validation failed";
        if (validation?.final_url) {
            return `${base}\nURL: ${validation.final_url}`;
        }
        return base;
    };

    const saveEmbeddingConfig = async () => {
        if (!indexPath) {
            setErrorMsg("Please scan a folder first to set the index location");
            return;
        }

        if (embeddingProvider === "local") {
            try {
                await tauriService.saveProviderConfig(indexPath, "local", localModel || undefined, localEndpoint || undefined);
                setErrorMsg("");
            } catch (error: any) {
                setErrorMsg(error.toString());
            }
            return;
        }

        if (embeddingProvider === "gcp") {
            if (!gcpProjectId || !gcpLocation || !gcpModelId) {
                setErrorMsg("Please fill in Project ID, Location, and Model ID");
                return;
            }
            if (!useGcpAdc && !gcpServiceAccountPath && !hasExistingGcpKey) {
                setErrorMsg("Select a service account JSON file or enable gcloud ADC");
                return;
            }

            try {
                if (!useGcpAdc) {
                    const validation = await tauriService.validateGcpConfig(
                        gcpProjectId,
                        gcpLocation,
                        gcpModelId,
                        gcpServiceAccountPath || "",
                        gcpEndpoint || undefined
                    );
                    if (!validation || !validation.success) {
                        setErrorMsg(validation?.message || "GCP validation failed");
                        return;
                    }
                }

                await tauriService.saveGcpConfig(
                    indexPath,
                    gcpProjectId,
                    gcpLocation,
                    gcpModelId,
                    useGcpAdc ? "" : gcpServiceAccountPath,
                    gcpEndpoint || undefined
                );
                await tauriService.saveProviderConfig(indexPath, "gcp", localModel || undefined, undefined);
                await loadProviderConfig(indexPath); // refresh state to confirm persistence
                setHasExistingGcpKey(!useGcpAdc && !!gcpServiceAccountPath);
                setGcpConfigured(true);
                setShowGcpConfig(false);
                setErrorMsg("");
            } catch (error: any) {
                setErrorMsg(error.toString());
            }
            return;
        }

        // Azure logic
        if (!azureEndpoint || !azureDeployment) {
            setErrorMsg("Please fill in endpoint and deployment name");
            return;
        }
        if (!azureApiKey && !hasExistingKey) {
            setErrorMsg("Please enter your API key");
            return;
        }

        try {
            // Validate configuration before saving
            const validation = await tauriService.validateAzureConfig(indexPath, azureEndpoint, azureApiKey || "", azureDeployment, azureApiVersion || undefined);

            if (validation && validation.success) {
                await tauriService.saveAzureConfig(
                    indexPath,
                    azureEndpoint,
                    azureApiKey,
                    azureDeployment,
                    azureApiVersion || undefined
                );
                await tauriService.saveProviderConfig(indexPath, "azure", localModel || undefined, undefined);
                setAzureConfigured(true);
                setShowAzureConfig(false);
                setErrorMsg("");
            } else {
                // Show validation message and offer suggestion if available
                const msg = formatValidationMessage(validation);
                if (validation?.suggested_endpoint) {
                    // Store suggestion and open modal instead of window.confirm
                    setValidationMessage(msg + `\nSuggested endpoint: ${validation.suggested_endpoint}`);
                    setValidationSuggested(validation.suggested_endpoint);
                    setShowValidationModal(true);
                } else {
                    alert(`Validation failed: ${msg}`);
                    setErrorMsg(msg);
                }
            }
        } catch (error: any) {
            setErrorMsg(error.toString());
        }
    };

    // Confirm modal handlers
    const applySuggestedEndpoint = async () => {
        if (!validationSuggested) return;
        try {
            await tauriService.saveAzureConfig(
                indexPath,
                validationSuggested,
                azureApiKey,
                azureDeployment,
                azureApiVersion || undefined
            );
            await tauriService.saveProviderConfig(indexPath, "azure", localModel || undefined, undefined);
            setAzureEndpoint(validationSuggested);
            setAzureConfigured(true);
            setShowAzureConfig(false);
            setShowValidationModal(false);
            setValidationSuggested(null);
            setValidationMessage("");
            setErrorMsg("");
        } catch (e: any) {
            setErrorMsg(e.toString());
        }
    };

    const cancelSuggestedEndpoint = () => {
        setShowValidationModal(false);
        setValidationSuggested(null);
        setValidationMessage("");
    };

    const handleGlobalSave = async () => {
        await saveEmbeddingConfig();
        setShowGlobalConfig(false);
    };

    // Validation results UI state
    const [validationResults, setValidationResults] = useState<any[]>([]);
    const [showValidationResults, setShowValidationResults] = useState(false);

    // Handle embed
    const handleEmbed = async () => {
        if (!indexPath) {
            setErrorMsg("No index available. Please scan a folder first.");
            return;
        }

        if (embeddingProvider === "azure" && !azureConfigured) {
            setShowAzureConfig(true);
            setErrorMsg("Please configure Azure first.");
            return;
        }

        if (embeddingProvider === "gcp" && !gcpConfigured) {
            setShowGcpConfig(true);
            setErrorMsg("Please configure GCP first.");
            return;
        }

        // Persist current provider choice so the backend doesn't fall back to Azure/local defaults
        if (embeddingProvider !== "multi") {
            try {
                await tauriService.saveProviderConfig(indexPath, embeddingProvider, localModel || undefined, localEndpoint || undefined);
            } catch (e: any) {
                setErrorMsg(e.toString());
                return;
            }
        }

        setEmbedStatus("running");
        setEmbedProgress(0);
        setEmbedProgressDetail(null);
        setErrorMsg("");
        setEmbedResult(null);

        // Start polling progress
        let pollHandle: any = null;
        try {
            pollHandle = setInterval(async () => {
                try {
                    const p = await tauriService.getEmbeddingProgress(indexPath);
                    if (p && p.total_files > 0) {
                        const percent = Math.round((p.processed_files / Math.max(1, p.total_files)) * 100);
                        setEmbedProgress(percent);
                        setEmbedProgressDetail({
                            processed: p.processed_files,
                            total: p.total_files,
                            status: p.status || "running",
                            currentBatch: p.current_batch,
                            totalBatches: p.total_batches,
                            batchSize: p.batch_size,
                        });
                        if (p.status == "complete") {
                            // finalize
                            setEmbedProgress(100);
                            setEmbedProgressDetail({
                                processed: p.total_files,
                                total: p.total_files,
                                status: "complete",
                                currentBatch: p.total_batches,
                                totalBatches: p.total_batches,
                                batchSize: p.batch_size,
                            });
                            setEmbedStatus("complete");
                            clearInterval(pollHandle);
                        }
                    }
                } catch (e) {
                    // ignore transient errors
                }
            }, 1000);

            const result = embeddingProvider === "multi"
                ? await tauriService.generateEmbeddingsMulti(indexPath, embedMaxFiles, embedBatchSize)
                : await tauriService.generateEmbeddings(indexPath, embedMaxFiles, embedBatchSize);
            console.log("Embed result:", result);
            setEmbedResult(result);
            setEmbedProgress(100);
            setEmbedProgressDetail(prev => prev ? { ...prev, processed: prev.total, status: "complete" } : { processed: 1, total: 1, status: "complete" });
            setEmbedStatus("complete");
            loadStats();
        } catch (error: any) {
            setEmbedStatus("error");
            setErrorMsg(error.toString());
        } finally {
            if (pollHandle) clearInterval(pollHandle);
        }
    };

    const handleCancelEmbed = async () => {
        if (!indexPath) return;
        try {
            await tauriService.cancelEmbedding(indexPath);
            setEmbedStatus("idle");
            setEmbedProgressDetail(prev => prev ? { ...prev, status: "cancelled" } : null);
        } catch (e: any) {
            setErrorMsg("Failed to cancel: " + e.toString());
        }
    };

    // Handle cluster
    const handleCluster = async () => {
        if (!indexPath) {
            setErrorMsg("No index available. Please scan a folder first.");
            return;
        }

        setClusterStatus("running");
        setErrorMsg("");

        try {
            await tauriService.createClusters(indexPath, numClusters);
            const clusterData = await tauriService.getClustersSummary(indexPath);
            setClusters(clusterData.clusters || []);
            setClusterStatus("complete");
            loadStats();
        } catch (error: any) {
            setClusterStatus("error");
            setErrorMsg(error.toString());
        }
    };

    // Handle search
    const handleSearch = async () => {
        if (!searchQuery.trim() || !indexPath) return;

        try {
            const results = await tauriService.search(searchQuery, indexPath, topK, semanticWeight);
            setSearchResults(results);
        } catch (error) {
            console.error("Search error:", error);
        }
    };

    // Handle timeline
    const handleTimeline = async () => {
        if (!indexPath) return;

        try {
            const data = await tauriService.getTimeline(indexPath, timelineDays);
            setTimelineData(data.timeline || []);
        } catch (error) {
            console.error("Timeline error:", error);
        }
    };

    // Navigation items
    const navItems: { id: ActiveSection; icon: string; label: string }[] = [
        { id: "status", icon: "📊", label: "Status" },
        { id: "scan", icon: "📁", label: "Scan" },
        { id: "embed", icon: "🧠", label: "Embeddings" },
        { id: "cluster", icon: "🗂️", label: "Clusters" },
        { id: "search", icon: "🔍", label: "Search" },
        { id: "timeline", icon: "📅", label: "Timeline" },
        { id: "git", icon: "📎", label: "Git Clippy" },
        { id: "nauti", icon: "🧭", label: "Nauti-Clippy" },
        { id: "org", icon: "🗃️", label: "Org" },
    ];

    return (
        <div className="app-container">
            {/* Sidebar Navigation */}
            <aside className="sidebar">
                <div className="sidebar-header">
                    <h1>⛵🐕 Wayfinder</h1>
                    <p className="tagline">by NautiDog</p>
                </div>

                <nav className="sidebar-nav">
                    {navItems.map(item => (
                        <button
                            key={item.id}
                            className={`nav-item ${activeSection === item.id ? "active" : ""}`}
                            onClick={() => setActiveSection(item.id)}
                        >
                            <span className="nav-icon">{item.icon}</span>
                            <span className="nav-label">{item.label}</span>
                        </button>
                    ))}
                </nav>

                <div className="sidebar-footer">
                    <button className="theme-toggle" onClick={toggleTheme} title="Toggle theme">
                        {isDark ? "☀️ Light Mode" : "🌙 Dark Mode"}
                    </button>
                </div>
            </aside>

            {/* Main Content */}
            <main className="main-content">
                {/* Top Stats Bar */}
                <div className="top-bar">
                    <div className="stats-summary">
                        <div className="stat-chip">
                            <span className="stat-label">Files</span>
                            <span className="stat-value">{indexStats?.total_files || 0}</span>
                        </div>
                        <div className="stat-chip">
                            <span className="stat-label">Embeddings</span>
                            <span className="stat-value">{indexStats?.has_embeddings ? "✓" : "—"}</span>
                        </div>
                        <div className="stat-chip">
                            <span className="stat-label">Clusters</span>
                            <span className="stat-value">{indexStats?.cluster_count || 0}</span>
                        </div>
                    </div>
                    <div className="top-actions">
                        {indexPath && (
                            <div className="index-path-display">
                                📁 {indexPath}
                                <button
                                    className="btn btn-small"
                                    style={{ marginLeft: '0.5rem' }}
                                    onClick={async () => {
                                        try {
                                            const selected = await open({ directory: true, multiple: false, title: 'Select index or scan folder' });
                                            if (selected && typeof selected === 'string') {
                                                const path = selected as string;
                                                const candidateIndex = path.replace(/\/+$/, '') + '/.wayfinder_index';
                                                // Prefer .wayfinder_index inside the selected folder, otherwise the folder itself
                                                let chosen = path;
                                                try {
                                                    const v1 = await tauriService.validateIndex(candidateIndex);
                                                    if (v1 && v1.index_valid) {
                                                        chosen = candidateIndex;
                                                    } else {
                                                        const v2 = await tauriService.validateIndex(path);
                                                        if (v2 && v2.index_valid) {
                                                            chosen = path;
                                                        }
                                                    }
                                                } catch (e) {
                                                    // ignore validation errors and just set selected
                                                    chosen = path;
                                                }

                                                setIndexPath(chosen);
                                                // Refresh stats and config
                                                await loadStats();
                                                await loadProviderConfig(chosen);
                                            }
                                        } catch (err) {
                                            console.error('Choose index error', err);
                                        }
                                    }}
                                >
                                    📂 Choose
                                </button>
                            </div>
                        )}
                        <button
                            className="icon-button"
                            title="Configure LLM & embeddings"
                            onClick={() => setShowGlobalConfig(true)}
                        >
                            ⚙️
                        </button>
                    </div>
                </div>

                {/* Error Display */}
                {errorMsg && (
                    <div className="error-banner">
                        ❌ {errorMsg}
                        <button onClick={() => setErrorMsg("")}>✕</button>
                    </div>
                )}

                {/* Status Section */}
                {activeSection === "status" && (
                    <section className="content-section">
                        <h2>📊 Index Status</h2>
                        {/* Reminders panel */}
                        <div className="reminders-panel" style={{ marginBottom: '1rem' }}>
                            <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center', flexWrap: 'wrap' }}>
                                <input
                                    type="text"
                                    placeholder="Add a reminder..."
                                    value={newReminderTitle}
                                    onChange={(e) => setNewReminderTitle(e.target.value)}
                                    style={{ flex: '1 1 240px' }}
                                />
                                <select value={newReminderDays} onChange={(e) => setNewReminderDays(parseInt(e.target.value))}>
                                    <option value={1}>Tomorrow</option>
                                    <option value={3}>+3 days</option>
                                    <option value={7}>+7 days</option>
                                </select>
                                <button className="btn btn-secondary" onClick={addReminder}>➕ Save</button>
                            </div>
                            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem', marginTop: '0.75rem' }}>
                                {(() => {
                                    const now = Date.now();
                                    const important = reminders.filter(r => {
                                        const dueTs = r.due ? Date.parse(r.due.replace(' ', 'T') + 'Z') : 0;
                                        return (r.severity === 'high') || (dueTs && dueTs < now);
                                    });
                                    const later = reminders.filter(r => !important.includes(r));
                                    const renderList = (items: Reminder[], label: string) => (
                                        <div className="reminder-group" style={{ border: '1px solid var(--border-color)', borderRadius: 'var(--radius-md)', padding: '0.75rem' }}>
                                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                                                <h4 style={{ margin: 0 }}>{label}</h4>
                                                <span style={{ color: 'var(--text-secondary)' }}>{items.length}</span>
                                            </div>
                                            {items.length === 0 ? (
                                                <p style={{ color: 'var(--text-secondary)' }}>Nothing here.</p>
                                            ) : (
                                                <div className="reminder-list" style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', marginTop: '0.5rem' }}>
                                                    {items.map(r => (
                                                        <div key={r.id} className="reminder-card" style={{ border: '1px solid var(--border-color)', borderRadius: 'var(--radius-sm)', padding: '0.5rem' }}>
                                                            <div style={{ display: 'flex', justifyContent: 'space-between', gap: '0.5rem', alignItems: 'center' }}>
                                                                <div>
                                                                    <strong>{r.title}</strong>
                                                                    {r.due && <div style={{ color: 'var(--text-secondary)', fontSize: '0.85rem' }}>Due: {r.due}</div>}
                                                                    {r.repo_path && <div style={{ color: 'var(--text-secondary)', fontSize: '0.85rem' }}>Repo: {r.repo_path}</div>}
                                                                </div>
                                                                <div style={{ display: 'flex', gap: '0.25rem' }}>
                                                                    <button className="btn btn-small" onClick={() => snoozeReminder(r.id, 24)}>+1d</button>
                                                                    <button className="btn btn-small" onClick={() => snoozeReminder(r.id, 72)}>+3d</button>
                                                                    <button className="btn btn-small" onClick={() => snoozeReminder(r.id, 168)}>+7d</button>
                                                                    <button className="btn btn-small" onClick={() => markReminderDone(r.id)}>Done</button>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    ))}
                                                </div>
                                            )}
                                        </div>
                                    );
                                    return (
                                        <>
                                            {renderList(important, 'Important')}
                                            {renderList(later, 'Later')}
                                        </>
                                    );
                                })()}
                            </div>
                        </div>
                        {!indexPath ? (
                            <div className="empty-state">
                                <p>No index loaded. Go to <strong>Scan</strong> to create one.</p>
                                <button className="btn btn-primary" onClick={() => setActiveSection("scan")}>
                                    📁 Start Scanning
                                </button>
                            </div>
                        ) : (
                            <div className="status-grid">
                                <div className="status-card">
                                    <h3>Files Indexed</h3>
                                    <span className="big-number">{indexStats?.total_files || 0}</span>
                                </div>
                                <div className="status-card">
                                    <h3>Embeddings</h3>
                                    <span className={`status-badge ${indexStats?.has_embeddings ? "success" : "pending"}`}>
                                        {indexStats?.has_embeddings ? "Generated" : "Not Generated"}
                                    </span>
                                </div>
                                <div className="status-card">
                                    <h3>Clusters</h3>
                                    <span className={`status-badge ${indexStats?.has_clusters ? "success" : "pending"}`}>
                                        {indexStats?.has_clusters ? `${indexStats.cluster_count} Clusters` : "Not Created"}
                                    </span>
                                </div>
                                <div className="status-card">
                                    <h3>Last Scan</h3>
                                    <span>{indexStats?.last_updated || "Unknown"}</span>
                                </div>
                                <div className="status-card">
                                    <h3>File Watcher</h3>
                                    <span className={`status-badge ${watcherActive ? "success" : "pending"}`}>
                                        {watcherActive ? "Running" : "Stopped"}
                                    </span>
                                    <div style={{ marginTop: '8px' }}>
                                        <label style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                                            <input
                                                type="checkbox"
                                                checked={watchAll}
                                                onChange={(e) => setWatchAll(e.target.checked)}
                                            />
                                            Watch all user folders
                                        </label>
                                        <small style={{ display: 'block', marginTop: '4px', color: 'var(--text-secondary)' }}>
                                            Uses default user folders (Desktop/Documents/Downloads). Uncheck to watch only the current index folder.
                                        </small>
                                        <div style={{ marginTop: '8px', display: 'flex', gap: '0.5rem' }}>
                                            <button className="btn btn-secondary btn-small" onClick={() => startWatcher()} disabled={watcherActive}>
                                                ▶ Start watcher
                                            </button>
                                            <button className="btn btn-secondary btn-small" onClick={() => stopWatcher()} disabled={!watcherActive}>
                                                ⏹ Stop watcher
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        )}
                    </section>
                )}

                {/* Scan Section */}
                {activeSection === "scan" && (
                    <section className="content-section">
                        <h2>📁 Scan Files</h2>
                        <p className="section-desc">Select file types and a folder to index.</p>

                        {/* File Type Selection */}
                        <div className="form-group">
                            <label>File Types to Scan:</label>
                            <div className="file-type-grid">
                                {Object.entries(FILE_TYPE_PRESETS).map(([name, extensions]) => (
                                    <label
                                        key={name}
                                        className={`file-type-option ${selectedTypes.includes(name) ? "selected" : ""}`}
                                    >
                                        <input
                                            type="checkbox"
                                            checked={selectedTypes.includes(name)}
                                            onChange={() => toggleFileType(name)}
                                        />
                                        <span className="file-type-name">{name}</span>
                                        <span className="file-type-exts">
                                            {extensions.slice(0, 3).join(", ")}
                                        </span>
                                    </label>
                                ))}
                            </div>
                        </div>

                        {/* Folder Selection */}
                        <div className="form-group">
                            <label>Folder to Scan:</label>
                            <div className="input-row">
                                <input
                                    type="text"
                                    placeholder="Enter folder path..."
                                    value={scanPath}
                                    onChange={(e) => setScanPath(e.target.value)}
                                    className="folder-input"
                                />
                                <button 
                                    className="btn btn-secondary"
                                    onClick={async () => {
                                        const selected = await open({
                                            directory: true,
                                            multiple: false,
                                            title: "Select folder to scan"
                                        });
                                        if (selected && typeof selected === 'string') {
                                            setScanPath(selected);
                                        }
                                    }}
                                >
                                    📂 Browse
                                </button>
                            </div>
                        </div>

                        {/* Index Path (optional) */}
                        <details className="advanced-options">
                            <summary>Advanced Options</summary>
                            <div className="form-group">
                                <label>Custom Index Location:</label>
                                <input
                                    type="text"
                                    placeholder="Leave empty for default"
                                    value={indexPath}
                                    onChange={(e) => setIndexPath(e.target.value)}
                                />
                            </div>
                        </details>

                        {/* Scan Button */}
                        <div className="action-row">
                            <button 
                                className="btn btn-primary btn-large"
                                onClick={handleScan}
                                disabled={scanStatus === "running"}
                            >
                                {scanStatus === "running" ? "🔄 Scanning..." : "🔍 Start Scan"}
                            </button>
                        </div>

                        {/* Progress */}
                        {scanStatus === "running" && (
                            <div className="progress-section">
                                <div className="progress-bar">
                                    <div className="progress-fill" style={{ width: `${scanProgress.percent}%` }} />
                                </div>
                                <p>Scanning files...</p>
                            </div>
                        )}

                        {/* Result */}
                        {scanStatus === "complete" && scanResult && (
                            <div className="success-message">
                                <h3>✅ Scan Complete!</h3>
                                <p>{scanResult.files_scanned} files indexed ({(scanResult.total_size / 1024 / 1024).toFixed(1)} MB)</p>
                                <p>Index saved to: {scanResult.index_path}</p>
                            </div>
                        )}
                    </section>
                )}

                {/* Embed Section */}
                {activeSection === "embed" && (
                    <section className="content-section">
                        <h2>🧠 Generate Embeddings</h2>
                        <p className="section-desc">Convert your indexed files into semantic vectors using a local model or cloud provider.</p>

                        {!indexPath ? (
                            <div className="empty-state">
                                <p>No index available. Please scan a folder first.</p>
                                <button className="btn btn-primary" onClick={() => setActiveSection("scan")}>
                                    📁 Go to Scan
                                </button>
                            </div>
                        ) : (
                            <>
                                <div className="config-section">
                                    <div className="config-header">
                                        <h3>🧩 Embedding Provider</h3>
                                        <span className={`config-status ${
                                            (embeddingProvider === "local") || 
                                            (embeddingProvider === "llama") ||
                                            (embeddingProvider === "azure" && azureConfigured) || 
                                            (embeddingProvider === "gcp" && gcpConfigured) 
                                            ? "configured" : "not-configured"
                                        }`}>
                                            {(embeddingProvider === "local") || 
                                            (embeddingProvider === "llama") ||
                                            (embeddingProvider === "azure" && azureConfigured) || 
                                            (embeddingProvider === "gcp" && gcpConfigured) 
                                            ? "✓ Configured" : "⚠ Not Configured"}
                                        </span>
                                        <button
                                            className="btn btn-small"
                                            onClick={() => {
                                                if (embeddingProvider === "azure") setShowAzureConfig(!showAzureConfig);
                                                else if (embeddingProvider === "gcp") setShowGcpConfig(!showGcpConfig);
                                            }}
                                            disabled={embeddingProvider === "local"}
                                        >
                                            {(embeddingProvider === "azure" && showAzureConfig) || (embeddingProvider === "gcp" && showGcpConfig) ? "Hide" : "Configure"}
                                        </button>
                                    </div>

                                    <div className="config-form">
                                        <div className="form-group">
                                            <label>Provider:</label>
                                            <select
                                                value={embeddingProvider}
                                                onChange={(e) => {
                                                    const value = e.target.value as any;
                                                    setEmbeddingProvider(value);
                                                    if (value === "local" || value === "llama") {
                                                        setAzureConfigured(true);
                                                        setGcpConfigured(true);
                                                        setErrorMsg("");
                                                    } else if (value === "azure") {
                                                        if (indexPath) loadAzureConfig(indexPath);
                                                        setShowAzureConfig(true);
                                                        setShowGcpConfig(false);
                                                    } else if (value === "gcp") {
                                                        if (indexPath) loadGcpConfig(indexPath);
                                                        setShowGcpConfig(true);
                                                        setShowAzureConfig(false);
                                                    }
                                                }}
                                            >
                                                <option value="local">Local (Deterministic CPU)</option>
                                                <option value="llama">Local llama.cpp (GPU/CPU)</option>
                                                <option value="azure">Azure OpenAI</option>
                                                <option value="gcp">Google Cloud Vertex AI</option>
                                                <option value="multi">Multi-Provider (GCP → Azure → Local)</option>
                                            </select>
                                            <small>
                                                {embeddingProvider === "local" && "Runs offline (deterministic, CPU)."}
                                                {embeddingProvider === "llama" && "Uses your local llama.cpp server (GPU/CPU)."}
                                                {embeddingProvider === "azure" && "Uses Azure OpenAI API."}
                                                {embeddingProvider === "gcp" && "Uses Google Vertex AI API."}
                                                {embeddingProvider === "multi" && "Tries GCP, then Azure, then local — always gets an embedding."}
                                            </small>
                                        </div>

                                        {(embeddingProvider === "local" || embeddingProvider === "llama") && (
                                            <div className="form-group">
                                                <label>Local Model:</label>
                                                <input
                                                    type="text"
                                                    placeholder="BAAI/bge-small-en-v1.5"
                                                    value={localModel}
                                                    onChange={(e) => setLocalModel(e.target.value)}
                                                />
                                                <small>For llama: e.g., embedding-gemma; for deterministic: BAAI/bge-small-en-v1.5</small>
                                            </div>
                                        )}

                                        {embeddingProvider === "llama" && (
                                            <div className="form-group">
                                                <label>Local Endpoint (llama.cpp server):</label>
                                                <input
                                                    type="text"
                                                    placeholder="http://127.0.0.1:8080"
                                                    value={localEndpoint}
                                                    onChange={(e) => setLocalEndpoint(e.target.value)}
                                                />
                                                <small>Run llama.cpp server with your Qwen/Gemma models and point here.</small>
                                            </div>
                                        )}

                                        {embeddingProvider === "azure" && showAzureConfig && (
                                            <div className="provider-config">
                                                <div className="form-group">
                                                    <label>Azure OpenAI Endpoint:
                                                        <span className="info-tooltip">ⓘ
                                                            <span className="tooltip-bubble">Use the resource endpoint such as https://&lt;name&gt;.cognitiveservices.azure.com</span>
                                                        </span>
                                                    </label>
                                                    <input
                                                        type="text"
                                                        placeholder="https://your-resource.cognitiveservices.azure.com"
                                                        value={azureEndpoint}
                                                        onChange={(e) => setAzureEndpoint(e.target.value)}
                                                    />
                                                </div>
                                                <div className="form-group">
                                                    <label>API Key: {hasExistingKey && <span style={{color: 'var(--success-color)', fontSize: '0.85em'}}> (saved)</span>}</label>
                                                    <input
                                                        type="password"
                                                        placeholder={hasExistingKey ? "Key saved - enter new to update" : "Your Azure OpenAI API key"}
                                                        value={azureApiKey}
                                                        onChange={(e) => setAzureApiKey(e.target.value)}
                                                    />
                                                </div>
                                                <div className="form-group">
                                                    <label>Deployment Name:</label>
                                                    <input
                                                        type="text"
                                                        placeholder="text-embedding-3-small"
                                                        value={azureDeployment}
                                                        onChange={(e) => setAzureDeployment(e.target.value)}
                                                    />
                                                </div>
                                                <div className="form-group">
                                                    <label>API Version (optional):</label>
                                                    <input
                                                        type="text"
                                                        placeholder="2024-02-01"
                                                        value={azureApiVersion}
                                                        onChange={(e) => setAzureApiVersion(e.target.value)}
                                                    />
                                                </div>
                                                <button className="btn btn-primary" onClick={saveEmbeddingConfig}>
                                                    💾 Save Azure Config
                                                </button>
                                            </div>
                                        )}

                                        {embeddingProvider === "gcp" && showGcpConfig && (
                                            <div className="provider-config">
                                                <div className="form-group">
                                                    <label>Project ID:</label>
                                                    <input
                                                        type="text"
                                                        placeholder="my-gcp-project-id"
                                                        value={gcpProjectId}
                                                        onChange={(e) => setGcpProjectId(e.target.value)}
                                                    />
                                                </div>
                                                <div className="form-group">
                                                    <label>Location:</label>
                                                    <input
                                                        type="text"
                                                        placeholder="us-central1"
                                                        value={gcpLocation}
                                                        onChange={(e) => setGcpLocation(e.target.value)}
                                                    />
                                                </div>
                                                <div className="form-group">
                                                    <label>Model ID:</label>
                                                    <input
                                                        type="text"
                                                        placeholder="text-embedding-004"
                                                        value={gcpModelId}
                                                        onChange={(e) => setGcpModelId(e.target.value)}
                                                    />
                                                </div>
                                                <div className="form-group">
                                                    <label>Service Account JSON: {hasExistingGcpKey && !useGcpAdc && <span style={{color: 'var(--success-color)', fontSize: '0.85em'}}> (saved)</span>}</label>
                                                    <div className="input-row">
                                                        <input
                                                            type="text"
                                                            placeholder={useGcpAdc ? "Using gcloud ADC" : (hasExistingGcpKey ? "Key saved - select new to update" : "Select your service-account.json")}
                                                            value={useGcpAdc ? "" : gcpServiceAccountPath}
                                                            readOnly
                                                            className="folder-input"
                                                            disabled={useGcpAdc}
                                                        />
                                                        <button 
                                                            className="btn btn-secondary"
                                                            disabled={useGcpAdc}
                                                            onClick={async () => {
                                                                const selected = await open({
                                                                    multiple: false,
                                                                    filters: [{ name: 'JSON', extensions: ['json'] }],
                                                                    title: "Select Service Account Key"
                                                                });
                                                                if (selected && typeof selected === 'string') {
                                                                    setGcpServiceAccountPath(selected);
                                                                }
                                                            }}
                                                        >
                                                            📂 Browse
                                                        </button>
                                                    </div>
                                                    <label style={{ display: 'flex', gap: '0.5rem', alignItems: 'center', marginTop: '6px' }}>
                                                        <input
                                                            type="checkbox"
                                                            checked={useGcpAdc}
                                                            onChange={(e) => setUseGcpAdc(e.target.checked)}
                                                        />
                                                        Use gcloud ADC (no key file)
                                                    </label>
                                                    <small>ADC uses your gcloud login (`gcloud auth application-default login`).</small>
                                                </div>
                                                <div className="form-group">
                                                    <label>Custom Endpoint (Optional):</label>
                                                    <input
                                                        type="text"
                                                        placeholder="https://custom-vm-url/predict"
                                                        value={gcpEndpoint}
                                                        onChange={(e) => setGcpEndpoint(e.target.value)}
                                                    />
                                                    <small>Leave empty to use standard Vertex AI endpoint</small>
                                                </div>
                                                <button className="btn btn-primary" onClick={saveEmbeddingConfig}>
                                                    💾 Save GCP Config
                                                </button>
                                            </div>
                                        )}
                                    </div>
                                </div>
                                <div className="info-box">
                                    <p><strong>Index:</strong> {indexPath}</p>
                                    <p><strong>Files:</strong> {indexStats?.total_files || 0}</p>
                                    <p><strong>Status:</strong> {indexStats?.has_embeddings ? "Embeddings exist" : "No embeddings yet"}</p>
                                </div>

                                <div className="form-group">
                                    <label>Test Options (optional):</label>
                                    <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                                        <input type="number" placeholder="Max files" value={embedMaxFiles ?? ''} onChange={(e) => setEmbedMaxFiles(e.target.value ? parseInt(e.target.value) : undefined)} />
                                        <input type="number" placeholder="Batch size" value={embedBatchSize ?? ''} onChange={(e) => setEmbedBatchSize(e.target.value ? parseInt(e.target.value) : undefined)} />
                                        <small style={{ color: 'var(--text-secondary)' }}>Use for quick tests</small>
                                    </div>
                                </div>

                                {embedStatus === "running" && (
                                    <div className="progress-section" style={{ marginTop: '1rem' }}>
                                        <div className="progress-bar">
                                            <div className="progress-fill" style={{ width: `${Math.min(100, embedProgress)}%` }} />
                                        </div>
                                        <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', marginTop: '0.5rem' }}>
                                            <p style={{ margin: 0, color: 'var(--text-secondary)' }}>
                                                Processing... {embedProgress}%
                                                {embedProgressDetail && embedProgressDetail.total > 0 && (
                                                    <span> ({embedProgressDetail.processed}/{embedProgressDetail.total} files)</span>
                                                )}
                                                {embedProgressDetail && embedProgressDetail.totalBatches && (
                                                    <span style={{ marginLeft: '0.5rem' }}>
                                                        · Batch {embedProgressDetail.currentBatch ?? 1}/{embedProgressDetail.totalBatches} (size {embedProgressDetail.batchSize ?? '?'} )
                                                    </span>
                                                )}
                                            </p>
                                            <button
                                                className="btn btn-secondary btn-small"
                                                onClick={handleCancelEmbed}
                                                title="Stop embedding and save progress so far"
                                            >
                                                ⏹ Cancel
                                            </button>
                                        </div>
                                    </div>
                                )}

                                <div className="action-row">
                                    <button
                                        className="btn btn-primary btn-large"
                                        onClick={handleEmbed}
                                        disabled={embedStatus === "running" || (embeddingProvider === "azure" && !azureConfigured) || (embeddingProvider === "gcp" && !gcpConfigured)}
                                    >
                                        {embedStatus === "running" ? "🔄 Generating..." : "🧠 Generate Embeddings"}
                                    </button>
                                    {embeddingProvider === "azure" && !azureConfigured && (
                                        <span className="hint">Configure Azure first</span>
                                    )}
                                    {embeddingProvider === "gcp" && !gcpConfigured && (
                                        <span className="hint">Configure GCP first</span>
                                    )}
                                        <button className="btn btn-secondary" onClick={async () => {
                                            try {
                                                setErrorMsg("");
                                                const root = (indexPath && indexPath.includes('.wayfinder_index')) ? indexPath.replace(/\\[^\\]*$/, '') : "C:/Temp";
                                                const res = await tauriService.validateAllAzureConfigs(root);
                                                // Show results in a modal or console for now
                                                console.log('ValidateAll result:', res);
                                                notify({ id: `validate-${Date.now()}`, title: 'Validation Complete', message: `Validated ${res.results.length} indices`, level: 'info', timeout: 8000 });
                                                // Store results for more detailed UI
                                                setValidationResults(res.results || []);
                                                setShowValidationResults(true);
                                            } catch (e: any) {
                                                setErrorMsg(e.toString());
                                            }
                                        }}>
                                            🔍 Validate Saved Configs
                                        </button>
                                </div>

                                {embedStatus === "running" && (
                                    <div className="progress-section">
                                        <div className="progress-bar">
                                            <div className="progress-fill" style={{ width: `${embedProgress}%` }} />
                                        </div>
                                        <p>Generating embeddings... {embedProgress}%</p>
                                    </div>
                                )}

                                {/* Validation modal for suggestions */}
                                <Modal
                                    visible={showValidationModal}
                                    title="Suggested Endpoint"
                                    message={validationMessage}
                                    onConfirm={applySuggestedEndpoint}
                                    onCancel={cancelSuggestedEndpoint}
                                    confirmLabel="Apply & Save"
                                    cancelLabel="Cancel"
                                />

                                {embedResult && (
                                    <div className="embed-result">
                                        <p>Cached: {embedResult.cached_count || 0} (unchanged files)</p>
                                        {embedResult.error_count > 0 && (
                                            <p className="warning">Errors: {embedResult.error_count} files failed</p>
                                        )}
                                        <p>Your files are now ready for semantic search and clustering.</p>
                                    </div>
                                )}

                                {embedStatus === "error" && (
                                    <div className="error-message">
                                        <h3>❌ Embedding Failed</h3>
                                        <p>{errorMsg}</p>
                                    </div>
                                )}

                                    {/* Validation Results Drawer */}
                                    {showValidationResults && (
                                        <div className="validation-results-drawer">
                                            <h3>Validation Results</h3>
                                            <button className="btn btn-small" onClick={() => setShowValidationResults(false)}>Close</button>
                                            <div className="results-list">
                                                {validationResults.map((r: any, i: number) => (
                                                    <div key={i} className="validation-item">
                                                        <h4>{r.index_dir}</h4>
                                                        {r.error && <p className="warning">Error: {r.error}</p>}
                                                        {r.config && (
                                                            <div>
                                                                <p>Endpoint: {r.config.endpoint}</p>
                                                                <p>Deployment: {r.config.deployment_name}</p>
                                                                <p>API Version: {r.config.api_version}</p>
                                                            </div>
                                                        )}
                                                        {r.validation && (
                                                            <div>
                                                                <p>Status: {r.validation.success ? 'OK' : 'Failed'}</p>
                                                                <p>Message: {r.validation.message}</p>
                                                                {r.validation.final_url && (
                                                                    <p>URL: {r.validation.final_url}</p>
                                                                )}
                                                                {r.validation.suggested_endpoint && (
                                                                    <div>
                                                                        <p>Suggested: {r.validation.suggested_endpoint}</p>
                                                                        <button className="btn btn-small" onClick={async () => {
                                                                            try {
                                                                                await tauriService.saveAzureConfig(r.index_dir, r.validation.suggested_endpoint, r.config.api_key || '', r.config.deployment_name || '', r.config.api_version || undefined);
                                                                                await tauriService.saveProviderConfig(r.index_dir, "azure", undefined);
                                                                                notify({ id: `applied-${i}`, title: 'Applied', message: `Applied suggestion to ${r.index_dir}`, level: 'success' });
                                                                            } catch (e: any) {
                                                                                notify({ id: `err-${i}`, title: 'Error', message: e.toString(), level: 'error' });
                                                                            }
                                                                        }}>Apply Suggestion</button>
                                                                    </div>
                                                                )}
                                                            </div>
                                                        )}
                                                    </div>
                                                ))}
                                            </div>
                                        </div>
                                    )}
                            </>
                        )}
                    </section>
                )}

                {/* Cluster Section */}
                {activeSection === "cluster" && (
                    <section className="content-section">
                        <h2>🗂️ Clusters</h2>
                        <p className="section-desc">Group files by similarity using your existing embeddings.</p>

                        {!indexPath ? (
                            <div className="empty-state">
                                <p>No index available. Please scan a folder first.</p>
                                <button className="btn btn-primary" onClick={() => setActiveSection("scan")}>
                                    📁 Start Scanning
                                </button>
                            </div>
                        ) : (
                            <>
                                <div className="form-group">
                                    <label>Number of clusters (optional):</label>
                                    <input
                                        type="number"
                                        placeholder="Auto (sqrt of file count)"
                                        value={numClusters ?? ""}
                                        onChange={(e) => setNumClusters(e.target.value ? parseInt(e.target.value) : undefined)}
                                        style={{ maxWidth: "240px" }}
                                    />
                                </div>

                                <div className="action-row" style={{ gap: "0.75rem" }}>
                                    <button
                                        className="btn btn-primary"
                                        onClick={handleCluster}
                                        disabled={clusterStatus === "running"}
                                    >
                                        {clusterStatus === "running" ? "🔄 Clustering..." : "🔗 Create Clusters"}
                                    </button>
                                    <button
                                        className="btn btn-secondary"
                                        onClick={async () => {
                                            try {
                                                const data = await tauriService.getClustersSummary(indexPath);
                                                setClusters(data.clusters || []);
                                                setErrorMsg("");
                                            } catch (e: any) {
                                                setErrorMsg(e.toString());
                                            }
                                        }}
                                        disabled={clusterStatus === "running"}
                                    >
                                        ↻ Refresh Clusters
                                    </button>
                                </div>

                                {clusterStatus === "error" && errorMsg && (
                                    <div className="error-message" style={{ marginTop: "1rem" }}>
                                        <p>{errorMsg}</p>
                                    </div>
                                )}

                                {clusters.length === 0 ? (
                                    <p className="no-results" style={{ marginTop: "1.5rem" }}>
                                        No clusters yet. Generate embeddings first, then create clusters.
                                    </p>
                                ) : (
                                    <div className="clusters-list" style={{ marginTop: "1.5rem" }}>
                                        <h4>Clusters ({clusters.length})</h4>
                                        {clusters.map((cluster: any) => {
                                            const size = cluster.file_count || cluster.files?.length || cluster.size || 0;
                                            const files = cluster.files || cluster.sample_files || [];
                                            return (
                                                <div key={cluster.id} className="cluster-item" style={{ border: "1px solid var(--border-color)", padding: "1rem", borderRadius: "var(--radius-md)", marginTop: "0.5rem" }}>
                                                    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                                                        <span className="cluster-title">{cluster.label || `Cluster ${cluster.id}`}</span>
                                                        <span className="cluster-size">{size} files</span>
                                                    </div>
                                                    {cluster.summary && (
                                                        <p style={{ color: "var(--text-secondary)", marginTop: "0.5rem" }}>{cluster.summary}</p>
                                                    )}
                                                    {files.length > 0 && (
                                                        <ul style={{ marginTop: "0.5rem", color: "var(--text-secondary)" }}>
                                                            {files.slice(0, 5).map((f: any, i: number) => (
                                                                <li key={i}>{f.name || f.path || f}</li>
                                                            ))}
                                                            {files.length > 5 && <li>...and {files.length - 5} more</li>}
                                                        </ul>
                                                    )}
                                                </div>
                                            );
                                        })}
                                    </div>
                                )}
                            </>
                        )}
                    </section>
                )}

                {/* Search Section */}
                {activeSection === "search" && (
                    <section className="content-section">
                        <h2>🔍 Search Files</h2>

                        {!indexPath ? (
                            <div className="empty-state">
                                <p>No index available. Please scan a folder first.</p>
                            </div>
                        ) : (
                            <>
                                <div className="search-controls">
                                    <div className="search-box">
                                        <input
                                            type="text"
                                                placeholder="What are you looking for?"
                                                value={searchQuery}
                                                onChange={(e) => setSearchQuery(e.target.value)}
                                                onKeyDown={(e) => e.key === "Enter" && handleSearch()}
                                                className="search-input"
                                            />
                                            <button className="btn btn-primary" onClick={handleSearch}>
                                                Search
                                            </button>
                                        </div>

                                        <div className="search-options">
                                            <div className="option-group">
                                                <label>Results: {topK}</label>
                                                <input
                                                    type="range"
                                                    min="1"
                                                    max="50"
                                                    value={topK}
                                                    onChange={(e) => setTopK(parseInt(e.target.value))}
                                                />
                                            </div>
                                            <div className="option-group">
                                                <label>Semantic Weight: {semanticWeight.toFixed(1)}</label>
                                                <input
                                                    type="range"
                                                    min="0"
                                                    max="100"
                                                    value={semanticWeight * 100}
                                                    onChange={(e) => setSemanticWeight(parseInt(e.target.value) / 100)}
                                                />
                                            </div>
                                        </div>
                                    </div>

                                    <div className="search-results">
                                        {searchResults.length === 0 ? (
                                            <p className="no-results">Enter a search term to find files</p>
                                        ) : (
                                            <>
                                                <p className="results-count">{searchResults.length} results found</p>
                                                {searchResults.map((result, i) => (
                                                    <div key={i} className="search-result-item">
                                                        <div className="result-header">
                                                            <span className="result-name">{result.name}</span>
                                                            {result.score && (
                                                                <span className="result-score">
                                                                    {(result.score * 100).toFixed(0)}% match
                                                                </span>
                                                            )}
                                                        </div>
                                                        <div className="result-path">{result.path}</div>
                                                        {result.preview && (
                                                            <div className="result-preview">{result.preview}</div>
                                                        )}
                                                    </div>
                                                ))}
                                            </>
                                        )}
                                    </div>
                            </>
                        )}
                    </section>
                )}

                {/* Timeline Section */}
                {activeSection === "timeline" && (
                    <section className="content-section">
                        <h2>📅 Timeline</h2>
                        <p className="section-desc">View your files organized by modification date.</p>

                        {!indexPath ? (
                            <div className="empty-state">
                                <p>No index available. Please scan a folder first.</p>
                            </div>
                        ) : (
                            <>
                                <div className="timeline-controls">
                                    <label>Show files from last:</label>
                                    <select
                                        value={timelineDays}
                                        onChange={(e) => setTimelineDays(parseInt(e.target.value))}
                                    >
                                        <option value="7">7 days</option>
                                        <option value="14">2 weeks</option>
                                        <option value="30">30 days</option>
                                        <option value="90">90 days</option>
                                    </select>
                                    <button className="btn btn-primary" onClick={handleTimeline}>
                                        Load Timeline
                                    </button>
                                    </div>

                                <div className="timeline-results">
                                    {timelineData.length === 0 ? (
                                        <p className="no-results">Click "Load Timeline" to view recent files</p>
                                    ) : (
                                        timelineData.map((day, i) => (
                                            <div key={i} className="timeline-day">
                                                <h3 className="day-header">{day.date}</h3>
                                                <div className="day-files">
                                                    {day.files?.map((file: any, j: number) => (
                                                        <div key={j} className="timeline-file">
                                                            <span className="file-name">{file.name}</span>
                                                            <span className="file-path">{file.path}</span>
                                                        </div>
                                                    ))}
                                                </div>
                                            </div>
                                        ))
                                    )}
                                </div>
                            </>
                        )}
                    </section>
                )}

                {/* Org Section */}
                {activeSection === "org" && (
                    <section className="content-section">
                        <h2>🗃️ Organize Projects</h2>
                        <p className="section-desc">Classify files into projects and build a reversible view with links.</p>

                        {!indexPath ? (
                            <div className="empty-state">
                                <p>No index available. Please scan a folder first.</p>
                                <button className="btn btn-primary" onClick={() => setActiveSection("scan")}>
                                    📁 Go to Scan
                                </button>
                            </div>
                        ) : (
                            <>
                                <div className="form-group">
                                    <label>Labels (JSON: {`{ "projectA": ["foo"], "projectB": ["bar"] }`})</label>
                                    <textarea
                                        className="folder-input"
                                        rows={6}
                                        value={labelSpec}
                                        onChange={(e) => setLabelSpec(e.target.value)}
                                    />
                                </div>
                                <div className="form-group">
                                    <label>Rules (JSON)</label>
                                    <textarea
                                        className="folder-input"
                                        rows={5}
                                        value={rulesSpec}
                                        onChange={(e) => setRulesSpec(e.target.value)}
                                    />
                                    <small>Use min_confidence, ambiguity_delta, include_patterns, exclude_patterns.</small>
                                </div>
                                <div className="form-group" style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                                    <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                                        <input type="checkbox" checked={useSmartClassifier} onChange={(e) => setUseSmartClassifier(e.target.checked)} />
                                        Smart LLM refine (Qwen 14B)
                                    </label>
                                    {useSmartClassifier && (
                                        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '0.5rem' }}>
                                            <input type="text" placeholder="http://localhost:5001" value={smartLlmEndpoint} onChange={(e) => setSmartLlmEndpoint(e.target.value)} />
                                            <input type="text" placeholder="qwen2.5-coder-14b-instruct" value={smartLlmModel} onChange={(e) => setSmartLlmModel(e.target.value)} />
                                            <input type="number" placeholder="LLM limit" value={smartLlmLimit} onChange={(e) => setSmartLlmLimit(e.target.value ? parseInt(e.target.value) : 0)} />
                                            <input type="number" step="0.1" placeholder="Min score before LLM" value={smartLlmMinScore} onChange={(e) => setSmartLlmMinScore(e.target.value ? parseFloat(e.target.value) : 0)} />
                                            <input type="number" placeholder="Top alternates" value={smartLlmTopAlts} onChange={(e) => setSmartLlmTopAlts(e.target.value ? parseInt(e.target.value) : 0)} />
                                            <small style={{ gridColumn: '1 / span 2', color: 'var(--text-secondary)' }}>LLM is only called for low-confidence/ambiguous files; limit controls how many files go to Qwen.</small>
                                        </div>
                                    )}
                                </div>
                                <div className="action-row" style={{ gap: '0.75rem' }}>
                                    <button className="btn btn-primary" onClick={handleClassify}>🔍 Classify</button>
                                    <button className="btn btn-secondary" onClick={handleBuildView} disabled={classified.length === 0}>🧭 Build View</button>
                                    <label style={{ display: 'flex', alignItems: 'center', gap: '0.4rem' }}>
                                        <input type="checkbox" checked={viewDryRun} onChange={(e) => setViewDryRun(e.target.checked)} /> Dry run
                                    </label>
                                    <input
                                        type="text"
                                        value={viewRoot}
                                        onChange={(e) => setViewRoot(e.target.value)}
                                        style={{ maxWidth: '320px' }}
                                        placeholder="Wayfinder/View"
                                    />
                                </div>

                                {classified.length > 0 && (
                                    <div className="table" style={{ marginTop: '1rem', border: '1px solid var(--border-color)', borderRadius: 'var(--radius-md)', padding: '0.75rem' }}>
                                        <h4>Classified ({classified.length})</h4>
                                        <div style={{ maxHeight: '320px', overflow: 'auto' }}>
                                            {classified.map((c, i) => (
                                                <div key={i} style={{ display: 'grid', gridTemplateColumns: '1fr 160px 80px', gap: '0.5rem', padding: '0.35rem 0', borderBottom: '1px solid var(--border-color)' }}>
                                                    <div style={{ wordBreak: 'break-all' }}>{c.path}</div>
                                                    <div>
                                        <strong>{c.project}</strong>
                                                        <div style={{ color: 'var(--text-secondary)', fontSize: '0.85rem' }}>Conf: {c.confidence.toFixed(2)}</div>
                                                    </div>
                                                    <div style={{ color: 'var(--text-secondary)', fontSize: '0.85rem' }}>
                                                        {c.alternates && c.alternates.length > 0 && (
                                                            <div>Alt: {c.alternates[0][0]} ({c.alternates[0][1].toFixed(2)})</div>
                                                        )}
                                                    </div>
                                                </div>
                                            ))}
                                        </div>
                                    </div>
                                )}

                                {viewPlan.length > 0 && (
                                    <div className="table" style={{ marginTop: '1rem', border: '1px solid var(--border-color)', borderRadius: 'var(--radius-md)', padding: '0.75rem' }}>
                                        <h4>View Plan ({viewPlan.length}) {viewDryRun ? '(dry-run)' : ''}</h4>
                                        <div style={{ maxHeight: '240px', overflow: 'auto' }}>
                                            {viewPlan.slice(0, 200).map((p, i) => (
                                                <div key={i} style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 90px', gap: '0.5rem', padding: '0.35rem 0', borderBottom: '1px solid var(--border-color)' }}>
                                                    <div style={{ wordBreak: 'break-all' }}>{p.from}</div>
                                                    <div style={{ wordBreak: 'break-all' }}>{p.to}</div>
                                                    <div>{p.link_type || 'planned'}</div>
                                                </div>
                                            ))}
                                            {viewPlan.length > 200 && <div style={{ color: 'var(--text-secondary)' }}>Showing first 200...</div>}
                                        </div>
                                    </div>
                                )}

                                {viewErrors.length > 0 && (
                                    <div className="error-message" style={{ marginTop: '0.75rem' }}>
                                        <h4>Errors</h4>
                                        <ul>
                                            {viewErrors.map((e, i) => (
                                                <li key={i}>{e}</li>
                                            ))}
                                        </ul>
                                    </div>
                                )}
                            </>
                        )}
                    </section>
                )}

                {/* Git Assistant Section */}
                {activeSection === "git" && (
                    <section className="content-section">
                        <h2>📎 Git Clippy Assistant</h2>
                        <p className="section-desc">Your friendly git helper for ADHD developers.</p>

                        <div className="form-group">
                            <label>Git repository path:</label>
                            <div className="input-row">
                                <input
                                    type="text"
                                    placeholder="Select a git repository"
                                    value={gitRepoPath}
                                    onChange={(e) => setGitRepoPath(e.target.value)}
                                    className="folder-input"
                                />
                                <button
                                    className="btn btn-secondary"
                                    onClick={async () => {
                                        const selected = await open({ directory: true, multiple: false, title: "Select git repository" });
                                        if (selected && typeof selected === 'string') {
                                            setGitRepoPath(selected);
                                        }
                                    }}
                                >
                                    📂 Browse
                                </button>
                                <button
                                    className="btn btn-tertiary"
                                    onClick={() => {
                                        if (scanPath) {
                                            setGitRepoPath(scanPath);
                                        } else if (indexPath) {
                                            const root = indexPath.includes('.wayfinder_index')
                                                ? indexPath.replace(/\\?\.wayfinder_index$/, '').replace(/\/\.wayfinder_index$/, '')
                                                : indexPath;
                                            setGitRepoPath(root);
                                        }
                                    }}
                                >
                                    Use scan folder
                                </button>
                            </div>
                            <small>Git Clippy can run on any repo, not just the last scanned folder.</small>
                        </div>

                        <GitAssistant
                            repoPath={gitRepoPath}
                            indexPath={indexPath}
                            chatEndpoint={llamaChatEndpoint || undefined}
                            chatModel={llamaChatModel || undefined}
                        />
                    </section>
                )}

                {activeSection === "nauti" && (
                    <section className="content-section">
                        <h2>🧭 Nauti-Clippy</h2>
                        <p className="section-desc">Watch non-git folders (e.g., Documents, Downloads, Pictures) for duplicates and copy/backup clutter.</p>
                        <NautiClippy defaultPaths={scanPath ? [scanPath] : []} />
                    </section>
                )}

                {showGlobalConfig && (
                    <div className="modal-overlay" onClick={() => setShowGlobalConfig(false)}>
                        <div className="modal modal-wide" onClick={(e) => e.stopPropagation()}>
                            <div className="modal-header">
                                <h3>⚙️ AI Configuration</h3>
                                <button className="close-btn" onClick={() => setShowGlobalConfig(false)}>×</button>
                            </div>
                            <div className="modal-content modal-content-scroll">
                                <div className="config-grid">
                                    <div className="form-block">
                                        <h4>Embedding Provider</h4>
                                        <div className="form-group">
                                            <label>Provider:</label>
                                            <select
                                                value={embeddingProvider}
                                                onChange={(e) => {
                                                    const value = e.target.value as any;
                                                    setEmbeddingProvider(value);
                                                    if (value === "local" || value === "llama") {
                                                        setAzureConfigured(true);
                                                        setGcpConfigured(true);
                                                        setErrorMsg("");
                                                        setShowAzureConfig(false);
                                                        setShowGcpConfig(false);
                                                    } else if (value === "azure") {
                                                        if (indexPath) loadAzureConfig(indexPath);
                                                        setShowAzureConfig(true);
                                                        setShowGcpConfig(false);
                                                    } else if (value === "gcp") {
                                                        if (indexPath) loadGcpConfig(indexPath);
                                                        setShowGcpConfig(true);
                                                        setShowAzureConfig(false);
                                                    }
                                                }}
                                            >
                                                <option value="local">Local (Deterministic CPU)</option>
                                                <option value="llama">Local llama.cpp (GPU/CPU)</option>
                                                <option value="azure">Azure OpenAI</option>
                                                <option value="gcp">Google Cloud Vertex AI</option>
                                                <option value="multi">Multi-Provider (GCP → Azure → Local)</option>
                                            </select>
                                        </div>

                                        {(embeddingProvider === "local" || embeddingProvider === "llama") && (
                                            <div className="form-group">
                                                <label>Local Model:</label>
                                                <input
                                                    type="text"
                                                    placeholder="BAAI/bge-small-en-v1.5"
                                                    value={localModel}
                                                    onChange={(e) => setLocalModel(e.target.value)}
                                                />
                                            </div>
                                        )}

                                        {embeddingProvider === "llama" && (
                                            <div className="form-group">
                                                <label>Local Endpoint (llama.cpp embeddings):</label>
                                                <input
                                                    type="text"
                                                    placeholder="http://127.0.0.1:8080"
                                                    value={localEndpoint}
                                                    onChange={(e) => setLocalEndpoint(e.target.value)}
                                                />
                                                <small>Use your embedding server base URL (e.g., http://localhost:5002).</small>
                                            </div>
                                        )}

                                        {embeddingProvider === "azure" && (
                                            <div className="provider-config">
                                                <div className="form-group">
                                                    <label>Azure OpenAI Endpoint:</label>
                                                    <input
                                                        type="text"
                                                        placeholder="https://your-resource.cognitiveservices.azure.com"
                                                        value={azureEndpoint}
                                                        onChange={(e) => setAzureEndpoint(e.target.value)}
                                                    />
                                                </div>
                                                <div className="form-group">
                                                    <label>API Key:</label>
                                                    <input
                                                        type="password"
                                                        placeholder={hasExistingKey ? "Key saved - enter new to update" : "Your Azure OpenAI API key"}
                                                        value={azureApiKey}
                                                        onChange={(e) => setAzureApiKey(e.target.value)}
                                                    />
                                                </div>
                                                <div className="form-group">
                                                    <label>Deployment Name:</label>
                                                    <input
                                                        type="text"
                                                        placeholder="text-embedding-3-small"
                                                        value={azureDeployment}
                                                        onChange={(e) => setAzureDeployment(e.target.value)}
                                                    />
                                                </div>
                                                <div className="form-group">
                                                    <label>API Version (optional):</label>
                                                    <input
                                                        type="text"
                                                        placeholder="2024-02-01"
                                                        value={azureApiVersion}
                                                        onChange={(e) => setAzureApiVersion(e.target.value)}
                                                    />
                                                </div>
                                            </div>
                                        )}

                                        {embeddingProvider === "gcp" && (
                                            <div className="provider-config">
                                                <div className="form-group">
                                                    <label>Project ID:</label>
                                                    <input
                                                        type="text"
                                                        placeholder="my-gcp-project-id"
                                                        value={gcpProjectId}
                                                        onChange={(e) => setGcpProjectId(e.target.value)}
                                                    />
                                                </div>
                                                <div className="form-group">
                                                    <label>Location:</label>
                                                    <input
                                                        type="text"
                                                        placeholder="us-central1"
                                                        value={gcpLocation}
                                                        onChange={(e) => setGcpLocation(e.target.value)}
                                                    />
                                                </div>
                                                <div className="form-group">
                                                    <label>Model ID:</label>
                                                    <input
                                                        type="text"
                                                        placeholder="text-embedding-004"
                                                        value={gcpModelId}
                                                        onChange={(e) => setGcpModelId(e.target.value)}
                                                    />
                                                </div>
                                                <div className="form-group">
                                                    <label>Custom Endpoint (optional):</label>
                                                    <input
                                                        type="text"
                                                        placeholder="https://custom-vm-url/predict"
                                                        value={gcpEndpoint}
                                                        onChange={(e) => setGcpEndpoint(e.target.value)}
                                                    />
                                                </div>
                                                <small>Upload keys in the Embeddings panel; ADC toggle also lives there.</small>
                                            </div>
                                        )}
                                    </div>

                                    <div className="form-block">
                                        <h4>Local Chat Defaults</h4>
                                        <div className="form-group">
                                            <label>Chat Endpoint:</label>
                                            <input
                                                type="text"
                                                placeholder="http://localhost:5001"
                                                value={llamaChatEndpoint}
                                                onChange={(e) => setLlamaChatEndpoint(e.target.value)}
                                            />
                                            <small>Base URL for the local chat server (used by Git Clippy chat).</small>
                                        </div>
                                        <div className="form-group">
                                            <label>Chat Model:</label>
                                            <input
                                                type="text"
                                                placeholder="qwen2.5-coder-1.5b-instruct"
                                                value={llamaChatModel}
                                                onChange={(e) => setLlamaChatModel(e.target.value)}
                                            />
                                            <small>Matches the model string your server exposes.</small>
                                        </div>
                                    </div>
                                </div>
                                {!indexPath && (
                                    <p className="hint" style={{ marginTop: '0.5rem' }}>
                                        Scan a folder first to persist embedding provider settings to an index.
                                    </p>
                                )}
                            </div>
                            <div className="modal-footer">
                                <button className="btn btn-secondary" onClick={() => setShowGlobalConfig(false)}>Close</button>
                                <button className="btn btn-primary" onClick={handleGlobalSave}>Save &amp; Close</button>
                            </div>
                        </div>
                    </div>
                )}
            </main>
        </div>
    );
}
