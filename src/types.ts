// TypeScript types for Tauri commands and responses

export interface ScanResult {
    files_scanned: number;
    total_size: number;
    index_path: string;
}

export interface EmbedResult {
    embeddings_generated: number;
    cached_count: number;
    message?: string;
}

export interface ClusterResult {
    clusters_created: number;
    total_files: number;
    message?: string;
}

export interface SearchResult {
    path: string;
    name: string;
    score: number;
    preview?: string;
}

export interface ClusterInfo {
    id: number;
    size: number;
    sample_files: string[];
    summary?: string;
    label?: string;
    file_count?: number;
    keywords?: string[];
    files?: Array<{ name: string; path: string }>;

}

export interface TimelineEntry {
    date: string;
    file_count: number;
    files: { name: string; path: string }[];
}

export interface TimelineData {
    entries?: TimelineEntry[];
    timeline?: TimelineEntry[];
}

export interface IndexStats {
    total_files: number;
    total_size_bytes: number;
    extensions: Record<string, number>;
    last_updated: string;
    scan_path?: string;
    has_embeddings?: boolean;
    has_clusters?: boolean;
    cluster_count?: number;
    embeddings_count?: number;
    age_buckets?: AgeBucket[];
}

export interface AgeBucket {
    label: string;
    count: number;
}

export interface IndexState {
    has_files: boolean;
    index_valid: boolean;
    message: string;
}

export interface SystemInfo {
    os: string;
    arch: string;
    device_type?: string;
}

// Cluster summary response
export interface ClustersSummary {
    clusters?: ClusterInfo[];
}

// Azure OpenAI config status
export interface AzureConfigStatus {
    configured: boolean;
    endpoint: string;
    deployment_name: string;
    api_version: string;
    has_key: boolean;
}

// Google Cloud Vertex AI config status
export interface GcpConfigStatus {
    configured: boolean;
    project_id: string;
    location: string;
    model_id: string;
    endpoint?: string;
    has_key: boolean; // Indicates if a service account path is stored locally
}

export type EmbeddingProvider = "local" | "llama" | "azure" | "gcp";

export interface ProviderConfigStatus {
    provider: EmbeddingProvider;
    local_model?: string;
    local_endpoint?: string;
    exists: boolean;
}

export interface AzureValidationResult {
    success: boolean;
    message: string;
    suggested_endpoint?: string;
    tried_versions?: string[];
    final_url?: string;
    status_code?: number;
}

export interface GcpValidationResult {
    success: boolean;
    message: string;
    project_id?: string;
    location?: string;
    model_id?: string;
    endpoint?: string;
}

// File watcher types
export type FileEventType = "Created" | "Modified" | { Renamed: { from: string } };

export interface FileEvent {
    path: string;
    file_name: string;
    event_type: FileEventType;
    doc_type: string;
    timestamp: string;
}

export interface FileEventsResponse {
    success: boolean;
    event_count: number;
    events: FileEvent[];
}

export interface WatcherStatus {
    success: boolean;
    is_running: boolean;
    watched_paths: string[];
    pending_events: number;
}

// Clusters data response
export interface ClustersData {
    has_clusters: boolean;
    clusters: {
        id: number;
        file_count: number;
        files: string[];
        label?: string;
    }[];
    created_at?: string;
}

// UI State types
export interface OperationProgress {
    operation: string;
    current: number;
    total: number;
    percent: number;
    status: "pending" | "running" | "complete" | "error";
    error?: string;
}

export interface BatchProgress {
    batch_id: string;
    total_files: number;
    processed_files: number;
    current_batch: number;
    total_batches: number;
    batch_size: number;
    status: string; // running | paused | complete | error
    started_at: string;
    last_updated: string;
    errors: string[];
}

// Git Assistant types
export interface GitStatus {
    is_repo: boolean;
    branch: string;
    uncommitted_files: number;
    staged_files: number;
    untracked_files: number;
    days_since_commit: number;
    last_commit_message: string | null;
    last_commit_date: string | null;
}

export interface ClippyAction {
    label: string;
    action_type: string;
    data?: any;
}

export interface ClippySuggestion {
    id: string;
    icon: string;
    title: string;
    description: string;
    actions: ClippyAction[];
    priority: number;
}

export interface DuplicateFile {
    original: string;
    duplicates: string[];
    content_hash: string;
    size_bytes: number;
    kept_mtime?: string;
    original_meta?: DuplicateMeta;
    duplicate_meta?: DuplicateMeta[];
}

export interface DuplicateMeta {
    path: string;
    size_bytes: number;
    modified?: string;
}

export interface FileSuggestion {
    file_path: string;
    suggestion: string;
    action: string;
    reason: string;
}

export interface CommitSuggestion {
    files: string[];
    suggested_message: string;
    category: string;
}

export interface GitClippyReport {
    status: GitStatus;
    urgency_level: string;
    message: string;
    suggestions: ClippySuggestion[];
    duplicates: DuplicateFile[];
    commit_suggestions: CommitSuggestion[];
}

export interface NautiReport {
    message: string;
    scanned_files: number;
    duplicates: DuplicateFile[];
    copy_pattern_files: FileSuggestion[];
}

// Reference insights and move planning
export interface RefNeighbor {
    path: string;
    score: number;
}

export interface FileInsight {
    path: string;
    imports: string[];
    mentions: string[];
    semantic_neighbors: RefNeighbor[];
    keywords: string[];
    score: number;
}

export interface MoveAction {
    action: string;
    from?: string;
    to?: string;
    path?: string;
    shim_content?: string;
    note?: string;
}

export interface MovePlan {
    plan_id: string;
    generated_at: string;
    target_root: string;
    steps: MoveAction[];
    revert_steps: MoveAction[];
}

export interface BuildRefIndexResult {
    success: boolean;
    built_at: string;
    refs: number;
}

export interface ApplyMovePlanResult {
    success: boolean;
    errors: string[];
    applied_plan: MovePlan;
}

export interface UndoMovePlanResult {
    success: boolean;
    errors: string[];
    reverted_plan: MovePlan;
}
