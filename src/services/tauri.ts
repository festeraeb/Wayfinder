// Tauri command wrapper service
import { invoke } from "@tauri-apps/api/core";
import * as Types from "../types";

export const tauriService = {
    async scanDirectory(path: string, indexDir: string, extensions?: string[], allowAll?: boolean): Promise<Types.ScanResult> {
        return invoke("scan_directory", { path, indexDir, extensions, allowAll });
    },

    async generateEmbeddings(indexDir: string, maxFiles?: number, batchSize?: number): Promise<Types.EmbedResult> {
        return invoke("generate_embeddings", { indexDir, maxFiles, batchSize });
    },

    async generateEmbeddingsMulti(indexDir: string, maxFiles?: number, batchSize?: number): Promise<Types.EmbedResult> {
        return invoke("generate_embeddings_multi", { indexDir, maxFiles, batchSize });
    },

    async getEmbeddingProgress(indexDir: string): Promise<Types.BatchProgress> {
        return invoke("get_embedding_progress", { indexDir });
    },

    async cancelEmbedding(indexDir: string): Promise<void> {
        return invoke("cancel_embedding", { indexDir });
    },

    async createClusters(
        indexDir: string,
        numClusters?: number
    ): Promise<Types.ClusterResult> {
        return invoke("create_clusters", {
            indexDir,
            numClusters,
        });
    },

    async search(
        query: string,
        indexDir: string,
        topK: number = 10,
        semanticWeight: number = 0.7
    ): Promise<Types.SearchResult[]> {
        return invoke("search", {
            query,
            indexDir,
            topK,
            semanticWeight,
        });
    },

    async getClustersSummary(indexDir: string): Promise<Types.ClustersSummary> {
        return invoke("get_clusters_summary", { indexDir });
    },

    async getTimeline(indexDir: string, days: number = 30): Promise<Types.TimelineData> {
        return invoke("get_timeline", { indexDir, days });
    },

    async getStats(indexDir: string): Promise<Types.IndexStats> {
        return invoke("get_stats", { indexDir });
    },

    async validateIndex(indexDir: string): Promise<Types.IndexState> {
        return invoke("validate_index", { indexDir });
    },

    async getSystemInfo(): Promise<Types.SystemInfo> {
        return invoke("get_system_info");
    },

    async saveAzureConfig(
        indexDir: string,
        endpoint: string,
        apiKey: string,
        deploymentName: string,
        apiVersion?: string
    ): Promise<{ success: boolean; message: string }> {
        return invoke("save_azure_config", {
            indexDir,
            endpoint,
            apiKey,
            deploymentName,
            apiVersion,
        });
    },

    async saveGcpConfig(
        indexDir: string,
        projectId: string,
        location: string,
        modelId: string,
        serviceAccountPath: string,
        endpoint?: string
    ): Promise<{ success: boolean; message: string }> {
        return invoke("save_gcp_config", {
            indexDir,
            projectId,
            location,
            modelId,
            serviceAccountPath,
            endpoint,
        });
    },

    async saveProviderConfig(
        indexDir: string,
        provider: Types.EmbeddingProvider | "multi",
        localModel?: string,
        localEndpoint?: string
    ): Promise<{ success: boolean; message: string }> {
        return invoke("save_provider_config", {
            indexDir,
            provider,
            localModel,
            localEndpoint,
        });
    },

    async loadAzureConfig(indexDir: string): Promise<Types.AzureConfigStatus> {
        return invoke("load_azure_config", { indexDir });
    },

    async loadGcpConfig(indexDir: string): Promise<Types.GcpConfigStatus> {
        return invoke("load_gcp_config", { indexDir });
    },

    async loadProviderConfig(indexDir: string): Promise<Types.ProviderConfigStatus> {
        return invoke("load_provider_config", { indexDir });
    },
    async validateAzureConfig(indexDir: string, endpoint: string, apiKey: string, deploymentName: string, apiVersion?: string): Promise<Types.AzureValidationResult> {
        return invoke("validate_azure_config", { indexDir, endpoint, apiKey, deploymentName, apiVersion });
    },

    async validateGcpConfig(projectId: string, location: string, modelId: string, serviceAccountPath: string, endpoint?: string): Promise<Types.GcpValidationResult> {
        return invoke("validate_gcp_config", { projectId, location, modelId, serviceAccountPath, endpoint });
    },

    async validateAllAzureConfigs(rootPath: string): Promise<any> {
        return invoke("validate_all_azure_configs", { rootPath });
    },

    async getClustersData(indexDir: string): Promise<Types.ClustersData> {
        return invoke("get_clusters_data", { indexDir });
    },

    // Reminders
    async listReminders(includeDone?: boolean): Promise<{ success: boolean; reminders: any[] }> {
        return invoke("list_reminders", { includeDone });
    },

    async addReminder(title: string, due?: string, severity?: string, linkPath?: string, repoPath?: string): Promise<{ success: boolean; reminder: any }> {
        return invoke("add_reminder", { title, due, severity, linkPath, repoPath });
    },

    async updateReminderStatus(id: string, status: string): Promise<{ success: boolean }> {
        return invoke("update_reminder_status", { id, status });
    },

    async snoozeReminder(id: string, hours: number): Promise<{ success: boolean }> {
        return invoke("snooze_reminder", { id, hours });
    },

    async classifyFiles(indexDir: string, labels: Record<string, string[]>, topN?: number): Promise<{ success: boolean; classified: any[] }> {
        return invoke("classify_files", { indexDir, labels, topN });
    },

    async buildViewPlan(classified: any[], viewRoot: string, preferJunction?: boolean, dryRun?: boolean): Promise<{ success: boolean; plan: any[]; applied: any[]; errors: string[] }> {
        return invoke("build_view_plan", { classified, viewRoot, preferJunction, dryRun });
    },

    // Git Assistant
    async getGitClippyReport(repoPath: string, indexDir?: string): Promise<Types.GitClippyReport> {
        return invoke("get_git_clippy_report", { repoPath, indexDir });
    },

    async getNautiClippyReport(paths: string[]): Promise<Types.NautiReport> {
        return invoke("get_nauticlippy_report", { paths });
    },

    async chatLlama(prompt: string, model?: string, endpoint?: string): Promise<{ text: string }> {
        return invoke("chat_llama", { prompt, model, endpoint });
    },

    async executeClippyAction(repoPath: string, action: string, data?: any): Promise<{ success: boolean; output: string }> {
        return invoke("execute_clippy_action", { repoPath, action, data });
    },

    async isGitRepo(path: string): Promise<boolean> {
        return invoke("is_git_repo", { path });
    },

    // File watcher
    async startFileWatcher(watchPaths?: string[]): Promise<{ success: boolean; watching?: string[]; message?: string }> {
        return invoke("start_file_watcher", { watchPaths });
    },

    async stopFileWatcher(): Promise<{ success: boolean; message?: string }> {
        return invoke("stop_file_watcher");
    },

    async getFileEvents(clear?: boolean): Promise<Types.FileEventsResponse> {
        return invoke("get_file_events", { clear });
    },

    async getWatcherStatus(): Promise<Types.WatcherStatus> {
        return invoke("get_watcher_status");
    },

    // Learning system integration
    async logSessionStart(sessionId: string, startedAt: string): Promise<void> {
        // Placeholder for backend integration
        return invoke("coach_start_session", { sessionId, startedAt });
    },
    async logSessionEnd(sessionId: string): Promise<void> {
        return invoke("coach_end_session", { sessionId });
    },
    async getLearningSuggestion(query: string, context?: any): Promise<any> {
        return invoke("coach_get_search_tip", { query, context });
    },
    async recordLearningResponse(suggestionType: string, accepted: boolean, originalValue: string, suggestedValue: string, finalValue: string): Promise<void> {
        return invoke("coach_record_response", { suggestionType, accepted, originalValue, suggestedValue, finalValue });
    },

    // Offline sync
    async cacheIndexLocally(indexDir: string, cacheDir: string): Promise<boolean> {
        return invoke("cache_index_locally", { indexDir, cacheDir });
    },
    async exportIndex(indexDir: string, exportPath: string): Promise<boolean> {
        return invoke("export_index", { indexDir, exportPath });
    },
    async importIndex(zipPath: string, targetDir: string): Promise<boolean> {
        return invoke("import_index", { zipPath, targetDir });
    },
};
