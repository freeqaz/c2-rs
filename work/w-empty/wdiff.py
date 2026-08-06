#!/usr/bin/env python3
"""wdiff.py — which emitted functions changed FBM verdict between two scans.

Lane w-empty measurement tooling. **Read-only with respect to `crates/`.**

    wdiff.py <before.jsonl> <after.jsonl>

Keys off the scan's own `fnbyte-differs-fn|<shape>|w…|first@…|<symbol>` witness
keys, per `(TU, symbol)`. A symbol that leaves the set became `exact` (or
`refused`); one that enters it became `differs`. Both directions are printed
with a count, because a lane that prints only the direction it wants is the
failure mode `docs/STATUS.md` trap 5 names.

The provenance record is never echoed — it carries this machine's directories.
"""

import json
import sys


def load(path):
    out = {}
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        src = r["src"]
        for k in (r.get("emit") or {}):
            if k.startswith("fnbyte-differs-fn|"):
                _, shape, words, first, sym = k.split("|", 4)
                out[(src, sym)] = (shape, words, first)
    return out


def main(argv):
    a, b = load(argv[0]), load(argv[1])
    gone = sorted(set(a) - set(b))
    new = sorted(set(b) - set(a))
    print("differs BEFORE %d  AFTER %d   left %d   entered %d"
          % (len(a), len(b), len(gone), len(new)))
    print("\n--- ENTERED `differs` (the regression direction) ---")
    for k in new:
        print("  %-60s %s  %s %s" % (k[1][:60], k[0], b[k][0], b[k][1]))
        print("      %s" % (b[k][2],))
    print("\n--- LEFT `differs` (the conversion direction) ---")
    for k in gone[:40]:
        print("  %-60s %s  %s %s" % (k[1][:60], k[0], a[k][0], a[k][1]))
    if len(gone) > 40:
        print("  … %d more" % (len(gone) - 40))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
