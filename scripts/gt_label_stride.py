#!/usr/bin/env python3
"""gt_label_stride.py — seed-free measurement of c2's compiler-label stride.

`docs/OBJ_GY_SHAPES.md` §3.5 measures the label counter against a *seed* read
out of the IL (`B = u32(.gl[7..11]) + 9`). That works but couples every
measurement to a second unknown, and §3.4 records the exact failure mode: **a
stride and a seed that are both unknown can absorb each other's error**. It also
forces one TU per probe, and the seed is a function of the source text, so two
TUs are never directly comparable.

This script removes the seed entirely by measuring *inside one TU*. Every probe
is compiled as

      <declarations>
      int a0(int a){ return ga(a)+1; }     <- anchor, plain Class-A framed
      <the probe function P>
      int a1(int a){ return ga(a)+2; }     <- anchor
      int a2(int a){ return ga(a)+3; }     <- anchor (self-check)

and, writing `first(F)` for the lowest `$M`/`$T` number in F's symbol group,

      extra(P)  = first(P)  - first(a0) - 5     (slots taken BEFORE P's own $M)
      stride(P) = first(a1) - first(a0) - 5     (total slots P consumes)

with `first(a2) - first(a1) == 5` asserted on every row as the in-TU control.
A leaf probe emits no labels, so only `stride` is defined for it. Anchors are
plain framed functions whose stride is 5 under `/Gy` (measured, and re-asserted
by the a2 control on every single row).

Both quantities are differences *within one object*, so the seed, the mangled
name lengths and the `/Gy` per-function surcharge all cancel — the two
measurement traps recorded in `docs/rungs/` (unreadable seeds from unequal name
lengths; a shell that does not word-split `$flags`) cannot reach this number.

Usage:
    scripts/gt_label_stride.py [--mode '/O1 /GS- /c'] [--keep] [probe ...]
    scripts/gt_label_stride.py --list

Env: C2RS_WIBO / C2RS_COMPILERS as for scripts/gt_capture.sh.
Exit status is 0 if every row's control held (it says nothing about whether a
prediction held — read the table).
"""

import os
import struct
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from gt_dump import Obj  # noqa: E402

ANCHOR_DECL = "int ga(int);"
ANCHORS = [
    "int a0(int a){ return ga(a)+1; }",
    "int a1(int a){ return ga(a)+2; }",
    "int a2(int a){ return ga(a)+3; }",
]

# Each probe: (name, decls, [lead functions], probe body, note).
# `lead` functions sit AHEAD of a0, so anything they introduce (notably
# `_fltused`) is charged to them and not to the probe.
FLOAT_LEAD = ["float ld(float a, float b){ return a*b; }"]

