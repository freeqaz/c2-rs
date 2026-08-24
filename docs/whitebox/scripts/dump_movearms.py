#!/usr/bin/env python3
"""Read the peephole's four class-1 (move) arms, and c2's pseudo-nop family.

Lane `w-r8idiom`; prereg `docs/rungs/_2026-08-24-w-r8idiom-prereg.md`; findings
`docs/whitebox/WB_R8IDIOM_FINDINGS.md`.  Whitebox tooling, outside the std-only
`crates/` workspace per CLAUDE.md.  Sibling of `dump_tailclass.py`, whose
`ref/P_OPATTR.md` §6 read arm 6 and named arm 14 as the unread violator.

    0x10c18373   arm 14 thunk -> FUN_10c16d83   `mr`    <- THIS LANE
    0x10c1837f   arm 16 thunk -> FUN_10c16e59   `vmr`
    0x10c1838b   arm  6 thunk -> FUN_10c16fbd   `fmr`   (read by w-tailread)
    0x10c18397   arm 15 thunk -> FUN_10c1707c   `mr.`
      -- four CONSECUTIVE 12-byte thunks, found by --arms, not assumed
    0x10c16cde   the shared UNLINK tail
    0x10b1b260   the mnemonic table (stride 12), and 0x10c37b7c its base words

What each mode computes, and why it is a computation and not an eyeball:

  --arms      For each handler: the two operand slots it loads, the field it
              compares, and then the MECHANICAL question this lane was sent to
              answer -- how many conditional branches lie between the
              same-register compare and the tail-call to the unlink, counting
              only those that can REACH the tail-call's block.  A handler that
              deletes unconditionally scores 0.  Reported as a number so
              "there is a guard" cannot be asserted by reading vibes off a
              listing.

  --nops      Every opcode in the mnemonic table whose base word is a nop form,
              with the word its encoder arm actually produces.  This is the
              mode that EXCLUDES a whole family: c2 owns nine pseudo-nops and
              not one of them encodes `or r8,r8,r8`.  The register operands of
              `nopstall` come from the byte table 0x10c37dcc, decoded here.

  --word W    Every occurrence of a 32-bit literal in the image, with the
              function that owns it -- and the warning that matters: BOTH the
              tables and the code live in `.text`, so a hit is a coincidence
              until its alignment and its owner are checked.  `dump_tailclass`
              was bitten by exactly this (seven phantom "references" invented
              by disassembling table bytes as code).

The image is sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258
(`C2_MAP_METHOD.md` §0); the script verifies the digest and refuses otherwise.
Function boundaries come from `docs/whitebox/ref/FUNCS.tsv`.  binutils
(`objdump`) is the only non-stdlib dependency.

Usage:
    python3 docs/whitebox/scripts/dump_movearms.py <c2.dll> --arms
    python3 docs/whitebox/scripts/dump_movearms.py <c2.dll> --nops
    python3 docs/whitebox/scripts/dump_movearms.py <c2.dll> --word 0x7d084378
    python3 docs/whitebox/scripts/dump_movearms.py <c2.dll> --chain
"""

import bisect
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

from dump_opcode_tables import (Image, PINNED_SHA256, MNEMONIC_TABLE_VA,   # noqa: E402
                                TABLE_STRIDE, BASE_WORD_TABLE_VA,
                                ENCODE_FORM_TABLE_VA, ARM_JUMP_TABLE_VA,
                                ARM_JUMP_TABLE_LEN, ARM_DEFAULT_VA)
from dump_expansion import disasm, target_of  # noqa: E402

UNLINK = 0x10C16CDE                 # the shared delete tail (w-tailread)
FORM37_ARM = 0x10BFA1AD             # the no-operand encoder form
FORM37_JUMPTAB = 0x10BFAFE9         # its 9-entry table, opcodes 0x277..0x27f
FORM37_FIRST_OP = 0x277
STALL_TABLE = 0x10C37DCC            # nopstall: cycles -> register number
STALL_CAP = 0xF
STALL_DEFAULT = 0x1F

