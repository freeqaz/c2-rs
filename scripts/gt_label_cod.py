#!/usr/bin/env python3
"""gt_label_cod.py — board #135: c2's label counter, read off the `.cod` listing
in ALLOCATION order.

`docs/ROADMAP.md` §9.3 read one framed + EH body and found that the counter is
not allocated in text-emission order: a funclet is allocated FIRST and emitted
LAST, and the `$M` block splits around the `$T` tables. One body is one sample.
This script widens that to a family of shapes (leaf, framed, EH, loop, switch,
several functions per TU, ctor/dtor, virtual, FP) across four flag sets, states
the allocation rule as a PREDICATE, and grades it — separately on the shapes it
was fitted on and on shapes it was not.

The instrument is `c2rs listing` (board #132), which is non-perturbing
(ROADMAP §9.1 result 1): the obj beside a `.cod` is byte-identical to the obj
without one. So this reads the same compiler the differential grades.

Usage:
    scripts/gt_label_cod.py gen                 # write the probe corpus
    scripts/gt_label_cod.py scan [--jobs N]     # capture + parse + dump rows
    scripts/gt_label_cod.py dump [probe]        # the allocation-order table
    scripts/gt_label_cod.py grade               # fit / hold out / score

TWO INSTRUMENT DEFECTS, both found before a verdict was read and both recorded
because each one produced a plausible-looking failing row:

  1. **A body-end label took the NEXT function's first offset.** `.cod` prints
     `$M(n+1):` immediately before `ENDP`, and a naive "the label sits at the
     next instruction's offset" walk crosses the function boundary — which
     under `/Gy` restarts at 0. Every multi-function TU then read
     "body-end 0 < prologue-end 12" and the ordering predicate went red on 15
     cells that were fine. Fixed by flushing at `ENDP` to (last offset + 4).
  2. **`/Ox` names its sections with a bare `.rdata` directive**, not
     `NAME SEGMENT`, so the segment tracker attributed every `/Ox` `$T` to
     `.XBLD$W` — the last `SEGMENT` line in the file's header. The state-table
     predicate then scored `n/a` on all twenty `/Ox` rows, which prints exactly
     like a predicate that passed.

Everything lands under `work/wlabel/` (gitignored). std-lib only; this is
tooling, outside the std-only Rust workspace.
"""

import json
import os
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WORK = os.path.join(ROOT, "work", "wlabel")
PROBES = os.path.join(WORK, "probes")
CODS = os.path.join(WORK, "cod")
ROWS = os.path.join(WORK, "rows.jsonl")
C2RS = os.path.join(ROOT, "target", "release", "c2rs")

MODES = {
    "O1-Oi-EHsc": ["/O1", "/Oi", "/EHsc", "/GS-", "/c"],
    "O1": ["/O1", "/GS-", "/c"],
    "O2-EHsc": ["/O2", "/EHsc", "/GS-", "/c"],
    "Ox": ["/Ox", "/GS-", "/c"],
}

