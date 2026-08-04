#!/usr/bin/env python3
"""Lane w-bss2: grade `OBJ_DATA_BSS_SHAPE.md` §5 against the REAL workload objs.

Input side (the allocator's input) comes from the IL `.gl` — glcensus.jsonl.
Output side (what c2 actually emitted) comes from the previous lane's obj census
— work/w-bss/census/sections.jsonl.  Nothing here compiles or constructs an obj.

Scores R0..R3 of `docs/rungs/_2026-08-04-w-bss2-prereg.md`.
"""
import json, os, sys, collections

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import models, glparse, paths, prov

SEC = paths.SECTIONS
GLC = os.path.join(HERE, "glcensus.jsonl")

# ---------------------------------------------------------------- incumbents
# The graded POPULATION, not a rate.  w-repro measured that moving the corpus to
# another directory leaves every printed rate healthy (94.0 % -> 93.5 %, 100 % ->
# 100 %) while the denominator quietly loses 20 %: MSVC's `?A0x<hash>`
# anonymous-namespace mangling is path-derived, so 48 TUs' symbols stop joining.
# `docs/STATUS.md` trap 5 -- absence reads as success unless something forbids
# it.  This is the thing that forbids it.
INCUMBENT = {
    "bss": dict(cells=117, skipped=4),
    "data": dict(cells=68, skipped=2),
}
INCUMBENT_AT = ("dc3 dd9a4bdc..940d07dc at ../dc3-decomp; "
                "docs/OBJ_DATA_BSS_SHAPE.md, rungs/2026-08-04-w-bss2.md")
# The same two numbers measured at a DIFFERENT directory, so the failure this
# check exists to catch is named with its own magnitude and cannot be mistaken
# for noise.
MOVED_PATH = dict(bss=(93, 28), data=(53, 17))


def check_provenance(strict=True):
    """Refuse to join two censuses that were not taken against the same corpus,
    at the same directory, with the same flags.

    Returns the two stamps.  Raises SystemExit(2) rather than returning a
    boolean: a checker that returns False is one forgotten `if` away from being
    a no-op, which is how this project got here.
    """
    try:
        a = prov.read(SEC)
        b = prov.read(GLC)
        prov.require_join(SEC, a, GLC, b)
        prov.require_input(b, "sections_sha256", SEC)
    except prov.ProvError as e:
        if strict:
            print(prov.banner(e), file=sys.stderr)
            print("\nRe-run with --no-prov-check to grade anyway. The scores\n"
                  "will be tagged UNVERIFIED, because they are.", file=sys.stderr)
            sys.exit(2)
        print(prov.banner(e), file=sys.stderr)
        return None, None
    print("provenance OK")
    print("  sections.jsonl  %s" % prov.describe(a))
    print("  glcensus.jsonl  %s" % prov.describe(b))
    return a, b


def load():
    secs = {json.loads(l)["src"]: json.loads(l) for l in open(SEC)}
    gls = {}
    for l in open(GLC):
        r = json.loads(l)
        if "err" not in r:
            gls[r["src"]] = r
    return secs, gls


def match(sec_syms, glrecs):
    by = {}
    for r in glrecs:
        by.setdefault(r["n"], r)
        if r["n"].startswith("$"):
            by.setdefault(r["n"][1:], r)
    return [(sy, by[sy["n"]]) for sy in sec_syms if sy["n"] in by]


def is_deferred(name, dyn):
    """A `??__E<path>@@YAXXZ` thunk, a `??__E<decorated>@@YAXXZ` one (class
    static members), or a `$<name>$initializer$` data record."""
    return (glparse.path_of(name) in dyn or name in dyn
            or (name.startswith("$") and name[1:] in dyn))


def build(secs, gls, kind):
    cells, skipped = [], collections.Counter()
    for src, rec in secs.items():
        g = gls.get(src)
        if not g:
            continue
        dyn = set(g["init"])
        for e in rec[kind]:
            if e["comdat"] or len(e["syms"]) < 2:
                continue
            m = match(e["syms"], g["keep"])
            if len(m) != len(e["syms"]):
                skipped["symbol absent from .gl"] += 1
                continue
            recs = [r for _, r in m]
            if len({r["n"] for r in recs}) != len(recs):
                skipped["ambiguous name match"] += 1
                continue
            c = dict(src=src, size=e["size"], recs=recs,
                     meta={r["n"]: (r["sz"], r["al"]) for r in recs},
                     obs={r["n"]: sy["v"] for sy, r in m},
                     defer={r["n"] for r in recs if is_deferred(r["n"], dyn)})
            c["bump"] = bump_order(c)
            cells.append(c)
    return cells, skipped


