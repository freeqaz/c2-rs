#!/usr/bin/env python3
"""dump_inlswitch.py -- what c2's 24 `-inl*` command-line switches actually SET,
what their operative defaults are, and who reads the result.

Lane `w-inlswitch`, 2026-08-28.  std only; tooling, outside the crates/ rule.

THE SHAPE, which is not what `#3718` assumed
--------------------------------------------
`work/w-inlfit/optmap.py` recovers c2's option-descriptor table and finds a run
of `-inl*` records whose value words tile 0x10c45db4..0x10c45e10.  Those words
are BSS (above the raw .data end 0x10c3cc00), so they are ZERO at load and the
option parser writes them only when the switch is on the command line.

They are not read by the inliner.  A single function, FUN_10b5b88f, SCATTERS
them -- together with 13 further unnamed words below the block -- into a
46-dword parameter record at [ecx+0x00..0xb4], and it is called TWICE:

    FUN_10b5ba71  ecx = 0x10c45ed0   -- "table B"
    FUN_10b5bc6e  ecx = 0x10c45e18   -- "table A"

which are exactly P_INLINE.md SS5's two POGO parameter tables.  Each caller then
runs a zero-guarded default sweep (`cmp ds:F,0 / jne skip / mov ds:F,<imm>`)
over its own table, so a switch left unset falls through to a default that
DIFFERS between the two tables.  FUN_10b5b86d then `rep movsd`s 46 dwords from
whichever table `DAT_10c6f1c8` selects into the live record at 0x10c3f510.

So a switch's "load-time default" is not a value at its own address -- that is
always 0 -- it is the value the sweep installs at its DESTINATION field.  This
script recovers both sweeps and joins them to the switch names.

WHAT IT PRINTS
    per switch: name, descriptor record, value word, destination offset,
    table-A default, table-B default, live address, reader count + addresses.

CONTROLS (SS3 of work/w-inlswitch/PREREG.md; #3336)
    --controls runs three, two of which must come back RED/GREEN as stated
    before any absence claim from this script may be quoted.

usage:  dump_inlswitch.py [--controls] [--detail] [--modes]
        --detail  also print each reader with its owner function and +-3
                  instructions of context, which is where "what it decides"
                  comes from.
        --modes   walk the writer sets of DAT_10c3de20 and DAT_10c6f1c8 --
                  c2's effective and requested POGO mode -- and print the
                  switch that reaches each value.
env:    C2RS_OBJDUMP_ASM (default ~/ghidra-projects/export/c2/objdump_intel.asm)
        C2RS_C2DLL       (default compilers/X360/16.00.11886.00/c2.dll)
"""
import os
import re
import struct
import sys

ASM = os.environ.get('C2RS_OBJDUMP_ASM',
                     os.path.expanduser('~/ghidra-projects/export/c2/objdump_intel.asm'))
DLL = os.environ.get('C2RS_C2DLL', 'compilers/X360/16.00.11886.00/c2.dll')
FUNCS = os.environ.get('C2RS_GHIDRA_FUNCS',
                       os.path.expanduser('~/ghidra-projects/export/c2/functions.tsv'))

SCATTER = (0x10b5b88f, 335)          # FUN_10b5b88f  -- value words -> [ecx+off]
TABLE_B = (0x10b5ba71, 509, 0x10c45ed0)   # FUN_10b5ba71
TABLE_A = (0x10b5bc6e, 541, 0x10c45e18)   # FUN_10b5bc6e
LIVE = 0x10c3f510                    # rep movsd destination, FUN_10b5b86d
NFIELD = 46                          # 0x2e dwords, the movsd count

LINE = re.compile(r'^([0-9a-f]{8}):\t([0-9a-f ]+?)\s*\t(\S+)\s*(.*)$')


# ---------------------------------------------------------------- image / PE

def pe_sections(d):
    pe = struct.unpack_from('<I', d, 0x3c)[0]
    nsec = struct.unpack_from('<H', d, pe + 6)[0]
    optsz = struct.unpack_from('<H', d, pe + 20)[0]
    base = struct.unpack_from('<I', d, pe + 24 + 28)[0]
    off = pe + 24 + optsz
    out = []
    for i in range(nsec):
        e = d[off + 40 * i: off + 40 * (i + 1)]
        name = e[0:8].rstrip(b'\0').decode()
        vsz, va, rsz, ro = struct.unpack_from('<IIII', e, 8)
        out.append((name, base + va, vsz, ro, rsz))
    return base, out


