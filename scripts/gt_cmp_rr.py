#!/usr/bin/env python3
"""gt_cmp_rr.py — the REGISTER-REGISTER comparison spine and its label surcharge.

`scripts/gt_cmp_spine.py` measures the comparison spine over `<value> <rel>
<literal>`. This one measures it over `<value> <rel> <value>` — the form
`return a->m() <rel> b->n();` produces, where both operands are call results and
one of them is in a callee-saved register.

`docs/CMP_PRODUCES_A_VALUE.md` reading 4 established that the difference is
formed with `subf`/`sub` rather than `addi r11,a,-k`; WCB took the `==` cell of
that. This script measures the other five, and the two axes reading 1 and
reading 3 say are traps: the **result type** (`bool` against `int`) and the
**label-counter surcharge**.

Two modes:

    scripts/gt_cmp_rr.py               the 20-cell spine grid (6 rels signed
                                       x {int,bool}, 4 rels unsigned x {int,bool}),
                                       all in ONE translation unit
    scripts/gt_cmp_rr.py --stride [--mode '/Ox /GS- /Gy /c']
                                       the label stride and leading count, by
                                       `gt_label_stride.py`'s seed-free in-TU
                                       method with its `a2` control

Env: C2RS_WIBO / C2RS_COMPILERS as for scripts/gt_capture.sh.

Outside the std-only Rust workspace on purpose — tooling, never linked into the
port. Exit status is 0 if every probe compiled; read the table for the finding.
"""

import os
import re
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))

DECLS = "struct U { int m() const; int n() const; unsigned um() const; unsigned un() const; };\n"

RELS = [("ge", ">="), ("gt", ">"), ("lt", "<"), ("le", "<="), ("eq", "=="), ("ne", "!=")]


def spine_src():
    out = [DECLS]
    for sign, (l, r) in (("s", ("m", "n")), ("u", ("um", "un"))):
        for key, op in RELS:
            if sign == "u" and key in ("eq", "ne"):
                continue  # the `==`/`!=` fold does not read signedness
            for res in ("int", "bool"):
                nm = "%s_%s_%s" % (sign, key, res[0])
                out.append(
                    "%s %s(const U* p, const U* q) { return p->%s() %s q->%s(); }"
                    % (res, nm, l, op, r)
                )
    return "\n".join(out) + "\n"


def capture(src, mode, workdir, tag):
    cpp = os.path.join(workdir, "%s.cpp" % tag)
    open(cpp, "w").write(src)
    r = subprocess.run(
        [os.path.join(HERE, "gt_capture.sh"), cpp] + mode.split(),
        capture_output=True, text=True,
    )
    path = r.stdout.strip()
    if not path or not os.path.exists(path):
        sys.stderr.write(r.stderr)
        return None
    return path


def run_spine(mode):
    wd = tempfile.mkdtemp(prefix="gt-cmp-rr")
    obj = capture(spine_src(), mode, wd, "rr")
    if obj is None:
        print("SKIP: toolchain absent")
        return 0
    d = subprocess.run(
        [os.path.join(HERE, "gt_dump.py"), obj, "--text-only"],
        capture_output=True, text=True,
    ).stdout
    cur, size, body, rows = None, None, [], []
    # The sentinel flushes the LAST group; it must satisfy the same regex, or
    # the final probe silently vanishes from the table.
    for line in d.splitlines() + ["-- .text #0 (0 B) ?END@@"]:
        m = re.match(r"-- \.text #\d+ \((\d+) B\) \?(\w+)@@", line)
        if m:
            if cur:
                rows.append((cur, size, body))
            cur, size, body = m.group(2), int(m.group(1)), []
        elif cur and re.match(r"\s+[0-9a-f]{4}\s", line):
            body.append(line.rstrip())
    print("===== register-register spine  %s" % mode)
    print("  %-10s %5s  %s" % ("probe", "size", "post-call words"))
    seen = {}
    for name, size, body in rows:
        bl = max((j for j, x in enumerate(body) if " bl " in x), default=None)
        if bl is None:
            continue
        end = min((j for j, x in enumerate(body) if "addi 1, 1," in x), default=len(body))
        words = [x.split("  ", 3)[-1].strip().replace("\t", " ") for x in body[bl + 1:end]]
        txt = " ; ".join(words)
        print("  %-10s %4dB  %s" % (name, size, txt))
        seen[name] = txt
    # The claim: for `>` and `<` the RESULT TYPE does not move a byte, and for
    # `>=`/`<=` it does. Printed as a verdict so the grid is self-reading.
    print("  ---- result-type axis (`int` against `bool`), per relation")
    for sign in ("s", "u"):
        for key, op in RELS:
            a, b = seen.get("%s_%s_i" % (sign, key)), seen.get("%s_%s_b" % (sign, key))
            if a is None or b is None:
                continue
            print("  %s %-2s  %s" % (sign, op, "same" if a == b else "DIFFERENT"))
    return 0


