#!/usr/bin/env python3
"""Dump c2.dll's final-expansion switch: its arms, and how many words each emits.

Read R6 (`docs/whitebox/READ_PLAN_2026-08-21.md` §3; spec page
`docs/whitebox/ref/P_EXPAND.md`).  Whitebox tooling, outside the std-only
`crates/` workspace per CLAUDE.md.

    0x10c0d57e   FUN_10c0d57e, the final-expansion switch (3899 B)
    0x10c182b4   FUN_10c182b4, the peephole pass (426 B)
    0x10c18460   the peephole's arm jump table   (stride 4)
    0x10c184a8   the peephole's byte index       (stride 1, opcode-1)

Two things are computed, and they answer the read's deliverable
("which opcodes expand to how many words") from opposite ends:

  --arms      the dispatch tree of FUN_10c0d57e, recovered from the
              compare/subtract chain, giving the OPCODE -> ARM VA map;
  --words     for each arm VA, the number of INSTRUCTION-CONSTRUCTOR calls
              reachable on any path from that arm to the function's exit,
              as a (min, max) pair over paths.  Each constructor call
              allocates one list node, sets `node[1] = opcode` and sets bit 0
              of `node+9` -- R2's "real instruction" bit -- so one call is
              one candidate emitted word.
  --peephole  the peephole pass's 18-arm table, decoded from its own two
              tables, including the arm-6 row absent from every prior doc.

The 16 instruction constructors are the functions that call the list-insert
wrapper 0x10bd5732; they are listed in CONSTRUCTORS below and were obtained
by inverting the export's call graph, then confirmed by reading each body.

Disassembly comes from `objdump -d -M intel` run on the pinned image itself,
NOT from the Ghidra flat export -- so this script depends on no artifact
older than the image.  binutils is the only non-stdlib dependency.

The image this record is written against is
sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258
(C2_MAP_METHOD.md §0); the script verifies the digest and refuses otherwise.

Usage:
    python3 docs/whitebox/scripts/dump_expansion.py <c2.dll> --arms
    python3 docs/whitebox/scripts/dump_expansion.py <c2.dll> --words
    python3 docs/whitebox/scripts/dump_expansion.py <c2.dll> --peephole
"""

import os
import re
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from dump_opcode_tables import Image, PINNED_SHA256, MNEMONIC_TABLE_VA, TABLE_STRIDE

EXPAND_LO = 0x10C0D57E
EXPAND_HI = 0x10C0E4B9          # == FUN_10c0e4b9, the next function

PEEP_LO = 0x10C182B4
PEEP_HI = PEEP_LO + 426
PEEP_JUMP_TABLE = 0x10C18460    # stride 4, one target per arm
PEEP_BYTE_INDEX = 0x10C184A8    # stride 1, indexed by (opcode - 1)
PEEP_INDEX_LEN = 0x293          # the bound tested: (u32)(op-1) > 0x292 -> done

# The instruction constructors: every function that calls the list-insert
# wrapper 0x10bd5732.  One call == one list node with the real-instruction bit.
CONSTRUCTORS = {
    0x10BD59AA, 0x10BD722E, 0x10BD726D, 0x10BD72B0, 0x10BD72FB,
    0x10BD7354, 0x10BD73AC, 0x10BD7413, 0x10BD748B, 0x10BD74F8,
    0x10BD75FF, 0x10BD7652, 0x10BD76E6, 0x10BD7780, 0x10BD77DB,
    0x10BD7814,
}

# Helpers that are themselves multi-word emitters; a call to one of these is
# NOT one word, and the word count is delegated.  Read R6 §3.
DELEGATES = {
    0x10C216F5: "prologue thunk (param_4 = 0)",
    0x10C21719: "prologue thunk (param_4 = second entry)",
    0x10BFFB72: "-> FUN_10bffaa3, the restore driver",
    0x10C0A2E2: "the rlandi expander",
    0x10BFF95C: "the prologue driver",
    0x10C0B6FA: "the frame allocator",
    0x10C07910: "the register-save emitter",
    0x10C07ACE: "the register-restore emitter",
}

BRANCH = {
    "jmp", "je", "jne", "jz", "jnz", "ja", "jae", "jb", "jbe", "jg", "jge",
    "jl", "jle", "js", "jns", "jo", "jno", "jp", "jnp",
}
UNCOND = {"jmp", "ret"}

# `mov eax,DWORD PTR [esi+0x4]` at 0x10c0d58f loads param_2[1], the opcode.
OPCODE_REG = "eax"

OPCODE_LOAD = re.compile(r"DWORD PTR \[e[a-z]{2}\+0x4\]")

LINE = re.compile(r"^\s*([0-9a-f]+):\s+((?:[0-9a-f]{2} )+)\s*\t(\S+)\s*(.*)$")


