#!/usr/bin/env python3
"""Which of c2's 79 encode arms does the 878-TU workload actually REACH?

Lane `w-encarms`, wave 18.  Whitebox tooling (python, outside the std-only
`crates/` workspace per CLAUDE.md; graded by its own `--self-test`, the shape
`scripts/gate_identity_diff.sh --self-test` set).

`docs/whitebox/ref/P_ENCODE.md` §10.4 says which arms the PORT implements
(27 of 79).  `crates/c2-harness/src/subsys.rs:669`'s own caveat says what is
missing beside it:

    OUTPUT PROXY, NOT A SITE COUNT ... Says nothing about which of the 79 arms
    the workload takes

This script answers exactly that.  For every executable `.text` word of the
real-`c2` reference objs of the workload, it attributes the word to a c2 opcode
(base-word table `0x10c3a578`), hence a form (`0x10c39b18`), hence an arm
(the jump table at `0x10bfae2d`), and histograms over the 79 arms.

    python3 work/w-encarms/armhist.py --self-test
    python3 work/w-encarms/armhist.py --objs <dir> --census <sections.jsonl>

ATTRIBUTION, and its honesty.  A PPC word is attributed by clearing the operand
bits its form implies and looking the residue up in c2's own base-word table.
The masks below are the standard PPC field layouts, tried MOST-SPECIFIC FIRST;
a word whose skeleton matches no table row is counted `unattributed` and
printed.  The denominator is printed beside every numerator (`#1002`).

AMBIGUITY IS NOT HIDDEN.  Several c2 opcodes share one base word (`or` and
`mr`; `addi` and `li`), so a word can name more than one opcode and therefore
more than one arm.  Two counts are reported per arm:

    unique   the word's candidate set names exactly this arm
    any      this arm is in the word's candidate set

`unique` is the one with teeth.
"""

import json
import os
import struct
import sys

# ---------------------------------------------------------------- PPC masks

# Each entry: (name, mask).  The mask keeps the bits that are NOT operand
# fields for that PPC form.  Tried in this order, most-specific first; the
# first mask whose residue is a base-word table row wins.
#
# PROVENANCE: these are the PowerPC ISA's own field layouts (Book I), not a
# reading of `c2.dll`.  Nothing here is adopted into `crates/`.
MASKS = [
    ("X/XO/XL/XFX (ext 21..31)", 0xFC0007FF),
    ("A (ext 26..30, Rc)", 0xFC00003F),
    ("MD/MDS (ext 27..30, Rc)", 0xFC00001F),
    ("MD (ext 27..29, Rc)", 0xFC00001D),
    ("DS (XO 30..31)", 0xFC000003),
    ("M (Rc)", 0xFC000001),
    ("D/I/B (primary only)", 0xFC000000),
]

IMAGE_SCN_CNT_CODE = 0x00000020
IMAGE_SCN_MEM_EXECUTE = 0x20000000
MACHINE_POWERPCBE = 0x01F2


def load_table(path):
    """ENCODE_OPCODES.txt -> {op: (mnemonic, base_word, form, arm)}."""
    rows = {}
    with open(path) as fh:
        for line in fh:
            if line.startswith("#") or not line.strip():
                continue
            f = line.split()
            rows[int(f[0], 16)] = (f[2], int(f[3], 16), int(f[4]), f[5])
    return rows


def build_index(rows, masks=None):
    """One index per mask: `base_word -> [op, ...]`, over the opcodes the mask
    can legally answer for.

    THE SUBSET RULE, and it is what keeps a broad mask from answering for an
    opcode it cannot see.  A mask indexes an opcode only when **every fixed bit
    of that opcode's base word survives the mask** (`bw & ~mask == 0`).  So the
    primary-only mask indexes `addi`/`ori`/`b` -- whose base words are a bare
    primary opcode -- and does NOT index `add`, whose extended opcode would have
    been masked away.  Without this rule every unmatched X-form word would fall
    through to the primary-only mask and be attributed to an unrelated D-form
    opcode, which is a silent wrong answer rather than an honest residue.
    """
    masks = MASKS if masks is None else masks
    idx = []
    for name, mask in masks:
        m = {}
        for op, (_mn, bw, _form, _arm) in rows.items():
            if bw & ~mask == 0:
                m.setdefault(bw, []).append(op)
        idx.append((name, mask, m))
    return idx