# The stride probes. `P` is the function under test; `gt_label_stride.build_src`
# wraps it in three plain framed anchors so the stride is read out of the object
# rather than assumed from the flags string.
STRIDE_PROBES = [
    ("rr-plain-add", "int  P(const U* p, const U* q){ return p->m() + q->n(); }"),
    ("rr-i-s-eq", "int  P(const U* p, const U* q){ return p->m() == q->n(); }"),
    ("rr-b-s-eq", "bool P(const U* p, const U* q){ return p->m() == q->n(); }"),
    ("rr-i-s-gt", "int  P(const U* p, const U* q){ return p->m() >  q->n(); }"),
    ("rr-b-s-gt", "bool P(const U* p, const U* q){ return p->m() >  q->n(); }"),
    ("rr-i-s-lt", "int  P(const U* p, const U* q){ return p->m() <  q->n(); }"),
    ("rr-b-s-lt", "bool P(const U* p, const U* q){ return p->m() <  q->n(); }"),
    ("rr-b-s-ge", "bool P(const U* p, const U* q){ return p->m() >= q->n(); }"),
    ("rr-b-s-le", "bool P(const U* p, const U* q){ return p->m() <= q->n(); }"),
    ("rr-i-u-gt", "int  P(const U* p, const U* q){ return p->um() >  q->un(); }"),
    ("rr-b-u-gt", "bool P(const U* p, const U* q){ return p->um() >  q->un(); }"),
    ("rr-b-u-lt", "bool P(const U* p, const U* q){ return p->um() <  q->un(); }"),
]


def run_stride(mode):
    sys.path.insert(0, HERE)
    import gt_label_stride as G

    wd = tempfile.mkdtemp(prefix="gt-cmp-rr-stride")
    print("===== stride  %s\n  %-14s %7s %8s  control" % (mode, "probe", "stride", "leading"))
    for name, body in STRIDE_PROBES:
        o = G.capture(G.build_src(DECLS, [], body), mode, wd, name)
        if o is None:
            print("SKIP: toolchain absent")
            return 0
        groups = {g["name"]: g for g in G.groups(o)}

        def first(nm):
            for k, g in groups.items():
                if k.startswith("?" + nm + "@"):
                    return min(g["labels"]) if g["labels"] else None
            return None

        a0, a1, a2, p = first("a0"), first("a1"), first("a2"), first("P")
        if None in (a0, a1, a2):
            print("  %-14s ANCHORS MISSING" % name)
            continue
        anchor = a2 - a1
        ctl = "OK(%d)" % anchor if anchor in (4, 5) else "CTL-BROKEN(%d)" % anchor
        lead = "-" if p is None else str(p - a0 - anchor)
        print("  %-14s %7d %8s  %s" % (name, a1 - a0 - anchor, lead, ctl))
    return 0


def main():
    args = sys.argv[1:]
    modes = []
    while "--mode" in args:
        i = args.index("--mode")
        modes.append(args[i + 1])
        del args[i:i + 2]
    if "--stride" in args:
        for m in modes or ["/Ox /GS- /Gy /c", "/O1 /GS- /Gy /c"]:
            if run_stride(m):
                return 1
        return 0
    for m in modes or ["/O1 /GS- /Gy /c", "/Ox /GS- /Gy /c"]:
        if run_spine(m):
            return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
