#!/usr/bin/env python3
"""Read what c2.dll does with a PSEUDO-OPCODE -- one above the machine opcode
space that `ref/P_ENCODE.md` bounds at `_last = 0x295`.

Lane `w-2e4`; prereg `docs/rungs/_2026-08-24-w-2e4-prereg.md`; findings
`docs/whitebox/WB_2E4_FINDINGS.md`.  Whitebox tooling, outside the std-only
`crates/` workspace per CLAUDE.md.  Written to answer the question
`w-r8idiom` refused: `WB_R8IDIOM_FINDINGS.md` §6 read three things about
`0x2e4` and declined to name it.

    0x10b1b260   the MNEMONIC table, stride 12 -- `_first` .. `_last`(0x295)
                 .. `illegal`(0x296) and THEN IT ENDS.  Row 0x2e4 of it lands
                 0x39c past the end, inside the table below, and yields the
                 real-looking string `twle`.  That is the trap this tool's
                 --names mode exists to make impossible to fall into.
    0x10b1d180   the EXTENDED-MNEMONIC (assembler alias) table, stride 16,
                 `_first` .. `_last` at row 120: alias -> {base opcode, BO, BI}.
                 A NEW ADDRESS for this record.
    0x10bd3750   the node allocator: `cl` = KIND, written to node[+8]
    0x10bd76e6   the kind-0x12 (branch) TUPLE CONSTRUCTOR: `ecx` = opcode
    0x10bd3824   splice BEFORE   ·   0x10bd3815   splice AFTER
                 (`+0x00` = next, `+0x10` = prev -- see --mint's notes)

Modes, and why each is a computation rather than an eyeball:

  --names N   Does ANY table in this image name opcode N?  Enumerates every
              array of pointers-to-C-strings in `.text`/`.data` at strides
              4/8/12/16, reports the ones long enough to hold an index N, and
              -- the part that matters -- runs a CONTROL of four known rows
              (0x21 `bc`, 0x22 `bca`, 0x276 `nop`, 0x290 `emit`) against each
              candidate.  A table that cannot fail its control is measuring
              itself (`w-r8idiom` defect 1).  `--names-selftest` watches the
              control FAIL at a deliberately wrong base.

  --sites N   Every instruction in `.text` carrying N as a 32-bit immediate,
              with owning function and TU, classified MINT / TEST / SELECT.
              MINT means the value reaches `ecx` of a known constructor with
              no intervening write -- computed, not assumed, because
              `w-r8idiom` published a table captioned "who mints 0x2e4" whose
              rows are mostly not mints.

  --mint N    Decodes each MINT site's five stack arguments and its `dl`,
              naming the splice callback and the operand source.

  --family N  For each TEST site, the OTHER opcode immediates compared against
              the same base within a window -- i.e. the predicate family N is
              tested as a member of.  This is what turns "tested with je in 18
              TUs" into a contract.

  --denom N   A disassembler-independent denominator: a raw byte scan for the
              little-endian dword, so `--sites`'s count can be checked against
              a number that does not depend on objdump synchronising.

The image this record is written against is
sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258
(`C2_MAP_METHOD.md` §0); the script verifies the digest and refuses otherwise.
Function boundaries come from `docs/whitebox/ref/FUNCS.tsv`.  binutils
(`objdump`) is the only non-stdlib dependency.

Usage:
    python3 docs/whitebox/scripts/dump_pseudoop.py <c2.dll> --names 0x2e4
    python3 docs/whitebox/scripts/dump_pseudoop.py <c2.dll> --names-selftest
    python3 docs/whitebox/scripts/dump_pseudoop.py <c2.dll> --sites 0x2e4
    python3 docs/whitebox/scripts/dump_pseudoop.py <c2.dll> --mint 0x2e4
    python3 docs/whitebox/scripts/dump_pseudoop.py <c2.dll> --family 0x2e4
    python3 docs/whitebox/scripts/dump_pseudoop.py <c2.dll> --denom 0x2e4
    python3 docs/whitebox/scripts/dump_pseudoop.py <c2.dll> --sites 0x2e4 --split 3
"""

import bisect
import os
import re
import struct
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

from dump_opcode_tables import (Image, PINNED_SHA256,          # noqa: E402
                                MNEMONIC_TABLE_VA, TABLE_STRIDE)
from dump_expansion import CONSTRUCTORS, disasm, LINE          # noqa: E402

FUNCS_TSV = os.path.join(HERE, "..", "ref", "FUNCS.tsv")

