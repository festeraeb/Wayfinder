"""
Markdown Scanner & Adaptive Workflow Assistant

An intelligent markdown file organization and discovery tool designed for ADHD minds
that generate ideas faster than they can organize them.
"""

__version__ = "0.1.0"
__author__ = "Claude"
all
# Prewarm torch/model at startup if run as main
if __name__ == "__main__":
    try:
        from .embeddings import prewarm_torch
        prewarm_torch()
    except Exception as e:
        print(f"[Startup] Prewarm failed: {e}")
