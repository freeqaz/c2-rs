#!/usr/bin/env python3
"""Re-derive c2.dll's `.ex` OPERAND-CLASS dispatch and all 29 class arms.

Lane `w-opclass`, board #3585-#3590.  Whitebox tooling (outside the std-only
`crates/` workspace, per CLAUDE.md).

    python3 docs/whitebox/scripts/dump_opclass.py <c2.dll> --verify
    python3 docs/whitebox/scripts/dump_opclass.py <c2.dll> --arms
    python3 docs/whitebox/scripts/dump_opclass.py <c2.dll> --grammar
    python3 docs/whitebox/scripts/dump_opclass.py <c2.dll> --prims
    python3 docs/whitebox/scripts/dump_opclass.py <c2.dll> --classmap
    python3 docs/whitebox/scripts/dump_opclass.py <c2.dll> --cross

**ONE hard-coded address.**  `OPERAND_DECODER_VA` is the only address this file
asserts.  The class table VA, the class bound, the number of classes, the jump
table VA, the out-of-range arm, all 29 arm targets, the function's epilogue and
every primitive callee are *derived* from the operand bytes of instructions
decoded from that entry.  That is `w-ilarms`'s rule (`dump_ilarms.py`'s module
doc) and it is why re-running this can falsify its own constants instead of only
confirming that the bytes at a remembered address have not moved.

`WB_READER_FINDINGS.md` §3 (lane `wb-reader`, 2026-08-08, board #1591) already
publishes a one-line grammar per class.  **This script does not read that page**
— agreement is a control (`--cross` prints the comparison against
`dump_ilarms.py`'s independent class-byte reader) and disagreement would be the
finding.  `#3547` is why: a prior page's cell was wrong in both of its clauses
and nobody had re-derived it.

Image pin: sha256
c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258
(`C2_MAP_METHOD.md` §0).  The script verifies the digest and refuses otherwise.

`objdump` is the only non-stdlib dependency, invoked on the pinned image itself
exactly as `dump_expansion.py` does.
"""

import hashlib
import re
import struct
import subprocess
import sys

PINNED = "c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258"

# The ONLY hard-coded address in this file: the `.ex` operand decoder's entry.
OPERAND_DECODER_VA = 0x10B3D610

# Reporting joins only.  No claim below depends on either:
#  - the per-opcode operand-class table is DERIVED from the decoder's `movzx`;
#    this constant exists solely so `--cross` can assert the two agree.
CROSS_CLASS_TABLE_VA = 0x10B25E48

LINE = re.compile(r"^\s*([0-9a-f]+):\s+((?:[0-9a-f]{2} )+)\s*\t(\S+)\s*(.*)$")
UNCOND = {"jmp"}
BRANCH = {
    "jmp", "je", "jne", "jz", "jnz", "ja", "jae", "jb", "jbe", "jg", "jge",
    "jl", "jle", "js", "jns", "jo", "jno", "jp", "jnp",
}


class Image:
    """Minimal PE reader: VA -> bytes.  Written for this lane."""

    def __init__(self, path):
        self.path = path
        self.raw = open(path, "rb").read()
        got = hashlib.sha256(self.raw).hexdigest()
        if got != PINNED:
            raise SystemExit(f"REFUSING: sha256 {got} != pinned {PINNED}")
        e_lfanew = struct.unpack_from("<I", self.raw, 0x3C)[0]
        if self.raw[e_lfanew:e_lfanew + 4] != b"PE\0\0":
            raise SystemExit("not a PE")
        coff = e_lfanew + 4
        nsec, = struct.unpack_from("<H", self.raw, coff + 2)
        optsz, = struct.unpack_from("<H", self.raw, coff + 16)
        opt = coff + 20
        magic, = struct.unpack_from("<H", self.raw, opt)
        if magic != 0x10B:
            raise SystemExit(f"expected PE32, got magic {magic:#x}")
        self.base, = struct.unpack_from("<I", self.raw, opt + 28)
        self.sections = []
        sh = opt + optsz
        for i in range(nsec):
            o = sh + i * 40
            name = self.raw[o:o + 8].rstrip(b"\0").decode("ascii", "replace")
            vsize, vaddr, rsize, raddr = struct.unpack_from("<IIII", self.raw, o + 8)
            self.sections.append((name, vaddr, vsize, raddr, rsize))

    def off(self, va):
        rva = va - self.base
        for name, vaddr, vsize, raddr, rsize in self.sections:
            if vaddr <= rva < vaddr + max(vsize, rsize):
                d = rva - vaddr
                if d < rsize:
                    return raddr + d
                raise SystemExit(f"VA {va:#x} is in {name} but past raw data")
        raise SystemExit(f"VA {va:#x} in no section")

    def bytes(self, va, n):
        o = self.off(va)
        return self.raw[o:o + n]

    def u8(self, va):
        return self.bytes(va, 1)[0]

    def u32(self, va):
        return struct.unpack("<I", self.bytes(va, 4))[0]

    def section_of(self, va):
        rva = va - self.base
        for name, vaddr, vsize, raddr, rsize in self.sections:
            if vaddr <= rva < vaddr + max(vsize, rsize):
                return name
        return "?"