# ---------------------------------------------------------------------------
# The probe corpus. `fit` marks the shapes the rule is allowed to be fitted on;
# every other shape is HELD OUT — the rule was written against the `fit` rows
# and the `held` rows were not looked at until it was fixed.
# ---------------------------------------------------------------------------
PROBES_SRC = {
    # ---- FIT shapes -------------------------------------------------------
    "leaf": ("fit", """
int f(int a) { return a + 1; }
"""),
    "framed": ("fit", """
int g(int);
int f(int a) { return g(a) + 1; }
"""),
    "eh_try": ("fit", """
int mk(int);
int f(int n) { int s = 0; try { s = mk(n); } catch (...) { s = -1; } return s; }
"""),
    "eh_dtor": ("fit", """
struct T { T(); ~T(); int v; };
int mk(int);
int f(int n) { T t; return mk(n) + t.v; }
"""),
    "multi3": ("fit", """
int g(int);
int a(int x) { return x + 1; }
int b(int x) { return g(x) + 2; }
int c(int x) { return x * 3; }
"""),

    # ---- HELD-OUT shapes --------------------------------------------------
    "loop": ("held", """
int u(int);
int f(int n) { int s = 0; for (int i = 0; i < n; ++i) s += u(i); return s; }
"""),
    "switch8": ("held", """
int u(int);
int f(int n) {
  switch (n) {
    case 0: return u(0); case 1: return u(1); case 2: return u(2);
    case 3: return u(3); case 4: return u(4); case 5: return u(5);
    case 6: return u(6); case 7: return u(7); default: return -1;
  }
}
"""),
    "eh_loop_div": ("held", """
struct T { T(); ~T(); int v; };
int mk(int); int use(int);
int f(int n) {
  T t; int s = 0;
  try { s = mk(n); } catch (...) { s = -1; }
  for (int i = 0; i < n; ++i) s += use(i);
  return s / (n + 1);
}
"""),
    "eh_two_catch": ("held", """
int mk(int);
int f(int n) {
  int s = 0;
  try { s = mk(n); } catch (int e) { s = e; } catch (...) { s = -1; }
  return s;
}
"""),
    "eh_nested": ("held", """
struct T { T(); ~T(); int v; };
int mk(int);
int f(int n) {
  T a; int s = 0;
  try { T b; try { s = mk(n); } catch (...) { s = -1; } s += b.v; }
  catch (...) { s = -2; }
  return s + a.v;
}
"""),
    "eh_beside_plain": ("held", """
int mk(int);
int p(int x) { return x + 1; }
int f(int n) { int s = 0; try { s = mk(n); } catch (...) { s = -1; } return s; }
int q(int x) { return mk(x) + 2; }
"""),
    "two_eh": ("held", """
int mk(int);
int f(int n) { int s = 0; try { s = mk(n); } catch (...) { s = -1; } return s; }
int g(int n) { int s = 0; try { s = mk(n); } catch (...) { s = -2; } return s; }
"""),
    "cmp_rel": ("held", """
int g(int); int h(int);
bool f(int a, int b) { return g(a) < h(b); }
"""),
    "float_leaf": ("held", """
float f(float a) { return a * 2.5f + 1.5f; }
"""),
    "virt": ("held", """
struct B { virtual int v(int); virtual ~B(); };
int f(B* b, int x) { return b->v(x); }
"""),
    "ctor_dtor": ("held", """
struct T { T(int); ~T(); int v; };
int mk(int);
struct U { T a; T b; U(int x); };
U::U(int x) : a(x), b(x + 1) {}
int f(int n) { U u(n); return mk(u.a.v); }
"""),
    "many_locals": ("held", """
int g(int, int, int, int, int, int, int, int);
int f(int a) {
  int x0=a, x1=a+1, x2=a+2, x3=a+3, x4=a+4, x5=a+5, x6=a+6, x7=a+7;
  return g(x0,x1,x2,x3,x4,x5,x6,x7);
}
"""),
    "eh_loop_two_fn": ("held", """
struct T { T(); ~T(); int v; };
int mk(int); int use(int);
int f(int n) { T t; int s = 0; for (int i=0;i<n;++i) s += use(i); return s + t.v; }
int g(int n) { int s = 0; try { s = mk(n); } catch (...) { s = -1; } return s; }
"""),
    "five_leaves": ("held", """
int a(int x){return x+1;} int b(int x){return x+2;} int c(int x){return x+3;}
int d(int x){return x+4;} int e(int x){return x+5;}
"""),
    "leaf_then_framed": ("held", """
int g(int);
int a(int x) { return x + 1; }
int f(int x) { return g(x) + 1; }
"""),
}


def gen():
    os.makedirs(PROBES, exist_ok=True)
    for name, (_grp, src) in PROBES_SRC.items():
        with open(os.path.join(PROBES, name + ".cpp"), "w") as fh:
            fh.write(src.lstrip())
    print("wrote %d probes to %s" % (len(PROBES_SRC), PROBES))


