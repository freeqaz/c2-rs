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
    scripts/sweep_shapes.py --check                  # GATE MODE, see below

# `--check` — the zero rows, asserted rather than reported

Pass A is a *report*, and a report nobody reads is a shape that quietly goes
back to zero. On 2026-08-04 lane w-shapes closed the last twelve zero rows; the
day after, a fragment deleted or a generator that silently stops emitting one of
its axes puts a row back at zero and **no instrument fails**. `expr_sweep.sh`
would keep printing `checked=N mismatches=0` over a corpus that had lost a
shape, which is `docs/STATUS.md` trap 5 in its purest form — absence reading as
success.

So `--check` is the same pass A with an exit code:

  * every marker must have at least one case (baseline
    `C2RS_MAX_ZERO_MARKERS`, default **0** — raising it needs a reason written
    beside the number, the same discipline `C2RS_SWEEP_MAX_UNGRADED=96` carries);
  * every fragment must emit at least one case, which is the observable symptom
    of the counter bug `sweep_gen.py` was restructured for;
  * the marker table and the corpus must both be NON-EMPTY, because "0 markers
    have zero cases" is also what a table of zero markers prints.

It needs no toolchain and no compiler, so it is a `gate.sh --selftest` case.
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
    # ---- RTTI (lane w-gr, task #40) -----------------------------------------
    # `.rdata$r` is 24,163 sections over 676 of the workload's 871 objs and was
    # the last section name the corpus could not produce. The four rows are NOT
    # interchangeable and the order they are read in matters: `dynamic_cast` and
    # `typeid` are the shapes one *expects* to mint RTTI and measurably do not
    # (their `??_R0` descriptors land in `.data`), while the record block hangs
    # off the **vftable**, which is emitted by whichever TU generates a
    # constructor or destructor body. A corpus with the first two and not the
    # last would report three green rows and produce zero `.rdata$r`.
    ("dynamic_cast",               r"\bdynamic_cast\s*<"),
    ("typeid",                     r"\btypeid\s*\("),
    ("virtual inheritance",        r"[:,]\s*(?:public\s+|private\s+|protected\s+)?virtual\b"),
    ("polymorphic ctor/dtor def",  None),           # special: see markers_of
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
# An out-of-line constructor or destructor DEFINITION: `S::S(…){` / `S::~S(…){`,
# the same class name on both sides. Crossed with `virtual` appearing anywhere in
# the TU, this is the marker for "a vftable is emitted here" — measured (lane
# w-gr) to be the one thing that mints `.rdata$r`, and not implied by any of
# `dynamic_cast`, `typeid`, `virtual`, multiple inheritance or virtual
# inheritance, each of which produces none on its own.
CTOR_DTOR_DEF = re.compile(r"\b(\w+)\s*::\s*~?\1\s*\([^)]*\)\s*(?::[^{;]*)?\{")
VIRTUAL = re.compile(r"\bvirtual\b")


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
    if VIRTUAL.search(src) and CTOR_DTOR_DEF.search(src):
        out.add("polymorphic ctor/dtor def")
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


