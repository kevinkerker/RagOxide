#!/usr/bin/env python3
"""Check corpus/questions.toml (question catalogue) for form and consistency.

Checks:
- required fields, allowed values and types, unique ids, id prefix = language;
- evidence/near-miss entries point to documents in manifest.toml whose file
  exists, with the anchor that fits the format (page, slide, sheet + cells,
  heading_path) and within the page/slide count of the manifest; sheet names
  are checked against the workbook;
- kind and evidence agree (multi_part has at least two parts, unanswerable
  questions have no evidence but near misses);
- number of questions and language split;
- for HTML and Markdown documents: the quote occurs in the document text and
  every heading_path element is a heading of the document;
- shared_terms really occur as words in the question and in a quote;
- the reproducible review sample (see sample_ids) matches the question ids
  listed in questions-sample.md.

It prints warnings (not errors) when a question shares a run of five words
with one of its quotes, and a summary of the distribution.

Standard library only. Uses tomllib (Python 3.11+); on older versions a
minimal parser for the TOML subset used in questions.toml is used.

Usage: python corpus/verify_questions.py [--sample]
  --sample  only print the ids of the review sample
Exit code 0 when everything matches, 1 otherwise.
"""

import html.parser
import random
import re
import sys
import unicodedata
import zipfile
from pathlib import Path

CORPUS_DIR = Path(__file__).resolve().parent
QUESTIONS = CORPUS_DIR / "questions.toml"
SAMPLE_DOC = CORPUS_DIR / "questions-sample.md"
MANIFEST = CORPUS_DIR / "manifest.toml"
DOCUMENTS_DIR = CORPUS_DIR / "documents"

EXPECTED_TOTAL = 50
LANGUAGE_SHARE = (0.4, 0.6)  # allowed share of each language
MIN_UNANSWERABLE = 5
LANGUAGES = {"de", "en"}
KINDS = {"fact", "table_value", "multi_part", "unanswerable"}
FEATURES = {
    "scan", "scan_table", "multi_column", "digital_table", "office",
    "footnote", "speaker_notes", "merged_cells", "broken_text_layer",
    "parallel_text", "cross_document", "multiple_locations",
    "compound", "inflection", "synonym", "abbreviation",
}
# Same text as another corpus document, excluded from questions (README).
EXCLUDED_DOCUMENTS = {
    "de-zenodo-10409864-mustervertrag-datennutzung-v3-tracked-changes",
}
QUESTION_FIELDS = {
    "id": str, "language": str, "question": str, "answerable": bool,
    "kind": str, "features": list, "answer": str, "shared_terms": list,
    "term_variants": list,
}
OPTIONAL_QUESTION_FIELDS = {"notes": str, "evidence": list, "near_miss": list}
ANCHOR_BY_FORMAT = {
    "pdf": "page", "jpeg": "page", "pptx": "slide", "xlsx": "sheet",
    "docx": "heading_path", "html": "heading_path", "markdown": "heading_path",
}
ANCHOR_FIELDS = {"page", "slide", "sheet", "cells", "heading_path"}
ID_PATTERN = re.compile(r"^(de|en)-\d{2}$")
CELLS_PATTERN = re.compile(r"^[A-Z]{1,3}[1-9]\d*(:[A-Z]{1,3}[1-9]\d*)?$")
SAMPLE_SEED = 20261115
SAMPLE_PER_LANGUAGE = 5


# --- minimal TOML subset parser (Python < 3.11) ---------------------------

STRING_RE = r'"(?:[^"\\]|\\.)*"'


def unescape(value):
    def repl(match):
        esc = match.group(1)
        simple = {'"': '"', "\\": "\\", "n": "\n", "t": "\t", "r": "\r"}
        if esc in simple:
            return simple[esc]
        if esc.startswith("u") or esc.startswith("U"):
            return chr(int(esc[1:], 16))
        raise ValueError(f"unsupported escape \\{esc}")
    return re.sub(r'\\(u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8}|.)', repl, value)


