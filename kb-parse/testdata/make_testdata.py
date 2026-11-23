"""Generates the package's own test files in this folder (see ../README.md, "Eigene Testdaten").

Run from any folder with Python 3.10 or later:

    python make_testdata.py

Needs openpyxl, python-docx, python-pptx, msoffcrypto-tool, Pillow, numpy, pypdf and
PyMuPDF.
Raster text uses the Arial font of Windows (C:/Windows/Fonts/arial.ttf). All contents
are made up by this script; the files are written next to it, together with
expected.json (page sizes and the ink boxes of the marker words, measured on the
rendered displayed page).
"""

import io
import json
import zipfile
import zlib
from pathlib import Path

import fitz  # PyMuPDF, used only to render pages for the expected boxes
from msoffcrypto.format.ooxml import OOXMLFile
import numpy
import openpyxl
from docx import Document
from docx.oxml import OxmlElement
from docx.oxml.ns import qn
from openpyxl.styles.numbers import BUILTIN_FORMATS
from PIL import Image, ImageDraw, ImageFont
from pptx import Presentation
from pptx.util import Inches, Pt
from pypdf import PdfReader, PdfWriter

HERE = Path(__file__).resolve().parent
FONT = "C:/Windows/Fonts/arial.ttf"
EXPECTED = {}


def font(size):
    return ImageFont.truetype(FONT, size)


# ---------------------------------------------------------------- XLSX


def set_cached_value(path, sheet_xml, cell, value):
    """openpyxl writes formulas without their result; Excel stores both."""
    data = io.BytesIO(path.read_bytes())
    out = io.BytesIO()
    with zipfile.ZipFile(data) as src, zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as dst:
        for item in src.infolist():
            content = src.read(item.filename)
            if item.filename == sheet_xml:
                text = content.decode("utf-8")
                start = text.index(f'<c r="{cell}"')
                end = text.index("</c>", start)
                text = text[:end] + f"<v>{value}</v>" + text[end:]
                content = text.encode("utf-8")
            dst.writestr(item, content)
    path.write_bytes(out.getvalue())


def make_formats():
    wb = openpyxl.Workbook()
    ws = wb.active
    ws.title = "Formate"
    ws.append(["Fall", "Wert"])
    rows = [
        ("tausender", "#,##0", 12000),
        ("dezimal", "#,##0.00", 1234.5),
        ("eine-stelle", "0.0", 648.686708086533),
        ("euro-gebiet", "#,##0.00 [$€-407]", 1234.5),
        ("euro-vorn", '"€"#,##0.00', -5),
        ("klammer", "#,##0_);(#,##0)", -1500),
        ("eingebaut-37", BUILTIN_FORMATS[37], -1234.5),
        ("eingebaut-39", BUILTIN_FORMATS[39], -1234.5),
        ("prozent", "0.00%", 0.1234),
        ("text", "@", "00123"),
    ]
    for label, fmt, value in rows:
        ws.append([label, value])
        cell = ws.cell(row=ws.max_row, column=2)
        if label == "klammer":
            # openpyxl would store this code as built-in format 37; Excel stores a code
            # typed by the user as a format of its own (id 164 and up).
            cell.number_format = "0"
            cell._style.numFmtId = wb._number_formats.add(fmt) + 164
        else:
            cell.number_format = fmt
    ws.append(["basis", 1234.5])
    ws.append(["formel", "=B12*2"])
    ws.cell(row=ws.max_row, column=2).number_format = "#,##0.00"
    path = HERE / "formats.xlsx"
    wb.save(path)
    set_cached_value(path, "xl/worksheets/sheet1.xml", "B13", 2469)