PROBES = [
    # --- controls: the classes whose stride is already measured -------------
    ("plain", "int gp(int);", [],
     "int P(int a){ return gp(a)+1; }",
     "Class A, one NEW callee external"),
    ("plain-3callees", "int gp1(int); int gp2(int); int gp3(int);", [],
     "int P(int a){ return gp1(a)+gp2(a)+gp3(a); }",
     "Class A, THREE new callee externals"),
    ("gpr1", "int gp(int);", [],
     "int P(int a){ return gp(a)+a; }",
     "Class B, 1 saved GPR inline"),
    ("gpr2", "int gp(int);", [],
     "int P(int a,int b){ return gp(a)+gp(b); }",
     "Class B, 2 saved GPRs inline"),
    ("gpr3", "int gp(int);", [],
     "int P(int a,int b,int c){ return gp(a)+gp(b)+gp(c); }",
     "Class C, __savegprlr_29"),
    ("gpr7", "int gp(int);", [],
     "int P(int a,int b,int c,int d,int e,int f,int g){"
     " return gp(a)+gp(b)+gp(c)+gp(d)+gp(e)+gp(f)+gp(g); }",
     "Class C, __savegprlr_25"),
    # --- the FP side --------------------------------------------------------
    ("fp0", "double gd(double);", [],
     "double P(double a){ return gd(a)+gd(a); }",
     "FP-touching framed, 1 saved FPR inline, NO constant, first FP fn"),
    ("fp0-led", "double gd(double);", FLOAT_LEAD,
     "double P(double a){ return gd(a)+gd(a); }",
     "same, but _fltused already charged to the lead"),
    ("fpr3", "double gd(double);", [],
     "double P(double a,double b,double c){ return gd(a)+gd(b)+gd(c); }",
     "Class D, 3 saved FPRs inline, first FP fn"),
    ("fpr3-led", "double gd(double);", FLOAT_LEAD,
     "double P(double a,double b,double c){ return gd(a)+gd(b)+gd(c); }",
     "Class D, 3 saved FPRs inline, _fltused charged to the lead"),
    ("fpr4", "double gd(double);", [],
     "double P(double a,double b,double c,double d){"
     " return gd(a)+gd(b)+gd(c)+gd(d); }",
     "Class E, __savefpr_28, first FP fn"),
    ("fpr4-led", "double gd(double);", FLOAT_LEAD,
     "double P(double a,double b,double c,double d){"
     " return gd(a)+gd(b)+gd(c)+gd(d); }",
     "Class E, __savefpr_28, _fltused charged to the lead   <== THE PROBE"),
    ("fpr5-led", "double gd(double);", FLOAT_LEAD,
     "double P(double a,double b,double c,double d,double e){"
     " return gd(a)+gd(b)+gd(c)+gd(d)+gd(e); }",
     "Class E, __savefpr_27"),
    ("both", "double gd(double); int gp(int);", [],
     "double P(int i1,int i2,int i3,int i4,double d1,double d2,double d3,double d4){"
     " return gp(i1)+gp(i2)+gp(i3)+gp(i4)+gd(d1)+gd(d2)+gd(d3)+gd(d4); }",
     "Class F, BOTH helper pairs, first FP fn   <== THE PROBE"),
    ("both-led", "double gd(double); int gp(int);", FLOAT_LEAD,
     "double P(int i1,int i2,int i3,int i4,double d1,double d2,double d3,double d4){"
     " return gp(i1)+gp(i2)+gp(i3)+gp(i4)+gd(d1)+gd(d2)+gd(d3)+gd(d4); }",
     "Class F, BOTH helper pairs, _fltused led   <== THE PROBE"),
    # --- does a SECOND user of the same helper pay again? -------------------
    ("gpr3-dup", "int gp(int);",
     ["int lg(int a,int b,int c){ return gp(a)+gp(b)+gp(c); }"],
     "int P(int a,int b,int c){ return gp(a)+gp(b)+gp(c); }",
     "Class C reusing the SAME __savegprlr_29 an earlier function introduced"),
    ("gpr3-dup-wide", "int gp(int);",
     ["int lg(int a,int b,int c){ return gp(a)+gp(b)+gp(c); }"],
     "int P(int a,int b,int c,int d){ return gp(a)+gp(b)+gp(c)+gp(d); }",
     "Class C needing a DIFFERENT width (__savegprlr_28) from the lead's _29"),
    ("gpr3-const-led", "int gp(int); float gf(float);", FLOAT_LEAD,
     "float P(int a,int b,int c){ return (float)(gp(a)+gp(b)+gp(c))*2.5f; }",
     "Class C helper AND one new pooled constant"),
    # --- pooled .rdata constants -------------------------------------------
    ("const1-led", "float gf(float);", FLOAT_LEAD,
     "float P(float a){ return gf(a)*2.5f; }",
     "framed, ONE pooled .rdata constant, _fltused led"),
    ("const2-led", "float gf(float);", FLOAT_LEAD,
     "float P(float a){ return gf(a)*2.5f+3.5f; }",
     "framed, TWO pooled .rdata constants, _fltused led"),
    ("const1-dup-led", "float gf(float);",
     ["float ld(float a, float b){ return a*b; }",
      "float ldc(float a){ return a*2.5f; }"],
     "float P(float a){ return gf(a)*2.5f; }",
     "framed, reuses a constant an EARLIER function already pooled"),
    # --- leaves, for the stride column only ---------------------------------
    ("leaf-int", "", [], "int P(int a){ return a+1; }", "int leaf"),
    ("leaf-tail", "int gp(int);", [], "int P(int a){ return gp(a); }",
     "tail-call leaf, one NEW callee external"),
    ("leaf-float", "", [], "float P(float a,float b){ return a*b; }",
     "float leaf, first FP fn"),
    ("leaf-float-led", "", FLOAT_LEAD, "float P(float a,float b){ return a*b; }",
     "float leaf, _fltused charged to the lead"),
    ("leaf-float-c1-led", "", FLOAT_LEAD, "float P(float a){ return a*2.5f; }",
     "float leaf, ONE pooled constant, _fltused led"),
    ("leaf-float-c2-led", "", FLOAT_LEAD,
     "float P(float a){ return a*2.5f+3.5f; }",
     "float leaf, TWO pooled constants, _fltused led"),
    ("leaf-double-led", "", FLOAT_LEAD, "double P(double a){ return a; }",
     "double leaf, _fltused led"),
    # --- the adversarial block: classes that mint NOTHING extra yet whose
    #     stride OBJ_GY_SHAPES.md §3.6/§3.6a already measures above 1. If
    #     `stride == minted` is right, these must all come back 1.
    ("leaf-cmp-eq", "", [], "int P(int a){ return a==5; }",
     "comparison leaf, ==5 (§3.6a says 1)"),
    ("leaf-cmp-lt0", "", [], "int P(int a){ return a<0; }",
     "comparison leaf, <0 (§3.6a says 1)"),
    ("leaf-cmp-lt5", "", [], "int P(int a){ return a<5; }",
     "comparison leaf, signed <5 (§3.6a says 3)  <== REFUTER"),
    ("leaf-cmp-ge5", "", [], "int P(int a){ return a>=5; }",
     "comparison leaf, signed >=5 (§3.6a says 3)  <== REFUTER"),
    ("leaf-cmp-ult5", "", [], "int P(unsigned a){ return a<5u; }",
     "comparison leaf, unsigned <5 (§3.6a says 1)"),
    ("leaf-cmp-gt0", "", [], "int P(int a){ return a>0; }",
     "comparison leaf, signed >0 (§3.6a says 3)  <== REFUTER"),
    ("leaf-static", "", [], "static int Punused(int a){ return a+1; }\n"
     "int P(int a){ return Punused(a)+1; }",
     "an internal (static) function ahead of P: does it cost 1?"),
    ("leaf-branch", "", [], "int P(int a){ if (a) return 1; return 2; }",
     "control flow — a branch needs no symbol"),
    ("leaf-loop", "", [], "int P(int a){ int s=0; for(int i=0;i<a;i++) s+=i; return s; }",
     "a loop — needs no symbol"),
    ("leaf-switch", "", [],
     "int P(int a){ switch(a){case 1:return 3;case 2:return 5;case 7:return 9;"
     "case 8:return 11;case 9:return 13;default:return 0;} }",
     "a switch — a jump table would be a new section + symbol"),
    ("leaf-string", "", [], "const char* P(){ return \"hello\"; }",
     "a string literal — mints a `??_C@` symbol and a .rdata section"),
    ("leaf-string2", "", [],
     "const char* P(int a){ return a ? \"hello\" : \"world\"; }",
     "TWO string literals, two .rdata COMDATs"),
    ("leaf-cmp2", "", [], "int P(int a,int b){ return (a<5) + (b<7); }",
     "TWO signed comparisons in one leaf"),
    ("leaf-cmp3", "", [], "int P(int a,int b,int c){ return (a<5)+(b<7)+(c<9); }",
     "THREE signed comparisons in one leaf"),
    ("leaf-loop2", "", [],
     "int P(int a){ int s=0; for(int i=0;i<a;i++) for(int j=0;j<a;j++) s+=j; return s; }",
     "a nested loop"),
    ("framed-cmp", "int gp(int);", [],
     "int P(int a){ return gp(a) + (a<5); }",
     "framed AND a signed comparison — is the +2 pre-allocated like the rest?"),
    ("framed-loop", "int gp(int);", [],
     "int P(int a){ int s=0; for(int i=0;i<a;i++) s+=gp(i); return s; }",
     "framed AND a loop"),
    # --- separated: the internal-linkage function, on its own -------------
    ("static-leaf", "int gs(int);", ["static int lst(int a){ return gs(a)+a; }"],
     "int P(int a){ return lst(a)+lst(a+1); }",
     "P is framed; the LEAD is a static (internal-linkage) function"),
]


