#!/usr/bin/env python3
"""labels.py — read every compiler-minted symbol out of a probe obj and print it
against the TU's `.gl` label seed.

Lane w-main2 measurement tooling. Read-only with respect to `crates/`.

Usage:  labels.py <obj> [<obj> ...]

For each obj it prints the label symbols (`$M`, `$T`, `__unwind$`, `__catch$`)
with their section, value and storage class, and — when a sibling `<stem>.seed`
file holds the `.gl` counter — the OFFSET of every label from that seed. The
offsets are the thing a model has to predict; the absolute numbers are a
statement about one compilation.
"""
import re
import struct
import sys


def symbols(path):
    d = open(path, 'rb').read()
    nsym = struct.unpack_from('<I', d, 12)[0]
    symptr = struct.unpack_from('<I', d, 8)[0]
    strtab = symptr + 18 * nsym
    out = []
    i = 0
    while i < nsym:
        off = symptr + 18 * i
        raw = d[off:off + 8]
        if raw[:4] == b'\0\0\0\0':
            noff = struct.unpack_from('<I', raw, 4)[0]
            end = d.index(b'\0', strtab + noff)
            name = d[strtab + noff:end].decode('latin1')
        else:
            name = raw.rstrip(b'\0').decode('latin1')
        val, sec, typ, sc, naux = struct.unpack_from('<IhHBB', d, off + 8)
        out.append((name, val, sec, sc))
        i += 1 + naux
    return out


LABEL = re.compile(r'^(\$M|\$T|__unwind\$|__catch\$)(\d+)$')


def main(argv):
    for path in argv:
        seed = None
        try:
            seed = int(open(path.rsplit('.', 1)[0] + '.seed').read().strip())
        except OSError:
            pass
        print('== %s   seed=%s' % (path, seed))
        rows = []
        for name, val, sec, sc in symbols(path):
            m = LABEL.match(name)
            if not m:
                continue
            rows.append((int(m.group(2)), m.group(1), name, val, sec, sc))
        for n, kind, name, val, sec, sc in sorted(rows):
            d = '' if seed is None else '  seed%+d' % (n - seed)
            print('   %-18s val=0x%-4x sec=%d sc=%d%s' % (name, val, sec, sc, d))


if __name__ == '__main__':
    main(sys.argv[1:])