def file_off(secs, va):
    for _n, sva, _vsz, ro, rsz in secs:
        if sva <= va < sva + rsz:
            return ro + (va - sva)
    return None


def wstr(d, secs, va):
    o = file_off(secs, va)
    if o is None:
        return None
    out = []
    while True:
        w = struct.unpack_from('<H', d, o)[0]
        if w == 0:
            break
        if w > 0x7e:
            return None
        out.append(chr(w))
        o += 2
        if len(out) > 40:
            return None
    return ''.join(out)


# ------------------------------------------------------------------ listing

def load_listing():
    body = {}
    with open(ASM, errors='replace') as f:
        for ln in f:
            m = LINE.match(ln)
            if m:
                body[int(m.group(1), 16)] = (m.group(3), m.group(4).strip())
    return body


def load_funcs():
    out = []
    with open(FUNCS) as f:
        next(f)
        for ln in f:
            p = ln.rstrip('\n').split('\t')
            if len(p) >= 3:
                out.append((int(p[0], 16), int(p[1]), p[2]))
    out.sort()
    return out


def owner(funcs, va):
    lo, hi, best = 0, len(funcs) - 1, None
    while lo <= hi:
        m = (lo + hi) // 2
        if funcs[m][0] <= va:
            best = funcs[m]
            lo = m + 1
        else:
            hi = m - 1
    if best and best[0] <= va < best[0] + best[1]:
        return best[2]
    return '-'


# ------------------------------------------------- the option-descriptor run
# Same method and the same phase anchor as work/w-inlfit/optmap.py: the table
# is BSS, so it is recovered from the run of stores that BUILDS it, and the
# record phase is FOUND on the "-EHs"/"-EHa" boolean pair rather than assumed.

def descriptors(d, secs, body):
    slots = {}
    for a, (mn, ops) in body.items():
        if mn != 'mov':
            continue
        m = re.match(r'^(?:DWORD|WORD) PTR ds:0x([0-9a-f]+),0x([0-9a-f]+)$', ops)
        if not m:
            continue
        t = int(m.group(1), 16)
        if not (0x10c29000 <= a < 0x10c2a800):
            continue
        slots[t] = int(m.group(2), 16)
    anchor = None
    for t, v in slots.items():
        if wstr(d, secs, v) == '-EHs' and slots.get(t + 8) in (0x101, 0x501):
            anchor = t
            break
    if anchor is None:
        return None, slots
    lo = min(slots)
    recs = {}
    start = lo - ((lo - anchor) % 12)
    for t in range(start, max(slots) + 1, 12):
        n = slots.get(t)
        if n is None:
            continue
        nm = wstr(d, secs, n)
        if nm is None:
            continue
        recs[t] = (nm, slots.get(t + 4), slots.get(t + 8))
    return recs, slots


# --------------------------------------------------- the scatter and sweeps

def scatter_map(body):
    """FUN_10b5b88f: `mov eax,ds:SRC` then `mov [ecx+OFF],eax` -> {off: src}."""
    lo, size = SCATTER
    src = None
    out = {}
    for a in sorted(k for k in body if lo <= k < lo + size):
        mn, ops = body[a]
        if mn != 'mov':
            continue
        m = re.match(r'^eax,ds:0x([0-9a-f]+)$', ops)
        if m:
            src = (int(m.group(1), 16), a)
            continue
        m = re.match(r'^DWORD PTR \[ecx(?:\+0x([0-9a-f]+))?\],eax$', ops)
        if m and src is not None:
            off = int(m.group(1), 16) if m.group(1) else 0
            out[off] = src
            src = None
    return out


REG = ['eax', 'ecx', 'edx', 'ebx', 'esp', 'ebp', 'esi', 'edi']


