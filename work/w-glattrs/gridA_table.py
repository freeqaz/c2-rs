#!/usr/bin/env python3
"""gridA_table.py — re-derive GRID-A's table from the captured `.gl` files.

Never accumulated: `twins.py` writes the artifacts, this reads them back.

The two twins differ in the source by `__declspec(noinline) ` vs 21 spaces, so
they are byte-length-identical and compiled from one path. Their `.gl` files
therefore differ in exactly three places, and each is named rather than lumped:

  * offset 22 — a one-byte length/flag that tracks the ATTR field's own width;
  * a 16-byte block — the source content hash;
  * **the ATTR varU**, which is where the experiment is.

`__declspec(noinline)` clears bit `0x40` of ATTR's low byte AND pushes the varU
value over `0x8000`, so the field goes from 2 bytes to 4 and the `.gl` grows by
exactly 2 — which is `w-target`'s nicmp2 observation *"only `.gl` moves, by 2
bytes"*, with the mechanism attached.

The falsifiable claim: **the first differing byte after the hash block is at the
ATTR offset the 3-byte escape decode predicts, and it differs by exactly 0x40.**
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import glrec  # noqa: E402

CALLEE = "?callee@@YAHH@Z"
HASH_LO, HASH_HI = 160, 180  # the 16-byte source-hash block, bracketed loosely


def rec_of(ildir):
    f = [x for x in os.listdir(ildir) if x.endswith(".gl")][0]
    gl = open(os.path.join(ildir, f), "rb").read()
    for v, r in glrec.walk(gl, glrec.framed_incumbent):
        if v == "ok" and r["name"] == CALLEE:
            return gl, r
    return gl, None


def main():
    base = os.path.join(HERE, "gridA", "il")
    tags = sorted(os.listdir(base))
    cells = {}
    for t in tags:
        n = int(t[3:6])
        kind = t[7:9]
        prof = t[10:]
        cells.setdefault((prof, n), {})[kind] = t
    print(f"{'prof':>4} {'n':>3} {'SIZE':>5} {'form':>7} {'attrOff':>8} {'Δoff':>5} "
          f"{'attrPl':>7} {'attrNi':>7} {'xor':>4} {'Δlen':>5} {'firstDiffPastHash':>18} {'HIT':>4}")
    ok = tot = 0
    prev_off = {}
    for (prof, n) in sorted(cells, key=lambda k: (k[0], k[1])):
        c = cells[(prof, n)]
        if "pl" not in c or "ni" not in c:
            continue
        ga, ra = rec_of(os.path.join(base, c["pl"]))
        gb, rb = rec_of(os.path.join(base, c["ni"]))
        if ra is None or rb is None:
            print(f"{prof:>4} {n:>3}  no callee record")
            continue
        ao = ra["attr_off"]
        diffs = [i for i in range(min(len(ga), len(gb))) if ga[i] != gb[i]]
        past = [i for i in diffs if i > HASH_HI]
        first = past[0] if past else None
        xor = ga[ao] ^ gb[ao]
        hit = (first == ao) and xor == 0x40
        ok += hit
        tot += 1
        # The offset the ESCAPE costs, relative to the direct-form record at the
        # same `p`: the black-box read of the escape's width.
        doff = ao - ra["p"] - 5
        print(f"{prof:>4} {n:>3} {ra['size']:>5} {ra['form']:>7} {ao:>8} {doff:>5} "
              f"{ra["attr"]:>7x} {rb["attr"]:>7x} {xor:>4x} "
              f"{len(gb) - len(ga):>5} {str(first):>18} {str(hit):>4}")
    print(f"\ndiff-at-predicted-offset AND xor==0x40: {ok}/{tot}")
    # The endianness proof, black box: the SIZE ladder's slope must not break at
    # the escape boundary. `mix` steps 12 per statement.
    print("\nThe SIZE ladder across the direct/escape boundary (endianness):")
    for prof in ("O1", "Ox"):
        row = []
        for n in sorted(x[1] for x in cells if x[0] == prof):
            _, r = rec_of(os.path.join(base, cells[(prof, n)]["pl"]))
            be = None
            if r["form"] == "escape":
                # the same three bytes read the other way round
                gl, _ = rec_of(os.path.join(base, cells[(prof, n)]["pl"]))
                q = r["attr_off"] - 3
                be = (gl[q + 1] << 8) | gl[q + 2]
            row.append((n, r["size"], r["form"], be))
        print(f"  {prof}:")
        for n, s, f, be in row:
            pred = 19 + 12 * n
            print(f"     n={n:>3} SIZE={s:>5} ({f:>7})  19+12n={pred:>5} "
                  f"{'OK' if s == pred else 'MISMATCH'}"
                  + (f"   big-endian would read {be}" if be is not None else ""))
    return 0


if __name__ == "__main__":
    sys.exit(main())
