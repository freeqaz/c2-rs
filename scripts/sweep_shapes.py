#!/usr/bin/env python3
"""**What the generated corpus cannot express — at any flag profile.**

`scripts/mode_invariance.py` answers "how much of the *flag* axis can the
generated corpus see". This answers the other half, and on 2026-08-04's evidence
it is the half where the defects are:

> A defect survives exactly as long as **no instrument can represent its shape at
> its flags.** "Cannot represent" has more sources than the flag axis.

The day's three wrong-emit families:

| defect | needed | why nothing saw it |
|---|---|---|
| board #232 | an implicit destructor **x** the packed path | in the corpus; graded at one profile |
| w-order Y-a | an empty-bodied unwind target **x** `/EHsc` | in the corpus; `expr_sweep.sh` runs only `/Ox` |
| w-sect (2026-08-04) | a TU with **no functions at all** and namespace-scope data | **the corpus cannot express it.** Every fragment is written to produce a function, so no flag profile would have helped |

The third is not a flag gap and no cross would ever have closed it. Enumerating
what the corpus cannot say is cheap — it needs no toolchain for the source pass
— and it is the list that says where the next fragment should go.

# Two passes, and the honest scope of each

**Pass A (source markers, no toolchain).** A table of shape markers matched
against every generated `.cpp`. It is a *text* scan and it is approximate in one
direction only: a marker reported PRESENT is present (the regex matched real
source), and a marker reported ABSENT means no case's text matched — which is
the claim that matters, because the actionable output is the zero rows. A
smarter parser could only find *more* absences, never fewer.

**Pass B (obj section shapes, needs the toolchain).** Captures each case's
reference obj at the workload's own profile and records the multiset of COFF
section names. Compared against the dc3 workload's own section census
(`work/w-bss/census/sections.jsonl`, 871 objs) this says which obj *shapes* the
corpus can produce and which the workload has but the corpus does not — the axis
w-sect's defect lives on, stated as a measurement rather than as a story.

Usage:

    scripts/sweep_shapes.py                          # pass A only, no toolchain
    scripts/sweep_shapes.py --objs work/w-modes/sh   # + pass B, whole corpus
    scripts/sweep_shapes.py --objs DIR --sample 8    # + pass B on a stride
"""

import argparse
import os
import re
import subprocess
import sys
import threading
from concurrent.futures import ThreadPoolExecutor

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import sweep_gen  # noqa: E402

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# (marker, regex). Ordered roughly by how load-bearing the shape has been on this
# project. A row that comes back 0 is a shape no flag profile can reach.
MARKERS = [
    ("function definition",        r"\)\s*(?:const\s*)?\{"),
    ("NO function definition",     None),            # special: negation of the above
    ("namespace-scope data def",   None),            # special: see below
    ("static (internal) function", r"^\s*static\s+[A-Za-z_].*\)\s*\{"),
    ("static local variable",      r"\{[^}]*\bstatic\s+\w+\s+\w+\s*[=;]"),
    ("virtual function",           r"\bvirtual\b"),
    ("pure virtual",               r"=\s*0\s*;"),
    ("multiple inheritance",       r":\s*(?:public\s+)?\w+\s*,\s*(?:public\s+)?\w+\s*\{"),
    ("constructor/destructor",     r"~\w+\s*\(|\b(\w+)::\1\s*\("),
    ("template",                   r"\btemplate\s*<"),
    ("namespace",                  r"\bnamespace\b"),
    ("anonymous namespace",        r"\bnamespace\s*\{"),
    ("try/catch",                  r"\btry\b|\bcatch\s*\("),
    ("throw",                      r"\bthrow\b"),
    ("switch",                     r"\bswitch\s*\("),
    ("loop (for/while/do)",        r"\bfor\s*\(|\bwhile\s*\(|\bdo\s*\{"),
    ("goto / label",               r"\bgoto\b"),
    ("if / branch",                r"\bif\s*\("),
    ("ternary ?:",                 r"\?[^:;]*:"),
    ("logical && ||",              r"&&|\|\|"),
    ("bitwise & | ^ ~",            r"(?<![&])&(?![&])|(?<!\|)\|(?!\|)|\^|~(?!\w)"),
    ("shift << >>",                r"<<|>>"),
    ("division / modulo",          r"/(?![/*])|%"),
    ("array type",                 r"\w+\s*\[\s*\d*\s*\]"),
    ("struct/class by value param", None),           # special
    ("union",                      r"\bunion\b"),
    ("enum",                       r"\benum\b"),
    ("bitfield",                   r":\s*\d+\s*;"),
    ("operator overload",          r"\boperator\b"),
    ("function pointer",           r"\(\s*\*\s*\w*\s*\)\s*\("),
    ("varargs ...",                r"\.\.\."),
    ('extern "C"',                 r'extern\s*"C"'),
    ("__declspec",                 r"__declspec"),
    ("#pragma",                    r"#\s*pragma"),
    ("#include",                   r"#\s*include"),
    ("inline keyword",             r"\binline\b"),
    ("const member function",      r"\)\s*const\s*\{"),
    ("reference type &",           r"\w+\s*&\s*\w+\s*[,)=]"),
    ("long long / __int64",        r"\blong\s+long\b|__int64"),
    ("unsigned",                   r"\bunsigned\b"),
    ("short",                      r"\bshort\b"),
    ("char arithmetic",            r"\bchar\b"),
    ("bool",                       r"\bbool\b"),
    ("float / double",             r"\bfloat\b|\bdouble\b"),
    ("string literal",             r'"'),
    ("wide char / L\"\"",          r'\bwchar_t\b|L"'),
    ("new / delete",               r"\bnew\b|\bdelete\b"),
    ("cast",                       r"static_cast|reinterpret_cast|const_cast|\(\s*(?:int|char|float|double|void|unsigned|long|short)\s*\*?\s*\)\s*\w"),
    ("sizeof",                     r"\bsizeof\b"),
    ("typedef",                    r"\btypedef\b"),
    ("volatile",                   r"\bvolatile\b"),
    ("const",                      r"\bconst\b"),
    ("pointer",                    r"\*"),
    ("default argument",           r"\w+\s*=\s*[^,)]+\s*[,)]\s*(?:const\s*)?\{"),
    ("main()",                     r"\bmain\s*\("),
    ("nested class",               r"\{[^}]*\b(?:struct|class)\s+\w+\s*\{"),
]

