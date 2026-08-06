#!/usr/bin/env python3
"""grid.py — hand probes for INLINE-P, at the WORKLOAD's own flags.

Lane w-inline measurement tooling. **Read-only with respect to `crates/`.**

WHY A NEW GENERATOR AND NOT `scripts/gt_inline_decline.py`
----------------------------------------------------------
That script is the incumbent's own instrument and it is not being replaced: its
449 rungs are what `INLINE-P` is transcribed from. What it cannot do is the two
things this lane needs. It captures at `/O1 /GS- /c` and `/Ox /GS- /c`, and its
fourteen ladders are all `static int f(int)`-shaped. This grid runs at
`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc` — read from
`work/dc3-workload/flags.txt`, never transcribed — and its families are the
spellings the workload actually contains.

WHAT A CELL IS, AND HOW IT IS GRADED
------------------------------------
One `.cpp` per (family, k, N): a callee `c1` grown by `k` rungs, called from `P`
at `N` sites. Graded from OBJ BYTES (#843), by §6.15's own observable:

    a REL24 from P to c1 survives  <=>  c2 DECLINED that site.

`s` is **measured**, never predicted: the callee's own `.text` COMDAT length is
read out of the same obj. So a family whose rung is a different number of bytes
still lands on the same index axis, which is the whole point of indexing on `s`.

    #869: frame words are printed, never asserted.
    #644: no positional readers -- every field is walked.
    w-refbind: no absolute register number is anchored anywhere in here.

Usage:
    grid.py --out DIR --families a,b,c --kmax N [--n 1,4] [--emit-only]
    grid.py --list
"""

import hashlib
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from scan_obj import (  # noqa: E402
    UNBOUNDED, IMAGE_SYM_CLASS_STATIC, SELECT,
    read_obj, annotate_params, is_leaf, n_max, sched_index,
)

REPO = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))

# A rung: one statement that emits a fixed, small number of instructions and
# keeps exactly one value live, so `s` walks the index axis in even steps and the
# register pressure does not drift (§6.16's finding: pressure is an axis).
RUNG = "  v = v * {m} + {m};\n"

PRELUDE = """
extern int gs(int);
extern int gsink;
"""


def rungs(k, start=3):
    return "".join(RUNG.format(m=start + 2 * i) for i in range(k))


# Every family returns (callee_decl_and_def, call_expr_template, extra).
# `{i}` in the call template is the site number, so no two sites CSE.
FAMILIES = {}


def fam(name):
    def deco(f):
        FAMILIES[name] = f
        return f
    return deco


@fam("ext-plain")
def _ext_plain(k):
    """A plain external free function — the CONTROL. §6.17.4 step at s 64/68."""
    return (f"int c1(int a) {{ int v = a; {rungs(k)} return v; }}\n",
            "s += c1(s + {i});", "")


@fam("ext-inline")
def _ext_inline(k):
    """The same body with `inline`. §6.17.5 buys 8 bytes, in BOTH classes."""
    return (f"inline int c1(int a) {{ int v = a; {rungs(k)} return v; }}\n",
            "s += c1(s + {i});", "")


@fam("member-inclass")
def _member_inclass(k):
    """A member defined IN class — implicitly `inline`, and `this` is param 0."""
    return (f"struct MB {{ int m; int c1(int a) {{ int v = a + m; {rungs(k)} return v; }} }};\n",
            "s += ob.c1(s + {i});", "static MB ob;\n")


@fam("member-outclass")
def _member_outclass(k):
    """The same member defined OUT of class and WITHOUT `inline` — §6.17.3 says
    this behaves like a plain external, i.e. 8 bytes tighter than the row above."""
    return ("struct MB { int m; int c1(int a); };\n"
            f"int MB::c1(int a) {{ int v = a + m; {rungs(k)} return v; }}\n",
            "s += ob.c1(s + {i});", "static MB ob;\n")


@fam("static-plain")
def _static_plain(k):
    """Internal linkage — SCHEDULE D's graduated middle lives here and nowhere
    else (§6.17.4: "linkage decides whether the graduated part exists for you")."""
    return (f"static int c1(int a) {{ int v = a; {rungs(k)} return v; }}\n",
            "s += c1(s + {i});", "")