def make_tables():
    wb = openpyxl.Workbook()

    def sheet(name, rows, merge=None):
        ws = wb.create_sheet(name)
        for r, row in enumerate(rows, start=1):
            for c, value in enumerate(row, start=1):
                if value is not None:
                    ws.cell(row=r, column=c, value=value)
        for m in merge or []:
            ws.merge_cells(m)
        return ws

    wb.remove(wb.active)
    sheet("TitelLeerKopf", [["Titel A"], [], ["Name", "Menge", "Preis"], ["Apfel", 3, 1.5], ["Birne", 4, 2.5]])
    sheet("TitelKopf", [["Titel B"], ["Name", "Menge", "Preis"], ["Apfel", 3, 1.5], ["Birne", 4, 2.5]])
    sheet(
        "TitelVerbunden",
        [["Titel C"], ["Name", "Menge", "Preis"], ["Apfel", 3, 1.5], ["Birne", 4, 2.5]],
        merge=["A1:C1"],
    )
    sheet("EinzelwertSpalteB", [[None, "Einzelwert"], ["Name", "Menge", "Preis"], ["Apfel", 3, 1.5]])
    sheet("OhneTitel", [["Name", "Menge", "Preis"], ["Apfel", 3, 1.5], ["Birne", 4, 2.5]])
    sheet(
        "KopfZweiZeilig",
        [["Gruppe", None, "Einzeln"], ["Name", "Menge", "Preis"], ["Apfel", 3, 1.5], ["Birne", 4, 2.5]],
        merge=["A1:B1"],
    )
    sheet("Luecken", [["Name", None, "Preis"], ["Apfel", 3, 1.5], ["Birne", 4, 2.5]])
    sheet("OhneVolleZeile", [["Titel D"], ["Apfel", 3], [None, 4, 2.5]])
    hidden = sheet("Versteckt", [["GEHEIMBLATT-555"], ["noch geheim"]])
    hidden.sheet_state = "hidden"
    ws = sheet(
        "Ausgeblendet",
        [
            ["Name", "Menge", "SPALTE-VERSTECKT-2", "Preis"],
            ["Apfel", 3, "SPALTE-VERSTECKT-3", 1.5],
            ["ZEILE-VERSTECKT-1", 7, "SPALTE-VERSTECKT-4", 9.5],
            ["Birne", 4, "SPALTE-VERSTECKT-5", 2.5],
        ],
    )
    ws.row_dimensions[3].hidden = True
    ws.column_dimensions["C"].hidden = True
    # A merged title directly above the table, wider and narrower than the table.
    sheet(
        "TitelBreit",
        [["Titel E"], ["Name", "Menge", "Preis"], ["Apfel", 3, 1.5], ["Birne", 4, 2.5]],
        merge=["A1:E1"],
    )
    sheet(
        "TitelSchmal",
        [["Titel F"], ["Name", "Menge", "Preis"], ["Apfel", 3, 1.5], ["Birne", 4, 2.5]],
        merge=["A1:B1"],
    )
    # The header row and the first column are hidden.
    ws = sheet(
        "KopfAusgeblendet",
        [
            ["Nr", "KOPF-VERSTECKT-1", "KOPF-VERSTECKT-2", "KOPF-VERSTECKT-3"],
            ["1", "Name", "Menge", "Preis"],
            ["2", "Apfel", 3, 1.5],
            ["3", "Birne", 4, 2.5],
        ],
    )
    ws.row_dimensions[1].hidden = True
    ws.column_dimensions["A"].hidden = True
    wb.save(HERE / "tables.xlsx")


# ---------------------------------------------------------------- PPTX


def add_text(shapes, text, top):
    box = shapes.add_textbox(Inches(1), Inches(top), Inches(6), Inches(0.8))
    box.text_frame.text = text
    return box


