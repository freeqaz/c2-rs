#!/usr/bin/env python3
"""coffdump — inspect a PPC/Xbox 360 MSVC COFF .obj: sections, symbols
(with objdiff-style inferred sizes), relocations, EH funclets, and a
hexdump/byte-diff of any one symbol's bytes.

This targets the *port's output* objects (and the reference c2.dll's captured
output objects) for manual eyeballing during debugging. It is NOT the
correctness judge -- that is `c2rs diff` (crates/c2-obj), which does the real
byte-exact compare with TimeDateStamp zeroed. Use coffdump when you already
know two objects differ (or want to browse one) and want to see *why* in a
terminal without pulling the crate into a debugger.

Adapted from milohax/rb3-xenon's scripts/analysis/coffx.py (COFF reader) and
its objdiff-mirroring size-inference / funclet-signature helpers. Pure
stdlib, no external deps, no repo-specific state -- takes .obj paths as args.

Usage:
  coffdump.py sections   <obj>
  coffdump.py symbols    <obj> [--section NAME] [--kind F|O|S|U]
  coffdump.py symbol     <obj> <name> [--mask-relocs]
  coffdump.py relocs     <obj> <section-name-or-index>
  coffdump.py funclets   <obj>
  coffdump.py diff       <obj-a> <obj-b> <name> [--mask-relocs]

Symbol kinds: F=function O=object(data) S=section U=unknown/label.
--mask-relocs zeroes the 4-byte window at each relocation site before
hexdumping/diffing (objdiff's funclet_signature) -- use it to see whether two
symbols differ ONLY in relocation targets (e.g. resolved to different but
equivalent runtime addresses) vs. in actual instruction/data bytes.
"""
import argparse
import struct
import sys

IMAGE_SYM_CLASS_EXTERNAL = 2
IMAGE_SYM_CLASS_STATIC = 3
IMAGE_SYM_CLASS_LABEL = 6
IMAGE_SCN_CNT_CODE = 0x00000020
IMAGE_SCN_MEM_EXECUTE = 0x20000000

K_FUNC, K_OBJ, K_SEC, K_UNK = 'F', 'O', 'S', 'U'
LABEL_PREFIXES = ('.L', 'LAB_', 'switchD_')


class Sec:
    __slots__ = ('name', 'vsize', 'vaddr', 'rawsize', 'rawptr', 'relptr', 'nrel',
                 'chars', 'data', 'relocs', 'is_code', 'index')


class Sym:
    __slots__ = ('name', 'value', 'sec', 'typ', 'cls', 'naux', 'size', 'index', 'kind')


def read_coff(data):
    """Parse a (non-bigobj) little-endian PPC/x86 COFF .obj into (sections, symbols)."""
    if len(data) < 20:
        return None, None
    mach, nsec, tds, symoff, nsym, optsz, ch = struct.unpack_from('<HHIIIHH', data, 0)
    if symoff == 0 or nsym == 0:
        return None, None
    strtab = symoff + nsym * 18
    secs = []
    so = 20 + optsz
    for i in range(nsec):
        o = so + i * 40
        if o + 40 > len(data):
            return None, None
        s = Sec()
        nb = data[o:o + 8]
        if nb[:1] == b'/':
            try:
                a = strtab + int(nb[1:].rstrip(b'\0').decode())
                e = data.find(b'\0', a)
                s.name = data[a:e].decode('ascii', 'replace')
            except Exception:
                s.name = nb.rstrip(b'\0').decode('ascii', 'replace')
        else:
            s.name = nb.rstrip(b'\0').decode('ascii', 'replace')
        s.vsize, s.vaddr, s.rawsize, s.rawptr, s.relptr, _lp, s.nrel, _nl = \
            struct.unpack_from('<IIIIIIHH', data, o + 8)
        s.chars = struct.unpack_from('<I', data, o + 36)[0]
        s.index = i
        s.is_code = bool(s.chars & (IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE))
        if s.rawptr and s.rawsize:
            s.data = data[s.rawptr:s.rawptr + s.rawsize]
        else:
            s.data = b''
        s.relocs = []
        for r in range(s.nrel):
            ro = s.relptr + r * 10
            if ro + 10 > len(data):
                break
            va, symidx, typ = struct.unpack_from('<IIH', data, ro)
            s.relocs.append((va, symidx, typ))
        secs.append(s)

    syms = []
    i = 0
    while i < nsym:
        off = symoff + i * 18
        if off + 18 > len(data):
            break
        nb = data[off:off + 8]
        if nb[:4] == b'\x00\x00\x00\x00':
            a = strtab + struct.unpack_from('<I', nb, 4)[0]
            e = data.find(b'\0', a)
            name = data[a:e if e >= 0 else len(data)].decode('ascii', 'replace')
        else:
            name = nb.split(b'\x00')[0].decode('ascii', 'replace')
        val = struct.unpack_from('<I', data, off + 8)[0]
        sec = struct.unpack_from('<h', data, off + 12)[0]
        typ = struct.unpack_from('<H', data, off + 14)[0]
        cls = data[off + 16]
        naux = data[off + 17]
        s = Sym()
        s.name, s.value, s.sec, s.typ, s.cls, s.naux = name, val, sec, typ, cls, naux
        s.size = 0
        s.index = i
        syms.append(s)
        i += 1 + naux
    return secs, syms


