#!/usr/bin/env python3
"""gt_eh_cod.py — boards #133 and #138: the EH record layout and the label-number
gaps, read off c2's own `.cod` assembly listing.

`docs/EH_RECORDS.md` §8.3 decoded the EH `.rdata` from **obj bytes**. The
listing seam (board #132) makes c2 print the same records with **field
boundaries and its own symbol names**, so this script is a *second,
name-carrying source* for a layout already derived — the #136 relationship
(ROADMAP §9.9.3). The byte model is therefore a control that can go red, which a
blank page would not have been.

Two jobs:

  #133  Decode every EH-owned data COMDAT into NAMED fields and check
        TOTALITY — every datum claimed by exactly one field, with any residue
        PRINTED rather than counted. Graded on shapes the layout was not fitted
        on, and cross-checked against §8.3's byte-derived `FuncInfo`.

  #138  §9.12 measured the inter-stage label gaps at 2..11 and 0..3 and refused
        to model them. This measures what governs them, by holding the EH shape
        fixed and varying ONE `LABEL_COUNTER.md` §1.1 surcharge at a time —
        including a surcharge that is measured to cost ZERO (a string literal),
        which is the cell that can refute the model.

Usage:
    scripts/gt_eh_cod.py gen                # write both probe corpora
    scripts/gt_eh_cod.py scan [--jobs N]    # capture .cod for every (probe,mode)
    scripts/gt_eh_cod.py records [probe]    # #133: the decoded records
    scripts/gt_eh_cod.py totality           # #133: A1, the residue, printed
    scripts/gt_eh_cod.py predict            # #133: A2, held-out cells scored
    scripts/gt_eh_cod.py gaps               # #138: the gap table
    scripts/gt_eh_cod.py grade              # everything, scored

Everything lands under `work/weh/` (gitignored). std-lib only; this is tooling,
outside the std-only Rust workspace.
"""

import json
import os
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WORK = os.path.join(ROOT, "work", "weh")
PROBES = os.path.join(WORK, "probes")
CODS = os.path.join(WORK, "cod")
ROWS = os.path.join(WORK, "rows.jsonl")
C2RS = os.path.join(ROOT, "target", "release", "c2rs")

MODES = {
    "EHsc-O1": ["/O1", "/Oi", "/EHsc", "/GS-", "/c"],
    "EHa-O1": ["/O1", "/Oi", "/EHa", "/GS-", "/c"],
    "EHsc-O2": ["/O2", "/EHsc", "/GS-", "/c"],
    "EHsc-Ox": ["/Ox", "/EHsc", "/GS-", "/c"],
}

# ---------------------------------------------------------------------------
# #133 corpus. The axes are STRUCTURAL COUNTS, per ROADMAP §9.13.1 consequence
# 2: a generated axis is only as good as the axes it varies, and a block that
# varied argument VALUES but never ARITY hid a mis-emit. So: number of try
# blocks, nesting depth, catches per try, destructible objects, functions per
# TU — not just the contents of one try.
#
# `fit` = looked at while the layout was written. Everything else is HELD OUT
# and its A2 prediction is registered in PREDICT below before capture.
# ---------------------------------------------------------------------------
COMMON = """
struct T { T(); ~T(); int v; };
struct E1 { int a; };
struct E2 { int b; };
struct V { V(); V(const V&); ~V(); int c; };
int mk(int);
int use(int);
"""

