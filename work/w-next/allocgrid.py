#!/usr/bin/env python3
"""allocgrid.py — the MIXED-KIND store run, where `codegen::alloc` stops.

`crates/c2-core/src/codegen/alloc.rs` ships four clauses and refuses one
population by name:

    "A run mixing constant and register-derived producers.  Clause 2 is
     measured only on the supplementary probe, never on the held-out
     partition, so it is not shipped."

`xboxheap.cpp`'s constructor is EXACTLY that mix — `addi r11,r3,8` (register-
derived, 2 uses) beside `li r10,0` (constant, 1 use).  The question this grid
exists to answer, and nothing else:

    Does CLAUSE 1 (use count, descending) settle the mixed run on its own
    whenever the use counts DIFFER, so that clause 2 is needed only for the
    TIE — or does the mix change the rule?

Two partitions, declared here before the run:

  * `diff-*`  — mixed run, producer use counts DIFFER.  Clause 1 alone predicts
                the allocation.  PREDICTION: the higher-use producer takes r11.
  * `tie-*`   — mixed run, use counts EQUAL.  Only clause 2 can order these.
                PREDICTION REGISTERED SEPARATELY: register-derived takes r11.

Anchor control: `anchor` must reproduce `addi 11 / li 10`, or the harness is
not measuring what it claims.

Every cell is a LEAF (no call), so this measures the allocator alone and not
the framed-call class beside it.
"""

import os
import subprocess
import sys
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
REFOBJ = os.path.join(ROOT, "work", "w-frame", "refobj.sh")
DUMP = os.path.join(ROOT, "scripts", "gt_dump.py")
DC3 = os.environ.get("C2RS_DC3", "/home/free/code/milohax/dc3-decomp")

# S has 10 int-sized fields at 0,4,...,36 and an inner L at 40.
HEAD = """\
struct L%(t)s { int n; int p; };
struct S%(t)s {
    int f0; int f1; int f2; int f3; int f4;
    int f5; int f6; int f7; int f8; int f9;
    L%(t)s inner;
};
void g%(t)s(S%(t)s* s, int u, int v) {
%(body)s
}
"""


def cell(tag, body):
    return HEAD % dict(t=tag, body=body)


def build():
    c = {}

    def add(name, body):
        t = name.replace("-", "_")
        c[name] = cell(t, body % {"t": t})

    # ---- ANCHOR: xboxheap's own producer pair, as a leaf -------------------
    # addi (2 uses) + li (1 use).  Must read addi->r11, li->r10.
    add("anchor", """\
    L%(t)s& q = s->inner;
    s->f0 = 0;
    q.n = (int)&q;
    q.p = (int)&q;""")

    # ---- PARTITION `diff`: mixed kinds, use counts DIFFER ------------------
    # reg-derived 2 uses vs constant 1 use  -> clause 1 says reg-derived r11
    add("diff-reg2-const1", """\
    L%(t)s& q = s->inner;
    s->f0 = 7;
    q.n = (int)&q;
    q.p = (int)&q;""")
    # reg-derived 1 use vs constant 2 uses  -> clause 1 says CONSTANT r11.
    # This is the discriminating cell: clause 2 would say reg-derived.
    add("diff-reg1-const2", """\
    L%(t)s& q = s->inner;
    s->f0 = 7;
    s->f1 = 7;
    q.n = (int)&q;""")
    # reg-derived 3 uses vs constant 1
    add("diff-reg3-const1", """\
    L%(t)s& q = s->inner;
    s->f0 = 7;
    q.n = (int)&q;
    q.p = (int)&q;
    s->f2 = (int)&q;""")
    # reg-derived 1 use vs constant 3 uses
    add("diff-reg1-const3", """\
    L%(t)s& q = s->inner;
    s->f0 = 7;
    s->f1 = 7;
    s->f2 = 7;
    q.n = (int)&q;""")

    # ---- PARTITION `tie`: mixed kinds, use counts EQUAL --------------------
    # Only clause 2 can order these.
    add("tie-1-1", """\
    L%(t)s& q = s->inner;
    s->f0 = 7;
    q.n = (int)&q;""")
    add("tie-2-2", """\
    L%(t)s& q = s->inner;
    s->f0 = 7;
    s->f1 = 7;
    q.n = (int)&q;
    q.p = (int)&q;""")
    add("tie-1-1-swapped", """\
    L%(t)s& q = s->inner;
    q.n = (int)&q;
    s->f0 = 7;""")

    # ---- CONTROLS: pure partitions, where the shipped rule IS graded -------
    add("pure-const", """\
    s->f0 = 7;
    s->f1 = 9;""")
    add("pure-reg", """\
    L%(t)s& q = s->inner;
    q.n = (int)&q;
    s->f0 = (int)(&q) + 4;""")

    return c


def words(obj):
    out = subprocess.run([sys.executable, DUMP, obj, "--text-only"],
                         capture_output=True, text=True).stdout
    res, cur = {}, None
    for line in out.splitlines():
        if line.startswith("-- .text"):
            cur = line.split(") ", 1)[-1].strip()
            res[cur] = []
        elif cur:
            p = line.split()
            if len(p) >= 3 and len(p[1]) == 8:
                res[cur].append(" ".join(p[2:]).split(";")[0].strip())
    return res


def run(a):
    name, src, out = a
    cpp = os.path.join(out, name + ".cpp")
    open(cpp, "w").write(src)
    obj = os.path.join(out, name + ".obj")
    r = subprocess.run([REFOBJ, os.path.relpath(cpp, DC3), obj],
                       capture_output=True, text=True,
                       env=dict(os.environ, C2RS_DC3=DC3))
    if r.returncode != 0 or not os.path.exists(obj):
        return name, None
    return name, words(obj)


def main():
    out = os.path.join(HERE, "allocgrid")
    os.makedirs(out, exist_ok=True)
    cells = build()
    with cf.ThreadPoolExecutor(max_workers=8) as ex:
        res = dict(ex.map(run, [(n, s, out) for n, s in sorted(cells.items())]))
    ok = 0
    for name in sorted(res):
        w = res[name]
        if w is None:
            print("FAIL %s" % name)
            continue
        ok += 1
        for sec, ws in w.items():
            print("== %-20s %s" % (name, sec))
            for i, d in enumerate(ws):
                print("   %2d  %s" % (i, d))
            print()
    print("graded %d of %d" % (ok, len(res)))


if __name__ == "__main__":
    main()
