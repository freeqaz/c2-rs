#!/usr/bin/env python3
"""table.py — re-measure EVERY row of `docs/LABEL_COUNTER.md` §4.2.1 against the
oracle, as a SERIES, with the seed-cancelling instrument.

Lane **w-labeltable**. Control: `work/w-labeltable/PREREG.md`, committed before
the first `cl.exe`.

# Why this exists

§4.2.1 publishes 17 leaf-loop surcharges. Four lanes have measured that table
wrong and always in one direction — the one that makes a fence look **dearer to
lift than it is** (#3091/#3126, #3148). `w-slots` left the re-audit as its top
found-and-not-taken because `work/w-slots/lead.py` made it one command.

# The method correction (#3147)

`w-slots` read a charge out of one cell's obj and the objs said **3**; the true
charge is **2** and the third slot is the TU's. **Only a SERIES separates a
per-function charge from a per-TU constant.** So every row here is measured at
`n = 1, 2, 3` copies of the probe body and reported as the fit `L(n) = k*n + c`.
A row publishes `k` only when `c == 0`; a non-zero `c` is named, never folded in.

# The two instruments, and why both are run

    LEAD    `[P x n, z9]`, z9 framed and LAST.  lead = real $M(z9) - base,
            base = counter + 9 + 3*segs + nleaf.  Each TU's OWN `.gl` counter is
            subtracted, so the seed cancels INSIDE the TU (#3148).
    STRIDE  `a0 . P . a1 . a2`, anchors framed.  stride = first(a1) - first(a0)
            - base, base measured in-obj as first(a2) - first(a1).  This is the
            instrument §4.2.1 was measured with.

`coff::plan_labels` charges a leaf `label_lead + 1`, so `stride == lead + 1` and
**§4.2.1's `surcharge` column and the LEAD are the same quantity.** Both are
seed-free; they should agree. Where they do not, this prints both columns and
picks neither (PREREG F4).

# The bridge cell

`?HashString` is the ONE leaf loop in this repo whose charge the oracle has
already settled (`w-fenceb`: lead **2**, three mutants red, control green). It is
run through BOTH instruments. If this script cannot reproduce that 2, every row
it prints is suspect — PREREG F3, and the lane says FAILED rather than publish.

    work/w-labeltable/table.py                     # the 17 rows + series, /O1
    work/w-labeltable/table.py --mode '/Ox /GS- /c'
    work/w-labeltable/table.py --bdnz              # w-bdnz's 8 cross-TU rows
    work/w-labeltable/table.py --framed            # SS4's 6 framed rows
    work/w-labeltable/table.py --rows leaf-ptrwalk hashstring

Exit status is non-zero only if a CONTROL failed. It is never non-zero because a
published number did not reproduce — the table is the result.
"""

import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "scripts"))
sys.path.insert(0, os.path.join(REPO, "work", "w-fenceb"))
import gt_label_stride as G  # noqa: E402
from gt_dump import Obj  # noqa: E402
import labelil as L  # noqa: E402

LABEL_SEED_GAP = 9
FRAMED_DECL = "int gz(int);"
FRAMED_TAIL = "int z9(int a){ return gz(a)+7; }"

