#!/usr/bin/env python3
"""eboundary.py — where does MECHANISM E stop?

Lane w-inline measurement tooling. Read-only with respect to `crates/`.

`docs/INLINE_PREDICATE.md` §1 states mechanism E — *a call whose callee's SOURCE
body is empty is never emitted, and `/Ob` does not govern it* — and closes with
`NOT MODELLED` on three probes and one boundary. This file walks the boundary.

**Run AFTER the lane's rule was frozen and after GRID-2b was graded.** Nothing
here feeds `INLINE-P`; E is a separate mechanism and no constant of the
incumbent moves. Every row is a measurement, published as one; the section it
feeds says `NOT MODELLED` either way.

Each probe is compiled twice at the workload's own flags — with and without
`/Ob0` — and the question asked of each row is the same one p6/p2 answered:

    does the caller keep a REL24 to the callee at /Ob0?
      NO   -> mechanism E (the front end dropped the call)
      YES  -> not E; whatever /O1 does to it is mechanism I

Usage: eboundary.py [outdir]
"""

import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)
from scan_obj import read_obj  # noqa: E402

CALLER = "\nint sink;\nvoid use() { %s }\n"

PROBES = {
    # --- known answers, carried so the table can be read against them --------
    "e0-empty":        ("void g() {}", "g();"),
    "e0-ctl-nonempty": ("int g(int a) { return a + 1; }", "sink += g(sink);"),
    # --- the boundary --------------------------------------------------------
    "e1-unused-local": ("void g(int a) { int x; }", "g(sink);"),
    "e2-dead-store":   ("void g(int a) { int x = a; }", "g(sink);"),
    "e3-return-param": ("int g(int a) { return a; }", "sink += g(sink);"),
    "e4-return-const": ("int g() { return 0; }", "sink += g();"),
    "e5-static-empty": ("static void g() {}", "g();"),
    "e6-inline-empty": ("inline void g() {}", "g();"),
    "e7-member-empty": ("struct S { int m; void g() {} };\nstatic S s;",
                        "s.g();"),
    "e8-virtual-empty": ("struct S { int m; virtual void g() {} };\nstatic S s;",
                         "s.S::g();"),
    "e9-empty-loop":   ("void g(int a) { for (int i = 0; i < a; ++i) {} }",
                        "g(sink);"),
    "e10-empty-body-arg-effect": ("void g(int a) {}", "g(sink++);"),
}


def sib(name):
    d = REPO
    while d != "/":
        if os.path.isdir(os.path.join(d, "..", name)):
            return os.path.abspath(os.path.join(d, "..", name))
        d = os.path.dirname(d)
    return None


def main(argv):
    out = argv[0] if argv else os.path.join(HERE, "eb")
    os.makedirs(out, exist_ok=True)
    wibo = os.environ.get("C2RS_WIBO") or os.path.join(sib("wibo") or "", "build/release/wibo")
    cl = os.path.join(REPO, "compilers/X360/16.00.11886.00/cl.exe")
    if not (os.path.exists(wibo) and os.path.exists(cl)):
        print("SKIP: toolchain absent")
        return 3
    flags = open(os.path.join(REPO, "work/dc3-workload/flags.txt")).read().split()
    print(f"{'probe':30s} {'callee s':>8s} {'/O1 call':>9s} {'/Ob0 call':>10s}  verdict")
    for name, (decl, call) in PROBES.items():
        src = os.path.join(out, name + ".cpp")
        open(src, "w").write(decl + "\n" + CALLER % call)
        r = {}
        for tag, extra in (("o1", []), ("ob0", ["/Ob0"])):
            obj = os.path.join(out, f"{name}_{tag}.obj")
            subprocess.run(
                [wibo, cl] + flags + extra +
                ["/Fo" + "Z:" + os.path.abspath(obj).replace("/", "\\"),
                 "Z:" + os.path.abspath(src).replace("/", "\\")],
                capture_output=True, cwd=out,
                env={**os.environ, "TMP": out, "TEMP": out, "WIBO_FS_CACHE": "1"})
            if not os.path.exists(obj):
                r[tag] = None
                continue
            fns = read_obj(obj)
            # The callee is whichever defined function `use` could name; found by
            # walking the symbols, never by position (#644).
            callee = next((n for n in fns if n.startswith("?g@") or "?g@" in n), None)
            user = next((n for n in fns if n.startswith("?use@")), None)
            r[tag] = (fns[callee].size if callee else None,
                      bool(user and callee and callee in fns[user].rel24))
        if r.get("o1") is None or r.get("ob0") is None:
            print(f"{name:30s} {'—':>8s} {'COMPILE-FAIL':>9s}")
            continue
        s, o1 = r["o1"]
        _, ob0 = r["ob0"]
        verdict = "E — front-end elision" if not ob0 else (
            "I — inline expansion" if not o1 else "neither: the call survives")
        print(f"{name:30s} {str(s):>8s} {'yes' if o1 else 'no':>9s} "
              f"{'yes' if ob0 else 'no':>10s}  {verdict}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
