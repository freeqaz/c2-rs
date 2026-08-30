#!/usr/bin/env python3
"""c7_price.py -- what does ADOPTING C7's ceiling VALUE cost, in the port's unit?

Lane `w-emitprice`, 2026-08-29.  std only; tooling, outside the crates/ rule.

RE-READ, NOT A PROBE.  `work/w-sizebracket/series.jsonl` is 176 committed cells,
each already compiled and graded against real c2.  Read-before-probe
(`WHITEBOX_LEVERAGE_2026-08-21.md`): the cells exist, so nothing is recompiled.
This is `w-lowerband`'s own move on the same file, asking a different question.

THE QUESTION
------------
C7 is the ceiling's VALUE: `DAT_10c46318 = 0x10 << DAT_10c2ea98`, and
`P_INLINE` SS6.8.6 settles `k = 3` at run time, so the value is **128**.  Its
unit is `WORD [sym+0x50]` -- a pre-codegen INSTRUCTION count.

The port's counterpart (C8, `fitted`) is three constants in EMITTED BYTES:
`splice::INLINE_UNBOUNDED_BYTES` 64 (the accept region -- the port actually
performs the expansion there), `comdat::INLINE_DECLINE_BYTES` 128 (the refusal),
`comdat::INLINE_DECLINE_LOOP_BYTES` 80.

`w-lowerband` SS6.7.1 already graded `.gl SIZE < 128` and found 16 /O1
counterexamples in both directions.  That grades the WRONG unit twice over:
SS2.1b measured `.gl SIZE` as an upper bound on the tested quantity, and the
port's constant is not `.gl SIZE` either.  So this script grades the ONE unit
the port can act on -- `callee_text`, the callee's emitted `.text` bytes -- and
prints, for every candidate threshold, the two error counts SEPARATELY:

    FALSE-INLINE   the rule says "c2 inlines" and c2 KEPT the call.
    FALSE-KEEP     the rule says "c2 keeps" and c2 INLINED it.

AND THE CONSEQUENCE OF EACH FLIPS BETWEEN THE PORT'S TWO SEAMS, which is the
single most important thing on this page and is why no summed error count can
price C7:

  splice.rs S7, `body <= INLINE_UNBOUNDED_BYTES` -- the ACCEPT region. The port
  PERFORMS the expansion here.
      FALSE-INLINE -> the port splices a body c2 did not  = WRONG EMIT
      FALSE-KEEP   -> the port keeps a call c2 inlined    = lost reach

  comdat::fenced_inlined_callee, `body <= INLINE_DECLINE_BYTES` -- a REFUSAL.
  The port declines the whole TU here.
      FALSE-INLINE -> the fence fires where c2 kept       = lost reach
      FALSE-KEEP   -> the fence does NOT fire where c2 inlined, so the port
                      emits a `bl` c2 did not             = WRONG EMIT

So the SAME error count is a wrong emit at one seam and lost reach at the other.
`fenced_inlined_callee`'s own doc says this in words -- "the hazard is
inverted" -- and this script is the first thing to put a number on it.
PROGRESS_METRIC SS5.2: a wrong emit scores STRICTLY BELOW the refusal it
replaced, so the two columns may never be summed or traded.

usage:  c7_price.py [--controls]
"""
import json
import os
import sys

PATH = os.environ.get('C2RS_SIZEBRACKET_SERIES', 'work/w-sizebracket/series.jsonl')

# c2's ceiling, in c2's unit (P_INLINE SS6.6.1 / SS6.8.6, k = 3 at run time).
C2_CEILING_INSTRS = 0x10 << 3           # 128

# The port's three incumbent constants, in the port's unit.
PORT = {
    'splice::INLINE_UNBOUNDED_BYTES  [ACCEPT seam: FALSE-INLINE = wrong emit]': 64,
    'comdat::INLINE_DECLINE_LOOP_BYTES [FENCE seam: FALSE-KEEP = wrong emit]': 80,
    'comdat::INLINE_DECLINE_BYTES     [FENCE seam: FALSE-KEEP = wrong emit]': 128,
}

# Candidate translations of "128 instructions" into emitted bytes.  Each is a
# claim about the instructions -> bytes converter that SS6.6.1 names as the
# second missing link and SS6.7 confirms is still missing.
CANDIDATES = {
    'C7 ADOPTED: 128 raw as a byte count': 128,
    'C7 ADOPTED: 128 instrs x 4 B/PPC word (1:1 instrs:words)': 512,
    'C7 ADOPTED through a converter FITTED to the /O1 bracket (108,116]': 112,
}