def attribute(word, idx):
    """Return (mask_name, [op, ...]) or (None, [])."""
    for name, mask, m in idx:
        cand = m.get(word & mask)
        if cand:
            return name, cand
    return None, []


# `P_ENCODE.md` §6's table, read from the OUTPUT side: which encode FORMS can be
# standing at a site carrying each relocation type.  A form absent from a row
# cannot have produced that site, so the relocation prunes the candidate set.
#
# THIS PRUNE IS CONDITIONED ON A `[R]` READING and is reported as a SEPARATE
# pass for exactly that reason: §6 is instruction-level reading of `c2.dll`,
# not an obj-confirmed fact, and folding it into the raw histogram would launder
# a hypothesis into a measurement.
#
#   REL24     0x10bf976d, from form 7 (`bl`).  Form 6 (`b`) reaches it by
#             falling into form 7's tail when the target is not a local label
#             (§5.3), so both forms are admitted here and the ARM is 10bfa285
#             either way.
#   REFHI     0x10bf96ea, from form 51 when the opcode is `addis`, and form 30
#             (`lau`) by `jmp 0x10bfa522` into form 51's tail.
#   REFLO     0x10bf9721 from form 29 (`lal`); 0x10bf9808 from the two D-form
#             memory composers, forms 21/45/46 (load) and 27/58/71 (store).
#   SECREL16  0x10bf9758, from form 34 (`loffs`).
#   ADDR32    form 65 (`DCD`).
#   IFGLUE    form 37, and only at opcode 0x280 (`rsttoc`).
REL_FORMS = {
    "REL24": {6, 7},
    "REFHI": {51, 30},
    "REFLO": {29, 21, 45, 46, 27, 58, 71},
    "SECREL16": {34},
    "ADDR32": {65},
    "IFGLUE": {37},
}


# ---------------------------------------------------------------- COFF

# IMAGE_REL_PPC_* -- the type codes `P_ENCODE.md` §6 says the encoder asks for.
RELNAME = {
    0x02: "ADDR32",
    0x06: "REL24",
    0x0D: "IFGLUE",
    0x0F: "SECREL16",
    0x10: "REFHI",
    0x11: "REFLO",
    0x12: "PAIR",
}


def text_words(data):
    """Yield `(word, reloc_type_or_None)` for every executable-section word.

    The relocation is the ONLY thing in the obj that can tell two c2 opcodes
    with the same base word apart -- `bl` (form 7, REL24) from `bgip` (form 2,
    none), `addis`/`lau` (REFHI) from `lis` (none).  `P_ENCODE.md` §6 is the
    table of which arm asks for which type; this is that table read from the
    output side.
    """
    if len(data) < 20:
        return
    machine, nsec = struct.unpack_from("<HH", data, 0)
    if machine != MACHINE_POWERPCBE:
        return
    opt = struct.unpack_from("<H", data, 16)[0]
    base = 20 + opt
    for i in range(nsec):
        off = base + i * 40
        if off + 40 > len(data):
            return
        # COFF section header: name 0, vsize 8, vaddr 12, rawsize 16, rawptr 20,
        # relptr 24, lineptr 28, nrel 32(u16), nline 34(u16), chars 36.
        size, ptr, prel = struct.unpack_from("<III", data, off + 16)
        nrel = struct.unpack_from("<H", data, off + 32)[0]
        chars = struct.unpack_from("<I", data, off + 36)[0]
        if not (chars & IMAGE_SCN_MEM_EXECUTE) or not (chars & IMAGE_SCN_CNT_CODE):
            continue
        if ptr == 0 or size == 0 or ptr + size > len(data):
            continue
        rel = {}
        if prel and nrel and prel + 10 * nrel <= len(data):
            for k in range(nrel):
                va, _sym, ty = struct.unpack_from("<IIH", data, prel + 10 * k)
                # PAIR carries an addend, not a site; it never annotates a word.
                if ty != 0x12:
                    rel[va] = ty
        n = size & ~3
        for j in range(0, n, 4):
            yield struct.unpack_from(">I", data, ptr + j)[0], rel.get(j)