FUNC_DEF = re.compile(r"\)\s*(?:const\s*)?\{")
# A namespace-scope object DEFINITION: a line that is not a function, not a
# declaration-only `extern`, and not inside a brace. Deliberately conservative.
DATA_DEF = re.compile(
    r"^\s*(?!extern\b)(?:static\s+|const\s+|volatile\s+|__declspec\([^)]*\)\s*)*"
    r"(?:unsigned\s+|signed\s+|long\s+|short\s+)*"
    r"(?:int|char|float|double|bool|wchar_t|void|[A-Z]\w*)\s*\*?\s*"
    r"\w+\s*(?:\[[^\]]*\])?\s*(?:=[^;]*)?;\s*$",
    re.M,
)
BYVAL_PARAM = re.compile(r"\(\s*(?!void\b)(?:const\s+)?[A-Z]\w*\s+\w+\s*[,)]")


def markers_of(src):
    out = set()
    body_stripped = re.sub(r"\{[^{}]*\}", "{}", src)
    has_func = bool(FUNC_DEF.search(src))
    for name, rx in MARKERS:
        if rx is None:
            continue
        if re.search(rx, src, re.M):
            out.add(name)
    if not has_func:
        out.add("NO function definition")
    if DATA_DEF.search(body_stripped):
        out.add("namespace-scope data def")
    if BYVAL_PARAM.search(src):
        out.add("struct/class by value param")
    return out


# ---- pass B: obj section shapes ---------------------------------------------