def sweep(body, entry, size, tbase):
    """Recover a zero-guarded default sweep.

    Linear constant tracking over the function body, deliberately tiny and
    deliberately POISONING on anything it does not model -- an unknown value is
    reported as `?`, never silently guessed.
    """
    regs = {}
    pend = None            # last `push imm`
    out = {}
    guarded = set()
    last_cmp = None
    for a in sorted(k for k in body if entry <= k < entry + size):
        mn, ops = body[a]

        # ---- constant tracking
        if mn == 'push':
            m = re.match(r'^0x([0-9a-f]+)$', ops)
            pend = int(m.group(1), 16) if m else None
            continue
        if mn == 'pop':
            if ops in REG:
                if pend is None:
                    regs.pop(ops, None)
                else:
                    regs[ops] = pend
            pend = None
            continue
        pend = None
        if mn == 'xor':
            m = re.match(r'^(\w+),(\w+)$', ops)
            if m and m.group(1) == m.group(2) and m.group(1) in REG:
                regs[m.group(1)] = 0
                continue
        if mn == 'inc' and ops in REG:
            if ops in regs:
                regs[ops] = (regs[ops] + 1) & 0xffffffff
            continue
        if mn == 'dec' and ops in REG:
            if ops in regs:
                regs[ops] = (regs[ops] - 1) & 0xffffffff
            continue

        # ---- the zero guard: `cmp DWORD PTR ds:F,eax` (eax==0) or `,0x0`
        if mn == 'cmp':
            m = re.match(r'^DWORD PTR ds:0x([0-9a-f]+),(\w+|0x0)$', ops)
            if m:
                rhs = m.group(2)
                zero = (rhs == '0x0') or (regs.get(rhs) == 0)
                last_cmp = (int(m.group(1), 16), zero)
            else:
                last_cmp = None
            continue

        # ---- the stores
        tgt = val = None
        m = re.match(r'^(?:DWORD PTR )?ds:0x([0-9a-f]+),(\S+)$', ops)
        if mn == 'mov' and m:
            tgt = int(m.group(1), 16)
            rhs = m.group(2)
            if rhs.startswith('0x'):
                val = int(rhs, 16)
            elif rhs in REG:
                val = regs.get(rhs)
            else:
                val = None
        elif mn == 'or' and m:
            tgt = int(m.group(1), 16)
            rhs = m.group(2)
            # `or ds:F,0xffffffff` on a field known 0 == store -1
            if rhs == '0xffffffff':
                val = 0xffffffff
        if tgt is None:
            if mn.startswith('j'):
                pass
            continue
        if not (tbase <= tgt < tbase + 4 * NFIELD):
            continue
        off = tgt - tbase
        out[off] = val
        if last_cmp and last_cmp[0] == tgt and last_cmp[1]:
            guarded.add(off)
        last_cmp = None
    return out, guarded


def readers(body, funcs, va):
    """Every instruction whose operand text names `ds:<va>` -- reads AND writes,
    classified, so an absence is an absence of both."""
    tag = 'ds:0x%08x' % va
    hits = []
    for a in sorted(body):
        mn, ops = body[a]
        if tag in ops:
            # a store has the address on the LEFT of the comma
            lhs = ops.split(',')[0]
            kind = 'W' if tag in lhs and mn in (
                'mov', 'add', 'sub', 'or', 'and', 'xor', 'inc', 'dec') else 'R'
            if mn == 'cmp' or mn == 'test':
                kind = 'R'
            hits.append((a, kind, mn, ops, owner(funcs, a)))
    return hits


# ------------------------------------------------------------------ controls

