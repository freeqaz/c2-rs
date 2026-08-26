#!/usr/bin/env python3
"""apply_marks.py — insert one PROV[X] doc line above each named const.

Lane `w-provenance` scratch. COMMENT-ONLY by construction: it inserts a line
that begins `///` or `//` and touches nothing else. Re-runnable — a const that
already carries a marker in its attached block is skipped.

Usage: python3 work/w-provenance/apply_marks.py work/w-provenance/marks.tsv
TSV columns: path <TAB> const-name <TAB> marker-text (without the /// prefix)
"""
import re
import sys

ITEM = r"^(\s*)(?:pub(?:\([a-z:]+\))?\s+)?(?:const|static)\s+(?:mut\s+)?{}\s*:"


def main(tsv):
    rows = []
    with open(tsv, encoding="utf-8") as fh:
        for line in fh:
            line = line.rstrip("\n")
            if not line.strip() or line.startswith("#"):
                continue
            path, name, text = line.split("\t", 2)
            rows.append((path, name, text))

    by_file = {}
    for path, name, text in rows:
        by_file.setdefault(path, []).append((name, text))

    inserted = skipped = missing = 0
    for path, items in by_file.items():
        with open(path, encoding="utf-8") as fh:
            lines = fh.read().split("\n")
        for name, text in items:
            pat = re.compile(ITEM.format(re.escape(name)))
            idx = None
            for i, ln in enumerate(lines):
                if pat.match(ln):
                    idx = i
                    break
            if idx is None:
                print(f"MISSING {path}: {name}", file=sys.stderr)
                missing += 1
                continue
            # already marked in the attached block?
            j = idx - 1
            already = False
            while j >= 0 and re.match(r"^\s*(?://|#\[)", lines[j]):
                if "PROV[" in lines[j]:
                    already = True
                    break
                j -= 1
            if already:
                skipped += 1
                continue
            indent = pat.match(lines[idx]).group(1)
            prefix = "///" if (idx > 0 and lines[idx - 1].strip().startswith("///")) else "//"
            lines.insert(idx, f"{indent}{prefix} PROV[{text}")
            inserted += 1
        with open(path, "w", encoding="utf-8") as fh:
            fh.write("\n".join(lines))
    print(f"inserted {inserted}, already-marked {skipped}, missing {missing}")
    return 1 if missing else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1]))
