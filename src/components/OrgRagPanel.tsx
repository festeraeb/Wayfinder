import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './OrgRagPanel.css';

interface RagContextData {
    context_document: string;
    summary: {
        total_files: number;
        classified_count: number;
        unclassified_count: number;
        recent_moves_count: number;
    };
    ready_for_rag: boolean;
}

interface Props {
    indexPath: string;
}

export const OrgRagPanel: React.FC<Props> = ({ indexPath }) => {
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [ragContext, setRagContext] = useState<RagContextData | null>(null);
    const [copied, setCopied] = useState(false);

    const loadRagContext = async () => {
        if (!indexPath) {
            setError("Please select an index directory first");
            return;
        }

        setLoading(true);
        setError(null);

        try {
            const result = await invoke<RagContextData>('generate_org_rag_context', {
                indexDir: indexPath,
                maxFilesToShow: 20,
            });
            setRagContext(result);
        } catch (e) {
            setError(`Failed to load RAG context: ${String(e)}`);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        loadRagContext();
    }, [indexPath]);

    const handleCopyToClipboard = () => {
        if (ragContext?.context_document) {
            navigator.clipboard.writeText(ragContext.context_document);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        }
    };

    const handleDownload = () => {
        if (ragContext?.context_document) {
            const element = document.createElement('a');
            const file = new Blob([ragContext.context_document], { type: 'text/plain' });
            element.href = URL.createObjectURL(file);
            element.download = `org_rag_context_${Date.now()}.txt`;
            document.body.appendChild(element);
            element.click();
            document.body.removeChild(element);
        }
    };

    const handleRefresh = () => {
        loadRagContext();
    };

    if (error) {
        return (
            <div className="org-rag-panel">
                <div className="error-box">
                    <strong>Error:</strong> {error}
                </div>
                <button className="btn-primary" onClick={handleRefresh}>
                    Retry
                </button>
            </div>
        );
    }

    if (loading || !ragContext) {
        return (
            <div className="org-rag-panel loading">
                <p>Loading current index state...</p>
            </div>
        );
    }

    return (
        <div className="org-rag-panel">
            <div className="rag-header">
                <h2>🧠 ORG System RAG Context</h2>
                <p className="subtitle">
                    This is what cheap models "see" when classifying your files. 
                    Fully grounded in your actual data.
                </p>
            </div>

            <div className="summary-cards">
                <div className="card">
                    <div className="card-value">{ragContext.summary.total_files}</div>
                    <div className="card-label">Total Files</div>
                </div>
                <div className="card">
                    <div className="card-value">{ragContext.summary.classified_count}</div>
                    <div className="card-label">Already Classified</div>
                </div>
                <div className="card warning">
                    <div className="card-value">{ragContext.summary.unclassified_count}</div>
                    <div className="card-label">Need Sorting</div>
                </div>
                <div className="card">
                    <div className="card-value">{ragContext.summary.recent_moves_count}</div>
                    <div className="card-label">Recent Moves</div>
                </div>
            </div>

            <div className="rag-document-section">
                <div className="section-header">
                    <h3>RAG Context Document</h3>
                    <p className="explanation">
                        This document is sent to the cheap model along with your classification task.
                        It includes concrete examples, current state, and recent history.
                        No abstract reasoning—just facts the model can ground in.
                    </p>
                </div>

                <div className="toolbar">
                    <button 
                        className="btn-secondary" 
                        onClick={handleCopyToClipboard}
                        title="Copy to clipboard"
                    >
                        {copied ? "✓ Copied!" : "📋 Copy to Clipboard"}
                    </button>
                    <button 
                        className="btn-secondary" 
                        onClick={handleDownload}
                        title="Download as text file"
                    >
                        📥 Download
                    </button>
                    <button 
                        className="btn-secondary" 
                        onClick={handleRefresh}
                        title="Refresh context"
                    >
                        🔄 Refresh
                    </button>
                </div>

                <pre className="rag-document">{ragContext.context_document}</pre>
            </div>

            <div className="instructions-box">
                <h4>How to Use This with Cheap Models</h4>
                <ol>
                    <li>
                        <strong>Copy the RAG context above</strong> (click "Copy to Clipboard")
                    </li>
                    <li>
                        <strong>Open your model chat</strong> (Claude 3.5 Sonnet, Llama 2, gpt-4o-mini, etc.)
                    </li>
                    <li>
                        <strong>Paste this context</strong> as the system prompt/context
                    </li>
                    <li>
                        <strong>Give your instruction:</strong>
                        <pre className="instruction-example">
"Based on this context and the rules provided, classify these 10 unclassified files 
into projectA or projectB. Return JSON with path and project for each."
                        </pre>
                    </li>
                    <li>
                        <strong>Model responds</strong> with concrete classifications (not abstract reasoning)
                    </li>
                    <li>
                        <strong>You review</strong> the classifications before applying them
                    </li>
                </ol>

                <h4>Why This Works with Cheap Models</h4>
                <ul>
                    <li>
                        ✅ Model sees REAL DATA (files, keywords, examples) not abstract concepts
                    </li>
                    <li>
                        ✅ No complex reasoning needed—just pattern matching from examples
                    </li>
                    <li>
                        ✅ Cheap models are good at "follow this exact pattern" tasks
                    </li>
                    <li>
                        ✅ Grounded in current state → much more likely to stay on task
                    </li>
                </ul>

                <h4>Safety First</h4>
                <ul>
                    <li>
                        ✅ Always review model output before applying moves
                    </li>
                    <li>
                        ✅ Use the "dry run" mode to preview changes
                    </li>
                    <li>
                        ✅ Keep the move history (you can undo anything)
                    </li>
                </ul>
            </div>
        </div>
    );
};

export default OrgRagPanel;
