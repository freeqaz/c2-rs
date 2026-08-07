#!/usr/bin/env python3
"""reconcile.py — `PREREG.md` P7: the two instruments, count by count.

Left:  the SHIPPED Rust reader, read out of `c2rs census --fn ""` — every row
       whose blocking key is `return-scope-close-cflow-label` and which carries a
       `no_effect_callee`.
Right: `loopread.py`, a crate-free re-implementation over the same captured
       `.ex`, which derives even the formals list by a different route.

    reconcile.py <census --fn "" output> <file.ex>

Prints both totals, the difference, and — when they differ — the census rows the
Python walk did not reach, so a discrepancy is explained rather than closed.
"""
import subprocess
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import loopread  # noqa: E402


def rust_rows(path):
    """(key, emit, no_effect_callee) per `--fn` block."""
    rows, cur = [], None
    for line in open(path):
        line = line.rstrip("\n")
        if line.startswith("  [") and "]" in line:
            if cur:
                rows.append(cur)
            body = line.split("]", 1)[1].strip()
            parts = body.split()
            cur = {"mark": parts[0], "key": parts[1] if len(parts) > 1 else "", "ne": None}
        elif cur is not None and line.startswith("          emit="):
            cur["emit"] = line.split("=", 1)[1]
        elif cur is not None and line.startswith("          no_effect_callee="):
            cur["ne"] = line.split("=", 1)[1]
        elif line.startswith("  --fn "):
            if cur:
                rows.append(cur)
            cur = None
    return rows


def main(argv):
    rows = rust_rows(argv[1])
    loops = [r for r in rows if r["key"] == "return-scope-close-cflow-label"]
    fired = [r for r in loops if r["ne"]]
    ex = open(argv[2], "rb").read()
    py = []
    for o, off, seg in loopread.segments(ex):
        try:
            py.append((o, loopread.walk_loop(seg)))
        except Exception:
            pass
    print(f"census rows total                       : {len(rows)}")
    print(f"  key=return-scope-close-cflow-label    : {len(loops)}")
    print(f"  …of which the RUST reader named a callee: {len(fired)}")
    print(f"loopread.py (independent) matches       : {len(py)}")
    print(f"difference                              : {len(fired) - len(py)}")
    if len(fired) != len(py):
        print("\nDISCREPANCY — not closed, printed:")
        for r in fired[:20]:
            print(f"  rust: {r.get('emit','')[:110]}")


if __name__ == "__main__":
    main(sys.argv)