def disasm(path, lo, hi):
    """[(va, hexbytes, mnemonic, operands)] straight from the pinned image."""
    out = subprocess.run(
        ["objdump", "-d", "-M", "intel",
         "--start-address=%#x" % lo, "--stop-address=%#x" % hi, path],
        capture_output=True, text=True, check=True).stdout
    insns = []
    for line in out.splitlines():
        m = LINE.match(line)
        if m:
            insns.append((int(m.group(1), 16), m.group(2).strip(),
                          m.group(3), m.group(4).strip()))
    return insns


def target_of(op_text):
    m = re.match(r"^(0x[0-9a-f]+)", op_text)
    return int(m.group(1), 16) if m else None


class Decoder:
    """Decode the operand-class dispatch head, deriving every table."""

    def __init__(self, img, head_va=OPERAND_DECODER_VA):
        self.img = img
        self.head_va = head_va
        # Decode forward from the entry until the indexed `jmp` is found.  The
        # window is generous; the head is < 0x40 bytes in practice and the loop
        # refuses rather than guessing if the shape is not there.
        insns = disasm(img.path, head_va, head_va + 0x60)
        self.head = []
        self.class_table_va = None
        self.bound = None
        self.ja_target = None
        self.jump_table_va = None
        for va, hexb, mn, ops in insns:
            self.head.append((va, hexb, mn, ops))
            if mn == "movzx" and "+0x" in ops and self.class_table_va is None:
                m = re.search(r"\[e[a-z]{2}\+(0x[0-9a-f]+)\]", ops)
                if m:
                    self.class_table_va = int(m.group(1), 16)
            elif mn == "cmp" and self.bound is None and self.class_table_va is not None:
                m = re.search(r",\s*(0x[0-9a-f]+|\d+)$", ops)
                if m:
                    self.bound = int(m.group(1), 0)
            elif mn == "ja" and self.ja_target is None:
                self.ja_target = target_of(ops)
            elif mn == "jmp" and "*4+" in ops:
                m = re.search(r"\[eax\*4\+(0x[0-9a-f]+)\]", ops)
                if m:
                    self.jump_table_va = int(m.group(1), 16)
                break
        for name, v in (("class table", self.class_table_va),
                        ("class bound", self.bound),
                        ("out-of-range ja", self.ja_target),
                        ("jump table", self.jump_table_va)):
            if v is None:
                raise SystemExit(f"REFUSING: could not derive the {name} from "
                                 f"{head_va:#x} — the shape this script decodes "
                                 f"is not there")
        self.n_classes = self.bound + 1
        self.arms = [img.u32(self.jump_table_va + 4 * k)
                     for k in range(self.n_classes)]
        # The function body is everything from the entry up to the jump table,
        # which sits immediately after it (checked in --verify).
        self.body_lo = head_va
        self.body_hi = self.jump_table_va
        self.insns = disasm(img.path, self.body_lo, self.body_hi)
        self.by_va = {va: i for i, (va, _, _, _) in enumerate(self.insns)}
        self.arm_entries = set(self.arms)
        self.epilogue = self._derive_epilogue()

    def _derive_epilogue(self):
        """The common exit — DERIVED as the block ending in `leave; ret`.

        Not assumed to be class 00's arm (it is, and that is a *result*: the
        payload-free class jumps straight to the epilogue).
        """
        for i, (va, _, mn, _) in enumerate(self.insns):
            if mn.startswith("ret"):
                # walk back to the start of this basic block: the first
                # instruction that nothing falls through into is where every
                # arm's `jmp` lands, so take the lowest jump target <= va that
                # is also >= the previous control-transfer.
                targets = set()
                for va2, _, mn2, ops2 in self.insns:
                    if mn2 in BRANCH:
                        t = target_of(ops2)
                        if t is not None and t <= va:
                            targets.add(t)
                cand = [t for t in targets if t <= va]
                # the epilogue head is the largest such target that still
                # reaches `ret` by straight-line fallthrough
                for t in sorted(cand, reverse=True):
                    j = self.by_va.get(t)
                    if j is None:
                        continue
                    ok = True
                    k = j
                    while k < len(self.insns) and self.insns[k][0] <= va:
                        if self.insns[k][2] in BRANCH:
                            ok = False
                            break
                        k += 1
                    if ok:
                        return t
                return va
        raise SystemExit("REFUSING: no `ret` inside the decoder body")

    # -- derived views -----------------------------------------------------

    def distinct_arms(self):
        return sorted(set(self.arms))

    def shared_arms(self):
        d = {}
        for k, t in enumerate(self.arms):
            d.setdefault(t, []).append(k)
        return {t: ks for t, ks in d.items() if len(ks) > 1}

    def opcode_classes(self, lo=0x00, hi=0xBF):
        """opcode -> class byte, straight out of the derived table VA."""
        return {o: self.img.u8(self.class_table_va + o) for o in range(lo, hi + 1)}

    def walk_arm(self, k):
        """Ordered trace of one class arm.

        Returns [(va, kind, detail)] where kind is 'call', 'cond', 'join' or
        'exit'.  Conditional branches are recorded with BOTH successors and the
        walk continues down the fallthrough, then down the taken side, so a
        gated read shows up as a `cond` row rather than being silently taken or
        silently dropped.
        """
        start = self.arms[k]
        trace = []
        seen = set()
        work = [start]
        while work:
            va = work.pop(0)
            while True:
                if va in seen:
                    trace.append((va, "join", f"-> {va:#x} (already walked)"))
                    break
                if va == self.epilogue:
                    trace.append((va, "exit", "epilogue"))
                    break
                i = self.by_va.get(va)
                if i is None:
                    trace.append((va, "exit", "outside the decoder body"))
                    break
                seen.add(va)
                _, _, mn, ops = self.insns[i]
                nxt = self.insns[i + 1][0] if i + 1 < len(self.insns) else None
                if mn == "call":
                    t = target_of(ops)
                    trace.append((va, "call", t))
                    # A callee whose return address is ANOTHER class arm's entry
                    # does not return: the compiler laid the next arm where the
                    # fallthrough would be, which it can only do for a noreturn
                    # call.  Derived from the arm table, not from a list of
                    # known error routines.
                    if nxt in self.arm_entries:
                        cls = [f"{i:02X}" for i, a in enumerate(self.arms) if a == nxt]
                        trace.append((nxt, "exit",
                                      f"callee does not return — the next byte is "
                                      f"class {'/'.join(cls)}'s own arm"))
                        break
                    va = nxt
                elif mn in UNCOND:
                    t = target_of(ops)
                    if t is None:
                        trace.append((va, "exit", f"indirect {mn} {ops}"))
                        break
                    va = t
                elif mn in BRANCH:
                    t = target_of(ops)
                    trace.append((va, "cond", f"{mn} -> {t:#x} / fall {nxt:#x}"))
                    if t is not None and t not in seen:
                        work.append(t)
                    va = nxt
                elif mn.startswith("ret"):
                    trace.append((va, "exit", "ret"))
                    break
                else:
                    va = nxt
                if va is None:
                    trace.append((0, "exit", "end of body"))
                    break
        return trace

    def call_seq(self, k):
        """The ordered call targets of arm `k` along its first path."""
        return [d for _, kind, d in self.walk_arm(k) if kind == "call"]


