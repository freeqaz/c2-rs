#!/usr/bin/env python3
"""ilcmp.py — R4: what does the binding do to the IL c2 actually reads?

Declared in `work/w-refbind/PREREG.md` addendum §9.3, committed at `9064c2c`
before this file existed.

c2's ONLY input is the IL bundle, so two spellings that emit different obj bytes
must have different IL. That half is a tautology and is not the claim. R4
registers that the **bound** spelling's `.ex` (the function-body stream) is
STRICTLY LARGER than the unbound one's — and records a byte-IDENTICAL `.ex` as
the outcome worth the most, because it would put the whole effect in the symbol
table.

THE CONFOUND THIS FILE EXISTS TO AVOID
--------------------------------------
`bindgrid.py` gives every cell a unique struct and function name so the objs can
share a directory. Those names are IN the IL — a `.sy`/`.gl` stream carries them
verbatim — so comparing two bindgrid captures by size would compare their NAMES.
Every case here is written with the SAME struct name, the SAME function name and
the same formals; only the body differs.

Four cases, chosen because `refprobe.out` already grades them:

    none      direct `s->inner.aN`                         obj: prod first, prod r11
    ref       `L& q = s->inner;` (offset 96)               obj: const first, const r11
    head      `L& q = s->head;`  (offset 0, R8)            obj: prod first, prod r11
    unnamed   `(&s->inner)->aN`, same addresses, no name   obj: prod first, prod r11

so the IL sizes can be read against a KNOWN two-class partition of the objs.

SHIPS NOTHING.  Usage:  ilcmp.py
"""

import hashlib
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
C2RS = os.path.join(ROOT, "target", "release", "c2rs")
FLAGS = os.path.join(ROOT, "work", "dc3-workload", "flags.txt")


def _sib(name):
    d = ROOT
    while d != os.path.dirname(d):
        cand = os.path.join(os.path.dirname(d), name)
        if os.path.isdir(cand):
            return cand
        d = os.path.dirname(d)
    return None


DC3 = os.environ.get("C2RS_DC3") or _sib("dc3-decomp")

STRUCT = """\
struct L { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct S {
    L head;
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    L inner;
    L inner2;
};
"""

CASES = [
    ("none", "    s->f0 = 7;\n"
             "    s->inner.a0 = u << 3;\n    s->inner.a1 = u << 3;"),
    ("ref", "    L& q = s->inner;\n    s->f0 = 7;\n"
            "    q.a0 = u << 3;\n    q.a1 = u << 3;"),
    ("head", "    L& q = s->head;\n    s->f0 = 7;\n"
             "    q.a0 = u << 3;\n    q.a1 = u << 3;"),
    ("unnamed", "    s->f0 = 7;\n"
                "    (&s->inner)->a0 = u << 3;\n    (&s->inner)->a1 = u << 3;"),
]


def main():
    out = os.path.join(HERE, "ilcmp")
    os.makedirs(out, exist_ok=True)
    rows = {}
    for name, body in CASES:
        cpp = os.path.join(out, name + ".cpp")
        open(cpp, "w").write(STRUCT + "void g(S* s, int u, int v) {\n%s\n}\n" % body)
        ildir = os.path.join(out, name + ".il")
        os.makedirs(ildir, exist_ok=True)
        r = subprocess.run([C2RS, "capture", os.path.relpath(cpp, DC3),
                            "--keep-il", ildir, "--flags-file", FLAGS,
                            "--cwd", DC3],
                           capture_output=True, text=True, cwd=ROOT)
        if r.returncode != 0:
            print("  %-10s CAPTURE FAILED rc=%d" % (name, r.returncode))
            print("    " + (r.stderr or r.stdout).strip().splitlines()[-1:][0]
                  if (r.stderr or r.stdout).strip() else "")
            continue
        files = {}
        for fn in sorted(os.listdir(ildir)):
            p = os.path.join(ildir, fn)
            if not os.path.isfile(p):
                continue
            b = open(p, "rb").read()
            # the capture names embed a per-run tag; key on the EXTENSION
            files[os.path.splitext(fn)[1] or fn] = b
        rows[name] = files
        print("  %-10s %s" % (name, "  ".join(
            "%s=%d" % (e, len(b)) for e, b in sorted(files.items()))))

    if "none" not in rows or "ref" not in rows:
        print("\n  R4 UNGRADED — a capture failed")
        return 1

    exts = sorted(set().union(*(set(v) for v in rows.values())))
    print("\n  per-stream size, and identity against `none`:")
    print("  %-10s %s" % ("stream", "  ".join("%-22s" % n for n, _ in CASES)))
    for e in exts:
        cells = []
        for n, _ in CASES:
            b = rows.get(n, {}).get(e)
            if b is None:
                cells.append("%-22s" % "-")
                continue
            same = (b == rows["none"].get(e))
            cells.append("%-22s" % ("%d %s %s" % (
                len(b), "SAME" if same else "DIFF",
                hashlib.sha256(b).hexdigest()[:8])))
        print("  %-10s %s" % (e, "  ".join(cells)))

    # ---- where, exactly, do they differ ------------------------------------
    def firstdiff(a, b):
        for i in range(min(len(a), len(b))):
            if a[i] != b[i]:
                return i
        return min(len(a), len(b))

    ex = {n: rows[n][".ex"] for n, _ in CASES if ".ex" in rows.get(n, {})}
    if "ref" in ex and "head" in ex:
        a, b = ex["ref"], ex["head"]
        dd = [j for j in range(min(len(a), len(b))) if a[j] != b[j]]
        print("\n  `ref` (bind at displacement 96) vs `head` (bind at 0):")
        print("    equal length: %s | differing byte offsets: %s"
              % (len(a) == len(b), [hex(x) for x in dd]))
        for x in dd[:4]:
            print("      ref  @%04x  %s" % (x, a[max(0, x - 10):x + 10].hex(" ")))
            print("      head @%04x  %s" % (x, b[max(0, x - 10):x + 10].hex(" ")))

    if "none" in ex and "ref" in ex:
        a, b = ex["none"], ex["ref"]
        i = firstdiff(a, b)
        k = 0
        while k < min(len(a), len(b)) and a[len(a) - 1 - k] == b[len(b) - 1 - k]:
            k += 1
        print("\n  `none` vs `ref` — the edit window (first diff 0x%x, common tail %d):"
              % (i, k))
        print("    none %dB: %s" % (len(a) - k - i, a[i:len(a) - k].hex(" ")))
        print("    ref  %dB: %s" % (len(b) - k - i, b[i:len(b) - k].hex(" ")))

    ex_none = rows["none"].get(".ex")
    ex_ref = rows["ref"].get(".ex")
    print("\n  R4 as registered — `.ex` of the bound spelling is STRICTLY LARGER:")
    if ex_none is None or ex_ref is None:
        print("    UNGRADED — no `.ex` stream in the capture")
        return 1
    print("    none %d bytes | ref %d bytes | %s"
          % (len(ex_none), len(ex_ref),
             "HIT" if len(ex_ref) > len(ex_none)
             else "**MISS — byte-IDENTICAL**" if ex_ref == ex_none
             else "**MISS**"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
