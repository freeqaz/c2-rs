#!/usr/bin/env python3
"""scrub.py — replace this checkout's absolute path with `<repo>` in the files
named on the command line, in place, and then **assert** no `/home/` survives.

`CLAUDE.md` forbids committing absolute machine paths. The assert is the point:
a scrubber that silently misses a form is worse than none, so this one dies
rather than writing a file it did not fully clean.
"""
import os
import sys

ROOT = os.path.realpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

for path in sys.argv[1:]:
    with open(path, "r", encoding="utf-8", errors="replace") as fh:
        text = fh.read()
    text = text.replace(ROOT, "<repo>")
    if "/home/" in text:
        bad = [ln for ln in text.splitlines() if "/home/" in ln]
        raise SystemExit(f"scrub.py: {path} still carries an absolute path:\n  " + "\n  ".join(bad[:5]))
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(text)
    print(f"scrubbed {path}")
