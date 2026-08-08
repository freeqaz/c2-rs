#!/usr/bin/env python3
"""GRID-I — the wb-inline obj-check grid.

    grid.py gen   <dir>            emit the cell sources + the frozen table
    grid.py run   <dir> <repo>     compile every cell with real cl.exe under wibo
    grid.py score <dir>            score the rivals against the measured verdicts

THE VERDICT FUNCTION IS THREE-VALUED, deliberately (wb-memcpy §4's control):

    inlined   the caller's .text carries NO REL24 against the callee symbol
    called    it carries >= 1
    absent    the caller or the callee is missing from the obj

`absent` exists because a two-valued verdict would score a cell that did not
occur as evidence for whichever rival predicted the missing side. Any cell
that comes back `absent` is reported as ASSUMPTION UNMET and excluded from
every rival pair it was carrying, with separation re-asserted on the rest —
the rule wb-frame §5.2 registered and then had to use on its C7.

Scratch only: nothing here is committed as a fixture, and no cell reaches
`crates/`.
"""
import os
import subprocess
import sys
import json

HERE = os.path.dirname(os.path.abspath(__file__))

# --------------------------------------------------------------------------
# The callee bodies. Size is swept by a straight-line arithmetic chain, which
# is the one axis where the emitted word count is a linear function of a source
# knob, so `s` (bytes of the callee's own .text) is controllable to the word.
# --------------------------------------------------------------------------


def chain(k, var="a", base=0):
    """k statements that CANNOT be folded: each reads a distinct extern slot.

    GRID-I v1 used `a = a*3 + i` and c2 folded the whole chain to two words
    (`mullw` + `sub`) at every k -- the size axis did not occur.  That is
    recorded as a refuted cell design, not smoothed over: an arithmetic chain
    over one local is a constant-propagation identity, so the only ladder that
    moves the emitted size is one whose every rung touches memory the compiler
    cannot see through.
    """
    return "".join("  %s += tbl[%d];\n" % (var, base + i) for i in range(k))


def callee(name, k, linkage="static", extra_params=0, forceinline=False,
           nonleaf=False, varargs=False, recurse=False):
    q = ""
    if forceinline:
        q = "__forceinline "
    elif linkage == "static":
        q = "static "
    ps = ["int a"] + ["int p%d" % i for i in range(extra_params)]
    if varargs:
        ps.append("...")
    body = chain(k)
    if nonleaf:
        body = "  a += sink(a);\n" + body
    if recurse:
        body = "  if (a > 1000) return %s(a - 1);\n" % name + body
    return "%sint %s(%s) {\n%s  return a;\n}\n" % (q, name, ", ".join(ps), body)


def caller(name, callee_name, nsites, bulk=0, conditional=False, args=1):
    """A caller with `nsites` calls to `callee_name`, plus `bulk` filler words.

    `bulk` inflates the CALLER's own instruction count, which is the input to
    the budget `B = clamp(2*caller_instrs, 1000, 35000)` this lane reads at
    0x10b626d8. No published rival has a caller-side input at all.
    """
    a = ", ".join(["x"] + ["0"] * (args - 1))
    body = "".join("  x += %s(%s);\n" % (callee_name, a) for _ in range(nsites))
    if conditional:
        body = "".join("  if (x & %d) x += %s(%s);\n" % (1 << i, callee_name, a)
                       for i in range(nsites))
    return ("int %s(int x) {\n%s%s  return x;\n}\n"
            % (name, chain(bulk, "x", 4000), body))


PRELUDE = "extern int sink(int);\nextern int tbl[];\n"


# --------------------------------------------------------------------------
# THE CELLS.  Each cell is one obj: prelude + callee(s) + one caller.
# `flags` names the cl.exe mode.  `pred` holds every rival's frozen verdict.
# --------------------------------------------------------------------------

MODES = {
    "O1":    "/O1 /GS- /c",
    "O2":    "/O2 /GS- /c",
    "O1Ot":  "/O1 /Ot /GS- /c",
    "O2Os":  "/O2 /Os /GS- /c",
    "O1Ob0": "/O1 /Ob0 /GS- /c",
}