# ---------------------------------------------------------------------------
# parse
# ---------------------------------------------------------------------------
RE_PROC = re.compile(r"^(\S+)\s+PROC\s+NEAR")
RE_ENDP = re.compile(r"^(\S+)\s+ENDP")
RE_LABEL = re.compile(r"^(\$M|\$T)(\d+):")
RE_DATA_LABEL = re.compile(r"^(\$T|\$M)(\d+)\s+DD\s+(\S+)")
RE_INSN = re.compile(r"^\s+([0-9a-f]{5,8})\s+([0-9a-f]{8})\s")
RE_SEG = re.compile(r"^(\S+)\s+SEGMENT")
RE_SEG_DOT = re.compile(r"^\s+(\.[A-Za-z$0-9_]+)\s*$")
RE_FUNCLET = re.compile(r"^(__catch\$|__unwind\$|__tryend\$)(\d+):")


def parse_cod(path):
    """One TU's listing → labels, each carrying its allocation number, kind,
    owning PROC (index in text order) and owning BODY (main, or a funclet)."""
    with open(path, "r", errors="replace") as fh:
        lines = fh.read().splitlines()

    labels, funcs = [], []
    fn = None
    fn_ix = -1
    body = None            # None outside a PROC; "main" or a funclet name
    seg = None
    pending = []
    last_off = None
    n_endprolog = 0

    def flush(off):
        for p in pending:
            p["text_off"] = off
        del pending[:]

    for raw in lines:
        line = raw.rstrip("\n")
        m = RE_SEG.match(line)
        if m:
            seg = m.group(1)
            continue
        m = RE_SEG_DOT.match(line)
        if m:
            seg = m.group(1)
            continue
        m = RE_PROC.match(line)
        if m:
            fn = m.group(1)
            fn_ix += 1
            funcs.append(fn)
            body = "main"
            seg = ".code"
            last_off = None
            continue
        m = RE_ENDP.match(line)
        if m:
            # A body-end `$M` sits immediately before ENDP and belongs to THIS
            # function, at one instruction past its last — not to whatever
            # function the next offset comes from. Instrument defect 1.
            flush((last_off + 4) if last_off is not None else None)
            fn, body, last_off = None, None, None
            continue
        if line.strip() == ".endprolog":
            n_endprolog += 1
            continue
        m = RE_FUNCLET.match(line)
        if m:
            name = m.group(1) + m.group(2)
            body = name
            labels.append(dict(num=int(m.group(2)), kind="funclet", where="code",
                               seg=seg, fn=fn, fn_ix=fn_ix, body=name,
                               text_off=None, name=name))
            pending.append(labels[-1])
            continue
        m = RE_LABEL.match(line)
        if m:
            kind = {"$M": "M", "$T": "T"}[m.group(1)]
            labels.append(dict(num=int(m.group(2)), kind=kind, where="code",
                               seg=seg, fn=fn, fn_ix=fn_ix, body=body,
                               text_off=None, name=m.group(1) + m.group(2)))
            pending.append(labels[-1])
            continue
        m = RE_DATA_LABEL.match(line)
        if m:
            kind = {"$M": "M", "$T": "T"}[m.group(1)]
            # A `.pdata` $T row sits OUTSIDE any PROC and names the body it
            # describes in its operand (`$T2754 DD ?fr@@YAHH@Z`,
            # `$T2599 DD __catch$2570`). Bind by that operand, not by position:
            # binding by position put every one of them in `fn_ix = -1` and the
            # triple predicate read 0/56 — instrument defect 3.
            labels.append(dict(num=int(m.group(2)), kind=kind, where="data",
                               seg=seg, fn=None, fn_ix=None,
                               body=None, describes=m.group(3).rstrip("H,"),
                               text_off=None, name=m.group(1) + m.group(2)))
            continue
        m = RE_INSN.match(line)
        if m:
            off = int(m.group(1), 16)
            last_off = off
            flush(off)
            continue
    # ---- attribute the data labels by their DD operand --------------------
    owner = {}          # body symbol -> (fn_ix, body-name)
    for l in labels:
        if l["where"] == "code" and l["fn"] is not None:
            owner.setdefault(l["fn"], (l["fn_ix"], "main"))
            if l["kind"] == "funclet":
                owner[l["name"]] = (l["fn_ix"], l["name"])
    for ix, f in enumerate(funcs):
        owner.setdefault(f, (ix, "main"))
    for l in labels:
        if l["where"] != "data":
            continue
        tgt = l.get("describes")
        if tgt in owner:
            l["fn_ix"], l["body"] = owner[tgt]
        elif tgt and tgt.startswith("$M"):
            # the `.rdata` EH state table points at its first state $M; bind it
            # to that label's owner.
            for k in labels:
                if k["name"] == tgt and k["fn_ix"] is not None:
                    l["fn_ix"], l["body"] = k["fn_ix"], "state"
                    break
        if l["fn_ix"] is None:
            l["fn_ix"], l["body"] = -1, "unbound"
    return dict(labels=labels, funcs=funcs, n_endprolog=n_endprolog)


