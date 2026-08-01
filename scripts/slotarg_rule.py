#!/usr/bin/env python3
"""The W-SLOTARG ordering rule, stated as a TOTAL function, and checked against
every cell of both capture grids.  A rule that needs a free parameter per cell is
§9.14.7's disease, so this predicts the full instruction sequence — mnemonic,
destination, source and immediate — not just a shape."""
import sys
from slotarg_read import read

SCRATCH = 11


def split_off(k):
    """c2's `addis`/`addi` split for an offset that does not fit a signed 16-bit
    immediate.  Returns a list of (mnemonic, imm) applied in order."""
    if k == 0:
        return []
    if -0x8000 <= k < 0x8000:
        return [('addi', k)]
    hi = (k + 0x8000) >> 16
    lo = k - (hi << 16)
    return [('addis', hi)] if lo == 0 else [('addis', hi), ('addi', lo)]


def predict(dests, lits, addr_slot, base, k):
    """dests: register per call slot that this body writes (in slot order).
       lits:  {slot: value}. addr_slot: slot of the computed address.
       base:  the base formal's home register. k: the byte offset.
    Returns a list of (mnemonic, dst, src, imm)."""
    d = dests[addr_slot]
    parts = split_off(k)
    wide = bool(parts) and parts[0][0] == 'addis'
    # the address's own words. The word that READS THE BASE is always words[0].
    if not parts:
        words = [] if d == base else [('mr', d, base, None)]
    elif len(parts) == 1:
        m, imm = parts[0]
        words = [(m, d, base, imm)]
    else:
        words = [(parts[0][0], d, base, parts[0][1]),
                 (parts[1][0], d, d, parts[1][1])]

    walk = [('li', dests[s], 0, v) for s, v in sorted(lits.items(),
                                                      key=lambda kv: -dests[kv[0]])]
    clobbered = base in {dests[s] for s in lits}

    # The address's own DESCENDING position among the walk.  For the WIDE form
    # c2 additionally keeps at least one walk word ahead of the address — the
    # schedule is chosen for the two-word idiom, and a zero low half is dropped
    # afterwards rather than re-scheduled.
    pos = sum(1 for s in lits if dests[s] > d)
    idx = min(max(pos, 1), len(walk)) if wide else pos

    # A wide offset whose base would be overwritten by a walk word that runs
    # BEFORE the address's anchor is pre-saved into r11 and computed after the
    # whole walk. `addi`-only offsets never take this arm: their single word
    # simply moves to the front instead.
    if wide and clobbered and any(w[1] == base for w in walk[:idx]):
        moved = [(words[0][0], d, SCRATCH, words[0][3])] + words[1:]
        return [('mr', SCRATCH, base, None)] + walk + moved
    if len(words) == 2:
        # The high half is hoisted to the very front (§9.13.1's `lis` hoist).
        # The low half keeps the address's own descending anchor while the base
        # is safe; once a walk word is going to destroy the base, c2 closes the
        # computation as early as it legally can — ONE word after the high half.
        lo_at = 1 if clobbered else idx
        return [words[0]] + walk[:lo_at] + [words[1]] + walk[lo_at:]
    if clobbered and words:
        # One word that reads a base the walk is about to destroy: it goes
        # ahead of the whole walk.
        return words + walk
    return walk[:idx] + words + walk[idx:]


def fmt(seq):
    out = []
    for m, dst, src, imm in seq:
        if m == 'mr':
            out.append(f'mr r{dst},r{src}')
        elif m == 'li':
            out.append(f'li r{dst},{imm}')
        else:
            out.append(f'{m} r{dst},r{src},{imm}')
    return ' ; '.join(out)


def observed(ins):
    out = []
    for _, op, arg in ins:
        if op == 'b':
            continue
        a = arg.split('\t')[0].strip()
        if op == 'li':
            r, v = a.split(',')
            out.append(f'li {r},{int(v)}')
        elif op == 'mr':
            out.append(f'mr {a}')
        else:
            r, s, v = a.split(',')
            out.append(f'{op} {r},{s},{int(v)}')
    return ' ; '.join(out)


def cells_grid1(path='grid.cod'):
    for n, ins in read(path).items():
        if not n.startswith('g_'):
            continue
        _, kind, steps, off, nargs, slot = n.split('_')
        off, nargs, slot = int(off), int(nargs[:-1]), int(slot)
        if kind == 'free':
            base = 3                       # t is param 0
            dests = [3 + i for i in range(nargs)]
        else:
            base = 4                       # r is param 0, t is param 1
            dests = [4 + i for i in range(nargs)]
        lits = {i: 10 + i for i in range(nargs) if i != slot}
        yield n, dests, lits, slot, base, off, ins


def cells_grid2(path='grid2.cod'):
    for n, ins in read(path).items():
        if not n.startswith('h_'):
            continue
        _, off, nargs, slot, pad = n.split('_')
        off, nargs, slot, pad = int(off), int(nargs[:-1]), int(slot), int(pad[1:])
        base = 3 + pad                     # t is the last param
        dests = [3 + i for i in range(nargs)]
        lits = {i: 10 + i for i in range(nargs) if i != slot}
        yield n, dests, lits, slot, base, off, ins


if __name__ == '__main__':
    ok = bad = 0
    fails = []
    for gen in (cells_grid1, cells_grid2):
        for n, dests, lits, slot, base, off, ins in gen():
            p, o = fmt(predict(dests, lits, slot, base, off)), observed(ins)
            if p == o:
                ok += 1
            else:
                bad += 1
                fails.append((n, p, o))
    print(f'rule agrees with c2 on {ok}/{ok + bad} cells; {bad} disagree')
    for n, p, o in fails[:15]:
        print(f'\n  {n}\n    predicted {p}\n    observed  {o}')
