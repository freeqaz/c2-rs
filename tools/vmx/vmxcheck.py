#!/usr/bin/env python3
"""vmxcheck.py -- THE DELIVERABLE. Verify a VMX128 decode is CORRECT, not
merely PRODUCED.

Pure stdlib. Tooling, outside the std-only Rust workspace. **Not a gate**: the
sole judge of the port remains the real `c2.dll` under wibo plus a byte-exact
obj compare (CLAUDE.md). Nothing here is or becomes the correctness judge.

THE PROBLEM THIS TOOL EXISTS FOR
--------------------------------
`llvm-mc -triple=powerpc` decodes VMX128 into plausible legal modern PowerPC
with NO diagnostic. `10 23 20 c3` is `lvx128 vr1,r3,r4`; LLVM prints
`vucmprlb 1, 3, 4` and exits 0. A tool that only asks "did the disassembler
complain?" reports success. This repo has 16 recorded instances of absence read
as success, and the generalizing fix on record is a POSITIVE CHECK WITH A
PRINTED COUNT. So:

  * every decode is graded against Microsoft's own `/FAcs` listing for the same
    word -- an oracle that cannot be wrong about what `c2` emitted, because it
    is `c2` narrating what it emitted;
  * a decode that matches no table row is UNRECOGNIZED and is a failure, never
    a skip;
  * a decode whose mnemonic differs from the listing's is a MISMATCH and is a
    failure;
  * the number of instructions VERIFIED is printed, and **a run that verifies
    zero exits non-zero.** A run that graded nothing must not look like a run
    that agreed.

`--selftest` proves the instrument can still fail: it perturbs the decode on
OUR side only and requires exactly one mismatch per in-scope instruction.
(Corrupting the input is not a control -- both sides would read the same
corrupted input and agree. `tools/llvm/xcheck.py` records that this was tried
there first and correctly reported 0 diffs.)

USAGE
    tools/vmx/vmxcheck.py work/w-vmx/lst/*/*.cod
    tools/vmx/vmxcheck.py --llvm  ...      # also measure the collisions
    tools/vmx/vmxcheck.py --selftest ...   # show the check can fail
    tools/vmx/vmxcheck.py --tsv out.tsv ...

Produce the listings with `work/w-vmx/listing.sh <src> <outdir>`.
"""
import argparse
import collections
import os
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, _HERE)

import codparse                      # noqa: E402
import llvmmc                        # noqa: E402
import vmx128                        # noqa: E402
import vmx128_isa as ISA             # noqa: E402

IN_SCOPE_PRIMARY = (4, 5, 6)

V_VERIFIED = "VERIFIED"
V_MNEMONIC = "MNEMONIC-ONLY"
V_MISMATCH = "MISMATCH"
V_UNRECOG = "UNRECOGNIZED"


def norm_ops(ops):
    return ",".join(o.strip() for o in ops)


