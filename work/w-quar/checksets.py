#!/usr/bin/env python3
"""checksets.py — verify the committed prediction file against itself and the grade.

`pred21.sets.txt` is self-digesting: each block header carries the sha256 of the
lines that follow it.  This re-derives every digest, then re-grades `JFP_ALIAS`
straight out of the committed text — so the verdict can be reproduced from the
git object alone, with no jsonl and no frozen model in the loop.

    usage: checksets.py <pred21.sets.txt> <truth-dir>
"""
import hashlib
import os
import sys


def slug(s):
    return s.replace("/", "__").replace("\\", "__")


def blocks(path):
    src = model = None
    want = None
    names = []
    for ln in open(path):
        if ln.startswith("== "):
            if src is not None:
                yield src, model, want, names
            head = ln[3:].rstrip("\n").split("  ")
            src, model = head[0], head[1]
            want = head[3].split("=", 1)[1]
            names = []
        else:
            names.append(ln.rstrip("\n"))
    if src is not None:
        yield src, model, want, names


def main():
    setsp, truthd = sys.argv[1], sys.argv[2]
    ok = bad = 0
    exact = {}
    for src, model, want, names in blocks(setsp):
        got = hashlib.sha256(("\n".join(names) + "\n").encode()).hexdigest()
        if got == want:
            ok += 1
        else:
            bad += 1
            print("  DIGEST MISMATCH %s %s" % (src, model))
        tf = os.path.join(truthd, slug(src) + ".txt")
        if os.path.exists(tf):
            E = set(x for x in open(tf).read().split() if x)
            if set(names) == E:
                exact.setdefault(model, []).append(src)
    print("blocks %d ; digests verified %d ; mismatched %d" % (ok + bad, ok, bad))
    for m in sorted(exact):
        print("  %-10s exact %d  (re-graded from the COMMITTED text alone)"
              % (m, len(exact[m])))
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