def bump_order(c):
    """The ascending-address order iff the layout is EXACTLY a bump allocation
    in that order with no unaccounted bytes; else None.

    This is the discriminating test: hole reuse, pass-over and best-fit all
    produce a layout that IS a bump in *some* order, so if this passes, the
    allocator question reduces entirely to the walk order.
    """
    names = sorted(c["obs"], key=lambda n: c["obs"][n])
    cur = 0
    for n in names:
        sz, nat = c["meta"][n]
        p = (cur + models.al_natsz(sz, nat) - 1) & ~(models.al_natsz(sz, nat) - 1)
        if p != c["obs"][n]:
            return None
        cur = p + sz
    return names if cur == c["size"] else None


def walks(c):
    R, D = c["recs"], c["defer"]
    gl = [r["n"] for r in sorted(R, key=lambda r: r["i"])]
    gid = [r["n"] for r in sorted(
        R, key=lambda r: (1 << 60 if r["gid"] is None else r["gid"], r["i"]))]

    def split(o):
        return ([n for n in o if n not in D]
                + [n for n in o if n in D][::-1])
    return {"A1 (.gl order, deferred reversed last)": split(gl),
            "A1 (id order, deferred reversed last)": split(gid),
            ".gl file order, no split": gl,
            "ascending id, no split": gid}


def population_check(kind, cells, skipped):
    """Compare a COUNT, never a status (`docs/STATUS.md` trap 5's mitigation).

    Returns True when the graded population is the incumbent one.  The rates are
    NOT consulted: the whole point is that they stay healthy while this moves.
    """
    inc = INCUMBENT[kind]
    n = len(cells)
    absent = skipped.get("symbol absent from .gl", 0)
    total = n + sum(skipped.values())
    share = 100.0 * absent / max(1, total)
    inc_share = (100.0 * inc["skipped"]
                 / max(1, inc["cells"] + inc["skipped"]))
    ok = (n == inc["cells"])
    print("  POPULATION %-4s graded %3d  incumbent %3d  %s"
          % (kind, n, inc["cells"],
             "OK" if ok else "*** CHANGED by %+d (%+.1f %%) ***"
             % (n - inc["cells"],
                100.0 * (n - inc["cells"]) / inc["cells"])))
    if absent:
        flag = "WARNING" if share > inc_share + 1e-9 else "note"
        print("  %s  'symbol absent from .gl': %d of %d candidate sections "
              "(%.1f %%; incumbent %.1f %%)"
              % (flag, absent, total, share, inc_share))
        if flag == "WARNING":
            mp_cells, mp_absent = MOVED_PATH[kind]
            print("           A rising share here is the path-bound join "
                  "failing symbol by symbol.")
            print("           Measured at a moved corpus path: %s graded %d "
                  "(vs %d), absent %d (vs %d)."
                  % (kind, mp_cells, inc["cells"], mp_absent, inc["skipped"]))
    if not ok:
        print("           The RATES below are computed on this smaller "
              "population and will look fine.")
        print("           Incumbent measured at: %s" % INCUMBENT_AT)
    return ok


