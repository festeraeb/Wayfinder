# DOCX Extractor stub
# Uses python-docx to extract text from .docx files

def extract_docx_text(file_path):
    try:
        from docx import Document
        doc = Document(file_path)
        return "\n".join([p.text for p in doc.paragraphs])
    except ImportError:
        raise RuntimeError("python-docx not installed")
    except Exception as e:
        return f"[DOCX extraction error: {e}]"