# Handlers only.  THE THUNKS ARE NOT HARD-CODED, and that is deliberate: this
# file did hard-code them at first, from `P_OPATTR.md` §6.1, which names the
# HANDLERS and never claimed to give thunk addresses.  Two of the four guesses
# were wrong and --arms said so out loud ("MISMATCH (thunk calls …)"), which is
# the only reason they were caught.  They are now FOUND, by scanning the image
# for every `call rel32` that lands on the handler.
ARMS = [
    ("arm  6", "fmr", 0x10C16FBD),
    ("arm 14", "mr", 0x10C16D83),
    ("arm 15", "mr.", 0x10C1707C),
    ("arm 16", "vmr", 0x10C16E59),
]

FUNCS = os.path.join(HERE, "..", "ref", "FUNCS.tsv")


def load_funcs():
    rows = []
    for line in open(FUNCS):
        if line.startswith("#"):
            continue
        f = line.rstrip("\n").split("\t")
        try:
            rows.append((int(f[0], 16), int(f[1]), f[3] if len(f) > 3 else "?"))
        except (ValueError, IndexError):
            continue
    rows.sort()
    return rows


def owner(rows, starts, va):
    i = bisect.bisect_right(starts, va) - 1
    if i < 0:
        return None
    s, n, tu = rows[i]
    return (s, n, tu, va < s + n)


# ------------------------------------------------------------------- --arms

def callers_of(img, handler):
    """Every `e8 rel32` in the image whose target is `handler`.

    Byte-exact and alignment-blind on purpose: a hit is REPORTED with its
    owning function so a coincidence can be told from a call, the same way
    --word does.  The peephole thunks are 12-byte `mov ecx,esi / call H /
    jmp JOIN` bodies and fall out of this as the ones inside the arm table's
    address run.
    """
    out = []
    for name, vaddr, vsize, rawptr, rawsize in img.sections:
        if name != ".text":
            continue
        for off in range(rawptr, rawptr + rawsize - 5):
            if img.blob[off] != 0xE8:
                continue
            rel = int.from_bytes(img.blob[off + 1:off + 5], "little", signed=True)
            va = img.image_base + vaddr + (off - rawptr)
            if va + 5 + rel == handler:
                out.append(va)
    return out