EH_SRC = {
    # ---- FIT ---------------------------------------------------------------
    "fit_1try2catch_dtor_loop": ("fit", """
int f(int n) {
  T t; int s = 0;
  try { s = mk(n); } catch (int e) { s = e; } catch (...) { s = -1; }
  for (int i = 0; i < n; ++i) s += use(i);
  return s + t.v;
}
"""),
    "fit_nested2": ("fit", """
int f(int n) {
  T a; int s = 0;
  try {
    T b;
    try { s = mk(n); }
    catch (E1& e) { s = e.a; }
    catch (const E2& e) { s = e.b; }
    s += b.v;
  } catch (int e) { s = e; }
  catch (...) { s = -2; }
  return s + a.v;
}
"""),

    # ---- HELD OUT: the destructible-object axis, no try at all -------------
    "h_dtor1": ("held", """
int f(int n) { T a; return mk(n) + a.v; }
"""),
    "h_dtor2": ("held", """
int f(int n) { T a; T b; return mk(n) + a.v + b.v; }
"""),
    "h_dtor3": ("held", """
int f(int n) { T a; T b; T c; return mk(n) + a.v + b.v + c.v; }
"""),

    # ---- HELD OUT: the try-block-COUNT axis, sequential (not nested) -------
    "h_try1": ("held", """
int f(int n) {
  int s = 0;
  try { s = mk(n); } catch (int e) { s = e; }
  return s;
}
"""),
    "h_try2seq": ("held", """
int f(int n) {
  int s = 0;
  try { s = mk(n); } catch (int e) { s = e; }
  try { s += mk(n + 1); } catch (int e) { s -= e; }
  return s;
}
"""),
    "h_try3seq": ("held", """
int f(int n) {
  int s = 0;
  try { s = mk(n); } catch (int e) { s = e; }
  try { s += mk(n + 1); } catch (int e) { s -= e; }
  try { s += mk(n + 2); } catch (int e) { s ^= e; }
  return s;
}
"""),

    # ---- HELD OUT: the nesting-DEPTH axis ---------------------------------
    "h_nest3": ("held", """
int f(int n) {
  int s = 0;
  try {
    try {
      try { s = mk(n); } catch (E1& e) { s = e.a; }
    } catch (E2& e) { s = e.b; }
  } catch (int e) { s = e; }
  return s;
}
"""),

    # ---- HELD OUT: the catches-per-try axis -------------------------------
    "h_catch4": ("held", """
int f(int n) {
  int s = 0;
  try { s = mk(n); }
  catch (V v) { s = v.c; }
  catch (const E1& e) { s = e.a; }
  catch (E2& e) { s = e.b; }
  catch (...) { s = -1; }
  return s;
}
"""),
    "h_catch_ptr": ("held", """
int f(int n) {
  int s = 0;
  try { s = mk(n); } catch (const char* volatile p) { s = p ? 1 : 0; }
  return s;
}
"""),

    # ---- HELD OUT: functions-per-TU axis -----------------------------------
    "h_2fn": ("held", """
int f(int n) { int s = 0; try { s = mk(n); } catch (int e) { s = e; } return s; }
int g(int n) { int s = 0; try { s = mk(n); } catch (E1& e) { s = e.a; } return s; }
"""),
    "h_3fn_mixed": ("held", """
int p(int x) { return x + 1; }
int f(int n) { T a; return mk(n) + a.v; }
int g(int n) { int s = 0; try { s = mk(n); } catch (...) { s = -1; } return s; }
"""),

    # ---- HELD OUT: dtor AND try together, and rethrow ---------------------
    "h_dtor2_try2catch": ("held", """
int f(int n) {
  T a; T b; int s = 0;
  try { s = mk(n); } catch (int e) { s = e; } catch (...) { s = -1; }
  return s + a.v + b.v;
}
"""),
    "h_rethrow": ("held", """
int f(int n) {
  int s = 0;
  try { s = mk(n); } catch (int e) { if (e > 0) throw; s = e; }
  return s;
}
"""),
}

