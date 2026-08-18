#!/usr/bin/env python3
"""gridD.py — what the WHOLE-FILE refusal cost, constructed and graded by c2.

The workload witness is `src/xdk/nuiapi/headtracker.cpp`: it contains a
`__declspec(noinline)`-flagged record (`??$sprintf_s@…`, ATTR `0x88`, a
**direct**-form `SIZE` of 46) and, separately, a function whose `SIZE` escapes.
The incumbent reader refused the WHOLE file on the escaped record, so the
`noinline` bit on the small one was invisible too — and `IlFunction::inlinable`
read `None`, which `splice` treats exactly as it did before the field existed:
it may expand the body.

This grid reproduces that shape from source and grades it against real c2, so
the claim is a demonstration and not a reading of one workload file.

Four cells, at the workload profile:

    A  small noinline callee only                 -> reader decodes, bit visible
    B  A + a function whose SIZE escapes          -> INCUMBENT refuses the file
    C  small INLINABLE callee + escaping function -> the control: nothing to lose
    D  B at /Ox                                   -> profile scope

Per cell: real c2's own verdict on `caller -> small` (does the caller's `.text`
COMDAT carry a `REL24` naming it?), and both readers' whole-file answer.
"""

import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(ROOT, "work", "w-sizebracket"))
import glrec  # noqa: E402
from series import Obj  # noqa: E402

C2RS = os.path.join(ROOT, "target", "release", "c2rs")
SMALL = "?small@@YAHH@Z"
BIG = "?big@@YAHH@Z"
CALLER = "?caller@@YAHH@Z"


def big_body(n=14):
    L = ["  int s = a;"]
    for i in range(n):
        L.append(f"  s = (s * {2 * i + 3} + {i + 1}) ^ (s >> {(i % 7) + 1});")
    L.append("  return s;")
    return "\n".join(L)


def src(noinline_small, with_big):
    q = "__declspec(noinline) " if noinline_small else " " * 21
    out = f"{q}int small(int a) {{ return a + 1; }}\n"
    if with_big:
        out += f"int big(int a) {{\n{big_body()}\n}}\n"
    out += "int caller(int a) { return small(a)" + (" + big(a)" if with_big else "") + "; }\n"
    return out


def run(tag, text, flags, outdir):
    cpp = os.path.join(outdir, "cells", tag + ".cpp")
    os.makedirs(os.path.dirname(cpp), exist_ok=True)
    open(cpp, "w").write(text)
    ildir = os.path.join(outdir, "il", tag)
    obj = os.path.join(outdir, "obj", tag + ".obj")
    os.makedirs(os.path.dirname(obj), exist_ok=True)
    env = dict(os.environ, C2RS_REQUIRE_TOOLCHAIN="1")
    rel = lambda q: os.path.relpath(q, ROOT)
    r1 = subprocess.run([C2RS, "capture", rel(cpp), "--keep-il", rel(ildir),
                         "--flags-file", rel(flags)],
                        capture_output=True, text=True, cwd=ROOT, env=env)
    r2 = subprocess.run([C2RS, "compile", rel(cpp), "--keep-obj", rel(obj),
                         "--flags-file", rel(flags)],
                        capture_output=True, text=True, cwd=ROOT, env=env)
    if r1.returncode or r2.returncode:
        return {"tag": tag, "error": (r1.stderr + r2.stderr)[-300:]}
    gl = open(os.path.join(ildir,
              [x for x in os.listdir(ildir) if x.endswith(".gl")][0]), "rb").read()
    inc, new, recs = glrec.file_verdict(gl, glrec.framed_incumbent)
    by = {r["name"]: r for r in recs}
    o = Obj(open(obj, "rb").read())
    ci = o.comdat_of(CALLER)
    targets = o.relocation_targets(ci) if ci else []
    return {
        "tag": tag,
        "incumbent": "REFUSED" if inc else "ok",
        "incumbent_cause": inc,
        "new": "REFUSED" if new else "ok",
        "small_attr": by.get(SMALL, {}).get("attr"),
        "small_form": by.get(SMALL, {}).get("form"),
        "big_form": by.get(BIG, {}).get("form"),
        "big_size": by.get(BIG, {}).get("size"),
        "c2_keeps_small": SMALL in targets,
        "c2_keeps_big": BIG in targets,
    }


def main():
    outdir = os.path.join(HERE, "gridD")
    O1 = os.path.join(ROOT, "work", "w-sizebracket", "flags_O1.txt")
    OX = os.path.join(ROOT, "work", "w-sizebracket", "flags_Ox.txt")
    cells = [
        ("A_noinline_only_O1", src(True, False), O1),
        ("B_noinline_plus_escape_O1", src(True, True), O1),
        ("C_inlinable_plus_escape_O1", src(False, True), O1),
        ("D_noinline_plus_escape_Ox", src(True, True), OX),
    ]
    print(f"{'cell':>28} {'incumbent':>10} {'cause':>13} {'new':>8} "
          f"{'smallATTR':>10} {'bigSIZE':>8} {'bigForm':>8} "
          f"{'c2 keeps small':>15} {'c2 keeps big':>13}")
    for tag, text, flags in cells:
        r = run(tag, text, flags, outdir)
        if "error" in r:
            print(f"{tag:>28}  ERROR {r['error']}")
            continue
        a = "None" if r["small_attr"] is None else f"{r['small_attr']:02x}"
        print(f"{tag:>28} {r['incumbent']:>10} {str(r['incumbent_cause']):>13} "
              f"{r['new']:>8} {a:>10} {str(r['big_size']):>8} "
              f"{str(r['big_form']):>8} {str(r['c2_keeps_small']):>15} "
              f"{str(r['c2_keeps_big']):>13}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
