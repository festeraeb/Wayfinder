# PDF Extractor stub
# Uses PyPDF2 to extract text from .pdf files

def extract_pdf_text(file_path):
    try:
        import PyPDF2
        with open(file_path, 'rb') as f:
            reader = PyPDF2.PdfReader(f)
            return "\n".join([page.extract_text() or "" for page in reader.pages])
    except ImportError:
        raise RuntimeError("PyPDF2 not installed")
    except Exception as e:
        return f"[PDF extraction error: {e}]"