# ---------------------------------------------------------------------------
# #133 / A2 — registered BEFORE capture. Per held-out probe, predicted from the
# SOURCE alone: nTryBlocks, nIPMapEntries, maxState, the number of __catchsym$
# arrays, the HandlerType count in each (as a sorted list), and the number of
# funclets. `None` = declined rather than guessed; a declined cell scores 0 and
# is reported as declined, never as a pass.
# ---------------------------------------------------------------------------
PREDICT = {
    #                    nTry  maxState  nIP  nCatchsym  handlers   funclets
    "h_dtor1":          dict(nTryBlocks=0, maxState=1, nIPMapEntries=2,
                             nCatchsym=0, handlers=[], nFunclets=1),
    "h_dtor2":          dict(nTryBlocks=0, maxState=2, nIPMapEntries=4,
                             nCatchsym=0, handlers=[], nFunclets=2),
    "h_dtor3":          dict(nTryBlocks=0, maxState=3, nIPMapEntries=6,
                             nCatchsym=0, handlers=[], nFunclets=3),
    "h_try1":           dict(nTryBlocks=1, maxState=1, nIPMapEntries=None,
                             nCatchsym=1, handlers=[1], nFunclets=1),
    "h_try2seq":        dict(nTryBlocks=2, maxState=2, nIPMapEntries=None,
                             nCatchsym=2, handlers=[1, 1], nFunclets=2),
    "h_try3seq":        dict(nTryBlocks=3, maxState=3, nIPMapEntries=None,
                             nCatchsym=3, handlers=[1, 1, 1], nFunclets=3),
    "h_nest3":          dict(nTryBlocks=3, maxState=3, nIPMapEntries=None,
                             nCatchsym=3, handlers=[1, 1, 1], nFunclets=3),
    "h_catch4":         dict(nTryBlocks=1, maxState=2, nIPMapEntries=None,
                             nCatchsym=1, handlers=[4], nFunclets=4),
    "h_catch_ptr":      dict(nTryBlocks=1, maxState=1, nIPMapEntries=None,
                             nCatchsym=1, handlers=[1], nFunclets=1),
    "h_2fn":            dict(nTryBlocks=1, maxState=1, nIPMapEntries=None,
                             nCatchsym=1, handlers=[1], nFunclets=1),
    "h_3fn_mixed":      dict(nTryBlocks=0, maxState=1, nIPMapEntries=2,
                             nCatchsym=0, handlers=[], nFunclets=1),
    "h_dtor2_try2catch": dict(nTryBlocks=1, maxState=4, nIPMapEntries=None,
                              nCatchsym=1, handlers=[2], nFunclets=4),
    "h_rethrow":        dict(nTryBlocks=1, maxState=1, nIPMapEntries=None,
                             nCatchsym=1, handlers=[1], nFunclets=1),
}
# For multi-function TUs the prediction above is for the FIRST EH function in
# `.text` order (`h_2fn`'s `f`, `h_3fn_mixed`'s `f`).

# ---------------------------------------------------------------------------
# #138 corpus. ONE axis moves per probe against `g_base`. The EH shape is held
# fixed at "one destructible local, no try" — `EH_RECORDS.md` §9.8's own shape,
# where its model `G = 4 + Sum(minting surcharges)` is claimed exact on 25/27.
# ---------------------------------------------------------------------------
GAP_COMMON = """
struct T { T(); ~T(); int v; };
int mk(int);
int use(int);
int mk2(int, int);
"""

GAP_SRC = {
    # the baseline: 1 destructible local, no try.
    "g_base": """
int f(int n) { T a; return mk(n) + a.v; }
""",
    # +_fltused: the first FP-touching function in the TU.
    "g_flt": """
int f(int n) { T a; float x = (float)n; return mk((int)(x + x)) + a.v; }
""",
    # +a newly pooled FP constant (distinct (bits,width)).
    "g_pool": """
int f(int n) { T a; float x = (float)n * 2.5f; return mk((int)x) + a.v; }
""",
    # +a loop. LABEL_COUNTER.md §2.1: costs slots and MINTS NOTHING.
    "g_loop": """
int f(int n) { T a; int s = 0; for (int i = 0; i < n; ++i) s += use(i); return s + a.v; }
""",
    # +a signed relational over two call results. §1.1: +2, MINTS NOTHING.
    "g_cmprr": """
int f(int n) { T a; int s = (mk(n) < use(n)) ? 1 : 0; return s + a.v; }
""",
    # THE CONTROL. A string literal makes an .rdata COMDAT and a `??_C@` symbol
    # and costs ZERO slots (§2.1). If G moves here, "G is the §1.1 surcharge
    # block" is REFUTED. This is the cell that can go red.
    "g_str": """
const char* gp;
int f(int n) { T a; gp = "hello"; return mk(n) + a.v; }
""",
    # +a second distinct gprlr width, introduced by a SECOND function.
    "g_gpr2": """
int wide(int a,int b,int c,int d,int e,int f2,int g,int h) { return mk2(a+b+c+d, e+f2+g+h); }
int f(int n) { T a; return mk(n) + a.v; }
""",
    # led: an EH function already in the TU, so __CxxFrameHandler is paid.
    "g_led": """
int lead(int n) { T z; return mk(n) + z.v; }
int f(int n) { T a; return mk(n) + a.v; }
""",
    # phases that emit nothing: an unreferenced static, which c2 discards
    # (ROADMAP §9.5's `globally unreferenced` disjunct).
    "g_static": """
static int dead(int x) { return x * 3 + 1; }
int f(int n) { T a; return mk(n) + a.v; }
""",
    "g_static4": """
static int d1(int x) { return x * 3 + 1; }
static int d2(int x) { return x * 5 + 2; }
static int d3(int x) { return x * 7 + 3; }
static int d4(int x) { return x * 9 + 4; }
int f(int n) { T a; return mk(n) + a.v; }
""",
    # the try/catch shape, to test whether G behaves as it does with no try.
    "g_try": """
int f(int n) { T a; int s = 0; try { s = mk(n); } catch (int e) { s = e; } return s + a.v; }
""",
}

