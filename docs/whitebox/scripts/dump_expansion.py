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
# The bound tested is `(u32)(op - 1) > 0x292 -> done`, so the index holds
# 0x293 entries covering opcodes 0x001..0x293.  vmr128 (0x294) is OUTSIDE it
# and reaches the pass's own exit, not an arm -- reading one entry too many
# invents a bogus arm 51 whose jump-table word is 0x11111111.
PEEP_INDEX_LEN = 0x293

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


# The per-opcode attribute byte table the dispatch tail consults at
# 0x10c0e30b: `mov cl,BYTE PTR [eax+0x10c3afd8] / and cl,0x7 / cmp cl,2`.
# The low 3 bits are an opcode CLASS; class 2 routes to 0x10c0e40f.
CLASS_TABLE = 0x10C3AFD8
CLASS_MASK = 0x7

# 0x10c0e30b is not an arm: it is the dispatch TAIL, which re-dispatches on
# CLASS_TABLE rather than on the opcode.  Paths reaching it are reported
# separately, never merged into the opcode->arm map.
DISPATCH_TAIL = 0x10C0E30B


def opcode_bound(insns):
    """The largest opcode constant the tree actually tests, + 1.

    Used as the walk's upper bound so the map is bounded by what the CODE
    discriminates rather than by a number chosen to make the output look
    tidy.  Everything above it reaches the dispatch tail by construction.
    """
    best = 0
    for _, mn, ops in insns:
        if mn in ("cmp", "sub", "add", "lea"):
            for m in re.finditer(r"0x([0-9a-f]+)", ops):
                v = int(m.group(1), 16)
                if v <= 0x1000:
                    best = max(best, v)
    return best + 1