def read_arm(img, path, name, mnem, handler, rows, starts):
    print("== %s  `%s`   handler %#x" % (name, mnem, handler))
    sites = callers_of(img, handler)
    print("   `call %#x` sites in .text: %d -> %s"
          % (handler, len(sites), ", ".join(hex(v) for v in sites) or "none"))
    for site in sites:
        thunk = site - 2
        t = disasm(path, thunk, thunk + 12)
        print("   thunk %#x: %s" % (thunk,
                                    " / ".join("%s %s" % (m, o) for _, m, o in t[:3])))
    o = owner(rows, starts, handler)
    if not o or o[0] != handler:
        print("   REFUSE: %#x is not a function start in FUNCS.tsv" % handler)
        return
    size, tu = o[1], o[2]
    ins = disasm(path, handler, handler + size)
    print("   handler: %d bytes, tu=%s, %d instructions" % (size, tu, len(ins)))

    # The same-register test: the first `cmp` whose two sides are both values
    # loaded from `+0x1c` of the two operand slots (+0x28 src, +0x2c dst).
    loads_1c = [va for va, m, ops in ins if m == "mov" and "+0x1c]" in ops]
    cmps = [(va, ops) for va, m, ops in ins if m in ("cmp", "sub")]
    print("   loads of operand[+0x1c]: %s"
          % ", ".join(hex(v) for v in loads_1c[:4]))
    if not cmps:
        print("   NO compare -- this arm does not test the operands")
        return
    cmp_va = cmps[0][0]
    print("   first compare at %#x  (%s)" % (cmp_va, cmps[0][1]))

    # The unlink: a call/jmp whose target is UNLINK, direct or via a block that
    # falls into one.
    unlink_sites = [va for va, m, ops in ins
                    if m in ("jmp", "call") and target_of(ops) == UNLINK]
    if not unlink_sites:
        # arm 14 reaches the unlink through an internal join; find a jmp to a
        # VA inside this handler that itself jmps to UNLINK.
        by_va = {va: (m, ops) for va, m, ops in ins}
        for va, m, ops in ins:
            t2 = target_of(ops) if m in ("jmp", "je", "jne") else None
            if t2 in by_va and by_va[t2][0] == "jmp" \
                    and target_of(by_va[t2][1]) == UNLINK:
                unlink_sites.append(va)
    if not unlink_sites:
        print("   NO path to the unlink %#x in this body" % UNLINK)
        return
    first_unlink = min(unlink_sites)
    print("   tail-call to the unlink %#x at %#x" % (UNLINK, first_unlink))

    # THE MEASUREMENT.  Conditional branches strictly between the compare and
    # the unlink, split by whether they can skip PAST the unlink (a guard that
    # could refuse the delete) or only jump forward WITHIN the equal path (a
    # sub-case that rejoins before it).
    #
    # ✘ AND THE FIRST ONE IS NOT A GUARD.  The compare's OWN not-equal exit
    # jumps past the unlink by construction -- that is what "the registers
    # differ" means -- so counting it made every arm score "GUARDED", including
    # arm 6, which `P_OPATTR.md` §6.1 read as an unconditional deleter.  Four
    # for four is the tell: a classifier that returns the same answer for every
    # input is measuring itself.  The equal path begins AFTER that branch.
    conds = [(va, m, ops) for va, m, ops in ins
             if m.startswith("j") and m != "jmp" and cmp_va < va < first_unlink]
    if not conds:
        print("   NO conditional branch between the compare and the unlink")
        return
    tva, tm, tops = conds[0]
    print("   the same-register test's own not-equal exit: %#x %s %s"
          % (tva, tm, tops))
    guards, inner = [], []
    for va, m, ops in conds[1:]:
        t2 = target_of(ops)
        (guards if (t2 is None or t2 > first_unlink) else inner).append(
            (va, m, ops, t2))
    print("   conditional branches between the compare and the unlink: %d"
          % (len(guards) + len(inner)))
    print("     GUARDS (can skip past the unlink):        %d" % len(guards))
    for va, m, ops, t2 in guards:
        print("       %#x %s %s" % (va, m, ops))
    print("     inner (rejoin before it, cannot refuse):  %d" % len(inner))
    for va, m, ops, t2 in inner:
        print("       %#x %s %s" % (va, m, ops))
    print("   VERDICT: %s"
          % ("the delete is GUARDED" if guards
             else "the delete is UNCONDITIONAL on the same-register path"))
    print()


# ------------------------------------------------------------------- --nops

def form37_word(img, op, rows, starts, path):
    """The word opcode `op` (encoder form 37) actually produces.

    Read from the arm rather than assumed: the base word is only the skeleton,
    and every arm ORs its own register fields into it.
    """
    base = img.u32(BASE_WORD_TABLE_VA + op * 4)
    if not FORM37_FIRST_OP <= op <= FORM37_FIRST_OP + 8:
        return base, "not in the form-37 switch (base word verbatim)"
    arm = img.u32(FORM37_JUMPTAB + (op - FORM37_FIRST_OP) * 4)
    if arm == ARM_DEFAULT_VA:
        return base, "arm is the join at %#x -- base word verbatim" % arm
    ins = disasm(path, arm, arm + 0x30)
    for va, m, ops in ins:
        if m == "or" and ops.startswith("ebx,0x"):
            imm = int(ops.split(",")[1], 16)
            w = base | imm
            return w, "arm %#x: base | %#x" % (arm, imm)
        if m == "mov" and ops.startswith("ebx,0x"):
            return int(ops.split(",")[1], 16), "arm %#x: literal" % arm
        if m == "jmp" and "0x10bfa3b3" in ops:
            break
    return None, ("arm %#x: computed at run time (see --nops notes)" % arm)


def regs_of(w, base):
    """(RS,RA,RB) that `w` adds on top of `base`, if it is an X-form or."""
    d = w ^ base
    return ((d >> 21) & 31, (d >> 16) & 31, (d >> 11) & 31)