def controls(body, funcs):
    ok = True
    print("CONTROLS -- watched before any verdict from this script is quoted "
          "(#3336)\n")

    # C1 GREEN: the reference enumerator must find the two writers of
    # DAT_10c46318 that P_INLINE SS6.6.1 establishes independently.
    h = readers(body, funcs, 0x10c46318)
    w = [x for x in h if x[1] == 'W']
    r = [x for x in h if x[1] == 'R']
    good = (len(w) == 2 and len(r) == 1
            and {x[0] for x in w} == {0x10b5e4d7, 0x10b5e4e8}
            and r[0][0] == 0x10b5fc8a)
    ok &= good
    print(f"  C1 GREEN  DAT_10c46318: {len(w)} writers {sorted(hex(x[0]) for x in w)}, "
          f"{len(r)} reader {[hex(x[0]) for x in r]}")
    print(f"            expected writers 0x10b5e4d7/0x10b5e4e8, reader 0x10b5fc8a "
          f"(P_INLINE SS6.6.1, established independently of this script)"
          f"  -> {'GREEN' if good else 'RED -- THE ENUMERATOR IS BROKEN'}")

    # C2 RED: an address no instruction can name.  Zero hits required.
    h = readers(body, funcs, 0xdeadbe00)
    bad = len(h) != 0
    ok &= not bad
    print(f"  C2 RED    planted address 0xdeadbe00: {len(h)} hits, 0 required"
          f"  -> {'RED as required (no false positives)' if not bad else 'BROKEN'}")

    # C3 RED: the descriptor harvest must COLLAPSE when pointed elsewhere.
    # Re-run the store harvest over a window that holds no descriptor run.
    n_here = sum(1 for a, (mn, ops) in body.items()
                 if 0x10c29000 <= a < 0x10c2a800 and mn == 'mov'
                 and re.match(r'^(?:DWORD|WORD) PTR ds:0x[0-9a-f]+,0x[0-9a-f]+$', ops))
    n_off = sum(1 for a, (mn, ops) in body.items()
                if 0x10b5b000 <= a < 0x10b5c800 and mn == 'mov'
                and re.match(r'^(?:DWORD|WORD) PTR ds:0x[0-9a-f]+,0x[0-9a-f]+$', ops))
    coll = n_off * 4 < n_here
    ok &= coll
    print(f"  C3 RED    immediate-store harvest: {n_here} in the descriptor window "
          f"0x10c29000..0x10c2a800, {n_off} in the same-sized band "
          f"0x10b5b000..0x10b5c800")
    print(f"            -> {'collapses; the window discriminates' if coll else 'DOES NOT COLLAPSE -- no discriminating power'}")

    print(f"\n  CONTROLS: {'GREEN' if ok else 'RED -- DO NOT QUOTE THIS SCRIPT'}\n")
    return ok


# ---------------------------------------------------------------------- main

