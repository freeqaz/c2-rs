#!/usr/bin/env python3
"""LIMB 2 — cross c2's per-class operand grammar against the port's readers.

Lane `w-opclass`, board #3585-#3590.  Whitebox tooling (outside the std-only
`crates/` workspace, per CLAUDE.md).

    python3 docs/whitebox/scripts/cross_opclass_port.py <c2.dll>            # the table
    python3 docs/whitebox/scripts/cross_opclass_port.py <c2.dll> --counts   # just the counts
    python3 docs/whitebox/scripts/cross_opclass_port.py <c2.dll> --controls # the controls only

`WB_ILARMS_MAP.md` §2.3 defines `NARROW` with two limbs.  Limb 1 (gate) it
closed: `NARROW(gate)` is 0 of 95.  Limb 2 — *"consumes fewer operand fields
than the arm's class implies"* — it left open on 65 of its 68 `MATCHED*` rows
because it did not read the class arms.  This script closes limb 2, mechanically,
so the count is re-derivable rather than asserted in prose.

## The two sides, and which is `[R]` and which is `[src]`

* **c2's side is `[R]` and is DERIVED AT RUN TIME**, by importing
  `dump_opclass.py` and walking the 29 class arms out of the pinned image.  No
  class grammar is written down in this file.
* **The port's side is `[src]`** — a table below, one row per opcode, each with
  the `crates/…:LINE` it was transcribed from.  It is hand-transcribed and that
  is a real hazard, so it carries two controls (`--controls`): the opcode set
  must equal `labels/ilarms_portmap.txt`'s 68 `MATCHED*` rows exactly, and every
  cited line must still contain the opcode's own hex literal.

## The verdict vocabulary is `_2026-08-26-w-opclass-prereg.md` §1, fixed before
## any measurement, four values and no fifth

    MATCHED         same field sequence, same width function, on every input
                    the class accepts
    NARROW(fields)  some class-legal input on which the port advances LESS or
                    refuses
    WIDE(fields)    some class-legal input on which the port advances MORE
    UNRESOLVED      the class arms do not decide it; the residual read is named

A row that is both narrow and wide reports WIDE(fields) — the unsafe direction
— and the reason strings carry both.
"""

import importlib.util
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(os.path.dirname(HERE)))
CF = "crates/c2-il/src/func/body/shapes/control_flow.rs"
CODEC = "crates/c2-il/src/codec.rs"
PORTMAP = os.path.join(HERE, "..", "labels", "ilarms_portmap.txt")

# ---------------------------------------------------------------------------
# c2's stream primitives, keyed by the callee VA the arm walk yields.
#
# The WIDTH SETS are derived by decoding each primitive's body (see
# `WB_OPCLASS_FINDINGS.md` §3, which prints the `inc` counts per path).  They are
# written here as a *decision table*, not as a re-read, because the comparison
# below needs them as sets; `--controls` asserts every VA named here is one the
# arm walk actually produced, so a stale VA cannot sit unnoticed.
C2_PRIM = {
    0x10C1F8FC: ("GetByte", "{1}"),
    0x10C1F90A: ("skip", "{1,2,3,…} LEB continuation run, unbounded"),
    0x10C1F91B: ("varU", "{2,4} — 2 LE bytes, +2 more iff byte1 bit7"),
    0x10C1F9A6: ("i16c", "{1,3} — signed byte, or 0x80 + 2 LE"),
    0x10C1F9E9: ("i32c", "{1,5} — signed byte, or 0x80 + 4 LE"),
    0x10C1FAE7: ("i64c", "{1,9} — signed byte, or 0x80 + 8 LE"),
    0x10C1FC5B: ("str", "bounded NUL-terminated run"),
    0x10C1FEEF: ("dec10", "{10} — 8 raw bytes then 2 raw bytes"),
    0x10B3D546: ("TYPE", "word{1,2,3} + [i32c iff aggregate escape] + [skip iff global]"),
    0x10B99977: ("sym()", "{0} — a TU symbol-table lookup, reads no stream byte"),
    0x10B9761E: ("sub4F", "format-string interpreter, ref/P_SUB4F.md"),
    0x10C2022A: ("alloc", "{0} — allocator, reads no stream byte"),
    0x10C1EEB6: ("C1001", "{0} — noreturn"),
    0x10B33526: ("C1001", "{0} — noreturn"),
    0x10BE6CAF: ("fp-cvt", "{0}"),
    0x10BE6D39: ("fp-cvt", "{0}"),
    0x10BE6D4F: ("fp-cvt", "{0}"),
    0x10BE6D76: ("fp-cvt", "{0}"),
    0x10BE6D8C: ("fp-cvt", "{0}"),
    0x10C1F0AC: ("diag", "{0}"),
}