# The control rows.  Four opcodes whose mnemonics are published in
# `ref/P_DAG.md` §2.1 and `ref/P_ENCODE.md`, and were re-read here.
CONTROL = {0x21: "bc", 0x22: "bca", 0x276: "nop", 0x290: "emit"}

# The two list splices, read at 0x10bd3824 / 0x10bd3815.  `+0x00` is NEXT and
# `+0x10` is PREV: 0x10bd417d inserts a fresh label AFTER a tuple using
# 0x10bd3815's exact body, and `WB_MERGER4_FINDINGS.md` independently reads
# "walks both predecessors BACKWARDS through tuple+0x10".
SPLICE = {0x10BD3824: "splice BEFORE (+0x10 side)",
          0x10BD3815: "splice AFTER  (+0x00 side)"}

# Helpers a mint site commonly uses to produce its target-label argument.
ARGHELP = {
    0x10BD417D: "get-or-create the FALL-THROUGH label after this tuple",
    0x10BD415E: "wrap a label as a tuple",
    0x10B9A455: "mint a fresh label symbol",
}

REGS = ("eax", "ecx", "edx", "ebx", "esp", "ebp", "esi", "edi")


def load_funcs():
    rows = []
    with open(FUNCS_TSV) as fh:
        for line in fh:
            if line.startswith("#") or line.startswith("addr"):
                continue
            p = line.rstrip("\n").split("\t")
            if len(p) < 4:
                continue
            rows.append((int(p[0], 16), int(p[1]), p[3]))
    rows.sort()
    return rows, [r[0] for r in rows]


def owner(rows, starts, va):
    i = bisect.bisect_right(starts, va) - 1
    if i < 0:
        return None, "?"
    s, size, tu = rows[i]
    return (s, tu) if va < s + size else (None, "?")


def text_range(img):
    for name, vaddr, vsize, rawptr, rawsize in img.sections:
        if name == ".text":
            base = img.image_base + vaddr
            return base + 0x400, base + rawsize
    raise SystemExit("REFUSE: no .text")


def disasm_text(path, img, split=1):
    """Disassemble .text, optionally in `split` chunks with different
    boundaries.  The split is the INSTRUMENT-DEFECT knob: the site count must
    not depend on where objdump is told to start, and if it does the number is
    a property of the traversal (three lanes have now been bitten by exactly
    this shape).  Chunks overlap by 64 B and results are de-duplicated by VA."""
    lo, hi = text_range(img)
    if split <= 1:
        return disasm(path, lo, hi)
    seen = {}
    step = (hi - lo) // split
    for i in range(split):
        a = lo + i * step
        b = hi if i == split - 1 else lo + (i + 1) * step + 64
        for va, mn, ops in disasm(path, a, b):
            seen[va] = (mn, ops)
    return [(va, mn, ops) for va, (mn, ops) in sorted(seen.items())]


# ----------------------------------------------------------------- --names

def cstr(img, va, cap=64):
    o = img.off(va)
    if o is None:
        return None
    e = img.blob.find(b"\0", o, o + cap)
    if e < 0 or e == o:
        return None
    s = img.blob[o:e]
    return s.decode("ascii") if all(32 <= c < 127 for c in s) else None