@fam("tmpl")
def _tmpl(k):
    """A function template instantiation — EXTERNAL + SELECT_ANY, and a shape
    §6.15-§6.20 has no row for anywhere (§6.15.7's own closing paragraph)."""
    return (f"template <class T> T c1(T a) {{ T v = a; {rungs(k)} return v; }}\n"
            "template int c1<int>(int);\n",
            "s += c1<int>(s + {i});", "")


@fam("varargs")
def _varargs(k):
    """§6.18.5 — the only genuine categorical refusal, in BOTH linkage classes."""
    return (f"int c1(int a, ...) {{ int v = a; {rungs(k)} return v; }}\n",
            "s += c1(s + {i}, 1);", "")


@fam("virt-ptr")
def _virt_ptr(k):
    """A virtual member through a POINTER — §6.18.4: there is no call to
    decline, because the site does not name a callee."""
    return ("struct VB { int m; virtual int c1(int a); };\n"
            f"int VB::c1(int a) {{ int v = a + m; {rungs(k)} return v; }}\n",
            "s += pv->c1(s + {i});", "static VB vb; static VB *pv = &vb;\n")


@fam("site-if")
def _site_if(k):
    """Each site in its own basic block — §6.19.9 moves the 1->0 CEILING by 96
    bytes and NOTHING else. Below the ceiling this must agree with `ext-plain`."""
    return (f"int c1(int a) {{ int v = a; {rungs(k)} return v; }}\n",
            "if (s & {mask}) {{ s += c1(s + {i}); }}", "")


@fam("site-eh")
def _site_eh(k):
    """The site sits where a destructible object is live — the `/EHsc` shape the
    incumbent's `/O1 /GS- /c` captures cannot contain at all."""
    return ("struct D { int x; D(); ~D(); };\n"
            f"int c1(int a) {{ int v = a; {rungs(k)} return v; }}\n",
            "{{ D d; s += c1(s + {i}); gsink += d.x; }}", "")


@fam("two-level")
def _two_level(k):
    """c1 calls c0, and c0 is ALSO defined here — §6.18.10's "every call inside a
    callee in this section is to an undefined external", violated on purpose."""
    return ("static int c0(int a) { return a * 7 + 1; }\n"
            f"int c1(int a) {{ int v = c0(a); {rungs(k)} return v; }}\n",
            "s += c1(s + {i});", "")


@fam("recurse")
def _recurse(k):
    """Direct recursion — §6.19.5/§6.19.10: "the one call-graph shape that has
    never appeared on either side of the pair in the whole of §6.15-§6.19"."""
    return ("int c1(int a);\n"
            f"int c1(int a) {{ int v = a; {rungs(k)} return a > 0 ? c1(a - 1) + v : v; }}\n",
            "s += c1(s + {i});", "")


def source(family, k, n):
    decl, call, extra = FAMILIES[family](k)
    sites = "\n  ".join(
        call.format(i=i + 1, mask=1 << (i + 1)) for i in range(n)
    )
    return (PRELUDE + decl + extra +
            f"int P(int a) {{\n  int s = gs(a) + a;\n  {sites}\n  return s;\n}}\n")


def compile_cell(src_text, workdir, tag):
    cpp = os.path.join(workdir, tag + ".cpp")
    obj = os.path.join(workdir, tag + ".obj")
    open(cpp, "w").write(src_text)
    r = subprocess.run([os.path.join(REPO, "work", "w-fnbyte", "probe.sh"), cpp, obj],
                       capture_output=True, text=True)
    if not os.path.exists(obj) or os.path.getsize(obj) == 0:
        return None, (r.stdout + r.stderr).strip()
    return obj, None