# Callees that consume no stream byte — dropped from the field sequence.
C2_ZERO = {0x10B99977, 0x10C2022A, 0x10BE6CAF, 0x10BE6D39, 0x10BE6D4F,
           0x10BE6D76, 0x10BE6D8C, 0x10C1F0AC}

# ---------------------------------------------------------------------------
# The PORT side: opcode -> (file, line, field sequence, note)
#
# Field tokens: 'op'  the opcode byte itself
#               'ty'  Scan::ty     -> readers::read_type
#               'tok' Scan::tok    -> readers::read_token_var
#               'vint'Scan::vint   -> readers::read_varint
#               'b'   one raw byte
#               'lit' lit_payload
#               'desc'eat_class_descriptor
#               'fix00' the hard-coded `00 00`
#               'esc43' the `43 <sub>` escape arm
#               'end4F' the statement-layer end (not an operand read at all)
PORT = {
    0x02: (CF, 848, ["op"], ""), 0x03: (CF, 848, ["op"], ""),
    0x04: (CF, 848, ["op"], ""),
    0x05: (CF, 882, ["op"], ""), 0x06: (CF, 882, ["op"], ""),
    0x09: (CF, 882, ["op"], ""), 0x0A: (CF, 882, ["op"], ""),
    0x0B: (CF, 882, ["op"], ""), 0x0C: (CF, 882, ["op"], ""),
    0x0D: (CF, 882, ["op"], ""), 0x0E: (CF, 882, ["op"], ""),
    0x1A: (CF, 882, ["op"], ""), 0x1B: (CF, 882, ["op"], ""),
    0x1C: (CF, 882, ["op"], ""),
    0x1F: (CF, 882, ["op"], ""), 0x20: (CF, 882, ["op"], ""),
    0x21: (CF, 882, ["op"], ""), 0x22: (CF, 882, ["op"], ""),
    0x23: (CF, 882, ["op"], ""), 0x24: (CF, 882, ["op"], ""),
    0x0F: (CF, 897, ["op", "ty"], ""), 0x10: (CF, 897, ["op", "ty"], ""),
    0x11: (CF, 897, ["op", "ty"], ""), 0x12: (CF, 897, ["op", "ty"], ""),
    0x13: (CF, 897, ["op", "ty"], ""), 0x15: (CF, 897, ["op", "ty"], ""),
    0x16: (CF, 897, ["op", "ty"], ""), 0x17: (CF, 897, ["op", "ty"], ""),
    0x18: (CF, 897, ["op", "ty"], ""), 0x19: (CF, 897, ["op", "ty"], ""),
    0x35: (CF, 897, ["op", "ty"], ""), 0x36: (CF, 897, ["op", "ty"], ""),
    0x26: (CF, 842, ["op", "tok"], ""),
    0x27: (CF, 938, ["op", "ty"], ""),
    0x28: (CF, 1033, ["op", "fix00"], "hard-coded `28 00 00`"),
    0x29: (CF, 751, ["op", "tok"], ""),
    0x2C: (CF, 1044, ["op", "ty", "vint"], ""),
    0x30: (CF, 943, ["op", "ty"], ""),
    0x32: (CF, 909, ["op", "ty"], ""),
    0x33: (CF, 829, ["op", "ty", "lit"], ""),
    0x38: (CF, 757, ["op", "tok"], ""), 0x39: (CF, 757, ["op", "tok"], ""),
    0x3A: (CF, 763, ["op", "tok"], ""),
    0x3B: (CF, 769, ["op", "tok"], ""),
    0x3C: (CF, 774, ["op", "ty", "tok"], ""),
    0x3D: (CF, 780, ["op", "tok"], ""),
    0x40: (CF, 1054, ["op", "ty"], ""),
    0x41: (CF, 909, ["op", "ty"], ""),
    0x43: (CF, 1066, ["op", "esc43"], "reads 0x43 as an ESCAPE"),
    0x44: (CF, 1025, ["op"], ""),
    0x46: (CODEC, 1343, ["op"], "codec only — no control_flow site"),
    0x4B: (CF, 789, ["op"], ""),
    0x4C: (CF, 1205, ["op"], ""),
    0x4F: (CF, 795, ["end4F"], "statement layer: ends the walk"),
    0x53: (CF, 723, ["op"], ""),
    0x54: (CF, 730, ["op", "b"], "+2 fixed, the byte compared to the depth"),
    0x55: (CF, 909, ["op", "ty"], ""),
    0x5C: (CF, 969, ["op", "ty", "vint"], ""),
    0x5D: (CF, 984, ["op", "vint", "vint"], ""),
    0x5E: (CF, 984, ["op", "vint", "vint"], ""),
    0x64: (CF, 1154, ["op", "ty"], ""),
    0x66: (CF, 1088, ["desc"], "eat_class_descriptor: `66 <byte n> n×LEB`"),
    0x67: (CF, 1110, ["op", "vint", "tok"], ""),
    0x99: (CF, 1164, ["op", "ty", "vint"], ""),
    0x9A: (CF, 1129, ["op", "ty"], ""),
    0x9B: (CF, 1170, ["op", "ty", "tok"], ""),
    0xB9: (CF, 818, ["op", "tok", "ty"], ""),
    0xBD: (CF, 1182, ["op", "ty", "b", "vint"], "the `b` is the calling convention"),
}

