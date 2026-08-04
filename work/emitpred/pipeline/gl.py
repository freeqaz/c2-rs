"""Separator-aware `.gl` name-table reader.

Verbatim port of the harness rules, carried over from the w-phase7plan lane's
known-answer-gated `work/lane-c/readers.py`:

  R1 codec_framed  = crates/c2-il/src/codec.rs::gl_offset_framed
  R2 emit_framed   = crates/c2-il/src/func/bind.rs::emit_offset_framed
  R3 symbol_runs   = crates/c2-il/src/func/gl.rs::symbol_runs(sep26)

NEVER use raw `strings` on a `.gl`: the 00|26 separator alphabet concatenates
adjacent names.
"""
import struct
import bisect

EMIT_MAX_NAME_TO_OFFSET = 32


def codec_framed(gl):
    """o >= 7, gl[o]==0x80, gl[o-7]==0x80, gl[o-5]==0x10, gl[o-4..o-1]==0"""
    out = []
    n = len(gl)
    p = 0
    while p + 5 <= n:
        if (p >= 7 and gl[p] == 0x80 and gl[p - 7] == 0x80 and gl[p - 5] == 0x10
                and gl[p - 4] == 0 and gl[p - 3] == 0 and gl[p - 2] == 0 and gl[p - 1] == 0):
            out.append((p, struct.unpack_from('<I', gl, p + 1)[0]))
        p += 1
    return out


def emit_framed(gl, step=5):
    """bind::emit_offset_framed. gl_body_record_names walks with p += 5 on a hit,
    p += 1 otherwise; EmitBinding::new does the same. step kept explicit."""
    out = []
    n = len(gl)
    p = 0
    while p + 5 <= n:
        if (p >= 7 and gl[p] == 0x80 and gl[p - 7] == 0x80
                and gl[p - 4] == 0 and gl[p - 3] == 0 and gl[p - 2] == 0 and gl[p - 1] == 0):
            out.append((p, struct.unpack_from('<I', gl, p + 1)[0]))
            p += step
        else:
            p += 1
    return out


def symbol_runs(gl, sep26):
    """gl::symbol_runs. Returns (start, end, name, sepbyte) where sepbyte is the
    OPENING separator gl[start-1]."""
    n = len(gl)
    if sep26:
        is_sep = lambda b: b == 0 or b == 0x26
    else:
        is_sep = lambda b: b == 0
    out = []
    i = 0
    while i < n:
        if not is_sep(gl[i]):
            i += 1
            continue
        start = i + 1
        end = start
        while end < n and not is_sep(gl[end]):
            end += 1
        if end >= n or end == start:
            i += 1
            continue
        b = gl[start:end]
        plausible = all(0x21 <= c <= 0x7E for c in b) and (
            b[0] == 0x3F or (65 <= b[0] <= 90) or (97 <= b[0] <= 122) or b[0] == 0x5F)
        if plausible:
            out.append((start, end, b.decode('ascii'), gl[i]))
            i = end
        else:
            i += 1
    return out


def record_names(gl, sep26=True, step=5):
    """bind::gl_body_record_names, but per record rather than as a set:
    [(offset_field_pos, body_off, name|None, sep|None, dist|None)]"""
    runs = symbol_runs(gl, sep26)
    ends = [r[1] for r in runs]
    res = []
    for (p, v) in emit_framed(gl, step):
        k = bisect.bisect_right(ends, p)
        if k >= 1 and p - ends[k - 1] <= EMIT_MAX_NAME_TO_OFFSET:
            res.append((p, v, runs[k - 1][2], runs[k - 1][3], p - ends[k - 1]))
        else:
            res.append((p, v, None, None, None))
    return res