# ---------------------------------------------------------------------------
# scan
# ---------------------------------------------------------------------------
def one(args):
    name, mode, flags = args
    cpp = os.path.join(PROBES, name + ".cpp")
    out = os.path.join(CODS, "%s.%s.cod" % (name, mode))
    cmd = [C2RS, "listing", cpp, "--out", out]
    for f in flags:
        cmd += ["--flag", f]
    r = subprocess.run(cmd, capture_output=True, text=True, cwd=ROOT)
    if r.returncode != 0 or not os.path.exists(out):
        return dict(probe=name, mode=mode, ok=False,
                    err=(r.stderr or r.stdout).strip()[:200])
    p = parse_cod(out)
    p.update(probe=name, mode=mode, ok=True,
             group=PROBES_SRC[name][0], flags=flags)
    return p


def scan(jobs):
    os.makedirs(CODS, exist_ok=True)
    work = [(n, m, f) for n in PROBES_SRC for m, f in MODES.items()]
    with ThreadPoolExecutor(max_workers=jobs) as ex:
        rows = list(ex.map(one, work))
    with open(ROWS, "w") as fh:
        for r in rows:
            fh.write(json.dumps(r) + "\n")
    ok = sum(1 for r in rows if r["ok"])
    print("scan: %d/%d captured -> %s" % (ok, len(rows), ROWS))
    for r in rows:
        if not r["ok"]:
            print("  FAIL %-16s %-12s %s" % (r["probe"], r["mode"], r["err"]))


def load():
    with open(ROWS) as fh:
        return [json.loads(x) for x in fh if x.strip()]


def dump(only=None):
    for r in load():
        if not r["ok"] or (only and r["probe"] != only):
            continue
        print("\n=== %s / %s  (%s)  funcs=%d endprolog=%d" %
              (r["probe"], r["mode"], r["group"], len(r["funcs"]), r["n_endprolog"]))
        for l in sorted(r["labels"], key=lambda x: x["num"]):
            print("   %-16s %-8s %-6s %-9s fn=%-2s body=%-14s off=%s" %
                  (l["name"], l["kind"], l["where"], l["seg"] or "-",
                   l["fn_ix"], l["body"] or "-",
                   ("0x%x" % l["text_off"]) if l["text_off"] is not None else "-"))


# ---------------------------------------------------------------------------
# THE RULE, as a predicate over one TU's labels.
#
# In ALLOCATION order (ascending label number) the counter is consumed
# **per function, in `.text` order**, and within one function:
#
#   1. one **funclet-entry** label per funclet the function needs
#      (`__catch$k` / `__unwind$k`), FIRST, before any of that function's
#      `$M`/`$T`;
#   2. the function's **EH state-transition `$M`** block (the labels the
#      `.rdata` state table's `DD` rows point at), ascending;
#   3. the state table's own `$T`, in `.rdata`;
#   4. then one **triple** per emitted body — the main body first, then each
#      funclet in emission order — each triple exactly
#      `$M(n)` prologue end · `$M(n+1)` body end · `$T(n+2)` `.pdata` record,
#      **consecutive**, and the triples of one function consecutive with each
#      other (stride 3).
#
# Steps 1–3 are empty for a function with no EH, which collapses the rule to
# the single triple `coff::plan_labels` already ships. The numbers BETWEEN the
# stages are not contiguous — c2 consumes slots it never emits — and that gap
# is `NOT MODELLED` here; every predicate below is ORDINAL, and the round says
# so rather than fitting a stride to five samples.
# ---------------------------------------------------------------------------
def per_function(row):
    """Group a TU's labels by owning function index."""
    g = {}
    for l in row["labels"]:
        g.setdefault(l["fn_ix"], []).append(l)
    return {k: sorted(v, key=lambda x: x["num"]) for k, v in g.items() if k >= 0}


