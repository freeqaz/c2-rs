#!/usr/bin/env python3
"""spellgrid.py — GRID S, the SPELLING-AXIS POPULATION TABLE.

Declared in `work/w-spell/PREREG.md` §2, committed before this file existed.
This file is committed before a single cell is compiled.

WHAT IT MEASURES
----------------
For a store run with exactly two distinct producers — one register-derived
(sixteen spellings) and one single-word constant `li rX,7` — which of the two
takes the TOP pool register, as a function of

    (observed mnemonic, producer uses, constant uses, store-base structure)

Four lanes (w-next, w-alloc2, w-refbind, w-seam) agree that no key over
`ProducerKind` survives off its own cells and that the separating axis is the
producer's SPELLING.  This grid enumerates that axis instead of fitting to it.

THE INSTRUMENT, AND WHY IT IS BUILT THIS WAY  (PREREG §1)
---------------------------------------------------------
* **No regex names a source register.**  w-refbind's `refprobe` scored its two
  most valuable cells out of regime because adding a formal moved `u`/`v` from
  `r4`/`r5` to `r5`/`r6` and its producer regexes were anchored on `r4`.  Here
  the producer's register is read off ITS OWN STORE: this grid assigns the
  producer's run displacements 96,100,104,… and the constant's 32,36,40, so
  `stw rS, 96(rB)` names the producer's register whatever `rB` is.
* **The mnemonic is OBSERVED, never assumed.**  Board #843 has fired three
  times (`sub` not `subf`; `slwi` not `rlwinm`; and again in w-refbind's first
  grade run).  This grader matches NO per-spelling regex.  Having found the
  producer's register it finds the instruction that DEFINES it and records the
  mnemonic c2 printed.  The table is keyed on c2's own spelling.
* **#644.**  A register defined more than once (a rematerialised `lwz`, a
  two-instruction `addi`+`addi` interior address) is OUT OF REGIME — never a
  hit, never a miss.  `selected / reached / graded / out-of-regime /
  compile-failed` are five separate printed counters (STATUS trap 5).
* **ORDER and ALLOC are read separately** and printed side by side.

SHIPS NOTHING.  Usage:  spellgrid.py [--only SUBSTR] [--jobs N] [--flags alt]
"""

import os
import re
import subprocess
import sys
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
REFOBJ = os.path.join(ROOT, "work", "w-frame", "refobj.sh")
DUMP = os.path.join(ROOT, "scripts", "gt_dump.py")
SRCDIR = os.path.join(HERE, "gridS")


def _sib(name):
    d = ROOT
    while d != os.path.dirname(d):
        cand = os.path.join(os.path.dirname(d), name)
        if os.path.isdir(cand):
            return cand
        d = os.path.dirname(d)
    return None


DC3 = os.environ.get("C2RS_DC3") or _sib("dc3-decomp")

# head@0(32) · f0..ff@32..92 · inner@96(32) · inner2@128(32).  Identical to
# `work/w-refbind/holdout.py`'s layout so the two lanes' cells are comparable.
STRUCT = """\
struct L%(t)s { int a0; int a1; int a2; int a3; int a4; int a5; int a6; int a7; };
struct S%(t)s {
    L%(t)s head;
    int f0; int f1; int f2; int f3; int f4; int f5; int f6; int f7;
    int f8; int f9; int fa; int fb; int fc; int fd; int fe; int ff;
    L%(t)s inner;
    L%(t)s inner2;
};
"""
OFF_INNER, OFF_INNER2, OFF_F0, OFF_F8 = 96, 128, 32, 64
PROD_OFFS = [OFF_INNER + 4 * i for i in range(8)]      # 96 100 104 …
CONST_OFFS = [OFF_F0 + 4 * i for i in range(8)]        # 32 36 40 …

