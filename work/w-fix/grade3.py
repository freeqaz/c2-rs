#!/usr/bin/env python3
"""grade3.py — compile GRID-3 twice and grade it PER CALL EDGE.

Lane w-fix measurement tooling. **Read-only with respect to `crates/`.**

    grade3.py <celldir> <objdir> [--jobs N]

`work/w-empty/grade_cells.py` grades one (caller, callee) pair per cell, which
is all a one-step rule needs. A fixpoint is a statement about a *chain*, so this
grader takes a list of edges per cell and scores each of them:

    E        no REL24 caller->callee at EITHER setting — the call was dropped
    I        none at /O1, one at /Ob0 — inline expansion
    CALL     one at both — an ordinary call
    NODIRECT the caller's body contains a `bcctrl`: this observable cannot tell
             "dropped" from "there was never a direct call". NOT GRADED.
    NOEDGE   the caller or the callee COMDAT could not be resolved. NOT GRADED.

and, beside the verdict, the observable the port actually has to reproduce:
**is the caller's whole `.text` COMDAT one `4e800020`**. A chain link can lose
its relocation and still not be a bare `blr` (`f05_side_effect_arg`), and a rule
that ships bytes has to see the difference.

The per-cell ANCHOR control is checked before any edge of that cell is scored.

#843: graded from obj bytes, never from a listing. #644: the callee's symbol is
resolved by name through the symbol table, never by position.
"""

import concurrent.futures
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "scripts"))
sys.path.insert(0, os.path.join(ROOT, "work", "w-inline"))

from scan_obj import read_obj  # noqa: E402

sys.path.insert(0, HERE)
from gen_cells3 import CELLS  # noqa: E402

PROBE = os.path.join(ROOT, "work", "w-empty", "probe2.sh")
ANCHOR_SYM = "?anchor@@YAXXZ"
ANCHOR_TARGET = "?ext_anchor@@YAXXZ"
BLR = 0x4E800020


def mangle_prefix(spec):
    """The mangled-name PREFIX one demangled spelling must start with.

    Derived from the spelling's structure and matched with `startswith`, so
    `?h@@` cannot be mistaken for `?h1@@`.
    """
    if "::" in spec:
        cls, member = spec.split("::", 1)
        if member == cls:
            return "??0%s@@" % cls
        if member == "~" + cls:
            return "??1%s@@" % cls
        return "?%s@%s@@" % (member, cls)
    return "?%s@@" % spec


def find(fns, prefix):
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
        r = subprocess.run([PROBE, src, obj], capture_output=True, text=True, env=env)
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
        for cid, outs in ex.map(lambda c: compile_cell(c[0], celldir, objdir), CELLS):
            built[cid] = outs

    print("cells: %d   compiled(/O1): %d   compiled(/Ob0): %d   edges: %d" % (
        len(CELLS),
        sum(1 for v in built.values() if v["o1"]),
        sum(1 for v in built.values() if v["ob0"]),
        sum(len(e) for _, e, _ in CELLS),
    ))
    print()
    hdr = "%-22s %-14s %-8s %-6s %-9s %s" % (
        "cell", "edge", "verdict", "blr?", "rel24", "caller .text at /O1")
    print(hdr)
    print("-" * len(hdr))

    counts = {}
    rows = []
    for cid, edges, _ in CELLS:
        o1, ob0 = built[cid]["o1"], built[cid]["ob0"]
        bad = None
        if not o1 or not ob0:
            bad = "compile failed"
            f1 = f0 = None
        else:
            f1, f0 = read_obj(o1), read_obj(ob0)
            for fns in (f1, f0):
                a = fns.get(ANCHOR_SYM)
                if a is None or ANCHOR_TARGET not in a.rel24:
                    bad = "anchor control failed"
        for caller, callee in edges:
            tag = "%s->%s" % (caller, callee)
            if bad:
                rows.append((cid, tag, "NOEDGE", "-", "-", bad))
                counts["NOEDGE"] = counts.get("NOEDGE", 0) + 1
                continue
            cl1 = find(f1, mangle_prefix(caller))
            cl0 = find(f0, mangle_prefix(caller))
            ce1 = find(f1, mangle_prefix(callee))
            ce0 = find(f0, mangle_prefix(callee))
            cname = ce1.name if ce1 else (ce0.name if ce0 else None)
            # An edge whose callee this TU does not DEFINE has no COMDAT to
            # resolve, so the name comes from the caller's own relocation
            # targets — still by name and still by prefix, never by position.
            # If no relocation names it at either setting the edge is NOEDGE and
            # not `E`: "the call was dropped" and "no such call was ever
            # emitted" are indistinguishable then, which is trap 5.
            if cname is None:
                pre = mangle_prefix(callee)
                cand = {t for cl in (cl1, cl0) if cl for t in cl.rel24 if t.startswith(pre)}
                cname = cand.pop() if len(cand) == 1 else None
            if cl1 is None or cl0 is None or cname is None:
                rows.append((cid, tag, "NOEDGE", "-", "-", "caller/callee COMDAT not found"))
                counts["NOEDGE"] = counts.get("NOEDGE", 0) + 1
                continue
            n1 = sum(1 for t in cl1.rel24 if t == cname)
            n0 = sum(1 for t in cl0.rel24 if t == cname)
            if any(is_bcctrl(w) for w in cl1.words):
                v = "NODIRECT"
            elif n1 == 0 and n0 == 0:
                v = "E"
            elif n1 == 0 and n0 > 0:
                v = "I"
            elif n1 > 0 and n0 > 0:
                v = "CALL"
            else:
                v = "ODD"
            counts[v] = counts.get(v, 0) + 1
            blr = "blr" if cl1.words == [BLR] else "%dw" % len(cl1.words)
            words = " ".join("%08x" % w for w in cl1.words)
            rows.append((cid, tag, v, blr, "%d->%d" % (n1, n0), words))

    for r in rows:
        print("%-22s %-14s %-8s %-6s %-9s %s" % r)
    print()
    print("edge verdicts: " + "  ".join("%s %d" % kv for kv in sorted(counts.items())))
    graded = sum(v for k, v in counts.items() if k in ("E", "I", "CALL"))
    print("graded: %d of %d edges" % (graded, sum(len(e) for _, e, _ in CELLS)))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