# How each port field compares to the c2 primitive it stands opposite.
#   '='  same width function on every class-legal input
#   '<'  port advances less, or refuses  -> NARROW(fields)
#   '>'  port advances more              -> WIDE(fields)
#   '?'  undecidable from the class arms -> UNRESOLVED
# Keyed (port token, c2 primitive name).  A pair absent from here is '?' with
# "not compared" as the reason, so a silent default cannot pass for a decision.
PAIR = {
    ("tok", "varU"): ("=", "read_token_var is 2 bytes + 2 more iff byte1 bit7 — "
                           "identical to varU (0x10c1f91b)"),
    ("vint", "i32c"): ("=", "read_varint is a signed byte, or 0x80 + 4 LE — "
                            "identical to i32c (0x10c1f9e9)"),
    ("b", "i32c"): ("<", "the port reads ONE fixed byte where c2 reads an i32c: "
                         "at payload 0x80 c2 takes 5 and the port takes 1"),
    ("b", "i16c"): ("<", "one fixed byte where c2 reads an i16c (0x80 -> 3)"),
    ("b", "GetByte"): ("=", "one raw byte on both sides"),
    ("vint", "GetByte"): (">", "read_varint takes FIVE bytes on the marker 0x80 "
                               "where c2's class arm reads ONE raw byte "
                               "(0x10b3d694) — the port over-reads by 4"),
    ("vint", "i16c"): ("?", "read_varint's escape is 0x80 + 4 LE and i16c's is "
                            "0x80 + 2 LE; they agree only below the escape"),
    ("fix00", "varU"): ("<", "the port hard-codes the literal `00 00` and refuses "
                             "everything else; `00 00` IS a varU of 0, so the port "
                             "accepts exactly one of the varU's values"),
    ("esc43", "NONE"): (">", "class 00 is payload-free: c2 advances 1. The port "
                             "advances 4 on sub-byte 0x42 and 2 on 0x37"),
    ("end4F", "i16c"): ("<", "the port has no general 0x4F reader: `step` ends the "
                             "statement list and only four fixed `4F` shapes are "
                             "recognised in codec.rs, against c2's i16c + a "
                             "format-string interpreter over 64 field codes"),
    ("desc", "i32c"): ("<", "the port reads the arity as ONE byte where c2 reads an "
                            "i32c; at 0x80 c2 takes 5 and the port takes 1 and then "
                            "reads four payload bytes as LEB tokens"),
    ("lit", "dec10"): ("?", "both sides read TEN raw bytes for a real literal, so "
                            "the WIDTH agrees; the DISCRIMINATOR does not — the port "
                            "tests `kind & 0x0f == 0x0a` on the raw kind byte and c2 "
                            "tests `node[+4] & 0xf000 == 0x5000` on the LOWERED word"),
    ("lit", "i64c"): ("?", "the port takes 1+8 when the raw tag is 0x88 and 1+4 "
                           "otherwise; c2's i64c is 1 or 9. Same intent, two "
                           "different fields — needs the lowering"),
    ("lit", "i32c"): ("?", "the port switches on the RAW tag byte (`tag == 0x88`) "
                           "and c2 on the LOWERED type word `node[+4]`; equivalence "
                           "needs FUN_10b3d40a (0x10b3d40a) and FUN_10c1fe9d "
                           "(0x10c1fe9d), which this lane did not read"),
}