def fn_triples(ls):
    """The (M,M,T-in-.pdata) triples among one function's labels."""
    by = {l["num"]: l for l in ls}
    out, used = [], set()
    for n in sorted(by):
        a, b, c = by.get(n), by.get(n + 1), by.get(n + 2)
        if not (a and b and c) or set((n, n + 1, n + 2)) & used:
            continue
        if (a["kind"], b["kind"], c["kind"]) != ("M", "M", "T"):
            continue
        if a["where"] != "code" or b["where"] != "code" or c["where"] != "data":
            continue
        if c["seg"] != ".pdata":
            continue
        out.append((n, n + 1, n + 2))
        used |= set((n, n + 1, n + 2))
    return out


def predicates(row):
    """Each predicate → (True/False/None, detail). `None` means the row cannot
    express it; the n/a count is printed beside every score, because a
    predicate scored only where it cannot fail prints like one that passed."""
    out = {}
    fns = per_function(row)

    # ---- P1: every `.pdata` $T closes a consecutive (M, M, T) triple -------
    allT = [l for l in row["labels"] if l["kind"] == "T" and l["seg"] == ".pdata"]
    if allT:
        got = set()
        for ls in fns.values():
            got |= set(t[2] for t in fn_triples(ls))
        want = set(l["num"] for l in allT)
        out["P1_triple"] = (got == want, "pdata $T %s vs triple-closing %s" %
                            (sorted(want), sorted(got)))
    else:
        out["P1_triple"] = (None, "no .pdata $T")

    # ---- P2: within a triple, $M(n) is the prologue end and $M(n+1) the
    #          body end — so off(n) < off(n+1), in the SAME body.
    cells = []
    for ls in fns.values():
        by = {l["num"]: l for l in ls}
        for (a, b, _c) in fn_triples(ls):
            cells.append((a, by[a]["text_off"], by[a]["body"],
                          b, by[b]["text_off"], by[b]["body"]))
    if cells:
        ok = all(oa is not None and ob is not None and oa < ob and ba == bb
                 for (_a, oa, ba, _b, ob, bb) in cells)
        out["P2_prolog_lt_end"] = (ok, str(cells))
    else:
        out["P2_prolog_lt_end"] = (None, "no triple")

    # ---- P3: one triple per emitted body — main plus one per funclet -------
    cells = []
    for ix, ls in fns.items():
        nf = len([l for l in ls if l["kind"] == "funclet"])
        cells.append((ix, len(fn_triples(ls)), 1 + nf))
    framed = [(i, g, w) for (i, g, w) in cells if g]
    if framed:
        out["P3_one_per_body"] = (all(g == w for (_i, g, w) in framed), str(framed))
    else:
        out["P3_one_per_body"] = (None, "no framed function")

    # ---- P4: a function's funclet-entry labels come BEFORE its own $M/$T ---
    cells = []
    for ix, ls in fns.items():
        fl = [l["num"] for l in ls if l["kind"] == "funclet"]
        mt = [l["num"] for l in ls if l["kind"] in ("M", "T")]
        if fl and mt:
            cells.append((ix, max(fl), min(mt)))
    if cells:
        out["P4_funclet_first"] = (all(f < m for (_i, f, m) in cells), str(cells))
    else:
        out["P4_funclet_first"] = (None, "no function with a funclet")

    # ---- P5: …and are EMITTED after the main body — allocated first,
    #          emitted last. Graded on text offset within the function.
    cells = []
    for ix, ls in fns.items():
        fl = [l for l in ls if l["kind"] == "funclet" and l["text_off"] is not None]
        ts = fn_triples(ls)
        if not fl or not ts:
            continue
        by = {l["num"]: l for l in ls}
        main_end = by[ts[0][1]]["text_off"]
        for l in fl:
            cells.append((ix, l["name"], l["text_off"], main_end))
    if cells:
        out["P5_funclet_emitted_last"] = (
            all(off is not None and me is not None and off >= me
                for (_i, _n, off, me) in cells), str(cells))
    else:
        out["P5_funclet_emitted_last"] = (None, "no funclet with an offset")

    # ---- P6: the state-table $T (.rdata) is allocated BELOW the triples ----
    cells = []
    for ix, ls in fns.items():
        st = [l["num"] for l in ls if l["kind"] == "T" and l["seg"] == ".rdata"]
        ts = fn_triples(ls)
        if st and ts:
            cells.append((ix, max(st), ts[0][0]))
    if cells:
        out["P6_state_below"] = (all(s < t for (_i, s, t) in cells), str(cells))
    else:
        out["P6_state_below"] = (None, "no .rdata state table")

    # ---- P7: the $M block SPLITS around the $T tables — but ONLY where
    #          there is EH. §9.3 read this off an EH body and it is an EH
    #          statement; a plain framed function is M,M,T and does not split.
    cells = []
    for ix, ls in fns.items():
        has_eh = any(l["kind"] == "funclet" for l in ls)
        Ms = sorted(l["num"] for l in ls if l["kind"] == "M")
        Ts = sorted(l["num"] for l in ls if l["kind"] == "T")
        if not (Ms and Ts):
            continue
        split = any(Ms[0] < t < Ms[-1] for t in Ts)
        cells.append((ix, has_eh, split, Ms, Ts))
    eh_cells = [c for c in cells if c[1]]
    if eh_cells:
        out["P7_split_iff_eh"] = (all(c[2] for c in eh_cells), str(eh_cells))
    else:
        out["P7_split_iff_eh"] = (None, "no EH function")
    plain = [c for c in cells if not c[1]]
    if plain:
        out["P7b_plain_no_split"] = (all(not c[2] for c in plain), str(plain))
    else:
        out["P7b_plain_no_split"] = (None, "no non-EH framed function")

    # ---- P8: a function's triples are consecutive with each other,
    #          stride exactly 3 — main, then funclets in emission order.
    cells = []
    for ix, ls in fns.items():
        ts = fn_triples(ls)
        if len(ts) >= 2:
            cells.append((ix, [t[0] for t in ts]))
    if cells:
        out["P8_triple_stride_3"] = (
            all(all(b - a == 3 for a, b in zip(v, v[1:])) for (_i, v) in cells),
            str(cells))
    else:
        out["P8_triple_stride_3"] = (None, "no function with two triples")

    # ---- P9: functions are allocated in .text order -----------------------
    firsts = []
    for ix in sorted(fns):
        ns = [l["num"] for l in fns[ix]]
        if ns:
            firsts.append((ix, min(ns)))
    if len(firsts) >= 2:
        out["P9_fn_text_order"] = (
            all(a[1] < b[1] for a, b in zip(firsts, firsts[1:])), str(firsts))
    else:
        out["P9_fn_text_order"] = (None, "fewer than two labelled functions")

    # ---- P10: the main body's triple precedes every funclet's triple ------
    cells = []
    for ix, ls in fns.items():
        by = {l["num"]: l for l in ls}
        ts = fn_triples(ls)
        if len(ts) < 2:
            continue
        cells.append((ix, [by[t[0]]["body"] for t in ts]))
    if cells:
        out["P10_main_triple_first"] = (
            all(v[0] == "main" and all(x != "main" for x in v[1:])
                for (_i, v) in cells), str(cells))
    else:
        out["P10_main_triple_first"] = (None, "no function with two triples")

    return out