def callee_inventory(d):
    """{callee VA: [class indices that call it]} over all arms."""
    inv = {}
    for k in range(d.n_classes):
        for t in d.call_seq(k):
            inv.setdefault(t, set()).add(k)
    return {t: sorted(ks) for t, ks in sorted(inv.items())}


def cmd_verify(img, d):
    print("== the operand decoder head, decoded from raw bytes ==")
    for va, hexb, mn, ops in d.head:
        print(f"  {va:08x}  {hexb:<24}  {mn} {ops}")
    print()
    print("== derived, not assumed ==")
    print(f"  class table          {d.class_table_va:#x}   [{img.section_of(d.class_table_va)}]"
          f"   (from the `movzx` displacement)")
    print(f"  class bound          {d.bound:#x}  =>  {d.n_classes} classes 0x00..{d.bound:#04x}"
          f"   (from the `cmp` + unsigned `ja`)")
    print(f"  out-of-range arm     {d.ja_target:#x}   (from the `ja` rel32)")
    print(f"  class jump table     {d.jump_table_va:#x} .. "
          f"{d.jump_table_va + 4 * d.n_classes - 1:#x}"
          f"   stride 4, {d.n_classes} entries   [{img.section_of(d.jump_table_va)}]")
    print(f"  epilogue (derived)   {d.epilogue:#x}")
    print()

    print("== are the 29 targets distinct?  (the check nobody ran on THIS table) ==")
    dist = d.distinct_arms()
    print(f"  {d.n_classes} table entries, {len(dist)} distinct targets")
    for t, ks in sorted(d.shared_arms().items()):
        print(f"  SHARED {t:#x} <- classes {[f'{k:02X}' for k in ks]}")
    print()

    print("== containment ==")
    inside = sum(1 for t in d.arms if d.body_lo <= t < d.body_hi)
    print(f"  targets inside [{d.body_lo:#x},{d.body_hi:#x})   {inside} of {d.n_classes}")
    n_ref = sum(1 for t in d.arms if t == d.ja_target)
    refc = [k for k, t in enumerate(d.arms) if t == d.ja_target]
    print(f"  classes whose target IS the out-of-range arm  {n_ref}"
          f"  -> {[f'{k:02X}' for k in refc]}")
    print(f"  real (non-refusing) class arms               {d.n_classes - n_ref}"
          f" of {d.n_classes}")
    print()

    print("== the tables are exactly packed ==")
    end = d.jump_table_va + 4 * d.n_classes
    print(f"  body ends {d.body_hi:#x}; jump table starts {d.jump_table_va:#x}"
          f"  (gap {d.jump_table_va - d.body_hi} B)")
    tail = list(img.bytes(end, 16))
    print(f"  16 bytes past the jump table: {' '.join(f'{x:02x}' for x in tail)}")
    print(f"  next VA after the table: {end:#x}")
    print()

    print("== the class table's own extent, read from its bytes ==")
    ocs = d.opcode_classes(0x00, 0xFF)
    over = [o for o in range(0x100) if ocs[o] > d.bound]
    if not over:
        print("  every byte 0x00..0xff carries a legal class — the bytes bound nothing")
        return
    first = over[0]
    print(f"  first opcode whose class byte EXCEEDS the bound {d.bound:#x}: {first:#04x}"
          f"  (class {ocs[first]:#04x})")
    print(f"  so the table is self-evidently a table over 0x00..{first - 1:#04x}"
          f"  ({first} entries) and no further")
    print("  NOTE: that is an upper bound the BYTES force, not the consumer's")
    print("  domain.  A caller's own `cmp`/`ja` may be tighter; this script does")
    print("  not read any caller, and a published extent narrower than "
          f"0x00..{first - 1:#04x} is a CHOICE that needs its own citation.")