# ---------------------------------------------------------------------------
# The 17 rows of §4.2.1, transcribed VERBATIM from `work/w-loop/loopcost.py`'s
# Q1 list (the instrument that produced the published table), with the probe's
# name parameterized so the series can put `n` copies in one TU. `%d` is the
# copy index; nothing else is changed.
#
# The third column is the PUBLISHED surcharge. It is quoted here so the
# disagreement is printed by the instrument rather than assembled by hand.
# ---------------------------------------------------------------------------
ROWS = [
    ("leaf-none", "int P%d(int a){ return a+1; }", 0),
    ("leaf-if", "int P%d(int a){ if (a) return 5; return a+1; }", 0),
    ("leaf-while",
     "int P%d(int a){ int r=0; while (a) { r=r+a; a=a-1; } return r; }", 2),
    ("leaf-dowhile",
     "int P%d(int a){ int r=0; do { r=r+a; a=a-1; } while (a); return r; }", 1),
    ("leaf-for",
     "int P%d(int a){ int r=0; for (int i=0;i<a;i++) r=r+i; return r; }", 2),
    ("leaf-for-k",
     "int P%d(int a){ int r=0; for (int i=0;i<10;i++) r=r+a; return r; }", 2),
    ("leaf-for-stride",
     "int P%d(int a){ int r=0; for (int i=0;i<a;i+=3) r=r+i; return r; }", 2),
    ("leaf-for-down",
     "int P%d(int a){ int r=0; for (int i=a;i>0;i--) r=r+i; return r; }", 2),
    ("leaf-for-cont",
     "int P%d(int a){ int r=0; for (int i=0;i<a;i++){ if (i==3) continue;"
     " r=r+i; } return r; }", 2),
    ("leaf-for-live",
     "int P%d(int a){ int r=0; int i; for (i=0;i<a;i++) r=r+i; return r+i; }", 2),
    ("leaf-idxload",
     "int P%d(const int* v,int n){ int r=0; for (int i=0;i<n;i++) r=r+v[i];"
     " return r; }", 2),
    ("leaf-forever",
     "int P%d(int a){ int r=0; for (;;) { r=r+a; a=a-1; if (!a) break; }"
     " return r; }", 3),
    ("leaf-for-break",
     "int P%d(int a){ int r=0; for (int i=0;i<a;i++){ r=r+i; if (r>100) break; }"
     " return r; }", 3),
    ("leaf-ptrwalk",
     "int P%d(const char* s){ int r=0; for (const char* p=s; *p; p++) r=r+*p;"
     " return r; }", 3),
    ("leaf-for2",
     "int P%d(int a){ int r=0; for (int i=0;i<a;i++) r=r+i;"
     " for (int j=0;j<a;j++) r=r+j; return r; }", 4),
    ("leaf-fornest",
     "int P%d(int a){ int r=0; for (int i=0;i<a;i++)"
     " for (int j=0;j<a;j++) r=r+j; return r; }", 4),
    ("leaf-goto-back",
     "int P%d(int a){ int r=0; top%d: r=r+a; a=a-1; if (a) goto top%d;"
     " return r; }", 1),
]

# The BRIDGE CELL. `?HashString`'s body, transcribed verbatim from
# `fixtures/cpp/whash_ptr_walk_loop.cpp` — the one leaf loop whose charge the
# ORACLE has settled (`w-fenceb` §3.3: lead 2, three mutants red). Its published
# number here is the shipped `label_lead` term, not a `LABEL_COUNTER.md` row.
BRIDGE = [
    ("hashstring",
     "int P%d(const char *str, int i) {\n"
     "    int ret = 0;\n"
     "    for (unsigned char *u = (unsigned char *)str; *u != 0; u++) {\n"
     "        ret = (*u + ret * 0x7F) % i;\n"
     "    }\n"
     "    return ret;\n"
     "}", 2),
]

# THE LADDER between the two pointer walks. §4.2.1's `leaf-ptrwalk` charges 3
# and `?HashString` charges 2, and both numbers are measured, so the row is not
# "one high" — the two are different shapes. This ladder walks one from the
# other a token at a time so the separating term is MEASURED rather than
# asserted. The `published` column is the §4.2.1 row's 3 for the whole ladder,
# which is what makes a step visible as a disagreement.
PTRWALK_LADDER = [
    ("pw0-signed",
     "int P%d(const char* s){ int r=0; for (const char* p=s; *p; p++) r=r+*p;"
     " return r; }", 3),
    ("pw1-unsigned",
     "int P%d(const unsigned char* s){ int r=0;"
     " for (const unsigned char* p=s; *p; p++) r=r+*p; return r; }", 3),
    ("pw2-ne0",
     "int P%d(const unsigned char* s){ int r=0;"
     " for (const unsigned char* p=s; *p != 0; p++) r=r+*p; return r; }", 3),
    ("pw3-mul",
     "int P%d(const unsigned char* s){ int r=0;"
     " for (const unsigned char* p=s; *p != 0; p++) r=(*p + r*0x7F);"
     " return r; }", 3),
    ("pw4-mod",
     "int P%d(const unsigned char* s, int i){ int r=0;"
     " for (const unsigned char* p=s; *p != 0; p++) r=(*p + r*0x7F) % i;"
     " return r; }", 3),
]