def make_slides():
    prs = Presentation()
    blank = prs.slide_layouts[6]
    titled = prs.slide_layouts[5]

    s1 = prs.slides.add_slide(titled)
    s1.shapes.title.text = "Folie Eins"
    add_text(s1.shapes, "Inhalt der ersten Folie", 2)
    s1.notes_slide.notes_text_frame.text = "Notiz zur ersten Folie"

    s2 = prs.slides.add_slide(blank)
    group = s2.shapes.add_group_shape()
    add_text(group.shapes, "GRUPPENTEXT-A", 1)
    inner = group.shapes.add_group_shape()
    add_text(inner.shapes, "VERSCHACHTELT-B", 3)

    s3 = prs.slides.add_slide(blank)
    add_text(s3.shapes, "INHALT-AUSGEBLENDET-789", 2)
    s3.notes_slide.notes_text_frame.text = "NOTIZ-AUSGEBLENDET-321"
    s3._element.set("show", "0")

    s4 = prs.slides.add_slide(titled)
    s4.shapes.title.text = "Folie Vier"
    add_text(s4.shapes, "Inhalt der vierten Folie", 2)
    prs.save(HERE / "slides.pptx")


# ---------------------------------------------------------------- DOCX


def add_hyperlink(paragraph, url, text):
    part = paragraph.part
    r_id = part.relate_to(url, "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink", is_external=True)
    link = OxmlElement("w:hyperlink")
    link.set(qn("r:id"), r_id)
    run = OxmlElement("w:r")
    t = OxmlElement("w:t")
    t.text = text
    run.append(t)
    link.append(run)
    paragraph._p.append(link)


GERMAN_STYLE_IDS = {"Title": "Titel", "Heading1": "berschrift1", "Heading2": "berschrift2"}


def german_style_ids(path):
    """Word in German stores these built-in styles under German ids."""
    data = io.BytesIO(path.read_bytes())
    out = io.BytesIO()
    with zipfile.ZipFile(data) as src, zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as dst:
        for item in src.infolist():
            content = src.read(item.filename)
            if item.filename in ("word/styles.xml", "word/document.xml"):
                text = content.decode("utf-8")
                for old, new in GERMAN_STYLE_IDS.items():
                    text = text.replace(f'w:styleId="{old}"', f'w:styleId="{new}"')
                    text = text.replace(f'<w:pStyle w:val="{old}"/>', f'<w:pStyle w:val="{new}"/>')
                    text = text.replace(f'<w:basedOn w:val="{old}"/>', f'<w:basedOn w:val="{new}"/>')
                    text = text.replace(f'<w:next w:val="{old}"/>', f'<w:next w:val="{new}"/>')
                    text = text.replace(f'<w:link w:val="{old}Char"/>', f'<w:link w:val="{new}Char"/>')
                content = text.encode("utf-8")
            dst.writestr(item, content)
    path.write_bytes(out.getvalue())


def make_docx():
    doc = Document()
    section = doc.sections[0]
    section.header.paragraphs[0].text = "KOPFZEILE-TEST"
    section.footer.paragraphs[0].text = "FUSSZEILE-TEST"
    doc.add_paragraph("Handbuch der Testdatei", style="Title")
    doc.add_paragraph("Einleitender Absatz vor dem ersten Kapitel.")
    doc.add_paragraph("Kapitel Eins", style="Heading 1")
    p = doc.add_paragraph("Dieser Absatz hat einen ")
    add_hyperlink(p, "https://example.org/", "Verweis")
    p.add_run(" und ")
    p.add_run("fett gesetzten").bold = True
    p.add_run(" sowie ")
    p.add_run("kursiven").italic = True
    p.add_run(" Text.")
    doc.add_paragraph("Abschnitt Eins Punkt Eins", style="Heading 2")
    p = doc.add_paragraph("Sichtbarer Satz mit ")
    p.add_run("VERBORGEN-WORT-123 ").font.hidden = True
    p.add_run("weiterem Text.")
    hidden = doc.add_paragraph()
    hidden.add_run("GANZ-VERBORGEN-456").font.hidden = True
    doc.add_paragraph("Kapitel Zwei", style="Heading 1")
    doc.add_paragraph("Schlussabsatz.")
    # Built-in style ids as English Word and python-docx write them; German Word shows
    # them as "Titel", "Überschrift 1" and "Überschrift 2".
    doc.save(HERE / "structure.docx")
    # The same document as German Word stores it: built-in styles under German ids.
    path = HERE / "structure-de.docx"
    doc.save(path)
    german_style_ids(path)

    encrypted = io.BytesIO()
    plain = io.BytesIO()
    small = Document()
    small.add_paragraph("Vertraulicher Inhalt")
    small.save(plain)
    plain.seek(0)
    office = OOXMLFile(plain)
    office.encrypt("geheim", encrypted)
    (HERE / "encrypted.docx").write_bytes(encrypted.getvalue())