# The size ladder, in emitted words of the callee's own body.
# k statements -> k words + `blr`; a static one-parameter leaf therefore emits
# about k+1 words = 4k+4 bytes.  The rungs bracket every published boundary:
#   16 words  = 64 B   INLINE-P's EXTERNAL ceiling / its `i <= 16` unbounded arm
#   41 words  = 164 B  INLINE-P's conditional 1->0 ceiling
#   65 words  = 260 B  INLINE-P's STATIC hard ceiling
LADDER = [4, 5, 7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 18, 19, 24,
          29, 30, 31, 32, 34, 35, 36, 38, 42, 50, 62, 66, 80, 120, 130]


def build_cells():
    cells = []

    def add(cid, src, kind, **kw):
        cells.append(dict(id=cid, src=src, kind=kind, **kw))

    # --- A: the size ladder, static callee, ONE site, five flag sets --------
    for m in MODES:
        for k in LADDER:
            src = PRELUDE + callee("cg", k, "static") + caller("cf", "cg", 1)
            add("A_%s_k%d" % (m, k), src, "ladder", mode=m, k=k, n=1,
                linkage="static")

    # --- B: the same ladder with EXTERNAL linkage, ONE site -----------------
    for m in ("O1", "O2"):
        for k in LADDER:
            src = PRELUDE + callee("cg", k, "extern") + caller("cf", "cg", 1)
            add("B_%s_k%d" % (m, k), src, "ladder", mode=m, k=k, n=1,
                linkage="extern")

    # --- C: SITE COUNT at three sizes, static ------------------------------
    for m in ("O1", "O2"):
        for k in (10, 24, 50):
            for n in (1, 3, 9):
                src = PRELUDE + callee("cg", k, "static") + caller("cf", "cg", n)
                add("C_%s_k%d_n%d" % (m, k, n), src, "sites", mode=m, k=k, n=n,
                    linkage="static")

    # --- D: CALLER SIZE — the discriminator no published rival has ----------
    # Same callee, same one site; only the caller's own instruction count moves.
    # The reading: the budget is 2*caller_instrs floored at 1000, and a callee
    # of <= 40 instructions is never charged against it, so caller size must
    # move NOTHING here.  A caller-side budget rule fitted the other way would.
    for m in ("O1", "O2"):
        for k in (24, 50, 120):
            for bulk in (0, 700):
                src = PRELUDE + callee("cg", k, "static") + \
                    caller("cf", "cg", 1, bulk=bulk)
                add("D_%s_k%d_b%d" % (m, k, bulk), src, "caller", mode=m, k=k,
                    n=1, bulk=bulk, linkage="static")

    # --- E: CONDITIONAL sites ----------------------------------------------
    for m in ("O1", "O2"):
        for k in (24, 50, 120):
            src = PRELUDE + callee("cg", k, "static") + \
                caller("cf", "cg", 1, conditional=True)
            add("E_%s_k%d" % (m, k), src, "cond", mode=m, k=k, n=1,
                linkage="static")

    # --- F: the categorical clauses ----------------------------------------
    for m in ("O1", "O2", "O1Ob0"):
        src = PRELUDE + callee("cg", 120, "static", forceinline=True) + \
            caller("cf", "cg", 1)
        add("F_force_%s" % m, src, "cat", mode=m, k=120, n=1, linkage="static",
            tag="forceinline")
        src = PRELUDE + callee("cg", 6, "static", varargs=True) + \
            "int cf(int x) {\n  return cg(x, 1, 2);\n}\n"
        add("F_varargs_%s" % m, src, "cat", mode=m, k=6, n=1, linkage="static",
            tag="varargs")
        src = PRELUDE + callee("cg", 6, "static", recurse=True) + \
            caller("cf", "cg", 1)
        add("F_recurse_%s" % m, src, "cat", mode=m, k=6, n=1, linkage="static",
            tag="recurse")
        src = PRELUDE + callee("cg", 6, "static") + caller("cf", "cg", 1)
        add("F_small_%s" % m, src, "cat", mode=m, k=6, n=1, linkage="static",
            tag="small")

    # --- G: the ANCHOR shape — six callees of supershuffle's size, one site -
    # ?supershuffle calls six 21-26-word bodies and c2 inlines exactly one.
    for m in ("O1", "O2"):
        for k in (18, 22, 26):
            body = "".join(callee("sh%d" % i, k + i, "extern") for i in range(6))
            calls = "".join("  x += sh%d(x);\n" % i for i in range(6))
            src = (PRELUDE + body +
                   "int cf(int x) {\n%s  return x;\n}\n" % calls)
            add("G_%s_k%d" % (m, k), src, "anchor", mode=m, k=k, n=1,
                linkage="extern", multi=True)

    return cells


