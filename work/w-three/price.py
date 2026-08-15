#!/usr/bin/env python3
"""w-three — the per-TU PRICE LADDER for the three reader-clear TUs.

Tooling, outside the std-only workspace (same status as scripts/plot_perf.py).
It joins rows a `c2rs gap` scan already wrote; it computes nothing the compiler
does not, decides nothing, and is on no acceptance path.

WHAT IT REFUSES RATHER THAN REPORTING A NULL
--------------------------------------------
`w-loo`'s zero-reach guard is the precedent: without it, its mutant printed 52
margins of 0 and read as a clean null. Four independent refusals here:

  R1  fewer than 800 TU rows in the stream            (a truncated scan)
  R2  a requested TU is absent from the stream        (a typo reads as a null)
  R3  a required `emit` key is missing from EVERY row (a renamed instrument)
  R4  every requested row has an empty `emit` map     (a broken join)

Usage:  price.py <scan.jsonl> [--tu SRC]...
"""
import json
import sys

TARGETS = [
    "src/system/decomp_pch.cpp",
    "src/system/math/vec.cpp",
    "src/system/os/NetworkSocket.cpp",
]
# THE POSITIVE CONTROLS. M2 = a `match` TU (must show `decodes`, no gate cause).
# M3 = a `frontier` TU (A/\B/\C true, reader-blocked) — a THIRD distinct profile.
# A probe that renders these the same as the targets is measuring nothing.
CONTROL_MATCH = ["src/system/math/Primes.cpp", "src/system/math/Sort.cpp"]
CONTROL_MATCH_E = ["src/Main.cpp"]
CONTROL_FRONTIER = ["src/system/rndobj/wordwrap.cpp", "src/keygen_xbox.cpp"]

REQUIRED_EMIT_KEYS = ["emit-emitted", "emit-gate-segments"]


def die(msg):
    print(f"REFUSE: {msg}", file=sys.stderr)
    sys.exit(2)


def load(path):
    rows = {}
    for line in open(path):
        r = json.loads(line)
        s = r.get("src")
        if s:
            rows[s] = r
    if len(rows) < 800:
        die(f"{path}: {len(rows)} TU rows (< 800) — a truncated scan, not a result [R1]")
    seen = set()
    for r in rows.values():
        seen |= set(r.get("emit", {}))
    missing = [k for k in REQUIRED_EMIT_KEYS if k not in seen]
    if missing:
        die(f"{path}: emit keys absent from EVERY row: {missing} — a renamed instrument [R3]")
    return rows


def g(r, k):
    return r.get("emit", {}).get(k, 0)


def letters(tsv, src):
    for line in open(tsv):
        if line.startswith("#"):
            continue
        f = line.rstrip("\n").split("\t")
        if f and f[0] == src:
            return f[-1]
    return "?"


def ladder(r, tsv):
    src = r["src"]
    seg = r.get("fn_total") or 0
    gbs = r.get("gl_body_starts") or (0, 0)
    sb = r.get("selective_bind") or (0, 0, 0, 0)
    out = []
    out.append(f"### {src}")
    out.append(f"    class            {r['class']}      factor letters  {letters(tsv, src)}")
    out.append(f"    gate stops at    {r.get('gate_cause')}")
    out.append(f"    causes EVALUATED {r.get('gate_causes')}   (downstream of the binding is NOT evaluated)")
    out.append(f"    .ex segments     {seg}      .gl names {r.get('fn_names')}   in-class bodies {r.get('fn_in_class')}")
    out.append(f"    .gl SPELLS a body-start for {gbs[0]} of {gbs[1]}  -> {gbs[1]-gbs[0]} segments can bind to NO record")
    out.append(f"    a .gl record NAMES {sb[0]} of {sb[1]}  (unclaimed mangled {sb[2]}, unclaimed inline-fit {sb[3]})")
    # data_tu clause 1, decided from the same segment count the gate uses.
    dt = "REFUSED at clause 1 (is_empty_module false)" if seg > 0 else "clause 1 passes"
    out.append(f"    IlBundle::data_tu (factor E, the functionless-data path): {dt}")
    out.append(f"    emitted .text COMDATs {g(r,'emit-emitted')}   in-class {g(r,'emit-in-class')}"
               f"   bound {g(r,'emit-bound')}   unbound-with-record {g(r,'emit-unbound-has-record')}"
               f"   unbound-NO-record {g(r,'emit-unbound-no-record')}")
    out.append(f"    fnbyte  denominator {g(r,'fnbyte-denominator')}  exact {g(r,'fnbyte-exact')}"
               f"  unbound {g(r,'fnbyte-unbound')}  differs {g(r,'fnbyte-differs')}"
               f"  refused-parse {g(r,'fnbyte-refused-parse')}")
    bden, bex = g(r, "bytefrac-denominator"), g(r, "bytefrac-exact")
    if bden:
        bpct = 100.0 * bex / bden
        out.append(f"    BYTES   denominator {bden}  exact {bex} ({bpct:.2f} %)"
                   f"  unaccounted {g(r,'bytefrac-unaccounted')}")
        # **The guard `w-vocabgap`'s M2 earned.** A TU that matches through
        # factor E carries reader-refused emitted functions its whole-TU
        # recognizer never lowers, so BOTH `fnbyte-exact` and `bytefrac-exact`
        # are 0 against a positive denominator — and the first draft of this
        # line divided by it and produced a traceback where a number or a
        # refusal belongs. `src/Main.cpp` is that TU and it is in the cohort as
        # a control, which is the only reason the defect was seen.
        if g(r, "fnbyte-denominator") and bex:
            fnpct = 100.0 * g(r, "fnbyte-exact") / g(r, "fnbyte-denominator")
            out.append(f"    ** the two readings of 'how done is this TU': "
                       f"FUNCTIONS {fnpct:.1f} %  vs  BYTES {bpct:.2f} %"
                       f"  -> {fnpct/bpct:.1f}x apart **")
        else:
            out.append(f"    ** the FUNCTIONS/BYTES ratio is NOT COMPUTED here: "
                       f"fnbyte-denominator {g(r,'fnbyte-denominator')}, "
                       f"bytefrac-exact {bex} — a ratio on a zero denominator is a "
                       f"refusal, not a 0 **")
    else:
        out.append(f"    BYTES   no denominator — the reference obj has NO .text bytes at all")
    secs = sorted(k.split("|", 1)[1] for k in r.get("emit", {}) if k.startswith("emit-sec-name|"))
    out.append(f"    obj sections {g(r,'emit-sec-count')} ({g(r,'emit-sec-distinct')} distinct): {secs}")
    out.append(f"    emit_blockers keys {len(r.get('emit_blockers', {}))}"
               f"   (= 'never asked' when the gate stops before the emitted loop)")
    out.append(f"    factor A residue: rows emitted {g(r,'afail-row-emitted')}"
               f"  rows NOT emitted {g(r,'afail-row-not-emitted')}"
               f"  rows unnamed {g(r,'afail-row-unnamed')}")
    return "\n".join(out)