# The TYPE comparison is its own row because it is ONE primitive standing behind
# many opcodes, and because its divergences are per-input rather than per-opcode.
TY_VERDICT = ("<", [
    "c2's TYPE word has a ONE-BYTE short form (b1 < 0x80, 0x10c1fe98); "
    "readers::read_type returns None on `tag & 0x80 == 0` and the walk blocks",
    "c2's three-byte form (b1 & 0x40, 0x10c1fe63) masks b2 with 0x7f and does not "
    "test its bit 7; read_type REFUSES a wide type whose second byte has bit 7 clear",
    "c2 reads the aggregate out-of-line size as a plain i32c (0x10b3d59f); "
    "read_type refuses any value < 32",
    "c2's trailing skip (0x10b3d5b4) is an UNBOUNDED continuation run; read_type's "
    "id loop refuses at shift > 28, i.e. past 5 bytes",
])

# Environment reads the port has BAKED rather than modelled. Not per-input, so
# they are reported beside the verdicts and folded into none of them.
BAKED = {
    "0x10c472e8+0xcac": ("read at 0x10b3d5ab (the TYPE trailing skip) and again at "
                         "0x10b3d919 (class 1C's trailing i32c). The port reads BOTH "
                         "unconditionally, so it has hard-coded this global as "
                         "NON-ZERO in two places at once — and the two agreeing is "
                         "what makes the assumption self-consistent rather than "
                         "merely unnoticed"),
    "0x10c67fc0": ("read at 0x10b3d64d — when it is ZERO, opcode 0x42 alone takes "
                   "NO operand and class 02's whole body is skipped. The port has no "
                   "reader for 0x42 at all, so it has not baked a value; the arm that "
                   "consumes 0x42's bytes is the 0x43 escape, which assumes the "
                   "NON-zero branch"),
    "0x10c2edc4": ("read at 0x10b3d8ce — class 19 (opcode 0xBD) reads an i32c when it "
                   "is set and an i16c when it is clear. The port reads Scan::vint "
                   "(== i32c) unconditionally, baking NON-ZERO"),
}


def load_opclass(path):
    spec = importlib.util.spec_from_file_location(
        "dump_opclass", os.path.join(HERE, "dump_opclass.py"))
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    img = m.Image(path)
    return m, img, m.Decoder(img)


# ---------------------------------------------------------------------------
# The FIVE guarded classes.  The arm walk reports a `cond` row for exactly these
# and for no others (asserted in `--controls`), so the list below is a reading of
# a branch this script located, not a list of classes somebody remembered.
#
# `kind` separates the two sorts of guard, and the separation is the point:
#   'env'   — a global.  The port cannot see it, so it has BAKED one branch.  The
#             verdict is taken on the baked branch and the other is reported in
#             the BAKED block, folded into no count.
#   'input' — a predicate on the record being decoded.  Every branch is reachable
#             from a legal stream, so the verdict folds over ALL of them.
C2_ALT = {
    0x02: ("env", 0, [
        ("DAT_10c67fc0 != 0, or the opcode is not 0x42  [0x10b3d64d]", ["varU"]),
        ("DAT_10c67fc0 == 0 AND the opcode IS 0x42      [0x10b3d656]", []),
    ]),
    0x06: ("input", 0, [
        ("node[+4] & 0xf000 == 0x5000 — a real          [0x10b3d6d3]",
         ["TYPE", "dec10"]),
        ("node[+4] & 0x0fff == 8 — an 8-byte scalar     [0x10b3d794]",
         ["TYPE", "i64c"]),
        ("otherwise                                     [0x10b3d798]",
         ["TYPE", "i32c"]),
    ]),
    0x19: ("env", 0, [
        ("DAT_10c2edc4 != 0                             [0x10b3d8ce]",
         ["TYPE", "GetByte", "i32c"]),
        ("DAT_10c2edc4 == 0                             [0x10b3d8ce]",
         ["TYPE", "GetByte", "i16c"]),
    ]),
    0x1A: ("input", 0, [
        ("i32c gives n, then n x skip                   [0x10b3d8f6]",
         ["i32c", "skip"]),
    ]),
    0x1C: ("env", 0, [
        ("[DAT_10c472e8+0xcac] != 0                     [0x10b3d919]",
         ["TYPE", "i32c"]),
        ("[DAT_10c472e8+0xcac] == 0                     [0x10b3d920]", ["TYPE"]),
    ]),
}


