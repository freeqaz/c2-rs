# W-BLOCKIR — the `_neg` cells' clause keys, probe-verified

A `_neg` file whose cells all trip the same clause tests one thing eleven times.
This is the per-cell key, read out of the production's **own** decline path by a
32-line scratch print at its dispatch site (`work/w-blockir/scratch.patch`,
reverted before the gate — `git diff master -- crates/c2-il/src/func/body/mod.rs`
carries no scratch at this lane's tip).

The census cannot show this: the production is **non-committal**, so a body it
declines still reports the *arm's* blocker (`expr-cmp-eq` on eleven of the
twelve), which is exactly the property that makes the fence order neutral and
exactly why the keys had to be read another way.

| # | cell | clause that fires | what c2 emits instead |
|---:|---|---|---|
| 1 | `n_sub` | `fwalk-compound-op` | swaps the two loads; `fsubs`, 48 B (`c7`) |
| 2 | `n_div` | `fwalk-compound-op` | the same swap with `fdivs`, 48 B (`c8`) |
| 3 | `n_signed` | **`fwalk-guard-type`** | `cmpwi cr6,r3,0` + `bclr 4,25`, 48 B (`c9`) |
| 4 | `n_double` | **`fwalk-body-dst`** | `lfdx`/`lfd`/`fadd`/`stfd`, stride 8, 48 B (`c11`) |
| 5 | `n_int` | **`fwalk-body-rhs1-type`** | `lwzx`/`lwz`/`add`/`stw`, 48 B (`c14`) |
| 6 | `n_step2` | **`fwalk-incr-lit-type`** | `addi -1`/`srwi 1`/`addi +1` trip count, 60 B (`e3`) |
| 7 | `n_ctru` | **`fwalk-then-return`** | a second live value, interleaved schedule, 68 B (`e1`) |
| 8 | `n_after` | **`fwalk-for-scope-close`** | continues past the `bdnz` into a REFHI/REFLO pair, 60 B (`e2`) |
| 9 | `n_bound` | **`fwalk-bound-not-formal0`** | **two** guards, 56 B (`e4`) |
| 10 | `n_two` | **`fwalk-compound-arity`** | a second `stfsx` inside the loop, `bdnz .-24`, 56 B (`e5`) |
| 11 | `n_desc` | **`fwalk-binary-operands-descending`** | **byte-identical to `IPP::Mul`**, 52 B (`c1`) |
| 12 | `n_noguard` | **none — never reaches this reader** | **byte-identical to `Add_InPlace`**, 48 B (`c10`) |

**Eleven cells reach the production and trip TEN distinct clauses**; `n_sub` and
`n_div` share one by design (the same clause, a different opcode byte, and the
comment in the fixture says so).

## Two cells whose key is not the one they were designed for, recorded rather than adjusted

* **`n_step2`** was designed for `fwalk-step-not-1` and fires
  `fwalk-incr-lit-type`. `i += 2` is the `0F` compound spelling where `i++` is
  `35`, so the stream diverges one token *earlier* than the step literal. The
  clause it was designed for is therefore **not exercised by this file**, and the
  fixture comment says so instead of claiming it.
* **`n_ctru`** was designed for `fwalk-body-end` and fires `fwalk-then-return`,
  because its guard is `return 0;` — a value return — so the then-clause diverges
  before the loop body is reached.

Both still grade what they claim to grade: c2's bodies for them are 60 B and
68 B and nothing in this class could emit either. What is corrected is the
*attribution*, not the cell.

## The twelfth is a dispatch fact, not a clause

`n_noguard` has no `if (n == 0) return;`, so its body's first statement is
`i = 0` and the segment dispatches on `26` into the **other** arm of the ladder.
This production is never asked. That is recorded because a `_neg` file which
silently counted it would be claiming a clause it does not exercise — and because
the cell itself is the sharpest thing in the file: **c2 emits exactly the same 48
bytes with the guard and without it**, since the `for` rotation needs the
zero-trip test anyway. The guard is redundant in the obj and load-bearing in the
IL.
