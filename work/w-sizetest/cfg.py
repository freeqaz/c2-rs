#!/usr/bin/env python3
"""Intra-procedural CFG + dominator tool over the objdump export of c2.dll.

WHY THIS EXISTS
---------------
`P_INLINE.md` §6.5 concludes that `edi` at `0x10b5fc95` is the constant `0x2000`
because *"edi is callee-saved and nothing between the two writes it"*.  That is
an argument about an ADDRESS INTERVAL, and the question is a question about
PATHS: a jump that lands after `0x10b5fc31` and before `0x10b5fc95`, from a
block that never executed `0x10b5fc31`, would make the conclusion false while
leaving the interval argument literally true.  Board `#3830` reached the
opposite conclusion — `edi` is a caller-supplied parameter — and the wave-21
brief inherited it.

Neither claim can be settled by reading a listing top to bottom.  So this tool
computes, for a target instruction T and a candidate instruction D inside one
function, whether **every** path from the function entry to T executes D
(D dominates T), using an iterative dominator fixpoint over the basic blocks it
recovers from the listing.  It also prints reaching-definition style register
writes, so "no other writer" is a query result rather than an assertion.

BLIND SPOTS, stated rather than assumed (`#3505` is six for six):
  * Indirect branches (`jmp reg` / `jmp [table]`) are reported and make the CFG
    INCOMPLETE — the tool prints a warning and refuses a dominance verdict.
  * Calls are treated as fall-through: a callee that longjmps or that clobbers a
    callee-saved register is not modelled.  Callee-saved registers (ebx esi edi
    ebp) are safe under the platform ABI; caller-saved (eax ecx edx) are NOT,
    and the tool says so when asked about one.
  * Exception unwind edges are not modelled.
  * The listing is objdump's decode; a mid-instruction address will not appear
    as a block head and is reported as "not an instruction boundary".

Usage:
  python3 work/w-sizetest/cfg.py ENTRY END                    # blocks + edges
  python3 work/w-sizetest/cfg.py ENTRY END --dom TARGET       # dominators of T
  python3 work/w-sizetest/cfg.py ENTRY END --writes REG       # writes to REG
"""
import os
import re
import sys

ASM = os.environ.get('C2_OBJDUMP') or os.path.expanduser(
    '~/ghidra-projects/export/c2/objdump_intel.asm')
LINE = re.compile(r'^\s*([0-9a-f]{8}):\s+((?:[0-9a-f]{2} )+)\s*\t(.*)$')

COND = re.compile(r'^j(?!mp\b)[a-z]+$')
TARGET = re.compile(r'0x([0-9a-f]+)\s*$')

CALLEE_SAVED = {'ebx', 'esi', 'edi', 'ebp', 'esp'}
# first operand written
DEST_WRITERS = {
    'mov', 'add', 'sub', 'and', 'or', 'xor', 'inc', 'dec', 'neg', 'not',
    'shl', 'shr', 'sar', 'rol', 'ror', 'adc', 'sbb', 'imul', 'xchg', 'lea',
    'movzx', 'movsx', 'pop', 'btr', 'bts', 'btc', 'cdq', 'div', 'idiv', 'mul',
}


def load(lo, hi):
    insns = []
    with open(ASM, 'r', errors='replace') as fh:
        for line in fh:
            m = LINE.match(line)
            if not m:
                continue
            va = int(m.group(1), 16)
            if va < lo:
                continue
            if va >= hi:
                break
            insns.append((va, m.group(3).strip()))
    return insns


def build(insns):
    addrs = [va for va, _ in insns]
    aset = set(addrs)
    text = dict(insns)
    leaders = {addrs[0]}
    edges = {}          # va -> list of successor VAs
    indirect = []
    for i, (va, t) in enumerate(insns):
        mnem = t.split()[0]
        nxt = addrs[i + 1] if i + 1 < len(insns) else None
        if mnem == 'jmp' or COND.match(mnem):
            m = TARGET.search(t)
            if not m:
                indirect.append((va, t))
                edges[va] = []
                continue
            tgt = int(m.group(1), 16)
            succ = [tgt]
            if COND.match(mnem) and nxt is not None:
                succ.append(nxt)
                leaders.add(nxt)
            if tgt in aset:
                leaders.add(tgt)
            edges[va] = succ
        elif mnem in ('ret', 'retn', 'iret'):
            edges[va] = []
        else:
            edges[va] = [nxt] if nxt is not None else []
    return addrs, aset, text, leaders, edges, indirect


