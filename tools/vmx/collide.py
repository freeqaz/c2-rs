#!/usr/bin/env python3
"""collide.py -- enumerate and MEASURE every way stock tooling lies about
VMX128.

Pure stdlib. Tooling, outside the std-only Rust workspace. Never a gate.

Two independent halves, kept separate on purpose because they answer different
questions and have different standing:

  EXACT (no sampling, no LLVM needed)
      Two (mask, pattern) rows admit a common 32-bit word iff
      `(patA ^ patB) & maskA & maskB == 0`. That is decidable, so the set of
      VMX128 encodings that are ALSO a legal non-VMX128 opcode-4/5/6 encoding
      is computed, not estimated. Source: the `vmx128_isa.py` tables.

  MEASURED (asks the real llvm-mc)
      For each VMX128 row, build concrete words and ask
      `llvm-mc --disassemble -triple=powerpc`. Three outcomes:
        SILENT     llvm-mc printed a legal instruction that is not the VMX128
                   one, and printed no diagnostic.        <- the dangerous one
        REFUSED    llvm-mc said "invalid instruction encoding".  <- safe
        AGREES     llvm-mc printed the same mnemonic (only possible where the
                   encoding really is plain AltiVec).
      The EXACT half cannot predict this: LLVM's PowerPC table contains
      Power10 encodings (`lxvp`, `stxvp`, `vucmprlb`) that `isa.yaml` -- a
      Gekko/Broadway/Xenon table -- does not carry at all. So the two halves
      find genuinely different collisions and both are reported.

Sampling is deliberately register-varied: a row is probed with several register
numbers, because VMX128's high register bits live inside opcode-extension space
and the answer LLVM gives CHANGES with the register number. `lvx128 vr1,...`
decodes as `vucmprlb`; `lvx128 vr63,...` is refused. A single sample per row
would report one of those and hide the other, which is exactly the failure this
lane is about.

Usage:
    tools/vmx/collide.py                 # exact + measured
    tools/vmx/collide.py --exact-only
    tools/vmx/collide.py --md            # markdown, for docs/VMX128_DECODE.md
"""
import argparse
import collections
import os
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, _HERE)

import llvmmc                        # noqa: E402
import vmx128                        # noqa: E402
import vmx128_isa as ISA             # noqa: E402

# Register numbers chosen to move every high bit of every split field:
# 0 (all high bits clear), 31 (low field full, high clear), 32 (high bit 0 set
# only), 63, 64 (VA128's bit-21 piece), 127 (everything set).
REGS = (0, 1, 31, 32, 63, 64, 96, 127)


def samples_for(name, mask, pattern):
    out = []
    for r in REGS:
        w = vmx128.sample_word(mask, pattern, vd=r, va=r, vb=r, vc=r % 8,
                               ra=r % 32, rb=r % 32, imm=r % 32)
        if w not in out:
            out.append(w)
    return out


def exact_collisions():
    rows = []
    for vname, vmask, vpat, _a in ISA.VMX128:
        hits = []
        for oname, omask, opat, src, _b in ISA.OPCODE456_OTHER:
            if vmx128.encodings_overlap(vmask, vpat, omask, opat):
                hits.append((oname, src))
        rows.append((vmx128.MS_MNEMONIC.get(vname, vname), vmask, vpat, hits))
    return rows