def build_src(decls, leads, probe):
    parts = [ANCHOR_DECL]
    if decls:
        parts.append(decls)
    parts += leads
    parts.append(ANCHORS[0])
    parts.append(probe)
    parts.append(ANCHORS[1])
    parts.append(ANCHORS[2])
    return "\n".join(parts) + "\n"


def capture(src, mode, workdir, tag):
    cpp = os.path.join(workdir, "%s.cpp" % tag)
    open(cpp, "w").write(src)
    env = dict(os.environ)
    r = subprocess.run(
        [os.path.join(HERE, "gt_capture.sh"), cpp] + mode.split(),
        capture_output=True, text=True, env=env,
    )
    path = r.stdout.strip()
    if not path or not os.path.exists(path):
        sys.stderr.write(r.stderr)
        return None
    return Obj(open(path, "rb").read())


# The symbols c2 *mints* itself, as opposed to the ones the IL hands it (the
# function's own name and its callees). Everything else in a function's group —
# section symbols, `$M`/`$T` labels, `__real@` pool entries — is minted too.
SYNTH_EXTERNALS = ("_fltused", "__savegprlr_", "__restgprlr_",
                   "__savefpr_", "__restfpr_", "__real@", "__xmm@")


def minted(group):
    """Count the symbol-table entries in one function's group that c2 minted.

    The claim under test: **stride == this number**. The group's entries are
    the function symbol (from the IL — not counted), the callee externals it
    introduced (from the IL — not counted), and everything else (counted).
    Aux records are not symbol-table entries for this purpose.
    """
    n = 0
    for kind, name in group["entries"]:
        if kind == "fn":
            continue
        if kind == "undef" and not name.startswith(SYNTH_EXTERNALS):
            continue          # a callee the IL named
        n += 1
    return n