def c2_walk_fields(d, cls):
    """Every stream-consuming callee the arm walk produced, in walk order."""
    seq, guards = [], []
    for va, kind, detail in d.walk_arm(cls):
        if kind == "call":
            if detail in C2_ZERO:
                continue
            seq.append(C2_PRIM.get(detail, (f"?{detail:#x}", "unknown"))[0])
        elif kind == "cond":
            guards.append(f"{va:#x} {detail}")
    return seq, guards


def c2_fields(d, cls):
    """(alternatives, guards) — [(guard text, [field])], one entry if unguarded."""
    seq, guards = c2_walk_fields(d, cls)
    if cls in C2_ALT:
        _, _, alts = C2_ALT[cls]
        return alts, guards
    return [("", seq)], guards


def portmap_matched():
    ops = []
    txt = open(os.path.join(HERE, "..", "labels", "ilarms_portmap.txt")).read()
    sec = txt.split("per-arm opcode detail")[1]
    for ln in sec.splitlines():
        m = re.match(r"\s+0x([0-9a-f]{2}) MATCHED\*", ln)
        if m:
            ops.append(int(m.group(1), 16))
    return sorted(ops)


def controls(d):
    """Every control this cross depends on, run before any verdict is printed."""
    ok = True

    def chk(name, cond, detail=""):
        nonlocal ok
        ok = ok and cond
        print(f"  {'PASS' if cond else 'FAIL'}  {name}{('  ' + detail) if detail else ''}")

    want = portmap_matched()
    chk(f"the port table covers exactly the {len(want)} MATCHED* opcodes",
        sorted(PORT) == want,
        f"mine={len(PORT)} portmap={len(want)} "
        f"missing={[hex(o) for o in want if o not in PORT]} "
        f"extra={[hex(o) for o in PORT if o not in want]}")

    # The cited `control_flow.rs` line is not trusted: it is RE-DERIVED by
    # locating the `match` arm whose pattern list names the opcode, inside
    # `fn step` / `fn operand`, and the transcription must equal the derivation.
    # A hand-typed line number that drifts is exactly `#3367`'s failure mode.
    src = open(os.path.join(REPO, CF)).read().splitlines()
    heads = {}
    pat = re.compile(r"^\s{8}((?:0x[0-9A-Fa-f]{2}\s*\|\s*)*0x[0-9A-Fa-f]{2})\s*(=>|$)")
    for i, ln in enumerate(src):
        m = pat.match(ln.rstrip())
        if not m:
            continue
        ops = [int(x, 16) for x in re.findall(r"0x([0-9A-Fa-f]{2})", m.group(1))]
        tail = ln.rstrip()
        j = i
        while not tail.endswith("{") and not tail.endswith(",") and \
                "=>" not in tail and j + 1 < len(src):
            j += 1
            tail = src[j].rstrip()
            ops += [int(x, 16) for x in re.findall(r"0x([0-9A-Fa-f]{2})", tail)]
        for o in ops:
            heads.setdefault(o, i + 1)
    bad = []
    for op, (f, line, _, _) in sorted(PORT.items()):
        if f != CF:
            continue
        got = heads.get(op)
        if got != line:
            bad.append((hex(op), f"cited {line}", f"derived {got}"))
    chk("every cited control_flow.rs line equals the line DERIVED from the "
        "match arm that names the opcode", not bad,
        f"{len(bad)} drifted: {bad[:6]}")

    seen = set()
    for k in range(d.n_classes):
        for _, kind, det in d.walk_arm(k):
            if kind == "call":
                seen.add(det)
    unknown = sorted(t for t in seen if t not in C2_PRIM)
    chk("every callee the arm walk produced is named in C2_PRIM", not unknown,
        f"unnamed: {[hex(t) for t in unknown]}")
    stale = sorted(t for t in C2_PRIM if t not in seen)
    chk("every VA named in C2_PRIM is one the arm walk produced", not stale,
        f"stale: {[hex(t) for t in stale]}")
    return ok


