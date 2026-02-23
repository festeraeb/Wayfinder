import google.generativeai as genai
import os

# Get API key from environment (uses GOOGLE_API_KEY or falls back to service account)
# For service account, we need to use a different approach
from google.auth import default

print("Setting up Google Generative AI credentials...")
credentials, project = default()
genai.configure(credentials=credentials)

print("Testing embedding model...")
result = genai.embed_content(
    model="models/embedding-001",
    content="Hello world"
)

print("✓ Embeddings API working!")
print(f"  Dimension: {len(result['embedding'])}")
print(f"  Sample values: {result['embedding'][:5]}")