def preds_of(addrs, edges):
    p = {a: [] for a in addrs}
    for a, succ in edges.items():
        for s in succ:
            if s in p:
                p[s].append(a)
    return p


def dominators(addrs, edges, entry, aset):
    """Iterative dominator fixpoint at INSTRUCTION granularity."""
    pred = preds_of(addrs, edges)
    allset = frozenset(addrs)
    dom = {a: (frozenset([entry]) if a == entry else allset) for a in addrs}
    changed = True
    while changed:
        changed = False
        for a in addrs:
            if a == entry:
                continue
            ps = [p for p in pred[a] if p in dom]
            if not ps:
                new = frozenset([a])          # unreachable: only itself
            else:
                new = frozenset.intersection(*[dom[p] for p in ps]) | {a}
            if new != dom[a]:
                dom[a] = new
                changed = True
    return dom


def reachable(addrs, edges, entry):
    seen, stack = set(), [entry]
    while stack:
        a = stack.pop()
        if a in seen:
            continue
        seen.add(a)
        for s in edges.get(a, []):
            if s in edges:
                stack.append(s)
    return seen


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    lo = int(sys.argv[1], 16)
    hi = int(sys.argv[2], 16)
    insns = load(lo, hi)
    if not insns:
        print('no instructions in range', file=sys.stderr)
        return 1
    addrs, aset, text, leaders, edges, indirect = build(insns)
    entry = addrs[0]
    rest = sys.argv[3:]

    if indirect:
        print('!! INDIRECT BRANCH(ES) — CFG INCOMPLETE, no dominance verdict:')
        for va, t in indirect:
            print('   %08x: %s' % (va, t))

    live = reachable(addrs, edges, entry)
    dead = [a for a in addrs if a not in live]

    if '--dom' in rest:
        tgt = int(rest[rest.index('--dom') + 1], 16)
        if tgt not in aset:
            print('%08x is NOT an instruction boundary in [%08x,%08x)'
                  % (tgt, lo, hi))
            return 1
        if indirect:
            return 1
        dom = dominators(addrs, edges, entry, aset)
        d = sorted(dom[tgt])
        print('dominators of %08x  (%d instructions, entry %08x)'
              % (tgt, len(d), entry))
        for a in d:
            print('   %08x: %s' % (a, text[a]))
        return 0

    if '--writes' in rest:
        reg = rest[rest.index('--writes') + 1].lower()
        print('writes to %s in [%08x,%08x):' % (reg, lo, hi))
        for va, t in insns:
            mnem = t.split()[0]
            ops = t[len(mnem):].strip()
            first = ops.split(',')[0].strip()
            hit = (mnem in DEST_WRITERS and first == reg) or \
                  (mnem == 'pop' and first == reg) or \
                  (mnem in ('xchg',) and reg in ops)
            if hit:
                mark = '' if va in live else '   [UNREACHABLE]'
            if hit:
                print('   %08x: %-40s%s' % (va, t, mark))
        if reg in CALLEE_SAVED:
            print('   (%s is CALLEE-SAVED: calls in range preserve it)' % reg)
        else:
            print('   !! %s is CALLER-SAVED: every `call` above is an '
                  'unmodelled clobber' % reg)
        return 0

    print('entry %08x, %d instructions, %d block leaders'
          % (entry, len(addrs), len(leaders)))
    if dead:
        print('UNREACHABLE from entry (%d):' % len(dead))
        for a in dead:
            print('   %08x: %s' % (a, text[a]))
    pred = preds_of(addrs, edges)
    print('block leaders and their in-edges:')
    for a in sorted(leaders):
        ins = [p for p in pred[a] if p + 0 != a]
        print('   %08x  <- %s   | %s'
              % (a, ' '.join('%08x' % p for p in sorted(ins)) or '(entry)',
                 text[a]))
    return 0


if __name__ == '__main__':
    sys.exit(main())