def sym_kind(s):
    """object-crate CoffSymbol::kind() -> objdiff SymbolKind."""
    if s.cls == IMAGE_SYM_CLASS_STATIC and s.value == 0 and s.naux > 0:
        return K_SEC
    derived = K_FUNC if ((s.typ >> 4) & 0xF) == 0x2 else K_OBJ
    if s.cls in (IMAGE_SYM_CLASS_EXTERNAL, IMAGE_SYM_CLASS_STATIC, 105):
        return derived
    if s.cls == IMAGE_SYM_CLASS_LABEL:
        return K_UNK
    return K_UNK


def infer_sizes(secs, syms):
    """Faithful port of objdiff `infer_symbol_sizes` (COFF symbols always
    start with size 0, so downstream tooling must infer contiguous extents)."""
    for s in syms:
        s.kind = sym_kind(s)
        s.size = 0
    lst = [s for s in syms if s.sec > 0 and s.sec - 1 < len(secs)]
    lst.sort(key=lambda s: (s.sec - 1, 0 if s.kind == K_SEC else 1, s.value, s.index))
    n = len(lst)
    i = 0
    last_end = (-1, 0)
    while i < n:
        s = lst[i]
        sidx = s.sec - 1
        i += 1
        if s.size != 0:
            continue
        if last_end[0] == sidx and last_end[1] > s.value:
            continue
        j = i
        nxt = None
        while j < n:
            t = lst[j]
            if t.sec - 1 != sidx:
                break
            islabel = (t.size == 0 and t.cls == IMAGE_SYM_CLASS_STATIC
                       and any(t.name.startswith(p) for p in LABEL_PREFIXES))
            if s.kind in (K_FUNC, K_OBJ):
                ok = t.kind in (K_FUNC, K_OBJ)
            else:
                ok = True
            if ok and not islabel:
                nxt = t
                break
            j += 1
        sec = secs[sidx]
        secsize = sec.rawsize if sec.rawsize else sec.vsize
        nxt_addr = nxt.value if nxt is not None else secsize
        if s.kind == K_SEC and not sec.is_code:
            newsize = 0
        else:
            newsize = max(0, nxt_addr - s.value)
        if newsize > 0:
            s.size = newsize
            if s.kind != K_SEC:
                last_end = (sidx, s.value + newsize)
    return syms


def funclet_signature(sec, sym, mask_relocs):
    """Symbol bytes, optionally with a 4-byte window zeroed at every
    relocation site inside the symbol (objdiff's funclet_signature) -- lets
    you tell "differs only in relocation target" from "differs in bytes"."""
    if sym.size == 0:
        return None
    start, end = sym.value, sym.value + sym.size
    if end > len(sec.data):
        return None
    b = bytearray(sec.data[start:end])
    if mask_relocs:
        for (va, si, typ) in sec.relocs:
            if va < start or va >= end:
                continue
            o = va - start
            for k in range(o, min(o + 4, len(b))):
                b[k] = 0
    return bytes(b)


def is_funclet_like(name):
    if name.startswith('__unwind$'):
        return name[9:].isdigit() and len(name) > 9
    if name.startswith('__catch$'):
        return name[8:].isdigit() and len(name) > 8
    if name.startswith('__unwind__merged_'):
        return True
    if name.startswith('fn_'):
        r = name[3:]
        return len(r) == 8 and all(c in '0123456789abcdefABCDEF' for c in r)
    if name.startswith('??__E') or name.startswith('??__F'):
        return True
    return False


def load(path):
    with open(path, 'rb') as f:
        data = f.read()
    secs, syms = read_coff(data)
    if secs is None:
        sys.exit(f"coffdump: not a readable (non-bigobj) COFF: {path}")
    infer_sizes(secs, syms)
    return secs, syms


def find_section(secs, ident):
    if ident.isdigit():
        i = int(ident)
        if 0 <= i < len(secs):
            return secs[i]
    for s in secs:
        if s.name == ident:
            return s
    sys.exit(f"coffdump: no such section: {ident}")


def find_symbol(syms, name):
    matches = [s for s in syms if s.name == name]
    if not matches:
        sys.exit(f"coffdump: no such symbol: {name}")
    # Prefer a defined (non-external) symbol with a nonzero inferred size.
    matches.sort(key=lambda s: (s.size == 0, s.sec <= 0))
    return matches[0]


def hexdump(data, mark=None):
    """16-bytes-per-line hex + ASCII. `mark` is an optional set of byte
    offsets to flag with '*' after the line (byte-diff mode)."""
    out = []
    for off in range(0, len(data), 16):
        chunk = data[off:off + 16]
        hexpart = ' '.join(f'{b:02x}' for b in chunk)
        asciipart = ''.join(chr(b) if 32 <= b < 127 else '.' for b in chunk)
        flag = '  *' if mark and any((off + k) in mark for k in range(len(chunk))) else ''
        out.append(f'{off:06x}  {hexpart:<47}  {asciipart}{flag}')
    return '\n'.join(out)