def measure(rows):
    mc = llvmmc.find_llvm_mc()
    if not mc:
        return None, None
    words, index = [], []
    for name, mask, pat, _h in rows:
        for w in samples_for(name, mask, pat):
            words.append(w)
            index.append((name, w))
    res = llvmmc.disassemble(words, mc=mc)
    per = collections.defaultdict(lambda: collections.Counter())
    detail = collections.defaultdict(list)
    for (name, w), (ok, txt) in zip(index, res):
        t = llvmmc.normalize(txt)
        if not ok:
            per[name]["REFUSED"] += 1
            detail[name].append((w, "REFUSED", t))
        else:
            lm = t.split(" ")[0]
            if lm == name:
                per[name]["AGREES"] += 1
                detail[name].append((w, "AGREES", t))
            else:
                per[name]["SILENT"] += 1
                detail[name].append((w, "SILENT", t))
    return per, detail


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--exact-only", action="store_true")
    ap.add_argument("--md", action="store_true", help="markdown table")
    a = ap.parse_args()

    rows = exact_collisions()
    n_with = sum(1 for _n, _m, _p, h in rows if h)

    per, detail = (None, None)
    if not a.exact_only:
        per, detail = measure(rows)

    if per is None and not a.exact_only:
        print("SKIP: llvm-mc absent -- exact half only", file=sys.stderr)

    if a.md:
        print("| VMX128 opcode | pattern/mask | collides with (isa.yaml, "
              "opcode 4/5/6) | llvm-mc: silent / refused / agrees |")
        print("|---|---|---|---|")
        for name, mask, pat, hits in rows:
            c = per.get(name, collections.Counter()) if per else {}
            h = ", ".join("`%s`" % n for n, _s in hits) or "—"
            m = ("%d / %d / %d" % (c.get("SILENT", 0), c.get("REFUSED", 0),
                                   c.get("AGREES", 0))) if per else "—"
            print("| `%s` | `%08x`/`%08x` | %s | %s |" % (name, pat, mask, h, m))
        return 0

    W = 44
    print("VMX128 collision analysis")
    print()
    print("EXACT half -- computed from the tables, no sampling")
    print("  %-*s %d" % (W, "VMX128 rows", len(rows)))
    print("  %-*s %d" % (W, "non-VMX128 opcode-4/5/6 rows",
                         len(ISA.OPCODE456_OTHER)))
    print("  %-*s %d" % (W, "VMX128 rows sharing a word with one", n_with))
    print("  %-*s %d" % (W, "colliding (VMX128, other) pairs",
                         sum(len(h) for _n, _m, _p, h in rows)))
    print()
    for name, mask, pat, hits in rows:
        if hits:
            print("  %-14s %08x/%08x  <-> %s" % (
                name, pat, mask,
                ", ".join("%s [%s]" % (n, s) for n, s in hits)))

    if per is None:
        print()
        print("MEASURED half: SKIP -- llvm-mc absent")
        return 0

    print()
    print("MEASURED half -- %s, %d words per row (registers %s)"
          % (llvmmc.version() or "llvm-mc", len(REGS),
             ",".join(str(r) for r in REGS)))
    tot = collections.Counter()
    for name, _m, _p, _h in rows:
        for k, v in per.get(name, {}).items():
            tot[k] += v
    n_words = sum(tot.values())
    print("  %-*s %d" % (W, "sampled VMX128 encodings", n_words))
    print("  %-*s %d   (%.1f%%)" % (W, "SILENTLY MIS-DECODED (no diagnostic)",
                                    tot["SILENT"],
                                    100.0 * tot["SILENT"] / max(n_words, 1)))
    print("  %-*s %d   (%.1f%%)" % (W, "refused (safe -- you get an error)",
                                    tot["REFUSED"],
                                    100.0 * tot["REFUSED"] / max(n_words, 1)))
    print("  %-*s %d" % (W, "agrees", tot["AGREES"]))
    print("  %-*s %d of %d" % (
        W, "rows with at least one SILENT sample",
        sum(1 for n, _m, _p, _h in rows if per.get(n, {}).get("SILENT")),
        len(rows)))
    print("  %-*s %d of %d" % (
        W, "rows that are ALWAYS silently wrong",
        sum(1 for n, _m, _p, _h in rows
            if per.get(n, {}).get("SILENT") and not per[n].get("REFUSED")),
        len(rows)))
    print()
    print("  what llvm-mc says instead, by frequency:")
    what = collections.Counter()
    for name in detail:
        for _w, verdict, t in detail[name]:
            if verdict == "SILENT":
                what[t.split(" ")[0]] += 1
    for k, v in what.most_common(20):
        print("    %-16s %d" % (k, v))
    print()
    print("  per row (silent/refused/agrees), and one silent example:")
    for name, _m, _p, _h in rows:
        c = per.get(name, collections.Counter())
        ex = next((("%08x -> %s" % (w, t)) for w, v, t in detail[name]
                   if v == "SILENT"), "-")
        print("    %-14s %d/%d/%d  %s" % (name, c.get("SILENT", 0),
                                          c.get("REFUSED", 0),
                                          c.get("AGREES", 0), ex))
    if n_words == 0:
        print("\nFAIL: sampled 0 encodings.")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