# ---------------------------------------------------------------- HTML, Markdown


def make_text_formats():
    (HERE / "structure.html").write_text(
        """<!DOCTYPE html>
<html lang="de"><head><meta charset="utf-8"><title>Testseite Struktur</title></head>
<body>
<h1>Handbuch</h1>
<p>Einleitung des Handbuchs.</p>
<h2>Kapitel Eins</h2>
<p>Ein Absatz mit einem <a href="https://example.org/">Verweis</a> und <b>fettem</b> Text.</p>
<h3>Abschnitt Eins Punkt Eins</h3>
<p>Text im Abschnitt.</p>
<h2>Kapitel Zwei</h2>
<p>Text im zweiten Kapitel.</p>
</body></html>
""",
        encoding="utf-8",
        newline="\n",
    )
    (HERE / "landmarks.html").write_text(
        """<!DOCTYPE html>
<html lang="de"><head><meta charset="utf-8"><title>Seite mit Rahmen</title></head>
<body>
<header><nav><a href="/">MENUE-KOPF-11</a> <a href="/a">MENUE-KOPF-12</a></nav></header>
<nav><ul><li>NAVIGATION-21</li><li>NAVIGATION-22</li></ul></nav>
<aside><p>SEITENLEISTE-31</p></aside>
<main>
<p>Einleitung des Hauptteils vor der ersten Überschrift.</p>
<h1>Hauptteil</h1>
<p>Inhalt des Hauptteils.</p>
</main>
<footer><p>FUSSZEILE-41</p></footer>
</body></html>
""",
        encoding="utf-8",
        newline="\n",
    )
    (HERE / "structure.md").write_text(
        "# Handbuch\n\nEinleitung des Handbuchs.\n\n## Kapitel Eins\n\n"
        "Ein Absatz mit einem [Verweis](https://example.org/) und **fettem** Text.\n\n"
        "### Abschnitt Eins Punkt Eins\n\nText im Abschnitt.\n\n## Kapitel Zwei\n\nText im zweiten Kapitel.\n",
        encoding="utf-8",
        newline="\n",
    )


# ---------------------------------------------------------------- PDF writer