def cmd_sections(args):
    secs, _ = load(args.obj)
    print(f'{"idx":>3}  {"name":<16} {"rawsize":>8} {"vaddr":>8} {"nrel":>5}  code')
    for s in secs:
        print(f'{s.index:3d}  {s.name:<16} {s.rawsize:8d} {s.vaddr:8d} {s.nrel:5d}  {"yes" if s.is_code else ""}')


def cmd_symbols(args):
    secs, syms = load(args.obj)
    print(f'{"idx":>5}  {"kind":<4} {"section":<16} {"value":>8} {"size":>8}  name')
    for s in syms:
        if args.kind and s.kind != args.kind:
            continue
        secname = secs[s.sec - 1].name if 0 < s.sec <= len(secs) else str(s.sec)
        if args.section and secname != args.section:
            continue
        print(f'{s.index:5d}  {s.kind:<4} {secname:<16} {s.value:8d} {s.size:8d}  {s.name}')


def cmd_symbol(args):
    secs, syms = load(args.obj)
    sym = find_symbol(syms, args.name)
    secname = secs[sym.sec - 1].name if 0 < sym.sec <= len(secs) else str(sym.sec)
    print(f'{sym.name}  kind={sym.kind} section={secname} value={sym.value} size={sym.size} '
          f'class={sym.cls} naux={sym.naux}')
    if sym.sec <= 0 or sym.sec > len(secs):
        return
    b = funclet_signature(secs[sym.sec - 1], sym, args.mask_relocs)
    if b is not None:
        print(hexdump(b))


def cmd_relocs(args):
    secs, syms = load(args.obj)
    sec = find_section(secs, args.section)
    by_idx = {s.index: s for s in syms}
    print(f'{"va":>8}  {"symidx":>6}  {"type":>4}  target')
    for va, symidx, typ in sec.relocs:
        tgt = by_idx.get(symidx)
        print(f'{va:8d}  {symidx:6d}  {typ:4d}  {tgt.name if tgt else "?"}')


def cmd_funclets(args):
    _, syms = load(args.obj)
    for s in syms:
        if is_funclet_like(s.name):
            print(f'{s.kind}  {s.value:8d}  {s.size:8d}  {s.name}')


def cmd_diff(args):
    secs_a, syms_a = load(args.obj_a)
    secs_b, syms_b = load(args.obj_b)
    sym_a = find_symbol(syms_a, args.name)
    sym_b = find_symbol(syms_b, args.name)
    if sym_a.sec <= 0 or sym_a.sec > len(secs_a) or sym_b.sec <= 0 or sym_b.sec > len(secs_b):
        sys.exit(f'coffdump: {args.name} has no section-relative body in one of the objects')
    ba = funclet_signature(secs_a[sym_a.sec - 1], sym_a, args.mask_relocs)
    bb = funclet_signature(secs_b[sym_b.sec - 1], sym_b, args.mask_relocs)
    if ba is None or bb is None:
        sys.exit(f'coffdump: {args.name} has zero size in one of the objects')
    if ba == bb:
        print(f'{args.name}: identical ({len(ba)} bytes'
              f'{", relocation windows masked" if args.mask_relocs else ""})')
        return
    n = min(len(ba), len(bb))
    diff_offsets = {i for i in range(n) if ba[i] != bb[i]}
    diff_offsets |= set(range(n, max(len(ba), len(bb))))
    first = min(diff_offsets)
    print(f'{args.name}: DIFFERS -- {len(diff_offsets)} byte(s) differ, sizes {len(ba)} vs {len(bb)}, '
          f'first diff at offset {first:#x}')
    print(f'-- {args.obj_a} --')
    print(hexdump(ba, mark=diff_offsets))
    print(f'-- {args.obj_b} --')
    print(hexdump(bb, mark=diff_offsets))
    sys.exit(1)


def main():
    p = argparse.ArgumentParser(prog='coffdump', description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = p.add_subparsers(dest='cmd', required=True)

    s = sub.add_parser('sections'); s.add_argument('obj'); s.set_defaults(fn=cmd_sections)

    s = sub.add_parser('symbols'); s.add_argument('obj')
    s.add_argument('--section'); s.add_argument('--kind', choices=[K_FUNC, K_OBJ, K_SEC, K_UNK])
    s.set_defaults(fn=cmd_symbols)

    s = sub.add_parser('symbol'); s.add_argument('obj'); s.add_argument('name')
    s.add_argument('--mask-relocs', action='store_true'); s.set_defaults(fn=cmd_symbol)

    s = sub.add_parser('relocs'); s.add_argument('obj'); s.add_argument('section')
    s.set_defaults(fn=cmd_relocs)

    s = sub.add_parser('funclets'); s.add_argument('obj'); s.set_defaults(fn=cmd_funclets)

    s = sub.add_parser('diff'); s.add_argument('obj_a'); s.add_argument('obj_b')
    s.add_argument('name'); s.add_argument('--mask-relocs', action='store_true')
    s.set_defaults(fn=cmd_diff)

    args = p.parse_args()
    args.fn(args)


if __name__ == '__main__':
    main()