# ---------------------------------------------------------------- self-test

def self_test():
    """Watched-RED control set.  Exits 1 on any failure."""
    here = os.path.dirname(os.path.abspath(__file__))
    repo = os.path.abspath(os.path.join(here, "..", ".."))
    rows = load_table(os.path.join(repo, "docs/whitebox/ref/ENCODE_OPCODES.txt"))
    idx = build_index(rows)
    bad = 0

    # T1 -- the table is the size the page says it is.
    if len(rows) != 660:
        print(f"T1 FAIL: {len(rows)} opcode rows, expected 660")
        bad += 1

    # T2 -- hand-decoded real words attribute to the right mnemonic.
    cases = [
        (0x7C0802A6, "mfspr"),   # mflr r0
        (0x7D8802A6, "mfspr"),   # mflr r12
        (0x7D8803A6, "mtspr"),   # mtlr r12
        (0x48000001, "bl"),      # bl <self>
        (0x9421FFF0, "stwu"),
        (0x9181FFF8, "stw"),     # stw r12,-8(r1)
        (0x8181FFF8, "lwz"),
        (0x4E800020, "blr"),
        (0x7C641B78, "or"),      # or r4,r3,r3  (also `mr`)
        (0x38600000, "addi"),    # li r3,0      (also `li`)
        (0xFC000090, "fmr"),
        (0xFC00002A, "fadd"),
        (0x10000000 | (4 << 21) | (5 << 16) | (6 << 11) | 0x0A, "vaddfp"),
    ]
    for word, want in cases:
        _m, cand = attribute(word, idx)
        mns = {rows[o][0].rstrip(".") for o in cand}
        if want not in mns:
            print(f"T2 FAIL: {word:#010x} -> {sorted(mns)}, expected {want}")
            bad += 1

    # T3 -- CONTROL, both directions, and each must MOVE.  `#3336`: a control
    # nobody has seen fail is decoration.  P_ENCODE §8.2's own control set is
    # the model: a mutated field width has to change the attribution.
    #
    #   T3a  WIDEN every mask to keep the RB field (bits 16..20, value
    #        0x0000F800) -- every word with a non-zero RB must stop attributing.
    #   T3b  NARROW every mask by dropping the XO/OE bit 21 (value 0x400) --
    #        an `addo` word must stop naming `addo`.  It does NOT go dark: its
    #        residue becomes `addc`'s base word, so the mutation produces a
    #        SILENT WRONG ANSWER, which is the failure mode worth having a
    #        control for.
    probe = [
        0x7C641B78,                                      # or   r4,r3,r3  RB=3
        0x7C000214 | (3 << 21) | (4 << 16) | (5 << 11),  # add  r3,r4,r5  RB=5
        0x7C0802A6,                                      # mflr r0        RB=0
    ]
    if sum(1 for w in probe if attribute(w, idx)[1]) != len(probe):
        print("T3 FAIL: baseline does not attribute all three probes")
        bad += 1
    widened = build_index(rows, [(n, m | 0x0000F800) for n, m in MASKS])
    dark = sum(1 for w in probe if not attribute(w, widened)[1])
    if dark != 2:
        print(f"T3a FAIL: widening the mask over RB left {dark} probes dark, expected 2")
        bad += 1
    narrowed = build_index(rows, [(n, m & ~0x00000400) for n, m in MASKS])
    addco = 0x7C000414 | (3 << 21) | (4 << 16) | (5 << 11)
    base_mn = {rows[o][0] for o in attribute(addco, idx)[1]}
    mut_mn = {rows[o][0] for o in attribute(addco, narrowed)[1]}
    if "addco" not in base_mn:
        print(f"T3b FAIL: baseline read an `addco` word as {sorted(base_mn)}")
        bad += 1
    if "addco" in mut_mn or not mut_mn:
        print(f"T3b FAIL: dropping the OE bit read the `addco` word as {sorted(mut_mn)} -- "
              f"the bit is not load-bearing")
        bad += 1

    # T4 -- every arm named by the opcode table is one of the 79.
    arms_path = os.path.join(repo, "docs/whitebox/ref/ENCODE_ARMS.txt")
    arms = set()
    with open(arms_path) as fh:
        for line in fh:
            if not line.startswith("#") and line.strip():
                arms.add(line.split()[0])
    seen = {a for (_m, _b, _f, a) in rows.values()}
    # 0x10bfae1b is the default tail, reached both by the `ja` and by 12 forms.
    if not seen.issubset(arms | {"10bfae1b", "-"}):
        print(f"T4 FAIL: opcode table names arms not in ENCODE_ARMS.txt: {sorted(seen - arms)}")
        bad += 1
    if len(arms) != 79:
        print(f"T4 FAIL: {len(arms)} arms, expected 79")
        bad += 1

    print("self-test: RED" if bad else f"self-test: GREEN ({len(rows)} opcodes, {len(arms)} arms, {len(cases)} decode cases)")
    return 1 if bad else 0