class Pdf:
    """A minimal PDF writer: Helvetica text and RGB images, one stream per page."""

    def __init__(self):
        self.objects = []

    def add(self, body):
        self.objects.append(body)
        return len(self.objects)

    def stream(self, data, extra=""):
        compressed = zlib.compress(data)
        return self.add(
            f"<< /Length {len(compressed)} /Filter /FlateDecode {extra}>>\nstream\n".encode("latin-1")
            + compressed
            + b"\nendstream"
        )

    def image(self, img):
        img = img.convert("RGB")
        w, h = img.size
        return self.stream(
            img.tobytes(),
            f"/Type /XObject /Subtype /Image /Width {w} /Height {h} /ColorSpace /DeviceRGB /BitsPerComponent 8 ",
        )

    def write(self, path, pages):
        """pages: list of dicts with media, crop (or None), rotate, content (bytes), images."""
        font = self.add(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>")
        bold = self.add(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>")
        pages_id = len(self.objects) + 2 * len(pages) + 1
        kids = []
        for page in pages:
            content = self.stream(page["content"])
            xobjects = " ".join(f"/Im{i} {obj} 0 R" for i, obj in enumerate(page.get("images", [])))
            box = lambda b: "[" + " ".join(f"{v:g}" for v in b) + "]"
            crop = f"/CropBox {box(page['crop'])} " if page.get("crop") else ""
            kids.append(
                self.add(
                    (
                        f"<< /Type /Page /Parent {pages_id} 0 R /MediaBox {box(page['media'])} {crop}"
                        f"/Rotate {page.get('rotate', 0)} /Contents {content} 0 R "
                        f"/Resources << /Font << /F1 {font} 0 R /F2 {bold} 0 R >> /XObject << {xobjects} >> >> >>"
                    ).encode("latin-1")
                )
            )
        assert self.add(
            f"<< /Type /Pages /Kids [{' '.join(f'{k} 0 R' for k in kids)}] /Count {len(kids)} >>".encode("latin-1")
        ) == pages_id
        catalog = self.add(f"<< /Type /Catalog /Pages {pages_id} 0 R >>".encode("latin-1"))
        out = io.BytesIO()
        out.write(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n")
        offsets = []
        for i, body in enumerate(self.objects, start=1):
            offsets.append(out.tell())
            out.write(f"{i} 0 obj\n".encode("latin-1") + body + b"\nendobj\n")
        xref = out.tell()
        out.write(f"xref\n0 {len(self.objects) + 1}\n0000000000 65535 f \n".encode("latin-1"))
        for off in offsets:
            out.write(f"{off:010d} 00000 n \n".encode("latin-1"))
        out.write(f"trailer\n<< /Size {len(self.objects) + 1} /Root {catalog} 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode("latin-1"))
        path.write_bytes(out.getvalue())


def esc(text):
    return text.replace("\\", "\\\\").replace("(", "\\(").replace(")", "\\)").encode("cp1252")


def frame_transform(rotate, box):
    """Maps a point of the displayed page (x right, y down from its top left) to user
    space, and gives the text direction and the up direction in user space."""
    x0, y0, x1, y1 = box
    if rotate == 0:
        return (lambda dx, dy: (x0 + dx, y1 - dy)), (1, 0), (0, 1)
    if rotate == 90:
        return (lambda dx, dy: (x0 + dy, y0 + dx)), (0, 1), (-1, 0)
    if rotate == 180:
        return (lambda dx, dy: (x1 - dx, y0 + dy)), (-1, 0), (0, -1)
    return (lambda dx, dy: (x1 - dy, y1 - dx)), (0, -1), (1, 0)


def text_op(rotate, box, dx, dy, size, text, rgb, font_name="F1"):
    to_user, (ax, ay), (ux, uy) = frame_transform(rotate, box)
    ex, ey = to_user(dx, dy)
    r, g, b = rgb
    return (
        f"BT /{font_name} {size} Tf {r} {g} {b} rg {ax} {ay} {ux} {uy} {ex:.2f} {ey:.2f} Tm (".encode("latin-1")
        + esc(text)
        + b") Tj ET\n"
    )


FRAME_VARIANTS = [
    ("rotate-0", [0, 0, 595, 842], None, 0),
    ("rotate-90", [0, 0, 595, 842], None, 90),
    ("rotate-180", [0, 0, 595, 842], None, 180),
    ("rotate-270", [0, 0, 595, 842], None, 270),
    ("cropbox", [0, 0, 595, 842], [60, 100, 520, 760], 0),
    ("cropbox-rotate-90", [0, 0, 595, 842], [60, 100, 520, 760], 90),
    ("negative-origin", [-50, -40, 545, 802], None, 0),
]

RED, BLUE = (1, 0, 0), (0, 0, 1)
PARAGRAPH = [
    "Dieser Absatz steht in der Mitte der angezeigten Seite und",
    "dient als gewoehnlicher Fliesstext zwischen den beiden Markierungen.",
    "Die Markierungen stehen oben links und unten rechts der Seite.",
]


def frame_content(rotate, media, crop):
    box = crop or media
    x0, y0, x1, y1 = box
    w, h = (x1 - x0, y1 - y0) if rotate in (0, 180) else (y1 - y0, x1 - x0)
    ops = text_op(rotate, box, 40, 60, 22, "OBENLINKS", RED)
    for i, line in enumerate(PARAGRAPH):
        ops += text_op(rotate, box, 60, h / 2 + i * 16, 12, line, (0, 0, 0))
    ops += text_op(rotate, box, w - 40 - 22 * 0.62 * 11, h - 40, 22, "UNTENRECHTS", BLUE)
    if crop:
        # Outside the CropBox, inside the MediaBox: must not appear in any element.
        mx0, my0 = media[0], media[1]
        ops += f"BT /F1 14 Tf 0 0.6 0 rg 1 0 0 1 {mx0 + 5} {my0 + 20} Tm (AUSSERHALB) Tj ET\n".encode("latin-1")
    return ops


def ink_boxes(path):
    """Size of each displayed page and the ink box of the red and the blue marker, in
    points, measured on the page rendered with /Rotate and CropBox applied."""
    pages = []
    with fitz.open(path) as doc:
        for page in doc:
            zoom = 4
            pix = page.get_pixmap(matrix=fitz.Matrix(zoom, zoom), alpha=False)
            px = numpy.frombuffer(pix.samples, dtype=numpy.uint8).reshape(pix.height, pix.width, 3).astype(int)
            r, g, b = px[:, :, 0], px[:, :, 1], px[:, :, 2]
            masks = {
                "OBENLINKS": (r > 160) & (g < 110) & (b < 110),
                "UNTENRECHTS": (b > 160) & (r < 110) & (g < 110),
            }
            boxes = {}
            for name, mask in masks.items():
                ys, xs = numpy.nonzero(mask)
                boxes[name] = [int(xs.min()), int(ys.min()), int(xs.max()) + 1, int(ys.max()) + 1]
            pages.append(
                {
                    "width": round(page.rect.width, 2),
                    "height": round(page.rect.height, 2),
                    "markers": {k: [round(v / zoom, 2) for v in bb] for k, bb in boxes.items()},
                }
            )
    return pages


def make_frame_pdfs():
    text_pages = []
    for _, media, crop, rotate in FRAME_VARIANTS:
        text_pages.append({"media": media, "crop": crop, "rotate": rotate, "content": frame_content(rotate, media, crop)})
    Pdf().write(HERE / "frames-text.pdf", text_pages)
    EXPECTED["frames-text.pdf"] = {"pages": ink_boxes(HERE / "frames-text.pdf")}

    # The same pages as pure scans: the unrotated MediaBox rendered to an image that
    # fills the MediaBox, with the same CropBox and /Rotate.
    flat = HERE / "frames-flat.tmp.pdf"
    Pdf().write(flat, [{"media": p["media"], "content": p["content"]} for p in text_pages])
    pdf = Pdf()
    scan_pages = []
    with fitz.open(flat) as doc:
        for page, (_, media, crop, rotate) in zip(doc, FRAME_VARIANTS):
            pix = page.get_pixmap(matrix=fitz.Matrix(150 / 72, 150 / 72), alpha=False)
            img = Image.frombytes("RGB", (pix.width, pix.height), pix.samples)
            image = pdf.image(img)
            w, h = media[2] - media[0], media[3] - media[1]
            content = f"q {w} 0 0 {h} {media[0]} {media[1]} cm /Im0 Do Q\n".encode("latin-1")
            scan_pages.append({"media": media, "crop": crop, "rotate": rotate, "content": content, "images": [image]})
    flat.unlink()
    pdf.write(HERE / "frames-scan.pdf", scan_pages)
    EXPECTED["frames-scan.pdf"] = {"pages": ink_boxes(HERE / "frames-scan.pdf")}
    EXPECTED["frames"] = [name for name, *_ in FRAME_VARIANTS]


def printed_text_image(lines, size=(1100, 320)):
    img = Image.new("RGB", size, "white")
    draw = ImageDraw.Draw(img)
    for i, line in enumerate(lines):
        draw.text((30, 30 + i * 90), line, fill="black", font=font(60))
    return img


def make_origin_pdf():
    pdf = Pdf()
    a4 = [0, 0, 595, 842]
    digital = b""
    lines = [
        "Digitale Seite ohne Bild.",
        "Der gesamte Text dieser Seite steht in der Textschicht.",
        "Es gibt keine Texterkennung auf dieser Seite.",
    ]
    for i, line in enumerate(lines):
        digital += text_op(0, a4, 60, 100 + i * 18, 12, line, (0, 0, 0))
    image = pdf.image(printed_text_image(["Gedruckter Text im Bild", "Rechnung Nummer 4711"]))
    mixed = b""
    for i, line in enumerate(["Digitale Seite mit einem Bild.", "Unter diesem Absatz steht ein Bild mit gedrucktem Text."]):
        mixed += text_op(0, a4, 60, 100 + i * 18, 12, line, (0, 0, 0))
    mixed += b"q 440 0 0 128 60 520 cm /Im0 Do Q\n"
    pdf.write(
        HERE / "origin.pdf",
        [
            {"media": a4, "content": digital},
            {"media": a4, "content": mixed, "images": [image]},
        ],
    )


def make_heading_pdf():
    a4 = [0, 0, 595, 842]
    body = [
        "Dieser Abschnitt beschreibt den Entwurf einer Regelung fuer die Pruefung",
        "von Geraeten. Er besteht aus mehreren Saetzen, damit die Seite wie ein",
        "gewoehnliches Dokument mit Ueberschriften und Fliesstext aussieht.",
    ]
    ops = text_op(0, a4, 60, 80, 20, "1 Einleitung", (0, 0, 0), "F2")
    for i, line in enumerate(body):
        ops += text_op(0, a4, 60, 120 + i * 16, 11, line, (0, 0, 0))
    ops += text_op(0, a4, 60, 220, 20, "2 (Entwurf)", (0, 0, 0), "F2")
    for i, line in enumerate(body):
        ops += text_op(0, a4, 60, 260 + i * 16, 11, line, (0, 0, 0))
    Pdf().write(HERE / "heading.pdf", [{"media": a4, "content": ops}])

    writer = PdfWriter(clone_from=PdfReader(HERE / "heading.pdf"))
    writer.encrypt(user_password="geheim", owner_password="besitzer", algorithm="RC4-128")
    with open(HERE / "encrypted.pdf", "wb") as f:
        writer.write(f)


# ---------------------------------------------------------------- images


def marker_page(size, label=None):
    """A white page with the red marker top left, the blue one bottom right and a
    paragraph in between; returns the image and the ink boxes in pixels."""
    w, h = size
    img = Image.new("RGB", size, "white")
    draw = ImageDraw.Draw(img)
    f = font(max(24, w // 22))
    top_left = (int(w * 0.05), int(h * 0.04))
    draw.text(top_left, "OBENLINKS", fill=(255, 0, 0), font=f)
    a = draw.textbbox(top_left, "OBENLINKS", font=f)
    tw = draw.textlength("UNTENRECHTS", font=f)
    bottom_right = (int(w * 0.95 - tw), int(h * 0.92))
    draw.text(bottom_right, "UNTENRECHTS", fill=(0, 0, 255), font=f)
    b = draw.textbbox(bottom_right, "UNTENRECHTS", font=f)
    body = font(max(16, w // 45))
    lines = [label or "Ein Absatz in der Mitte der Seite.", "Er steht zwischen den beiden Markierungen."]
    for i, line in enumerate(lines):
        draw.text((int(w * 0.08), int(h * 0.45) + i * int(w // 30)), line, fill="black", font=body)
    return img, {"OBENLINKS": list(a), "UNTENRECHTS": list(b)}


def exif_orientation(value):
    exif = Image.Exif()
    exif[0x0112] = value
    return exif


def make_images():
    # Displayed landscape 1754 x 1240, stored portrait 1240 x 1754 with EXIF orientation 6
    # (the viewer turns it 90 degrees clockwise).
    shown, boxes = marker_page((1754, 1240))
    stored = shown.transpose(Image.Transpose.ROTATE_90)
    stored.save(HERE / "exif6-150dpi.jpg", quality=90, dpi=(150, 150), exif=exif_orientation(6))
    stored.save(HERE / "exif6-nodpi.jpg", quality=90, exif=exif_orientation(6))
    for name in ("exif6-150dpi.jpg", "exif6-nodpi.jpg"):
        EXPECTED[name] = {"pages": [{"width": 1754, "height": 1240, "markers": boxes}]}

    page, boxes = marker_page((1240, 1754))
    page.save(HERE / "portrait-300dpi.png", dpi=(300, 300))
    EXPECTED["portrait-300dpi.png"] = {"pages": [{"width": 1240, "height": 1754, "markers": boxes}]}

    sizes = [((1240, 1754), 150, "Erste Seite der Datei."), ((1654, 1170), 200, "Zweite Seite der Datei."), ((1000, 1400), 120, "Dritte Seite der Datei.")]
    frames, expected = [], []
    for size, dpi, label in sizes:
        img, boxes = marker_page(size, label)
        frames.append((img.convert("L"), dpi))
        expected.append({"width": size[0], "height": size[1], "markers": boxes})
    write_tiff(HERE / "pages.tif", frames)
    EXPECTED["pages.tif"] = {"pages": expected}
    write_tiff(HERE / "single.tif", frames[:1])
    EXPECTED["single.tif"] = {"pages": expected[:1]}

    # A one-page TIFF whose pointer to the next IFD points past the end of the file.
    data = bytearray((HERE / "single.tif").read_bytes())
    ifd = int.from_bytes(data[4:8], "little")
    count = int.from_bytes(data[ifd : ifd + 2], "little")
    next_at = ifd + 2 + count * 12
    data[next_at : next_at + 4] = (len(data) + 4096).to_bytes(4, "little")
    (HERE / "damaged.tif").write_bytes(bytes(data))


def write_tiff(path, frames):
    """Multi-page TIFF with one resolution per page (Pillow writes one for all)."""
    first, *rest = frames
    buffers = []
    for img, dpi in frames:
        b = io.BytesIO()
        img.save(b, format="TIFF", compression="tiff_deflate", dpi=(dpi, dpi))
        buffers.append(b.getvalue())
    if len(frames) == 1:
        path.write_bytes(buffers[0])
        return
    out = io.BytesIO()
    with Image.open(io.BytesIO(buffers[0])) as im0:
        im0.save(
            out,
            format="TIFF",
            save_all=True,
            compression="tiff_deflate",
            dpi=(first[1], first[1]),
            append_images=[Image.open(io.BytesIO(b)) for b in buffers[1:]],
        )
    data = bytearray(out.getvalue())
    # Set XResolution / YResolution of each IFD to the page's own value.
    ifd = int.from_bytes(data[4:8], "little")
    for _, dpi in frames:
        count = int.from_bytes(data[ifd : ifd + 2], "little")
        for i in range(count):
            entry = ifd + 2 + i * 12
            tag = int.from_bytes(data[entry : entry + 2], "little")
            if tag in (282, 283):
                offset = int.from_bytes(data[entry + 8 : entry + 12], "little")
                data[offset : offset + 8] = dpi.to_bytes(4, "little") + (1).to_bytes(4, "little")
        ifd = int.from_bytes(data[ifd + 2 + count * 12 : ifd + 6 + count * 12], "little")
    path.write_bytes(bytes(data))


def main():
    make_formats()
    make_tables()
    make_slides()
    make_docx()
    make_text_formats()
    make_frame_pdfs()
    make_origin_pdf()
    make_heading_pdf()
    make_images()
    (HERE / "expected.json").write_text(json.dumps(EXPECTED, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")


if __name__ == "__main__":
    main()