def section_names(path):
    with open(path, "rb") as fh:
        b = fh.read()
    if len(b) < 20:
        return None
    nsec = int.from_bytes(b[2:4], "little")
    opt = int.from_bytes(b[16:18], "little")
    base = 20 + opt
    names = []
    for i in range(nsec):
        off = base + 40 * i
        if off + 40 > len(b):
            return None
        raw = b[off:off + 8].rstrip(b"\0")
        names.append(raw.decode("ascii", "replace"))
    return names


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--frag-dir", default=os.path.join(REPO, "scripts/sweep.d"))
    ap.add_argument("--objs", default="", help="capture objs into DIR (needs the toolchain)")
    ap.add_argument("--flags", default="/O1 /Oi /EHsc",
                    help="profile for pass B (default: the dc3 workload's own)")
    ap.add_argument("--sample", type=int, default=0, help="cases per fragment for pass B (0 = all)")
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--workload", default=os.path.join(REPO, "work/w-bss/census/sections.jsonl"))
    args = ap.parse_args()

    frags = sweep_gen.load_all(args.frag_dir)
    ncases = sum(len(cs) for _, cs in frags)
    print("corpus: %d fragments, %d generated cases (source pass needs no toolchain)"
          % (len(frags), ncases))

    hits = {name: [0, set()] for name, _ in MARKERS}
    per_case = {}
    for stem, srcs in frags:
        for i, s in enumerate(srcs, 1):
            m = markers_of(s)
            per_case["%s-%04d.cpp" % (stem, i)] = m
            for name in m:
                hits[name][0] += 1
                hits[name][1].add(stem)

    print()
    print("SHAPE MARKER                     cases  fragments  ")
    print("-------------------------------  -----  ---------  ")
    absent = []
    for name, _ in MARKERS:
        n, fs = hits[name]
        print("%-31s  %5d  %9d  %s" % (name, n, len(fs), "" if n else "<-- CANNOT EXPRESS"))
        if n == 0:
            absent.append(name)
    print()
    print("%d of %d shape markers have ZERO cases in the corpus:" % (len(absent), len(MARKERS)))
    for a in absent:
        print("    %s" % a)
    print()
    print("A zero row is a shape no flag profile reaches. The flag cross cannot")
    print("close any of them; only a new scripts/sweep.d/ fragment can.")

    if not args.objs:
        return 0

    # ---- pass B ----------------------------------------------------------------
    out = os.path.abspath(args.objs)
    cases_dir = os.path.join(out, "cases")
    os.makedirs(cases_dir, exist_ok=True)
    for n in os.listdir(cases_dir):
        if n.endswith(".cpp"):
            os.unlink(os.path.join(cases_dir, n))
    sweep_gen.write_cases(cases_dir, args.frag_dir, "", quiet=True)

    byfrag = {}
    for n in sorted(os.listdir(cases_dir)):
        if n.endswith(".cpp"):
            byfrag.setdefault(n.rsplit("-", 1)[0], []).append(n)
    picked = []
    for frag in sorted(byfrag):
        cs = byfrag[frag]
        if args.sample:
            k = max(1, (len(cs) + args.sample - 1) // args.sample)
            cs = cs[::k][: args.sample]
        picked.extend(cs)

    objdir = os.path.join(out, "obj")
    os.makedirs(objdir, exist_ok=True)
    flags = args.flags.split() + ["/GS-", "/c"]
    lock = threading.Lock()
    shapes = {}
    fails = []

    def do(case):
        src = os.path.join(cases_dir, case)
        objp = os.path.join(objdir, case[:-4] + ".obj")
        env = dict(os.environ)
        env["GT_OUT"] = objp
        env["WIBO_FS_CACHE"] = "1"
        subprocess.run([os.path.join(REPO, "scripts/gt_capture.sh"), src] + flags,
                       capture_output=True, env=env)
        names = section_names(objp) if os.path.exists(objp) else None
        with lock:
            if names is None:
                fails.append(case)
            else:
                key = tuple(sorted(set(names)))
                shapes.setdefault(key, []).append(case)
        if os.path.exists(objp):
            os.unlink(objp)

    # Toolchain probe first: absent -> SKIP, exit 0, never a vacuous pass.
    do(picked[0])
    if fails and len(fails) == 1 and not shapes:
        print()
        print("SKIP: toolchain absent — pass B would be vacuous")
        return 0

    with ThreadPoolExecutor(max_workers=args.jobs) as ex:
        list(ex.map(do, picked))

    graded = sum(len(v) for v in shapes.values())
    print()
    print("pass B: %d cases submitted at [%s], %d graded, %d capture failures"
          % (len(picked), " ".join(flags), graded, len(fails)))
    if graded == 0:
        print("VACUOUS: nothing was graded; every count below would read 0 and pass.")
        return 3

    print()
    print("OBJ SECTION-NAME SETS the corpus produces (%d distinct):" % len(shapes))
    for key in sorted(shapes, key=lambda k: -len(shapes[k])):
        print("  %6d cases  %s" % (len(shapes[key]), " ".join(key)))

    corpus_names = set()
    for key in shapes:
        corpus_names |= set(key)

    if os.path.exists(args.workload):
        import json
        wl_names = set()
        wl_shapes = {}
        n_wl = 0
        with open(args.workload) as fh:
            for line in fh:
                line = line.strip()
                if not line:
                    continue
                row = json.loads(line)
                order = row.get("order") or []
                wl_names |= set(order)
                wl_shapes.setdefault(tuple(sorted(set(order))), 0)
                wl_shapes[tuple(sorted(set(order)))] += 1
                n_wl += 1
        print()
        print("workload (%s, %d objs): %d distinct section names"
              % (os.path.relpath(args.workload, REPO), n_wl, len(wl_names)))
        only_wl = sorted(wl_names - corpus_names)
        print("  section names the WORKLOAD has and the corpus CANNOT produce (%d):" % len(only_wl))
        for n in only_wl:
            print("      %s" % n)
        only_corpus = sorted(corpus_names - wl_names)
        if only_corpus:
            print("  and the corpus has that the workload does not (%d): %s"
                  % (len(only_corpus), " ".join(only_corpus)))
        n_no_text = sum(c for k, c in wl_shapes.items() if ".text" not in k)
        print("  workload objs with NO .text at all: %d" % n_no_text)
        corpus_no_text = sum(len(v) for k, v in shapes.items() if ".text" not in k)
        print("  corpus  cases with NO .text at all: %d" % corpus_no_text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