# --------------------------------------------------------------------------
# THE RIVALS.  Each returns "inlined" / "called" for a cell.
# --------------------------------------------------------------------------

CAL = {}


def s_bytes(k, extra=1):
    """The callee's own emitted `.text` size in bytes, MEASURED at /O1.

    `INLINE_PREDICATE.md` §2 defines `s` as "G's own emitted `.text` size at
    /O1", obj-readable. Calibration measures that INPUT; it does not touch the
    verdict under test, and it is run and committed BEFORE the per-cell
    predictions are frozen, so no prediction is fitted to an outcome.
    """
    if CAL.get(k):
        return CAL[k]
    return 4 * (k + extra)


def r_incumbent(c):
    """R1 = INLINE-P exactly as INLINE_PREDICATE.md §2 publishes it.

    index = s (STATIC) | s - 4*(nparams-1) - 8*[inline] (EXTERNAL), minus
    48*[leaf].  Every ladder callee here is a leaf with one parameter and is
    not `inline`, so index = s - 48 in both classes.
    """
    if c.get("tag") == "varargs":
        return "called"
    if c.get("tag") == "forceinline":
        # `inline(G)` shaves 8 bytes; nothing else in the model fires.
        idx = s_bytes(c["k"]) - 48 - 8
        return "inlined" if idx <= 64 else "called"
    if c.get("tag") == "recurse":
        return "called"          # non-leaf by its own `bl`
    idx = s_bytes(c["k"]) - 48
    n = c["n"]
    if c["linkage"] == "extern":
        return "inlined" if idx <= 64 else "called"
    i = idx // 4
    if i >= 65:
        nmax = 0
    elif i <= 16:
        nmax = 99
    else:
        nmax = min(9, 1 + 19 // (i - 16))
    if c.get("kind") == "cond" and 160 < idx <= 260:
        nmax = 0
    return "inlined" if n <= nmax else "called"


def r_ceiling(c):
    """R2 = THIS LANE'S READING.

    Candidacy (0x10b5fdfd): when the favor-speed bit 0x10c2e310 is CLEAR the
    callee's tuple count must be < 0x10c46318 = 16 << 0x10c2ea98; when it is
    SET that ceiling is skipped and the profitability model runs instead.
    Budget (0x10b60a0e / 0x10b624a2): a callee of <= 0x28 = 40 tuples is never
    charged and never declined for affordability; a larger one must fit
    B = clamp(2*caller_tuples, 1000, 35000).  __forceinline (flag 0x2000)
    bypasses both.  Depth cap 16 (0x10b609ae).

    Stated as a *behavioural* rival with the one free parameter C = the tuple
    ceiling in emitted words, registered at C = 65 because the image value
    0x10c2ea98 = 3 gives 128 and the option decoder is expected to lower it;
    the ladder brackets 16/41/65 so the grid LOCATES C rather than assuming it.
    """
    if c.get("tag") == "varargs":
        return "called"
    if c.get("tag") == "forceinline":
        return "inlined"
    if c.get("tag") == "recurse":
        return "called"
    if c["mode"] == "O1Ob0":
        return "called"
    favor_speed = c["mode"] in ("O2", "O1Ot")
    words = s_bytes(c["k"]) // 4
    if favor_speed:
        return "inlined" if words <= 40 else "called"
    return "inlined" if words < 65 else "called"


def r_size64(c):
    """R3 = the strawman every earlier round was compatible with: `s <= 64 B`."""
    if c.get("tag") in ("varargs", "recurse"):
        return "called"
    if c.get("tag") == "forceinline":
        return "inlined"
    return "inlined" if s_bytes(c["k"]) <= 64 else "called"


def r_ob(c):
    """R4 = the /Ob-level rival: inline everything except at /Ob0."""
    if c.get("tag") == "varargs":
        return "called"
    return "called" if c["mode"] == "O1Ob0" else "inlined"


def r_nosites(c):
    """R5 = INLINE-P with SCHEDULE D removed — the ceiling only, no site count.

    Separates "the site count is an input" from "the size ceiling is".
    """
    if c.get("tag") == "varargs":
        return "called"
    if c.get("tag") == "forceinline":
        return "inlined"
    if c.get("tag") == "recurse":
        return "called"
    idx = s_bytes(c["k"]) - 48
    return "inlined" if idx // 4 <= 64 else "called"


RIVALS = {
    "R1-INCUMBENT": r_incumbent,
    "R2-CEILING": r_ceiling,
    "R3-SIZE64": r_size64,
    "R4-OBLEVEL": r_ob,
    "R5-NOSITES": r_nosites,
}


# --------------------------------------------------------------------------


def cal(d, repo):
    """Measure the CALLEE's own emitted size `s` at /O1, per ladder rung."""
    os.makedirs(os.path.join(d, "cal"), exist_ok=True)
    env = dict(os.environ)
    env.setdefault("C2RS_WIBO",
                   os.environ.get("C2RS_WIBO", "../wibo/build/release/wibo"))
    env.setdefault("C2RS_COMPILERS", os.path.join(repo, "compilers"))
    sys.path.insert(0, os.path.join(repo, "scripts"))
    from gt_dump import Obj
    out = {}
    for k in sorted(set(LADDER) | {6}):
        src = PRELUDE + callee("cg", k, "extern")
        cpp = os.path.join(d, "cal", "cal_k%d.cpp" % k)
        open(cpp, "w").write(src)
        p = subprocess.run(
            [os.path.join(repo, "scripts", "gt_capture.sh"), cpp,
             "/O1", "/GS-", "/c"],
            capture_output=True, text=True, env=env)
        path = p.stdout.strip().splitlines()[-1] if p.stdout.strip() else ""
        if not path or not os.path.exists(path):
            out[str(k)] = None
            print("  k=%-4d CAPTURE FAILED" % k)
            continue
        o = Obj(open(path, "rb").read())
        sz = None
        for sy in o.symbols:
            if sy["sec"] > 0 and (sy["name"].startswith("?cg@")
                                  or sy["name"] in ("cg", "_cg")):
                sz = o.sections[sy["sec"] - 1]["rawsize"]
        out[str(k)] = sz
        os.remove(path)
        print("  k=%-4d s=%s B  (%s words)"
              % (k, sz, sz // 4 if sz else "?"))
    json.dump(out, open(os.path.join(d, "calib.json"), "w"), indent=1)


def gen(d):
    global CAL
    cp = os.path.join(d, "calib.json")
    if os.path.exists(cp):
        CAL = {int(k): v for k, v in json.load(open(cp)).items() if v}
        print("calibration loaded: %d rungs" % len(CAL))
    else:
        print("WARNING: no calib.json — predictions would use the REFUTED "
              "linear guess. Run `cal` first.")
        return 2
    os.makedirs(os.path.join(d, "src"), exist_ok=True)
    cells = build_cells()
    for c in cells:
        open(os.path.join(d, "src", c["id"] + ".cpp"), "w").write(c["src"])
    table = []
    for c in cells:
        row = {k: v for k, v in c.items() if k != "src"}
        row["pred"] = {name: fn(c) for name, fn in RIVALS.items()}
        table.append(row)
    json.dump(table, open(os.path.join(d, "frozen.json"), "w"), indent=1)
    sep = separation(table)
    json.dump(sep, open(os.path.join(d, "separation.json"), "w"), indent=1)
    print("cells: %d" % len(cells))
    print("pair separation (discriminating cells):")
    worst = 10 ** 9
    names = list(RIVALS)
    for i, a in enumerate(names):
        for b in names[i + 1:]:
            n = sep["%s|%s" % (a, b)]
            worst = min(worst, len(n))
            print("  %-26s %-26s %4d" % (a, b, len(n)))
    print("MINIMUM OVER ALL PAIRS: %d" % worst)
    return 0 if worst >= 4 else 1


def separation(table):
    out = {}
    names = list(RIVALS)
    for i, a in enumerate(names):
        for b in names[i + 1:]:
            out["%s|%s" % (a, b)] = [
                r["id"] for r in table if r["pred"][a] != r["pred"][b]]
    return out


def run(d, repo):
    table = json.load(open(os.path.join(d, "frozen.json")))
    env = dict(os.environ)
    env["C2RS_WIBO"] = env.get(
        "C2RS_WIBO", os.environ.get("C2RS_WIBO", "../wibo/build/release/wibo"))
    env["C2RS_COMPILERS"] = env.get(
        "C2RS_COMPILERS", os.path.join(repo, "compilers"))
    out = {}
    for i, r in enumerate(table):
        cpp = os.path.join(d, "src", r["id"] + ".cpp")
        p = subprocess.run(
            [os.path.join(repo, "scripts", "gt_capture.sh"), cpp]
            + MODES[r["mode"]].split(),
            capture_output=True, text=True, env=env)
        path = p.stdout.strip().splitlines()[-1] if p.stdout.strip() else ""
        if not path or not os.path.exists(path):
            out[r["id"]] = {"verdict": "absent", "why": "compile failed"}
            continue
        out[r["id"]] = verdict(open(path, "rb").read(), r)
        os.remove(path)
        if (i + 1) % 25 == 0:
            sys.stderr.write("  %d/%d\n" % (i + 1, len(table)))
    json.dump(out, open(os.path.join(d, "measured.json"), "w"), indent=1)
    print("measured %d cells" % len(out))


def verdict(data, r):
    sys.path.insert(0, os.path.join(os.environ.get("C2RS_REPO", ""), "scripts"))
    from gt_dump import Obj
    o = Obj(data)
    callees = ["sh%d" % i for i in range(6)] if r.get("multi") else ["cg"]

    def is_(nm, base):
        # c2 emits C++ mangled names: `?cg@@YAHH@Z`.
        return nm == base or nm == "_" + base or nm.startswith("?" + base + "@")

    csec = None
    csize = None
    for sy in o.symbols:
        if sy["sec"] > 0 and is_(sy["name"], "cf"):
            csec = o.sections[sy["sec"] - 1]
        if sy["sec"] > 0 and is_(sy["name"], "cg"):
            csize = o.sections[sy["sec"] - 1]["rawsize"]
    if csec is None:
        return {"verdict": "absent", "why": "no caller symbol"}
    names = set()
    for (_, symidx, _t) in o.relocs(csec):
        sy = o.sym_by_index(symidx)
        if sy:
            names.add(sy["name"])
    hit = sorted(cn for cn in callees if any(is_(nm, cn) for nm in names))
    body = len(o.raw(csec))
    if r.get("multi"):
        v = ("called" if len(hit) == len(callees)
             else ("inlined" if not hit else "partial"))
        return {"verdict": v, "called": hit, "bytes": body}
    return {"verdict": "called" if hit else "inlined", "bytes": body,
            "s_callee": csize}


def score(d):
    table = {r["id"]: r for r in json.load(open(os.path.join(d, "frozen.json")))}
    meas = json.load(open(os.path.join(d, "measured.json")))
    unmet = [k for k, v in meas.items() if v["verdict"] in ("absent",)]
    graded = [k for k in table if k not in unmet]
    print("cells %d · graded %d · ASSUMPTION UNMET %d"
          % (len(table), len(graded), len(unmet)))
    for u in unmet:
        print("  UNMET %s: %s" % (u, meas[u].get("why", "")))
    print()
    for name in RIVALS:
        ok = sum(1 for k in graded if table[k]["pred"][name] == meas[k]["verdict"])
        print("  %-26s %4d / %-4d" % (name, ok, len(graded)))
    print()
    print("misses per rival (first 20):")
    for name in RIVALS:
        bad = [k for k in graded if table[k]["pred"][name] != meas[k]["verdict"]]
        print("  %s: %d" % (name, len(bad)))
        for k in bad[:20]:
            print("     %-22s pred=%-8s meas=%-8s (%s)"
                  % (k, table[k]["pred"][name], meas[k]["verdict"],
                     meas[k].get("bytes")))
    # re-assert separation over the graded cells only
    sep = json.load(open(os.path.join(d, "separation.json")))
    worst = 10 ** 9
    for pair, ids in sep.items():
        live = [i for i in ids if i in graded]
        worst = min(worst, len(live))
    print("\nseparation over GRADED cells, minimum over all pairs: %d" % worst)


if __name__ == "__main__":
    cmd = sys.argv[1]
    if cmd == "cal":
        cal(sys.argv[2], sys.argv[3])
    elif cmd == "gen":
        sys.exit(gen(sys.argv[2]))
    elif cmd == "run":
        os.environ["C2RS_REPO"] = sys.argv[3]
        run(sys.argv[2], sys.argv[3])
    elif cmd == "score":
        score(sys.argv[2])