def cells():
    rows, seen = [], set()
    with open(PATH) as f:
        for ln in f:
            ln = ln.strip()
            if not ln:
                continue
            r = json.loads(ln)
            if r['tag'] in seen:
                continue
            seen.add(r['tag'])
            rows.append(r)
    return rows


def grade(rows, key, thr):
    """`key < thr => c2 inlines`.  Returns (wrong_accept, wrong_refuse, n)."""
    wa = wr = n = 0
    for r in rows:
        v, arm = r.get(key), r.get('arm')
        inl = None if arm is None else (arm == 'inlined')
        if v is None or inl is None:
            continue
        n += 1
        pred = v < thr
        if pred and not inl:
            wa += 1
        elif not pred and inl:
            wr += 1
    return wa, wr, n


def table(rows, label):
    print(f'--- {label} ---')
    print(f'{"rule":<66} {"n":>4} {"FALSE-INLINE":>13} {"FALSE-KEEP":>12}')
    for name, thr in list(PORT.items()) + list(CANDIDATES.items()):
        wa, wr, n = grade(rows, 'callee_text', thr)
        print(f'{name:<66} {n:>4} {wa:>13} {wr:>12}')
    wa, wr, n = grade(rows, 'gl_size', C2_CEILING_INSTRS)
    print(f'{"[c2 unit] .gl SIZE < 128 -- w-lowerband SS6.7.1s own grading":<66}'
          f' {n:>4} {wa:>13} {wr:>12}')
    print()


def controls():
    """#3336 -- watched RED before any verdict is quoted."""
    rows = cells()
    ok = True

    # C1 GREEN: the file is the one w-lowerband read -- 168 unique tags.
    print(f'C1 (GREEN expected, 168): unique cells = {len(rows)}')
    ok &= len(rows) == 168

    # C2 RED: a threshold below every cell must be ALL wrong-refuse and NO
    # wrong-accept.  A grader that reports errors on both sides here is scoring
    # something other than the rule.
    wa, wr, n = grade(rows, 'callee_text', 0)
    print(f'C2 (RED expected: thr=0 -> 0 false-inline, many false-keep): '
          f'{wa} / {wr} of {n}')
    ok &= (wa == 0 and wr > 0)

    # C3 RED: a threshold above every cell must be the mirror image.
    wa, wr, n = grade(rows, 'callee_text', 10 ** 9)
    print(f'C3 (RED expected: thr=inf -> many false-inline, 0 false-keep): '
          f'{wa} / {wr} of {n}')
    ok &= (wa > 0 and wr == 0)

    # C4: w-lowerband's published /O1 numbers must re-derive EXACTLY from this
    # file through an independent implementation, or this script is reading a
    # different population than the page it is arguing with.
    o1 = [r for r in rows if r.get('profile') == 'O1']
    wa, wr, n = grade(o1, 'gl_size', 128)
    print(f'C4 (GREEN expected, w-lowerband SS6.7.1 /O1 = 8 and 8): '
          f'.gl SIZE < 128 on {n} /O1 cells -> {wa} kept-below / {wr} inlined-above')
    ok &= (wa == 8 and wr == 8)

    print()
    print('CONTROLS', 'PASS' if ok else 'FAIL')
    return 0 if ok else 1


def main():
    if '--controls' in sys.argv[1:]:
        return controls()
    rows = cells()
    print(f'cells: {len(rows)} unique tags, from {PATH} (committed; nothing recompiled)')
    print()
    print('THE CONSEQUENCE OF EACH ERROR FLIPS BETWEEN THE TWO SEAMS:')
    print('  ACCEPT seam (splice.rs S7) : FALSE-INLINE = WRONG EMIT · FALSE-KEEP = lost reach')
    print('  FENCE  seam (comdat.rs)    : FALSE-INLINE = lost reach · FALSE-KEEP = WRONG EMIT')
    print('PROGRESS_METRIC SS5.2: a wrong emit scores STRICTLY BELOW the refusal it')
    print('replaced, so the two columns may never be summed and never traded.')
    print()
    for prof in ('O1', 'Ox'):
        sub = [r for r in rows if r.get('profile') == prof]
        if sub:
            table(sub, f'profile {prof} ({len(sub)} cells)'
                       f'{"  <- THE WORKLOADS OWN PROFILE" if prof == "O1" else ""}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