# `w-bdnz`'s eight cells, transcribed from `work/w-bdnz/probe/*.cpp` at run time
# (they are tracked files), re-differenced with the seed-cancelling form.
BDNZ_CELLS = ["lab_ctl", "lab_forever", "lab_loop", "lab_while", "lab_dowhile",
              "lab_goto", "lab_op", "lab_uns"]

# §4's SIX framed rows that §4.2.1 pairs itself against. Framed probes: the LEAD
# instrument cannot read them (it requires every function before z9 to be a
# leaf), so these are cross-checked with §4's own STRIDE instrument only. The
# bodies are `work/w-label/cflabels.py`'s framed form: the probe calls `gp`.
FRAMED_ROWS = [
    ("cf-if", "int gp(int);\nint P(int a,int b){ if (a) return gp(b)+1;"
     " return gp(b)+2; }", 0),
    ("cf-while", "int gp(int);\nint P(int a,int b){ int r=0; while (a)"
     " { r=r+gp(b); a=a-1; } return r; }", 2),
    ("cf-dowhile", "int gp(int);\nint P(int a,int b){ int r=0; do"
     " { r=r+gp(b); a=a-1; } while (a); return r; }", 1),
    ("cf-for", "int gp(int);\nint P(int a,int b){ int r=0;"
     " for (int i=0;i<a;i++) r=r+gp(b); return r; }", 2),
    ("cf-fornest", "int gp(int);\nint P(int a,int b){ int r=0;"
     " for (int i=0;i<a;i++) for (int j=0;j<a;j++) r=r+gp(b); return r; }", 4),
    ("cf-goto-back", "int gp(int);\nint P(int a,int b){ int r=0;"
     " top: r=r+gp(b); a=a-1; if (a) goto top; return r; }", 1),
]


# ---------------------------------------------------------------------------
# The LEAD instrument. `work/w-slots/lead.py`'s arithmetic, taking source text
# instead of a path so the series can be generated. Reproduced rather than
# imported because lead.py takes a `.cpp` path and the series needs 51 of them;
# the two are checked against each other on every w-slots cell by `--selfcheck`.
# ---------------------------------------------------------------------------
def lead_of_src(src, mode, wd, tag):
    cpp = os.path.join(wd, "%s.cpp" % tag)
    open(cpp, "w").write(src)
    r = subprocess.run([os.path.join(REPO, "scripts", "gt_capture.sh"), cpp]
                       + mode.split(), capture_output=True, text=True)
    objp = r.stdout.strip()
    if not objp or not os.path.exists(objp):
        return None, "CAPTURE FAILED", {}
    o = Obj(open(objp, "rb").read())
    framed = [g for g in G.groups(o) if g["labels"]]
    fl = os.path.join(wd, "flags_%s.txt" % tag)
    open(fl, "w").write("/nologo " + mode + "\n")
    out = os.path.join(wd, "il_" + tag)
    os.makedirs(out, exist_ok=True)
    subprocess.run([os.path.join(REPO, "target", "release", "c2rs"),
                    "capture", cpp, "--keep-il", out, "--flags-file", fl],
                   capture_output=True, text=True)
    got = {f.rsplit(".", 1)[-1]: os.path.join(out, f) for f in os.listdir(out)}
    if "gl" not in got or "ex" not in got:
        return None, "IL CAPTURE FAILED", {}
    counter = int.from_bytes(open(got["gl"], "rb").read()[7:11], "little")
    segs = L.ex_segments(open(got["ex"], "rb").read())
    nleaf = len(segs) - len(framed)
    info = {"counter": counter, "segs": len(segs), "nframed": len(framed),
            "obj": objp}
    if not framed:
        # The SEPARATING CONTROL shape: a leaf-only TU mints no labels, so the
        # counter never reaches this obj (board #742) and no charge can break
        # it. Positive on content: the label-symbol count is printed.
        info["labels"] = len(label_syms(o))
        return None, "NO FRAMED FUNCTION (labels=%d)" % info["labels"], info
    base = counter + LABEL_SEED_GAP + 3 * len(segs) + nleaf
    real = min(framed[0]["labels"])
    info.update({"base": base, "real": real, "framed_name": framed[0]["name"]})
    return real - base, "", info


