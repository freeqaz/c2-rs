#!/usr/bin/env python3
"""CONFIRMATION 2 — the workload-wide width check for opcode `0xBD`.

`w-divsplit`'s `shape.py` is the precedent and the standard: a width may enter
`chain_skip_form` only when a capture witnesses it AND the workload shows the
rival readings are excluded by the bytes. This is the second half for `0xBD`.

    bdwalk.py <il-root>            # il-root/NNNN/{TU,*.ex}

## The claim under test

    CALL := BD  <TYPE ret>  <flags:1 RAW byte>  <varint fn-type-id>

(`docs/IL_CALL_GRAMMAR.md` §2.1). Call it **W**. Three rival readings:

    R1  BD <TYPE> <varint>            — no flags byte at all
    R2  BD <TYPE>                     — no trailing fields
    R3  BD <TYPE> <varint> <varint>   — flags read as a varint, not a raw byte

## How a reading is judged, and why this control CAN go red

A `BD` is a postfix operator whose argument region runs to a `4C`
(`IL_CALL_GRAMMAR.md` §3). So the byte a reading lands on must OPEN AN OPERAND
TOKEN — an argument push, a nested operand, or the `4C` of a zero-argument
call. `LEGAL_OPEN` below is the operand-opcode set taken from
`chain_skip_form`'s own pinned table plus the tokens the call grammar names, and
nothing else; a landing byte outside it is a DESYNC and is counted as one.

The positive question the PREREG demands — *would this go red if the width were
wrong in the most likely way?* The most likely way to be wrong is dropping the
flags byte (R1). R1 is scored on the same corpus by the same predicate, and its
desync count is printed beside W's. If R1 does not come out overwhelmingly red
then this instrument is not discriminating and says so.

## Anchoring — how a site is known to be a real `BD` and not a data byte

`IL_CALL_GRAMMAR.md` §2.1 records that a raw `BD` byte-scan of one TU finds
8,638 hits of which 10 are false, inside data. So sites are ANCHORED by the
direct-call form `26 <token> BD` (§3's `direct` row) rather than by a bare byte
scan, and the unanchored population is counted separately rather than dropped
silently. Every count here carries its denominator.

Read-only. Writes nothing but its report on stdout. Consumes captured IL, which
is never committed.
"""

import collections
import glob
import os
import sys

AGGREGATE_CLASS = 0x6
TYPE_TAG_WIDE_BIT = 0x40
TYPE_WIDE_MARK_BIT = 0x80


def read_leb(b, i):
    """LEB128 — the TYPE id inside read_type only."""
    val = 0
    shift = 0
    while True:
        if i >= len(b):
            return None
        x = b[i]
        val |= (x & 0x7F) << shift
        i += 1
        if not (x & 0x80):
            return (val, i)
        shift += 7
        if shift > 28:
            return None


def read_varint(b, i):
    """Mirrors `readers.rs::read_varint`: a signed byte, or `80` + 4 LE bytes."""
    if i >= len(b):
        return None
    if b[i] == 0x80:
        if i + 4 >= len(b):
            return None
        v = int.from_bytes(bytes(b[i + 1:i + 5]), "little", signed=True)
        return (v, i + 5)
    v = b[i] - 256 if b[i] > 127 else b[i]
    return (v, i + 1)


def read_type(b, p):
    """Mirrors `readers.rs::read_type`. -> (tag, kind, id, width) or None."""
    if p >= len(b):
        return None
    tag = b[p]
    if not (tag & 0x80):
        return None
    i = p + 1
    if tag & TYPE_TAG_WIDE_BIT:
        if i >= len(b) or not (b[i] & TYPE_WIDE_MARK_BIT):
            return None
        i += 1
    if i >= len(b):
        return None
    kind = b[i]
    i += 1
    if (kind & 0x0F) == AGGREGATE_CLASS:
        size5 = ((tag & 0x01) << 4) | (kind >> 4)
        if size5 == 0:
            r = read_leb(b, i)
            if r is None or r[0] < 32:
                return None
            i = r[1]
    r = read_leb(b, i)
    if r is None:
        return None
    return (tag, kind, r[0], r[1] - p)