NAMES = ["P1_triple", "P2_prolog_lt_end", "P3_one_per_body", "P4_funclet_first",
         "P5_funclet_emitted_last", "P6_state_below", "P7_split_iff_eh",
         "P7b_plain_no_split", "P8_triple_stride_3", "P9_fn_text_order",
         "P10_main_triple_first"]


def grade():
    rows = [r for r in load() if r["ok"]]
    nfit = len(set(r["probe"] for r in rows if r["group"] == "fit"))
    nheld = len(set(r["probe"] for r in rows if r["group"] == "held"))
    print("%-24s %-26s %-26s" % ("", "FITTED (%d shapes)" % nfit,
                                 "HELD OUT (%d shapes)" % nheld))
    print("%-24s %-26s %-26s" % ("predicate", "true/total (n/a)",
                                 "true/total (n/a)"))
    fails = []
    tot_fit = tot_held = ok_fit = ok_held = 0
    for p in NAMES:
        cells = {}
        for grp in ("fit", "held"):
            t = f = n = 0
            for r in rows:
                if r["group"] != grp:
                    continue
                v, det = predicates(r)[p]
                if v is None:
                    n += 1
                elif v:
                    t += 1
                else:
                    f += 1
                    fails.append((p, r["probe"], r["mode"], det))
            cells[grp] = (t, f, n)
        ok_fit += cells["fit"][0]; tot_fit += cells["fit"][0] + cells["fit"][1]
        ok_held += cells["held"][0]; tot_held += cells["held"][0] + cells["held"][1]

        def fmt(c):
            t, f, n = c
            tot = t + f
            pct = ("%5.1f%%" % (100.0 * t / tot)) if tot else "   n/a"
            return "%3d/%-3d %s (%d n/a)" % (t, tot, pct, n)
        print("%-24s %-26s %-26s" % (p, fmt(cells["fit"]), fmt(cells["held"])))

    print("%-24s %-26s %-26s" % (
        "TOTAL",
        "%3d/%-3d %5.1f%%" % (ok_fit, tot_fit, 100.0 * ok_fit / max(tot_fit, 1)),
        "%3d/%-3d %5.1f%%" % (ok_held, tot_held, 100.0 * ok_held / max(tot_held, 1))))

    if fails:
        print("\nFAILING CELLS (%d):" % len(fails))
        for p, probe, mode, det in fails:
            print("  %-24s %-16s %-12s %s" % (p, probe, mode, det[:150]))
    else:
        print("\nno failing cell")

    # --- the control that decides whether any of this is news ---------------
    print("\n--- CONTROL: what `coff::plan_labels` already predicts ---")
    print("The shipped model says a TU's labels are nothing but one contiguous")
    print("(M,M,T) triple per framed function. If that were already true of the")
    print("held-out shapes there would be no gap for #135 to close.")
    for grp in ("fit", "held"):
        t = f = 0
        rowsg = [r for r in rows if r["group"] == grp]
        for r in rowsg:
            allnums = set(l["num"] for l in r["labels"])
            if not allnums:
                continue
            covered = set()
            for ls in per_function(r).values():
                for x in fn_triples(ls):
                    covered |= set(x)
            (t, f) = (t + 1, f) if covered == allnums else (t, f + 1)
        tot = t + f
        print("  %-5s shipped model accounts for EVERY label in: %d/%d  %s" %
              (grp, t, tot, ("%5.1f%%" % (100.0 * t / tot)) if tot else "n/a"))
    # …and split by EH, which is where the two models are supposed to differ.
    for tag, want_eh in (("EH rows      ", True), ("non-EH rows  ", False)):
        t = f = 0
        for r in rows:
            has_eh = any(l["kind"] == "funclet" for l in r["labels"])
            if has_eh != want_eh:
                continue
            allnums = set(l["num"] for l in r["labels"])
            if not allnums:
                continue
            covered = set()
            for ls in per_function(r).values():
                for x in fn_triples(ls):
                    covered |= set(x)
            (t, f) = (t + 1, f) if covered == allnums else (t, f + 1)
        tot = t + f
        print("  %s shipped model complete: %d/%d  %s" %
              (tag, t, tot, ("%5.1f%%" % (100.0 * t / tot)) if tot else "n/a"))


