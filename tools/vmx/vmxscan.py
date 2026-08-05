#!/usr/bin/env python3
"""vmxscan.py -- how much VMX128 is actually in the workload?

Pure stdlib (reuses `tools/coffdump.py`'s COFF reader). Tooling, outside the
std-only Rust workspace. Never a gate.

This is the number that decides whether VMX128 codegen is worth building at
all. It is a *census*, and this repo's memory is explicit that a census is a
driver and not a target -- so the output here is deliberately shaped as a
denominator-carrying report, never a bare "N TUs use vectors".

SAFETY: this walks an explicit directory of objects, one level deep, and NEVER
globs `work/capture-cache` or `.claude/worktrees`. Two kernel OOM kills on this
box came from a bare recursive walk from the repo root. Build the object tree
with `tools/vmx/build_objs.sh`, which puts one obj per TU in its own directory.

Usage:
    tools/vmx/build_objs.sh                       # ~30 s, 878 TUs, ~102 MB
    tools/vmx/vmxscan.py work/w-vmx/objs
    tools/vmx/vmxscan.py --tsv out.tsv work/w-vmx/objs
"""
import argparse
import collections
import os
import struct
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, _HERE)
sys.path.insert(0, os.path.dirname(_HERE))      # tools/

import coffdump as C                 # noqa: E402
import vmx128                        # noqa: E402
import vmx128_isa as ISA             # noqa: E402

# AltiVec / VMX load-store under primary opcode 31 (X-form, bits 21..30).
# Present here so "0 of these" is a stated measurement and not an omission.
AV31 = {6: "lvsl", 7: "lvebx", 38: "lvsr", 39: "lvehx", 71: "lvewx",
        103: "lvx", 135: "stvebx", 167: "stvehx", 199: "stvewx", 231: "stvx",
        359: "lvxl", 487: "stvxl", 342: "dst", 374: "dstst", 822: "dss"}


