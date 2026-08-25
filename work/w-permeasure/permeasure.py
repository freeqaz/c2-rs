#!/usr/bin/env python3
"""THE PERMUTER PRE-MEASUREMENT — is a decomp near-miss shaped like a port miss?

    scripts/permeasure.py control  <port_fndiff.jsonl>
    scripts/permeasure.py measure  <dc3-decomp-root> <port_fndiff.jsonl> [--json OUT]

Lane `w-permeasure`, 2026-08-25. Board #3534-#3538.
`docs/rungs/_2026-08-25-w-permeasure-prereg.md` is the registered plan.

WHY THIS FILE EXISTS AT ALL, given that the instrument already ships
---------------------------------------------------------------------
`crates/c2-harness/src/gap/fndiff.rs` is the lens. It is 1,369 lines, it
shipped 2026-08-06, it prints `DIFF STRUCTURE` on every scan, and board #3369
is a whole row about a planning doc that proposed building it again. Nothing
here rebuilds it as an instrument for the port.

But it can only be pointed at `(port body, c2 body)` pairs the gap scan
produces, and this lane's question is about a DIFFERENT population — the
`(hand-written decomp obj, original target obj)` pairs in `../dc3-decomp`.
Pointing the Rust lens at two arbitrary objs needs a `crates/` edit, and this
lane writes zero `crates/` bytes. So the lens is RE-EXPRESSED here, and

    ** THE RE-EXPRESSION IS GRADED BEFORE IT IS USED. **

`fndiff.rs::to_json` writes `port_hex` and `ref_hex` — the whole word list of
both bodies — beside its OWN `first`/`equal`/`sub`/`ins`/`del`/`same_multiset`/
`classes`/`csig`/`sig`. `control` feeds this file's alignment ONLY those two
arrays and demands it re-derive every one of those fields exactly, row for row.
`measure` REFUSES TO RUN until `control` passes at the registered threshold.
That is board #2064's rule turned on its author: a rescoring harness that
cannot reproduce the published scores is measuring something else.

Tooling, like `scripts/fndiff_report.py` and `scripts/plot_perf.py` — outside
the std-only Rust workspace, nothing in `crates/` depends on it, and it
LICENSES NO EMIT. It reaches no numerator and appears in no accept/refuse path.
"""

import json
import os
import struct
import sys
from collections import Counter, defaultdict

# ===========================================================================
# THE LENS — a line-for-line re-expression of crates/c2-harness/src/gap/fndiff.rs
# ===========================================================================
#
# Every function below names the Rust item it mirrors. Divergence is not a
# style question here: the control fails on it.

OP, REG, IMM, DISP, TARGET, SHIFT, MASK, CR, SPR, FLAG = range(10)
TAG = {
    OP: "opcode", REG: "reg", IMM: "imm", DISP: "disp", TARGET: "branch-target",
    SHIFT: "shift", MASK: "mask", CR: "cr-field", SPR: "spr", FLAG: "flag",
}

A_FORM_XO = (18, 20, 21, 22, 23, 24, 25, 26, 28, 29, 30, 31)


def _mask(width):
    return 0xFFFFFFFF if width >= 32 else (1 << width) - 1


def bits(w, hi, lo):
    """`fndiff::bits` — PPC bit numbering, bit 0 is the MSB."""
    return (w >> (31 - lo)) & _mask(lo - hi + 1)


def _f(name, kind, hi, lo, w):
    return (name, kind, hi, lo, bits(w, hi, lo))


def _reencode(fields):
    """`Decoded::reencode` — the round-trip that makes a decode a decode."""
    out = 0
    for (_n, _k, hi, lo, val) in fields:
        out |= (val & _mask(lo - hi + 1)) << (31 - lo)
    return out & 0xFFFFFFFF


def _a_form(w):
    return [
        _f("FRT", REG, 6, 10, w), _f("FRA", REG, 11, 15, w),
        _f("FRB", REG, 16, 20, w), _f("FRC", REG, 21, 25, w),
        _f("XO", OP, 26, 30, w), _f("Rc", FLAG, 31, 31, w),
    ]


def _decode_31(w):
    """`fndiff::decode_31`."""
    xo = bits(w, 21, 30)
    if xo in (0, 32):
        return "X", [
            _f("BF", CR, 6, 8, w), _f("rsv9", FLAG, 9, 9, w), _f("L", FLAG, 10, 10, w),
            _f("RA", REG, 11, 15, w), _f("RB", REG, 16, 20, w),
            _f("XO", OP, 21, 30, w), _f("rsv31", FLAG, 31, 31, w),
        ]
    if xo in (339, 467):
        return "XFX", [
            _f("RST", REG, 6, 10, w), _f("SPR", SPR, 11, 20, w),
            _f("XO", OP, 21, 30, w), _f("Rc", FLAG, 31, 31, w),
        ]
    if xo == 144:
        return "XFX", [
            _f("RS", REG, 6, 10, w), _f("rsv11", FLAG, 11, 11, w),
            _f("FXM", MASK, 12, 19, w), _f("rsv20", FLAG, 20, 20, w),
            _f("XO", OP, 21, 30, w), _f("Rc", FLAG, 31, 31, w),
        ]
    if xo == 824:
        return "X", [
            _f("RS", REG, 6, 10, w), _f("RA", REG, 11, 15, w),
            _f("SH", SHIFT, 16, 20, w), _f("XO", OP, 21, 30, w),
            _f("Rc", FLAG, 31, 31, w),
        ]
    return "X", [
        _f("RT", REG, 6, 10, w), _f("RA", REG, 11, 15, w), _f("RB", REG, 16, 20, w),
        _f("XO", OP, 21, 30, w), _f("Rc", FLAG, 31, 31, w),
    ]


