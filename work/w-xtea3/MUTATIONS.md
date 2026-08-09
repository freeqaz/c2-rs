# w-xtea3 — must-fail mutations, 5 of 5

A fence nobody has broken on purpose is a fence nobody has graded. Each row
deletes exactly one clause of one recognizer, rebuilds, and grades the ONE
`_neg` fixture that clause is supposed to hold out. `work/w-xtea3/mutate.sh`
applies and restores the edit; every restore is `git checkout -- <path>`, which
is why §3 below exists.

Shipping verdict for every `_neg` cell is **`vocab-gap`** at `/O1`, `/O1 /Oi`
and `/Ox`.

| # | clause deleted | `_neg` cell | mutated verdict |
|---|---|---|---|
| **M1** | `nonce_add_run`'s **run length**: admit a one-statement run and emit the two-statement plan anyway | `wxtea3_nonce1_neg.cpp` | **`mismatch` 1** |
| **M2** | `nonce_add_run::eat_addend`'s **merged** clause: the addend is a 4-byte value widened to eight | `wxtea3_nonce_u64_neg.cpp` | **`mismatch` 1** |
| **M3** | `xtea_round_loop`'s **`sum += <delta>`** statement, made optional | `wxtea3_nosum_neg.cpp` | **`mismatch` 1** |
| **M4** | `xtea_round_loop::SHR_K` — accept any right shift and emit the measured one | `wxtea3_shift6_neg.cpp` | **`mismatch` 1** |
| **M5** | `xtea_encrypt_loop`'s **`mNonce[i] += 1`** statement, made optional | `wxtea3_nobump_neg.cpp` | **`mismatch` 1** |

**5 of 5**, every one a live wrong-bytes emit against real `c2.dll` — which is
the only grading this project accepts.

---

## 1. M2 took two attempts, and the first failure is the finding

The cell was `vocab-gap` under the obvious mutation (delete the `2C` widening
check), because `eat_addend` **separately** required the addend's type to be a
4-byte GPR value. Two clauses, one fact, and the cell graded **neither**:
deleting either left the other refusing it.

That is `w-xtea2` **#2665**'s shape, and the repair it prescribes is **merging
rather than adding cells**. The two halves are now one clause with one refusal
key — `nonce-addend-is-not-a-4-byte-value-widened-to-eight` — and M2 deletes the
whole conjunction.

**The second attempt failed too**, for the same reason one level in: a mutation
that set only `four = true` still left `widened` refusing. **A merged clause's
must-fail mutation must delete the merged clause**, not one of its terms, and
that is now what M2 does.

## 2. What is NOT graded here, said out loud

* The **inline fence's loop clause** (`comdat::INLINE_DECLINE_LOOP_BYTES`) is a
  *widening*, so breaking it produces a REFUSAL and not a `mismatch`: reverting
  it turns `wxtea3_encrypt_loop.cpp` and the workload TU from `match` into
  `codegen-gap`. That is a must-**refuse** row, verified, and it is recorded as
  a different kind of evidence rather than counted among the five.
* The three **mode gates** are graded by the whole-corpus scan rather than here:
  all eight new fixtures are `vocab-gap` at `/Ox` under the tip binary and
  `mismatch` 0 at every one of the 18 gate lanes.

## 3. THE HARNESS DISCARDED THE FIX IT WAS WRITTEN TO GRADE

`mutate.sh` restores with `git checkout -- <path>` in an EXIT trap. Its first
run after the M2 merge was written — and before it was committed — silently
reverted the merge to the committed version. Nothing said so: the mutation
printed a verdict, `git status` was clean, and the only evidence was the clause
being back in its split form.

This is board **#2668**'s shape (`hatch_red.py` discarding uncommitted `crates/`
edits while printing *"final crates/ diff: EMPTY"*) arriving through a lane's
**own instrument** rather than through a gate row, and the same rule closes it:
**commit before running anything that restores.**