# The sixteen spellings, exactly as PREREG §2 lists them.  The C++ is the only
# thing declared here; which instruction c2 selects is MEASURED.
SPELLINGS = [
    ("self",    "(int)&s->inner"),               # interior address, self
    ("cross",   "(int)&s->inner2"),              # interior address, cross
    ("addi",    "(u + 5)"),
    ("add",     "(u + v)"),
    ("sub",     "(u - v)"),
    ("and",     "(u & v)"),
    ("or",      "(u | v)"),
    ("xor",     "(u ^ v)"),
    ("neg",     "(-u)"),
    ("nor",     "(~u)"),
    ("slwi",    "(u << 3)"),
    ("srwi",    "(int)((unsigned)u >> 3)"),
    ("srawi",   "(u >> 3)"),
    ("extsh",   "(int)(short)u"),
    ("lwz",     "s->f8"),
    ("formal",  "u"),                            # a formal copy — no producer
]

COUNTS = [(1, 1), (2, 1), (3, 1), (2, 2), (2, 3)]
BASES = ("1base", "2base")


class Cell(object):
    def __init__(self, sp, expr, ru, cu, bases):
        self.sp, self.expr, self.ru, self.cu, self.bases = sp, expr, ru, cu, bases
        self.name = "S-%s-%s-r%dk%d" % (sp, bases, ru, cu)

    def source(self):
        t = self.name.replace("-", "_")
        body = []
        if self.bases == "2base":
            # A bind at a NON-ZERO displacement — #865 says this is one way to
            # put a second store-base value in the body; w-refbind R8 says a
            # bind at displacement 0 is not.  Exactly ONE bind, so §5.1's
            # extra-`lwz` confound cannot fire.
            body.append("    L%(t)s& q = s->inner;")
            pslot = "q.a%d"
        else:
            pslot = "s->inner.a%d"
        for i in range(self.cu):
            body.append("    s->f%s = 7;" % "0123456789abcdef"[i])
        for i in range(self.ru):
            body.append("    %s = %s;" % (pslot % i, self.expr))
        return ((STRUCT % dict(t=t))
                + "void g%s(S%s* s, int u, int v) {\n%s\n}\n"
                % (t, t, "\n".join(body) % dict(t=t)))


def build():
    return [Cell(sp, e, ru, cu, b)
            for sp, e in SPELLINGS for ru, cu in COUNTS for b in BASES]


# ---------------------------------------------------------------- instrument
STORE_RX = re.compile(r"^(st[bhwd]u?)\s+(\d+),\s*(-?\d+)\((\d+)\)$")
DEF_RX = re.compile(r"^([a-z][a-z0-9._]*)\s+(\d+),")


def dis(obj):
    out = subprocess.run([sys.executable, DUMP, obj, "--text-only"],
                         capture_output=True, text=True).stdout
    res = []
    for line in out.splitlines():
        p = line.split()
        if len(p) >= 3 and len(p[1]) == 8:
            res.append(" ".join(p[2:]).split(";")[0].strip())
    return res


def defs_of(words, reg):
    """Every (index, mnemonic) that DEFINES `reg`.  A store defines nothing."""
    out = []
    for i, w in enumerate(words):
        if STORE_RX.match(w):
            continue
        m = DEF_RX.match(w)
        if m and int(m.group(2)) == reg:
            out.append((i, m.group(1)))
    return out


def stores(words):
    """[(index, mnemonic, src_reg, disp, base_reg)] in EMISSION order."""
    out = []
    for i, w in enumerate(words):
        m = STORE_RX.match(w)
        if m:
            out.append((i, m.group(1), int(m.group(2)),
                        int(m.group(3)), int(m.group(4))))
    return out


def observe(words, offs_prod, offs_const):
    """dict of readings, or a string reason for OUT OF REGIME."""
    st = stores(words)
    pr = {s[2] for s in st if s[3] in offs_prod}
    cr = {s[2] for s in st if s[3] in offs_const}
    if len(pr) != 1:
        return "the producer's stores name %d distinct registers" % len(pr)
    if len(cr) != 1:
        return "the constant's stores name %d distinct registers" % len(cr)
    preg, creg = pr.pop(), cr.pop()
    if preg == creg:
        return "both runs store out of r%d" % preg
    pd, cd = defs_of(words, preg), defs_of(words, creg)
    if len(pd) != 1:
        return ("the producer's r%d is defined %d times (#644)"
                % (preg, len(pd)))
    if len(cd) != 1:
        return "the constant's r%d is defined %d times (#644)" % (creg, len(cd))
    return dict(preg=preg, creg=creg,
                pmnem=pd[0][1], cmnem=cd[0][1],
                pidx=pd[0][0], cidx=cd[0][0],
                store_order=[s[3] for s in st],
                base_regs=sorted({s[4] for s in st}))