def decode(w):
    """`fndiff::decode` — returns (form, fields) or None. Never a guess."""
    p = bits(w, 0, 5)
    if p == 3:
        form, fields = "D", [
            _f("TO", FLAG, 6, 10, w), _f("RA", REG, 11, 15, w), _f("SI", IMM, 16, 31, w)]
    elif p in (7, 8, 12, 13, 14, 15):
        form, fields = "D", [
            _f("RT", REG, 6, 10, w), _f("RA", REG, 11, 15, w), _f("SI", IMM, 16, 31, w)]
    elif p in (24, 25, 26, 27, 28, 29):
        form, fields = "D", [
            _f("RS", REG, 6, 10, w), _f("RA", REG, 11, 15, w), _f("UI", IMM, 16, 31, w)]
    elif p in (10, 11):
        form, fields = "D", [
            _f("BF", CR, 6, 8, w), _f("rsv9", FLAG, 9, 9, w), _f("L", FLAG, 10, 10, w),
            _f("RA", REG, 11, 15, w), _f("IMM", IMM, 16, 31, w)]
    elif 32 <= p <= 55:
        form, fields = "D", [
            _f("RST", REG, 6, 10, w), _f("RA", REG, 11, 15, w), _f("D", DISP, 16, 31, w)]
    elif p in (58, 62):
        form, fields = "DS", [
            _f("RST", REG, 6, 10, w), _f("RA", REG, 11, 15, w),
            _f("DS", DISP, 16, 29, w), _f("XO", OP, 30, 31, w)]
    elif p == 16:
        form, fields = "B", [
            _f("BO", FLAG, 6, 10, w), _f("BI", CR, 11, 15, w), _f("BD", TARGET, 16, 29, w),
            _f("AA", FLAG, 30, 30, w), _f("LK", FLAG, 31, 31, w)]
    elif p == 18:
        form, fields = "I", [
            _f("LI", TARGET, 6, 29, w), _f("AA", FLAG, 30, 30, w), _f("LK", FLAG, 31, 31, w)]
    elif p == 19:
        form, fields = "XL", [
            _f("BO", FLAG, 6, 10, w), _f("BI", CR, 11, 15, w), _f("BH", CR, 16, 20, w),
            _f("XO", OP, 21, 30, w), _f("LK", FLAG, 31, 31, w)]
    elif p in (20, 21, 23):
        third = _f("RB", REG, 16, 20, w) if p == 23 else _f("SH", SHIFT, 16, 20, w)
        form, fields = "M", [
            _f("RS", REG, 6, 10, w), _f("RA", REG, 11, 15, w), third,
            _f("MB", MASK, 21, 25, w), _f("ME", MASK, 26, 30, w), _f("Rc", FLAG, 31, 31, w)]
    elif p == 30:
        x4 = bits(w, 27, 30)
        if x4 in (8, 9):
            form, fields = "MDS", [
                _f("RS", REG, 6, 10, w), _f("RA", REG, 11, 15, w), _f("RB", REG, 16, 20, w),
                _f("mb", MASK, 21, 26, w), _f("XO", OP, 27, 30, w), _f("Rc", FLAG, 31, 31, w)]
        else:
            form, fields = "MD", [
                _f("RS", REG, 6, 10, w), _f("RA", REG, 11, 15, w), _f("sh", SHIFT, 16, 20, w),
                _f("mb", MASK, 21, 26, w), _f("XO", OP, 27, 29, w),
                _f("sh2", SHIFT, 30, 30, w), _f("Rc", FLAG, 31, 31, w)]
    elif p == 31:
        form, fields = _decode_31(w)
    elif p == 59:
        form, fields = "A", _a_form(w)
    elif p == 63:
        if bits(w, 26, 30) in A_FORM_XO:
            form, fields = "A", _a_form(w)
        else:
            form, fields = "X", [
                _f("FRT", REG, 6, 10, w), _f("FRA", REG, 11, 15, w), _f("FRB", REG, 16, 20, w),
                _f("XO", OP, 21, 30, w), _f("Rc", FLAG, 31, 31, w)]
    else:
        return None
    fields = [_f("OPCD", OP, 0, 5, w)] + fields
    if _reencode(fields) != w:
        return None                       # the round-trip rule, structurally
    return form, fields


def oe_set(w):
    """`fndiff::oe_set`."""
    return bits(w, 0, 5) == 31 and bits(w, 21, 21) == 1


def classify_pair(port, refw):
    """`fndiff::classify_pair` — `undecoded` is never a guess."""
    a = decode(port)
    b = decode(refw)
    if a is None or b is None:
        return "undecoded", []
    fa_form, fa_fields = a
    fb_form, fb_fields = b
    bmap = {n: (k, hi, lo, v) for (n, k, hi, lo, v) in fb_fields}
    amap_op = next(v for (n, _k, _h, _l, v) in fa_fields if n == "OPCD")
    bmap_op = next(v for (n, _k, _h, _l, v) in fb_fields if n == "OPCD")
    # Different instruction => `opcode`, decided BEFORE any field is compared.
    # This ordering is the #977 fix; reversing it reports 470 words undecoded.
    if amap_op != bmap_op:
        return "opcode", ["OPCD"]
    if fa_form != fb_form:
        return "opcode", ["FORM"]
    kinds, names = [], []
    for (n, k, _hi, _lo, v) in fa_fields:
        if n not in bmap:
            return "opcode", ["LAYOUT"]
        if v != bmap[n][3]:
            if k not in kinds:
                kinds.append(k)
            names.append(n)
    kinds.sort()
    if OP in kinds:
        return "opcode", names
    if not kinds:
        return "equal", names
    if len(kinds) == 1:
        return TAG[kinds[0]], names
    return "mixed:" + "+".join(TAG[k] for k in kinds), names


LCS_CELL_CAP = 400_000
BODY_CAP = 64

EQ, SUB, INS, DEL = "E", "S", "I", "D"


def _pair_runs(edits):
    """`fndiff::pair_runs` — LCS emits only ins/del; a one-word register change
    is one SUBSTITUTION, and without this pass no field comparison exists."""
    out = []
    k = 0
    n = len(edits)
    while k < n:
        dels, inss = [], []
        start = k
        while k < n:
            t = edits[k]
            if t[0] == DEL:
                dels.append(t[1])
            elif t[0] == INS:
                inss.append(t[1])
            else:
                break
            k += 1
        if k == start:
            out.append(edits[k])
            k += 1
            continue
        paired = min(len(dels), len(inss))
        for t in range(paired):
            out.append((SUB, inss[t], dels[t]))
        for i in inss[paired:]:
            out.append((INS, i))
        for j in dels[paired:]:
            out.append((DEL, j))
    return out


def align(port, refw, cap=LCS_CELL_CAP):
    """`fndiff::align` — prefix/suffix strip, LCS interior, then pair_runs."""
    n, m = len(port), len(refw)
    pre = 0
    while pre < n and pre < m and port[pre] == refw[pre]:
        pre += 1
    suf = 0
    while suf < n - pre and suf < m - pre and port[n - 1 - suf] == refw[m - 1 - suf]:
        suf += 1
    a = port[pre:n - suf]
    b = refw[pre:m - suf]
    edits = [(EQ, i, i) for i in range(pre)]
    capped = len(a) * len(b) > cap
    interior = []
    if capped:
        k = min(len(a), len(b))
        interior += [(SUB, pre + i, pre + i) for i in range(k)]
        interior += [(INS, pre + i) for i in range(k, len(a))]
        interior += [(DEL, pre + j) for j in range(k, len(b))]
    else:
        la, lb = len(a), len(b)
        # Classic LCS DP filled from the back, exactly as the Rust does.
        dp = [0] * ((la + 1) * (lb + 1))
        for i in range(la - 1, -1, -1):
            base = i * (lb + 1)
            nxt = (i + 1) * (lb + 1)
            ai = a[i]
            for j in range(lb - 1, -1, -1):
                if ai == b[j]:
                    dp[base + j] = dp[nxt + j + 1] + 1
                else:
                    x, y = dp[nxt + j], dp[base + j + 1]
                    dp[base + j] = x if x > y else y
        i = j = 0
        while i < la and j < lb:
            if a[i] == b[j]:
                interior.append((EQ, pre + i, pre + j))
                i += 1
                j += 1
            elif dp[(i + 1) * (lb + 1) + j] >= dp[i * (lb + 1) + j + 1]:
                interior.append((INS, pre + i))
                i += 1
            else:
                interior.append((DEL, pre + j))
                j += 1
        while i < la:
            interior.append((INS, pre + i))
            i += 1
        while j < lb:
            interior.append((DEL, pre + j))
            j += 1
        interior = _pair_runs(interior)
    edits += interior
    edits += [(EQ, n - suf + k, m - suf + k) for k in range(suf)]
    return edits, capped


