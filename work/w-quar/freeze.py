#!/usr/bin/env python3
"""freeze.py — sha256 every file the frozen model's answer actually depends on.

`predict.py` alone is not the model: it imports landed-lane modules by value, and
a change to any of them changes the prediction.  The digest list is over the
WHOLE closure, and it is committed BEFORE any quarantined obj is read so the
model that was tested can be reconstructed exactly.

The list is not hand-kept.  `predict.py` is imported and every module it left in
`sys.modules` that lives under `work/` is digested at its RESOLVED `__file__` —
so a module that silently resolves to a different lane's copy of an ambiguous
name (`scan.py`, `glowner.py` and `marks.py` all exist in more than one lane)
is recorded as the file that was really used, not as the one intended.

    usage: freeze.py <main-repo-root>
"""
import hashlib
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))


def main():
    main_root = os.path.abspath(sys.argv[1])
    os.environ.setdefault("C2RS_LANEROOT", main_root)
    sys.path.insert(0, HERE)
    import predict  # noqa: F401

    seen = {}
    for name, mod in list(sys.modules.items()):
        f = getattr(mod, "__file__", None)
        if not f:
            continue
        f = os.path.abspath(f)
        if os.sep + "work" + os.sep not in f:
            continue
        seen[f] = name
    rows = []
    for f in sorted(seen):
        h = hashlib.sha256(open(f, "rb").read()).hexdigest()
        # This lane's own files live in a worktree; name them by lane path so no
        # absolute machine path is ever committed.
        under_main = (f.startswith(main_root + os.sep)
                      and os.path.join(".claude", "worktrees") not in f)
        rel = (os.path.relpath(f, main_root) if under_main
               else "work/w-quar/" + os.path.basename(f))
        rows.append((h, rel, seen[f]))
    for h, rel, name in rows:
        print("%s  %-44s  (module %s)" % (h, rel, name))
    agg = hashlib.sha256(
        "".join("%s %s\n" % (h, rel) for h, rel, _ in rows).encode()).hexdigest()
    print("\nMODEL-CLOSURE-SHA256  %s   over %d files" % (agg, len(rows)))


if __name__ == "__main__":
    main()