# ---------------------------------------------------------------- main

def main(argv):
    if "--self-test" in argv:
        return self_test()

    here = os.path.dirname(os.path.abspath(__file__))
    repo = os.path.abspath(os.path.join(here, "..", ".."))
    objdir = None
    census = os.path.join(repo, "work/w-bss/census/sections.jsonl")
    for i, a in enumerate(argv):
        if a == "--objs":
            objdir = argv[i + 1]
        elif a == "--census":
            census = argv[i + 1]
    if objdir is None:
        print("usage: armhist.py --objs <build/373307D9> [--census <sections.jsonl>]")
        return 2

    rows = load_table(os.path.join(repo, "docs/whitebox/ref/ENCODE_OPCODES.txt"))
    idx = build_index(rows)

    ported = set()
    with open(os.path.join(repo, "work/w-encmap/armmap.txt")) as fh:
        for line in fh:
            if line.startswith("| `10bf"):
                ported.add(line.split("`")[1])

    srcs = [json.loads(l)["src"] for l in open(census)]
    present, missing = [], []
    for s in srcs:
        p = os.path.join(objdir, s.rsplit(".", 1)[0] + ".obj")
        (present if os.path.exists(p) else missing).append(p if os.path.exists(p) else s)

    # Opcodes whose base word is ZERO cannot be told from `.text` padding, and
    # three of them (`rlandi`, `rlandi.`, `deadtmp`) are not instructions the
    # workload can contain.  Attributing an all-zero word to them was the first
    # draft's silent 100 % (`P_ENCODE.md` §2.2 is the reason it is wrong).
    zero_ops = {op for op, (_m, bw, _f, _a) in rows.items() if bw == 0}

    uniq, anyc, opcount, unattr, refined = {}, {}, {}, {}, {}
    relsplit = {}        # arm -> {reloc name or "-": count}
    primary = {}         # primary opcode -> count
    total = zero = 0
    for p in present:
        with open(p, "rb") as fh:
            data = fh.read()
        for w, rt in text_words(data):
            total += 1
            primary[w >> 26] = primary.get(w >> 26, 0) + 1
            if w == 0:
                zero += 1
                continue
            _m, cand = attribute(w, idx)
            cand = [o for o in cand if o not in zero_ops]
            if not cand:
                unattr[w] = unattr.get(w, 0) + 1
                continue
            arms = {rows[o][3] for o in cand}
            rn = RELNAME.get(rt, "-") if rt is not None else "-"
            for o in cand:
                opcount[o] = opcount.get(o, 0) + 1
            for a in arms:
                anyc[a] = anyc.get(a, 0) + 1
                relsplit.setdefault(a, {})[rn] = relsplit.setdefault(a, {}).get(rn, 0) + 1
            if len(arms) == 1:
                a = arms.pop()
                uniq[a] = uniq.get(a, 0) + 1
            # -- second pass: §6's relocation prune, kept separate on purpose.
            keep = REL_FORMS.get(rn)
            pruned = [o for o in cand if rows[o][2] in keep] if keep else cand
            parms = {rows[o][3] for o in pruned} if pruned else set()
            if len(parms) == 1:
                a = next(iter(parms))
                refined[a] = refined.get(a, 0) + 1

    nun = sum(unattr.values())
    den = total - zero
    print(f"# workload objs: {len(present)} present / {len(srcs)} census TUs "
          f"({len(missing)} with no obj under the reference build)")
    for m in missing:
        print(f"#   no obj: {m}")
    print(f"# executable .text words: {total}")
    print(f"# all-zero words (padding, or `emit 0`): {zero} "
          f"({100.0 * zero / max(total, 1):.4f} %) -- EXCLUDED, see zero_ops")
    print(f"# denominator (non-zero words): {den}")
    print(f"# attributed: {den - nun} ({100.0 * (den - nun) / max(den, 1):.4f} %)   "
          f"unattributed: {nun} over {len(unattr)} distinct words")
    print(f"# ported arms (work/w-encmap/armmap.txt): {len(ported)} of 79")
    print()

    all_arms = {}
    for op, (_mn, _bw, form, arm) in rows.items():
        all_arms.setdefault(arm, set()).add(form)

    unmapped = [a for a in all_arms if a not in ported]
    u_uniq = [a for a in unmapped if uniq.get(a, 0)]
    u_anyonly = [a for a in unmapped if not uniq.get(a, 0) and anyc.get(a, 0)]
    u_zero = [a for a in unmapped if not anyc.get(a, 0)]
    r_uniq = [a for a in unmapped if refined.get(a, 0)]
    r_zero = [a for a in unmapped if not refined.get(a, 0) and not anyc.get(a, 0)]
    print("## THE ANSWER — of the unmapped arms, over this workload")
    print(f"##   {len(unmapped):2} unmapped arms  (79 total - {len(ported)} ported)")
    print(f"##   {len(u_uniq):2} reached UNAMBIGUOUSLY (some word names this arm and no other)")
    print(f"##   {len(u_anyonly):2} reached only AMBIGUOUSLY (every word that could be this arm "
          f"could equally be another)")
    print(f"##   {len(u_zero):2} NOT REACHED AT ALL — zero words in {len(present)} workload objs")
    print(f"##   unambiguous: {' '.join(sorted(u_uniq))}")
    print(f"##   ambiguous  : {' '.join(sorted(u_anyonly))}")
    print()
    print("## SECOND PASS — the same words with §6's relocation prune applied.")
    print("## CONDITIONED ON A [R] READING, not an obj-confirmed fact; kept separate")
    print("## so a hypothesis is not laundered into a measurement.")
    print(f"##   {len(r_uniq):2} unmapped arms named UNIQUELY once the relocation prunes the candidates")
    print(f"##   {len(r_zero):2} still not reached at all")
    print(f"##   refined-unique: {' '.join(sorted(r_uniq))}")
    print()

    print("arm       ported  uniq_words     refined_uniq       any_words   reloc split           forms")
    for a in sorted(all_arms, key=lambda x: (-refined.get(x, 0), -uniq.get(x, 0), -anyc.get(x, 0), x)):
        rs = relsplit.get(a, {})
        rstxt = " ".join(f"{k}={v}" for k, v in sorted(rs.items(), key=lambda kv: -kv[1]))
        print(f"{a}  {'Y' if a in ported else '.':>6}  {uniq.get(a, 0):11}  {refined.get(a, 0):15}  "
              f"{anyc.get(a, 0):15}   {rstxt:<20} {','.join(str(f) for f in sorted(all_arms[a]))}")
    print()
    print("# primary-opcode census of the same words")
    for po, c in sorted(primary.items(), key=lambda kv: -kv[1]):
        print(f"#   primary {po:2}  x{c}")
    print()
    print("# top unattributed words")
    for w, c in sorted(unattr.items(), key=lambda kv: -kv[1])[:25]:
        print(f"#   {w:#010x}  primary {w >> 26:2}  x{c}")
    print()
    print("# per-opcode counts (top 70)")
    for op, c in sorted(opcount.items(), key=lambda kv: -kv[1])[:70]:
        mn, bw, form, arm = rows[op]
        print(f"#   {op:#06x} {mn:<12} form {form:3}  arm {arm}  "
              f"{'PORTED' if arm in ported else 'unmapped':8}  x{c}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
