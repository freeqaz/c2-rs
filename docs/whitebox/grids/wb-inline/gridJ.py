#!/usr/bin/env python3
"""GRID-J — the ANCHOR grid: what separates ?shuffle2 from ?shuffle3?

Measured, from the real dc3 `keygen_xbox.cpp` at the workload's own flags:

    ?shuffle1  104 B  CALLED      ?shuffle4   84 B  CALLED
    ?shuffle2   60 B  INLINED     ?shuffle5   88 B  CALLED
    ?shuffle3   84 B  CALLED      ?shuffle6   88 B  CALLED

All six are `void f(char*)`, EXTERNAL, non-`inline`, leaf, and every one is a
single counted `bdnz` loop over a byte array. `INLINE-P` reads
`index = s - 48*[leaf] <= 64`, i.e. `s <= 112`, so it predicts **all six
inlined** and is wrong on five. GRID-I measured the straight-line EXTERNAL
boundary at `/O1 /GS- /c` as `(100, 116]`, which agrees with `INLINE-P` — so
the anchor's callees are outside the class GRID-I swept.

FROZEN PREDICTIONS, written before the first cl.exe of this grid:

  R1-INCUMBENT   the boundary is at s = 112 in BOTH families and BOTH flag
                 sets; the loop changes nothing, because `INLINE-P` has no
                 loop term anywhere.
  R6-LOOP        the boundary in emitted BYTES is STRICTLY LOWER for a loop
                 body than for a straight-line body at the same flags, and
                 the loop boundary brackets the anchor's (60, 84].
  R7-FLAGS       the workload's `/Oi /EHsc /GR` move the boundary relative to
                 `/O1 /GS- /c`, and the loop is irrelevant.

R6 and R7 are separated by the straight-line ladder at workload flags: R7
predicts it moves off (100,116], R6 predicts it does not.
"""
import json
import os
import subprocess
import sys

REPO = "/home/free/code/milohax/c2-rs"
WIBO = "/home/free/code/milohax/wibo/build/release/wibo"

WORKLOAD = ("/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc").split()
PLAIN = "/O1 /GS- /c".split()

MODES = {"work": WORKLOAD, "plain": PLAIN}


def loop_body(j):
    """A counted loop over a byte array — the anchor's own shape."""
    stmts = "".join("    p[i + %d] = (char)(p[i + %d] + %d);\n" % (j2, j2 + 1, j2)
                    for j2 in range(j))
    return ("void cg(char *p) {\n  for (int i = 0; i < 8; i++) {\n"
            "%s  }\n}\n" % stmts)


def line_body(j):
    stmts = "".join("  p[%d] = (char)(p[%d] + %d);\n" % (i, i + 1, i)
                    for i in range(j))
    return "void cg(char *p) {\n%s}\n" % stmts


CALLER = "void cf(char *p) {\n  cg(p);\n}\n"

J = list(range(1, 15))


def cells():
    out = []
    for mode in MODES:
        for shape, fn in (("loop", loop_body), ("line", line_body)):
            for j in J:
                out.append(dict(id="%s_%s_j%d" % (mode, shape, j), mode=mode,
                                shape=shape, j=j,
                                src=fn(j) + CALLER,
                                callee_only=fn(j)))
    return out


def cap(src, mode, tag, d):
    cpp = os.path.abspath(os.path.join(d, tag + ".cpp"))
    open(cpp, "w").write(src)
    env = dict(os.environ)
    env["C2RS_WIBO"] = WIBO
    env["C2RS_COMPILERS"] = os.path.join(REPO, "compilers")
    obj = cpp[:-4] + ".obj"
    zp = lambda p: "z:" + p.replace("/", "\\")
    r = subprocess.run(
        [WIBO, os.path.join(REPO, "compilers/X360/16.00.11886.00/cl.exe")]
        + MODES[mode] + ["/Fo" + zp(obj), zp(os.path.abspath(cpp))],
        capture_output=True, text=True, env=env)
    if not os.path.exists(obj):
        sys.stderr.write(r.stdout + r.stderr)
        return None
    data = open(obj, "rb").read()
    os.remove(obj)
    return data


def main():
    d = sys.argv[1]
    os.makedirs(d, exist_ok=True)
    sys.path.insert(0, os.path.join(REPO, "scripts"))
    from gt_dump import Obj
    res = []
    for c in cells():
        data = cap(c["callee_only"], c["mode"], c["id"] + "_only", d)
        s = None
        if data:
            o = Obj(data)
            for sy in o.symbols:
                if sy["sec"] > 0 and sy["name"].startswith("?cg@"):
                    s = o.sections[sy["sec"] - 1]["rawsize"]
        data = cap(c["src"], c["mode"], c["id"], d)
        v = "absent"
        if data:
            o = Obj(data)
            sec = None
            for sy in o.symbols:
                if sy["sec"] > 0 and sy["name"].startswith("?cf@"):
                    sec = o.sections[sy["sec"] - 1]
            if sec is not None:
                names = set()
                for (_, si, _t) in o.relocs(sec):
                    sy = o.sym_by_index(si)
                    if sy:
                        names.add(sy["name"])
                v = ("called" if any(n.startswith("?cg@") for n in names)
                     else "inlined")
        res.append(dict(id=c["id"], mode=c["mode"], shape=c["shape"],
                        j=c["j"], s=s, verdict=v))
        print("  %-16s s=%-5s %s" % (c["id"], s, v))
    json.dump(res, open(os.path.join(d, "gridJ.json"), "w"), indent=1)

    print("\nBOUNDARIES (last inlined s -> first called s):")
    for mode in MODES:
        for shape in ("loop", "line"):
            rows = [r for r in res if r["mode"] == mode and r["shape"] == shape
                    and r["s"]]
            rows.sort(key=lambda r: r["s"])
            lo = max([r["s"] for r in rows if r["verdict"] == "inlined"] or [0])
            hi = min([r["s"] for r in rows if r["verdict"] == "called"]
                     or [10 ** 9])
            print("  %-6s %-5s   (%s, %s]" % (mode, shape, lo, hi))


if __name__ == "__main__":
    main()