def grade_cell(obj, n):
    """(measured s, index, N_max, predicted, observed) for the cell's `c1`.

    The callee is found by *demangled* name ending in `c1`, never by position in
    the section table (#644): the section order is not the definition order.
    """
    fns = read_obj(obj)
    annotate_params(fns)
    cands = [f for name, f in fns.items()
             if (f.demangled or name).split("(")[0].rstrip().endswith("c1")
             or "c1<" in (f.demangled or name)]
    if len(cands) != 1:
        return None, f"callee not unique: {[ (f.demangled or f.name) for f in cands]}"
    c1 = cands[0]
    callers = [f for f in fns.values() if f is not c1 and c1.name in f.rel24]
    survived = len(callers) > 0
    leaf = is_leaf(c1, fns)
    nm = n_max(c1, leaf, drop_leaf_term=True)     # FROZEN by addendum 2
    predicted = "INLINED-ALL" if nm >= n else "DECLINED"
    observed = "DECLINED" if survived else "INLINED-ALL"
    frame_words = sum(1 for w in c1.words if (w >> 16) == 0x9421)   # printed, #869
    return {
        "callee": c1.name,
        "s": c1.size,
        "linkage": "STATIC" if c1.sc == IMAGE_SYM_CLASS_STATIC else "EXTERNAL",
        "selection": SELECT.get(c1.selection, str(c1.selection)),
        "nparams": c1.nparams if c1.parse_ok else -1,
        "varargs": int(bool(c1.varargs)),
        "leaf": int(leaf),
        "index": sched_index(c1, False),
        "nmax": "inf" if nm >= UNBOUNDED else nm,
        "stwu_words": frame_words,
        "predicted": predicted,
        "observed": observed,
        "verdict": "HIT" if predicted == observed else "MISS",
    }, None


HDR = ("family\tk\tN\tcallee\ts\tlinkage\tselection\tnparams\tvarargs\tleaf\t"
       "index\tnmax\tstwu_words\tpredicted\tobserved\tverdict")


def main(argv):
    if "--list" in argv:
        for f in FAMILIES:
            print(f, FAMILIES[f].__doc__.split("\n")[0])
        return 0
    out = argv[argv.index("--out") + 1]
    fams = (argv[argv.index("--families") + 1].split(",")
            if "--families" in argv else list(FAMILIES))
    kmax = int(argv[argv.index("--kmax") + 1]) if "--kmax" in argv else 12
    ns = [int(x) for x in (argv[argv.index("--n") + 1].split(",")
                           if "--n" in argv else ["1", "4"])]
    os.makedirs(out, exist_ok=True)

    # The grid is stamped BEFORE it is compiled, so what was run is what was
    # frozen. The stamp covers every source byte of every cell.
    plan = []
    h = hashlib.sha256()
    for fa in fams:
        for k in range(kmax + 1):
            for n in ns:
                s = source(fa, k, n)
                plan.append((fa, k, n, s))
                h.update(f"{fa}|{k}|{n}|".encode())
                h.update(s.encode())
    stamp = h.hexdigest()
    open(os.path.join(out, "GRID.sha256"), "w").write(
        f"{stamp}  families={','.join(fams)} kmax={kmax} n={','.join(map(str, ns))} "
        f"cells={len(plan)}\n")
    print(f"GRID stamp {stamp}  cells {len(plan)}", file=sys.stderr)
    if "--emit-only" in argv:
        for fa, k, n, s in plan:
            open(os.path.join(out, f"{fa}_k{k}_n{n}.cpp"), "w").write(s)
        return 0

    rows, errs = [], []
    for fa, k, n, s in plan:
        tag = f"{fa}_k{k}_n{n}"
        obj, err = compile_cell(s, out, tag)
        if obj is None:
            errs.append((tag, err))
            continue
        g, err = grade_cell(obj, n)
        if g is None:
            errs.append((tag, err))
            continue
        rows.append([fa, k, n, g["callee"], g["s"], g["linkage"], g["selection"],
                     g["nparams"], g["varargs"], g["leaf"], g["index"], g["nmax"],
                     g["stwu_words"], g["predicted"], g["observed"], g["verdict"]])
    open(os.path.join(out, "grid.tsv"), "w").write(
        "\n".join([HDR] + ["\t".join(str(c) for c in r) for r in rows]) + "\n")
    hit = sum(1 for r in rows if r[-1] == "HIT")
    print(f"cells graded {len(rows)}  ungradeable {len(errs)}  "
          f"HIT {hit}  MISS {len(rows) - hit}", file=sys.stderr)
    for t, e in errs[:20]:
        print(f"  UNGRADEABLE {t}: {e}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
