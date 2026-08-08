#!/usr/bin/env python3
"""w-mmio — R-GUARD-UNIMODAL, the park rule as grid 1 measured it.

`w-clear` published (board #1414) *"break the cycle by saving the LOWEST slot's
home into r11 … hoist the maximal prefix whose destination register is strictly
increasing"*.  Grid 1 refutes the first half: **the guard's own scrutinee is the
anchor whenever it can be**, and #1414's five cells could not see this because
in all five the guard's formal and the cycle minimum were the same register
`r3`.

THE RULE, in three clauses:

  1. ANCHOR.  Let `A` be the register the guard compares, if it lies in the
     permutation's cycle **and** the chain rooted at it is UNIMODAL (§3);
     otherwise `A` is the cycle's LOWEST argument register.
  2. CHAIN.   `r11 <- A`, then `A <- s(A)`, `s(A) <- s^2(A)`, … , and finally
     `s^-1(A) <- r11`.  (`s(d)` is the register destination `d` is filled from.)
  3. SPLIT.   The ENTRY block takes the park plus the strictly ASCENDING prefix
     of the chain; the call site takes the rest, which is then strictly
     DESCENDING — the `moves_descending` rule the emitter already implements
     for the unguarded case.

Clause 3 makes clause 1's unimodality test what it is: a chain can be laid out
as `ascending prefix | descending suffix` **exactly when its destination
sequence is unimodal**, so an anchor that would make the chain dip and rise
again is not a legal anchor at all, and c2 falls back to the minimum. For a
cycle of length <= 3 the minimum is ALWAYS unimodal, which is why the fallback
never fails inside the shipped class.
"""

ARG_REG = [3, 4, 5, 6, 7, 8, 9, 10]


def chain_from(perm, anchor_slot):
    """(dest_reg, src_reg) in dependency order; src_reg 11 closes the cycle."""
    m, moves, cur = anchor_slot, [], anchor_slot
    while True:
        nxt = perm[cur]
        if nxt == m:
            moves.append((ARG_REG[cur], 11))
            return moves
        moves.append((ARG_REG[cur], ARG_REG[nxt]))
        cur = nxt


def unimodal(dests):
    i = 0
    while i + 1 < len(dests) and dests[i] < dests[i + 1]:
        i += 1
    while i + 1 < len(dests) and dests[i] > dests[i + 1]:
        i += 1
    return i == len(dests) - 1


def predict(perm, cycle, guard_slots):
    """The rule. Returns (anchor_slot, entry_moves, call_moves)."""
    anchor = min(cycle)
    for gs in guard_slots:                       # the FIRST guard that can anchor
        if gs in cycle:
            d = [x for x, _ in chain_from(perm, gs)]
            if unimodal(d):
                anchor = gs
            break
    moves = chain_from(perm, anchor)
    dests = [d for d, _ in moves]
    j = 0
    while j + 1 < len(dests) and dests[j] < dests[j + 1]:
        j += 1
    return anchor, moves[:j], moves[j:]