def cmd_classmap(img, d):
    from collections import defaultdict
    ocs = d.opcode_classes(0x00, 0xBF)
    per = defaultdict(list)
    for o, c in ocs.items():
        per[c].append(o)
    print("class  n   arm VA       opcodes")
    for c in sorted(per):
        ops = " ".join(f"{o:02x}" for o in per[c])
        arm = d.arms[c] if c < d.n_classes else None
        tag = "  <-- REFUSAL" if arm == d.ja_target else ""
        print(f"  {c:02X}  {len(per[c]):>3}  {arm:#010x}  {ops}{tag}")


def cmd_arms(img, d):
    for k in range(d.n_classes):
        arm = d.arms[k]
        tag = "   *** REFUSAL (== the out-of-range `ja` target)" \
            if arm == d.ja_target else ""
        print(f"=== class {k:02X}   arm {arm:#010x}{tag}")
        for va, kind, detail in d.walk_arm(k):
            if kind == "call":
                print(f"    {va:08x}  call   {detail:#x}")
            else:
                print(f"    {va:08x}  {kind:<5}  {detail}")
        print()


def cmd_prims(img, d):
    inv = callee_inventory(d)
    print("callee      classes that reach it (derived)                     size  shape")
    for t, ks in inv.items():
        # derive the callee's own extent: from its entry to the first `ret`
        ins = disasm(img.path, t, t + 0x200)
        size = None
        calls = []
        for va, _, mn, ops in ins:
            if mn == "call":
                c = target_of(ops)
                if c is not None:
                    calls.append(c)
            if mn.startswith("ret"):
                size = va + 1 - t
                break
        shape = f"{len(calls)} call(s)"
        if calls:
            shape += ": " + " ".join(f"{c:#x}" for c in dict.fromkeys(calls))
        print(f"{t:#010x}  {str([f'{k:02X}' for k in ks]):<48}  "
              f"{size if size is not None else '?':>4}  {shape}")