def verdict_over(op, cls, alts, pseq):
    """Fold the per-alternative verdicts of one opcode.

    'env'-guarded classes are judged on the BAKED branch only (index 1 of the
    `C2_ALT` row); 'input'-guarded classes fold over every branch, because every
    branch is reachable from a legal stream.
    """
    kind, primary = ("plain", 0)
    if cls in C2_ALT:
        kind, primary, _ = C2_ALT[cls]
    if kind == "env":
        use = [alts[primary]]
    else:
        use = alts
    marks, reasons = [], []
    for guard, seq in use:
        v, rs = verdict(op, cls, seq, pseq)
        marks.append({"MATCHED": "=", "NARROW(fields)": "<",
                      "WIDE(fields)": ">", "UNRESOLVED": "?"}[v])
        pre = f"[{guard.split('[')[0].strip()}] " if guard else ""
        reasons += [(mk, head, pre + why) for mk, head, why in rs]
    return fold(marks), reasons


def verdict(op, cls, cseq, pseq):
    """(verdict, [reasons]) for one opcode against ONE class alternative."""
    reasons = []
    marks = []
    pfields = [t for t in pseq if t != "op"]
    # c2 fields, in order; the opcode byte is implicit on both sides
    cfields = list(cseq)
    if not cfields and pfields:
        # class 00: payload-free. Anything the port reads is over-reading.
        for t in pfields:
            m, why = PAIR.get((t, "NONE"), ("?", "not compared"))
            marks.append(m)
            reasons.append((m, f"{t} vs (nothing)", why))
        return fold(marks), reasons
    if len(cfields) != len(pfields):
        # a field-count difference is decided by direction, not by pairing
        if len(pfields) < len(cfields):
            missing = cfields[len(pfields):]
            for t, c in zip(pfields, cfields):
                m, why = compare(t, c)
                marks.append(m)
                reasons.append((m, f"{t} vs {c}", why))
            marks.append("<")
            reasons.append(("<", "field count",
                            f"the port reads {len(pfields)} field(s) where the class "
                            f"reads {len(cfields)} — missing {missing}"))
        else:
            extra = pfields[len(cfields):]
            for t, c in zip(pfields, cfields):
                m, why = compare(t, c)
                marks.append(m)
                reasons.append((m, f"{t} vs {c}", why))
            marks.append(">")
            reasons.append((">", "field count",
                            f"the port reads {len(pfields)} field(s) where the class "
                            f"reads {len(cfields)} — extra {extra}"))
        return fold(marks), reasons
    for t, c in zip(pfields, cfields):
        m, why = compare(t, c)
        marks.append(m)
        reasons.append((m, f"{t} vs {c}", why))
    return fold(marks), reasons


def compare(port_tok, c2_prim):
    if port_tok == "ty":
        if c2_prim != "TYPE":
            return "?", f"the port reads a TYPE where the class reads {c2_prim}"
        return TY_VERDICT[0], "readers::read_type vs c2's TYPE — " + \
            "; ".join(TY_VERDICT[1])
    return PAIR.get((port_tok, c2_prim), ("?", "not compared"))


def fold(marks):
    if "?" in marks:
        return "UNRESOLVED"
    if ">" in marks:
        return "WIDE(fields)"
    if "<" in marks:
        return "NARROW(fields)"
    return "MATCHED"