def groups(o):
    """Walk the symbol table in order and split it into per-function groups.

    A group opens at a DEFINED function symbol (EXTERNAL, type 0x0020, sec>0)
    and absorbs every following symbol until the next one: its labels, the
    externals it introduced, and its .pdata/.rdata section symbols.
    """
    out = []
    cur = None
    for s in o.symbols:
        # A defined function symbol — EXTERNAL normally, STATIC for an
        # internal-linkage function, which is why the storage class is not the
        # discriminator here.
        defined_fn = s["sc"] in (2, 3) and s["type"] == 0x0020 and s["sec"] > 0
        if defined_fn:
            cur = {"name": s["name"], "sec": s["sec"], "labels": [], "syms": [],
                   "sections": [], "entries": [("fn", s["name"])]}
            out.append(cur)
            continue
        if cur is None:
            continue
        n = s["name"]
        if (n.startswith("$M") or n.startswith("$T")) and n[2:].isdigit():
            cur["labels"].append(int(n[2:]))
            cur["entries"].append(("label", n))
        elif s["sc"] == 3 and s["sec"] > 0 and n.startswith("."):
            cur["sections"].append(n)
            cur["entries"].append(("secsym", n))
        elif s["sec"] == 0:
            cur["entries"].append(("undef", n))
        else:
            cur["entries"].append(("other", n))
        cur["syms"].append(n)
    return out


def prologue_class(o, sec_index):
    """(nGPRsaved, nFPRsaved, helpers) read out of one function's .text."""
    sec = o.sections[sec_index - 1]
    d = o.raw(sec)
    rels = {va: sym for va, sym, ty in o.relocs(sec)}
    ngpr = nfpr = 0
    helpers = []
    for i in range(0, len(d), 4):
        w = struct.unpack_from(">I", d, i)[0]
        op = w >> 26
        rt = (w >> 21) & 31
        ra = (w >> 16) & 31
        # Saves live strictly BEFORE the `stwu r1,-F(r1)`; anything after it is
        # a spill or an outgoing-argument store and must not be counted.
        if op == 37 and rt == 1 and ra == 1:
            break
        if op == 62 and ra == 1 and (w & 3) == 0:
            ngpr += 1
        if op == 54 and ra == 1:
            nfpr += 1
        if i in rels:
            nm = o.sym_by_index(rels[i])["name"]
            if nm.startswith("__savegprlr_"):
                ngpr += 32 - int(nm.rsplit("_", 1)[1])
                helpers.append("gprlr")
            elif nm.startswith("__savefpr_"):
                nfpr += 32 - int(nm.rsplit("_", 1)[1])
                helpers.append("fpr")
    return ngpr, nfpr, helpers