# bodies inlined away: k identical always-inlinable callees, EH shape fixed.
for _k in range(0, 5):
    _calls = " + ".join("inl%d(n)" % _i for _i in range(_k)) or "0"
    _defs = "\n".join("static int inl%d(int x) { return x + %d; }" % (_i, _i)
                      for _i in range(_k))
    GAP_SRC["g_inl%d" % _k] = """
%s
int f(int n) { T a; return mk(n) + a.v + (%s); }
""" % (_defs, _calls)


def gen():
    os.makedirs(PROBES, exist_ok=True)
    n = 0
    for name, (_g, src) in EH_SRC.items():
        with open(os.path.join(PROBES, name + ".cpp"), "w") as fh:
            fh.write(COMMON.lstrip() + src.lstrip())
        n += 1
    for name, src in GAP_SRC.items():
        with open(os.path.join(PROBES, name + ".cpp"), "w") as fh:
            fh.write(GAP_COMMON.lstrip() + src.lstrip())
        n += 1
    print("wrote %d probes to %s" % (n, PROBES))


# ---------------------------------------------------------------------------
# parse
# ---------------------------------------------------------------------------
RE_PROC = re.compile(r"^(\S+)\s+PROC\s+NEAR")
RE_ENDP = re.compile(r"^(\S+)\s+ENDP")
RE_SEG = re.compile(r"^(\S+)\s+SEGMENT")
RE_SEG_DOT = re.compile(r"^\s+(\.[A-Za-z$0-9_]+)\s*$")
RE_INSN = re.compile(r"^\s+([0-9a-f]{5,8})\s+([0-9a-f]{8})\s")
# a data definition that OPENS a named record:  `SYM DD  value`
RE_NAMED_DATUM = re.compile(r"^(\S+)\s+(DD|DW|DB|DQ)\s+(.*)$")
# a continuation datum:  `\tDD\tvalue`
RE_CONT_DATUM = re.compile(r"^\s+(DD|DW|DB|DQ)\s+(.*)$")
RE_ORG = re.compile(r"^\s+ORG\s+\$\+(\d+)\s*$")
# every label DEFINITION that carries a number from c2's TU-wide counter
RE_NUMBERED_LABEL = re.compile(r"^(\$M|\$T)(\d+):")
RE_FUNCLET = re.compile(r"^(__catch\$|__unwind\$|__tryend\$)(\d+):")
# a numbered label that is DEFINED as data (`$T2606\tDD\t?f@@YAHH@Z`)
RE_DATA_LABEL = re.compile(r"^(\$M|\$T)(\d+)\s+DD\s+(\S+)")
# per-function local labels, a DIFFERENT and small number space
RE_LOCAL_LABEL = re.compile(r"^(\$LN|\$LL|\$L)(\d+)@(\S+):")


