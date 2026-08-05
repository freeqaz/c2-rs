// **W71 — the chain whose intermediate REGISTERS the port allocates wrongly,
// refused.** `lane w-build`.
//
// This file is here because of a **live wrong-bytes emit found on master
// `4d6aa58`** with nothing of this lane's own in the tree:
//
//     int f(int a,int b,int c,int d) { return ((a + b) * c) * d; }
//
//     c2   /Ox   add r11,r3,r4 ; mullw r10,r11,r5 ; mullw r3,r10,r6
//     port /Ox   add r11,r3,r4 ; mullw r11,r11,r5 ; mullw r3,r11,r6
//                                      ^^^   Port=Mismatch @ offset 541
//
// `select_text` decides the whole chain at once — at `/Ox`, a chain containing
// any addition puts every intermediate in r11, otherwise they descend r11, r10,
// r9. That rule came from an enumeration of 11,664 four-leaf chains and is right
// about every one of them. `((a + b) * c) * d` contains an addition and descends
// anyway.
//
// ## Why nothing caught it — two instruments, both blind, for different reasons
//
// **`scripts/sweep.d/10-int-chains.py` enumerates three leaves.** `l1 o1 l2 o2
// l3` has exactly ONE intermediate, and with one intermediate every candidate
// rule puts it in r11. The 11,664-case four-leaf enumeration `il_accum4.cpp`
// records was a one-off; nothing carried the axis into the standing sweep.
// `scripts/sweep.d/12-alloc-depth.py` is that axis made standing — 72 cases,
// and it reports **1 mismatch against master and 0 here**, which is the check
// that it is not vacuous.
//
// **And no fixture had the shape**, which is what this file fixes: the sweep is
// not part of `scripts/gate.sh`, so a fragment alone would leave the regression
// outside the merge gate.
//
// The first revision of that fragment emitted `a o1 b o2 c o3 d` **unbracketed**
// and reported `mismatches=0` against the very master binary whose mis-emit it
// was written to catch — C++ precedence reassociates `a + b * c * d` into
// `a + ((b*c)*d)`, a depth-3 tree the parser refuses outright. A grid that
// cannot contain its own counterexample, in this lane's own work, caught by
// requiring the fragment to reproduce the known failure before committing it.
//
// ## What ships is a REFUSAL, and deliberately not a fix
//
// All 23 measured cells (`work/w-build/probe/alloc-Ox.cod`, `bits3-Ox.cod`, and
// the four `il_accum4.cpp` records) fit one rule: **an intermediate goes to r11
// when its CONSUMER is an `add`, and takes the next descending scratch
// otherwise.** It reproduces both `chain_has_add` misses and every case
// `chain_has_add` gets right — including `(a & b) - c - d + e`, whose allocation
// is r11, r10, r11, which no whole-chain rule can produce at all.
//
// Twenty-three witnesses is not eleven thousand, and replacing a rule that
// survived an enumeration with one that has not faced it is how the per-chain
// accumulator bug got in the first time — two rules that coincide on short
// inputs. So the divergent region refuses under `expr-alloc-undetermined`, the
// hypothesis is written down at `intermediate_alloc_determined`, and the
// fragment that would validate it ships alongside. **Fixing it is the next
// lane's rung, and it now has a failing test to fix against.**
//
// ## The boundary
//
// The two rules classify intermediate `k` by different things — the port by
// "does the chain contain an add", c2 by "is `op[k+1]` an add" — so they can
// only disagree where an add's result feeds a **non**-add, and only with three
// or more operators (with one intermediate the descending sequence *starts* at
// r11, so both rules agree). Exactly 1 of the fragment's 72 cases is in that
// region, which is why the guard costs almost nothing.
//
// Both rows below must return `NotImplemented`. Their in-class neighbours — the
// ones this guard must NOT take — are in `w71_alloc_undetermined.cpp`.
int n_add_mul_mul (int a, int b, int c, int d)        { return ((a + b) * c) * d; }
int n_add_mul_mul5(int a, int b, int c, int d, int e) { return (((a + b) * c) * d) * e; }
