#!/usr/bin/env python3
"""Lane w-bss2: read the DATA-GLOBAL records out of an IL `.gl` file.

The `.gl` stream is a sequence of records; a namespace-scope *data* object's
record has the frame

    <u16 id LE> 00  <name> 00  [ALIGN-PREFIX]  T  K  00  02  SC  <size>  ...

with

  * ALIGN-PREFIX  two bytes `0xc2+2*log2(a)` `0x81`, present when the object
    carries an explicit or otherwise non-default alignment; `a` is then THE
    alignment, and the type byte degrades to 0x81.
  * T             the type's alignment class when no prefix is present:
                  0x82 -> 1, 0x84 -> 2, 0x86 -> 4, 0x88 -> 8.
  * SC            0x01 external linkage, 0x04 internal (`static`).
  * <size>        one byte if < 0x80, else `0x80` followed by a LE32.

`00 02` at +2/+3 is the discriminator that separates data records from function
records (which carry 0x03/0x04/0x05 at +2).

Every field here was located by one-axis probe diffs at the workload's flags and
is scored in `docs/OBJ_DATA_BSS_SHAPE.md` §A; nothing is assumed.
"""
import re

SHELL = ('.XBLD$W', '__C1_11886', '__C2_11886', '@comp.id')
TALIGN = {0x82: 1, 0x84: 2, 0x86: 4, 0x88: 8}


def _size_at(d, p):
    """(value, next_index) for the record's size field."""
    b = d[p]
    if b < 0x80:
        return b, p + 1
    if b == 0x80:
        return int.from_bytes(d[p + 1:p + 5], 'little'), p + 5
    return None, p + 1


def globals_in_order(d):
    """[(name, size, align, sc)] in `.gl` FILE order — the walk order of §5.2."""
    out = []
    for m in re.finditer(rb'[ -~]{1,512}\x00', d):
        name = m.group()[:-1].decode()
        if name in SHELL or '\\' in name or '/' in name:
            continue
        if not (name[0].isalpha() or name[0] in '?_$'):
            continue
        p = m.end()
        if p + 6 > len(d):
            continue
        # Record id: the LE16 immediately before the name, with an extra 0x00
        # separator for `?`-decorated (external) names and none for `$` ones.
        s = m.start()
        gid = None
        if name.startswith('$'):
            if s >= 2:
                gid = int.from_bytes(d[s - 2:s], 'little')
        elif s >= 3 and d[s - 1] == 0x00:
            gid = int.from_bytes(d[s - 3:s - 1], 'little')
        # `q` ends up at the K byte; the type slot before it is 1 byte (plain
        # type code, alignment implied) or 2 (align prefix + degraded 0x81).
        if 0xc2 <= d[p] <= 0xd6 and (d[p] - 0xc2) % 2 == 0 and d[p + 1] == 0x81:
            align = 1 << ((d[p] - 0xc2) >> 1)
            q = p + 2
        elif d[p] in TALIGN:
            align = TALIGN[d[p]]
            q = p + 1
        else:
            continue
        if q + 5 > len(d):
            continue
        if d[q + 1] != 0x00 or d[q + 2] != 0x02:
            continue          # function record, or not a data record
        sc = d[q + 3]
        if sc not in (0x01, 0x04):
            continue
        size, _ = _size_at(d, q + 4)
        if size is None:
            continue
        out.append(dict(name=name, size=size, align=align, sc=sc, kind=d[q], gid=gid))
    return out


def key(name):
    """`?x@@3DA` -> `x`; `$x` -> `x`; else unchanged (the source identifier)."""
    if name.startswith('?') and '@@' in name:
        return name[1:name.index('@@')]
    if name.startswith('$') and len(name) > 1:
        return name[1:]
    return name