def parse_cod(path):
    """One listing -> (a) the named data records in emission order, (b) every
    label carrying a TU-counter number, (c) the local `$LN`/`$LL` labels, which
    are a separate space, (d) the PROC list."""
    with open(path, "r", errors="replace") as fh:
        lines = fh.read().splitlines()

    records = []      # {name, seg, data:[str], org:[int]}
    cur = None
    seg = None
    labels = []       # TU-counter labels
    locals_ = []      # $LN/$LL
    funcs = []
    fn = None
    fn_ix = -1
    body = None
    pending = []
    last_off = None

    def flush(off):
        for p in pending:
            p["text_off"] = off
        del pending[:]

    def close():
        nonlocal cur
        if cur is not None:
            records.append(cur)
            cur = None

    for raw in lines:
        line = raw.rstrip("\n")
        if line.startswith(";"):
            continue
        m = RE_SEG.match(line)
        if m:
            seg = m.group(1)
            close()
            continue
        m = RE_SEG_DOT.match(line)
        if m:
            seg = m.group(1)
            close()
            continue
        m = RE_PROC.match(line)
        if m:
            close()
            fn = m.group(1)
            fn_ix += 1
            funcs.append(fn)
            body = "main"
            seg = ".code"
            last_off = None
            continue
        m = RE_ENDP.match(line)
        if m:
            # gt_label_cod.py instrument defect 1: a body-end `$M` sits just
            # before ENDP and belongs to THIS function, at one past its last
            # instruction — not to the next function, which under /Gy restarts
            # its offsets at 0.
            flush((last_off + 4) if last_off is not None else None)
            fn, body, last_off = None, None, None
            continue
        if line.strip() == ".endprolog":
            continue
        m = RE_FUNCLET.match(line)
        if m:
            name = m.group(1) + m.group(2)
            body = name
            labels.append(dict(num=int(m.group(2)), kind="funclet", where="code",
                               name=name, fn=fn, fn_ix=fn_ix, body=name,
                               text_off=None))
            pending.append(labels[-1])
            continue
        m = RE_NUMBERED_LABEL.match(line)
        if m:
            labels.append(dict(num=int(m.group(2)),
                               kind={"$M": "M", "$T": "T"}[m.group(1)],
                               where="code", name=m.group(1) + m.group(2),
                               fn=fn, fn_ix=fn_ix, body=body, text_off=None))
            pending.append(labels[-1])
            continue
        m = RE_LOCAL_LABEL.match(line)
        if m:
            locals_.append(dict(prefix=m.group(1), num=int(m.group(2)),
                                fn=m.group(3)))
            continue
        m = RE_INSN.match(line)
        if m:
            last_off = int(m.group(1), 16)
            flush(last_off)
            continue
        m = RE_ORG.match(line)
        if m and cur is not None:
            cur["org"].append(int(m.group(1)))
            cur["data"].append("ORG+%s" % m.group(1))
            continue
        m = RE_DATA_LABEL.match(line)
        if m:
            # `$T2606 DD ?f@@YAHH@Z` in .pdata, or `$T2601 DD $M2591` in the EH
            # .rdata: a numbered label that OPENS a data record. It is both.
            labels.append(dict(num=int(m.group(2)),
                               kind={"$M": "M", "$T": "T"}[m.group(1)],
                               where="data", name=m.group(1) + m.group(2),
                               fn=None, fn_ix=None, body=None,
                               describes=m.group(3).rstrip("H,"),
                               text_off=None))
            close()
            cur = dict(name=m.group(1) + m.group(2), seg=seg,
                       data=[m.group(3).strip()], org=[])
            continue
        m = RE_NAMED_DATUM.match(line)
        if m and fn is None:
            close()
            cur = dict(name=m.group(1), seg=seg, data=[m.group(3).strip()],
                       org=[])
            continue
        m = RE_CONT_DATUM.match(line)
        if m and cur is not None:
            cur["data"].append(m.group(2).strip())
            continue
    close()
    return dict(records=records, labels=labels, locals=locals_, funcs=funcs)


# ---------------------------------------------------------------------------
# #133 — decode the named records into named FIELDS.
# ---------------------------------------------------------------------------
def n_of(tok):
    """A `DD` operand as an int when it is a literal, else None (a relocation)."""
    t = tok.strip().rstrip(",")
    if re.fullmatch(r"0[0-9a-fA-F]*H", t):
        v = int(t[:-1], 16)
        return v - (1 << 32) if v >= (1 << 31) else v
    if re.fullmatch(r"\d+", t):
        return int(t)
    return None


FUNCINFO_FIELDS = ["magic", "maxState", "pUnwindMap", "nTryBlocks",
                   "pTryBlockMap", "nIPMapEntries", "pIPtoStateMap",
                   "pESTypeList", "EHFlags"]