def label_syms(o):
    return [s["name"] for s in o.symbols
            if (s["name"].startswith("$M") or s["name"].startswith("$T"))
            and s["name"][2:].isdigit()]


def series_src(body, n):
    """`n` copies of the probe, then the SAME framed z9. The copies differ only
    in their own name, which is what `%d` is for — a `goto` label needs the same
    treatment or the second copy will not compile."""
    parts = [FRAMED_DECL]
    for i in range(1, n + 1):
        parts.append(body.replace("%d", str(i)))
    parts.append(FRAMED_TAIL)
    return "\n".join(parts) + "\n"


def leafonly_src(body, n):
    """The separating control: the same bodies with NO framed function."""
    return "\n".join(body.replace("%d", str(i)) for i in range(1, n + 1)) + "\n"


def fit(leads):
    """(k, c, residual) for L(n) = k*n + c over n = 1..len(leads), fitted on the
    two END points so the middle point is a RESIDUAL rather than an input."""
    n1, nN = 1, len(leads)
    k_num = leads[-1] - leads[0]
    if nN == n1:
        return None, None, None
    if k_num % (nN - n1) != 0:
        return None, None, "non-integer slope"
    k = k_num // (nN - n1)
    c = leads[0] - k
    resid = [leads[i] - (k * (i + 1) + c) for i in range(len(leads))]
    return k, c, resid


# ---------------------------------------------------------------------------
def run_rows(rows, mode, wd, nmax=3):
    out = []
    for name, body, published in rows:
        leads, notes, infos = [], [], []
        for n in range(1, nmax + 1):
            lead, note, info = lead_of_src(series_src(body, n), mode, wd,
                                           "%s_n%d" % (name.replace("-", "_"), n))
            leads.append(lead)
            notes.append(note)
            infos.append(info)
        ctl_lead, ctl_note, ctl_info = lead_of_src(
            leafonly_src(body, 1), mode, wd, "%s_ctl" % name.replace("-", "_"))
        out.append({"name": name, "published": published, "leads": leads,
                    "notes": notes, "infos": infos,
                    "ctl": (ctl_lead, ctl_note, ctl_info)})
    return out


