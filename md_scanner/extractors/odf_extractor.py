# ODF Extractor stub
# Uses odfpy to extract text from .odt, .ods, .odp files

def extract_odf_text(file_path):
    try:
        from odf.opendocument import load
        from odf.text import P
        doc = load(file_path)
        text = []
        for p in doc.getElementsByType(P):
            text.append(str(p))
        return "\n".join(text)
    except ImportError:
        raise RuntimeError("odfpy not installed")
    except Exception as e:
        return f"[ODF extraction error: {e}]"