# ---------------------------------------------------------------------------
# falsify — a 100 % over 312 cells is exactly the shape this project reads as
# success when it is really absence (`docs/ROADMAP.md` §9.1, twelfth instance).
# Each mutation below breaks ONE thing about the allocation and the predicate
# that names it must go red. A predicate that survives its own mutation is
# measuring nothing and is reported as such.
# ---------------------------------------------------------------------------
def mutate(rows, how):
    import copy
    rows = copy.deepcopy(rows)
    for r in rows:
        ls = r["labels"]
        if how == "T_off_by_one":
            # every `.pdata` $T one higher: the triple stops closing.
            for l in ls:
                if l["kind"] == "T" and l["seg"] == ".pdata":
                    l["num"] += 1
        elif how == "funclet_last":
            # the funclet allocated AFTER everything instead of first.
            mx = max([l["num"] for l in ls] or [0])
            for l in ls:
                if l["kind"] == "funclet":
                    l["num"] = mx + 1
                    mx += 1
        elif how == "swap_prolog_end":
            # $M(n) and $M(n+1) exchange their text offsets.
            by = {l["num"]: l for l in ls}
            for n in sorted(by):
                a, b = by.get(n), by.get(n + 1)
                if a and b and a["kind"] == b["kind"] == "M" \
                        and a["where"] == b["where"] == "code":
                    a["text_off"], b["text_off"] = b["text_off"], a["text_off"]
        elif how == "state_above":
            # the `.rdata` state table allocated above the triples.
            mx = max([l["num"] for l in ls] or [0])
            for l in ls:
                if l["kind"] == "T" and l["seg"] == ".rdata":
                    l["num"] = mx + 1
                    mx += 1
        elif how == "funclet_triple_first":
            # the funclet's triple allocated ahead of the main body's.
            for ix in set(l["fn_ix"] for l in ls):
                sub = [l for l in ls if l["fn_ix"] == ix]
                mains = [l for l in sub if l["body"] == "main" and l["kind"] in ("M", "T")]
                fun = [l for l in sub if l["body"] not in ("main", "state", None)
                       and l["kind"] in ("M", "T")]
                if not mains or not fun:
                    continue
                d = max(l["num"] for l in fun) - min(l["num"] for l in mains) + 1
                for l in mains:
                    l["num"] += d
                for l in fun:
                    l["num"] -= d
        elif how == "fn_reverse":
            # functions allocated in REVERSE text order.
            ixs = sorted(set(l["fn_ix"] for l in ls if l["fn_ix"] is not None and l["fn_ix"] >= 0))
            if len(ixs) >= 2:
                base = {i: min(l["num"] for l in ls if l["fn_ix"] == i) for i in ixs}
                rev = dict(zip(ixs, [base[i] for i in reversed(ixs)]))
                for l in ls:
                    if l["fn_ix"] in base:
                        l["num"] = l["num"] - base[l["fn_ix"]] + rev[l["fn_ix"]]
        elif how == "stride4":
            # the triples of one function spaced 4 apart instead of 3.
            for ix in set(l["fn_ix"] for l in ls):
                sub = sorted([l for l in ls if l["fn_ix"] == ix], key=lambda x: x["num"])
                for k, l in enumerate(sub):
                    l["num"] += k
    return rows