def main():
    strict = "--no-prov-check" not in sys.argv
    if not strict:
        bar = "=" * 72
        print(bar)
        print("UNVERIFIED — provenance checking was disabled with "
              "--no-prov-check.")
        print("Every score below is a claim about a corpus nobody wrote down.")
        print(bar)
    check_provenance(strict)
    secs, gls = load()
    global POP_OK
    POP_OK = True

    r0_ok = r0_n = 0
    for src, rec in secs.items():
        g = gls.get(src)
        if not g:
            continue
        for kind in ("data", "bss"):
            for e in rec[kind]:
                if e["comdat"] and len(e["syms"]) == 1:
                    m = match(e["syms"], g["keep"])
                    if m:
                        r0_n += 1
                        r0_ok += (m[0][1]["sz"] == e["size"])
    print("R0  .gl size == COMDAT SizeOfRawData: %d/%d = %.2f%%   [registered >=95%%]"
          % (r0_ok, r0_n, 100.0 * r0_ok / max(1, r0_n)))

    for kind in ("bss", "data"):
        cells, skipped = build(secs, gls, kind)
        pure = [c for c in cells if c["bump"]]
        print("\n=== %s: %d non-COMDAT sections with >=2 symbols   skipped %s"
              % (kind, len(cells), dict(skipped)))
        POP_OK &= population_check(kind, cells, skipped)
        print("  pure bump allocation in ascending-address order: %d/%d"
              % (len(pure), len(cells)))
        rows = []
        for wname in walks(cells[0]):
            for mname, f in models.ALL.items():
                ok = 0
                for c in cells:
                    pred, tot = f(walks(c)[wname], c["meta"])
                    ok += all(pred[n] == c["obs"][n] for n in c["obs"]) and tot == c["size"]
                rows.append((ok, wname, mname))
        rows.sort(reverse=True)
        seen = set()
        for ok, wname, mname in rows:
            if wname in seen:
                continue
            seen.add(wname)
            tag = "REG" if mname in models.REGISTERED else "exp"
            print("   best for %-38s %s %-30s %3d/%d = %.1f%%"
                  % (wname, tag, mname, ok, len(cells), 100.0 * ok / len(cells)))
        # walk-order-only score, on the cells where the bump test already passed
        print("  walk order alone, on the %d pure-bump cells:" % len(pure))
        for wname in walks(cells[0]):
            ok = sum(1 for c in pure if walks(c)[wname] == c["bump"])
            print("     %-40s %3d/%d = %.1f%%"
                  % (wname, ok, len(pure), 100.0 * ok / len(pure)))
        wkey = ("A1 (.gl order, deferred reversed last)" if kind == "bss"
                else "A1 (id order, deferred reversed last)")
        for lab, sel in (("uniform size", lambda c: len({s for s, _ in c["meta"].values()}) == 1),
                         ("mixed size", lambda c: len({s for s, _ in c["meta"].values()}) > 1),
                         ("no deferred obj", lambda c: not c["defer"]),
                         ("has deferred obj", lambda c: bool(c["defer"])),
                         ("2 objects", lambda c: len(c["obs"]) == 2),
                         (">2 objects", lambda c: len(c["obs"]) > 2)):
            sub = [c for c in pure if sel(c)]
            ok = sum(1 for c in sub if walks(c)[wkey] == c["bump"])
            print("       [%s] %-16s %3d/%3d" % (wkey.split()[0], lab, ok, len(sub)))
        if kind == "bss":
            r3(cells)


def r3(cells):
    mixed = [c for c in cells if c["defer"] and len(c["defer"]) < len(c["obs"])]
    bad = [c["src"] for c in mixed
           if min(c["obs"][n] for n in c["defer"])
           <= max(c["obs"][n] for n in c["obs"] if n not in c["defer"])]
    print("\nR3  sections mixing eager and deferred: %d;  ADDRESS interleaving: %d"
          % (len(mixed), len(bad)))
    for s in bad[:10]:
        print("      %s" % s)
    multi = [c for c in cells if len(c["defer"]) > 1]
    ok = sum(1 for c in multi
             if sorted(c["defer"], key=lambda n: c["obs"][n])
             == [n for n in sorted(c["recs"], key=lambda r: r["i"])
                 and [r["n"] for r in sorted(c["recs"], key=lambda r: r["i"])]
                 if n in c["defer"]][::-1])
    print("    deferred block ascending address == reverse .gl order: %d/%d" % (ok, len(multi)))


if __name__ == "__main__":
    POP_OK = True
    main()
    if not POP_OK and "--allow-population-change" not in sys.argv:
        print("\n" + "=" * 72, file=sys.stderr)
        print("GRADED POPULATION CHANGED — exiting 3.", file=sys.stderr)
        print("The rates above are real, and they are about a different set of\n"
              "sections than the incumbent. A percentage that stays healthy\n"
              "while its denominator shrinks is docs/STATUS.md trap 5; this is\n"
              "the thing that forbids it. If the change is intended (a genuinely\n"
              "new corpus), re-run with --allow-population-change and RECORD the\n"
              "new population as the incumbent.", file=sys.stderr)
        print("=" * 72, file=sys.stderr)
        sys.exit(3)