def run_cell(a):
    name, outdir, flags = a
    cpp = os.path.join(SRCDIR, name + ".cpp")
    obj = os.path.join(outdir, name + ".obj")
    env = dict(os.environ, C2RS_DC3=DC3)
    script = REFOBJ
    if flags:
        env["C2RS_SPELL_FLAGS"] = flags
        script = os.path.join(HERE, "refobj_flags.sh")
    r = subprocess.run([script, os.path.relpath(cpp, DC3), obj],
                       capture_output=True, text=True, env=env)
    if r.returncode != 0 or not os.path.exists(obj):
        return name, None
    return name, dis(obj)


def main():
    only, jobs, flags, tag = None, 8, None, "gridS"
    argv = sys.argv[1:]
    while argv:
        a = argv.pop(0)
        if a == "--only":
            only = argv.pop(0)
        elif a == "--jobs":
            jobs = int(argv.pop(0))
        elif a == "--flags":
            flags = argv.pop(0)
            tag = "gridS_alt"
        else:
            print("unknown arg %r" % a)
            return 2

    cells = build()
    if only:
        cells = [c for c in cells if only in c.name]
    os.makedirs(SRCDIR, exist_ok=True)
    for c in cells:
        open(os.path.join(SRCDIR, c.name + ".cpp"), "w").write(c.source())
    outdir = os.path.join(HERE, tag + "_obj")
    os.makedirs(outdir, exist_ok=True)

    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        res = dict(ex.map(run_cell, [(c.name, outdir, flags) for c in cells]))

    log = open(os.path.join(HERE, tag + "_dis.txt"), "w")
    rows, reached, graded, oor, fail = [], 0, 0, 0, 0
    print("  %-26s | %-7s %-7s | %-6s | %-5s | %s"
          % ("cell", "prodreg", "constreg", "winner", "first", "mnemonic"))
    print("  " + "-" * 86)
    for c in cells:
        w = res[c.name]
        if w is None:
            print("  %-26s | COMPILE FAILED" % c.name)
            fail += 1
            continue
        reached += 1
        log.write("== %s\n%s\n\n" % (c.name, "\n".join(w)))
        o = observe(w, PROD_OFFS[:c.ru], CONST_OFFS[:c.cu])
        if isinstance(o, str):
            print("  %-26s | OUT OF REGIME: %s" % (c.name, o))
            oor += 1
            rows.append((c, None, o))
            continue
        graded += 1
        winner = "prod" if o["preg"] > o["creg"] else "const"
        first = "prod" if o["pidx"] < o["cidx"] else "const"
        rows.append((c, o, winner))
        print("  %-26s | r%-6d r%-6d | %-6s | %-5s | %s"
              % (c.name, o["preg"], o["creg"], winner, first, o["pmnem"]))
    log.close()

    print("\n  selected %d | reached %d | GRADED %d | out-of-regime %d |"
          " compile-failed %d" % (len(cells), reached, graded, oor, fail))

    # ---- THE POPULATION TABLE — the deliverable --------------------------
    counts = [k for k in COUNTS]
    print("\n  GRID S — POPULATION TABLE.  `P` = the producer takes the top"
          " pool register, `c` = the constant does,")
    print("  `-` = out of regime.  Rows are (C++ spelling, mnemonic c2"
          " selected); columns are (ru,cu) per base mode.")
    hdr = "  %-10s %-8s " % ("spelling", "mnemonic")
    for b in BASES:
        for ru, cu in counts:
            hdr += "%s:%d/%d " % (b[0], ru, cu)
    print(hdr)
    print("  " + "-" * len(hdr))
    by = {}
    mnem = {}
    for c, o, verdict in rows:
        by[(c.sp, c.bases, c.ru, c.cu)] = (verdict if o else "-")
        if o:
            mnem.setdefault(c.sp, set()).add(o["pmnem"])
    for sp, _e in SPELLINGS:
        ms = sorted(mnem.get(sp, []))
        line = "  %-10s %-8s " % (sp, ",".join(ms) if ms else "?")
        for b in BASES:
            for ru, cu in counts:
                v = by.get((sp, b, ru, cu))
                line += "%-6s " % ("P" if v == "prod" else
                                   "c" if v == "const" else "-")
        print(line)

    # ---- S2: the same (mnemonic, ru, cu, bases) must give the same winner --
    grp = {}
    for c, o, verdict in rows:
        if o:
            grp.setdefault((o["pmnem"], c.ru, c.cu, c.bases),
                           set()).add(verdict)
    s2 = [k for k, v in grp.items() if len(v) > 1]
    print("\n  S2 — cells agreeing on (mnemonic, ru, cu, bases) that DISAGREE"
          " on the winner: %d" % len(s2))
    for k in sorted(s2):
        print("     %s" % (k,))

    # ---- S4: 1base -> 2base never turns a loss into a win -----------------
    s4 = [(sp, ru, cu) for sp, _e in SPELLINGS for ru, cu in counts
          if by.get((sp, "1base", ru, cu)) == "const"
          and by.get((sp, "2base", ru, cu)) == "prod"]
    print("\n  S4 — spellings that LOSE at 1base and WIN at 2base (registered"
          " as none): %d" % len(s4))
    for k in s4:
        print("     %s" % (k,))

    # ---- S5: srawi vs srwi ------------------------------------------------
    dif = [(b, ru, cu) for b in BASES for ru, cu in counts
           if by.get(("srawi", b, ru, cu)) != by.get(("srwi", b, ru, cu))]
    print("\n  S5 — (ru,cu,bases) cells where `srawi` and `srwi` DISAGREE:"
          " %d of %d" % (len(dif), len(BASES) * len(counts)))
    for k in dif:
        print("     %s  srawi=%s srwi=%s"
              % (k, by.get(("srawi", k[0], k[1], k[2])),
                 by.get(("srwi", k[0], k[1], k[2]))))

    # ---- S6: extsh and lwz at (1,1) --------------------------------------
    print("\n  S6 — `extsh` and `lwz` at (1,1):")
    for sp in ("extsh", "lwz"):
        for b in BASES:
            print("     %-6s %-6s -> %s" % (sp, b, by.get((sp, b, 1, 1))))

    # ---- S3: is there ANY linear key 2*ru + b(spelling) > 2*cu ? ----------
    print("\n  S3 — exhaustive search for a per-spelling additive bonus"
          " b in [-8,8] with `2*ru + b > 2*cu` (per base mode):")
    total_res = 0
    for b in BASES:
        worst = None
        for sp, _e in SPELLINGS:
            obs = [(c.ru, c.cu, v) for c, o, v in rows
                   if o and c.sp == sp and c.bases == b]
            if not obs:
                continue
            best = min(sum(1 for ru, cu, v in obs
                           if (("prod" if 2 * ru + bb > 2 * cu else "const")
                               != v))
                       for bb in range(-8, 9))
            total_res += best
            if best and (worst is None or best > worst[1]):
                worst = (sp, best)
        print("     %-6s residual after the best per-spelling bonus:"
              " worst spelling %s" % (b, worst))
    print("     TOTAL residual cells no additive key can reach: %d"
          % total_res)
    print("     S3 (registered: >= 1 residual) -> %s"
          % ("HIT — no additive key fits" if total_res >= 1
             else "**MISS** — an additive key fits every graded cell"))

    # ---- S1 ---------------------------------------------------------------
    print("\n  S1 (registered: >= 80%% of %d cells graded) -> %d graded = "
          "%.1f%% -> %s" % (len(cells), graded, 100.0 * graded / len(cells),
                            "HIT" if graded >= 0.8 * len(cells) else "**MISS**"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
