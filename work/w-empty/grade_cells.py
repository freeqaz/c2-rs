#!/usr/bin/env python3
"""grade_cells.py — compile the w-empty GRID twice and read the verdict off the
reference objs.

Lane w-empty measurement tooling. **Read-only with respect to `crates/`.**

    grade_cells.py <celldir> <objdir> [--jobs N]

For every cell: compile at the workload's own flags and again with `/Ob0`
appended, then for the cell's CALLER print

    the whole `.text` COMDAT, word for word, at the workload's flags
    the REL24 targets, by NAME (#644: the symbol index is followed, never
        assumed to sit at a position)
    the same two at /Ob0

and grade

    E        no REL24 to the callee at EITHER setting — the front end dropped it
    I        none at /O1, one at /Ob0 — inline expansion
    CALL     one at both — an ordinary call
    NODIRECT the site is indirect (`bcctrl`): this observable cannot distinguish
             "dropped" from "there was never a direct call", exactly as
             `INLINE_PREDICATE.md` §4 records for `virt-ptr`. NOT GRADED.
    NOCELL   the reader could not find the caller, or the per-cell ANCHOR control
             failed. NOT GRADED — never scored as a verdict.

The anchor is checked first and a cell whose anchor lost its relocation is
refused, not scored.
"""

import os
import re
import subprocess
import sys
import concurrent.futures

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "scripts"))
sys.path.insert(0, os.path.join(ROOT, "work", "w-inline"))

from scan_obj import read_obj  # noqa: E402

sys.path.insert(0, HERE)
from gen_cells import CELLS  # noqa: E402

ANCHOR_SYM = "?anchor@@YAXXZ"
ANCHOR_TARGET = "?ext_anchor@@YAXXZ"


def mangle_prefix(spec):
    """The mangled-name PREFIX one demangled spelling must start with.

    Not a positional read of the mangled string (#644): the prefix is derived
    from the spelling's structure and the match is `startswith`, so a name that
    merely CONTAINS `g@` cannot be mistaken for `?g@@`.
    """
    if spec is None:
        return None
    if "::" in spec:
        cls, member = spec.split("::", 1)
        if member == cls:
            return "??0%s@@" % cls
        if member == "~" + cls:
            return "??1%s@@" % cls
        return "?%s@%s@@" % (member, cls)
    return "?%s@@" % spec


def find(fns, prefix):
    if prefix is None:
        return None
    hits = [f for n, f in fns.items() if n.startswith(prefix)]
    if len(hits) != 1:
        return None
    return hits[0]


def is_bcctrl(w):
    return (w >> 26) == 19 and ((w >> 1) & 0x3FF) == 528


def compile_cell(cid, celldir, objdir):
    src = os.path.join(celldir, cid + ".cpp")
    outs = {}
    for tag, extra in (("o1", []), ("ob0", ["/Ob0"])):
        obj = os.path.join(objdir, "%s.%s.obj" % (cid, tag))
        env = dict(os.environ)
        env["C2RS_EXTRA_FLAGS"] = " ".join(extra)
        r = subprocess.run(
            [os.path.join(HERE, "probe2.sh"), src, obj],
            capture_output=True, text=True, env=env,
        )
        outs[tag] = obj if (r.returncode == 0 and os.path.exists(obj)) else None
    return cid, outs


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    celldir, objdir = argv[0], argv[1]
    jobs = 8
    if "--jobs" in argv:
        jobs = int(argv[argv.index("--jobs") + 1])
    os.makedirs(objdir, exist_ok=True)

    built = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as ex:
        for cid, outs in ex.map(
            lambda c: compile_cell(c[0], celldir, objdir), CELLS
        ):
            built[cid] = outs

    print("cells: %d   compiled(/O1): %d   compiled(/Ob0): %d" % (
        len(CELLS),
        sum(1 for v in built.values() if v["o1"]),
        sum(1 for v in built.values() if v["ob0"]),
    ))
    print()
    hdr = "%-22s %-9s %-30s %s" % ("cell", "verdict", "?f .text at /O1", "rel24 /O1 -> /Ob0")
    print(hdr)
    print("-" * len(hdr))

    counts = {}
    rows = []
    for cid, callee, caller, _ in CELLS:
        o1, ob0 = built[cid]["o1"], built[cid]["ob0"]
        if not o1 or not ob0:
            rows.append((cid, "NOCELL", "compile failed", ""))
            counts["NOCELL"] = counts.get("NOCELL", 0) + 1
            continue
        f1, f0 = read_obj(o1), read_obj(ob0)
        # ---- the per-cell POSITIVE CONTROL, checked before any verdict -------
        ok = True
        for fns in (f1, f0):
            a = fns.get(ANCHOR_SYM)
            if a is None or ANCHOR_TARGET not in a.rel24:
                ok = False
        if not ok:
            rows.append((cid, "NOCELL", "anchor control failed", ""))
            counts["NOCELL"] = counts.get("NOCELL", 0) + 1
            continue
        cpre, fpre = mangle_prefix(callee), mangle_prefix(caller)
        c1, cl1 = find(f1, cpre), find(f1, fpre)
        cl0 = find(f0, fpre)
        if cl1 is None or cl0 is None:
            rows.append((cid, "NOCELL", "caller COMDAT not found", ""))
            counts["NOCELL"] = counts.get("NOCELL", 0) + 1
            continue
        # The callee's mangled name, taken from the /O1 obj when it is emitted
        # and from the /Ob0 obj otherwise (an elided callee may be dropped when
        # it has internal linkage and no surviving reference).
        cname = c1.name if c1 else (find(f0, cpre).name if find(f0, cpre) else None)
        n1 = sum(1 for t in cl1.rel24 if cname and t == cname)
        n0 = sum(1 for t in cl0.rel24 if cname and t == cname)
        if any(is_bcctrl(w) for w in cl1.words):
            v = "NODIRECT"
        elif cname is None:
            v = "NOCELL"
        elif n1 == 0 and n0 == 0:
            v = "E"
        elif n1 == 0 and n0 > 0:
            v = "I"
        elif n1 > 0 and n0 > 0:
            v = "CALL"
        else:
            v = "ODD"
        counts[v] = counts.get(v, 0) + 1
        words = " ".join("%08x" % w for w in cl1.words)
        rows.append((cid, v, words, "%d -> %d  (%s)" % (n1, n0, ",".join(cl1.rel24) or "-")))

    for r in rows:
        print("%-22s %-9s %-30s %s" % r)
    print()
    print("verdicts: " + "  ".join("%s %d" % kv for kv in sorted(counts.items())))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
