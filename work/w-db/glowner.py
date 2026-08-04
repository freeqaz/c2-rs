#!/usr/bin/env python3
"""glowner.py — the OWNER-SIDE `.gl` fields that `0x10b98e26`'s skips read.

w-mark found the root channel (the data initializer, in the `in` sub-stream) and
published the UNFILTERED reading of it: precision 0.27289, because
`0x10b98e26`/`0x10b98b00` carry owner tests it did not model.  Every field those
tests read is decoded here, from the byte stream, by disassembly.

THE RECORD HEADER, per tag.  `0x10b9b91f`..`0x10b9b93e` dispatches the tag byte
through a 0x1b-entry index at `0x10b9c615` into a jump table at `0x10b9c5d5`:

    tag 0x04 / 0x0e / 0x10   -> 0x10b9bdcf   KIND 4  (0x10b9bdfb: mov [esi+0x30],4)
    tag 0x01 / 0x02 / 0x1a   -> 0x10b9b945   KIND 1  (0x10b9b95f: mov [ebp-0x3c],1)
    tag 0x03                 -> 0x10b9bd3d   KIND 3
    tag 0x09                 -> 0x10b9c212   KIND 9  (the TYPE record)
    ... 12 more

and `0x10b98521: mov BYTE PTR [esi+0x30],bl` in the allocator `0x10b984c3` shows
`[sym+0x30]` is just the tag unless an arm overwrites it.

KIND 4 — `0x10b9bdcf`, the arm w-refs' `refs.head` already implements:

    <tag> <varU token -> +0x28> <byte -> +0x31 sep> <name>
          <byte -> +0x37 storage class, 0x10b9be0e>
          <i32c -> +0x40>
          <varU -> +0x20>                        THE FLAG WORD
          [ if (+0x20 & 0x200): <varU tok> -> +0x0c ]      0x10b9be6b
          ...

KIND 1 — `0x10b9b945`, decoded by THIS lane and by nobody before it:

    <tag> <byte -> +0x4d, saved at 0x10b9b957 and stored at 0x10b9b9d4>
          <varU token -> +0x28>  <byte -> +0x31 sep>  <name>
          <byte, READ AND DISCARDED at 0x10b9b9cc>
          <byte -> +0x37 bits 21..23, 0x10b9b9d7>
          <byte -> +0x37 bits 5..8 storage class, 0x10b9b9ee>
          <i32c -> +0x1c>
          <varU -> +0x20>                        THE FLAG WORD
          [ if (+0x20 & 0x200): <varU tok> -> +0x0c ]      0x10b9ba5f
          ...

    The template is a 0xa0-byte stack object memset at `0x10b9b945`
    (`lea eax,[ebp-0x6c]`), so `[ebp-0x1f]` is `+0x4d`, `[ebp-0x3c]` is `+0x30`,
    `[ebp-0x44]` is `+0x28`, `[ebp-0x4c]` is `+0x20`.

KIND 9 — `0x10b9c212`, the TYPE record, needed for `[[owner+0xc]+0x4d]`:

    <0x09> <varU token -> +0x28> <NUL-terminated string> <byte -> +0x4d>

    There is no separator byte before its string, so a separator-anchored scan
    (`model.indexable_runs`) cannot see it.  It is found here by searching for
    `09 <encoded token>` for exactly the tokens the owners ask about, and the
    search is fail-closed: a token whose pattern occurs zero times or more than
    once yields None rather than a guess.

WHAT READS WHAT — `0x10b98e26`, the initializer walk, once per module at
`0x10b7f0d2` (its only caller, via `0x10b3413d` in `0x10b34113`, whose only
caller is the p2 driver), BEFORE the compile loop at `0x10b7f15f`:

    G0   0x10b98e4a   [rec+0x20] & 1                -> skip the record
    S1   0x10b98e9f   ([owner+0x20] & 0x60) == 0x20 -> skip the record ENTIRELY
    W1   0x10b98b09   [owner+0x30] != 1             -> walk marks nothing
    W2   0x10b98b14   !([owner+0x20] & 0x480)       -> walk marks nothing
    S2   0x10b98ecd   [[owner+0xc]+0x4d] == 0x1d    -> skip the 0x10b98c0f pass
    S3   0x10b98ed9   kind 1 and [owner+0x20]&0x4000-> skip the 0x10b98c0f pass

Nothing here reads any c2 output.  stdlib only.
"""
import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "emitpred", "pipeline"))
sys.path.insert(0, os.path.join(HERE, "..", "w-roots"))
import il      # noqa: E402
import model   # noqa: E402
from glflags import var_u, i16c, i32c, get_byte, enc_var_u   # noqa: E402

KIND4_TAGS = (0x04, 0x0E, 0x10)
KIND1_TAGS = (0x01, 0x02)
TYPE_TAG = 0x09

OWNER_TOKEN_BIT = 0x200        # 0x10b9be6b / 0x10b9ba5f
WALK_BITS = 0x480              # 0x10b98b14 / 0x10b98c89
S1_MASK, S1_VALUE = 0x60, 0x20  # 0x10b98e9f
S3_BIT = 0x4000                # 0x10b98ed9
TYPE_1D = 0x1D                 # 0x10b98ecd / 0x10b98ba8
TYPE_0E = 0x0E                 # 0x10b98c35 / 0x10b98cbc


def _tail4(b, p):
    """From just past the name's NUL: the kind-4 post-name fields."""
    sc, p = get_byte(b, p)          # 0x10b9be0e
    _, p = i32c(b, p)               # +0x40
    f20, p = var_u(b, p)            # +0x20
    tok = None
    if f20 & OWNER_TOKEN_BIT:
        tok, p = var_u(b, p)        # +0x0c
    return sc, f20, tok, p


