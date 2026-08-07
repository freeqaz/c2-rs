#!/usr/bin/env python3
"""freeze.py — sha256 the WHOLE closure the root rule's answer depends on.

w-quar's freeze.py, pointed at this lane's model (`rootmodel` + `rules`) instead
of `predict.py`.  The list is not hand-kept: the model is imported and every
module it left in `sys.modules` that lives under `work/` is digested at its
RESOLVED `__file__`, so an ambiguous name (`scan.py`, `glowner.py`, `marks.py`
and `alias.py` each exist in more than one lane) is recorded as the file that was
really used rather than the one intended.

Committed BEFORE the held-out 200 are scored, so the model that was tested can be
reconstructed exactly and a later edit cannot be mistaken for the tested one.

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
    import rootmodel  # noqa: F401
    import rules      # noqa: F401

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
        under_main = (f.startswith(main_root + os.sep)
                      and os.path.join(".claude", "worktrees") not in f)
        rel = (os.path.relpath(f, main_root) if under_main
               else "work/w-root/" + os.path.basename(f))
        rows.append((h, rel, seen[f]))
    for h, rel, name in rows:
        print("%s  %-44s  (module %s)" % (h, rel, name))
    agg = hashlib.sha256(
        "".join("%s %s\n" % (h, rel) for h, rel, _ in rows).encode()).hexdigest()
    print("\nMODEL-CLOSURE-SHA256  %s   over %d files" % (agg, len(rows)))


if __name__ == "__main__":
    main()