def main():
    d = open(DLL, 'rb').read()
    base, secs = pe_sections(d)
    raw_data_end = None
    for n, sva, vsz, ro, rsz in secs:
        if n == '.data':
            raw_data_end = sva + rsz
    body = load_listing()
    funcs = load_funcs()

    # Print the listing path HOME-collapsed: this output is committed to the
    # `work/` evidence shelf, which `scripts/tracked_artifact_audit.sh` class 3
    # gates at ZERO absolute machine paths.
    home = os.path.expanduser('~')
    shown = ('~' + ASM[len(home):]) if ASM.startswith(home) else ASM
    print(f"image   : {DLL}  base 0x{base:08x}")
    print(f"listing : {shown}  {len(body)} decoded instruction starts")
    print(f"raw .data ends at 0x{raw_data_end:08x} -- everything at or above "
          f"this VA is BSS, ZERO AT LOAD\n")

    if '--controls' in sys.argv:
        if not controls(body, funcs):
            return 2
        if '--only-controls' in sys.argv:
            return 0

    recs, _slots = descriptors(d, secs, body)
    if not recs:
        sys.exit('descriptor anchor not found -- RED')
    by_value = {}
    for t, (nm, vp, kind) in recs.items():
        if vp is not None:
            by_value.setdefault(vp, (nm, t, kind))

    sc = scatter_map(body)
    defA, gA = sweep(body, TABLE_A[0], TABLE_A[1], TABLE_A[2])
    defB, gB = sweep(body, TABLE_B[0], TABLE_B[1], TABLE_B[2])

    print(f"scatter FUN_0x{SCATTER[0]:08x}: {len(sc)} of {NFIELD} parameter fields "
          f"are fed from a switch value word")
    srcs = sorted(v[0] for v in sc.values())
    print(f"  source block 0x{srcs[0]:08x}..0x{srcs[-1]:08x}  "
          f"({len(srcs)} words, "
          f"{'CONTIGUOUS' if srcs == list(range(srcs[0], srcs[-1] + 4, 4)) else 'WITH GAPS'})")
    print(f"table A (FUN_0x{TABLE_A[0]:08x}, base 0x{TABLE_A[2]:08x}): "
          f"{len(defA)} default stores, {len(gA)} of them zero-guarded")
    print(f"table B (FUN_0x{TABLE_B[0]:08x}, base 0x{TABLE_B[2]:08x}): "
          f"{len(defB)} default stores, {len(gB)} of them zero-guarded")
    print(f"live record 0x{LIVE:08x}, {NFIELD} dwords, filled by "
          f"`rep movsd` at 0x10b5b88a\n")

    def fmt(v):
        if v is None:
            return '?'
        if v > 0x7fffffff:
            return str(v - 0x100000000)
        return str(v)

    hdr = (f"{'off':>5} {'switch':<14}{'record':<12}{'value word':<12}"
           f"{'defA':>6}{'defB':>6}  {'live':<12}{'rd':>3}  readers")
    print(hdr)
    print('-' * len(hdr))
    named = tied = 0
    rows = []
    for off in range(0, 4 * NFIELD, 4):
        src = sc.get(off)
        nm, rec = '(no switch)', '-'
        if src:
            hit = by_value.get(src[0])
            if hit:
                nm, rec = hit[0], '0x%08x' % hit[1]
        live = LIVE + off
        rd = [x for x in readers(body, funcs, live) if x[1] == 'R']
        if src and nm != '(no switch)':
            named += 1
            if rd:
                tied += 1
        rows.append((off, nm, rec, src, defA.get(off), defB.get(off), rd))
        print(f"0x{off:03x} {nm:<14}{rec:<12}"
              f"{('0x%08x' % src[0]) if src else '-':<12}"
              f"{fmt(defA.get(off)):>6}{fmt(defB.get(off)):>6}  "
              f"0x{live:08x}  {len(rd):>3}  "
              f"{' '.join('0x%08x' % x[0] for x in rd[:4])}")

    inl = [r for r in rows if r[1].startswith('-inl')]
    print(f"\nswitch-fed fields      : {named} of {NFIELD}")
    print(f"  of which `-inl*`     : {len(inl)}")
    print(f"  with >=1 live reader : {tied} of {named} "
          f"({'-inl* only: %d of %d' % (sum(1 for r in inl if r[6]), len(inl))})")
    unfed = [r for r in rows if r[3] is None]
    print(f"fields with NO switch  : {len(unfed)} "
          f"({' '.join('0x%03x' % r[0] for r in unfed)})")
    unnamed = [r for r in rows if r[3] is not None and r[1] == '(no switch)']
    print(f"fed from a word with no descriptor name: {len(unnamed)} "
          f"({' '.join('0x%03x' % r[0] for r in unnamed)})")

    if '--detail' in sys.argv:
        print("\n\n=== READERS IN CONTEXT ===")
        print("Every live-record read, with its owner function and +-3 "
              "instructions.  This is the evidence behind every "
              "'what it decides' sentence in WB_INLSWITCH_FINDINGS.md.\n")
        addrs = sorted(body)
        pos = {a: i for i, a in enumerate(addrs)}
        for off, nm, rec, src, dA, dB, rd in rows:
            if not rd:
                continue
            print(f"--- +0x{off:03x}  {nm}  live 0x{LIVE + off:08x}  "
                  f"defA={fmt(dA)} defB={fmt(dB)}")
            for a, _k, _mn, _ops, own in rd:
                i = pos[a]
                print(f"    {own}")
                for j in range(max(0, i - 3), min(len(addrs), i + 4)):
                    va = addrs[j]
                    mn, ops = body[va]
                    print(f"     {'>>' if va == a else '  '} {va:08x}  "
                          f"{mn:<7}{ops[:62]}")
            print()

    if '--modes' in sys.argv:
        print("\n\n=== THE POGO MODE PAIR ===")
        print("DAT_10c3de20 is what `w-lowerband` SS7 filed as `389 refs, 10 "
              "writers, three values`.\nDAT_10c6f1c8 is the word that selects "
              "the parameter table at 0x10b5e50f.\n")
        for who, va in (('DAT_10c3de20 (effective mode)', 0x10c3de20),
                        ('DAT_10c6f1c8 (requested mode)', 0x10c6f1c8)):
            h = readers(body, funcs, va)
            w = [x for x in h if x[1] == 'W']
            fns = sorted({x[4] for x in w})
            print(f"{who}: {len(h)} references, {len(w)} WRITE instructions "
                  f"in {len(fns)} distinct owner functions")
            for a, _k, mn, ops, own in w:
                print(f"    0x{a:08x}  {own:<16} {mn:<5}{ops[:58]}")
            print()
    return 0


if __name__ == '__main__':
    sys.exit(main())