def signature(shape, port, refw, relocs=()):
    """`fndiff::signature`. `port`/`refw` are word lists; `relocs` are the
    REFERENCE COMDAT's (VirtualAddress, type) pairs."""
    p, r = list(port), list(refw)
    edits, capped = align(p, r)
    reloc_words, reloc_unaligned = [], 0
    for (va, _t) in relocs:
        if va % 4:
            reloc_unaligned += 1
        reloc_words.append(va // 4)
    reloc_set = set(reloc_words)

    sig = {
        "shape": shape,
        "port_words": len(p), "ref_words": len(r),
        "prefix": 0, "suffix": 0, "first": None,
        "equal": 0, "sub": 0, "ins": 0, "del": 0,
        "same_multiset": sorted(p) == sorted(r),
        "capped": capped,
        "classes": defaultdict(int),
        "sub_at_reloc": 0, "del_at_reloc": 0,
        "reloc_unaligned": reloc_unaligned, "reloc_count": len(relocs),
        "oe": False,
    }
    seen_diff = False
    for e in edits:
        if e[0] == EQ:
            sig["equal"] += 1
            if not seen_diff:
                sig["prefix"] += 1
        elif e[0] == SUB:
            seen_diff = True
            sig["sub"] += 1
            i, j = e[1], e[2]
            if sig["first"] is None:
                sig["first"] = j
            pw, rw = p[i], r[j]
            if oe_set(pw) or oe_set(rw):
                sig["oe"] = True
            cls, _names = classify_pair(pw, rw)
            sig["classes"][cls] += 1
            if j in reloc_set:
                sig["sub_at_reloc"] += 1
        elif e[0] == INS:
            seen_diff = True
            sig["ins"] += 1
        else:
            seen_diff = True
            sig["del"] += 1
            j = e[1]
            if sig["first"] is None:
                sig["first"] = j
            if j in reloc_set:
                sig["del_at_reloc"] += 1
    suf = 0
    for e in reversed(edits):
        if e[0] == EQ:
            suf += 1
        else:
            break
    sig["suffix"] = suf
    if sig["first"] is None:
        sig["first"] = 0
    # THE POSITIVE ACCOUNTING IDENTITY, per row. A broken alignment still
    # produces a tidy-looking cluster table (fndiff.rs module docs).
    sig["accounting_ok"] = (
        sig["equal"] + sig["sub"] + sig["del"] == sig["ref_words"]
        and sig["equal"] + sig["sub"] + sig["ins"] == sig["port_words"]
    )
    sig["classes"] = dict(sig["classes"])
    return sig


def csig(s):
    """`DiffSig::csig`."""
    if s["port_words"] == s["ref_words"]:
        lenrel = "same-len"
    elif s["port_words"] > s["ref_words"]:
        lenrel = "port-longer"
    else:
        lenrel = "ref-longer"
    t = (s["sub"] > 0, s["ins"] > 0, s["del"] > 0)
    editshape = {
        (True, False, False): "sub-only", (False, True, False): "ins-only",
        (False, False, True): "del-only", (True, True, False): "sub+ins",
        (True, False, True): "sub+del", (False, True, True): "ins+del",
        (True, True, True): "sub+ins+del", (False, False, False): "none",
    }[t]
    classes = "+".join(sorted(s["classes"])) if s["classes"] else "-"
    return "%s|%s|%s|%s%s" % (
        s["shape"], lenrel, editshape, classes,
        "|reorder" if s["same_multiset"] else "")


def sig_fine(s):
    """`DiffSig::sig`."""
    return "%s|first@%d|%ds%di%dd" % (csig(s), s["first"], s["sub"], s["ins"], s["del"])


def has_transfer(ws):
    """`DIFF_STRUCTURE.md` §3's predicate, and `fndiff_report.py::has_transfer`:
    transfers control anywhere other than by its own terminal `blr` — primary 16
    or 18, or primary 19 with XO 16/528. An indirect `bctrl` and a conditional
    tail are both counted."""
    for w in ws:
        p = bits(w, 0, 5)
        if p in (16, 18):
            return True
        if p == 19 and bits(w, 21, 30) in (16, 528):
            return True
    return False


def has_linked_call(ws):
    """`bl` / `bctrl` — a transfer that sets LR."""
    for w in ws:
        p = bits(w, 0, 5)
        if p == 18 and bits(w, 31, 31) == 1:
            return True
        if p == 19 and bits(w, 21, 30) in (16, 528) and bits(w, 31, 31) == 1:
            return True
    return False


# ===========================================================================
# THE CONTROL — board #2064's rule, turned on this file
# ===========================================================================

CONTROL_FIELDS = ("first", "equal", "sub", "ins", "del", "same_multiset", "capped")


def run_control(jsonl_path, verbose=True):
    """Re-derive fndiff.rs's own per-row fields from its own port_hex/ref_hex.

    Relocation counters are NOT in the control: the JSONL carries `reloc_count`
    but not the relocation LIST, so `sub_at_reloc`/`del_at_reloc` cannot be
    re-derived from the row. That exclusion is published, not hidden — a
    control's denominator is the first thing to lie.
    """
    total = truncated = checked = ok = 0
    fails = Counter()
    examples = []
    with open(jsonl_path) as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            total += 1
            row = json.loads(line)
            if row.get("body_truncated"):
                truncated += 1
                continue
            checked += 1
            p = [int(x, 16) for x in row["port_hex"]]
            r = [int(x, 16) for x in row["ref_hex"]]
            mine = signature(row["shape"], p, r)
            bad = []
            for k in CONTROL_FIELDS:
                if mine[k] != row[k]:
                    bad.append("%s(mine=%s theirs=%s)" % (k, mine[k], row[k]))
            if mine["classes"] != row["classes"]:
                bad.append("classes(mine=%s theirs=%s)" % (mine["classes"], row["classes"]))
            if csig(mine) != row["csig"]:
                bad.append("csig(mine=%s theirs=%s)" % (csig(mine), row["csig"]))
            if sig_fine(mine) != row["sig"]:
                bad.append("sig")
            if mine["prefix"] != row["prefix"] or mine["suffix"] != row["suffix"]:
                bad.append("prefix/suffix")
            if not mine["accounting_ok"] or not row["accounting_ok"]:
                bad.append("accounting")
            if bad:
                fails[bad[0].split("(")[0]] += 1
                if len(examples) < 5:
                    examples.append((row["tu"], row["sym"], bad))
            else:
                ok += 1
    rate = (ok / checked) if checked else 0.0
    if verbose:
        print("CONTROL — re-derive gap/fndiff.rs from its own port_hex/ref_hex")
        print("  jsonl rows                  %6d" % total)
        print("  excluded, body_truncated    %6d   (bodies over BODY_CAP=%d words;" % (truncated, BODY_CAP))
        print("                                      their word lists are cut, so the row's own")
        print("                                      counts cannot be re-derived from them)")
        print("  CHECKED (the denominator)   %6d" % checked)
        print("  reproduced exactly          %6d   = %.4f%%" % (ok, 100.0 * rate))
        print("  NOT reproduced              %6d" % (checked - ok))
        print("  fields compared: %s + classes + csig + sig + prefix/suffix + accounting"
              % ", ".join(CONTROL_FIELDS))
        print("  NOT in the control: sub_at_reloc / del_at_reloc — the JSONL carries")
        print("    reloc_count but not the relocation LIST, so they are not re-derivable here.")
        if fails:
            print("  first-failing field, by count: %s" % dict(fails))
            for tu, sym, bad in examples:
                print("    %s  %s" % (tu, sym[:70]))
                print("      %s" % "; ".join(bad[:4]))
    return {"total": total, "truncated": truncated, "checked": checked, "ok": ok, "rate": rate}


# ---------------------------------------------------------------------------
# CONTROL ARM 2 — the classes arm 1 CANNOT REACH
# ---------------------------------------------------------------------------
#
# Arm 1 is green on 1,968 of 1,968 rows, and **a green instrument is a
# statement about the population it can reach.** That population's 7,912
# substituted words are `opcode` 7,902 · `mixed:reg+disp` 7 · `reg` 3, over 25
# distinct primary opcodes. So arm 1 exercises the ALIGNMENT hard (the negative
# controls confirm: disabling `pair_runs` drops it to 7.06 %) and the FIELD
# CLASSIFIER barely at all — it never once produces `imm`, a bare `disp`,
# `branch-target`, `shift`, `mask`, `cr-field`, `spr` or `flag`.
#
# Those are precisely the classes this lane expects the DECOMP population to
# contain. Grading the classifier on a population that does not contain them
# would be the "green over a population that cannot fail" trap.
#
# So arm 2 replays `gap/fndiff.rs`'s OWN registered expectations — the
# assertions in its `mod tests` — through this file. It is a cross-check
# against the Rust's committed contract, not a restatement of this file's
# beliefs: every expected value below is copied from an `assert_eq!` in
# `crates/c2-harness/src/gap/fndiff.rs`, cited by test name.

def _w(*parts):
    v = 0
    for (val, hi, lo) in parts:
        v |= (val & _mask(lo - hi + 1)) << (31 - lo)
    return v & 0xFFFFFFFF


def _xform31(xo, a, b, c, rc=0):
    return _w((31, 0, 5), (a, 6, 10), (b, 11, 15), (c, 16, 20), (xo, 21, 30), (rc, 31, 31))


XCHECK = []


def _x(name, got, want, src):
    XCHECK.append((name, got == want, got, want, src))


def run_xcheck(verbose=True):
    XCHECK.clear()
    add345 = _xform31(266, 3, 4, 5)
    add346 = _xform31(266, 3, 4, 6)
    subf345 = _xform31(40, 3, 4, 5)
    addi_1 = _w((14, 0, 5), (3, 6, 10), (0, 11, 15), (1, 16, 31))
    addi_2 = _w((14, 0, 5), (3, 6, 10), (0, 11, 15), (2, 16, 31))
    lwz_80 = _w((32, 0, 5), (3, 6, 10), (1, 11, 15), (80, 16, 31))
    lwz_84 = _w((32, 0, 5), (3, 6, 10), (1, 11, 15), (84, 16, 31))
    lwz_0 = _w((32, 0, 5), (3, 6, 10), (1, 11, 15), (0, 16, 31))
    b_16 = (18 << 26) | ((-16) & 0x03FFFFFC)
    b_32 = (18 << 26) | ((-32) & 0x03FFFFFC)
    srawi = _xform31(824, 4, 3, 31)
    cmpw = 0x7F832000
    vmx = 0x10000000 | (4 << 26)

    # a_register_only_difference_is_classified_as_one
    c, fl = classify_pair(add345, add346)
    _x("reg-only pair is `reg`", c, "reg", "a_register_only_difference_is_classified_as_one")
    _x("…and names RB", fl, ["RB"], "a_register_only_difference_is_classified_as_one")
    # an_immediate_only_difference_is_classified_as_one
    _x("imm-only pair is `imm`", classify_pair(addi_1, addi_2)[0], "imm",
       "an_immediate_only_difference_is_classified_as_one")
    # a_displacement_only_difference_is_classified_as_one
    _x("disp-only pair is `disp`", classify_pair(lwz_80, lwz_84)[0], "disp",
       "a_displacement_only_difference_is_classified_as_one")
    # a_branch_target_only_difference_is_classified_as_one
    _x("branch-target-only pair", classify_pair(b_16, b_32)[0], "branch-target",
       "a_branch_target_only_difference_is_classified_as_one")
    # two_different_instructions_are_an_opcode_difference
    _x("add vs subf is `opcode`", classify_pair(add345, subf345)[0], "opcode",
       "two_different_instructions_are_an_opcode_difference")
    _x("add vs lwz is `opcode`", classify_pair(add345, lwz_0)[0], "opcode",
       "two_different_instructions_are_an_opcode_difference")
    # two_d_form_instructions_with_different_field_names… (#977's regression)
    _x("addi 0x38800000 decodes", decode(0x38800000) is not None, True,
       "two_d_form_instructions_with_different_field_names_are_an_opcode_difference")
    _x("lwz 0x81440004 decodes", decode(0x81440004) is not None, True,
       "two_d_form_instructions_with_different_field_names_are_an_opcode_difference")
    c, fl = classify_pair(0x38800000, 0x81440004)
    _x("addi vs lwz is `opcode`", c, "opcode",
       "two_d_form_instructions_with_different_field_names_are_an_opcode_difference")
    _x("…and names OPCD", fl, ["OPCD"],
       "two_d_form_instructions_with_different_field_names_are_an_opcode_difference")
    _x("stw vs addi is `opcode`", classify_pair(0x9181FFF8, 0x386BFFEC)[0], "opcode",
       "two_d_form_instructions_with_different_field_names_are_an_opcode_difference")
    # one_primary_with_two_layouts_is_an_opcode_difference
    _x("cmp and srawi share a form", decode(cmpw)[0] == decode(srawi)[0], True,
       "one_primary_with_two_layouts_is_an_opcode_difference")
    _x("cmp vs srawi is `opcode`", classify_pair(cmpw, srawi)[0], "opcode",
       "one_primary_with_two_layouts_is_an_opcode_difference")
    # an_unmodelled_form_is_undecoded_and_never_guessed
    _x("primary 4 (VMX) is undecoded", decode(vmx), None,
       "an_unmodelled_form_is_undecoded_and_never_guessed")
    _x("…and classifies `undecoded`", classify_pair(vmx, vmx ^ 1)[0], "undecoded",
       "an_unmodelled_form_is_undecoded_and_never_guessed")
    # captured_frame_words_decode_and_name_their_fields
    d = decode(0x7D8802A6)
    _x("mflr form is XFX", d[0], "XFX", "captured_frame_words_decode_and_name_their_fields")
    spr = next((v for (n, _k, _h, _l, v) in d[1] if n == "SPR"), None)
    _x("mflr SPR val is 0x100", spr, 0x100, "captured_frame_words_decode_and_name_their_fields")
    sprk = next((k for (n, k, _h, _l, _v) in d[1] if n == "SPR"), None)
    _x("mflr SPR kind is Spr", sprk, SPR, "captured_frame_words_decode_and_name_their_fields")
    _x("mtlr round-trips", _reencode(decode(0x7D8803A6)[1]), 0x7D8803A6,
       "captured_frame_words_decode_and_name_their_fields")
    _x("blr form is XL", decode(0x4E800020)[0], "XL",
       "captured_frame_words_decode_and_name_their_fields")
    bfk = next((k for (n, k, _h, _l, _v) in decode(cmpw)[1] if n == "BF"), None)
    _x("cmpw BF kind is Cr", bfk, CR, "captured_frame_words_decode_and_name_their_fields")
    # alignment_finds_an_insertion_rather_than_a_shift
    edits, capped = align([1, 9, 2, 3], [1, 2, 3])
    _x("insertion is not three subs",
       (sum(1 for e in edits if e[0] == INS), sum(1 for e in edits if e[0] == SUB)), (1, 0),
       "alignment_finds_an_insertion_rather_than_a_shift")
    _x("…and is not capped", capped, False, "alignment_finds_an_insertion_rather_than_a_shift")
    # adjacent_insert_and_delete_runs_pair_into_substitutions
    edits, _ = align([1, 9, 3], [1, 2, 3])
    subs = [e for e in edits if e[0] == SUB]
    _x("adjacent ins/del pair into one sub", len(subs), 1,
       "adjacent_insert_and_delete_runs_pair_into_substitutions")
    _x("…at (1,1)", (subs[0][1], subs[0][2]) if subs else None, (1, 1),
       "adjacent_insert_and_delete_runs_pair_into_substitutions")
    # the_accounting_identity_holds_on_every_edit_shape
    for p, r in ([1, 2, 3], [1, 9, 3]), ([1, 2, 3], [1, 3]), ([1, 3], [1, 2, 3]), \
                ([3, 2, 1], [1, 2, 3]), ([], [1, 2]):
        _x("accounting identity on %s/%s" % (p, r), signature("seq", p, r)["accounting_ok"], True,
           "the_accounting_identity_holds_on_every_edit_shape")
    # a_pure_reordering_reads_as_same_multiset
    s = signature("seq", [1, 2, 3], [3, 2, 1])
    _x("reordering sets same_multiset", s["same_multiset"], True,
       "a_pure_reordering_reads_as_same_multiset")
    _x("…and csig ends |reorder", csig(s).endswith("|reorder"), True,
       "a_pure_reordering_reads_as_same_multiset")
    # a_relocated_word_is_marked_as_one
    s = signature("tail", [0x38600000, 0x48000001], [0x38600000, 0x48000005], [(4, 6)])
    _x("relocated sub count", s["sub"], 1, "a_relocated_word_is_marked_as_one")
    _x("relocated sub_at_reloc", s["sub_at_reloc"], 1, "a_relocated_word_is_marked_as_one")
    _x("relocated pair is branch-target", s["classes"].get("branch-target"), 1,
       "a_relocated_word_is_marked_as_one")
    # the_cap_degrades_positionally_and_says_so
    edits, capped = align(list(range(1, 21)), list(range(101, 121)), cap=1)
    _x("cap degrades positionally", capped, True, "the_cap_degrades_positionally_and_says_so")
    _x("…to 20 subs", (len(edits), all(e[0] == SUB for e in edits)), (20, True),
       "the_cap_degrades_positionally_and_says_so")

    passed = sum(1 for x in XCHECK if x[1])
    if verbose:
        print()
        print("CONTROL ARM 2 — gap/fndiff.rs's OWN registered expectations, replayed")
        print("  (the classes arm 1's population cannot reach: imm, disp, branch-target, spr, cr-field)")
        print("  assertions replayed  %3d" % len(XCHECK))
        print("  reproduced           %3d" % passed)
        for name, okk, got, want, src in XCHECK:
            if not okk:
                print("    FAIL  %-40s got=%r want=%r   [%s]" % (name, got, want, src))
        cover = Counter()
        for a, b in ((add345, add346), (addi_1, addi_2), (lwz_80, lwz_84), (b_16, b_32),
                     (add345, subf345), (cmpw, srawi), (vmx, vmx ^ 1)):
            cover[classify_pair(a, b)[0]] += 1
        print("  classes this arm exercises that arm 1 does NOT: %s"
              % ", ".join(sorted(k for k in cover if k not in ("opcode",))))
    return len(XCHECK), passed


# ===========================================================================
# COFF — the decomp side's bodies, read the way c2-obj reads them
# ===========================================================================

COFF_HDR = 20
SEC_HDR = 40
SYM_LEN = 18
IMAGE_SCN_LNK_COMDAT = 0x00001000
IMAGE_SYM_CLASS_EXTERNAL = 2
IMAGE_SYM_CLASS_STATIC = 3
IMAGE_SYM_CLASS_LABEL = 6
IMAGE_SYM_CLASS_WEAK_EXTERNAL = 105
TEXT_PREFIX = b".text"

# IMAGE_REL_PPC_* — confirmed present in this corpus's `.text` COMDATs, and
# they are exactly the set DIFF_STRUCTURE.md §3.2 names ("3 REL24 + 2 x
# (REFHI + PAIR) + 2 x (REFLO + PAIR)").
R_ADDR32 = 0x02
R_REL24 = 0x06
R_REFHI = 0x10
R_REFLO = 0x11
R_PAIR = 0x12

# Which bits of the word the linker will overwrite, per relocation type. A word
# under a relocation carries a PLACEHOLDER in these bits; comparing them across
# two objs compares link state, not compiler behaviour.
RELOC_PATCH_MASK = {
    R_REL24: 0x03FFFFFC,   # the LI field, bits 6..29
    R_REFHI: 0x0000FFFF,
    R_REFLO: 0x0000FFFF,
    R_ADDR32: 0xFFFFFFFF,
}


def _str_at(strtab, i):
    end = strtab.find(b"\0", i)
    return strtab[i:end] if end >= 0 else strtab[i:]


def read_obj(b):
    """Every `.text*` COMDAT function body in one obj, by symbol name.

    Returns {name: (words, relocs_by_word)} or None (fail-closed).

    ** WHY THIS IS NOT `c2_obj::text_comdat_entries` VERBATIM, AND THE BUG THAT
    FORCED THE DIFFERENCE. **

    `c2-obj` claims ONE leader per COMDAT `.text` section and hands it the
    section's WHOLE raw data. That is sound for the objs c2-rs grades, where c2
    under `/Gy` puts one function per COMDAT starting at offset 0. It is WRONG
    on this corpus, and the first run of this lane published a shape built on
    it. In `../dc3-decomp`'s `src/App.obj`, section 72 is 128 bytes and holds:

        val=0    class=3 aux=1  .text                 <- section definition
        val=8    class=2        ??0FilePath@@QAA@PBD@Z  <- THE FUNCTION
        val=88   class=3 aux=0  __unwind$275902       <- NOT code
        val=24/72/88/104/128 class=6  $M2759xx        <- line labels, INTERIOR

    Taking the whole section gave the function 32 words against the target's
    20, with two leading zero words and a second prologue inside. So the body
    is `[symbol Value, next boundary)`, where a boundary is an EXTERNAL /
    WEAK-EXTERNAL / non-definition STATIC symbol — and `IMAGE_SYM_CLASS_LABEL`
    (6) symbols are INTERIOR and must never truncate a body.
    """
    if len(b) < COFF_HDR:
        return None
    nsec = struct.unpack_from("<H", b, 2)[0]
    psym = struct.unpack_from("<I", b, 8)[0]
    nsym = struct.unpack_from("<I", b, 12)[0]
    sym_end = psym + nsym * SYM_LEN
    if not psym or sym_end > len(b):
        return None
    strtab = b[sym_end:]

    secs, is_text = [], []
    for i in range(nsec):
        o = COFF_HDR + i * SEC_HDR
        raw = b[o:o + 8]
        if raw[:1] == b"/":
            name = _str_at(strtab, int(raw[1:].rstrip(b"\0").decode()))
        else:
            name = raw.rstrip(b"\0")
        size, ptr = struct.unpack_from("<II", b, o + 16)
        prel, = struct.unpack_from("<I", b, o + 24)
        nrel, = struct.unpack_from("<H", b, o + 32)
        chars, = struct.unpack_from("<I", b, o + 36)
        secs.append((name, size, ptr, prel, nrel, chars))
        is_text.append(name.startswith(TEXT_PREFIX) and bool(chars & IMAGE_SCN_LNK_COMDAT))

    # Pass 1: every symbol, with its name, so a relocation can name its target.
    allsyms = []
    i = 0
    while i < nsym:
        o = psym + i * SYM_LEN
        if o + SYM_LEN > len(b):
            return None
        naux = b[o + 17]
        if b[o:o + 4] == b"\0\0\0\0":
            at, = struct.unpack_from("<I", b, o + 4)
            nm = _str_at(strtab, at)
        else:
            nm = b[o:o + 8].rstrip(b"\0")
        val, = struct.unpack_from("<I", b, o + 8)
        secnum, = struct.unpack_from("<h", b, o + 12)
        sclass = b[o + 16]
        allsyms.append((nm.decode("utf-8", "replace"), val, secnum, sclass, naux))
        i += 1 + naux
        if i > nsym:
            return None
    symname = [s[0] for s in allsyms]
    # Relocation SymbolTableIndex indexes RAW records, aux included; rebuild the
    # raw-index -> name map rather than assuming aux-free numbering.
    raw_name = {}
    i = 0
    k = 0
    while i < nsym and k < len(allsyms):
        raw_name[i] = allsyms[k][0]
        i += 1 + allsyms[k][4]
        k += 1

    # Pass 2: boundaries per section.
    bounds = defaultdict(list)
    starts = defaultdict(list)
    for (nm, val, secnum, sclass, naux) in allsyms:
        if not (1 <= secnum <= nsec) or not is_text[secnum - 1]:
            continue
        if sclass == IMAGE_SYM_CLASS_STATIC and naux == 1:
            continue                       # the section definition itself
        if sclass == IMAGE_SYM_CLASS_LABEL:
            continue                       # INTERIOR: `$M…` line labels
        bounds[secnum - 1].append(val)
        if sclass in (IMAGE_SYM_CLASS_EXTERNAL, IMAGE_SYM_CLASS_WEAK_EXTERNAL) \
                and not nm.startswith("__unwind$"):
            starts[secnum - 1].append((val, nm))

    out = {}
    for s in range(nsec):
        if not is_text[s]:
            continue
        _n, size, ptr, prel, nrel, _c = secs[s]
        data = b[ptr:ptr + size] if ptr else b""
        rel = []
        for k in range(nrel):
            ro = prel + k * 10
            if ro + 10 > len(b):
                return None
            va, sidx = struct.unpack_from("<II", b, ro)
            typ, = struct.unpack_from("<H", b, ro + 8)
            rel.append((va, typ, raw_name.get(sidx, "?%d" % sidx)))
        bs = sorted(set(bounds[s]))
        for (val, nm) in starts[s]:
            nxt = next((x for x in bs if x > val), size)
            body = data[val:nxt]
            if len(body) < 4:
                continue
            ws = [int.from_bytes(body[i:i + 4], "big") for i in range(0, len(body) - 3, 4)]
            # ** TRAILING ZERO WORDS ARE SECTION ALIGNMENT PADDING, NOT CODE —
            # and leaving them in manufactured a cluster. ** When a function is
            # the last thing in its COMDAT the extent runs to the section end,
            # which is padded to alignment. That put exactly two 0x00000000
            # words on the end of 530 of 531 bodies in one cluster
            # (`port-longer|ins-only|-`, 25.8 % of N) — a "difference" that is
            # a layout constant, with no compiler decision in it at all.
            # 0x00000000 is not a legal PPC instruction (primary opcode 0 is
            # reserved and `decode` refuses it), so a real body cannot end in
            # one. Trimmed on BOTH sides, and counted.
            while ws and ws[-1] == 0:
                ws.pop()
            if len(ws) < 1:
                continue
            rbw = {}
            for (va, typ, tn) in rel:
                if val <= va < nxt and (va - val) % 4 == 0:
                    rbw[(va - val) // 4] = (typ, tn)
            if nm not in out:              # first definition wins; collisions counted
                out[nm] = (ws, rbw)
    return out


def normalize(ws, rbw):
    """Zero the bits the LINKER will overwrite, and return the relocation
    target NAME per word alongside.

    ** THIS IS BOARD #984, IN THE MIRROR, AND IT IS WHY THE FIRST RUN OF THIS
    LANE WAS AN ARTIFACT. ** There, byte equality CREDITED a relocated word it
    had not checked: two `bl`s to different callees are the same four bytes
    under `/Gy`, because the placeholder displacement is `-(offset of the branch
    word)` whatever the callee. Here the same fact runs the other way and
    PENALISES: the same call, compiled into a section at a different offset, is
    a DIFFERENT four bytes. `??0FilePath@@QAA@PBD@Z` sits at offset 8 in our
    obj and 0 in the target, so every one of its four `bl`s differed by exactly
    8 and the lens read four `branch-target` substitutions in a function
    `decomp.db` scores 100.0 %.

    So the comparison is done on words whose relocated field is zeroed, with
    the relocation's TARGET SYMBOL NAME compared separately and never summed
    into the byte verdict — the same separation `fnbyte-exact` and
    `fnbyte-reloc-differs` are kept in (#884, #986).
    """
    out = list(ws)
    names = {}
    for wi, (typ, tn) in rbw.items():
        if wi >= len(out):
            continue
        m = RELOC_PATCH_MASK.get(typ)
        if m is not None:
            out[wi] = out[wi] & (~m & 0xFFFFFFFF)
        names[wi] = (typ, tn)
    return out, names


# ===========================================================================
# THE MEASUREMENT
# ===========================================================================

def load_percents(root):
    """decomp.db `current_percent` — A SELECTOR, NEVER THE MEASUREMENT.

    It is objdiff's fuzzy match percentage, the exact scoring
    docs/FUNCTION_BYTE_MATCH.md refuses because it pays more for a wrong emit
    than for an honest refusal. It is used here only to name the band a human
    permuter is actually run on. Every shape figure comes from the bytes.
    """
    try:
        import sqlite3
    except ImportError:
        return {}
    p = os.path.join(root, "decomp.db")
    if not os.path.exists(p):
        return {}
    con = sqlite3.connect("file:%s?mode=ro" % p, uri=True)
    out = {}
    for sym, pct in con.execute(
            "select symbol, current_percent from functions where current_percent is not null"):
        out[sym] = pct
    con.close()
    return out


def measure(root, out_json=None):
    cfg = json.load(open(os.path.join(root, "objdiff.json")))
    units = cfg["units"]
    pct = load_percents(root)

    stats = Counter()
    rows = []
    refused_units = []
    for u in units:
        tp, bp = u.get("target_path"), u.get("base_path")
        if not tp or not bp:
            stats["unit-no-base-path"] += 1
            continue
        tpa, bpa = os.path.join(root, tp), os.path.join(root, bp)
        if not (os.path.exists(tpa) and os.path.exists(bpa)):
            stats["unit-obj-missing"] += 1
            continue
        stats["unit-pairable"] += 1
        tgt = read_obj(open(tpa, "rb").read())
        base = read_obj(open(bpa, "rb").read())
        if tgt is None or base is None:
            stats["unit-coff-refused"] += 1
            refused_units.append(u["name"])
            continue
        stats["unit-read"] += 1
        common = set(tgt) & set(base)
        stats["sym-target-only"] += len(set(tgt) - common)
        stats["sym-base-only"] += len(set(base) - common)
        for sym in sorted(common):
            tws, trbw = tgt[sym]
            bws, brbw = base[sym]
            stats["P"] += 1
            # THE NORMALISED COMPARISON. Two verdicts, kept apart and never
            # summed: the BYTES (relocated fields zeroed) and the relocation
            # TARGET NAMES.
            pn, pnames = normalize(bws, brbw)
            rn, rnames = normalize(tws, trbw)
            bytes_equal = pn == rn
            # PAIR (0x12) is excluded from the NAME comparison: its
            # `VirtualAddress` field carries the other half of a REFHI/REFLO
            # pair, not an address, so it does not name a site.
            same_targets = (
                [pnames[i][1] for i in sorted(pnames) if pnames[i][0] != R_PAIR] ==
                [rnames[i][1] for i in sorted(rnames) if rnames[i][0] != R_PAIR]
            )
            if bytes_equal and same_targets:
                stats["P-identical"] += 1
                continue
            if bytes_equal:
                # ** BROKEN OUT, NOT FOLDED INTO N — and this is the port side's
                # own convention, not a convenience. ** `DIFF STRUCTURE` profiles
                # `fnbyte-differs` (1,968 at this tree); the byte-identical /
                # wrong-target bodies are `fnbyte-reloc-differs` (530), credited
                # nowhere and NOT part of the population the cluster table
                # describes (#884, #986, STATUS.md line 293). Folding them in
                # here would compare 8,916 apples against 1,968 oranges.
                #
                # Measured: essentially all of this class is TEMPLATE
                # INSTANTIATION NAMING under COMDAT folding — e.g. ours calls
                # `??H?$_Bit_iter@_NPB_N@…` where the target names
                # `??H?$_Bit_iter@U_Bit_reference@…`, with the bodies identical
                # word for word. decomp.db carries a `merged_symbols` table for
                # exactly this. It is not a permuter case.
                stats["P-reloc-differs"] += 1
                continue
            stats["N"] += 1
            if not pn or not rn:
                stats["N-empty-body"] += 1
                continue
            # `shape` on the port side is the catalogue variant the port chose;
            # there is no such thing here. A constant keeps csig comparable
            # across rows and is named `decomp` so no reader mistakes it for one
            # of the port's shapes.
            s = signature("decomp", pn, rn, [(i * 4, trbw[i][0]) for i in sorted(trbw)])
            # Call-target disagreement, measured on the NAMES, jointly.
            pt = [pnames[i][1] for i in sorted(pnames)]
            rt = [rnames[i][1] for i in sorted(rnames)]
            rows.append({
                "unit": u["name"], "sym": sym,
                "pct": pct.get(sym),
                "csig": csig(s), "first": s["first"],
                "port_words": s["port_words"], "ref_words": s["ref_words"],
                "sub": s["sub"], "ins": s["ins"], "del": s["del"],
                "equal": s["equal"], "prefix": s["prefix"], "suffix": s["suffix"],
                "same_multiset": s["same_multiset"], "capped": s["capped"],
                "classes": s["classes"], "accounting_ok": s["accounting_ok"],
                "sub_at_reloc": s["sub_at_reloc"], "del_at_reloc": s["del_at_reloc"],
                "reloc_count": s["reloc_count"],
                "bytes_equal": bytes_equal, "same_targets": same_targets,
                "n_reloc_ours": len(pt), "n_reloc_theirs": len(rt),
                "ours_transfer": has_transfer(pn), "theirs_transfer": has_transfer(rn),
                "ours_call": has_linked_call(pn), "theirs_call": has_linked_call(rn),
            })
    if out_json:
        with open(out_json, "w") as fh:
            for r in rows:
                fh.write(json.dumps(r) + "\n")
    return stats, rows, refused_units


def run_xcheck3(rows, stats, pct_map, verbose=True):
    """CONTROL ARM 3 — an EXTERNAL cross-check this lane did not author.

    `decomp.db` scores a function 100.0 when objdiff calls it a complete match.
    That score is a selector and never this lane's measurement — but it is an
    INDEPENDENT opinion about which bodies are identical, and a lens that
    disagrees with it wholesale is reading the wrong bytes.

    ** THIS ARM IS THE ONE THAT FIRED. ** The lane's first run took a COMDAT
    section's whole raw data as the body, c2-obj-style. It reported
    `??0FilePath@@QAA@PBD@Z` — `current_percent` 100.0 — as 32 words against 20
    with four `branch-target` substitutions, and the headline it produced
    (84.8 % `port-longer|sub+ins|branch-target`) was an artifact of section
    offset, not a fact about decomp near-misses. It is reported as the budgeted
    surprise, not repaired quietly.
    """
    at100 = {s for s, p in pct_map.items() if p is not None and p >= 100.0}
    differing = {r["sym"] for r in rows}
    contradict = sorted(at100 & differing)
    n100 = len(at100)
    if verbose:
        print()
        print("CONTROL ARM 3 — decomp.db's own 100%% verdict against this lens's bytes")
        print("  functions decomp.db scores 100.0                  %6d" % n100)
        print("  … that this lens nevertheless calls BYTES-DIFFERING %5d   (was 6,850 before"
              % len(contradict))
        print("                                                              the extent fix)")
        print("  (a lens reading the wrong extents disagrees wholesale; the first")
        print("   run of this lane disagreed on thousands and the shape it printed")
        print("   was an artifact of section offset — see the docstring)")
        for s in contradict[:8]:
            print("      still contradicting: %s" % s[:78])
    return {"at100": n100, "contradicting": len(contradict), "examples": contradict[:20]}


def band(rows, lo):
    """Rows whose decomp.db percent is >= lo. Measured JOINTLY on the row —
    never inferred by multiplying a differ-rate by a band-rate."""
    return [r for r in rows if r["pct"] is not None and r["pct"] >= lo]


def report(rows, label, denom_note=""):
    n = len(rows)
    print()
    print("=" * 74)
    print("%s — n = %d bodies%s" % (label, n, denom_note))
    print("=" * 74)
    if not n:
        print("  EMPTY. Nothing below is a statement about anything.")
        return
    broken = sum(1 for r in rows if not r["accounting_ok"])
    capped = sum(1 for r in rows if r["capped"])
    print("  accounting breaks (known answer 0): %d    LCS-capped rows: %d" % (broken, capped))
    reorder = sum(1 for r in rows if r["same_multiset"])
    print("  PURE REORDERINGS (same instruction multiset): %d / %d = %.2f%%"
          % (reorder, n, 100.0 * reorder / n))
    w0 = sum(1 for r in rows if r["first"] == 0)
    print("  first word ALREADY WRONG:                     %d / %d = %.2f%%"
          % (w0, n, 100.0 * w0 / n))
    # ** THE WORD CENSUS EXCLUDES LCS-CAPPED ROWS, AND HERE IS WHY. **
    # A capped row is aligned POSITIONALLY, so every position becomes a `Sub`
    # — including positions where the two words are EQUAL, which is where the
    # otherwise-impossible `equal` class comes from. On the port side this
    # cannot arise: `fndiff-align-capped` is 0 on every scan. Including them
    # here would compare a positional fallback against a real LCS.
    live = [r for r in rows if not r["capped"]]
    cls = Counter()
    for r in live:
        cls.update(r["classes"])
    tot = sum(cls.values())
    print("  substituted WORDS by decoded field class (%d words over the %d NON-CAPPED"
          % (tot, len(live)))
    print("    rows; the %d capped rows are excluded — they align positionally, which"
          % (n - len(live)))
    print("    manufactures `Sub`s between EQUAL words. The port side has 0 capped rows):")
    if tot:
        for k, v in cls.most_common():
            print("      %-22s %7d  %6.2f%%" % (k, v, 100.0 * v / tot))
    # THE SPLIT THAT DECIDES WHICH PERMUTER TO BUILD, measured jointly per body.
    struct_free = [r for r in live if r["classes"] and "opcode" not in r["classes"]
                   and not any(k.startswith("mixed:") and "opcode" in k for k in r["classes"])]
    any_op = [r for r in live if any(k == "opcode" or ("mixed:" in k and "opcode" in k)
                                     for k in r["classes"])]
    nosub = [r for r in live if not r["classes"]]
    print("  BODIES by whether ANY substituted word differs in its OPCODE (joint, per body):")
    print("      no substitutions at all (pure ins/del)   %5d / %d = %5.1f%%"
          % (len(nosub), len(live), 100.0 * len(nosub) / max(len(live), 1)))
    print("      substitutions, NONE an opcode difference %5d / %d = %5.1f%%   <- operand-level only"
          % (len(struct_free), len(live), 100.0 * len(struct_free) / max(len(live), 1)))
    print("      at least one OPCODE difference           %5d / %d = %5.1f%%"
          % (len(any_op), len(live), 100.0 * len(any_op) / max(len(live), 1)))
    fb = Counter()
    for r in rows:
        f = r["first"]
        fb["w%d" % f if f <= 7 else ("w8-15" if f <= 15 else ("w16-31" if f <= 31 else "w32+"))] += 1
    print("  first divergence: %s" % " · ".join("%s:%d" % kv for kv in fb.most_common(8)))
    ot = sum(1 for r in rows if r["ours_transfer"])
    tt = sum(1 for r in rows if r["theirs_transfer"])
    dis = sum(1 for r in rows if r["ours_transfer"] != r["theirs_transfer"])
    disc = sum(1 for r in rows if r["ours_call"] != r["theirs_call"])
    print("  transfer census (DIFF_STRUCTURE §3's own predicate):")
    print("      our body contains a call or branch   %d / %d = %.1f%%" % (ot, n, 100.0 * ot / n))
    print("      target body contains one             %d / %d = %.1f%%" % (tt, n, 100.0 * tt / n))
    print("      JOINT: the two DISAGREE              %d / %d = %.1f%%   <- the inlining signature"
          % (dis, n, 100.0 * dis / n))
    print("      JOINT: linked-call presence disagrees %d / %d = %.1f%%"
          % (disc, n, 100.0 * disc / n))
    print("  top clusters (shape|length|edit shape|field classes):")
    for k, v in Counter(r["csig"] for r in rows).most_common(10):
        print("      %5d (%5.1f%%)  %s" % (v, 100.0 * v / n, k))


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    mode = sys.argv[1]
    if mode == "control":
        res = run_control(sys.argv[2])
        nx, px = run_xcheck()
        return 0 if (res["rate"] >= 0.99 and nx == px) else 1
    if mode == "measure":
        root, jsonl = sys.argv[2], sys.argv[3]
        out_json = None
        if "--json" in sys.argv:
            out_json = sys.argv[sys.argv.index("--json") + 1]
        # ** THE GATE. ** No decomp number is printed until the re-expressed
        # lens has re-derived the shipped one, on BOTH arms. Board #2064: a
        # control that runs after the interesting number is not a control.
        res = run_control(jsonl)
        nx, px = run_xcheck()
        print()
        if res["rate"] < 0.99 or nx != px:
            print("CONTROL FAILED (arm1 %.4f%%, arm2 %d/%d). Refusing to print any decomp number."
                  % (100.0 * res["rate"], px, nx))
            print("Prereg decline condition 2 applies.")
            return 1
        print("CONTROL PASSED on both arms. The lens re-derives the shipped instrument; measuring.")
        stats, rows, refused = measure(root, out_json)
        print()
        print("POPULATION (../dc3-decomp), read from the bytes, not from a score:")
        for k in ("unit-no-base-path", "unit-obj-missing", "unit-pairable",
                  "unit-coff-refused", "unit-read", "P", "P-identical",
                  "P-reloc-differs", "N", "N-empty-body",
                  "sym-target-only", "sym-base-only"):
            print("    %-20s %8d" % (k, stats[k]))
        run_xcheck3(rows, stats, load_percents(root))
        if refused:
            print("    COFF-refused units (fail-closed, c2-obj's rule): %s%s"
                  % (", ".join(refused[:6]), " …" if len(refused) > 6 else ""))
        scored = [r for r in rows if r["pct"] is not None]
        print("    of N, carrying a decomp.db percent: %d (unscored %d)"
              % (len(scored), len(rows) - len(scored)))
        report(rows, "N — EVERY differing body with both sides present",
               " (the whole reachable near-miss population)")
        for lo, name in ((50, "N50"), (90, "N90"), (99, "N99")):
            report(band(rows, lo),
                   "%s — differing AND decomp.db percent >= %d" % (name, lo),
                   " (joint, measured on the row)")
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main())