def report_nops(img, path, rows, starts):
    print("c2's pseudo-nop family, and the word each one EMITS")
    print("(the question this answers: is `mr r8,r8` one of c2's nops?)")
    print()
    print("%-6s %-14s %-12s %-12s %s"
          % ("op", "mnemonic", "base word", "emits", "how"))
    for op in range(0x270, 0x292):
        nm = img.cstr(img.u32(MNEMONIC_TABLE_VA + op * TABLE_STRIDE) or 0)
        if not nm or not nm.startswith("nop"):
            continue
        base = img.u32(BASE_WORD_TABLE_VA + op * 4)
        w, how = form37_word(img, op, rows, starts, path)
        rs = regs_of(w, base) if w is not None else None
        extra = ""
        if rs and rs[0] == rs[1] == rs[2] and rs[0]:
            extra = "  = or r%d,r%d,r%d" % rs
        print("%#-6x %-14s %#-12x %-12s %s%s"
              % (op, nm, base, ("%#x" % w) if w is not None else "(dynamic)",
                 how, extra))
    print()
    print("`nopstall`'s register comes from the byte table %#x, indexed by the"
          % STALL_TABLE)
    print("requested stall in cycles, capped at %#x with default %d:"
          % (STALL_CAP, STALL_DEFAULT))
    off = img.off(STALL_TABLE)
    tbl = list(img.blob[off:off + STALL_CAP + 1])
    for cyc, reg in enumerate(tbl):
        print("   %2d cycles -> or r%d,r%d,r%d" % (cyc, reg, reg, reg))
    print("   otherwise -> or r%d,r%d,r%d"
          % (STALL_DEFAULT, STALL_DEFAULT, STALL_DEFAULT))
    print()
    print("NONE of these is `or r8,r8,r8` (%#x).  A `mr r8,r8` in an obj is"
          % 0x7D084378)
    print("therefore NOT one of c2's nop pseudo-ops -- it is opcode %#x `mr`."
          % 0x272)


# ------------------------------------------------------------------- --word

def report_word(img, word, rows, starts, path):
    pat = word.to_bytes(4, "little")
    print("literal %#010x (little-endian %s) in the image:" % (word, pat.hex(" ")))
    b, hits = img.blob, []
    i = 0
    while True:
        i = b.find(pat, i)
        if i < 0:
            break
        hits.append(i)
        i += 1
    if not hits:
        print("  ABSENT -- 0 occurrences, so no code carries it as an immediate")
        return
    for off in hits:
        va = None
        for name, vaddr, vsize, rawptr, rawsize in img.sections:
            if rawptr <= off < rawptr + rawsize:
                va = img.image_base + vaddr + (off - rawptr)
                sec = name
                break
        o = owner(rows, starts, va) if va else None
        # Is it an operand of a real instruction, or a byte coincidence?  Ask
        # objdump to decode the 16 bytes before it and see whether any
        # instruction's extent covers this VA as an immediate.
        real = "?"
        if va:
            try:
                ins = disasm(path, va - 16, va + 8)
                real = "coincidence"
                for iva, m, ops in ins:
                    if iva < va < iva + 8 and ("%#x" % word) in ops:
                        real = "IMMEDIATE of %s %s @%#x" % (m, ops, iva)
            except Exception:
                real = "(objdump refused)"
        print("  %s %#010x  owner=%s  %s"
              % (sec, va or 0,
                 ("%#x/%s%s" % (o[0], o[2], "" if o[3] else " (gap)")) if o else "-",
                 real))
    print()
    print("BOTH the tables and the code live in `.text`, so a hit is a")
    print("coincidence until its alignment and its owner are checked --")
    print("`dump_tailclass.py` was bitten by exactly this and counted seven")
    print("phantom references invented by disassembling table bytes as code.")


# ------------------------------------------------------------------ --chain

PEEP_ENTRY = 0x10C182B4
EXPAND_ARM_2E4 = 0x10C0E194
PSEUDO_OP = 0x2E4
EMIT_OP = 0x290