def read_token_var(b, p):
    """Mirrors `readers.rs::read_token_var`: 2 bytes, or 4 when bit 7 of the
    second is set."""
    if p + 1 >= len(b):
        return None
    if not (b[p + 1] & 0x80):
        return (((b[p] << 8) | b[p + 1]), 2)
    if p + 3 >= len(b):
        return None
    return (((b[p] << 24) | (b[p + 1] << 16) | (b[p + 2] << 8) | b[p + 3]), 4)


# THE TREE'S OWN OPERAND VOCABULARY, taken arm for arm from
# `crates/c2-il/src/func/body/shapes/control_flow.rs` — `operand()` for the
# expression layer and `step()` for the statement layer. A landing byte outside
# it is one that neither of the tree's two statement-level readers will accept,
# i.e. a DESYNC.
#
# **It is deliberately NOT hand-tuned to make W look good, and it was WIDENED
# once after the first run** — a first draft omitted the EH-state trailer family
# (`5C`/`5D`/`5E`), which `operand()` has had all along, and 11 perfectly good
# `BD` sites landing on a `5D` were scored as desyncs by the instrument rather
# than by the bytes. Taking the set from the tree instead of from a hand list is
# the fix, and the check that it did not defang the control is that R1's and
# R2's red counts are unmoved by the widening: R1 lands on `0x80` and R2 on
# `0x00`, and NEITHER is in `operand()` either.
LEGAL_OPEN = (
    # operand() — the expression layer
    {0xB9, 0x33, 0x26}
    | {0x02, 0x03, 0x04}
    | {0x05, 0x06, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x1A, 0x1B, 0x1C}
    | set(range(0x1F, 0x25))
    | {0x0F, 0x10, 0x11, 0x12, 0x13, 0x15, 0x16, 0x17, 0x18, 0x19, 0x35, 0x36}
    | {0x32, 0x41, 0x55}
    | {0x27, 0x30}
    | {0x5C, 0x5D, 0x5E}
    | {0x44, 0x28, 0x2C, 0x40, 0x43, 0x66, 0x67, 0x9A, 0x64, 0x99, 0x9B}
    | {0xBD, 0x4C}
    # step() — the statement layer
    | {0x53, 0x54, 0x29, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x4B, 0x4F}
)


def read_W(b, p):
    """BD <TYPE> <1 raw byte> <varint>."""
    t = read_type(b, p + 1)
    if t is None:
        return None
    q = p + 1 + t[3] + 1
    v = read_varint(b, q)
    return None if v is None else v[1]


def read_R1(b, p):
    """BD <TYPE> <varint> — the flags byte dropped."""
    t = read_type(b, p + 1)
    if t is None:
        return None
    v = read_varint(b, p + 1 + t[3])
    return None if v is None else v[1]


def read_R2(b, p):
    """BD <TYPE>."""
    t = read_type(b, p + 1)
    return None if t is None else p + 1 + t[3]


def read_R3(b, p):
    """BD <TYPE> <varint flags> <varint id> — flags as a varint."""
    t = read_type(b, p + 1)
    if t is None:
        return None
    a = read_varint(b, p + 1 + t[3])
    if a is None:
        return None
    v = read_varint(b, a[1])
    return None if v is None else v[1]


READINGS = [("W", read_W), ("R1", read_R1), ("R2", read_R2), ("R3", read_R3)]