def main():
    if len(sys.argv) < 2:
        raise SystemExit(__doc__)
    m, img, d = load_opclass(sys.argv[1])
    mode = sys.argv[2] if len(sys.argv) > 2 else ""

    print("== controls (run before any verdict) ==")
    ok = controls(d)
    print(f"  => {'ALL CONTROLS PASS' if ok else 'A CONTROL FAILED — the table below '
                                                 'is not trustworthy'}")
    print()
    if mode == "--controls":
        return

    classes = d.opcode_classes()
    rows = []
    for op in sorted(PORT):
        cls = classes[op]
        alts, guards = c2_fields(d, cls)
        f, line, pseq, note = PORT[op]
        v, reasons = verdict_over(op, cls, alts, pseq)
        rows.append((op, cls, alts, guards, pseq, v, reasons, f, line, note))

    if mode != "--counts":
        print("== the cross, ordered by OPCODE (not by mass — prereg §7 item 6) ==")
        print(f"{'op':>4}  {'cls':>3}  {'c2 fields [R]':<34}  "
              f"{'port fields [src]':<22}  verdict")
        for op, cls, alts, guards, pseq, v, reasons, f, line, note in rows:
            for i, (guard, seq) in enumerate(alts):
                cs = " ".join(seq) if seq else "(payload-free)"
                ps = " ".join(t for t in pseq if t != "op") or "(payload-free)"
                if i == 0:
                    print(f"  {op:02x}  {cls:02X}   {cs:<34}  {ps:<22}  {v}")
                else:
                    print(f"      {'':2}   {cs:<34}  {'':22}  "
                          f"   ^ alt: {guard}")
            if len(alts) > 1:
                kind = C2_ALT[cls][0]
                print(f"      {'':2}   {'':34}  {'':22}     "
                      f"({kind}-guarded, {len(alts)} branches)")
        print()

    from collections import Counter
    c = Counter(r[5] for r in rows)
    n = len(rows)
    print(f"== limb 2, over the {n} `MATCHED*` opcodes `WB_ILARMS_MAP.md` §4 counts ==")
    for k in ("MATCHED", "NARROW(fields)", "WIDE(fields)", "UNRESOLVED"):
        print(f"  {k:<16} {c[k]:>3} of {n}")
    changed = n - c["MATCHED"]
    print(f"  -> {changed} of {n} rows change verdict away from MATCHED")
    prior = [0x28, 0x2C, 0x54]
    sub = [r for r in rows if r[0] not in prior]
    cs = Counter(r[5] for r in sub)
    print(f"\n== the 65 `w-ilarms` called genuinely unchecked "
          f"(the 68 less 0x28, 0x2c, 0x54) ==")
    for k in ("MATCHED", "NARROW(fields)", "WIDE(fields)", "UNRESOLVED"):
        print(f"  {k:<16} {cs[k]:>3} of {len(sub)}")
    print(f"  -> {len(sub) - cs['MATCHED']} of {len(sub)} change verdict")

    print("\n== the strict FIELD-COUNT reading (secondary denominator) ==")
    same = sum(1 for r in rows
               if all(len([t for t in r[4] if t != "op"]) == len(seq)
                      for _, seq in r[2]))
    print(f"  port field COUNT == class field COUNT on {same} of {n}")
    print("  (the primary reading is the width-function one — prereg §1, and it is")
    print("   `w-ilarms`'s own precedent at 0x28, where the counts are equal and the")
    print("   verdict is NARROW(fields) anyway)")

    print("\n== root causes, so the count is not read as N independent facts ==")
    causes = Counter()
    for op, cls, alts, guards, pseq, v, reasons, f, line, note in rows:
        if v == "MATCHED":
            continue
        want = {"WIDE(fields)": ">", "NARROW(fields)": "<",
                "UNRESOLVED": "?"}[v]
        head = next((h for mk, h, _ in reasons if mk == want), "?")
        causes[head] += 1
    for k, v2 in sorted(causes.items(), key=lambda x: (-x[1], x[0])):
        print(f"  {v2:>3}  {k}")
    print(f"  {len(causes)} distinct root cause(s)")

    print("\n== environment globals the port has BAKED (not folded into any verdict) ==")
    for g, why in BAKED.items():
        print(f"  {g}\n      {why}")

    print("\n== the UNRESOLVED rows, with the read that would close each ==")
    for op, cls, alts, guards, pseq, v, reasons, f, line, note in rows:
        if v == "UNRESOLVED":
            print(f"  {op:02x} (class {cls:02X}):")
            for mk, h, why in reasons:
                if mk == "?":
                    print(f"      {h}: {why}")


if __name__ == "__main__":
    main()