def disasm(path, lo, hi):
    """[(va, mnemonic, operand_text)] straight from the pinned image."""
    out = subprocess.run(
        ["objdump", "-d", "-M", "intel",
         "--start-address=%#x" % lo, "--stop-address=%#x" % hi, path],
        capture_output=True, text=True, check=True).stdout
    insns = []
    for line in out.splitlines():
        m = LINE.match(line)
        if m:
            insns.append((int(m.group(1), 16), m.group(3), m.group(4).strip()))
    return insns


def target_of(op_text):
    m = re.match(r"^(0x[0-9a-f]+)", op_text)
    return int(m.group(1), 16) if m else None


def build_cfg(insns):
    """-> (succ, calls) keyed by VA.  `calls` is the call target or None."""
    by_va = {va: i for i, (va, _, _) in enumerate(insns)}
    succ, calls = {}, {}
    for i, (va, mn, ops) in enumerate(insns):
        nxt = insns[i + 1][0] if i + 1 < len(insns) else None
        s = []
        if mn == "call":
            calls[va] = target_of(ops)
            if nxt is not None:
                s.append(nxt)
        elif mn in BRANCH:
            t = target_of(ops)
            if t is not None and t in by_va:
                s.append(t)
            if mn not in UNCOND and nxt is not None:
                s.append(nxt)
        elif mn.startswith("ret"):
            pass
        else:
            if nxt is not None:
                s.append(nxt)
        succ[va] = s
    return succ, calls


def count_words(entry, succ, calls):
    """(min, max, delegates) instruction-constructor calls from `entry` to exit.

    max is capped: a back edge (loop) makes the count unbounded, reported as
    None, which is itself a finding -- a loop in an arm means the word count is
    data-dependent and no constant can describe it.
    """
    best_min, best_max, dele, unbounded = {}, {}, set(), [False]

    def walk(va, stack):
        if va is None:
            return 0, 0
        if va in stack:
            unbounded[0] = True
            return 0, 0
        if va in best_min:
            return best_min[va], best_max[va]
        w = 1 if calls.get(va) in CONSTRUCTORS else 0
        if calls.get(va) in DELEGATES:
            dele.add(calls[va])
        s = succ.get(va, [])
        if not s:
            return w, w
        stack = stack | {va}
        lo = hi = None
        for t in s:
            a, b = walk(t, stack)
            lo = a if lo is None else min(lo, a)
            hi = b if hi is None else max(hi, b)
        lo, hi = w + lo, w + hi
        if not (stack - {va}) & set(best_min):
            best_min[va], best_max[va] = lo, hi
        return lo, hi

    lo, hi = walk(entry, frozenset())
    return lo, (None if unbounded[0] else hi), sorted(dele)


