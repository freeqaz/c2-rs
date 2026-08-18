#!/usr/bin/env python3
"""twins.py — GRID-A. The `.gl` SIZE escape's width, confirmed BLACK BOX.

One cell = one callee body, compiled TWICE from sources that differ only by
`__declspec(noinline)` on the callee, at one profile.  `__declspec(noinline)` is
the one thing measured to clear `FN_FLAG_INLINABLE` (`w-mmioclose`), so the two
`.gl` files must differ in **exactly one byte** and that byte is the record's
ATTR.  Where the callee's SIZE field escapes, the position of that byte relative
to the `0x80` is a direct, black-box read of the escape's width — no
disassembly involved.

Per cell we report:

  * the byte offsets at which the two `.gl` files differ, and the XOR;
  * the SIZE field's form and value, and the ATTR offset the 3-byte decode
    predicts;
  * `diff_at_predicted`  — the falsifiable claim;
  * real c2's own verdict on each twin: does the caller's `.text` COMDAT carry
    a `REL24` naming the callee?

Derived from the log, never accumulated.
"""

import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(ROOT, "work", "w-sizebracket"))
import glrec  # noqa: E402
from series import Obj  # noqa: E402  (the COFF reader w-sizebracket already wrote)

C2RS = os.path.join(ROOT, "target", "release", "c2rs")
CALLEE = "?callee@@YAHH@Z"
CALLER = "?caller@@YAHH@Z"


def body_mix(n):
    L = ["  int s = a;"]
    for i in range(n):
        L.append(f"  s = (s * {2 * i + 3} + {i + 1}) ^ (s >> {(i % 7) + 1});")
    L.append("  return s;")
    return "\n".join(L)


# The twins must be BYTE-LENGTH IDENTICAL and compiled from the SAME PATH.
# `.gl` carries the source path and every record's SRCPOS, so a 21-character
# attribute inserted at the front shifts all of them and the diff becomes 130
# bytes of noise instead of the one byte the experiment is about. Padding the
# plain twin with 21 spaces holds every other byte of the container fixed —
# which is what makes "the twins differ in EXACTLY ONE byte" a falsifiable
# claim rather than a hopeful one.
ATTRQ = "__declspec(noinline) "


def cell_src(n, noinline):
    q = ATTRQ if noinline else " " * len(ATTRQ)
    return (
        f"{q}int callee(int a) {{\n{body_mix(n)}\n}}\n"
        f"int caller(int a) {{ return callee(a) + 7; }}\n"
    )


def one(n, noinline, profile, flags, outdir):
    tag = f"mix{n:03d}_{'ni' if noinline else 'pl'}_{profile}"
    # ONE path for both twins — see `cell_src`.
    cpp = os.path.join(outdir, "cells", f"twin_{profile}.cpp")
    os.makedirs(os.path.dirname(cpp), exist_ok=True)
    open(cpp, "w").write(cell_src(n, noinline))
    ildir = os.path.join(outdir, "il", tag)
    obj = os.path.join(outdir, "obj", tag + ".obj")
    os.makedirs(os.path.dirname(obj), exist_ok=True)
    env = dict(os.environ, C2RS_REQUIRE_TOOLCHAIN="1")
    rel = lambda q: os.path.relpath(q, ROOT)
    r1 = subprocess.run(
        [C2RS, "capture", rel(cpp), "--keep-il", rel(ildir), "--flags-file", rel(flags)],
        capture_output=True, text=True, cwd=ROOT, env=env)
    r2 = subprocess.run(
        [C2RS, "compile", rel(cpp), "--keep-obj", rel(obj), "--flags-file", rel(flags)],
        capture_output=True, text=True, cwd=ROOT, env=env)
    if r1.returncode or r2.returncode:
        return {"tag": tag, "error": (r1.stderr + r2.stderr)[-300:]}
    gl = open(os.path.join(ildir,
              [x for x in os.listdir(ildir) if x.endswith(".gl")][0]), "rb").read()
    rec = None
    for verdict, r in glrec.walk(gl, glrec.framed_incumbent):
        if verdict == "ok" and r["name"] == CALLEE:
            rec = r
            break
    o = Obj(open(obj, "rb").read())
    ci = o.comdat_of(CALLER)
    kept = ci is not None and any(t == CALLEE for t in o.relocation_targets(ci))
    ki = o.comdat_of(CALLEE)
    return {
        "tag": tag, "n": n, "noinline": noinline, "profile": profile,
        "gl": gl, "gl_len": len(gl),
        "size": rec and rec["size"], "form": rec and rec["form"],
        "attr": rec and rec["attr"], "attr_off": rec and rec["attr_off"],
        "p": rec and rec["p"],
        "verdict": "kept" if kept else "inlined",
        "callee_text": o.sec(ki)["raw"] if ki else None,
    }


def main(argv):
    outdir = os.path.join(HERE, "gridA")
    ns = [int(x) for x in (argv[1].split(",") if len(argv) > 1 else
                           ["3", "5", "7", "9", "10", "12", "16", "20", "30"])]
    profiles = [("O1", os.path.join(ROOT, "work", "w-sizebracket", "flags_O1.txt")),
                ("Ox", os.path.join(ROOT, "work", "w-sizebracket", "flags_Ox.txt"))]
    log = open(os.path.join(HERE, "gridA.jsonl"), "w")
    print(f"{'cell':>16} {'SIZE':>6} {'form':>7} {'attrOff':>8} "
          f"{'diffOffs':>26} {'xor':>5} {'pred':>5} {'plain':>8} {'noinl':>8}")
    for profile, flags in profiles:
        for n in ns:
            a = one(n, False, profile, flags, outdir)
            b = one(n, True, profile, flags, outdir)
            if "error" in a or "error" in b:
                print(f"  ERROR {a.get('tag')}: {a.get('error') or b.get('error')}")
                continue
            ga, gb = a.pop("gl"), b.pop("gl")
            diffs = [i for i in range(min(len(ga), len(gb))) if ga[i] != gb[i]]
            xor = [ga[i] ^ gb[i] for i in diffs]
            pred = a["attr_off"] is not None and diffs == [a["attr_off"]]
            row = dict(a)
            row.update({"diffs": diffs, "xor": xor, "len_eq": len(ga) == len(gb),
                        "diff_at_predicted": pred,
                        "verdict_plain": a["verdict"], "verdict_noinline": b["verdict"],
                        "attr_plain": a["attr"], "attr_noinline": b["attr"]})
            log.write(json.dumps(row) + "\n")
            print(f"{a['tag'][:16]:>16} {str(a['size']):>6} {str(a['form']):>7} "
                  f"{str(a['attr_off']):>8} {str(diffs):>26} "
                  f"{','.join('%02x' % x for x in xor):>5} {str(pred):>5} "
                  f"{a['verdict']:>8} {b['verdict']:>8}")
    log.close()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