def print_rows(res, mode):
    print("== §4.2.1 re-measured: the LEAD instrument, as a SERIES")
    print("   mode: %s   TU = [n copies of P, the SAME framed z9]" % mode)
    print("   Every cell subtracts its OWN `.gl` counter, so the seed cancels")
    print("   INSIDE the TU (#3148). The n=1/2/3 cells of one row do NOT share")
    print("   a counter — they are different source texts — which is exactly")
    print("   why the difference is taken this way and not across them.")
    print()
    # CALIBRATION. `base = counter + 9 + 3*segs + nleaf` is a `/Gy` fact, and
    # `/Ox` is not `/Gy`: there the zero-controls read a constant NEGATIVE lead
    # (measured, and identical on two structurally different zero cells). That
    # constant is a per-TU term, so it lands in `c` and NEVER in `k` — the slope
    # is calibration-free at every mode. `c` is reported net of it so a mode's
    # own offset does not read as a per-TU charge.
    k0 = c0 = 0
    for r in res:
        if r["name"] == "leaf-none" and all(x is not None for x in r["leads"]):
            k0, c0, _r = fit(r["leads"])
            break
    if k0 or c0:
        print("   CALIBRATION: the zero-control reads k = %d, c = %d at this"
              " mode." % (k0, c0))
        print("   `base = counter + 9 + 3*segs + nleaf` is a `/Gy` fact and")
        print("   `/Ox` is not `/Gy`: there every function is over-charged by a")
        print("   CONSTANT, which lands in the SLOPE. Both zero-controls")
        print("   (`leaf-none`, `leaf-if`) read the same one, which is what")
        print("   makes it a calibration rather than a fudge. Columns below are")
        print("   net of it.")
        print()
    print("%-17s %5s | %5s %5s %5s | %4s %4s %6s | %s"
          % ("row", "pub", "L(1)", "L(2)", "L(3)", "k", "c", "resid", "verdict"))
    disagree, discrim, vacuous, bad = [], 0, [], 0
    for r in res:
        leads = r["leads"]
        if any(x is None for x in leads):
            print("%-17s %5d |  %s" % (r["name"], r["published"], r["notes"]))
            bad += 1
            continue
        k, c, resid = fit(leads)
        k -= k0
        c -= c0
        # A cell whose leaf-only probe stops being a LEAF is confounded: the
        # framed group this instrument reads is then no longer `z9`. `w-slots`
        # §3 hit exactly this at `/Ox` and marked it rather than quoting it.
        nf = [i.get("nframed") for i in r["infos"]]
        if any(x != 1 for x in nf):
            print("%-17s %5d | %5d %5d %5d |  CONFOUNDED: %s framed groups —"
                  " the probe is not a leaf at this mode, so the triple read is"
                  " not z9's"
                  % (r["name"], r["published"], leads[0], leads[1], leads[2], nf))
            bad += 1
            continue
        varies = len(set(leads)) > 1
        if varies:
            discrim += 1
        else:
            vacuous.append(r["name"])
        agrees = (k == r["published"] and c == 0)
        if not agrees:
            disagree.append((r["name"], r["published"], k, c))
        print("%-17s %5d | %5d %5d %5d | %4s %4s %6s | %s"
              % (r["name"], r["published"], leads[0], leads[1], leads[2],
                 k, c, ",".join(str(x) for x in resid),
                 "AGREES" if agrees else "**DISAGREES -> obj says %d%s**"
                 % (k, "" if c == 0 else " + %d per TU" % c)))
        cl, cn, ci = r["ctl"]
        if cl is not None or "NO FRAMED" not in cn:
            print("%-17s   SEPARATING CONTROL DID NOT HOLD: %s" % ("", cn))
            bad += 1
    print()
    print("rows: %d   discriminating series (L(n) varies with n): %d"
          % (len(res), discrim))
    print("structurally NON-discriminating (published +0, nothing could move): %s"
          % (", ".join(vacuous) or "none"))
    print("rows where measured != published: %d" % len(disagree))
    for nm, pub, k, c in disagree:
        print("   %-17s published %+d   obj %+d%s   (%s)"
              % (nm, pub, k, "" if c == 0 else " +%d/TU" % c,
                 "ONE HIGH" if pub == k + 1 else
                 "ONE LOW" if pub == k - 1 else "%+d" % (k - pub)))
    print("controls failed: %d" % bad)
    return bad, disagree, discrim


def print_mutants(res):
    """Absence is not success. For every row, the neighbouring charges k-1, k+1
    and 0 are checked against the REFERENCE OBJ's own symbol-table bytes: a
    charge k' predicts `$M(base + k')` for the framed function, and the obj says
    what it says. The separating control is the same body in a LEAF-ONLY TU,
    which mints no labels (board #742) and is therefore green under every
    charge — a mutant that reddens it is measuring something else.

    This is a WEAKER construction than `w-fenceb`/`w-slots`' `c2rs gap` battery,
    which routes the wrong charge through the shipped emitter. It is the same
    six bytes one layer earlier, and it is what is available for 16 of these 17
    rows, none of which the port emits. Stated, not dressed up."""
    print()
    print("== the mutants: a wrong charge is a wrong $M, checked against the")
    print("   reference obj's own bytes, with a leaf-only separating control")
    print()
    print("%-17s %6s %6s | %-28s | %s"
          % ("row", "base", "real", "charge k-1 / k / k+1 / 0", "control"))
    reds = greens = 0
    for r in res:
        if any(x is None for x in r["leads"]):
            continue
        k, c, _ = fit(r["leads"])
        i = r["infos"][0]
        cand = []
        for kk in (k - 1, k, k + 1, 0):
            if kk not in cand:
                cand.append(kk)
        cells = []
        for kk in cand:
            hit = (i["base"] + c + kk == i["real"])
            cells.append("%s%d:%s" % ("" if kk != k else "*", kk,
                                      "match" if hit else "MISMATCH"))
            if kk != k:
                if not hit:
                    reds += 1
                else:
                    greens += 1
        ctlok = r["ctl"][0] is None and "NO FRAMED" in r["ctl"][1]
        print("%-17s %6d %6d | %-28s | %s"
              % (r["name"], i["base"], i["real"], " ".join(cells),
                 "green (0 labels)" if ctlok else "!! CONTROL BROKEN"))
    print()
    print("mutant cells that RED (a wrong charge is a wrong obj): %d" % reds)
    print("mutant cells that stayed green (a wrong charge nothing detects): %d"
          % greens)
    # §4.2.3's channel claim, reproduced positively on content: a leaf-only TU
    # mints NO label symbol at all, so the counter never reaches its obj
    # (board #742) and no charge can break it. Counted, not assumed.
    ok = sum(1 for r in res
             if r["ctl"][0] is None and "NO FRAMED" in r["ctl"][1]
             and r["ctl"][2].get("labels") == 0)
    print("separating controls green with EXACTLY 0 label symbols: %d of %d"
          % (ok, len(res)))