def opcode_tree(insns):
    """Recover the dispatch tree of FUN_10c0d57e -> {opcode: arm VA}.

    FUN_10c0d57e is a BINARY DECISION TREE, not a jump table (this reproduces
    WB_SELECT_FINDINGS.md:668's PARTIAL), so the arm set cannot be read out of
    a table -- it has to be recovered from the comparison chain.  MSVC emits
    two idioms against the register holding `param_2[1]`:

        cmp eax,IMM / je ARM / ja HIGH     -- a binary-search pivot
        mov ecx,eax / sub ecx,IMM / je ARM / sub ecx,K / je ARM ...
                                           -- a dense run, flags from the sub

    A linear scan cannot do this: the opcode register is clobbered inside arm
    bodies, and the sub-runs are reached only by branches.  So this is a
    forward dataflow over the CFG, forking at every conditional branch,
    carrying `alias[reg] = k` ("reg holds opcode - k") and `const[reg] = c`.
    A path dies when the opcode register is clobbered or a call is reached
    (that is an arm body, not dispatch), which bounds the walk.
    """
    by_va = {va: i for i, (va, _, _) in enumerate(insns)}
    arms = {}
    seen = set()
    # state: (alias tuple, const tuple, last-modified alias reg, pending cmp)
    start = insns[0][0]
    work = [(start, (), (), None, None)]
    while work:
        va, al, co, last, pend = work.pop()
        key = (va, al, co, last, pend)
        if key in seen or va not in by_va:
            continue
        seen.add(key)
        if len(seen) > 400000:
            break
        alias, const = dict(al), dict(co)
        _, mn, ops = insns[by_va[va]]
        f = [x.strip() for x in ops.split(",")] if ops else []
        nxt = insns[by_va[va] + 1][0] if by_va[va] + 1 < len(insns) else None
        newpend = None

        if mn == "call":
            continue                      # arm body: dispatch is over
        if mn.startswith("ret"):
            continue
        if mn == "mov" and len(f) == 2:
            d, sr = f
            const.pop(d, None); alias.pop(d, None)
            if re.fullmatch(r"0x[0-9a-f]+", sr):
                const[d] = int(sr, 16)
            elif sr in alias:
                alias[d] = alias[sr]
            elif OPCODE_LOAD.fullmatch(sr):
                alias[d] = 0
        elif mn in ("sub", "add") and len(f) == 2 and f[0] in alias \
                and re.fullmatch(r"0x[0-9a-f]+", f[1]):
            k = int(f[1], 16)
            alias[f[0]] += k if mn == "sub" else -k
            last = f[0]
        elif mn in ("dec", "inc") and f and f[0] in alias:
            alias[f[0]] += 1 if mn == "dec" else -1
            last = f[0]
        elif mn == "cmp" and len(f) == 2:
            a, b = f
            if a in alias and re.fullmatch(r"0x[0-9a-f]+", b):
                newpend = (alias[a], int(b, 16))
            elif a in alias and b in const:
                newpend = (alias[a], const[b])
            elif b in alias and a in const:
                newpend = (alias[b], const[a])
        elif mn in ("test",):
            pass
        elif f and mn not in BRANCH:
            const.pop(f[0], None); alias.pop(f[0], None)
            if f[0] == OPCODE_REG:
                continue                  # the opcode is gone; stop this path

        t = target_of(ops) if mn in BRANCH else None
        if mn == "je" and t:
            k = (pend[0] + pend[1]) if pend is not None else alias.get(last)
            if k is not None and 0 < k <= 0x400:
                arms.setdefault(k, t)
        elif mn in ("jb", "jbe") and t and pend is not None:
            # `sub eax,0x26e / cmp eax,2 / jb ARM` is a RANGE arm: every opcode
            # in [base, base+imm) shares one body.  Equality-only recovery
            # misses these entirely -- rlandi/rlandi. is exactly this shape.
            base, imm = pend
            hi = base + imm - (1 if mn == "jb" else 0)
            if 0 < base and hi - base < 64 and hi <= 0x400:
                for k in range(base, hi + 1):
                    arms.setdefault(k, t)
        if mn in BRANCH:
            # A branch does not write the flags, so a pending compare survives
            # it -- MSVC emits `cmp / ja HIGH / je ARM`, and dropping `pend`
            # across the `ja` loses every binary-search pivot in the tree.
            t = target_of(ops)
            if t is not None and t in by_va:
                work.append((t, tuple(sorted(alias.items())),
                             tuple(sorted(const.items())), last, pend))
            if mn not in UNCOND and nxt is not None:
                work.append((nxt, tuple(sorted(alias.items())),
                             tuple(sorted(const.items())), last, pend))
        elif nxt is not None:
            work.append((nxt, tuple(sorted(alias.items())),
                         tuple(sorted(const.items())), last, newpend))
    return arms


def mnemonic(img, op):
    if op is None or op > 0x295:
        return None
    p = img.u32(MNEMONIC_TABLE_VA + op * TABLE_STRIDE)
    return img.cstr(p) if p else None


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    path, mode = argv[0], argv[1]
    img = Image(path)
    if img.digest != PINNED_SHA256:
        sys.stderr.write("REFUSE: sha256 %s is not the pinned image %s\n"
                         % (img.digest, PINNED_SHA256))
        return 1
    print("# %s sha256 %s… (matches the pinned digest)" % (path, img.digest[:12]))

    if mode == "--peephole":
        print("# FUN_10c182b4 peephole: byte index %#x, jump table %#x"
              % (PEEP_BYTE_INDEX, PEEP_JUMP_TABLE))
        idx_off = img.off(PEEP_BYTE_INDEX)
        arms = {}
        for op1 in range(PEEP_INDEX_LEN + 1):
            arm = img.blob[idx_off + op1]
            arms.setdefault(arm, []).append(op1 + 1)
        print("# arm  target      nopcodes  example opcodes")
        for arm in sorted(arms):
            tgt = img.u32(PEEP_JUMP_TABLE + arm * 4)
            ops = arms[arm]
            names = [n for n in (mnemonic(img, o) for o in ops[:6]) if n]
            print("%5d  %#010x  %8d  %s" % (arm, tgt, len(ops), ",".join(names)))
        return 0

    insns = disasm(path, EXPAND_LO, EXPAND_HI)
    print("# FUN_10c0d57e  %#x..%#x  %d instructions"
          % (EXPAND_LO, EXPAND_HI, len(insns)))
    arms = opcode_tree(insns)

    if mode == "--arms":
        print("# opcode  mnemonic      arm VA")
        for op in sorted(arms):
            print("%#08x  %-12s  %#010x" % (op, mnemonic(img, op) or "-", arms[op]))
        print("# %d distinct opcode values receive a non-default arm" % len(arms))
        return 0

    if mode == "--words":
        succ, calls = build_cfg(insns)
        print("# opcode  mnemonic      arm VA      words(min..max)  delegates")
        for op in sorted(arms):
            lo, hi, dele = count_words(arms[op], succ, calls)
            hs = "unbounded" if hi is None else str(hi)
            print("%#08x  %-12s  %#010x  %3s..%-9s  %s"
                  % (op, mnemonic(img, op) or "-", arms[op], lo, hs,
                     ",".join("%#x" % d for d in dele) or "-"))
        return 0

    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