def main():
    argv = sys.argv[1:]
    if not argv:
        die("usage: price.py <scan.jsonl> [--tu SRC]...")
    path = argv[0]
    tsv = path.rsplit(".", 1)[0] + ".tsv"
    extra = [argv[i + 1] for i, a in enumerate(argv) if a == "--tu"]
    rows = load(path)

    groups = [
        ("THE THREE — reader-clear, not `match` (#3184, #3191)", TARGETS + extra),
        ("POSITIVE CONTROL M2 — `match` through factor D", CONTROL_MATCH),
        ("POSITIVE CONTROL M2b — `match` through factor E (whole-TU recognizer)", CONTROL_MATCH_E),
        ("POSITIVE CONTROL M3 — the FRONTIER (A/\\B/\\C true, reader-blocked)", CONTROL_FRONTIER),
    ]
    picked = []
    for title, srcs in groups:
        print("=" * 78)
        print(f"== {title}")
        print("=" * 78)
        for s in srcs:
            if s not in rows:
                die(f"{s} is not in {path} — a requested TU absent reads as a null [R2]")
            picked.append(rows[s])
            print(ladder(rows[s], tsv))
            print()
    if not any(r.get("emit") for r in picked):
        die("every requested row has an EMPTY emit map — a broken join [R4]")

    # DISCRIMINATING CELLS — printed beside every number, never inferred.
    print("=" * 78)
    print("== DISCRIMINATING CELLS")
    print("=" * 78)
    prof = {}
    for r in picked:
        key = (r["class"], str(r.get("gate_cause")), tuple(r.get("gate_causes") or []))
        prof.setdefault(key, []).append(r["src"])
    print(f"distinct (class, gate_cause, gate_causes) profiles over {len(picked)} TUs: {len(prof)}")
    for k, v in sorted(prof.items(), key=lambda kv: -len(kv[1])):
        print(f"  {k[0]:<10} stop={k[1]:<24} all={list(k[2])}")
        for s in v:
            print(f"      {s}")
    empt = [r["src"] for r in picked if not r.get("emit_blockers")]
    nonempt = [r["src"] for r in picked if r.get("emit_blockers")]
    print(f"\nM7 — `emit_blockers` EMPTY on {len(empt)} of {len(picked)}, NON-EMPTY on {len(nonempt)}")
    print(f"     empty:     {empt}")
    print(f"     non-empty: {nonempt}")
    same_class = [r["src"] for r in picked
                  if r["class"] == "vocab-gap" and r.get("emit_blockers")]
    if not same_class:
        die("M7 has no witness: no `vocab-gap` TU with a NON-EMPTY blocker map in the "
            "cohort, so 'empty means never asked' is unfalsifiable here")
    print(f"     `vocab-gap` WITH keys (the M7 witness that empty != nothing-blocks): {same_class}")


if __name__ == "__main__":
    main()