def _tail1(b, p):
    """From just past the name's NUL: the kind-1 post-name fields."""
    _, p = get_byte(b, p)           # 0x10b9b9cc, read and discarded
    _, p = get_byte(b, p)           # 0x10b9b9d7 -> +0x37 bits 21..23
    sc, p = get_byte(b, p)          # 0x10b9b9ee -> +0x37 bits 5..8
    _, p = i32c(b, p)               # +0x1c
    f20, p = var_u(b, p)            # +0x20
    tok = None
    if f20 & OWNER_TOKEN_BIT:
        tok, p = var_u(b, p)        # +0x0c
    return sc, f20, tok, p


def read_symbols(glb):
    """{token: rec} for every separator-anchored `.gl` record whose header
    decodes, plus a stats dict.

    `rec` = {kind, tag, name, tok, f4d, f20, typetok, sc, hdr_end, f20_pos,
             f20_width}.  `f20_pos`/`f20_width` locate the flag word's bytes
             exactly — that is what the mutation control needs.

    Ambiguity is fail-closed: a token two records disagree about is dropped,
    exactly as `il.gl_symbol_index` does for names.
    """
    runs = model.indexable_runs(glb)
    out = {}
    st = {"runs": len(runs), "k4": 0, "k1": 0, "other_tag": 0, "no_token": 0,
          "hdr_fail": 0, "dup_tok": 0, "tag_hist": {}}
    for (s, e, nm, sep) in runs:
        # the separator at s-1 is the record's +0x31 byte; the token ends there
        got = None
        for w in (4, 2):
            p = s - 1 - w
            if p < 1:
                continue
            t = il.read_token_var(glb, p)
            if t is None or t[1] != w:
                continue
            # kind-4 layout: <tag><token><sep><name>
            if glb[p - 1] in KIND4_TAGS:
                got = (t[0], glb[p - 1], 4, None, p)
                break
            # kind-1 layout: <tag><f4d><token><sep><name>
            if p >= 2 and glb[p - 2] in KIND1_TAGS:
                got = (t[0], glb[p - 2], 1, glb[p - 1], p)
                break
        if got is None:
            st["no_token"] += 1
            continue
        tok, tag, kind, f4d, tokpos = got
        st["tag_hist"][tag] = st["tag_hist"].get(tag, 0) + 1
        try:
            f20pos_probe = e + 1
            if kind == 4:
                sc, f20, ttok, end = _tail4(glb, f20pos_probe)
                st["k4"] += 1
            else:
                sc, f20, ttok, end = _tail1(glb, f20pos_probe)
                st["k1"] += 1
        except (IndexError, ValueError, struct.error):
            st["hdr_fail"] += 1
            continue
        # locate the +0x20 varU bytes exactly
        p = f20pos_probe
        try:
            if kind == 4:
                _, p = get_byte(glb, p)
                _, p = i32c(glb, p)
            else:
                _, p = get_byte(glb, p)
                _, p = get_byte(glb, p)
                _, p = get_byte(glb, p)
                _, p = i32c(glb, p)
        except (IndexError, ValueError, struct.error):
            st["hdr_fail"] += 1
            continue
        f20pos = p
        f20w = len(enc_var_u(f20))
        rec = {"kind": kind, "tag": tag, "name": nm, "tok": tok, "f4d": f4d,
               "f20": f20, "typetok": ttok, "sc": sc, "hdr_end": end,
               "f20_pos": f20pos, "f20_width": f20w,
               "roundtrip": enc_var_u(f20) == glb[f20pos:f20pos + f20w]}
        prev = out.get(tok)
        if prev is None:
            out[tok] = rec
        elif prev != rec:
            st["dup_tok"] += 1
            out[tok] = None
    st["bound"] = sum(1 for v in out.values() if v is not None)
    return {k: v for k, v in out.items() if v is not None}, st


def type_kind(glb, tok, _cache={}):
    """`[type+0x4d]` for a kind-9 TYPE record reached by token, or None.

    `0x10b9c212`: `<0x09> <varU tok> <NUL-terminated string> <byte -> +0x4d>`.
    Fail-closed: 0 or >1 occurrences of `09 <enc(tok)>` -> None.
    """
    key = (id(glb), tok)
    if key in _cache:
        return _cache[key]
    pat = bytes((TYPE_TAG,)) + enc_var_u(tok)
    hits = []
    i = glb.find(pat)
    while i >= 0 and len(hits) < 3:
        j = glb.find(b"\x00", i + len(pat))
        if j >= 0 and j + 1 < len(glb):
            hits.append(glb[j + 1])
        i = glb.find(pat, i + 1)
    v = hits[0] if len(hits) == 1 else None
    _cache[key] = v
    return v


if __name__ == "__main__":
    for d in sys.argv[1:]:
        glb = open(os.path.join(d, "gl"), "rb").read()
        recs, st = read_symbols(glb)
        f20h = {}
        f4dh = {}
        for r in recs.values():
            if r["kind"] == 1:
                f20h[r["f20"]] = f20h.get(r["f20"], 0) + 1
                f4dh[r["f4d"]] = f4dh.get(r["f4d"], 0) + 1
        print("%-44s bound=%d k4=%d k1=%d nohdr=%d notok=%d dup=%d"
              % (os.path.basename(d), st["bound"], st["k4"], st["k1"],
                 st["hdr_fail"], st["no_token"], st["dup_tok"]))
        print("   tags:", {hex(k): v for k, v in sorted(st["tag_hist"].items())})
        print("   kind-1 +0x20:", sorted(f20h.items(), key=lambda x: -x[1])[:10])
        print("   kind-1 +0x4d:", sorted(f4dh.items(), key=lambda x: -x[1])[:10])
