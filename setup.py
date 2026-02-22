from setuptools import setup, find_packages

setup(
    name="md-scanner",
    version="0.1.0",
    description="Intelligent markdown file scanner and organizer for ADHD minds",
    author="Claude",
    author_email="noreply@anthropic.com",
    url="https://github.com/yourusername/md-scanner",
    packages=find_packages(),
    install_requires=[
        "sentence-transformers>=2.2.0",
        "numpy>=1.21.0",
        "scikit-learn>=1.0.0",
        "click>=8.0.0",
        "pydantic>=1.9.0",
        "tqdm>=4.60.0",
    ],
    entry_points={
        "console_scripts": [
            "md-scanner=md_scanner.cli:cli",
        ],
    },
    python_requires=">=3.8",
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Topic :: Office/Business",
        "Topic :: Text Editors",
    ],
)
