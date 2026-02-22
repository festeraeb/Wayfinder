import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './GitAssistant.css';

interface GitStatus {
    is_repo: boolean;
    branch: string;
    uncommitted_files: number;
    staged_files: number;
    untracked_files: number;
    days_since_commit: number;
    last_commit_message: string | null;
    last_commit_date: string | null;
}

interface ClippyAction {
    label: string;
    action_type: string;
    data?: any;
}

interface ClippySuggestion {
    id: string;
    icon: string;
    title: string;
    description: string;
    actions: ClippyAction[];
    priority: number;
}

interface DuplicateFile {
    original: string;
    duplicates: string[];
    content_hash: string;
}

interface CommitSuggestion {
    files: string[];
    suggested_message: string;
    category: string;
}

interface FileSuggestion {
    file_path: string;
    suggestion: string;
    action: string;
    reason: string;
}

interface GitClippyReport {
    status: GitStatus;
    urgency_level: string;
    message: string;
    suggestions: ClippySuggestion[];
    duplicates: DuplicateFile[];
    commit_suggestions: CommitSuggestion[];
    copy_pattern_files: FileSuggestion[];
}

interface Props {
    repoPath: string;
    indexPath: string;
}

export const GitAssistant: React.FC<Props> = ({ repoPath, indexPath }) => {
    const [report, setReport] = useState<GitClippyReport | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [showDuplicates, setShowDuplicates] = useState(false);
    const [showCommitSuggestions, setShowCommitSuggestions] = useState(false);
    const [showCopyPatterns, setShowCopyPatterns] = useState(false);
    const [dismissedSuggestions, setDismissedSuggestions] = useState<Set<string>>(new Set());
    const [actionResult, setActionResult] = useState<string | null>(null);

    const loadReport = async () => {
        if (!repoPath) return;

        setLoading(true);
        setError(null);

        try {
            const result = await invoke<GitClippyReport>('get_git_clippy_report', {
                repoPath,
                indexDir: indexPath || null
            });
            setReport(result);
        } catch (e) {
            setError(String(e));
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        loadReport();
    }, [repoPath, indexPath]);

    const handleAction = async (action: ClippyAction, suggestionId?: string) => {
        if (action.action_type === 'dismiss' && suggestionId) {
            dismissSuggestion(suggestionId);
            return;
        }

        if (action.action_type === 'show_duplicates') {
            setShowDuplicates(true);
            return;
        }

        if (action.action_type === 'show_copies') {
            setShowCopyPatterns(true);
            return;
        }

        if (action.action_type === 'cleanup') {
            // Direct cleanup - delete all duplicates
            try {
                const duplicatePaths = report?.duplicates.flatMap(d => d.duplicates) || [];
                if (duplicatePaths.length === 0) {
                    setActionResult('No duplicates to clean up');
                    return;
                }
                const result = await invoke<{ success: boolean, deleted: number, errors: string[] }>('delete_duplicate_files', {
                    filePaths: duplicatePaths
                });
                setActionResult(`Deleted ${result.deleted} duplicate files`);
                setTimeout(() => {
                    setActionResult(null);
                    loadReport();
                }, 2000);
            } catch (e) {
                setError(String(e));
            }
            return;
        }

        if (action.action_type === 'commit' || action.action_type === 'review') {
            setShowCommitSuggestions(true);
            return;
        }

        try {
            const result = await invoke<{ success: boolean, output: string }>('execute_clippy_action', {
                repoPath,
                action: action.action_type,
                data: action.data
            });

            setActionResult(result.output);

            // Refresh report after action
            setTimeout(() => {
                setActionResult(null);
                loadReport();
            }, 2000);
        } catch (e) {
            setError(String(e));
        }
    };

    const dismissSuggestion = (id: string) => {
        setDismissedSuggestions(prev => new Set([...prev, id]));
    };

    const getUrgencyColor = (level: string) => {
        switch (level) {
            case 'existential_crisis': return '#9b59b6';
            case 'panic': return '#e74c3c';
            case 'warning': return '#f39c12';
            case 'nudge': return '#3498db';
            default: return '#2ecc71';
        }
    };

    const getUrgencyEmoji = (level: string) => {
        switch (level) {
            case 'existential_crisis': return '😰';
            case 'panic': return '🚨';
            case 'warning': return '⚠️';
            case 'nudge': return '💡';
            default: return '✨';
        }
    };

    const getUrgencyLabel = (level: string) => {
        switch (level) {
            case 'existential_crisis': return 'HELP';
            case 'panic': return 'PANIC';
            case 'warning': return 'Warning';
            case 'nudge': return 'Nudge';
            default: return 'All Good';
        }
    };

    if (!repoPath) {
        return (
            <div className="git-assistant">
                <div className="git-not-configured">
                    <span className="clippy-icon">📎</span>
                    <p>Scan a directory first to enable Git Assistant</p>
                </div>
            </div>
        );
    }

    if (loading) {
        return (
            <div className="git-assistant">
                <div className="git-loading">
                    <span className="clippy-icon spinning">📎</span>
                    <p>Analyzing your repository...</p>
                </div>
            </div>
        );
    }

    if (error) {
        return (
            <div className="git-assistant">
                <div className="git-error">
                    <span className="clippy-icon">📎</span>
                    <p>Oops! {error}</p>
                    <button className="action-btn primary" onClick={loadReport}>Try again</button>
                </div>
            </div>
        );
    }

    if (!report) {
        return null;
    }

    if (!report.status.is_repo) {
        return (
            <div className="git-assistant">
                <div className="git-not-repo">
                    <span className="clippy-icon">📎</span>
                    <h3>Not a Git Repository</h3>
                    <p>This folder isn't tracked by git yet.</p>
                    <button
                        className="action-btn primary"
                        onClick={() => handleAction({ label: 'Init', action_type: 'git_init' })}
                    >
                        Initialize Git Repository
                    </button>
                </div>
            </div>
        );
    }

    const visibleSuggestions = report.suggestions.filter(s => !dismissedSuggestions.has(s.id));

    return (
        <div className="git-assistant">
            {/* Action Result Toast */}
            {actionResult && (
                <div className="action-toast">
                    ✅ {actionResult}
                </div>
            )}

            {/* Header with Clippy message */}
            <div className={`clippy-header urgency-${report.urgency_level}`} style={{ borderLeftColor: getUrgencyColor(report.urgency_level) }}>
                <span className="clippy-icon">{getUrgencyEmoji(report.urgency_level)}</span>
                <div className="clippy-message">
                    <p style={{ whiteSpace: 'pre-line' }}>{report.message}</p>
                    {report.urgency_level === 'existential_crisis' || report.urgency_level === 'panic' ? (
                        <span className="urgency-badge" style={{
                            background: getUrgencyColor(report.urgency_level),
                            color: 'white',
                            padding: '4px 8px',
                            borderRadius: '4px',
                            fontSize: '12px',
                            fontWeight: 'bold',
                            marginTop: '8px',
                            display: 'inline-block'
                        }}>
                            {getUrgencyLabel(report.urgency_level)}
                        </span>
                    ) : null}
                </div>
            </div>

            {/* Quick Stats */}
            <div className="git-stats">
                <div className="stat">
                    <span className="stat-icon">🌿</span>
                    <span className="stat-value">{report.status.branch || 'N/A'}</span>
                    <span className="stat-label">branch</span>
                </div>
                <div className="stat">
                    <span className="stat-icon">📝</span>
                    <span className="stat-value">{report.status.uncommitted_files}</span>
                    <span className="stat-label">uncommitted</span>
                </div>
                <div className="stat">
                    <span className="stat-icon">⏰</span>
                    <span className="stat-value">{report.status.days_since_commit}d</span>
                    <span className="stat-label">since commit</span>
                </div>
                {report.duplicates.length > 0 && (
                    <div className="stat warning">
                        <span className="stat-icon">🗑️</span>
                        <span className="stat-value">
                            {report.duplicates.reduce((acc, d) => acc + d.duplicates.length, 0)}
                        </span>
                        <span className="stat-label">duplicates</span>
                    </div>
                )}
            </div>

            {/* Suggestions */}
            {visibleSuggestions.length > 0 && (
                <div className="suggestions">
                    {visibleSuggestions.map(suggestion => (
                        <div key={suggestion.id} className={`suggestion priority-${suggestion.priority}`}>
                            <div className="suggestion-header">
                                <span className="suggestion-icon">{suggestion.icon}</span>
                                <h4>{suggestion.title}</h4>
                                <button
                                    className="dismiss-btn"
                                    onClick={() => dismissSuggestion(suggestion.id)}
                                    title="Dismiss"
                                >
                                    ×
                                </button>
                            </div>
                            <p className="suggestion-desc">{suggestion.description}</p>
                            <div className="suggestion-actions">
                                {suggestion.actions.map((action, i) => (
                                    <button
                                        key={i}
                                        className={`action-btn ${i === 0 ? 'primary' : 'secondary'}`}
                                        onClick={() => handleAction(action, suggestion.id)}
                                    >
                                        {action.label}
                                    </button>
                                ))}
                            </div>
                        </div>
                    ))}
                </div>
            )}

            {/* No suggestions message */}
            {visibleSuggestions.length === 0 && (
                <div className="no-suggestions">
                    <span className="big-emoji">🎉</span>
                    <p>Looking good! No suggestions at the moment.</p>
                </div>
            )}

            {/* Duplicates Modal */}
            {showDuplicates && report.duplicates.length > 0 && (
                <div className="modal-overlay" onClick={() => setShowDuplicates(false)}>
                    <div className="modal" onClick={e => e.stopPropagation()}>
                        <div className="modal-header">
                            <h3>🗑️ Duplicate Files</h3>
                            <button className="close-btn" onClick={() => setShowDuplicates(false)}>×</button>
                        </div>
                        <div className="modal-content">
                            {report.duplicates.map((dup, i) => (
                                <div key={i} className="duplicate-group">
                                    <div className="original">
                                        <span className="label">✓ Keep:</span>
                                        <span className="path">{dup.original}</span>
                                    </div>
                                    {dup.duplicates.map((d, j) => (
                                        <div key={j} className="duplicate">
                                            <span className="label">✗ Delete:</span>
                                            <span className="path">{d}</span>
                                        </div>
                                    ))}
                                </div>
                            ))}
                        </div>
                        <div className="modal-footer">
                            <button className="action-btn secondary" onClick={() => setShowDuplicates(false)}>
                                Cancel
                            </button>
                            <button
                                className="action-btn danger"
                                onClick={async () => {
                                    try {
                                        const duplicatePaths = report.duplicates.flatMap(d => d.duplicates);
                                        const result = await invoke<{ success: boolean, deleted: number, errors: string[] }>('delete_duplicate_files', {
                                            filePaths: duplicatePaths
                                        });
                                        setActionResult(`Deleted ${result.deleted} files`);
                                        setShowDuplicates(false);
                                        setTimeout(() => {
                                            setActionResult(null);
                                            loadReport();
                                        }, 2000);
                                    } catch (e) {
                                        setError(String(e));
                                    }
                                }}
                            >
                                Delete All Duplicates
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Commit Suggestions Modal */}
            {showCommitSuggestions && (
                <div className="modal-overlay" onClick={() => setShowCommitSuggestions(false)}>
                    <div className="modal" onClick={e => e.stopPropagation()}>
                        <div className="modal-header">
                            <h3>📝 Smart Commit Suggestions</h3>
                            <button className="close-btn" onClick={() => setShowCommitSuggestions(false)}>×</button>
                        </div>
                        <div className="modal-content">
                            {report.commit_suggestions.length === 0 ? (
                                <p>No specific commit suggestions. Consider committing all changes as WIP.</p>
                            ) : (
                                report.commit_suggestions.map((suggestion, i) => (
                                    <div key={i} className="commit-suggestion">
                                        <div className="commit-message">
                                            <span className="category">{suggestion.category}</span>
                                            <input
                                                type="text"
                                                defaultValue={suggestion.suggested_message}
                                                className="message-input"
                                            />
                                        </div>
                                        <div className="commit-files">
                                            {suggestion.files.slice(0, 10).map((file, j) => (
                                                <div key={j} className="file">
                                                    <input type="checkbox" defaultChecked />
                                                    <span>{file}</span>
                                                </div>
                                            ))}
                                            {suggestion.files.length > 10 && (
                                                <div className="file more">
                                                    ... and {suggestion.files.length - 10} more files
                                                </div>
                                            )}
                                        </div>
                                        <button
                                            className="action-btn primary"
                                            onClick={() => handleAction({
                                                label: 'Commit',
                                                action_type: 'commit',
                                                data: { message: suggestion.suggested_message }
                                            })}
                                        >
                                            Commit These Files
                                        </button>
                                    </div>
                                ))
                            )}

                            <div className="wip-option">
                                <h4>Or just save everything:</h4>
                                <button
                                    className="action-btn secondary full-width"
                                    onClick={() => handleAction({ label: 'WIP', action_type: 'wip_commit' })}
                                >
                                    🚀 Commit All as WIP
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            )}

            {/* Last commit info */}
            {report.status.last_commit_message && (
                <div className="last-commit">
                    <span className="label">Last commit:</span>
                    <span className="message">"{report.status.last_commit_message}"</span>
                </div>
            )}

            {/* Copy Patterns Modal */}
            {showCopyPatterns && report.copy_pattern_files && report.copy_pattern_files.length > 0 && (
                <div className="modal-overlay" onClick={() => setShowCopyPatterns(false)}>
                    <div className="modal" onClick={e => e.stopPropagation()}>
                        <div className="modal-header">
                            <h3>📋 Files with Copy/Backup Patterns</h3>
                            <button className="close-btn" onClick={() => setShowCopyPatterns(false)}>×</button>
                        </div>
                        <div className="modal-content">
                            <p className="modal-description">
                                These files have naming patterns like '_copy', '_backup', '_old', etc.
                                Consider using git branches instead!
                            </p>
                            {report.copy_pattern_files.map((file, i) => (
                                <div key={i} className="copy-pattern-item">
                                    <div className="file-path">{file.file_path}</div>
                                    <div className="file-suggestion">{file.suggestion}</div>
                                    <div className="file-reason">{file.reason}</div>
                                </div>
                            ))}
                        </div>
                        <div className="modal-footer">
                            <button className="action-btn secondary" onClick={() => setShowCopyPatterns(false)}>
                                Close
                            </button>
                            <button
                                className="action-btn primary"
                                onClick={() => {
                                    window.open('https://git-scm.com/book/en/v2/Git-Branching-Branches-in-a-Nutshell', '_blank');
                                }}
                            >
                                Learn Git Branching
                            </button>
                        </div>
                    </div>
                </div>
            )}

            <button className="refresh-btn" onClick={loadReport}>
                🔄 Refresh
            </button>
        </div>
    );
};

export default GitAssistant;
