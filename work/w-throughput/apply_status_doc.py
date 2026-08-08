#!/usr/bin/env python3
"""Update docs/STATUS.md's hand-written reproduction row for the merge gate.

Row 724 documented `scripts/gate.sh --jobs 8` and quoted `12/12 PASS, 2,940
verdicts` at tree `33cbdbe` — a 12-lane registry that is 18 lanes now. The gate's
default is 16 as of lane w-throughput, so the row is corrected to the invocation
that is actually the documented one (bare `scripts/gate.sh`) and re-quoted from a
run rather than from the page.

Usage: python3 work/w-throughput/apply_status_doc.py <verdict-text>
"""
import sys, pathlib

verdict = sys.argv[1] if len(sys.argv) > 1 else None
if not verdict:
    sys.exit("usage: apply_status_doc.py '<the verdict text to quote>'")

p = pathlib.Path("docs/STATUS.md")
s = p.read_text()

old = ("| **the merge gate** (12 mode lanes **+ the generated sweep + the mode cross**) "
       "| `scripts/gate.sh --jobs 8` — `12/12 PASS, 2,940 verdicts` at `33cbdbe` |")
new = ("| **the merge gate** (18 mode lanes **+ the generated sweep + the mode cross**) "
       "| `scripts/gate.sh --require-graded` — the default `--jobs` is **16** since "
       "2026-08-08 (it was 4, unchanged since the file was written; lane "
       "`w-throughput`, board #1323), and `--jobs` still overrides. " + verdict + " |")

n = s.count(old)
if n != 1:
    sys.exit(f"REFUSED: the gate row occurs {n} times, expected 1")
p.write_text(s.replace(old, new))
print("docs/STATUS.md: merge-gate row updated")
