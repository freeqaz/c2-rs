# GRID-N — the ARITY axis of `guard_chain_shared_tail`, six cells, one rule

Every cell compiled with the real `c2.dll` under wibo at the workload's own
flags and cwd (`work/w-vsnprnc/probe/n{3..8}.obj`). Arity is the ONLY axis:
same three guards on the same formal indices `[2,0,1]`, same `stb`, same two
error arms, same merged tail.

| n formals | `.text` | frame | HOISTED to the entry block | at the call, BEFORE the `lis` | AFTER the `lis` |
|---:|---:|---:|---|---|---|
| 3 | 144 | 96 | `mr r6,r5` | — | `r5←r4  r4←r3` |
| 4 | 148 | 96 | `mr r7,r6` | — | `r6←r5  r5←r4  r4←r3` |
| **5** | **152** | 96 | `mr r8,r7` | `r7←r6` | `r6←r5  r5←r4  r4←r3` |
| **6** | **156** | 96 | `mr r9,r8` | `r8←r7  r7←r6` | `r6←r5  r5←r4  r4←r3` |
| 7 | 160 | 96 | `mr r10,r9` | `r9←r8  r8←r7  r7←r6` | `r6←r5  r5←r4  r4←r3` |
| 8 | 164 | **112** | **NOTHING** — `stw r10,84(r1)` at the call instead | `r10←r9  r9←r8  r8←r7  r7←r6` | `r6←r5  r5←r4  r4←r3` |

`n5` is `vsnprnc.cpp::_vsprintf_s_l` word for word (152 B). `n6` is
`vswprnc.cpp::_vswprintf_s_l` (156 B).

## The rule, and the two hypotheses it settles

**THE `lis` IS EMITTED IMMEDIATELY BEFORE THE FIRST ROTATE MOVE WHOSE
DESTINATION IS r6 OR LOWER.** It is a statement about REGISTERS, not about
counts. 6 of 6, no exceptions.

The shipped fence named two hypotheses it could not separate at one witness:

* **"the `lis` is after the SECOND rotate step"** — true at n = 6 **and nowhere
  else**. REFUTED at n = 3, 4, 5, 7, 8.
* **"the `lis` is THREE steps before the last"** — true at n = 4…8, **REFUTED at
  n = 3**, where the hoist takes one of the three and only two moves follow.

Neither carried hypothesis survives. The register rule is the only one that fits
all six, and it fits `n = 8` too — where the count rules would have had nothing
to say, because the whole shape changes.

## The HOIST is also a rule, and it also was a constant

The emitter hard-codes `mr r9,r8`. Measured: **the topmost rotate step —
`ARG_REGS[n] ← ARG_REGS[n-1]` — is hoisted above the guards**, for n = 3…7.

## n = 8 is a DIFFERENT SHAPE and is refused, not extrapolated

The ninth argument does not fit the eight argument registers. c2 grows the frame
to 112, **spills `r10` to `84(r1)` at the call site**, and hoists nothing. This
is the boundary witness: the class is fenced to **3 ≤ n ≤ 7** and `n8.obj` is
why, rather than a guess about where it stops.

## STRUCTURAL BLIND SPOT of this grid

It varies arity and holds fixed: the guard indices, the guard count (3), the
store width, the callee count, the literal values, and the fact that the address
argument is in slot 0. It cannot see a rule in which the `lis`'s position
depends on any of those.