def grade(insn, perturb=False):
    d = vmx128.decode(insn.word)
    if d is None:
        return V_UNRECOG, None
    mnem = d.mnemonic
    ops = norm_ops(d.operands)
    if perturb:
        # Perturb OUR side only: one deterministic character change in the
        # mnemonic. Must show up as exactly one MISMATCH per in-scope insn.
        mnem = mnem + "~"
    if mnem != insn.mnemonic:
        return V_MISMATCH, d
    if ops != norm_ops(insn.operands):
        return V_MNEMONIC, d
    return V_VERIFIED, d


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("cod", nargs="*", help="MSVC /FAcs .cod listings")
    ap.add_argument("--llvm", action="store_true",
                    help="also ask llvm-mc about every in-scope word and "
                         "classify the collision")
    ap.add_argument("--selftest", action="store_true",
                    help="perturb our own decode; require one MISMATCH each")
    ap.add_argument("--tsv", help="write a per-instruction TSV here")
    ap.add_argument("--quiet", action="store_true")
    a = ap.parse_args()

    if not a.cod:
        print("SKIP: no .cod listings given -- produce them with "
              "work/w-vmx/listing.sh", file=sys.stderr)
        return 2

    missing = [p for p in a.cod if not os.path.exists(p)]
    if missing:
        print("SKIP: listing(s) absent: %s" % ", ".join(missing[:3]),
              file=sys.stderr)
        return 2

    insns = list(codparse.parse_many(a.cod))
    scope = [i for i in insns if (i.word >> 26) in IN_SCOPE_PRIMARY]

    verdicts = collections.Counter()
    per_mnem = collections.defaultdict(collections.Counter)
    rows = []
    failures = []
    for insn in scope:
        v, d = grade(insn, perturb=a.selftest)
        verdicts[v] += 1
        per_mnem[insn.mnemonic][v] += 1
        rows.append((insn, v, d))
        if v in (V_MISMATCH, V_UNRECOG):
            failures.append((insn, v, d))

    # ---- collision measurement -------------------------------------------
    llvm_stat = collections.Counter()
    llvm_ans = {}
    mc_version = None
    if a.llvm and scope:
        mc = llvmmc.find_llvm_mc()
        if not mc:
            llvm_stat["SKIP: llvm-mc absent"] = 1
        else:
            mc_version = llvmmc.version(mc)
            words = [i.word for i in scope]
            res = llvmmc.disassemble(words, mc=mc)
            for insn, (ok, txt) in zip(scope, res):
                t = llvmmc.normalize(txt)
                llvm_ans[id(insn)] = (ok, t)
                if not ok:
                    llvm_stat["REFUSED (safe)"] += 1
                else:
                    lm = t.split(" ")[0].split("\t")[0]
                    if lm == insn.mnemonic:
                        llvm_stat["AGREES"] += 1
                    elif _is_alias_of(lm, insn.mnemonic):
                        llvm_stat["AGREES (alias)"] += 1
                    else:
                        llvm_stat["SILENTLY WRONG"] += 1

    # ---- report -----------------------------------------------------------
    W = 34
    print("VMX128 decode verification")
    print("  oracle: cl.exe's own /FAcs listing (the compiler narrating its "
          "own bytes)")
    if a.selftest:
        print("  MODE: --selftest -- our decode is deliberately perturbed; "
              "every in-scope")
        print("        instruction MUST come back MISMATCH or the instrument "
              "is blind")
    print()
    print("  %-*s %d" % (W, "listings read", len(a.cod)))
    print("  %-*s %d" % (W, "machine-code lines read", len(insns)))
    print("  %-*s %d" % (W, "in scope (primary opcode 4/5/6)", len(scope)))
    print("  " + "-" * (W + 12))
    print("  %-*s %d" % (W, "decoded (matched a table row)",
                         len(scope) - verdicts[V_UNRECOG]))
    print("  %-*s %d" % (W, "VERIFIED (mnemonic + operands)", verdicts[V_VERIFIED]))
    print("  %-*s %d" % (W, "mnemonic-only (operand print order)",
                         verdicts[V_MNEMONIC]))
    print("  %-*s %d" % (W, "MISMATCH (wrong mnemonic)", verdicts[V_MISMATCH]))
    print("  %-*s %d" % (W, "UNRECOGNIZED (no table row)", verdicts[V_UNRECOG]))
    print("  " + "-" * (W + 12))

    seen_mnem = sorted(per_mnem)
    vmx_names = {vmx128.MS_MNEMONIC.get(n, n) for n, _m, _p, _a in ISA.VMX128}
    seen_vmx = [m for m in seen_mnem if m in vmx_names]
    print("  %-*s %d of %d" % (W, "distinct VMX128 mnemonics seen",
                               len(seen_vmx), len(ISA.VMX128)))
    print("  %-*s %d" % (W, "distinct other opcode-4/5/6 mnemonics",
                         len(seen_mnem) - len(seen_vmx)))

    if a.llvm:
        print()
        print("  collision classification vs llvm-mc -triple=powerpc")
        if mc_version:
            print("  %-*s %s" % (W, "llvm-mc", mc_version))
        for k in ("SILENTLY WRONG", "AGREES", "AGREES (alias)",
                  "REFUSED (safe)", "SKIP: llvm-mc absent"):
            if llvm_stat.get(k):
                print("  %-*s %d" % (W, k, llvm_stat[k]))

    if not a.quiet:
        print()
        print("  per mnemonic (listing's spelling):")
        for m in seen_mnem:
            c = per_mnem[m]
            tag = "VMX128" if m in vmx_names else "base/AltiVec"
            extra = ""
            if a.llvm:
                sw = sum(1 for i, _v, _d in rows if i.mnemonic == m
                         and llvm_ans.get(id(i), (False, ""))[0]
                         and llvm_ans[id(i)][1].split(" ")[0] != m
                         and not _is_alias_of(llvm_ans[id(i)][1].split(" ")[0], m))
                extra = "  llvm-silently-wrong %d" % sw
            print("    %-14s %-13s n=%-4d %s%s" % (
                m, tag, sum(c.values()),
                " ".join("%s=%d" % (k, v) for k, v in sorted(c.items())), extra))

    if failures:
        print()
        print("  FAILURES (first 20):")
        for insn, v, d in failures[:20]:
            print("    %s:%d  %08x  listing=%-28s ours=%s"
                  % (os.path.basename(insn.source), insn.line, insn.word,
                     insn.text(), d.text() if d else "(no table row)"))

    if a.tsv:
        with open(a.tsv, "w") as fh:
            fh.write("file\tline\tfunc\toffset\tword\tlisting\tdecoded\t"
                     "verdict\tllvm_ok\tllvm\n")
            for insn, v, d in rows:
                ok, t = llvm_ans.get(id(insn), ("", ""))
                fh.write("%s\t%d\t%s\t%d\t%08x\t%s\t%s\t%s\t%s\t%s\n" % (
                    insn.source, insn.line, insn.func or "", insn.offset,
                    insn.word, insn.text(), d.text() if d else "", v, ok, t))
        print("\n  wrote %s" % a.tsv)

    print()
    if a.selftest:
        want = len(scope)
        got = verdicts[V_MISMATCH]
        ok = want > 0 and got == want and verdicts[V_VERIFIED] == 0
        print("SELFTEST %s: perturbed our decode of %d in-scope instructions; "
              "comparison reported %d MISMATCH and %d VERIFIED"
              % ("PASS" if ok else "FAIL", want, got, verdicts[V_VERIFIED]))
        return 0 if ok else 1

    if len(scope) == 0:
        print("FAIL: 0 instructions in scope. A run that verified nothing is "
              "NOT a pass.")
        return 1
    if verdicts[V_VERIFIED] == 0:
        print("FAIL: 0 instructions VERIFIED out of %d in scope." % len(scope))
        return 1
    if failures:
        print("FAIL: %d MISMATCH + %d UNRECOGNIZED out of %d in scope."
              % (verdicts[V_MISMATCH], verdicts[V_UNRECOG], len(scope)))
        return 1
    print("PASS: %d/%d in-scope instructions VERIFIED against the compiler's "
          "own listing (%d mnemonic-only)."
          % (verdicts[V_VERIFIED], len(scope), verdicts[V_MNEMONIC]))
    return 0


# LLVM prints simplified mnemonics for some AltiVec encodings. These are the
# ones observed here; each is a *documented* PowerPC extended mnemonic, so the
# decode agrees and only the spelling differs. Anything not in this table
# counts as SILENTLY WRONG -- the list is deliberately short and explicit so
# that widening it is a visible edit, not a silent one.
_ALIASES = {
    ("vmr", "vor"),          # vor vD,vA,vA  ==  vmr vD,vA   (PEM extended mnem)
    ("vnot", "vnor"),        # vnor vD,vA,vA ==  vnot vD,vA
}


def _is_alias_of(llvm_mnem, listing_mnem):
    return (llvm_mnem, listing_mnem) in _ALIASES


if __name__ == "__main__":
    sys.exit(main())
