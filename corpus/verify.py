#!/usr/bin/env python3
"""Check corpus/manifest.toml against the files in corpus/documents/.

Checks: every manifest entry has its file, sha256 and size_bytes match,
required fields are present, ids and files are unique, and no file in
documents/ is missing from the manifest.

Standard library only. Uses tomllib (Python 3.11+); on older versions a
minimal parser for the subset of TOML used in manifest.toml is used.

Exit code 0 when everything matches, 1 otherwise.
"""

import hashlib
import re
import sys
from pathlib import Path

CORPUS_DIR = Path(__file__).resolve().parent
MANIFEST = CORPUS_DIR / "manifest.toml"
DOCUMENTS_DIR = CORPUS_DIR / "documents"

REQUIRED_FIELDS = [
    "id", "file", "title", "source_url", "retrieved", "license", "language",
    "format", "sha256", "size_bytes", "checks", "whole_document",
]
FILE_NAME_PATTERN = re.compile(r"^[a-z0-9][a-z0-9.-]*$")


def parse_minimal_toml(text):
    """Parse the TOML subset used by manifest.toml.

    Supports comments, top-level key/value pairs, [[documents]] tables,
    basic strings, integers, booleans and single-line arrays of strings.
    """
    root = {}
    current = root
    string_re = r'"(?:[^"\\]|\\.)*"'
    for number, raw in enumerate(text.splitlines(), start=1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        table = re.fullmatch(r"\[\[([A-Za-z0-9_]+)\]\]", line)
        if table:
            current = {}
            root.setdefault(table.group(1), []).append(current)
            continue
        pair = re.fullmatch(r"([A-Za-z0-9_]+)\s*=\s*(.+)", line)
        if not pair:
            raise ValueError(f"line {number}: unsupported syntax: {raw}")
        key, value = pair.group(1), pair.group(2).strip()
        if value.startswith('"'):
            if not re.fullmatch(string_re, value):
                raise ValueError(f"line {number}: unsupported string: {raw}")
            parsed = unescape(value[1:-1])
        elif value.startswith("["):
            items = re.findall(string_re, value)
            rebuilt = "[" + ", ".join(items) + "]"
            if re.sub(r"\s", "", rebuilt) != re.sub(r"\s", "", value):
                raise ValueError(f"line {number}: only arrays of strings are supported")
            parsed = [unescape(item[1:-1]) for item in items]
        elif value in ("true", "false"):
            parsed = value == "true"
        elif re.fullmatch(r"-?\d+", value):
            parsed = int(value)
        else:
            raise ValueError(f"line {number}: unsupported value: {raw}")
        if key in current:
            raise ValueError(f"line {number}: duplicate key {key}")
        current[key] = parsed
    return root


def unescape(value):
    escapes = {'"': '"', "\\": "\\", "n": "\n", "t": "\t"}
    return re.sub(r"\\(.)", lambda m: escapes.get(m.group(1), m.group(0)), value)


def load_manifest(path):
    data = path.read_bytes()
    try:
        import tomllib
    except ImportError:
        return parse_minimal_toml(data.decode("utf-8"))
    return tomllib.loads(data.decode("utf-8"))


def sha256_of(path):
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def main():
    errors = []
    manifest = load_manifest(MANIFEST)
    entries = manifest.get("documents", [])
    seen_ids, seen_files = set(), set()
    total = 0

    for index, entry in enumerate(entries, start=1):
        label = entry.get("id", f"entry {index}")
        missing = [field for field in REQUIRED_FIELDS if field not in entry]
        if missing:
            errors.append(f"{label}: missing fields: {', '.join(missing)}")
        if label in seen_ids:
            errors.append(f"{label}: duplicate id")
        seen_ids.add(label)

        name = entry.get("file")
        if not name:
            continue
        if name in seen_files:
            errors.append(f"{label}: file listed twice: {name}")
        seen_files.add(name)
        if not FILE_NAME_PATTERN.fullmatch(name):
            errors.append(f"{label}: file name not lowercase ASCII without spaces: {name}")

        path = DOCUMENTS_DIR / name
        if not path.is_file():
            errors.append(f"{label}: file missing: documents/{name}")
            continue
        size = path.stat().st_size
        total += size
        if size != entry.get("size_bytes"):
            errors.append(f"{label}: size {size} != manifest {entry.get('size_bytes')}")
        digest = sha256_of(path)
        if digest != entry.get("sha256"):
            errors.append(f"{label}: sha256 {digest} != manifest {entry.get('sha256')}")
        else:
            print(f"OK    {name}  {size} bytes")

    on_disk = {p.name for p in DOCUMENTS_DIR.iterdir() if p.is_file()} if DOCUMENTS_DIR.is_dir() else set()
    for name in sorted(on_disk - seen_files):
        errors.append(f"file without manifest entry: documents/{name}")

    print(f"{len(entries)} entries, {len(on_disk)} files, {total} bytes ({total / 1_000_000:.1f} MB)")
    for error in errors:
        print(f"ERROR {error}")
    if errors:
        print(f"FAILED: {len(errors)} error(s)")
        return 1
    print("PASSED")
    return 0


if __name__ == "__main__":
    sys.exit(main())
