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
import models, glparse

MAIN = "/home/free/code/milohax/c2-rs"
SEC = os.path.join(MAIN, "work/w-bss/census/sections.jsonl")
GLC = os.path.join(HERE, "glcensus.jsonl")


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


def main():
    secs, gls = load()

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
    main()