def run(name, decls, leads, probe, note, mode, workdir):
    src = build_src(decls, leads, probe)
    o = capture(src, mode, workdir, name.replace("-", "_"))
    if o is None:
        return None
    gs = groups(o)
    by = {g["name"]: g for g in gs}
    def find(sfx):
        for g in gs:
            if g["name"].startswith("?" + sfx + "@@") or g["name"] == sfx:
                return g
        return None
    a0, a1, a2, P = find("a0"), find("a1"), find("a2"), find("P")
    if not (a0 and a1 and a2 and P):
        return {"name": name, "error": "missing group (%s)" % [g["name"] for g in gs]}
    f = lambda g: min(g["labels"]) if g["labels"] else None
    # The anchor stride is measured IN THIS OBJ rather than assumed, so the
    # same script works packed (where it is 4) and under /Gy (where it is 5)
    # without a mode-dependent constant anywhere. a1->a2 is the calibration and
    # a0->a1 minus it is the probe's excess; two identical anchors either side
    # of the probe are what makes the seed cancel.
    base = f(a2) - f(a1)
    row = {
        "name": name, "note": note,
        "first_a0": f(a0), "first_P": f(P), "first_a1": f(a1), "first_a2": f(a2),
        "control": base,
        "stride": f(a1) - f(a0) - base,
        "extra": (f(P) - f(a0) - base) if f(P) is not None else None,
        "framed": f(P) is not None,
        "syms": [s for s in P["syms"] if not s.startswith("$")],
        "sections": P["sections"],
        "minted": minted(P),
    }
    row["prologue"] = prologue_class(o, P["sec"])
    return row


def main(argv):
    if "--list" in argv:
        for p in PROBES:
            print("%-20s %s" % (p[0], p[4]))
        return 0
    mode = "/O1 /GS- /c"
    if "--mode" in argv:
        i = argv.index("--mode")
        mode = argv[i + 1]
        del argv[i:i + 2]
    want = [a for a in argv[1:] if not a.startswith("--")]
    probes = [p for p in PROBES if not want or p[0] in want]

    print("mode: %s   (anchors: 3x plain Class-A framed; control = the anchor base, 5 /Gy, 4 packed)" % mode)
    print("  'minted' = symbol-table entries in P's group that c2 minted itself")
    print("             (everything but the function symbol and its IL-named callees).")
    print("  CLAIM UNDER TEST: stride == minted.")
    print()
    print("%-20s %6s %6s %7s %8s   %s"
          % ("probe", "extra", "stride", "minted", "control", "prologue / introduced"))
    bad = 0
    refuted = 0
    wd = tempfile.mkdtemp(prefix="gtlbl")
    for p in probes:
        row = run(p[0], p[1], p[2], p[3], p[4], mode, wd)
        if row is None:
            print("%-20s  CAPTURE FAILED" % p[0]); bad += 1; continue
        if "error" in row:
            print("%-20s  %s" % (p[0], row["error"])); bad += 1; continue
        ngpr, nfpr, helpers = row["prologue"]
        if row["control"] not in (4, 5):
            bad += 1
        mark = "" if row["stride"] == row["minted"] else "   <== REFUTES stride==minted"
        if row["stride"] != row["minted"]:
            refuted += 1
        print("%-20s %6s %6d %7d %8d   nGPR=%d nFPR=%d %s | %s %s%s" % (
            p[0],
            "-" if row["extra"] is None else row["extra"],
            row["stride"], row["minted"], row["control"],
            ngpr, nfpr, ",".join(helpers) or "-",
            " ".join(row["sections"]) or "-",
            " ".join(row["syms"]) or "-",
            mark,
        ))
        print("%-20s   %s" % ("", p[4]))
    print("controls failed: %d   stride!=minted: %d" % (bad, refuted))
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