# ---------------------------------------------------------------------------
def run_bdnz(mode, wd):
    """`work/w-bdnz/LABEL_LEAD.md`'s eight cells, re-differenced seed-free.

    Its own instrument is CROSS-TU — *"two TUs differ in exactly one function
    body"* — which is the construction #3148 refuted: a TU's `.gl` counter
    depends on its own source text, so a difference across two of them is a
    charge plus a counter gap. Its numbers are quoted verbatim into the shipped
    `IlFunction::label_slots` doc comment."""
    pub = {"lab_ctl": None, "lab_forever": 2, "lab_loop": 7, "lab_while": 7,
           "lab_dowhile": 6, "lab_goto": 8, "lab_op": 7, "lab_uns": 7}
    print("== `work/w-bdnz/LABEL_LEAD.md`, re-differenced with the")
    print("   seed-cancelling form.  mode: %s" % mode)
    print("   Its cells are SEPARATE TUs and do NOT share a `.gl` counter.")
    print()
    print("%-14s %8s %8s %8s | %9s %9s | %s"
          % ("cell", "counter", "base", "real $M", "published", "seed-free", "gap"))
    rows = []
    ctl_counter = None
    for nm in BDNZ_CELLS:
        p = os.path.join(REPO, "work", "w-bdnz", "probe", "%s.cpp" % nm)
        if not os.path.exists(p):
            print("%-14s  MISSING" % nm)
            continue
        lead, note, info = lead_of_src(open(p).read(), mode, wd, "bdnz_" + nm)
        if lead is None:
            print("%-14s  %s" % (nm, note))
            continue
        if nm == "lab_ctl":
            ctl_counter = info["counter"]
        gap = "" if ctl_counter is None else "%+d" % (info["counter"] - ctl_counter)
        print("%-14s %8d %8d %8d | %9s %9d | counter %s vs lab_ctl"
              % (nm, info["counter"], info["base"], info["real"],
                 "-" if pub[nm] is None else "+%d" % pub[nm], lead, gap))
        rows.append((nm, pub[nm], lead, info))
    print()
    print("REPRODUCTION CHECK — the published `$M` column must reproduce or the")
    print("re-differencing is of different objs:")
    for nm, p, lead, info in rows:
        print("   %-14s real $M%d" % (nm, info["real"]))
    return rows


def run_framed(mode, wd):
    print("== §4's SIX FRAMED rows, through §4's OWN instrument")
    print("   (`a0 . P . a1 . a2`; a framed probe is unreadable by the LEAD")
    print("    instrument, which needs every function before z9 to be a leaf)")
    print()
    print("   §4's OWN warning box is the correction, and this table exists to")
    print("   REPRODUCE it: `surcharge = stride - base - (minted - 5)`, NOT")
    print("   `stride - base`. A loop body that spills callee-saved registers")
    print("   pays a `minted` surcharge for the `__savegprlr_N`/`__restgprlr_N`")
    print("   pair (§1.1's +2) AND a control-flow surcharge, and a re-derivation")
    print("   that drops the `minted` column charges the first one twice. The")
    print("   NAIVE column below is what a lane that drops it reads.")
    print()
    print("%-14s %5s %6s %7s %7s %9s %7s %7s | %s"
          % ("row", "pub", "stride", "minted", "naive", "corrected", "extra",
             "control", "verdict"))
    bad = 0
    for nm, src, pubv in FRAMED_ROWS:
        decls, probe = src.split("\n", 1)
        row = G.run(nm, decls, [], probe, "", mode, wd)
        if row is None or "error" in row:
            print("%-14s  %s" % (nm, (row or {}).get("error", "CAPTURE FAILED")))
            bad += 1
            continue
        if row["control"] not in (4, 5):
            bad += 1
        naive = row["stride"] - 5
        got = row["stride"] - row["minted"]
        print("%-14s %5d %6d %7d %7d %9d %7s %7d | %s"
              % (nm, pubv, row["stride"], row["minted"], naive, got,
                 row["extra"], row["control"],
                 "AGREES" if got == pubv else
                 "**DISAGREES -> obj says %+d**" % got))
    print("controls failed: %d" % bad)
    return bad


