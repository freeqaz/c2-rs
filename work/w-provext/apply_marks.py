#!/usr/bin/env python3
"""apply_marks.py — insert one PROV[X] doc line above each named const.

Lane `w-provext` scratch, copied verbatim from `work/w-provenance/apply_marks.py`
(a peer lane's committed surface, never edited in place) and then given ONE
change: an optional `path:LINE` form, because this lane's scope contains files
that declare the same const name more than once — `c2-il/src/func/body/expr.rs`
declares five distinct function-local `const ON`, and name-only targeting always
hits the first.

Original lane `w-provenance` scratch. COMMENT-ONLY by construction: it inserts a line
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
            want_line = None
            if ":" in name:
                name, want_line = name.split(":", 1)
                want_line = int(want_line)
            pat = re.compile(ITEM.format(re.escape(name)))
            idx = None
            for i, ln in enumerate(lines):
                if not pat.match(ln):
                    continue
                # `path <TAB> NAME:LINE` pins the occurrence by its ORIGINAL
                # line number. Insertions above it shift later lines, so the
                # match is accepted at or after the requested line and the TSV
                # must list a file's sites in ascending line order.
                if want_line is not None and i + 1 < want_line:
                    continue
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
