import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
// import { readTextFile, writeTextFile, desktopDir } from '@tauri-apps/plugin-fs';
import './TaskDecomposerPanel.css';

interface Task {
    number: number;
    title: string;
    file_to_edit: string;
    why: string;
    steps: string[];
    code_stub: string;
    tests: string[];
    error_message: string;
    estimated_time: string;
    dependencies: string[];
}

interface DecomposedIdea {
    title: string;
    description: string;
    tasks: Task[];
    generated_at: string;
}

interface TaskDecomposerPanelProps {
    indexPath?: string;
}

export const TaskDecomposerPanel: React.FC<TaskDecomposerPanelProps> = ({ indexPath = "" }) => {
    const [ideaDescription, setIdeaDescription] = useState<string>("");
    const [model, setModel] = useState<string>("gpt-4o-mini");
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [decomposition, setDecomposition] = useState<DecomposedIdea | null>(null);
    const [exportLoading, setExportLoading] = useState(false);

    const handleGenerate = async () => {
        if (!ideaDescription.trim()) {
            setError("Please describe your idea");
            return;
        }

        if (!indexPath) {
            setError("Index directory not found");
            return;
        }

        setLoading(true);
        setError(null);

        try {
            const result = await invoke<DecomposedIdea>('decompose_idea', {
                indexDir: indexPath,
                idea: ideaDescription,
                model: model,
            });
            setDecomposition(result);
        } catch (e) {
            setError(`Failed to generate tasks: ${String(e)}`);
        } finally {
            setLoading(false);
        }
    };

    const handleExport = async () => {
        if (!decomposition) return;

        setExportLoading(true);
        try {
            const markdown = generateMarkdown(decomposition);
            const filename = `tasks_${Date.now()}.md`;
            
            // Create a blob and download using the browser's built-in functionality
            const blob = new Blob([markdown], { type: 'text/markdown' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = filename;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
            
            alert(`Exported: ${filename}`);
        } catch (e) {
            setError(`Failed to export: ${String(e)}`);
        } finally {
            setExportLoading(false);
        }
    };

    const generateMarkdown = (decomp: DecomposedIdea): string => {
        let md = `# Task Decomposition: ${decomp.title}\n\n`;
        md += `**Description**: ${decomp.description}\n\n`;
        md += `**Generated**: ${decomp.generated_at}\n\n`;
        md += `---\n\n`;

        for (const task of decomp.tasks) {
            md += `## Task ${task.number}: ${task.title}\n\n`;
            md += `**File to Edit**: \`${task.file_to_edit}\`\n\n`;
            md += `**Why This Matters**:\n${task.why}\n\n`;
            md += `**Exactly What To Do**:\n`;
            task.steps.forEach((step, i) => {
                md += `${i + 1}. ${step}\n`;
            });
            md += `\n**Code Stub**:\n\`\`\`\n${task.code_stub}\n\`\`\`\n\n`;
            md += `**Tests To Pass**:\n`;
            task.tests.forEach(test => {
                md += `- [ ] ${test}\n`;
            });
            md += `\n**If This Breaks, Error Message Should Be**:\n"${task.error_message}"\n\n`;
            md += `**Estimated Time**: ${task.estimated_time}\n\n`;
            if (task.dependencies.length > 0) {
                md += `**Dependencies**: ${task.dependencies.join(', ')}\n\n`;
            }
            md += `---\n\n`;
        }

        return md;
    };

    return (
        <div className="task-decomposer-panel">
            <h2>🧠 Task Decomposer</h2>
            <p className="description">
                Convert your ideas into hyper-specific tasks for 0.5 agents. 
                Outputs 8 concrete tasks with code stubs, test criteria, and error handling.
            </p>

            <div className="decomposer-form">
                <div className="form-group">
                    <label htmlFor="idea">Your Idea (be specific!):</label>
                    <textarea
                        id="idea"
                        value={ideaDescription}
                        onChange={(e) => setIdeaDescription(e.target.value)}
                        placeholder="e.g., 'Add a backup system that auto-snapshots every 5 minutes, keeps 30 snapshots, with restore from UI'"
                        rows={5}
                        disabled={loading}
                    />
                </div>

                <div className="form-group">
                    <label htmlFor="model">Model:</label>
                    <select
                        id="model"
                        value={model}
                        onChange={(e) => setModel(e.target.value)}
                        disabled={loading}
                    >
                        <option value="gpt-4o-mini">GPT-4o Mini (GitHub Models, free)</option>
                        <option value="claude-3.5-sonnet">Claude 3.5 Sonnet (free tier)</option>
                        <option value="ollama-local">Ollama Local (if running)</option>
                    </select>
                </div>

                <button
                    className="btn-primary"
                    onClick={handleGenerate}
                    disabled={loading || !ideaDescription.trim()}
                >
                    {loading ? "Generating Tasks..." : "Generate Task Decomposition"}
                </button>
            </div>

            {error && (
                <div className="error-box">
                    <strong>Error:</strong> {error}
                </div>
            )}

            {decomposition && (
                <div className="decomposition-results">
                    <div className="results-header">
                        <h3>{decomposition.title}</h3>
                        <button
                            className="btn-secondary"
                            onClick={handleExport}
                            disabled={exportLoading}
                        >
                            {exportLoading ? "Exporting..." : "📥 Export as Markdown"}
                        </button>
                    </div>

                    <p className="decomp-description">{decomposition.description}</p>

                    <div className="tasks-container">
                        {decomposition.tasks.map((task) => (
                            <div key={task.number} className="task-card">
                                <div className="task-header">
                                    <h4>Task {task.number}: {task.title}</h4>
                                    <span className="time-estimate">{task.estimated_time}</span>
                                </div>

                                <div className="task-content">
                                    <div className="task-section">
                                        <strong>File:</strong> <code>{task.file_to_edit}</code>
                                    </div>

                                    <div className="task-section">
                                        <strong>Why:</strong> {task.why}
                                    </div>

                                    <div className="task-section">
                                        <strong>Steps:</strong>
                                        <ol>
                                            {task.steps.map((step, i) => (
                                                <li key={i}>{step}</li>
                                            ))}
                                        </ol>
                                    </div>

                                    <div className="task-section">
                                        <strong>Code Stub:</strong>
                                        <pre className="code-stub">{task.code_stub}</pre>
                                    </div>

                                    <div className="task-section">
                                        <strong>Tests:</strong>
                                        <ul>
                                            {task.tests.map((test, i) => (
                                                <li key={i}>{test}</li>
                                            ))}
                                        </ul>
                                    </div>

                                    {task.dependencies.length > 0 && (
                                        <div className="task-section">
                                            <strong>Dependencies:</strong> {task.dependencies.join(', ')}
                                        </div>
                                    )}
                                </div>
                            </div>
                        ))}
                    </div>

                    <div className="instructions-box">
                        <h4>Next Steps:</h4>
                        <ol>
                            <li>Export these tasks to a markdown file</li>
                            <li>Review the task descriptions and adjust if needed</li>
                            <li>Hand to a 0.5 agent (Claude 3.5 Sonnet, Llama, etc.)</li>
                            <li>Agent codes each task independently</li>
                            <li>You review and merge the code</li>
                        </ol>
                    </div>
                </div>
            )}
        </div>
    );
};

export default TaskDecomposerPanel;
