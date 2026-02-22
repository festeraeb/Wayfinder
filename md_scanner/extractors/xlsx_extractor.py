# XLSX Extractor stub
# Uses openpyxl to extract text from .xlsx files

def extract_xlsx_text(file_path):
    try:
        import openpyxl
        wb = openpyxl.load_workbook(file_path, read_only=True)
        text = []
        for ws in wb.worksheets:
            for row in ws.iter_rows(values_only=True):
                text.append("\t".join([str(cell) if cell is not None else "" for cell in row]))
        return "\n".join(text)
    except ImportError:
        raise RuntimeError("openpyxl not installed")
    except Exception as e:
        return f"[XLSX extraction error: {e}]"

