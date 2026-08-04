#!/usr/bin/env python3
"""il.py — c1xx-side IL readers: `.gl` symbol index and the `.ex` body split.

Ports of the harness's own rules (`crates/c2-il/src/func/gl.rs`,
`crates/c2-il/src/func/readers.rs`, `crates/c2-il/src/codec.rs`), kept
deliberately close to the Rust so the two can be diffed:

  * `read_token_var`   — the big-endian variable-width operand token
                         (2 bytes when bit 7 of byte 1 is clear, else 4).
  * `gl_symbol_index`  — operand token -> symbol name, conflict-refusing.
  * `split_ex`         — `.ex` split at the `4F 1F` function-start marker;
                         the `.gl` `80 <LE32>` framed field holds exactly these
                         offsets, 1:1 and in function order.

Nothing here reads any c2 output.
"""
import struct

FN_START = b"\x4f\x1f"
NAME_SEPARATORS = (0x00, 0x26)
SYMBOL_RECORD_KINDS = (0x00, 0x04, 0x0E, 0x10)
SYMBOL_CHARS = set(range(0x30, 0x3A)) | set(range(0x41, 0x5B)) | set(
    range(0x61, 0x7B)
) | {ord("_"), ord("$"), ord("?"), ord("@")}


def read_token_var(b, p):
    """readers.rs::read_token_var — (token, width) or None."""
    if p + 1 >= len(b):
        return None
    b0 = b[p]
    b1 = b[p + 1]
    if not (b1 & 0x80):
        return ((b0 << 8) | b1, 2)
    if p + 3 >= len(b):
        return None
    return (((b0 << 24) | (b1 << 16) | (b[p + 2] << 8) | b[p + 3]), 4)


def _indexable(bs):
    return (
        len(bs) >= 3
        and (bs[0] == 0x3F or (0x41 <= bs[0] <= 0x5A) or (0x61 <= bs[0] <= 0x7A) or bs[0] == 0x5F)
        and all(c in SYMBOL_CHARS for c in bs)
    )


def gl_symbol_index(glb):
    """gl.rs::gl_symbol_index — {token: name}. Tokens two records of equal rank
    disagree about are DROPPED, never resolved to the first."""
    out = {}
    n = len(glb)
    i = 0
    while i < n:
        if not (0x21 <= glb[i] <= 0x7E):
            i += 1
            continue
        start = i
        while i < n and 0x21 <= glb[i] <= 0x7E:
            i += 1
        if i >= n:
            break
        end = i
        name_at = None
        for p in range(start, end):
            if p == 0:
                continue
            if glb[p - 1] in NAME_SEPARATORS and _indexable(glb[p:end]):
                name_at = p
        if name_at is None:
            continue
        q = name_at
        for w in (4, 2):
            if q < w + 2:
                continue
            p = q - 1 - w
            t = read_token_var(glb, p)
            if t is None or t[1] != w:
                continue
            if glb[p - 1] not in SYMBOL_RECORD_KINDS:
                break
            name = glb[q:end].decode("latin1")
            rank = 1 if "@@" in name else 0
            prev = out.get(t[0], "MISSING")
            if prev == "MISSING":
                out[t[0]] = (rank, name)
            elif prev is None:
                pass
            elif prev[0] < rank:
                out[t[0]] = (rank, name)
            elif prev[0] > rank:
                pass
            elif prev[1] != name:
                out[t[0]] = None
            break
    return {t: v[1] for t, v in out.items() if v is not None}


def split_ex(ex):
    """Offsets of every `4F 1F` function-start marker, in order."""
    out = []
    i = ex.find(FN_START)
    while i >= 0:
        out.append(i)
        i = ex.find(FN_START, i + 1)
    return out


def segments(ex):
    """[(start, end)] for each `4F 1F`-anchored body."""
    st = split_ex(ex)
    return [(st[k], st[k + 1] if k + 1 < len(st) else len(ex)) for k in range(len(st))]