def falsify():
    base = [r for r in load() if r["ok"]]
    muts = ["T_off_by_one", "funclet_last", "swap_prolog_end", "state_above",
            "funclet_triple_first", "fn_reverse", "stride4"]
    print("%-24s %s" % ("mutation", "predicates that went RED (cells)"))
    covered = set()
    for m in muts:
        rows = mutate(base, m)
        red = {}
        for r in rows:
            pr = predicates(r)
            for p in NAMES:
                v, _d = pr[p]
                if v is False:
                    red[p] = red.get(p, 0) + 1
        covered |= set(red)
        if red:
            print("%-24s %s" % (m, ", ".join("%s(%d)" % (k, v)
                                             for k, v in sorted(red.items()))))
        else:
            print("%-24s *** NOTHING WENT RED — this mutation is a no-op ***" % m)
    missing = [p for p in NAMES if p not in covered]
    print("\npredicates never falsified by any mutation: %s" %
          (", ".join(missing) if missing else "none — every predicate can go red"))


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    cmd = sys.argv[1]
    if cmd == "gen":
        gen()
    elif cmd == "scan":
        jobs = 6
        if "--jobs" in sys.argv:
            jobs = int(sys.argv[sys.argv.index("--jobs") + 1])
        scan(jobs)
    elif cmd == "dump":
        dump(sys.argv[2] if len(sys.argv) > 2 else None)
    elif cmd == "grade":
        grade()
    elif cmd == "falsify":
        falsify()
    else:
        print("unknown command %r" % cmd)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
