#!/usr/bin/env python3
"""The join, re-run at this lane's tip.

`CEILING.md` §12's table is `full coverage` (does `.gl` SPELL every segment's
body-start) intersected with the Phase-7 factor sets. This adds the two columns
a selective binding is actually about:

    NAMED      does a `.gl` RECORD name the segment (the gate's own framing)
    EMIT-SUBSET  is every symbol c2 EMITTED named by a record

and re-runs the whole table, by name, off this lane's own scan.

    work/w-selbind/join.py <scan.jsonl> <factors.tsv>
"""
import json
import sys


def main():
    rows = {}
    for line in open(sys.argv[1]):
        r = json.loads(line)
        if r.get("record") == "provenance":
            continue
        rows[r["src"]] = r

    # The header is a `# columns:` comment, not a row — every other `#` line is
    # prose. Parsed from the comment so the column ORDER is read and not assumed.
    fac = {}
    hdr = ["src", "class", "A", "B", "C", "D", "E", "letters"]
    for line in open(sys.argv[2]):
        if line.startswith("#"):
            if line.startswith("# columns:"):
                hdr = line.split(":", 1)[1].strip().split("<TAB>")
            continue
        f = line.rstrip("\n").split("\t")
        if len(f) < len(hdr):
            continue
        fac[f[hdr.index("src")]] = {k: v for k, v in zip(hdr, f)}

    def yes(s, k):
        v = fac.get(s, {}).get(k, "0")
        return v in ("1", "true", "yes", "Y")

    graded = [s for s, r in rows.items() if r["class"] != "capture-fail"]
    print("graded TUs: %d  (of %d scanned)" % (len(graded), len(rows)))
    print("factors-tsv columns: %s" % ",".join(hdr))

    def cover_full(s):
        v = rows[s].get("gl_body_starts")
        return bool(v) and v[0] == v[1]

    def named_full(s):
        v = rows[s].get("selective_bind")
        return bool(v) and v[0] == v[1] and v[0] > 0

    def named_some(s):
        v = rows[s].get("selective_bind")
        return bool(v) and 0 < v[0] < v[1]

    def sel_total(s):
        v = rows[s].get("selective_bind")
        return bool(v) and 0 < v[0] < v[1] and v[2] == 0 and v[3] == 0

    def emit_sub_gate(s):
        return rows[s].get("bind_checks", {}).get("selbind-emit-subset-gate-tus", 0) == 1

    def emit_sub_wide(s):
        return rows[s].get("bind_checks", {}).get("selbind-emit-subset-wide-tus", 0) == 1

    def has_emit(s):
        return rows[s].get("bind_checks", {}).get("selbind-emit-tus", 0) == 1

    A = {s for s in graded if yes(s, "A")}
    B = {s for s in graded if yes(s, "B")}
    C = {s for s in graded if yes(s, "C")}
    D = {s for s in graded if yes(s, "D")}
    E = {s for s in graded if yes(s, "E")}
    M = {s for s in graded if rows[s]["class"] == "match"}
    BC = B & C
    ABC = A & B & C
    FRONT = ABC - M
    REACH = BC - ABC

    pops = [
        ("match", M),
        ("A", A),
        ("B", B),
        ("C", C),
        ("D", D),
        ("E", E),
        ("B and C", BC),
        ("A and B and C", ABC),
        ("FRONTIER (ABC minus match)", FRONT),
        ("reach-pool (BC minus ABC)", REACH),
        ("D or E", D | E),
        ("ALL GRADED", set(graded)),
    ]
    print("\n%-30s %6s %10s %10s %10s %12s %12s"
          % ("population", "TUs", "cover=n/n", "named=n/n", "named some",
             "emit<=gate", "emit<=wide"))
    for name, p in pops:
        print("%-30s %6d %10d %10d %10d %12d %12d"
              % (name, len(p),
                 sum(1 for s in p if cover_full(s)),
                 sum(1 for s in p if named_full(s)),
                 sum(1 for s in p if named_some(s)),
                 sum(1 for s in p if emit_sub_gate(s)),
                 sum(1 for s in p if emit_sub_wide(s))))

    print("\nSELECTIVE population (a record names SOME but not ALL segments): %d"
          % sum(1 for s in graded if named_some(s)))
    for s in sorted(graded):
        if named_some(s):
            v = rows[s]["selective_bind"]
            print("   %-4s records %-5d segments %-6d unclaimed mangled %-5d inline-fit %-5d  %s"
                  % ("TOT" if sel_total(s) else "   ", v[0], v[1], v[2], v[3], s))

    print("\nTUs where EVERY emitted symbol is NAMED by a record the GATE's framing sees: %d of %d with any emitted symbol"
          % (sum(1 for s in graded if emit_sub_gate(s)),
             sum(1 for s in graded if has_emit(s))))
    for s in sorted(graded):
        if emit_sub_gate(s):
            v = rows[s].get("selective_bind")
            print("   %-8s records %-5d segments %-6d  %s" % (rows[s]["class"], v[0], v[1], s))

    wide_only = [s for s in graded if emit_sub_wide(s) and not emit_sub_gate(s)]
    print("\n…and under the WINDOW-FREE framing (board #2783, unshipped): %d, i.e. %d MORE"
          % (sum(1 for s in graded if emit_sub_wide(s)), len(wide_only)))
    print("   of those %d, in B and C: %d · in the reach-pool: %d · A: %d"
          % (len(wide_only),
             len([s for s in wide_only if s in BC]),
             len([s for s in wide_only if s in REACH]),
             len([s for s in wide_only if s in A])))


main()