def opcode_tree(insns, OPMAX):
    """Recover FUN_10c0d57e's dispatch -> {arm VA: sorted list of opcodes}.

    FUN_10c0d57e is a BINARY SEARCH TREE on `param_2[1]`, not a jump table
    (this reproduces WB_SELECT_FINDINGS.md:668's PARTIAL from the bytes), so
    there is no table to read and no fixed idiom to match.  The only correct
    recovery is to propagate the OPCODE INTERVAL along every path: each
    compare-and-branch narrows [lo, hi], and when a path reaches an arm body
    the surviving interval IS that arm's opcode set.

    This is what catches RANGE arms, which an equality-only scan misses by
    construction -- `cmp eax,0xb / jb default / cmp eax,0xd / jbe ARM` is the
    addi/addic/addic. arm, and `lea ecx,[eax-0x26e] / cmp ecx,1 / ja default`
    is the rlandi pair.  Neither contains a single `je` on the opcode.

    State per path: the interval, a set of excluded values (from `je`/`jne`
    fall-throughs), and alias[reg] = k meaning "reg holds opcode - k".
    """
    by_va = {va: i for i, (va, _, _) in enumerate(insns)}
    entry = insns[0][0]
    arms, wide, seen = {}, {}, set()

    # A body reached with a WIDE surviving interval was not discriminated for
    # those opcodes -- it is a shared fall-through, not a per-opcode arm.
    # Crediting it would inflate the arm map by hundreds of opcodes the tree
    # never tests.  Wide arrivals are reported separately, never merged.
    NARROW = 64

    def emit(arm, lo, hi, exc):
        if arm is None or lo > hi:
            return
        ops = [o for o in range(max(lo, 1), min(hi, OPMAX) + 1) if o not in exc]
        if not ops:
            return
        if len(ops) <= NARROW:
            arms.setdefault(arm, set()).update(ops)
        else:
            wide.setdefault(arm, set()).update(ops)

    # (va, lo, hi, excluded, alias, pending-compare-value, arm-entry)
    work = [(entry, 1, OPMAX, frozenset(), (("eax", 0),), None, None)]
    tail = set()
    while work:
        va, lo, hi, exc, al, pend, arm = work.pop()
        if va not in by_va or lo > hi:
            continue
        key = (va, lo, hi, exc, al, pend, arm)
        if key in seen or len(seen) > 300000:
            continue
        seen.add(key)
        alias = dict(al)
        _, mn, ops = insns[by_va[va]]
        f = [x.strip() for x in ops.split(",")] if ops else []
        nxt = insns[by_va[va] + 1][0] if by_va[va] + 1 < len(insns) else None
        t = target_of(ops) if mn in BRANCH else None

        if va == DISPATCH_TAIL:
            tail.update(o for o in range(max(lo, 1), min(hi, OPMAX) + 1)
                        if o not in exc)
            continue
        if mn == "call" or mn.startswith("ret"):
            emit(arm, lo, hi, exc)          # an arm body: dispatch is over
            continue

        newpend = pend
        if mn == "mov" and len(f) == 2:
            d, sr = f
            alias.pop(d, None)
            if sr in alias:
                alias[d] = alias[sr]
            elif OPCODE_LOAD.fullmatch(sr):
                alias[d] = 0
        elif mn == "lea" and len(f) == 2:
            m = re.fullmatch(r"\[(e[a-z]{2})([-+])0x([0-9a-f]+)\]", f[1])
            alias.pop(f[0], None)
            if m and m.group(1) in alias:
                k = int(m.group(3), 16)
                alias[f[0]] = alias[m.group(1)] + (k if m.group(2) == "-" else -k)
        elif mn in ("sub", "add") and len(f) == 2 and f[0] in alias \
                and re.fullmatch(r"0x[0-9a-f]+", f[1]):
            k = int(f[1], 16)
            alias[f[0]] += k if mn == "sub" else -k
            newpend = alias[f[0]]           # sub sets the flags against 0
        elif mn in ("dec", "inc") and f and f[0] in alias:
            alias[f[0]] += 1 if mn == "dec" else -1
            newpend = alias[f[0]]
        elif mn == "cmp" and len(f) == 2:
            x, y = f
            if x in alias and re.fullmatch(r"0x[0-9a-f]+", y):
                newpend = alias[x] + int(y, 16)
            else:
                newpend = None
        elif mn == "test":
            newpend = None
        elif f and mn not in BRANCH:
            alias.pop(f[0], None)
            if f[0] == "eax" and "eax" not in alias:
                emit(arm, lo, hi, exc)      # the opcode is dead: arm body
                continue

        def push(dst, l, h, e, a2):
            if dst is not None and l <= h:
                work.append((dst, l, h, e, tuple(sorted(a2.items())),
                             newpend if dst == nxt else None,
                             arm if dst == nxt else dst))

        if mn in BRANCH and t is not None:
            v = pend
            if mn == "jmp":
                work.append((t, lo, hi, exc, tuple(sorted(alias.items())),
                             pend, t))
            elif v is None:
                push(t, lo, hi, exc, alias); push(nxt, lo, hi, exc, alias)
            elif mn in ("je", "jz"):
                push(t, v, v, frozenset(), alias)
                push(nxt, lo, hi, exc | {v}, alias)
            elif mn in ("jne", "jnz"):
                push(t, lo, hi, exc | {v}, alias)
                push(nxt, v, v, frozenset(), alias)
            elif mn == "ja":
                push(t, max(lo, v + 1), hi, exc, alias)
                push(nxt, lo, min(hi, v), exc, alias)
            elif mn in ("jae", "jnb"):
                push(t, max(lo, v), hi, exc, alias)
                push(nxt, lo, min(hi, v - 1), exc, alias)
            elif mn == "jb":
                push(t, lo, min(hi, v - 1), exc, alias)
                push(nxt, max(lo, v), hi, exc, alias)
            elif mn == "jbe":
                push(t, lo, min(hi, v), exc, alias)
                push(nxt, max(lo, v + 1), hi, exc, alias)
            else:
                push(t, lo, hi, exc, alias); push(nxt, lo, hi, exc, alias)
        elif nxt is not None:
            work.append((nxt, lo, hi, exc, tuple(sorted(alias.items())),
                         newpend, arm))
    return arms, wide, tail


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
        for op1 in range(PEEP_INDEX_LEN):
            arm = img.blob[idx_off + op1]
            arms.setdefault(arm, []).append(op1 + 1)
        print("# arm  target      nopcodes  example opcodes")
        for arm in sorted(arms):
            tgt = img.u32(PEEP_JUMP_TABLE + arm * 4)
            ops = arms[arm]
            names = [n for n in (mnemonic(img, o) for o in ops[:6]) if n]
            body = disasm(path, tgt, tgt + 24)
            tail = [target_of(o) for _, m, o in body if m in ("call", "jmp")]
            mk = [t for t in tail if t in CONSTRUCTORS]
            print("%5d  %#010x  %8d  %-8s %s"
                  % (arm, tgt, len(ops),
                     "MINTS" if mk else "no-mint", ",".join(names)))
        print("# %d opcodes over %d arms; 'MINTS' means the arm thunk reaches an"
              " instruction constructor directly" % (sum(len(v) for v in arms.values()), len(arms)))
        return 0

    insns = disasm(path, EXPAND_LO, EXPAND_HI)
    print("# FUN_10c0d57e  %#x..%#x  %d instructions"
          % (EXPAND_LO, EXPAND_HI, len(insns)))
    OPMAX = opcode_bound(insns)
    arms, wide, tail = opcode_tree(insns, OPMAX)
    print("# opcode bound discriminated by the tree: %#x" % OPMAX)

    if mode == "--arms":
        byop = {o: a for a, os in arms.items() for o in os}
        print("# opcode  mnemonic      arm VA")
        for op in sorted(byop):
            print("%#08x  %-12s  %#010x" % (op, mnemonic(img, op) or "-", byop[op]))
        print("# %d distinct opcode values receive a non-default arm, "
              "over %d distinct arm bodies" % (len(byop), len(arms)))
        print("# %d opcodes reach the dispatch TAIL %#x (class table %#x)"
              % (len(tail), DISPATCH_TAIL, CLASS_TABLE))
        print("# %d shared fall-through bodies reached un-narrowed: %s"
              % (len(wide), ",".join("%#x(%d)" % (a, len(o))
                                     for a, o in sorted(wide.items()))))
        return 0

    if mode == "--words":
        succ, calls = build_cfg(insns)
        print("# arm VA      words(min..max)  nopcodes  opcodes / delegates")
        for arm in sorted(arms):
            lo, hi, dele = count_words(arm, succ, calls)
            hs = "unbounded" if hi is None else str(hi)
            os_ = sorted(arms[arm])
            names = [mnemonic(img, o) or ("%#x" % o) for o in os_[:8]]
            print("%#010x  %3s..%-9s  %8d  %s%s"
                  % (arm, lo, hs, len(os_), ",".join(names),
                     "  DELEGATES:" + ",".join("%#x" % d for d in dele) if dele else ""))
        return 0

    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