def report_chain(img, path, rows, starts):
    """The whole path from the pseudo-opcode to the word in the obj.

    Every step is READ from the image; the step that says WHY is marked as
    inference and is not one of them.
    """
    print("STEP 1 -- the peephole refuses the opcode before any arm sees it")
    ins = disasm(path, PEEP_ENTRY, PEEP_ENTRY + 0xC0)
    bounds = [(va, ops) for va, m, ops in ins
              if m == "cmp" and ops.endswith(("0x295", "0x292"))]
    for va, ops in bounds:
        print("   %#x  cmp %s" % (va, ops))
    print("   -> opcode %#x is >= 0x295, so FUN_%#x skips it at every one of"
          % (PSEUDO_OP, PEEP_ENTRY))
    print("      those bounds.  Arm 14 is never reached with it.")
    print()
    print("STEP 2 -- final expansion maps it to one arm")
    print("   FUN_0x10c0d57e (lower.c) opcode tree: %#x -> %#x"
          % (PSEUDO_OP, EXPAND_ARM_2E4))
    print("   (`dump_expansion.py --arms` recovers the tree; `P_EXPAND.md`")
    print("    §3 already scored this arm 1..1 words and named its opcode)")
    print()
    print("STEP 3 -- what that arm builds")
    for va, m, ops in disasm(path, EXPAND_ARM_2E4, EXPAND_ARM_2E4 + 0x26):
        note = ""
        if ops.startswith("0x7d084378") or ops == "0x7d084378":
            note = "   <== THE WORD: or r8,r8,r8 == `mr r8,r8`"
        if ops == "ecx,%#x" % EMIT_OP:
            note = "   <== opcode %#x, which the mnemonic table calls `%s`" % (
                EMIT_OP, img.cstr(img.u32(MNEMONIC_TABLE_VA
                                          + EMIT_OP * TABLE_STRIDE) or 0))
        print("   %#x  %-8s %s%s" % (va, m, ops, note))
    print()
    print("STEP 4 -- who mints opcode %#x" % PSEUDO_OP)
    seen = {}
    for name, vaddr, vsize, rawptr, rawsize in img.sections:
        if name != ".text":
            continue
        pat = b"\xb9" + PSEUDO_OP.to_bytes(4, "little")     # mov ecx,imm32
        off = rawptr
        end = rawptr + rawsize
        while True:
            off = img.blob.find(pat, off, end)
            if off < 0:
                break
            va = img.image_base + vaddr + (off - rawptr)
            o = owner(rows, starts, va)
            if o:
                seen.setdefault((o[0], o[2]), []).append(va)
            off += 1
    for (fn, tu), vas in sorted(seen.items()):
        print("   %#x/%-14s %d site(s): %s"
              % (fn, tu, len(vas), ", ".join(hex(v) for v in vas)))
    print("   (`mov ecx,imm32` only -- the ecx-passing convention c2 uses for")
    print("    the opcode argument.  Other registers carry it elsewhere; this")
    print("    is a lower bound on the minters, not the population.)")
    print()
    print("STEP 5 -- INFERENCE, not a read.  `%#x` keeps company with `%#x`"
          % (PSEUDO_OP, 0x21))
    print("   (`bc`) and `%#x` (`bca`) in three already-published branch-class"
          % 0x22)
    print("   predicates (WB_LOOP, WB_DAGCLIENTS, WB_MERGER4), so it is")
    print("   branch-LIKE; and it expands to exactly one inert word.  What it")
    print("   IS -- and its name -- is NOT read here and must not be quoted as")
    print("   though it were.")


def main(argv):
    if not argv:
        print(__doc__)
        return 2
    path = argv[0]
    img = Image(path)
    if img.digest != PINNED_SHA256:
        print("REFUSE: sha256 %s is not the pinned image %s"
              % (img.digest[:16], PINNED_SHA256[:16]))
        return 1
    rows = load_funcs()
    starts = [r[0] for r in rows]
    print("image OK: %s (%d B), %d functions in FUNCS.tsv"
          % (PINNED_SHA256[:16], len(img.blob), len(rows)))
    print()
    mode = argv[1] if len(argv) > 1 else None
    if mode == "--arms":
        for name, mnem, handler in ARMS:
            read_arm(img, path, name, mnem, handler, rows, starts)
        return 0
    if mode == "--nops":
        report_nops(img, path, rows, starts)
        return 0
    if mode == "--word":
        return report_word(img, int(argv[2], 0), rows, starts, path) or 0
    if mode == "--chain":
        report_chain(img, path, rows, starts)
        return 0
    print("REFUSE: pick a mode (--arms | --nops | --word W | --chain)")
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