def parse_value(value, number):
    if value.startswith('"'):
        if not re.fullmatch(STRING_RE, value):
            raise ValueError(f"line {number}: unsupported string")
        return unescape(value[1:-1])
    if value.startswith("["):
        items = re.findall(STRING_RE, value)
        rebuilt = "[" + ",".join(items) + "]"
        compact = re.sub(r",\s*\]", "]", re.sub(r"\s", "", value))
        if re.sub(r"\s", "", rebuilt) != compact and not (items == [] and compact == "[]"):
            raise ValueError(f"line {number}: only arrays of strings are supported")
        return [unescape(item[1:-1]) for item in items]
    if value in ("true", "false"):
        return value == "true"
    if re.fullmatch(r"-?\d+", value):
        return int(value)
    raise ValueError(f"line {number}: unsupported value: {value}")


def strip_comment(value):
    """Remove a trailing comment outside strings."""
    out, in_string, escaped = [], False, False
    for char in value:
        if in_string:
            out.append(char)
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
        elif char == '"':
            in_string = True
            out.append(char)
        elif char == "#":
            break
        else:
            out.append(char)
    return "".join(out).strip()


def parse_minimal_toml(text):
    """Parse the subset used here: key = value, [[a]] and [[a.b]] tables,
    basic single-line strings, integers, booleans, arrays of strings."""
    root = {}
    current = root
    for number, raw in enumerate(text.splitlines(), start=1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        table = re.fullmatch(r"\[\[([A-Za-z0-9_]+(?:\.[A-Za-z0-9_]+)?)\]\]", line)
        if table:
            parts = table.group(1).split(".")
            if len(parts) == 1:
                current = {}
                root.setdefault(parts[0], []).append(current)
            else:
                parents = root.get(parts[0])
                if not parents:
                    raise ValueError(f"line {number}: [[{table.group(1)}]] without parent")
                current = {}
                parents[-1].setdefault(parts[1], []).append(current)
            continue
        pair = re.fullmatch(r"([A-Za-z0-9_]+)\s*=\s*(.+)", line)
        if not pair:
            raise ValueError(f"line {number}: unsupported syntax: {raw}")
        key = pair.group(1)
        if key in current:
            raise ValueError(f"line {number}: duplicate key {key}")
        current[key] = parse_value(strip_comment(pair.group(2)), number)
    return root


def load_toml(path):
    text = path.read_text(encoding="utf-8")
    try:
        import tomllib
    except ModuleNotFoundError:
        return parse_minimal_toml(text)
    return tomllib.loads(text)


# --- text helpers ----------------------------------------------------------

def normalize(text):
    """NFC, soft hyphens removed, '|' as whitespace, whitespace collapsed."""
    text = unicodedata.normalize("NFC", text).replace("­", "")
    return " ".join(text.replace("|", " ").split())


def words(text):
    return re.findall(r"\w+", normalize(text).casefold())


def contains_word(text, term):
    pattern = r"(?<!\w)" + re.escape(normalize(term).casefold()) + r"(?!\w)"
    return re.search(pattern, normalize(text).casefold()) is not None


class VisibleText(html.parser.HTMLParser):
    """Visible text of an HTML file; block and cell boundaries become spaces.
    Skips script/style and elements hidden with visibility:hidden or
    display:none (e.g. Wikipedia sort padding)."""

    VOID = {"area", "base", "br", "col", "embed", "hr", "img", "input",
            "link", "meta", "param", "source", "track", "wbr"}
    BLOCK = {"p", "div", "li", "ul", "ol", "table", "tr", "td", "th",
             "tbody", "thead", "tfoot", "caption", "figcaption", "figure",
             "dd", "dt", "dl", "section", "blockquote", "pre", "br",
             "h1", "h2", "h3", "h4", "h5", "h6", "body", "header", "footer"}
    HEADINGS = {"h1", "h2", "h3", "h4", "h5", "h6"}

    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.parts = []
        self.stack = []  # (tag, hidden)
        self.hidden_depth = 0
        self.headings = []
        self.heading_buf = None

    def handle_starttag(self, tag, attrs):
        if tag in self.BLOCK:
            self.parts.append(" ")
        if tag in self.VOID:
            return
        style = (dict(attrs).get("style") or "").replace(" ", "").lower()
        hidden = (tag in ("script", "style") or "visibility:hidden" in style
                  or "display:none" in style)
        self.stack.append((tag, hidden))
        if hidden:
            self.hidden_depth += 1
        if tag in self.HEADINGS:
            self.heading_buf = []

    def handle_endtag(self, tag):
        if tag in self.BLOCK:
            self.parts.append(" ")
        if tag in self.HEADINGS and self.heading_buf is not None:
            self.headings.append(normalize("".join(self.heading_buf)))
            self.heading_buf = None
        if not any(t == tag for t, _ in self.stack):
            return
        while self.stack:
            open_tag, hidden = self.stack.pop()
            if hidden:
                self.hidden_depth -= 1
            if open_tag == tag:
                break

    def handle_data(self, data):
        if self.hidden_depth:
            return
        self.parts.append(data)
        if self.heading_buf is not None:
            self.heading_buf.append(data)


_text_cache = {}


def document_text(path, fmt):
    """Return (normalized text, set of normalized headings) for html/markdown."""
    if path in _text_cache:
        return _text_cache[path]
    raw = path.read_text(encoding="utf-8")
    if fmt == "html":
        parser = VisibleText()
        parser.feed(raw)
        parser.close()
        result = (normalize("".join(parser.parts)), set(parser.headings))
    else:
        headings = {normalize(m.group(1)) for m in
                    re.finditer(r"^#{1,6}\s+(.*?)\s*#*\s*$", raw, re.MULTILINE)}
        result = (normalize(raw), headings)
    _text_cache[path] = result
    return result


def xlsx_sheet_names(path):
    with zipfile.ZipFile(path) as archive:
        workbook = archive.read("xl/workbook.xml").decode("utf-8")
    names = re.findall(r'<sheet\b[^>]*\bname="([^"]*)"', workbook)
    return {html.unescape(n) for n in names}


# --- checks ------------------------------------------------------------------

def check_anchor(entry, doc, where, errors):
    fmt = doc["format"]
    expected = ANCHOR_BY_FORMAT.get(fmt)
    present = ANCHOR_FIELDS & set(entry)
    allowed = {"sheet", "cells"} if fmt == "xlsx" else {expected}
    if present != allowed:
        errors.append(f"{where}: format {fmt} needs anchor {sorted(allowed)}, has {sorted(present)}")
        return
    if expected == "page":
        page = entry["page"]
        if not isinstance(page, int) or not 1 <= page <= doc.get("pages", 0):
            errors.append(f"{where}: page {page!r} outside 1..{doc.get('pages')}")
    elif expected == "slide":
        slide = entry["slide"]
        if not isinstance(slide, int) or not 1 <= slide <= doc.get("slides", 0):
            errors.append(f"{where}: slide {slide!r} outside 1..{doc.get('slides')}")
    elif fmt == "xlsx":
        if not isinstance(entry["cells"], str) or not CELLS_PATTERN.match(entry["cells"]):
            errors.append(f"{where}: cells {entry['cells']!r} is not an A1 range")
        sheets = xlsx_sheet_names(DOCUMENTS_DIR / doc["file"])
        if entry["sheet"] not in sheets:
            errors.append(f"{where}: sheet {entry['sheet']!r} not in workbook")
    else:
        path = entry["heading_path"]
        if not isinstance(path, list) or not path or not all(isinstance(p, str) and p for p in path):
            errors.append(f"{where}: heading_path must be a non-empty list of strings")


def check_entry(entry, kind_of_entry, docs, where, errors):
    required = {"document", "quote"} | ({"part"} if kind_of_entry == "evidence" else {"reason"})
    allowed = required | ANCHOR_FIELDS | {"parallel"}
    missing = required - set(entry)
    unknown = set(entry) - allowed
    if missing:
        errors.append(f"{where}: missing {sorted(missing)}")
    if unknown:
        errors.append(f"{where}: unknown fields {sorted(unknown)}")
    doc_id = entry.get("document")
    doc = docs.get(doc_id)
    if doc is None:
        errors.append(f"{where}: document {doc_id!r} not in manifest")
        return None
    if doc_id in EXCLUDED_DOCUMENTS:
        errors.append(f"{where}: document {doc_id} is excluded from questions")
    if not (DOCUMENTS_DIR / doc["file"]).is_file():
        errors.append(f"{where}: file {doc['file']} missing")
        return None
    if not isinstance(entry.get("quote"), str) or len(entry.get("quote", "").strip()) < 3:
        errors.append(f"{where}: quote missing or too short")
    if kind_of_entry == "evidence":
        if not isinstance(entry.get("part"), int) or entry["part"] < 1:
            errors.append(f"{where}: part must be an integer >= 1")
    if "parallel" in entry:
        if entry["parallel"] is not True:
            errors.append(f"{where}: parallel must be true when present")
        if "parallel_text" not in doc.get("checks", []):
            errors.append(f"{where}: parallel on a document without parallel_text")
    check_anchor(entry, doc, where, errors)
    if doc["format"] in ("html", "markdown") and isinstance(entry.get("quote"), str):
        text, headings = document_text(DOCUMENTS_DIR / doc["file"], doc["format"])
        if normalize(entry["quote"]) not in text:
            errors.append(f"{where}: quote not found in {doc['file']}: {entry['quote'][:70]!r}")
        for heading in entry.get("heading_path", []):
            if normalize(heading) not in headings:
                errors.append(f"{where}: heading {heading!r} not a heading of {doc['file']}")
    return doc


def format_groups(question, docs):
    groups = set()
    for entry in question.get("evidence", []) + question.get("near_miss", []):
        doc = docs.get(entry.get("document"))
        if not doc:
            continue
        fmt = doc["format"]
        if fmt == "jpeg" or (fmt == "pdf" and "scan" in doc.get("checks", [])):
            groups.add("scan")
        elif fmt == "pdf":
            groups.add("pdf_digital")
        elif fmt in ("docx", "pptx", "xlsx"):
            groups.add("office")
        else:
            groups.add("html_markdown")
    return groups


def sample_ids(questions, docs):
    """Reproducible review sample: 5 de + 5 en, drawn with random.Random(seed);
    the seed starts at SAMPLE_SEED and is increased until every kind and every
    format group occurs at least once."""
    by_id = {q["id"]: q for q in questions}
    pools = {lang: sorted(q["id"] for q in questions if q["language"] == lang)
             for lang in sorted(LANGUAGES)}
    all_groups = {"scan", "pdf_digital", "office", "html_markdown"}
    for seed in range(SAMPLE_SEED, SAMPLE_SEED + 10000):
        rng = random.Random(seed)
        chosen = []
        for lang in sorted(pools):
            chosen += rng.sample(pools[lang], SAMPLE_PER_LANGUAGE)
        kinds = {by_id[i]["kind"] for i in chosen}
        groups = set().union(*(format_groups(by_id[i], docs) for i in chosen))
        if kinds == KINDS and all_groups <= groups:
            return seed, sorted(chosen)
    raise RuntimeError("no sample satisfies the constraints")


def main():
    manifest = load_toml(MANIFEST)
    docs = {d["id"]: d for d in manifest.get("documents", [])}
    data = load_toml(QUESTIONS)
    questions = data.get("questions", [])

    if "--sample" in sys.argv[1:]:
        seed, ids = sample_ids(questions, docs)
        print(f"seed {seed}: {' '.join(ids)}")
        return 0

    errors, warnings = [], []
    if data.get("schema_version") != 1:
        errors.append("schema_version must be 1")

    seen = set()
    for index, q in enumerate(questions, start=1):
        qid = q.get("id", f"#{index}")
        where = f"question {qid}"
        for field, typ in QUESTION_FIELDS.items():
            if field not in q:
                errors.append(f"{where}: missing field {field}")
            elif not isinstance(q[field], typ):
                errors.append(f"{where}: {field} must be {typ.__name__}")
        unknown = set(q) - set(QUESTION_FIELDS) - set(OPTIONAL_QUESTION_FIELDS)
        if unknown:
            errors.append(f"{where}: unknown fields {sorted(unknown)}")
        if not isinstance(qid, str) or not ID_PATTERN.match(qid):
            errors.append(f"{where}: id must match {ID_PATTERN.pattern}")
        elif qid in seen:
            errors.append(f"{where}: duplicate id")
        seen.add(qid)
        lang = q.get("language")
        if lang not in LANGUAGES:
            errors.append(f"{where}: language {lang!r} not allowed")
        elif isinstance(qid, str) and not qid.startswith(lang + "-"):
            errors.append(f"{where}: id prefix does not match language {lang}")
        kind = q.get("kind")
        if kind not in KINDS:
            errors.append(f"{where}: kind {kind!r} not allowed")
        bad = set(q.get("features", [])) - FEATURES
        if bad:
            errors.append(f"{where}: features not allowed: {sorted(bad)}")
        for field in ("question", "answer"):
            if isinstance(q.get(field), str) and len(q[field].strip()) < 10:
                errors.append(f"{where}: {field} too short")
        for variant in q.get("term_variants", []):
            if "->" not in variant:
                errors.append(f"{where}: term_variants entry needs 'question form -> text form': {variant!r}")

        answerable = q.get("answerable")
        evidence = q.get("evidence", [])
        near = q.get("near_miss", [])
        if (kind == "unanswerable") != (answerable is False):
            errors.append(f"{where}: kind unanswerable must go with answerable = false")
        if answerable:
            if not evidence:
                errors.append(f"{where}: answerable question without evidence")
            if near:
                errors.append(f"{where}: near_miss only for unanswerable questions")
        else:
            if evidence:
                errors.append(f"{where}: unanswerable question with evidence")
            if not near:
                errors.append(f"{where}: unanswerable question needs at least one near_miss")

        entry_docs = []
        for n, entry in enumerate(evidence, start=1):
            doc = check_entry(entry, "evidence", docs, f"{where} evidence {n}", errors)
            entry_docs.append(doc)
        for n, entry in enumerate(near, start=1):
            check_entry(entry, "near_miss", docs, f"{where} near_miss {n}", errors)

        parts = sorted({e.get("part") for e in evidence if isinstance(e.get("part"), int)})
        if evidence and parts != list(range(1, len(parts) + 1)):
            errors.append(f"{where}: parts must be numbered 1..n without gaps, got {parts}")
        if kind == "multi_part" and len(parts) < 2:
            errors.append(f"{where}: multi_part needs at least two parts")
        if kind in ("fact", "table_value") and parts and parts != [1]:
            errors.append(f"{where}: {kind} must have a single part")
        features = set(q.get("features", []))
        if any(e.get("parallel") for e in evidence) != ("parallel_text" in features):
            errors.append(f"{where}: feature parallel_text must match parallel evidence")
        doc_ids = {e.get("document") for e in evidence if not e.get("parallel")}
        if ("cross_document" in features) != (len(doc_ids) > 1):
            errors.append(f"{where}: feature cross_document must match evidence in several documents")
        per_part = {}
        for e in evidence:
            if not e.get("parallel"):
                per_part.setdefault(e.get("part"), []).append(e)
        multi = any(len(v) > 1 for v in per_part.values())
        if ("multiple_locations" in features) != multi:
            errors.append(f"{where}: feature multiple_locations must match several entries in one part")
        if "office" in features and not any(d and d["format"] in ("docx", "pptx", "xlsx") for d in entry_docs):
            if answerable:
                errors.append(f"{where}: feature office without office evidence")

        quotes = [e.get("quote", "") for e in evidence + near if isinstance(e.get("quote"), str)]
        for term in q.get("shared_terms", []):
            if not contains_word(q.get("question", ""), term):
                errors.append(f"{where}: shared term {term!r} not in question")
            if not any(contains_word(quote, term) for quote in quotes):
                errors.append(f"{where}: shared term {term!r} not in any quote")
        qwords = words(q.get("question", ""))
        grams = {tuple(qwords[i:i + 5]) for i in range(len(qwords) - 4)}
        for quote in quotes:
            qw = words(quote)
            if grams & {tuple(qw[i:i + 5]) for i in range(len(qw) - 4)}:
                warnings.append(f"{where}: question shares five consecutive words with a quote")
                break

    total = len(questions)
    if total != EXPECTED_TOTAL:
        errors.append(f"expected {EXPECTED_TOTAL} questions, found {total}")
    counts = {lang: sum(1 for q in questions if q.get("language") == lang) for lang in sorted(LANGUAGES)}
    for lang, count in counts.items():
        share = count / total if total else 0
        if not LANGUAGE_SHARE[0] <= share <= LANGUAGE_SHARE[1]:
            errors.append(f"language {lang}: {count} of {total} outside {LANGUAGE_SHARE}")
    unanswerable = sum(1 for q in questions if q.get("answerable") is False)
    if unanswerable < MIN_UNANSWERABLE:
        errors.append(f"only {unanswerable} unanswerable questions, need {MIN_UNANSWERABLE}")

    # review sample must match questions-sample.md
    sample_note = ""
    if not errors:
        seed, ids = sample_ids(questions, docs)
        sample_note = f"sample (seed {seed}): {' '.join(ids)}"
        if SAMPLE_DOC.is_file():
            listed = re.findall(r"^#{2,4}\s+((?:de|en)-\d{2})\b", SAMPLE_DOC.read_text(encoding="utf-8"), re.MULTILINE)
            if sorted(listed) != ids:
                errors.append(f"{SAMPLE_DOC.name} lists {sorted(listed)}, expected {ids}")
        else:
            errors.append(f"{SAMPLE_DOC.name} missing")

    # summary
    def tally(key):
        out = {}
        for q in questions:
            values = q.get(key, []) if key == "features" else [q.get(key)]
            for v in values:
                out[v] = out.get(v, 0) + 1
        return ", ".join(f"{k} {v}" for k, v in sorted(out.items(), key=lambda kv: str(kv[0])))
    by_doc = {}
    by_format = {}
    for q in questions:
        for doc_id in {e.get("document") for e in q.get("evidence", []) + q.get("near_miss", [])
                       if not e.get("parallel")}:
            by_doc[doc_id] = by_doc.get(doc_id, 0) + 1
            fmt = docs.get(doc_id, {}).get("format", "?")
            by_format[fmt] = by_format.get(fmt, 0) + 1
    print(f"questions: {total} ({', '.join(f'{k} {v}' for k, v in counts.items())}), unanswerable {unanswerable}")
    print(f"kinds: {tally('kind')}")
    print(f"features: {tally('features')}")
    print("documents (questions with evidence or near miss there):")
    for doc_id in sorted(docs):
        print(f"  {by_doc.get(doc_id, 0):2d}  {doc_id}")
    print(f"formats: {', '.join(f'{k} {v}' for k, v in sorted(by_format.items()))}")
    if sample_note:
        print(sample_note)
    for w in warnings:
        print(f"WARN  {w}")
    for e in errors:
        print(f"ERROR {e}")
    print("PASSED" if not errors else f"FAILED ({len(errors)} errors)")
    return 0 if not errors else 1


if __name__ == "__main__":
    sys.exit(main())