def check(frags, ncases):
    """`--check`: the zero rows as an assertion. Returns a process exit code.

    Everything here is a COUNT compared against a floor, never a status compared
    against a word — `docs/STATUS.md` trap 5's standing mitigation. The
    degeneracy guards come first on purpose: `0 markers have zero cases` is what
    an empty marker table prints, and `every fragment emitted a case` is what a
    corpus of no fragments satisfies.
    """
    rc = 0
    if len(MARKERS) < 1 or ncases < 1 or not frags:
        print("DEGENERATE: %d markers, %d fragments, %d cases — this check would"
              " pass by having nothing to check." % (len(MARKERS), len(frags), ncases))
        return 3

    empty = [stem for stem, cs in frags if not cs]
    if empty:
        print("FRAGMENT EMITTED ZERO CASES: %s" % ", ".join(empty))
        print("  A generator that stops emitting is the counter bug's own symptom")
        print("  (scripts/sweep_gen.py) and it is a hard error, never a smaller corpus.")
        rc = 1

    hits = {name: 0 for name, _ in MARKERS}
    for _stem, srcs in frags:
        for s in srcs:
            for name in markers_of(s):
                hits[name] += 1
    absent = [n for n, _ in MARKERS if not hits[n]]

    try:
        allowed = int(os.environ.get("C2RS_MAX_ZERO_MARKERS", "0"))
    except ValueError:
        allowed = 0
    print("check: %d of %d shape markers have zero cases (baseline %d),"
          " %d fragments all non-empty, %d cases"
          % (len(absent), len(MARKERS), allowed, len(frags), ncases))
    if len(absent) > allowed:
        for a in absent:
            print("    ZERO  %s" % a)
        print("  A zero row is a shape no flag profile reaches and no case grades.")
        print("  Every wrong-emit family found on 2026-08-04 lived in one. Close it")
        print("  with a scripts/sweep.d/ fragment, or raise C2RS_MAX_ZERO_MARKERS")
        print("  with the reason written next to the number.")
        rc = 1
    if rc == 0:
        print("SHAPE-CHECK: PASS")
    return rc


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--frag-dir", default=os.path.join(REPO, "scripts/sweep.d"))
    ap.add_argument("--objs", default="", help="capture objs into DIR (needs the toolchain)")
    # CORRECTED 2026-08-04 (lane w-gr): this default read `/O1 /Oi /EHsc` and
    # called itself "the dc3 workload's own". `work/dc3-workload/flags.txt` is
    #   `/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc /I …`
    # — short by `/GR`, which is the ONE flag that mints `.rdata$r`, which is
    # the ONE section this report existed to name as unproducible. So the
    # instrument was measuring at a profile that could not produce the thing it
    # was reporting missing, and would have gone on reporting it missing after a
    # fragment closed it. `/GR` is not this compiler's default: measured, the
    # section is absent both without the flag and with an explicit `/GR-`.
    ap.add_argument("--flags", default="/GR /O1 /Oi /EHsc",
                    help="profile for pass B (default: the dc3 workload's own, "
                         "verbatim from work/dc3-workload/flags.txt)")
    ap.add_argument("--sample", type=int, default=0, help="cases per fragment for pass B (0 = all)")
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--workload", default=os.path.join(REPO, "work/w-bss/census/sections.jsonl"))
    ap.add_argument("--check", action="store_true",
                    help="gate mode: exit non-zero if any marker has zero cases")
    args = ap.parse_args()

    frags = sweep_gen.load_all(args.frag_dir)
    ncases = sum(len(cs) for _, cs in frags)
    print("corpus: %d fragments, %d generated cases (source pass needs no toolchain)"
          % (len(frags), ncases))
    if args.check:
        return check(frags, ncases)

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
        # Two of these have never been a shape gap. The census records the two
        # XDK build-info sections as `.XBLD$W:C1` / `.XBLD$W:C2` (after the
        # `__C1_*` / `__C2_*` symbols they hold); the COFF section header field
        # spells both `.XBLD$W`, which is what this reader sees and what every
        # generated case already produces. They are a NAMING difference between
        # two readers, not a section the corpus lacks — so they are labelled and
        # subtracted, and the remainder is printed as its own number. A list
        # whose every row is a known artefact reads exactly like a list of real
        # gaps, and this one did for two lanes.
        artefact = [n for n in only_wl
                    if any(n.startswith(c) or c.startswith(n) for c in corpus_names)]
        honest = [n for n in only_wl if n not in artefact]
        for n in only_wl:
            note = ""
            if n in artefact:
                note = "   <-- READER ARTEFACT: the corpus produces %s" % (
                    " ".join(sorted(c for c in corpus_names
                                    if n.startswith(c) or c.startswith(n))))
            print("      %s%s" % (n, note))
        print("  HONEST REMAINDER — workload names with no corpus spelling at all: %d%s"
              % (len(honest), ("   " + " ".join(honest)) if honest else ""))
        if honest:
            print("    Each is a section no lane and no fragment can reach. Closing one")
            print("    needs BOTH halves: a scripts/sweep.d/ fragment for the shape and a")
            print("    scripts/lanes.txt row for the flag. `.rdata$r` needed both.")
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