def string_arrays(img, min_len=20):
    """Every maximal run of >= min_len consecutive entries whose first dword
    points at a printable C string, at each of four strides.

    EVERY 4-BYTE PHASE IS SCANNED, and that is not a detail.  The first
    version of this function started each stride at the section base, so a
    stride-12 table whose offset from the section base is not a multiple of 12
    was invisible -- and the table it made invisible was **the mnemonic table
    itself**, the one known-good name table in the image.  The defect was
    caught because a control that must pass did not appear in the enumeration
    at all.  A search for "is X named anywhere" that cannot see the one table
    that names things is not a search."""
    out = []
    for stride in (4, 8, 12, 16):
        for name, vaddr, vsize, rawptr, rawsize in img.sections:
            if name not in (".text", ".data"):
                continue
            base = img.image_base + vaddr
            for phase in range(stride // 4):
                origin = rawptr + phase * 4
                n = (rawsize - phase * 4 - 4) // stride
                i = 0
                while i < n:
                    v = struct.unpack_from("<I", img.blob, origin + i * stride)[0]
                    if v and cstr(img, v):
                        j = i
                        while j + 1 < n:
                            w = struct.unpack_from("<I", img.blob,
                                                   origin + (j + 1) * stride)[0]
                            if not (w and cstr(img, w)):
                                break
                            j += 1
                        if j - i + 1 >= min_len:
                            out.append((base + phase * 4 + i * stride, stride,
                                        j - i + 1))
                        i = j + 1
                    else:
                        i += 1
    return out


def control_score(img, base, stride):
    """How many of the four control rows this table reproduces."""
    hit = 0
    for op, want in CONTROL.items():
        p = img.u32(base + op * stride)
        if p and cstr(img, p) == want:
            hit += 1
    return hit


def mode_names(img, target):
    print("# --names 0x%x -- does ANY table in this image name it?" % target)
    print("#")
    print("# CONTROL rows required of a candidate: " +
          ", ".join("0x%x=%s" % kv for kv in sorted(CONTROL.items())))
    arrays = string_arrays(img)
    print("# %d char* arrays of >=20 consecutive entries in .text/.data\n" % len(arrays))
    print("%-12s %-6s %-6s %-8s %-8s %s" %
          ("base", "stride", "len", "covers?", "control", "row[N] / first rows"))
    named = []
    for base, stride, n in sorted(arrays):
        covers = "yes" if n > target else "no"
        ctl = control_score(img, base, stride)
        row = cstr(img, img.u32(base + target * stride) or 0) if n > target else None
        head = ", ".join(str(cstr(img, img.u32(base + k * stride) or 0))
                         for k in range(3))
        print("%-12s %-6d %-6d %-8s %d/4      %s" %
              (hex(base), stride, n, covers, ctl,
               (repr(row) + "   <== ") if row else "") + head)
        if covers == "yes" and ctl == len(CONTROL):
            named.append((base, stride, row))
    print()
    # The mnemonic table, checked explicitly -- it is the one a reader reaches
    # for, and it is the one that lies.
    mrow = cstr(img, img.u32(MNEMONIC_TABLE_VA + target * TABLE_STRIDE) or 0)
    last = cstr(img, img.u32(MNEMONIC_TABLE_VA + 0x295 * TABLE_STRIDE) or 0)
    end = MNEMONIC_TABLE_VA + 0x297 * TABLE_STRIDE
    print("# mnemonic table 0x%x: control %d/4, row 0x295=%r, row 0x296=%r,"
          % (MNEMONIC_TABLE_VA, control_score(img, MNEMONIC_TABLE_VA, TABLE_STRIDE),
             last, cstr(img, img.u32(MNEMONIC_TABLE_VA + 0x296 * TABLE_STRIDE) or 0)))
    print("#   so it ENDS at 0x%x. Row 0x%x would be at 0x%x, which is %#x PAST"
          % (end, target, MNEMONIC_TABLE_VA + target * TABLE_STRIDE,
             MNEMONIC_TABLE_VA + target * TABLE_STRIDE - end))
    print("#   the end and reads %r -- A COINCIDENCE, NOT A NAME." % mrow)
    print()
    if named:
        print("NAMED: opcode 0x%x is %s" % (target, named))
        return 0
    print("NOT NAMED: no table in this image both covers index 0x%x and passes"
          % target)
    print("the control. **The name of 0x%x is not readable from this binary.**"
          % target)
    return 0


def mode_names_selftest(img):
    """Watch the control FAIL. A classifier that cannot fail is measuring
    itself -- this is the fence, and it is run before --names is quoted."""
    ok = control_score(img, MNEMONIC_TABLE_VA, TABLE_STRIDE)
    print("control at the TRUE mnemonic base 0x%x: %d/4" % (MNEMONIC_TABLE_VA, ok))
    bad = 0
    for delta in (TABLE_STRIDE, -TABLE_STRIDE, 4, 0x1000):
        s = control_score(img, MNEMONIC_TABLE_VA + delta, TABLE_STRIDE)
        print("control at base+%#x: %d/4  %s" % (delta, s, "FAIL (good)" if s < 4 else "PASS (BAD!)"))
        if s >= 4:
            bad += 1
    for base, stride, n in sorted(string_arrays(img)):
        s = control_score(img, base, stride)
        if s == 4 and base != MNEMONIC_TABLE_VA:
            print("control PASSES at %s stride %d -- unexpected" % (hex(base), stride))
            bad += 1
    if ok != 4:
        print("REFUSE: the control does not pass at the true base")
        return 1
    if bad:
        print("REFUSE: the control passed where it must not")
        return 1
    print("OK: the control passes at the true base and fails everywhere else.")
    return 0


# ----------------------------------------------------------------- --sites

def classify(insns, idx_by_va, target):
    """MINT / TEST / SELECT for every immediate use of `target`."""
    imm = re.compile(r"\b0x%x\b" % target)
    out = []
    for i, (va, mn, ops) in enumerate(insns):
        if not imm.search(ops):
            continue
        kind = "TEST" if mn == "cmp" else "SELECT"
        detail = ""
        if mn == "mov":
            dst = ops.split(",")[0].strip()
            # follow forward: does this reach `ecx` at a constructor call with
            # no intervening write to the carrier?
            carrier = dst
            for j in range(i + 1, min(i + 24, len(insns))):
                v2, m2, o2 = insns[j]
                if m2 == "call":
                    t = o2.split()[0]
                    try:
                        tv = int(t, 16)
                    except ValueError:
                        tv = None
                    if tv in CONSTRUCTORS and carrier == "ecx":
                        kind, detail = "MINT", "-> 0x%x" % tv
                    break
                if m2 in ("jmp", "ret") or m2.startswith("j"):
                    break
                if m2 == "mov":
                    d2 = o2.split(",")[0].strip()
                    s2 = o2.split(",")[-1].strip()
                    if d2 == "ecx" and s2 == carrier:
                        carrier = "ecx"
                        continue
                    if d2 == carrier:
                        break
                elif carrier in o2.split(",")[0].strip():
                    break
            if kind == "SELECT":
                # is it compared, rather than passed?
                for j in range(i + 1, min(i + 8, len(insns))):
                    v2, m2, o2 = insns[j]
                    if m2 == "cmp" and re.search(r"\b%s\b" % carrier, o2):
                        kind, detail = "TEST", "(via %s)" % carrier
                        break
        out.append((va, mn, ops, kind, detail))
    return out


def mode_sites(path, img, target, split):
    rows, starts = load_funcs()
    insns = disasm_text(path, img, split)
    idx = {va: i for i, (va, _, _) in enumerate(insns)}
    sites = classify(insns, idx, target)
    from collections import Counter
    kinds = Counter(s[3] for s in sites)
    tus = Counter(owner(rows, starts, s[0])[1] for s in sites)
    fns = {owner(rows, starts, s[0])[0] for s in sites}
    print("# --sites 0x%x  (objdump split=%d, %d instructions in .text)"
          % (target, split, len(insns)))
    print("# %d sites | %s | %d functions | %d TUs"
          % (len(sites), dict(kinds), len(fns), len(tus)))
    print()
    for va, mn, ops, kind, detail in sites:
        fn, tu = owner(rows, starts, va)
        print("%-10s %-6s %-8s %-30s %-14s %s"
              % (hex(va), kind, mn, ops, tu, ("fn " + hex(fn)) if fn else "fn ?"))
    print()
    print("# by TU: " + ", ".join("%s %d" % kv for kv in tus.most_common()))
    return 0


def mode_mint(path, img, target, split):
    rows, starts = load_funcs()
    insns = disasm_text(path, img, split)
    sites = [s for s in classify(insns, {}, target) if s[3] == "MINT"]
    byva = {va: i for i, (va, _, _) in enumerate(insns)}
    print("# --mint 0x%x -- %d mint sites, arguments decoded" % (target, len(sites)))
    print("# 0x10bd76e6 is __fastcall(ecx=opcode, dl=cc) with 5 stack args,")
    print("# `ret 0x14`; LAST push is arg1 = the branch TARGET.")
    print()
    for va, mn, ops, kind, detail in sites:
        fn, tu = owner(rows, starts, va)
        i = byva[va]
        pushes, calls = [], []
        for j in range(max(0, i - 20), i):
            v2, m2, o2 = insns[j]
            if m2 == "push":
                pushes.append((v2, o2))
            elif m2 == "call":
                try:
                    calls.append((v2, int(o2.split()[0], 16)))
                except ValueError:
                    pass
            elif m2 in ("ret",) or (m2.startswith("j") and m2 != "jmp"):
                pushes, calls = [], []
        args = list(reversed(pushes[-5:]))
        print("%s  fn %s  %s" % (hex(va), hex(fn) if fn else "?", tu))
        for k, (v2, o2) in enumerate(args, 1):
            note = ""
            try:
                iv = int(o2, 16)
                note = "   <- " + SPLICE.get(iv, "")
            except ValueError:
                pass
            print("    arg%d  %-28s @%s%s" % (k, o2, hex(v2), note.rstrip()))
        for v2, t in calls[-3:]:
            if t in ARGHELP:
                print("    helper %s @%s  %s" % (hex(t), hex(v2), ARGHELP[t]))
        print()
    return 0


def mode_family(path, img, target, split, window=14, lo=0x10, hi=0x400):
    """Turn "tested with je in 18 TUs" into a contract.

    Two things are computed per TEST site: the CONDITION the test feeds (a
    `je`/`jne` is an equality membership test; a `ja`/`jb` is a range bound --
    they mean different things and a page that says only "tested with je"
    cannot tell them apart), and the set of OTHER opcode constants compared
    against the same base inside the window."""
    rows, starts = load_funcs()
    insns = disasm_text(path, img, split)
    imm = re.compile(r"\b0x%x\b" % target)
    other = re.compile(r"0x([0-9a-f]{2,4})\b")
    from collections import Counter
    fam, cond, tus = Counter(), Counter(), Counter()
    n_site = n_34 = 0
    f34 = re.compile(r"\[e[a-z]{2}\+0x34\],0x0")
    for i, (va, mn, ops) in enumerate(insns):
        if mn != "cmp" or not imm.search(ops):
            continue
        n_site += 1
        base = ops.split(",")[0].strip()
        nxt = insns[i + 1][1] if i + 1 < len(insns) else "?"
        cond[nxt] += 1
        tus[owner(rows, starts, va)[1]] += 1
        got = set()
        saw34 = False
        for j in range(max(0, i - window), min(len(insns), i + window + 1)):
            v2, m2, o2 = insns[j]
            if m2 != "cmp" and m2 != "sub":
                continue
            if f34.search(o2):
                saw34 = True
            if o2.split(",")[0].strip() != base:
                continue
            for m in other.finditer(o2.split(",")[-1]):
                x = int(m.group(1), 16)
                if lo <= x <= hi and x != target:
                    got.add(x)
        n_34 += saw34
        fam[tuple(sorted(got))] += 1
    print("# --family 0x%x -- what else is compared against the same base"
          % target)
    print("# window +-%d instructions; constants in 0x%x..0x%x counted"
          % (window, lo, hi))
    print("# %d TEST sites in %d TUs" % (n_site, len(tus)))
    bc = sum(n for k, n in fam.items() if 0x21 in k and 0x22 in k)
    print("# co-tested with BOTH 0x21 (bc) and 0x22 (bca): %d of %d (%.1f %%)"
          % (bc, n_site, 100.0 * bc / n_site))
    print("# window also carries a `cmp [reg+0x34],0x0`: %d of %d (%.1f %%)"
          % (n_34, n_site, 100.0 * n_34 / n_site))
    print()
    print("# the CONDITION each test feeds:")
    for k, n in cond.most_common():
        print("#   %-6s %4d" % (k, n))
    print()
    for key, n in fam.most_common():
        print("%4d  {%s}" % (n, ", ".join(hex(k) for k in key) or "-- alone --"))
    return 0


def mode_denom(img, target):
    """A denominator that does not depend on the disassembler synchronising."""
    needle = struct.pack("<I", target)
    for name, vaddr, vsize, rawptr, rawsize in img.sections:
        if name != ".text":
            continue
        blob = img.blob[rawptr:rawptr + rawsize]
        n = 0
        off = blob.find(needle)
        while off >= 0:
            n += 1
            off = blob.find(needle, off + 1)
        print("# raw byte scan of .text for %r: %d occurrences (ANY alignment)"
              % (needle, n))
        print("# This is an UPPER BOUND on --sites: it counts table bytes and")
        print("# straddling immediates too. It is here so --sites' count can be")
        print("# checked against something the disassembler cannot bias.")
    return 0


def main(argv):
    if len(argv) < 3:
        print(__doc__)
        return 2
    path = argv[1]
    img = Image(path)
    if img.digest != PINNED_SHA256:
        print("REFUSE: %s is sha256 %s, not the pinned %s"
              % (path, img.digest, PINNED_SHA256))
        return 1
    mode = argv[2]
    split = 1
    if "--split" in argv:
        split = int(argv[argv.index("--split") + 1])
    window = 14
    if "--window" in argv:
        window = int(argv[argv.index("--window") + 1])
    def arg():
        return int(argv[3], 0)
    if mode == "--names":
        return mode_names(img, arg())
    if mode == "--names-selftest":
        return mode_names_selftest(img)
    if mode == "--sites":
        return mode_sites(path, img, arg(), split)
    if mode == "--mint":
        return mode_mint(path, img, arg(), split)
    if mode == "--family":
        return mode_family(path, img, arg(), split, window)
    if mode == "--denom":
        return mode_denom(img, arg())
    print("REFUSE: unknown mode %r" % mode)
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