def cmd_grammar(img, d):
    """One derived line per class: the ordered call sequence + its guards."""
    inv = callee_inventory(d)
    print("class  arm VA       derived operand sequence")
    for k in range(d.n_classes):
        rows = d.walk_arm(k)
        parts = []
        for va, kind, detail in rows:
            if kind == "call":
                parts.append(f"call {detail:#x}")
            elif kind == "cond":
                parts.append(f"[{detail}]")
        if d.arms[k] == d.ja_target:
            parts = ["REFUSE (C1001)"]
        if not parts:
            parts = ["(nothing — straight to the epilogue)"]
        print(f"  {k:02X}   {d.arms[k]:#010x}  " + " ; ".join(parts))


def cmd_cross(img, d):
    """Control: the derived class table vs `dump_ilarms.py`'s independent read."""
    import importlib.util
    import os
    here = os.path.dirname(os.path.abspath(__file__))
    spec = importlib.util.spec_from_file_location(
        "dump_ilarms", os.path.join(here, "dump_ilarms.py"))
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    other = m.Image(sys.argv[1])
    disp = m.Dispatch(other)
    ok = True

    def chk(name, mine, theirs):
        nonlocal ok
        good = mine == theirs
        ok = ok and good
        print(f"  {'AGREE ' if good else 'DIFFER'}  {name}: mine={mine} theirs={theirs}")

    chk("class table VA", hex(d.class_table_va), hex(m.CLASS_TABLE_VA))
    # the 95 opcodes `dump_ilarms.py` says this dispatch HANDLES
    oa = disp.opcodes_of_arm()
    ref = set(disp.refusal_arms())
    handled = sorted(o for k, ops in oa.items() if k not in ref for o in ops)
    mine = {o: d.opcode_classes()[o] for o in handled}
    theirs = {o: other.u8(m.CLASS_TABLE_VA + o) for o in handled}
    chk(f"class byte over the {len(handled)} handled opcodes", mine == theirs, True)
    used = sorted(set(mine.values()))
    print(f"  distinct classes over the handled set: {len(used)} "
          f"-> {[f'{c:02X}' for c in used]}")
    over = [o for o, c in mine.items() if c > d.bound]
    chk("handled opcodes whose class exceeds the bound", len(over), 0)
    print(f"\n  {'ALL AGREE' if ok else 'DISAGREEMENT -- that is the finding'}")


def main():
    if len(sys.argv) < 3:
        raise SystemExit(__doc__)
    img = Image(sys.argv[1])
    d = Decoder(img)
    cmd = sys.argv[2]
    {"--verify": cmd_verify, "--arms": cmd_arms, "--grammar": cmd_grammar,
     "--prims": cmd_prims, "--classmap": cmd_classmap,
     "--cross": cmd_cross}[cmd](img, d)


if __name__ == "__main__":
    main()
