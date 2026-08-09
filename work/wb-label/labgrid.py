#!/usr/bin/env python3
"""labgrid.py — wb-label's obj-check grid.

`scripts/gt_label_stride.py`'s construction, with the probe in the MIDDLE
(`w-ifn`'s banner: a stride measurement is blind at both ends of a function
list), one TU per probe so nothing charged once per TU can land on a neighbour:

    int ga(int);
    <decls>
    int a0(int a){ return ga(a)+1; }      anchor
    <the probe P>
    int a1(int a){ return ga(a)+2; }      anchor
    int a2(int a){ return ga(a)+3; }      anchor / control

    base      = first(a2) - first(a1)          measured IN THIS OBJ
    stride(P) = first(a1) - first(a0) - base
    extra(P)  = first(P)  - first(a0) - base   (framed probes only)

`minted` is read for every row — `LABEL_COUNTER.md` §4's "read the minted
column" box: a probe that obliges a helper pair pays a MINTED surcharge as well
as a control-flow one, and a stride difference charges the first one twice.

Everything lands under work/wb-label/out/ (gitignored). std-lib only.
"""

import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(ROOT, "scripts"))
from gt_dump import Obj  # noqa: E402
from gt_label_stride import groups, minted  # noqa: E402

WORK = os.path.join(ROOT, "work", "wb-label", "out")
CAPTURE = os.path.join(ROOT, "scripts", "gt_capture.sh")

ANCHOR_DECL = "int ga(int);"
ANCHORS = [
    "int a0(int a){ return ga(a)+1; }",
    "int a1(int a){ return ga(a)+2; }",
    "int a2(int a){ return ga(a)+3; }",
]

SW_ARMS = [3, 9, 14, 21, 30, 44, 57, 68, 79, 91, 104, 120]


def sw_body(inner="s += %d;"):
    arms = "".join(" case %d: %s break;" % (v, inner % v) for v in SW_ARMS)
    return arms


# (name, decls, probe-source, modes)
PROBES = [
    # ---- controls, so the instrument is re-proved on every run -------------
    ("ctl-plain", "int gp(int);",
     "int P(int a){ return gp(a)+1; }", None),
    ("ctl-leaf", "",
     "int P(int a){ return a+1; }", None),
    ("ctl-for", "int gp(int);",
     "int P(int a){ int s=0; for(int i=0;i<a;i++) s+=gp(i); return s; }", None),
    # ---- X1..X6, frozen in WB_LABEL_PREREG_R2.md §3 -----------------------
    ("X1-switch-table", "int gp(int);",
     "int P(int a){ int s=gp(a); switch(a){%s default: s=-1; break; } return s; }"
     % sw_body(), None),
    ("X2-for-if", "int gp(int);",
     "int P(int a){ int s=gp(a); for(int i=0;i<a;i++){ if(i&1) s+=i; } return s; }", None),
    ("X3-while-ret", "int gp(int);",
     "int P(int a){ int s=gp(a); while(a>0){ if(s>100) return s; s+=a; a--; } return s; }",
     None),
    ("X4-try-2catch", "int gp(int);",
     "int P(int a){ int s=0; try { s=gp(a); } catch(int e){ s=e+1; } catch(...){ s=-1; } return s; }",
     ["/O1 /GS- /EHsc /c", "/Ox /GS- /EHsc /c"]),
    ("X5-switch-in-for", "int gp(int);",
     "int P(int a){ int s=gp(a); for(int i=0;i<a;i++){ switch(i){%s default: s=-1; break; } } return s; }"
     % sw_body(), None),
    ("X6-unroll", "int gp(int);",
     "int P(int a){ int s=gp(a); for(int i=0;i<4;i++) s+=a*i; return s; }", None),
    # ---- the PRIMITIVES, measured after X1..X6 were graded, so that the
    #      additivity rule R3 holds out has primitive values to compose ------
    ("p-none", "int gp(int);",
     "int P(int a){ return gp(a)+1; }", None),
    ("p-if", "int gp(int);",
     "int P(int a){ int s=gp(a); if(a>3) s+=7; return s; }", None),
    ("p-ifelse", "int gp(int);",
     "int P(int a){ int s=gp(a); if(a>3) s+=7; else s-=7; return s; }", None),
    ("p-for", "int gp(int);",
     "int P(int a){ int s=gp(a); for(int i=0;i<a;i++) s+=i; return s; }", None),
    ("p-while", "int gp(int);",
     "int P(int a){ int s=gp(a); while(a>0){ s+=a; a--; } return s; }", None),
    ("p-dowhile", "int gp(int);",
     "int P(int a){ int s=gp(a); do { s+=a; a--; } while(a>0); return s; }", None),
    ("p-switch", "int gp(int);",
     "int P(int a){ int s=gp(a); switch(a){%s default: s=-1; break; } return s; }"
     % sw_body(), None),
    # ---- R3 HELD-OUT compositions (predictions frozen in
    #      WB_LABEL_PREREG_R3.md before these were compiled) -----------------
    ("H1-if-in-while", "int gp(int);",
     "int P(int a){ int s=gp(a); while(a>0){ if(a&1) s+=7; a--; } return s; }", None),
    ("H2-ifelse-in-for", "int gp(int);",
     "int P(int a){ int s=gp(a); for(int i=0;i<a;i++){ if(i&1) s+=7; else s-=7; } return s; }",
     None),
    ("H3-two-ifs", "int gp(int);",
     "int P(int a){ int s=gp(a); if(a>3) s+=7; if(a>9) s+=11; return s; }", None),
    ("H4-switch-in-while", "int gp(int);",
     "int P(int a){ int s=gp(a); while(a>0){ switch(a){%s default: s=-1; break; } a--; } return s; }"
     % sw_body(), None),
    ("H5-for-in-for", "int gp(int);",
     "int P(int a){ int s=gp(a); for(int i=0;i<a;i++) for(int j=0;j<i;j++) s+=j; return s; }",
     None),
    ("H6-dowhile-in-if", "int gp(int);",
     "int P(int a){ int s=gp(a); if(a>3){ do { s+=a; a--; } while(a>0); } return s; }", None),
]

