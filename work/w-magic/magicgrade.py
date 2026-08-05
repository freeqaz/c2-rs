#!/usr/bin/env python3
"""magicgrade.py — is the `/Ox` magic multiplier DERIVABLE from `k`?

Lane **w-magic**, PREREG **R4**. R2 hit — `mulhw`/`mulhwu` does appear, at `/Ox`
and `/O2`, in `%` and never in `/`. This grades whether the multiplier is the
standard Granlund & Montgomery one, by **generating** `(M, s)` from `k` alone
(`kgrid.magic_signed` / `kgrid.magic_unsigned`, written out from the algorithm and
not read off any `c2` output) and comparing against the constant `c2` actually
materialized.

**The multiplier is recovered by constant-propagation, not by an offset.** #644
is the reason and this lane has now seen it four times live: at `/Ox` a cell like
`s-mod-100000` reads `lis lis ori ori mulhw …` — *two* interleaved materializations
— so "the `ori` after the `lis`" is not a rule that survives here. The propagator
tracks a value per register through `li`/`lis`/`ori` and then reads whichever
register the `mulhw` names.

    work/w-magic/magicgrade.py work/w-magic/fit_Ox.tsv
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import kgrid  # noqa: E402
import rule  # noqa: E402


def propagate(ws):
    """reg -> 32-bit constant, for as far as it can be tracked. Any instruction
    that writes a register the propagator does not model KILLS that register, so
    a stale value can never be read back as a constant."""
    regs = {}
    out = []
    for w in ws:
        m, _, f = kgrid.decode(w)
        d, a = f["D"], f["A"]
        if m == "li":
            regs[d] = f["SIMM"] & 0xFFFFFFFF
        elif m == "lis":
            regs[d] = (f["SIMM"] << 16) & 0xFFFFFFFF
        elif m == "ori":
            regs[a] = (regs.get(d, None) | f["UIMM"]) & 0xFFFFFFFF \
                if regs.get(d) is not None else None
        elif m in ("mulhw", "mulhwu"):
            out.append((m, regs.get(f["A"]), regs.get(f["B"])))
            regs[d] = None
        elif m in ("srawi",):
            out.append(("srawi", f["SH"], None))
            regs[a] = None
        elif m == "rlwinm":
            out.append(("rlwinm", (f["SH"], f["MB"], f["ME"]), None))
            regs[a] = None
        else:
            # Unmodelled: kill the destination. `subf`/`add`/`neg`/`mullw`/…
            # all write rD; `ori`-like forms write rA and are handled above.
            regs[d] = None
    return out


def main(argv):
    paths = [a for a in argv[1:] if not a.startswith("-")]
    if not paths:
        print(__doc__)
        return 0
    tot = magic = derived = differ = unrecovered = 0
    print("%-3s %-4s %12s %12s %12s %6s %6s %s"
          % ("sgn", "op", "k", "M(emitted)", "M(generated)", "s(emit)",
             "s(gen)", "verdict"))
    print("-" * 100)
    for p in paths:
        for ln in open(p).read().splitlines()[1:]:
            f = ln.split("\t")
            signed, op, k = f[1] == "s", f[2], int(f[3])
            ws = [int(x, 16) for x in f[7].split()]
            ev = propagate(ws)
            mh = [e for e in ev if e[0] in ("mulhw", "mulhwu")]
            tot += 1
            if not mh:
                continue
            magic += 1
            _, va, vb = mh[0]
            got = va if (va is not None and va not in (None,)) else vb
            # The dividend is the other operand and is not a constant, so
            # exactly one of the two should have propagated.
            cand = [v for v in (va, vb) if v is not None]
            if len(cand) != 1:
                unrecovered += 1
                print("%-3s %-4s %12d %12s %12s %6s %6s UNRECOVERED"
                      % (f[1], op, k, va, vb, "-", "-"))
                continue
            got = cand[0]
            if signed:
                M, s = kgrid.magic_signed(k)
                want = M & 0xFFFFFFFF
            else:
                M, s, _ = kgrid.magic_unsigned(k if k > 1 else 2)
                want = M & 0xFFFFFFFF
            shifts = [e[1] for e in ev if e[0] == "srawi"]
            se = shifts[0] if shifts else "-"
            v = "DERIVED" if got == want else "DIFFERS"
            if got == want:
                derived += 1
            else:
                differ += 1
            print("%-3s %-4s %12d %12s %12s %6s %6s %s"
                  % (f[1], op, k, "0x%08x" % got, "0x%08x" % want, se, s, v))
    print()
    print("rows %d · magic-bearing %d · DERIVED %d · DIFFERS %d · UNRECOVERED %d"
          % (tot, magic, derived, differ, unrecovered))
    if magic:
        print("derivable share: %d/%d = %.1f%%"
              % (derived, magic, 100.0 * derived / magic))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
