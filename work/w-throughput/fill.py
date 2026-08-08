#!/usr/bin/env python3
"""Substitute the lane's measured numbers into the rung doc and the board rows.

Placeholders are deliberate: the prose was written before the runs finished, and
a number typed into prose from memory is how this project's pages go stale. Every
one of these is filled from a run's own output.
"""
import pathlib, sys

SUBS = {}


def load(path):
    return pathlib.Path(path).read_text()


def main():
    if not SUBS:
        sys.exit("nothing to substitute — edit SUBS first")
    for f in ("docs/rungs/2026-08-08-w-throughput.md", "docs/BOARD.md",
              "scripts/gate.sh", "work/w-throughput/gate_ab.md"):
        p = pathlib.Path(f)
        if not p.exists():
            continue
        s = p.read_text()
        hit = 0
        for k, v in SUBS.items():
            if k in s:
                s = s.replace(k, v)
                hit += 1
        if hit:
            p.write_text(s)
            print(f"{f}: {hit} placeholder(s) filled")
    # Positive check: no placeholder may survive.
    left = []
    for f in ("docs/rungs/2026-08-08-w-throughput.md", "docs/BOARD.md",
              "scripts/gate.sh", "work/w-throughput/gate_ab.md"):
        p = pathlib.Path(f)
        if not p.exists():
            continue
        s = p.read_text()
        for k in SUBS:
            if k in s:
                left.append((f, k))
    if left:
        sys.exit(f"REFUSED: placeholders survived: {left}")
    print("no placeholder survives")


if __name__ == "__main__":
    main()