def run_stride(rows, mode, wd):
    """§4.2.1's OWN instrument on the same bodies — the cross-check that says
    whether the two seed-free instruments agree (PREREG F4)."""
    print("== the same rows through §4.2.1's OWN instrument (`a0 . P . a1 . a2`)")
    print("   surcharge = stride - stride(leaf-none) = stride - 1")
    print()
    print("%-17s %5s %6s %9s %8s | %s"
          % ("row", "pub", "stride", "surcharge", "control", "verdict"))
    bad = 0
    got = {}
    for nm, body, pubv in rows:
        probe = body.replace("%d", "")
        row = G.run(nm, "", [], probe, "", mode, wd)
        if row is None or "error" in row:
            print("%-17s  %s" % (nm, (row or {}).get("error", "CAPTURE FAILED")))
            bad += 1
            continue
        if row["control"] not in (4, 5):
            bad += 1
        s = row["stride"] - 1
        got[nm] = s
        print("%-17s %5d %6d %9d %8d | %s"
              % (nm, pubv, row["stride"], s, row["control"],
                 "AGREES" if s == pubv else "**DISAGREES -> %+d**" % s))
    print("controls failed: %d" % bad)
    return bad, got


def main(argv):
    mode = "/O1 /GS- /c"
    if "--mode" in argv:
        i = argv.index("--mode"); mode = argv[i + 1]; del argv[i:i + 2]
    want = []
    if "--rows" in argv:
        i = argv.index("--rows")
        want = [a for a in argv[i + 1:] if not a.startswith("--")]
    wd = tempfile.mkdtemp(prefix="wlabtab")
    bad = 0
    if "--bdnz" in argv:
        run_bdnz(mode, wd)
        return 0
    if "--framed" in argv:
        return 1 if run_framed(mode, wd) else 0
    rows = ROWS + BRIDGE
    if "--ladder" in argv:
        rows = PTRWALK_LADDER + BRIDGE + [ROWS[13]]
    if want:
        rows = [r for r in rows if r[0] in want]
    if "--stride" in argv:
        b, _ = run_stride(rows, mode, wd)
        return 1 if b else 0
    res = run_rows(rows, mode, wd)
    b, disagree, discrim = print_rows(res, mode)
    bad += b
    print_mutants(res)
    print()
    b2, stride_got = run_stride(rows, mode, wd)
    bad += b2
    print()
    print("== the two seed-free instruments, side by side (PREREG F4)")
    k0 = 0
    for r in res:
        if r["name"] == "leaf-none" and all(x is not None for x in r["leads"]):
            k0, _c, _r = fit(r["leads"])
            break
    print("   LEAD k is net of the zero-control's %d (see CALIBRATION above)" % k0)
    print("%-17s %9s %9s | %s" % ("row", "LEAD k", "STRIDE-1", "agree?"))
    ndis = 0
    for r in res:
        if any(x is None for x in r["leads"]):
            continue
        if any(i.get("nframed") != 1 for i in r["infos"]):
            print("%-17s %9s %9s | CONFOUNDED (probe is framed at this mode)"
                  % (r["name"], "-", stride_got.get(r["name"])))
            continue
        k, c, _ = fit(r["leads"])
        k -= k0
        s = stride_got.get(r["name"])
        ok = (s == k)
        if not ok:
            ndis += 1
        print("%-17s %9s %9s | %s" % (r["name"], k, s, "yes" if ok else "NO"))
    print("instrument disagreements: %d of %d" % (ndis, len(res)))
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