def decode(rec):
    """One named record -> (kind, list of (field, value)) and any UNCLAIMED
    residue, which is returned as data rather than as a count."""
    nm, d = rec["name"], rec["data"]
    if nm.startswith("__ehfuncinfo$"):
        fields = [(FUNCINFO_FIELDS[i] if i < len(FUNCINFO_FIELDS)
                   else "UNCLAIMED[%d]" % i, v) for i, v in enumerate(d)]
        return "FuncInfo", fields
    if nm.startswith("__unwindtable$"):
        out, i = [], 0
        while i + 1 < len(d):
            out += [("[%d].toState" % (i // 2), d[i]),
                    ("[%d].action" % (i // 2), d[i + 1])]
            i += 2
        if i < len(d):
            out.append(("UNCLAIMED[%d]" % i, d[i]))
        return "UnwindMap", out
    if nm.startswith("__catchsym$"):
        out, i = [], 0
        F = ["adjectives", "pType", "dispCatchObj", "addressOfHandler"]
        while i + 3 < len(d):
            for j, f in enumerate(F):
                out.append(("[%d].%s" % (i // 4, f), d[i + j]))
            i += 4
        while i < len(d):
            out.append(("UNCLAIMED[%d]" % i, d[i]))
            i += 1
        return "HandlerTypeArray", out
    if nm.startswith("__tryblocktable$"):
        out, i = [], 0
        F = ["tryLow", "tryHigh", "catchHigh", "nCatches", "pHandlerArray"]
        while i + 4 < len(d):
            for j, f in enumerate(F):
                out.append(("[%d].%s" % (i // 5, f), d[i + j]))
            i += 5
        while i < len(d):
            out.append(("UNCLAIMED[%d]" % i, d[i]))
            i += 1
        return "TryBlockMap", out
    if re.fullmatch(r"\$T\d+", nm) and rec["seg"] == ".rdata":
        out, i = [], 0
        while i + 1 < len(d):
            out += [("[%d].ip" % (i // 2), d[i]),
                    ("[%d].state" % (i // 2), d[i + 1])]
            i += 2
        if i < len(d):
            out.append(("UNCLAIMED[%d]" % i, d[i]))
        return "IpToStateMap", out
    if re.fullmatch(r"\$T\d+", nm) and rec["seg"] == ".pdata":
        f = [("BeginAddress", d[0])]
        f += [("unwindWord", d[1])] if len(d) > 1 else []
        f += [("UNCLAIMED[%d]" % i, v) for i, v in enumerate(d[2:], 2)]
        return "PdataEntry", f
    if nm.startswith("??_R0"):
        f = [("pVFTable", d[0])]
        f += [("spare", d[1])] if len(d) > 1 else []
        f += [("name", d[2])] if len(d) > 2 else []
        f += [("UNCLAIMED[%d]" % i, v) for i, v in enumerate(d[3:], 3)]
        return "TypeDescriptor", f
    return None, []


EH_RECORD_PREFIXES = ("__ehfuncinfo$", "__unwindtable$", "__catchsym$",
                      "__tryblocktable$", "??_R0")


def eh_records(p):
    """Every record this lane claims is part of the EH record set."""
    out = []
    for r in p["records"]:
        if r["name"].startswith(EH_RECORD_PREFIXES):
            out.append(r)
        elif re.fullmatch(r"\$T\d+", r["name"]) and r["seg"] in (".rdata", ".pdata"):
            out.append(r)
    return out


# ---------------------------------------------------------------------------
# #138 — the allocation spans and the gaps between them.
# ---------------------------------------------------------------------------
def gap_analysis(p):
    """Per function: the funclet block, the EH-state $M block, the state-table
    $T, the triples — and the UNNAMED slots between them."""
    labels = sorted([l for l in p["labels"]], key=lambda l: l["num"])
    if not labels:
        return None
    named = set(l["num"] for l in labels)
    lo, hi = labels[0]["num"], labels[-1]["num"]
    unnamed = [n for n in range(lo, hi + 1) if n not in named]

    funclets = [l["num"] for l in labels if l["kind"] == "funclet"]
    # the ip2state $M block = the $M labels referenced by the .rdata $T record
    ip_ms = []
    for r in p["records"]:
        if re.fullmatch(r"\$T\d+", r["name"]) and r["seg"] == ".rdata":
            for tok in r["data"]:
                m = re.fullmatch(r"\$M(\d+)", tok.strip().rstrip(","))
                if m:
                    ip_ms.append(int(m.group(1)))
    state_t = [int(r["name"][2:]) for r in p["records"]
               if re.fullmatch(r"\$T\d+", r["name"]) and r["seg"] == ".rdata"]
    pdata_t = sorted(int(r["name"][2:]) for r in p["records"]
                     if re.fullmatch(r"\$T\d+", r["name"]) and r["seg"] == ".pdata")

    triple_ms = sorted(n for n in named
                       if n not in funclets and n not in ip_ms
                       and n not in state_t and n not in pdata_t)

    out = dict(lo=lo, hi=hi, span=hi - lo + 1, n_named=len(named),
               n_unnamed=len(unnamed), unnamed=unnamed,
               funclets=sorted(funclets), ip_ms=sorted(ip_ms),
               state_t=sorted(state_t), pdata_t=pdata_t,
               triple_ms=triple_ms,
               locals=sorted(set((l["prefix"], l["num"]) for l in p["locals"])))
    # the two gaps §9.12 measured
    if funclets and ip_ms:
        out["G_funclet_to_ipM"] = min(ip_ms) - max(funclets) - 1
    if state_t and triple_ms:
        out["G_stateT_to_triple"] = min(triple_ms) - max(state_t) - 1
    return out


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
                    err=(r.stderr or r.stdout).strip()[:300])
    p = parse_cod(out)
    grp = EH_SRC[name][0] if name in EH_SRC else "gap"
    return dict(probe=name, mode=mode, ok=True, group=grp, flags=flags,
                records=p["records"], labels=p["labels"], locals=p["locals"],
                funcs=p["funcs"], gaps=gap_analysis(p))


def scan(jobs=6):
    os.makedirs(CODS, exist_ok=True)
    work = []
    for name in EH_SRC:
        for mode, flags in MODES.items():
            work.append((name, mode, flags))
    for name in GAP_SRC:
        for mode in ("EHsc-O1",):
            work.append((name, mode, MODES[mode]))
    with ThreadPoolExecutor(max_workers=jobs) as ex:
        rows = list(ex.map(one, work))
    with open(ROWS, "w") as fh:
        for r in rows:
            fh.write(json.dumps(r) + "\n")
    ok = sum(1 for r in rows if r["ok"])
    print("captured %d/%d listings" % (ok, len(rows)))
    for r in rows:
        if not r["ok"]:
            print("  FAIL %s %s: %s" % (r["probe"], r["mode"], r["err"]))


def load():
    with open(ROWS) as fh:
        return [json.loads(l) for l in fh if l.strip()]


# ---------------------------------------------------------------------------
# report
# ---------------------------------------------------------------------------
def cmd_records(argv):
    want = argv[0] if argv else None
    for r in load():
        if not r["ok"] or (want and r["probe"] != want):
            continue
        if r["probe"] in GAP_SRC and not want:
            continue
        print("=== %s [%s] %s" % (r["probe"], r["mode"], " ".join(r["flags"])))
        for rec in eh_records(r):
            kind, fields = decode(rec)
            print("  %-34s %-16s seg=%s  %d data" %
                  (rec["name"], kind or "?", rec["seg"], len(rec["data"])))
            for f, v in fields:
                print("      %-22s %s" % (f, v))
        print()


def cmd_totality(argv):
    """A1 — every datum claimed by a named field, residue PRINTED."""
    tot = {"fit": [0, 0], "held": [0, 0]}
    residue = []
    for r in load():
        if not r["ok"] or r["probe"] in GAP_SRC:
            continue
        grp = r["group"]
        for rec in eh_records(r):
            kind, fields = decode(rec)
            if kind is None:
                residue.append((r["probe"], r["mode"], rec["name"],
                                "NO DECODER", rec["data"]))
                tot[grp][1] += len(rec["data"])
                continue
            for f, v in fields:
                tot[grp][1] += 1
                if f.startswith("UNCLAIMED"):
                    residue.append((r["probe"], r["mode"], rec["name"], f, v))
                else:
                    tot[grp][0] += 1
    print("A1 TOTALITY — data claimed by a named field")
    for g in ("fit", "held"):
        c, t = tot[g]
        print("  %-6s %6d / %6d claimed   residue %d"
              % (g, c, t, t - c))
    print("\nRESIDUE, printed (never summarised):")
    if not residue:
        print("  (none)")
    for probe, mode, nm, f, v in residue:
        print("  %-24s %-9s %-34s %-14s %s" % (probe, mode, nm, f, v))


def _first_eh_fn_records(r):
    """The records belonging to the FIRST EH function in .text order."""
    out = {}
    for rec in eh_records(r):
        nm = rec["name"]
        for pre in ("__ehfuncinfo$", "__unwindtable$", "__tryblocktable$"):
            if nm.startswith(pre):
                out.setdefault(pre, rec)
    out["catchsyms"] = [rec for rec in eh_records(r)
                        if rec["name"].startswith("__catchsym$")]
    return out


def observed(r):
    """The A2 cells, read out of the .cod."""
    recs = _first_eh_fn_records(r)
    fi = recs.get("__ehfuncinfo$")
    if fi is None:
        return None
    d = fi["data"]
    o = dict(maxState=n_of(d[1]) if len(d) > 1 else None,
             nTryBlocks=n_of(d[3]) if len(d) > 3 else None,
             nIPMapEntries=n_of(d[5]) if len(d) > 5 else None,
             EHFlags=n_of(d[8]) if len(d) > 8 else None)
    # restrict to the first function's own catchsym arrays
    fn = fi["name"][len("__ehfuncinfo$"):]
    mine = [c for c in recs["catchsyms"]
            if c["name"][len("__catchsym$"):].startswith(fn + "$")]
    o["nCatchsym"] = len(mine)
    o["handlers"] = sorted(len(c["data"]) // 4 for c in mine)
    o["nFunclets"] = len([l for l in r["labels"] if l["kind"] == "funclet"
                          and l["fn"] == fn])
    return o


def cmd_predict(argv):
    """A2 — held-out cells, scored against PREDICT (registered pre-capture)."""
    cells = hits = declined = 0
    print("A2 — held-out structural counts, predicted from SOURCE before capture")
    print("     graded at EHsc-O1 (the workload profile)")
    print("  %-20s %-16s %8s %8s  %s" % ("probe", "cell", "pred", "obs", ""))
    for r in load():
        if not r["ok"] or r.get("group") != "held" or r["mode"] != "EHsc-O1":
            continue
        pred = PREDICT.get(r["probe"])
        obs = observed(r)
        if pred is None or obs is None:
            print("  %-20s  NO OBSERVATION" % r["probe"])
            continue
        for k, pv in pred.items():
            ov = obs.get(k)
            cells += 1
            if pv is None:
                declined += 1
                mark = "DECLINED"
            elif pv == ov:
                hits += 1
                mark = "hit"
            else:
                mark = "MISS"
            print("  %-20s %-16s %8s %8s  %s"
                  % (r["probe"], k, pv, ov, mark))
    print("\n  %d/%d exact = %.1f %%   (%d declined, scored 0)"
          % (hits, cells, 100.0 * hits / cells if cells else 0.0, declined))


def cmd_gaps(argv):
    rows = [r for r in load() if r["ok"] and r["probe"] in GAP_SRC]
    rows.sort(key=lambda r: r["probe"])
    print("#138 — the label-number gaps, one axis moved per probe")
    print("  %-12s %6s %6s %6s %6s %8s %8s  %s"
          % ("probe", "lo", "hi", "span", "named", "G_fn>M", "G_T>tri", "unnamed"))
    for r in rows:
        g = r["gaps"]
        if not g:
            print("  %-12s (no labels)" % r["probe"])
            continue
        print("  %-12s %6d %6d %6d %6d %8s %8s  %s"
              % (r["probe"], g["lo"], g["hi"], g["span"], g["n_named"],
                 g.get("G_funclet_to_ipM", "-"), g.get("G_stateT_to_triple", "-"),
                 g["unnamed"]))
    print()
    print("  the SEPARATE local-label space (`$LN`/`$LL`), which is NOT the TU counter:")
    for r in rows:
        g = r["gaps"]
        if g and g["locals"]:
            print("  %-12s %s" % (r["probe"], g["locals"]))


def cmd_grade(argv):
    cmd_totality([])
    print()
    cmd_predict([])
    print()
    cmd_gaps([])


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return
    cmd, argv = sys.argv[1], sys.argv[2:]
    if cmd == "gen":
        gen()
    elif cmd == "scan":
        j = 6
        if "--jobs" in argv:
            j = int(argv[argv.index("--jobs") + 1])
        scan(j)
    elif cmd == "records":
        cmd_records(argv)
    elif cmd == "totality":
        cmd_totality(argv)
    elif cmd == "predict":
        cmd_predict(argv)
    elif cmd == "gaps":
        cmd_gaps(argv)
    elif cmd == "grade":
        cmd_grade(argv)
    else:
        print(__doc__)


if __name__ == "__main__":
    main()