MODES = ["/O1 /GS- /c", "/Ox /GS- /c"]


def build(decls, probe):
    parts = [ANCHOR_DECL]
    if decls:
        parts.append(decls)
    parts.append(ANCHORS[0])
    parts.append(probe)
    parts.append(ANCHORS[1])
    parts.append(ANCHORS[2])
    return "\n".join(parts) + "\n"


def capture(src, mode, tag):
    os.makedirs(WORK, exist_ok=True)
    cpp = os.path.join(WORK, "%s.cpp" % tag)
    open(cpp, "w").write(src)
    r = subprocess.run([CAPTURE, cpp] + mode.split(),
                       capture_output=True, text=True)
    path = r.stdout.strip()
    if not path or not os.path.exists(path):
        sys.stderr.write(r.stderr[-2000:])
        return None
    return Obj(open(path, "rb").read())


def firsts(o):
    """{suffix: (first-label-number, minted)} for a0/P/a1/a2."""
    out = {}
    for g in groups(o):
        nm = g["name"]
        labels = [n for (k, n) in g["entries"] if k == "label"]
        nums = []
        for n in labels:
            d = "".join(c for c in n if c.isdigit())
            if d:
                nums.append(int(d))
        for sfx in ("a0", "a1", "a2", "P"):
            if ("?%s@@" % sfx) in nm:
                out[sfx] = (min(nums) if nums else None, minted(g))
    return out


def run(name, decls, probe, mode):
    tag = "%s_%s" % (name, mode.replace("/", "").replace(" ", ""))
    o = capture(build(decls, probe), mode, tag)
    if o is None:
        return None
    f = firsts(o)
    if not all(k in f for k in ("a0", "a1", "a2")):
        return {"err": "anchors missing: %s" % sorted(f)}
    base = f["a2"][0] - f["a1"][0]
    stride = f["a1"][0] - f["a0"][0] - base
    extra = (f["P"][0] - f["a0"][0] - base) if f.get("P", (None,))[0] else None
    return {
        "base": base, "stride": stride, "extra": extra,
        "minted": f.get("P", (None, None))[1],
        "a0": f["a0"][0], "a1": f["a1"][0], "a2": f["a2"][0],
        "P": f.get("P", (None,))[0],
    }


def main(argv):
    want = [a for a in argv if not a.startswith("-")]
    print("%-18s %-14s %5s %7s %6s %7s  %s" %
          ("probe", "mode", "base", "stride", "extra", "minted", "raw a0/P/a1/a2"))
    for (name, decls, probe, modes) in PROBES:
        if want and name not in want:
            continue
        for mode in (modes or MODES):
            r = run(name, decls, probe, mode)
            if r is None:
                print("%-18s %-14s  CAPTURE FAILED" % (name, mode))
                continue
            if "err" in r:
                print("%-18s %-14s  %s" % (name, mode, r["err"]))
                continue
            flag = "" if r["base"] in (4, 5) else "   <== CONTROL BROKEN"
            print("%-18s %-14s %5d %7d %6s %7s  %s/%s/%s/%s%s" %
                  (name, mode, r["base"], r["stride"],
                   r["extra"] if r["extra"] is not None else "-",
                   r["minted"] if r["minted"] is not None else "-",
                   r["a0"], r["P"], r["a1"], r["a2"], flag))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