def iter_objs(root):
    """One level of subdirectories, each holding one `o.obj`. Explicitly NOT a
    recursive walk."""
    if os.path.isfile(root):
        yield os.path.basename(root), root
        return
    for key in sorted(os.listdir(root)):
        p = os.path.join(root, key, "o.obj")
        if os.path.isfile(p):
            yield key, p


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("root", help="directory built by tools/vmx/build_objs.sh")
    ap.add_argument("--tsv")
    a = ap.parse_args()

    if not os.path.exists(a.root):
        print("SKIP: %s absent -- build it with tools/vmx/build_objs.sh"
              % a.root, file=sys.stderr)
        return 2

    vmx_names = {vmx128.MS_MNEMONIC.get(n, n) for n, _m, _p, _x in ISA.VMX128}

    n_tu = 0
    n_words = 0
    n_funcs = 0
    mnem = collections.Counter()
    tus_with = collections.Counter()          # mnemonic -> TUs containing it
    tu_vmx = []                               # (tu, vmx_word_count)
    funcs_with = []                           # (tu, func, count)
    av31 = collections.Counter()
    unrecognized = 0

    for key, path in iter_objs(a.root):
        n_tu += 1
        data = open(path, "rb").read()
        secs, syms = C.read_coff(data)
        if secs is None:
            continue
        C.infer_sizes(secs, syms)
        bysec = collections.defaultdict(list)
        for sy in syms:
            if 0 < sy.sec <= len(secs) and sy.size and \
                    secs[sy.sec - 1].name.startswith(".text"):
                bysec[sy.sec - 1].append(sy)
        tu_count = 0
        seen = set()
        for si, s in enumerate(secs):
            if not s.name.startswith(".text"):
                continue
            names = sorted(bysec.get(si, []), key=lambda x: x.value)
            n_funcs += len(names)
            per_func = collections.Counter()
            d = s.data
            for off in range(0, len(d) - 3, 4):
                w = struct.unpack_from(">I", d, off)[0]
                n_words += 1
                prim = w >> 26
                if prim == 31:
                    xo = (w >> 1) & 0x3FF
                    if xo in AV31:
                        av31[AV31[xo]] += 1
                    continue
                if prim not in (4, 5, 6):
                    continue
                dec = vmx128.decode(w)
                if dec is None:
                    unrecognized += 1
                    continue
                mnem[dec.mnemonic] += 1
                seen.add(dec.mnemonic)
                if dec.table == "VMX128":
                    tu_count += 1
                    owner = "?"
                    for sy in names:
                        if sy.value <= off < sy.value + sy.size:
                            owner = sy.name
                    per_func[owner] += 1
            for f, c in per_func.items():
                funcs_with.append((key, f, c))
        for m in seen:
            tus_with[m] += 1
        if tu_count:
            tu_vmx.append((key, tu_count))

    vmx_words = sum(c for m, c in mnem.items() if m in vmx_names)
    av_words = sum(c for m, c in mnem.items() if m not in vmx_names)

    W = 40
    print("VMX128 prevalence in the dc3 workload")
    print()
    print("  %-*s %d" % (W, "TUs scanned", n_tu))
    print("  %-*s %d" % (W, ".text instruction words", n_words))
    print("  %-*s %d" % (W, "COMDAT text symbols (functions)", n_funcs))
    print("  " + "-" * (W + 14))
    print("  %-*s %d   (%.3f%% of TUs)" % (
        W, "TUs containing >=1 VMX128 instruction", len(tu_vmx),
        100.0 * len(tu_vmx) / max(n_tu, 1)))
    print("  %-*s %d" % (W, "functions containing >=1 VMX128",
                         len({(t, f) for t, f, _c in funcs_with})))
    print("  %-*s %d   (%.4f%% of words)" % (
        W, "VMX128 instruction words", vmx_words,
        100.0 * vmx_words / max(n_words, 1)))
    print("  %-*s %d   (%.4f%% of words)" % (
        W, "plain AltiVec words (opcode 4)", av_words,
        100.0 * av_words / max(n_words, 1)))
    print("  %-*s %d" % (W, "AltiVec load/store under opcode 31",
                         sum(av31.values())))
    print("  %-*s %d" % (W, "opcode-4/5/6 words we could NOT decode",
                         unrecognized))
    print()
    print("  TUs, by VMX128 instruction count:")
    for t, c in sorted(tu_vmx, key=lambda x: -x[1]):
        print("    %-52s %d" % (t, c))
    print()
    print("  functions, by VMX128 instruction count:")
    for t, f, c in sorted(funcs_with, key=lambda x: -x[2]):
        print("    %-5d %s" % (c, f[:100]))
    print()
    print("  opcode histogram (all opcode-4/5/6 words):")
    for m, c in mnem.most_common():
        print("    %-14s %-13s %-6d  in %d TU(s)" % (
            m, "VMX128" if m in vmx_names else "AltiVec", c, tus_with[m]))
    if av31:
        print()
        print("  opcode-31 AltiVec load/store: %s" % dict(av31))

    if a.tsv:
        with open(a.tsv, "w") as fh:
            fh.write("tu\tfunction\tvmx128_words\n")
            for t, f, c in sorted(funcs_with, key=lambda x: -x[2]):
                fh.write("%s\t%s\t%d\n" % (t, f, c))
        print("\n  wrote %s" % a.tsv)

    if n_tu == 0:
        print("\nFAIL: scanned 0 objects. A scan that read nothing is not a "
              "measurement.")
        return 1
    if unrecognized:
        print("\nFAIL: %d opcode-4/5/6 words matched no table row. Every one "
              "of those is a hole in the decoder, not a zero." % unrecognized)
        return 1
    print("\nOK: %d TUs scanned, %d VMX128 words decoded, 0 undecoded "
          "opcode-4/5/6 words." % (n_tu, vmx_words))
    return 0


if __name__ == "__main__":
    sys.exit(main())