def main(root):
    tus = 0
    sites = 0
    unanchored = 0
    land = {n: collections.Counter() for n, _ in READINGS}
    desync = {n: 0 for n, _ in READINGS}
    undecodable = {n: 0 for n, _ in READINGS}
    flags = collections.Counter()
    typew = collections.Counter()
    idw = collections.Counter()
    total_width = collections.Counter()
    per_tu_sites = collections.Counter()
    anchor_kind = collections.Counter()
    per_anchor_n = collections.Counter()
    per_anchor_desync = {}
    per_anchor_undec = {}
    undec_ctx = []
    bad_ctx = []
    agree_W_R3 = 0

    for d in sorted(glob.glob(os.path.join(root, "*"))):
        exs = glob.glob(os.path.join(d, "*.ex"))
        if not exs:
            continue
        tus += 1
        tu = open(os.path.join(d, "TU")).read().strip()
        b = open(exs[0], "rb").read()
        n = len(b)
        i = 0
        while True:
            j = b.find(b"\xbd", i)
            if j < 0:
                break
            i = j + 1
            # ANCHOR A: the direct-call form `26 <token> BD` (§3 `direct`).
            hit = None
            for tw in (2, 4):
                k = j - 1 - tw
                if k >= 0 and b[k] == 0x26:
                    r = read_token_var(b, k + 1)
                    if r is not None and r[1] == tw and k + 1 + tw == j:
                        hit = "direct"
                        break
            # ANCHOR B: the member-call bind `99 <TYPE> <varint> BD` (§3 `member`).
            # A SECOND, structurally different population: it exercises exactly
            # the sites the direct anchor cannot see, so a width that happened to
            # be right only after a `26` would show up here.
            if hit is None:
                for k in range(max(0, j - 12), j):
                    if b[k] != 0x99:
                        continue
                    t = read_type(b, k + 1)
                    if t is None:
                        continue
                    v = read_varint(b, k + 1 + t[3])
                    if v is not None and v[1] == j:
                        hit = "member"
                        break
            if hit is None:
                unanchored += 1
                continue
            sites += 1
            anchor_kind[hit] += 1
            per_anchor_desync.setdefault(hit, {n: 0 for n, _ in READINGS})
            per_anchor_undec.setdefault(hit, {n: 0 for n, _ in READINGS})
            per_anchor_n[hit] += 1
            for name, f in READINGS:
                q = f(b, j)
                if q is None or q >= n:
                    per_anchor_undec[hit][name] += 1
                elif b[q] not in LEGAL_OPEN:
                    per_anchor_desync[hit][name] += 1
                    if name == "W":
                        bad_ctx.append((hit, tu, j, b[max(0, j - 8):j + 14].hex(" ")))
            per_tu_sites[tu] += 1
            for name, f in READINGS:
                q = f(b, j)
                if q is None or q >= n:
                    undecodable[name] += 1
                    if name == "W":
                        undec_ctx.append((hit, tu, j, b[max(0, j - 8):j + 14].hex(" ")))
                    continue
                land[name][b[q]] += 1
                if b[q] not in LEGAL_OPEN:
                    desync[name] += 1
            t = read_type(b, j + 1)
            if t is not None:
                typew[t[3]] += 1
                fp = j + 1 + t[3]
                if fp < n:
                    flags[b[fp]] += 1
                    v = read_varint(b, fp + 1)
                    if v is not None:
                        idw[v[1] - (fp + 1)] += 1
                        total_width[v[1] - j] += 1
                qw, q3 = read_W(b, j), read_R3(b, j)
                if qw is not None and qw == q3:
                    agree_W_R3 += 1

    print("=" * 72)
    print("w-bd CONFIRMATION 2 — opcode 0xBD over the dc3 workload")
    print("=" * 72)
    print(f"TUs with a captured .ex           {tus}")
    print(f"ANCHORED `26 <tok> BD` sites      {sites}")
    print(f"unanchored raw 0xBD bytes         {unanchored}   (not judged)")
    print()
    print("-- the landing byte, per reading (the DENOMINATOR is `sites`) --")
    for name, _ in READINGS:
        top = ", ".join(f"0x{v:02X}x{c}" for v, c in land[name].most_common(6))
        print(
            f"  {name:3s} desync {desync[name]:7d} / {sites}"
            f"   undecodable {undecodable[name]:6d}   top landings: {top}"
        )
    print()
    print(f"-- the fields, under W --")
    print(f"  return-TYPE width   {dict(sorted(typew.items()))}")
    print(f"  flags byte value    {dict(sorted(flags.items()))}")
    print(f"  fn-type-id width    {dict(sorted(idw.items()))}")
    print(f"  total token width   {dict(sorted(total_width.items()))}")
    print()
    print(f"-- R3 (flags as a varint) agrees with W at {agree_W_R3} of {sites} sites --")
    print(f"   PREREG registered R3 as INDISTINGUISHABLE on this corpus.")
    print()
    print("-- by ANCHOR: two structurally different populations --")
    for k in sorted(per_anchor_n):
        row = "  ".join(f"{nm} {per_anchor_desync[k][nm]}" for nm, _ in READINGS)
        und = "  ".join(f"{nm} {per_anchor_undec[k][nm]}" for nm, _ in READINGS)
        print(f"  {k:7s} n={per_anchor_n[k]:8d}")
        print(f"          desync:      {row}")
        print(f"          undecodable: {und}")
    print()
    # THE FALSE-ANCHOR SCREEN. A `BD` byte that lies strictly INSIDE the LEB
    # payload of a neighbouring TYPE is not an opcode at all — the member anchor
    # is a heuristic and this says how often it fires on one. Non-circular: it
    # asks only whether some type token starting BEFORE j spans past j, which is
    # a property of the bytes and not of any reading of BD.
    TOKEN_OPS = {0x26, 0x29, 0x38, 0x39, 0x3A, 0xB9}
    in_type = in_tok = in_desc = 0
    residue = []
    per_anchor_anom = collections.Counter()
    for tag, (a, tu, off, hexs) in (
        [("UNDEC", x) for x in undec_ctx] + [("DESYNC", x) for x in bad_ctx]
    ):
        per_anchor_anom[a] += 1
        w = bytes(int(x, 16) for x in hexs.split())
        j = min(8, off)
        hit = None
        for k in range(max(0, j - 8), j):
            t = read_type(w, k)
            if t is not None and k + t[3] > j:
                hit = "type"
                break
        if hit is None:
            for k in range(max(0, j - 6), j):
                if w[k] not in TOKEN_OPS:
                    continue
                r = read_token_var(w, k + 1)
                if r is not None and k + 1 + r[1] > j:
                    hit = "token"
                    break
        if hit is None:
            # `66 <arity> <arity tokens>` — the class-pair descriptor. Its
            # tokens are ordinary LEB-width tokens and a 0xBD can land inside
            # one exactly as it can inside a TYPE.
            for k in range(max(0, j - 8), j):
                if w[k] != 0x66 or k + 1 >= len(w):
                    continue
                q, ok = k + 2, True
                for _ in range(w[k + 1]):
                    r = read_token_var(w, q)
                    if r is None:
                        ok = False
                        break
                    q += r[1]
                if ok and q > j:
                    hit = "descriptor"
                    break
        if hit == "type":
            in_type += 1
        elif hit == "token":
            in_tok += 1
        elif hit == "descriptor":
            in_desc += 1
        else:
            residue.append((tag, a, tu, off, hexs))
    tot = len(undec_ctx) + len(bad_ctx)
    print("-- FALSE-ANCHOR SCREEN --")
    print(f"  anomalous sites (undecodable + desync)   {tot} of {sites}")
    print(f"    by anchor: " + "  ".join(f"{k} {v}" for k, v in sorted(per_anchor_anom.items())))
    print(f"  the 0xBD lies INSIDE a neighbouring TYPE's LEB payload   {in_type}")
    print(f"  the 0xBD lies INSIDE a neighbouring 4-byte TOKEN         {in_tok}")
    print(f"  the 0xBD lies INSIDE a `66 <n>` class descriptor's tokens {in_desc}")
    print(f"  UNEXPLAINED residue                                     {len(residue)}")
    print("  Neither is an opcode position, so neither is a CALL site. The")
    print("  screen is non-circular: it asks only whether a token starting")
    print("  BEFORE j spans past j, which is a property of the bytes and not")
    print("  of any reading of BD. The residue is printed in full:")
    for r in residue:
        print(f"    RESIDUE {r[0]:6s} {r[1]:6s} {r[2]} @ {r[3]}: {r[4]}")
    print()
    print(f"-- every site W does not read cleanly, in full ({len(undec_ctx)} undecodable + {len(bad_ctx)} desync) --")
    for tag, (a, tu, off, hexs) in ([("UNDEC", x) for x in undec_ctx] + [("DESYNC", x) for x in bad_ctx]):
        print(f"  {tag:6s} {a:6s} {tu} @ {off}: {hexs}")
    print()
    print(f"-- top TUs by site count --")
    for tu, c in per_tu_sites.most_common(5):
        print(f"  {c:6d}  {tu}")


if __name__ == "__main__":
    main(sys.argv[1])
